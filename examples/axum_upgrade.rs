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
                    Message::Binary(bin) => {
                        let text = String::from_utf8(bin.to_vec()).unwrap();
                        println!("BinaryMessage: {:?}", text);

                        socket
                            .send(Message::Binary(format!("recv(server) {}", text).into_bytes()))
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
