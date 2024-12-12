use futures_util::{SinkExt, StreamExt};
use log::info;
use std::collections::HashMap;
use std::net::SocketAddrV4;
use std::sync::Arc;
// use std::sync::mpsc::Sender;
use anyhow::Result;
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr: SocketAddrV4 = "127.0.0.1:8080".parse()?;

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

    let manager_clone = manager.clone();

    // For each incoming message, log the content to the standard output
    tokio::spawn(async move {
        println!("ws receive thread start.");
        while let Some(Ok(msg)) = read.next().await {
            if msg.is_text() || msg.is_binary() {
                let message_string = msg.to_string(); // 一時オブジェクトを変数で保持
                let msg_ = message_string.trim(); // trim() を呼び出して安全に参照
                println!("client {uuid} says... \"{msg_}\"");
                tx.send(message_string).await.unwrap();
            }
        }
        // ここに削除処理を追加（読み取りタスク終了時）
        let mut manager = manager_clone.lock().await;
        manager.remove(uuid).await; // 該当するUUIDを削除

        println!("ws receive thread end.");
    });

    let _ = tokio::spawn(async move {
        println!("echo thread start.");
        while let Some(m) = rx.recv().await {
            if let Err(e) = write.send(m.into()).await {
                eprintln!("Error sending to WebSocket: {}", e);
                break;
            }
        }
        println!("echo thread end.")
    });
}
