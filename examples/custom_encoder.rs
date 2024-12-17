use message_pack::{TextMessage, MessageType};
use std::fs::File;
use std::io::Write;

fn main() -> anyhow::Result<()> {
    let message = TextMessage {
        sender: "Alice".to_string(),
        room: 42,
        category: MessageType::Chat,
        content: "Hello, world!".to_string(),
    };

    let bytes = message.to_bytes();

    println!("{:?}", bytes);
    write_bytes_to_file(&bytes)?;

    let decoded: TextMessage = TextMessage::from_bytes(&bytes).unwrap();
    println!("{:?}", decoded);

    Ok(())
}

fn write_bytes_to_file(bytes: &[u8]) -> anyhow::Result<()> {
    // ファイルを作成
    let mut file = File::create("./alice1.bin")?;
    // バイナリデータを書き込み
    file.write_all(bytes)?;
    // ファイルをを明示的にフラッシュ
    file.flush()?;
    Ok(())
}
