use futures_util::{future, pin_mut, SinkExt, StreamExt};
use rnglib::{Language, RNG};
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    // let url = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));

    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (mut write, read) = ws_stream.split();

    {
        let name = {
            let rng = RNG::try_from(&Language::Fantasy).unwrap();

            let first_name = rng.generate_name();
            let last_name = rng.generate_name();
            format!("{first_name} {last_name}")
        };

        // `I am {name}` メッセージをWebSocketに送信
        let intro_message = format!("I am {name}");
        write
            .send(Message::Text(intro_message))
            .await
            .expect("メッセージ送信に失敗しました");
        println!("メッセージ `I am {}` が送信されました", name);
    }

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();

            // データの出力
            let mut stdout = tokio::io::stdout(); // mutable な stdout ハンドルの作成
            stdout.write_all(&data).await.unwrap();
            stdout.flush().await.unwrap(); // フラッシュを明示的に実行
            println!()
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}
