use axum::extract::ws::{Message, WebSocket};
use axum::extract::State;
use axum::extract::WebSocketUpgrade;
use axum::http::Method;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use axum::Json;
use clap::Parser;
use futures_util::stream::{self, Stream};
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use message_pack::{
    get_type, BinaryDeserializable, FileTransferMessage, ListMessage, MessageType, TextMessage,
};
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fs::exists;
use std::net::SocketAddrV4;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt as _;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ホスト名 (環境変数から取得またはデフォルト値を適用)
    #[arg(long, default_value_t = String::new())]
    hostname: String,
}

struct SocketWrapper {
    id: Uuid,
    socket: Sender<String>,
}

struct SocketManager {
    sockets: Arc<Mutex<HashMap<Uuid, SocketWrapper>>>,
}

impl SocketManager {
    fn new() -> Self {
        Self {
            sockets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn add(&mut self, socket: Sender<String>) -> Uuid {
        let id = Uuid::new_v4();
        let socket = SocketWrapper { id, socket };
        let mut sockets = self.sockets.lock().await;
        sockets.insert(id, socket);

        id
    }

    async fn remove(&mut self, id: Uuid) {
        let mut sockets = self.sockets.lock().await;

        if sockets.remove(&id).is_some() {
            info!("Socket with ID {} removed", id);
        }
    }

    async fn broadcast(&self, message: String) {
        let sockets = self.sockets.lock().await;
        for (_, socket_wrapper) in sockets.iter() {
            if let Err(err) = socket_wrapper.socket.send(message.clone()).await {
                warn!("Failed to send message to {}: {}", socket_wrapper.id, err);
            }
        }
    }

    async fn direct_message(&self, id: Uuid, message: String) {
        let sockets = self.sockets.lock().await; // ロックガードを束縛
        let target_socket = sockets.get(&id).unwrap(); // ロックガードからデータを取得

        if let Err(err) = target_socket.socket.send(message.clone()).await {
            warn!("Failed to send message to {}: {}", id, err);
        }
    }

    #[allow(dead_code)]
    async fn dump(&self) {
        let sockets = self.sockets.lock().await; // 非同期ロックを取得
        info!("Current sockets:");
        for (id, _sender) in sockets.iter() {
            info!("\t{}", id);
        }
    }
}

const UPLOAD_DIRNAME: &str = "./uploads";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // 環境変数 "HOSTNAME" の取得
    let env_hostname = env::var("HOSTNAME").ok();
    // コマンドライン引数または環境変数、最後にデフォルト値を設定
    let hostname = if !args.hostname.is_empty() {
        args.hostname
    } else if let Some(env) = env_hostname {
        env
    } else {
        String::from("127.0.0.1:8080") // デフォルト値
    };

    info!("hostname: {}", hostname);

    SimpleLogger::new()
        .with_level(log::LevelFilter::Info) // デフォルトログレベル (Info以上のみ出力)
        .init()?;

    let addr: SocketAddrV4 = hostname.parse()?;

    match exists(UPLOAD_DIRNAME) {
        Ok(true) => match fs::metadata(UPLOAD_DIRNAME) {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    warn!("Error: {} is not a directory", UPLOAD_DIRNAME);
                    std::process::exit(1);
                } else {
                    // do nothing
                }
            }
            Err(_) => {
                warn!("Error: cannot check is {} a directory", UPLOAD_DIRNAME);
                std::process::exit(1);
            }
        },
        Ok(false) => {
            if let Err(_) = fs::create_dir(UPLOAD_DIRNAME) {
                warn!("Error: creating directory {}", UPLOAD_DIRNAME);
                std::process::exit(1);
            } else {
                warn!("Directory {} is created", UPLOAD_DIRNAME);
            }
        }
        Err(_) => match fs::create_dir(UPLOAD_DIRNAME) {
            Err(e) => {
                warn!("Error: {}", e);
                std::process::exit(1);
            }
            _ => {}
        },
    }

    let socket: std::io::Result<TcpListener> = TcpListener::bind(&addr).await;
    let listener: TcpListener = socket.expect("Failed to bind socket");

    let socket_manager = Arc::new(Mutex::new(SocketManager::new()));

    let sse_sent = Arc::new(Mutex::new(0));

    let cors = cors_handler().await;

    let app = axum::Router::new()
        .nest_service("/", ServeDir::new("./front/dist"))
        .route(
            "/api/health.json",
            axum::routing::get(|| async { Json("{\"success\": \"true\"}") }),
        )
        .route(
            "/api/sse",
            get({
                let sse_sent = sse_sent.clone();
                move || sse_handler(sse_sent)
            }),
        )
        .route("/ws", axum::routing::get(handle_websocket))
        .layer(cors)
        .with_state(socket_manager)
        .with_state(sse_sent);

    let _ = axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn sse_handler(sent: Arc<Mutex<i32>>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::unfold(sent.clone(), |sent_ref| async move {
        let current_count;

        {
            let mut count = sent_ref.lock().await; // ロックをスコープ内で限定
            *count += 1;
            current_count = *count; // カウントの値をコピー
        }

        // ロックが解放された後にイベントを生成

        if current_count > 100 {
            {
                let mut count = sent_ref.lock().await; // ロックをスコープ内で限定
                *count = 0;
                info!("reset sent count");
            }
            None
        } else {
            let event = Event::default().data(format!("hi! ({})", current_count));
            Some((Ok(event), sent_ref)) // `sent_ref` を次に渡す（move 必要なし）
        }
    })
    .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn handle_websocket(
    State(manager): State<Arc<Mutex<SocketManager>>>,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_socket(manager, socket))
}

async fn handle_socket(manager: Arc<Mutex<SocketManager>>, mut socket: WebSocket) {
    if let Err(e) = socket
        .send(Message::from("connected(server)".to_string()))
        .await
    {
        warn!("Error while sending connected message: {:?}", e);
        return;
    }

    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // クライアントを管理に追加
    let uuid = {
        let mut manager = manager.lock().await;
        manager.add(tx.clone()).await
    };

    // クライアントへの送信タスク
    let manager_clone = manager.clone();
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender.send(Message::Text(message)).await.is_err() {
                warn!("Error sending message to client");
                break;
            }
        }

        // クライアント切断時に管理から削除
        manager_clone.lock().await.remove(uuid).await;
    });

    // クライアントから受信タスク
    let manager_clone = manager.clone();
    let manager_clone2 = manager.clone();
    let uuid_clone = uuid;
    tokio::spawn(async move {
        while let Some(Ok(msg)) = futures_util::StreamExt::next(&mut receiver).await {
            match msg {
                Message::Text(text) => {
                    // // メッセージを全クライアントに送信 (ブロードキャスト)
                    let message_string = text.trim().to_string(); // 安全に加工
                    info!("received: {}", message_string);

                    // 受け取ったメッセージを全クライアントにブロードキャスト
                    let manager = manager_clone.lock().await; // ロックを取得
                    manager.broadcast(message_string).await;
                }
                Message::Binary(m) => {
                    // let m: Vec<u8> = msg.into_data();

                    match get_type(&m[0]) {
                        MessageType::Chat => {
                            // chat
                            let chat_message = TextMessage::from_bytes(&*m).unwrap();
                            // チャットメッセージを何らかの形で文字列に変換してブロードキャスト
                            let message_string = format!(
                                "[Room {} - {}]: {}",
                                chat_message.room, chat_message.sender, chat_message.content
                            );

                            // クライアントにブロードキャスト
                            let manager = manager_clone.lock().await; // ロックを取得
                            manager.broadcast(message_string.clone()).await;
                        }
                        MessageType::Exit => {
                            // exit
                            // let d = TextMessage::from_bytes(&*m).unwrap();
                            info!("received exit message");

                            // ロックを使って離脱メッセージをブロードキャスト
                            let leave_message = format!("User {} has left the chat.", uuid);
                            {
                                let manager = manager_clone.lock().await;
                                manager.broadcast(leave_message).await;
                            } // ロックを解除

                            // UUIDの削除
                            {
                                let mut manager = manager_clone.lock().await;
                                manager.remove(uuid).await;
                            } // ロックを解除

                            // スレッド終了
                            break;
                        }
                        MessageType::FileTransfer => {
                            // file transfer
                            let d: FileTransferMessage =
                                FileTransferMessage::from_bytes(&*m).unwrap();

                            match d.category {
                                MessageType::FileTransfer => {
                                    if d.filename.is_empty() {
                                        warn!("Invalid filename received");
                                        return;
                                    }

                                    let default_path = UPLOAD_DIRNAME.to_string();
                                    let full_path = format!("{}/{}", default_path, d.filename);

                                    let transferred_bytes = format_bytes(d.content.len() as u64);
                                    info!(
                                        "uploaded: {} {} bytes transferred.",
                                        full_path,
                                        transferred_bytes.clone()
                                    );
                                    let mut f = File::create(full_path).await.unwrap();

                                    f.write_all(&d.content).await.unwrap();

                                    {
                                        let manager = manager_clone.lock().await;
                                        manager
                                            .direct_message(
                                                uuid,
                                                format!("{} bytes transferred.", transferred_bytes),
                                            )
                                            .await;
                                    } // ロックを解除
                                }
                                _ => {
                                    warn!("Invalid message category (3)");
                                }
                            }
                        }
                        MessageType::List => {
                            let d: ListMessage = ListMessage::from_bytes(&*m).unwrap();

                            match d.target.as_str() {
                                "socket" => {
                                    let manager = manager_clone.lock().await;
                                    let mut messages: Vec<String> = Vec::new();
                                    for (id, _socket_wrapper) in manager.sockets.lock().await.iter()
                                    {
                                        messages.push(format!("{}", id));
                                    }
                                    manager.direct_message(uuid, messages.join("\n")).await;
                                }
                                _ => {
                                    let manager = manager_clone.lock().await;
                                    let message = "Invalid target";
                                    manager.direct_message(uuid, message.to_string()).await;
                                }
                            }
                        }
                        _ => {
                            warn!("Invalid message category (4)");
                        }
                    }
                }
                _ => {
                    warn!("Received unknown message {:?}", msg);
                }
            }
        }

        // クライアント切断時に管理から削除
        manager_clone2.lock().await.remove(uuid_clone).await;
    });
}

fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    const TIB: u64 = GIB * 1024;

    if bytes >= TIB {
        format!("{:.2} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.2} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.2} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub async fn cors_handler() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        // .allow_credentials(true)
        .max_age(Duration::from_secs(86400)) // 1日間のプリフライトキャッシュ
}
