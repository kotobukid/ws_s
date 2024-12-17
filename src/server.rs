use futures_util::{SinkExt, StreamExt};
use log::info;
use message_pack::{BinaryMessage, MessageType, TextMessage, BinaryDeserializable};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddrV4;
use std::sync::Arc;
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

                match &m[0] {
                    0x01 => {
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
                    0x02 => {
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
                    0x03 => {
                        // file transfer
                        let d = BinaryMessage::from_bytes(&*m).unwrap();
                        match d.category {
                            MessageType::FileTransfer => {
                                println!("received (file): {:?}", d);
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
