use futures_util::{future, pin_mut, SinkExt, StreamExt};
use message_pack::{
    BinarySerializable, ExitMessage, FileTransferMessage, ListMessage, MessageType, TextMessage,
    UnifiedMessage,
};
use rfd::AsyncFileDialog;
use rnglib::{Language, RNG};
use std::env;
use clap::Parser;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use ws_s::utils::{
    parse_arguments, replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ホスト名 (環境変数から取得またはデフォルト値を適用)
    #[arg(long, default_value_t = String::new())]
    hostname: String,
}


#[tokio::main]
async fn main() {
    let args = Args::parse();

    // サーバーと同様
    let env_hostname = env::var("HOSTNAME").ok();
    let hostname = if !args.hostname.is_empty() {
        args.hostname
    } else if let Some(env) = env_hostname {
        env
    } else {
        String::from("127.0.0.1:8080") // デフォルト値
    };

    println!("hostname: {}", hostname);

    let url = format!("ws://{}/ws", hostname);

    let name = {
        let rng = RNG::try_from(&Language::Fantasy).unwrap();

        let first_name = rng.generate_name();
        let last_name = rng.generate_name();
        format!("{first_name} {last_name}")
    };

    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(name.to_string(), stdin_tx));

    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (mut write, read) = ws_stream.split();

    {
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
            let data = match message {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("{:?}", e);
                    std::process::exit(1)
                }
            };

            let data = data.into_data();
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

async fn read_stdin(name: String, tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);

        // 標準入力から読み取ったデータを文字列として解釈
        let input = match String::from_utf8(buf) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Invalid UTF-8 in stdin input");
                continue;
            }
        };

        let tokens: Result<Vec<String>, String> = parse_arguments(
            replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(&input).as_str(),
        );

        match tokens {
            Ok(tokens) => {
                if tokens.len() > 0 {
                    let command = tokens[0].as_str();
                    let args = &tokens[1..];

                    let chat_message: Option<UnifiedMessage> = match command {
                        "/exit" => Some(UnifiedMessage::Exit(ExitMessage {})),
                        "/file" => {
                            let file = AsyncFileDialog::new()
                                .add_filter("text", &["txt", "rs"])
                                .add_filter("rust", &["rs", "toml"])
                                .add_filter("any file", &["*"])
                                .set_directory("/")
                                .pick_file()
                                .await;

                            if let Some(file) = file {
                                let bytes = file.read().await;
                                println!("filename: {:?}", file.file_name());

                                Some(UnifiedMessage::FileTransferMessage(FileTransferMessage {
                                    category: MessageType::FileTransfer,
                                    room: 42,
                                    filename: file.file_name(),
                                    sender: name.clone(),
                                    content: bytes,
                                }))
                            } else {
                                None
                            }
                        }
                        "/list" => {
                            let target = if args.len() > 0 {
                                args[0].as_str()
                            } else {
                                "socket"
                            };

                            Some(UnifiedMessage::ListMessage(ListMessage {
                                category: MessageType::List,
                                room: 42,
                                target: target.to_string(),
                                sender: name.clone(),
                            }))
                        }
                        _ => Some(UnifiedMessage::ChatMessage(TextMessage {
                            sender: name.clone(),
                            room: 42, // 仮のルーム番号
                            category: MessageType::Chat,
                            content: input.trim().to_string(), // 標準入力からのメッセージ
                        })),
                    };

                    if let Some(chat_message) = chat_message {
                        // ChatMessage をバイナリ形式にエンコード
                        let binary_data = chat_message.to_bytes();

                        // バイナリデータを WebSocket メッセージとして送信
                        tx.unbounded_send(Message::binary(binary_data)).unwrap();
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
