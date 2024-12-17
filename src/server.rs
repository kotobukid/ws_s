use futures_util::{SinkExt, StreamExt};
use log::info;
use message_pack::{get_type, BinaryDeserializable, FileTransferMessage, MessageType, TextMessage};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddrV4;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
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
            println!("Socket with ID {} removed", id);
        }
    }

    async fn broadcast(&self, message: String) {
        let sockets = self.sockets.lock().await;
        for (_, socket_wrapper) in sockets.iter() {
            if let Err(err) = socket_wrapper.socket.send(message.clone()).await {
                eprintln!("Failed to send message to {}: {}", socket_wrapper.id, err);
            }
        }
    }

    async fn direct_message(&self, id: Uuid, message: String) {
        let sockets = self.sockets.lock().await; // ロックガードを束縛
        let target_socket = sockets.get(&id).unwrap(); // ロックガードからデータを取得

        if let Err(err) = target_socket.socket.send(message.clone()).await {
            eprintln!("Failed to send message to {}: {}", id, err);
        }
    }

    async fn dump(&self) {
        let sockets = self.sockets.lock().await; // 非同期ロックを取得
        println!("Current sockets:");
        for (id, _sender) in sockets.iter() {
            println!("\t{}", id);
        }
        println!();
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr: SocketAddrV4 = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".parse().unwrap())
        .parse()?;
    // let addr: SocketAddrV4 = "192.168.1.187:8080".parse()?;

    let socket: std::io::Result<TcpListener> = TcpListener::bind(&addr).await;
    let listener: TcpListener = socket.expect("Failed to bind socket");

    let socket_manager = Arc::new(Mutex::new(SocketManager::new()));

    println!("Listening on: {}", addr);
    while let Ok((stream, _)) = listener.accept().await {
        let socket_manager = socket_manager.clone();

        tokio::spawn(async move {
            accept_connection(socket_manager, stream).await;
        });
    }

    Ok(())
}

async fn accept_connection(manager: Arc<Mutex<SocketManager>>, stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::channel(1000);

    let uuid = {
        let mut manager = manager.lock().await;
        manager.add(tx.clone()).await
    };

    let manager_clone_1 = manager.clone();
    let manager_clone_2 = manager.clone();
    let manager_clone_3 = manager.clone();

    // For each incoming message, log the content to the standard output
    tokio::spawn(async move {
        println!("ws receive thread start.");
        while let Some(Ok(msg)) = read.next().await {
            if msg.is_text() {
                let message_string = msg.to_string().trim().to_string(); // 安全に加工
                println!("received: {}", message_string);

                // 受け取ったメッセージを全クライアントにブロードキャスト
                let manager = manager_clone_1.lock().await; // ロックを取得
                manager.broadcast(message_string).await;
            } else if msg.is_binary() {
                let m: Vec<u8> = msg.into_data();

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
                        let manager = manager_clone_1.lock().await; // ロックを取得
                        manager.broadcast(message_string.clone()).await;
                    }
                    MessageType::Exit => {
                        // exit
                        // let d = TextMessage::from_bytes(&*m).unwrap();
                        println!("received exit message");

                        // ロックを使って離脱メッセージをブロードキャスト
                        let leave_message = format!("User {} has left the chat.", uuid);
                        {
                            let manager = manager_clone_1.lock().await;
                            manager.broadcast(leave_message).await;
                        } // ロックを解除

                        // UUIDの削除
                        {
                            let mut manager = manager_clone_3.lock().await;
                            manager.remove(uuid).await;
                        } // ロックを解除

                        // スレッド終了
                        break;
                    }
                    MessageType::FileTransfer => {
                        // file transfer
                        let d: FileTransferMessage = FileTransferMessage::from_bytes(&*m).unwrap();

                        match d.category {
                            MessageType::FileTransfer => {
                                if d.filename.is_empty() {
                                    eprintln!("Invalid filename received");
                                    return;
                                }

                                let default_path = "./uploads"; // todo: あらかじめ作成しておかねばならないのを回避する
                                let full_path = format!("{}/{}", default_path, d.filename);

                                let transferred_bytes = format_bytes(d.content.len() as u64);
                                println!(
                                    "uploaded: {} {} bytes transferred.",
                                    full_path,
                                    transferred_bytes.clone()
                                );
                                let mut f = File::create(full_path).await.unwrap();

                                f.write_all(&d.content).await.unwrap();

                                {
                                    let manager = manager_clone_3.lock().await;
                                    manager
                                        .direct_message(
                                            uuid,
                                            format!("{} bytes transferred.", transferred_bytes),
                                        )
                                        .await;
                                } // ロックを解除
                            }
                            _ => {
                                eprintln!("Invalid message category");
                            }
                        }
                    }
                    _ => {
                        eprintln!("Invalid message category");
                    }
                }
            }
        }
        // 削除処理
        let mut manager = manager_clone_2.lock().await;
        manager.remove(uuid).await; // 該当するUUIDを削除

        println!("ws receive thread end.");
    });

    tokio::spawn(async move {
        println!("echo thread start.");
        while let Some(m) = rx.recv().await {
            if let Err(e) = write.send(m.into()).await {
                eprintln!("Error sending to WebSocket: {}", e);
                break;
            }
        }
        println!("echo thread end.")
    });

    manager.lock().await.dump().await;
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
