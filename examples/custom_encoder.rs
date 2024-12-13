use message_pack::{ChatMessage, MessageCategory};
use std::fs::File;
use std::io::Write;

fn main() -> anyhow::Result<()> {
    let message = ChatMessage {
        author: "Alice".to_string(),
        room: 42,
        category: MessageCategory::ChatMessage,
        message: "Hello, world!".to_string(),
    };

    let bytes = message.to_bytes();

    println!("{:?}", bytes);
    write_bytes_to_file(&bytes)?;

    let decoded: ChatMessage = ChatMessage::from_bytes(&bytes).unwrap();
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
