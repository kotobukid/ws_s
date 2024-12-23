use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use std::net::SocketAddrV4;

const SCRIPT: &str = r#"
<head>
<meta charset="utf-8">
<title>WebSocket</title>
<script>
window.onload = () => {
    const ws = new WebSocket(`${location.origin}/ws`);

    ws.onmessage = (event) => {
        console.log(event.data);
        document.getElementById('output').innerText = event.data;
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
};
</script>
</head>
<body>
    <form>
        <input type="text" id="input" />
        <button id="send_button">send</button>
        <br />
        <span id="output"></span>
    </form>
</body>
"#;

#[tokio::main]
async fn main() {
    let addr: SocketAddrV4 = "127.0.0.1:3000".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let app = Router::new()
        .route("/", get(|| async { Html(SCRIPT.to_string()) }))
        .route("/ws", get(handle_websocket));

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn handle_websocket(ws: WebSocketUpgrade) -> axum::response::Response {
    println!("handle_websocket {:?}", ws);
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    if let Err(e) = socket
        .send(Message::from("connected(server)".to_string()))
        .await
    {
        eprintln!("Error while sending connected message: {:?}", e);
        return;
    }

    tokio::spawn(async move {
        while let Some(result) = socket.recv().await {
            println!("recv(server) {:?}", result);

            match result {
                Ok(msg) => match msg {
                    Message::Text(text) => {
                        socket
                            .send(Message::from(format!("recv(server) {}", text)))
                            .await
                            .unwrap();

                        if text == String::from("exit") {
                            break;
                        }
                    }
                    _ => {
                        println!("{:?}", msg);
                    }
                },
                Err(e) => {
                    eprintln!("Error while sending message: {:?}", e);
                    break;
                }
            }
        }
    });
}
