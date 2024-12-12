use futures_util::{SinkExt, StreamExt};
use log::info;
use std::net::SocketAddrV4;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr: SocketAddrV4 = "127.0.0.1:8080".parse()?;

    let socket = TcpListener::bind(&addr).await;
    let listener = socket.expect("Failed to bind socket");

    println!("Listening on: {}", addr);
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
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

    // For each incoming message, log the content to the standard output
    tokio::spawn(async move {
        println!("ws receive thread start.");
        while let Some(Ok(msg)) = read.next().await {
            if msg.is_text() || msg.is_binary() {
                let message_string = msg.to_string(); // 一時オブジェクトを変数で保持
                let msg_ = message_string.trim(); // trim() を呼び出して安全に参照
                println!("client says... \"{msg_}\"");
                tx.send(message_string).await.unwrap();
            }
        }
        println!("ws receive thread end.");
    });

    let _ = tokio::spawn(async move {
        println!("echo thread start.");
        while let Some(m) = rx.recv().await {
            write.send(m.into()).await.unwrap();
        }
        println!("echo thread end.")
    });
}
