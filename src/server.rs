use axum::extract::ws::{Message, WebSocket};
use axum::extract::State;
use axum::extract::WebSocketUpgrade;
use axum::response::Html;
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use message_pack::{
    get_type, BinaryDeserializable, FileTransferMessage, ListMessage, MessageType, TextMessage,
};
use std::collections::HashMap;
use std::fs::exists;
use std::net::SocketAddrV4;
use std::sync::Arc;
use std::{env, fs};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener};
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

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

    async fn dump(&self) {
        let sockets = self.sockets.lock().await; // 非同期ロックを取得
        info!("Current sockets:");
        for (id, _sender) in sockets.iter() {
            println!("\t{}", id);
        }
        println!();
    }
}

const UPLOAD_DIRNAME: &str = "./uploads";
const SCRIPT: &str = r#"
<head>
<meta charset="utf-8">
<title>WebSocket</title>
<script>
window.onload = () => {

    const str_to_binary = (() => {
        const encoder = new TextEncoder();
        const encode = encoder.encode;
        return (str) => encoder.encode(str);
    })();

    const binary_to_str = (() => {
        const decoder = new TextDecoder();
        const decode = decoder.decode;
        return (bin) => decoder.decode(bin);
    })();

    const ws = new WebSocket(`${location.origin}/ws`);

    ws.onmessage = (event) => {

        // データの型を確認
        if (event.data instanceof Blob) {
            // バイナリデータの場合
            event.data.arrayBuffer().then(buffer => {
                const uint8Array = new Uint8Array(buffer);

                // ここでバイナリデータを処理
                // 例：最初の1バイトを見て処理を分岐
                // const firstByte = uint8Array[0];

                // 残りのデータをテキストとして処理する例
                // const decoder = new TextDecoder();
                // const text = decoder.decode(uint8Array.slice(1));

                const text = binary_to_str(uint8Array);

                document.getElementById('output').innerText = `binary(${text})`;
            });
        } else {
            // テキストデータの場合
            console.log(event.data);
            document.getElementById('output').innerText = `text(${event.data})`;
        }
    };

    ws.addEventListener('close', () => {
        console.log('closed(client)');
    });

    ws.addEventListener('open', () => {
        console.log('connected(client)');
        ws.send("hello");
    });

    document.getElementById('send_button').onclick = (e) => {
        e.preventDefault();e.preventDefault();
        let value = document.getElementById('input').value.trim();
        if (value) {
            ws.send(value);
            document.getElementById('input').value = '';
        }
    };

    document.getElementById('send_button2').onclick = (e) => {
        e.preventDefault();e.preventDefault();
        let value = document.getElementById('input').value.trim();
        if (value) {
            let v = str_to_binary(value);
            ws.send(v);
            document.getElementById('input').value = '';
        }
    }
};
</script>
</head>
<body>
    <form>
        <input type="text" id="input" />
        <button id="send_button">send</button>
        <button id="send_button2">send as binary</button>
        <br />
        <span id="output"></span>
    </form>
</body>
"#;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr: SocketAddrV4 = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".parse().unwrap())
        .parse()?;

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

    let app = axum::Router::new()
        .route(
            "/",
            axum::routing::get(|| async { Html(SCRIPT.to_string()) }),
        )
        .route("/ws", axum::routing::get(handle_websocket))
        .with_state(socket_manager);

    let _ = axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn handle_websocket(
    State(manager): State<Arc<Mutex<SocketManager>>>,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    info!("handle_websocket {:?}", ws);
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
        while let Some(Ok(msg)) = receiver.next().await {
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
