use std::fmt::Debug;
use std::io::{Cursor, Read};

#[derive(Eq)]
pub enum MessageType {
    Chat,
    Exit,
    FileTransfer,
    Unknown,
}

impl MessageType {
    pub fn to_bytes(&self) -> u8 {
        match self {
            MessageType::Chat => 0x01,
            MessageType::Exit => 0x02,
            MessageType::FileTransfer => 0x03,
            MessageType::Unknown => 0x00,
        }
    }

    pub fn from_bytes(data: &u8) -> Result<Self, String> {
        match data {
            0x01 => Ok(MessageType::Chat),
            0x02 => Ok(MessageType::Exit),
            0x03 => Ok(MessageType::FileTransfer),
            0x00 => Ok(MessageType::Unknown),
            _ => Err("Invalid message category".to_string()),
        }
    }
}

impl Debug for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Chat => write!(f, "ChatMessage"),
            MessageType::Exit => write!(f, "Exit"),
            MessageType::FileTransfer => write!(f, "FileTransfer"),
            MessageType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl PartialEq<Self> for MessageType {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

pub trait BinarySerializable {
    fn to_bytes(&self) -> Vec<u8>;
}

pub trait BinaryDeserializable {
    fn from_bytes(data: &[u8]) -> Result<Self, String>
    where
        Self: Sized;
}

pub trait SendMessage: BinarySerializable + BinaryDeserializable {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(data: &[u8]) -> Result<Self, String>
    where
        Self: Sized;
}

pub enum UnifiedMessage {
    ChatMessage(TextMessage),
    BinaryMessage(BinaryMessage),
    Exit(ExitMessage),
}

pub fn get_type(b: &u8) -> MessageType {
    match b {
        Some(0x01) => MessageType::Chat,
        Some(0x02) => MessageType::Exit,
        Some(0x03) => MessageType::FileTransfer,
        _ => MessageType::Unknown,
    }
}

impl BinarySerializable for UnifiedMessage {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            UnifiedMessage::ChatMessage(msg) => msg.to_bytes(), // TextMessage の to_bytes を呼び出し
            UnifiedMessage::BinaryMessage(msg) => msg.to_bytes(), // ByteMessage の to_bytes を呼び出し
            UnifiedMessage::Exit(msg) => msg.to_bytes(), // TextMessage の to_bytes を呼び出し
        }
    }
}

impl BinaryDeserializable for UnifiedMessage {
    fn from_bytes(data: &[u8]) -> Result<Self, String>
    where
        Self: Sized,
    {
        // まずカテゴリーを判定して、それに基づいた型のインスタンスを生成。
        if data.is_empty() {
            return Err("Input data is empty.".to_string());
        }
        let category = MessageType::from_bytes(&data[0])?;

        match category {
            MessageType::Chat => {
                let message = TextMessage::from_bytes(data)?;
                Ok(UnifiedMessage::ChatMessage(message))
            }
            MessageType::Exit => {
                let message = ExitMessage {};
                Ok(UnifiedMessage::Exit(message))
            }
            MessageType::FileTransfer => {
                let message = BinaryMessage::from_bytes(data)?;
                Ok(UnifiedMessage::BinaryMessage(message))
            }
            _ => Err("Invalid message category".to_string()),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TextMessage {
    pub sender: String,
    pub room: i32,
    pub category: MessageType,
    pub content: String,
}

impl BinarySerializable for TextMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        // 1バイト: データタイプ (例: 一般チャットメッセージ -> 0x01)
        buffer.push(self.category.to_bytes());

        // 2-5バイト: ルームID (i32をビッグエンディアン形式でエンコード)
        buffer.extend(&self.room.to_be_bytes());

        // 6バイト目: ユーザー名の長さ
        let author_bytes = self.sender.as_bytes();
        buffer.push(author_bytes.len() as u8);

        // 7バイト以降: ユーザー名のバイト列
        buffer.extend(author_bytes);

        // メッセージ本文の長さ (u16、ビッグエンディアン形式)
        let message_bytes = self.content.as_bytes();
        buffer.extend(&(message_bytes.len() as u16).to_be_bytes());

        // 本文のバイト列
        buffer.extend(message_bytes);

        // 最後に簡易的なチェックサム (すべてのバイトを合計して8ビットで切り捨て)
        let checksum: u8 = buffer.iter().fold(0, |acc, &x| acc.wrapping_add(x));
        buffer.push(checksum);

        buffer
    }
}

impl BinaryDeserializable for TextMessage {
    fn from_bytes(data: &[u8]) -> Result<Self, String>
    where
        Self: Sized,
    {
        let mut cursor = Cursor::new(data);

        // カテゴリ (1バイト: u8)
        let mut category_buf = [0u8; 1];
        cursor
            .read_exact(&mut category_buf)
            .map_err(|_| "Failed to read category")?;
        let category = MessageType::from_bytes(&category_buf[0])?;

        // ルームID (4バイト: i32)
        let mut room_buf = [0u8; 4];
        cursor
            .read_exact(&mut room_buf)
            .map_err(|_| "Failed to read room")?;
        let room = i32::from_be_bytes(room_buf);

        // ユーザー名の長さ (1バイト: u8)
        let mut author_len_buf = [0u8; 1];
        cursor
            .read_exact(&mut author_len_buf)
            .map_err(|_| "Failed to read author length")?;
        let author_len = author_len_buf[0] as usize;

        // ユーザー名 (可変長)
        let mut author_buf = vec![0u8; author_len];
        cursor
            .read_exact(&mut author_buf)
            .map_err(|_| "Failed to read author")?;
        let author = String::from_utf8(author_buf).map_err(|_| "Invalid UTF-8 in author")?;

        // メッセージ本文の長さ (2バイト: u16)
        let mut message_len_buf = [0u8; 2];
        cursor
            .read_exact(&mut message_len_buf)
            .map_err(|_| "Failed to read message length")?;
        let message_len = u16::from_be_bytes(message_len_buf) as usize;

        // メッセージ本文 (可変長)
        let mut message_buf = vec![0u8; message_len];
        cursor
            .read_exact(&mut message_buf)
            .map_err(|_| "Failed to read message")?;
        let message = String::from_utf8(message_buf).map_err(|_| "Invalid UTF-8 in message")?;

        // チェックサム (1バイト: 最後に検証可能)
        let mut checksum_buf = [0u8; 1];
        cursor
            .read_exact(&mut checksum_buf)
            .map_err(|_| "Failed to read checksum")?;
        let _checksum = checksum_buf[0];
        // チェックサムの検証ロジックを追加する場合はここで計算

        Ok(Self {
            sender: author,
            room,
            category,
            content: message,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct BinaryMessage {
    pub author: String,
    pub room: i32,
    pub category: MessageType,
    pub content: Vec<u8>,
}

impl BinarySerializable for BinaryMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        // 1バイト: データタイプ (例: 一般チャットメッセージ -> 0x01)
        buffer.push(self.category.to_bytes());

        // 2-5バイト: ルームID (i32をビッグエンディアン形式でエンコード)
        buffer.extend(&self.room.to_be_bytes());

        // 6バイト目: ユーザー名の長さ
        let author_bytes = self.author.as_bytes();
        buffer.push(author_bytes.len() as u8);

        // 7バイト以降: ユーザー名のバイト列
        buffer.extend(author_bytes);

        // メッセージ本文の長さ (u16、ビッグエンディアン形式)
        buffer.extend(&(self.content.len() as u16).to_be_bytes());

        // 本文のバイト列
        buffer.extend(self.content.clone());

        // 最後に簡易的なチェックサム (すべてのバイトを合計して8ビットで切り捨て)
        let checksum: u8 = buffer.iter().fold(0, |acc, &x| acc.wrapping_add(x));
        buffer.push(checksum);

        buffer
    }
}

impl BinaryDeserializable for BinaryMessage {
    fn from_bytes(data: &[u8]) -> Result<Self, String>
    where
        Self: Sized,
    {
        let mut cursor = Cursor::new(data);

        // カテゴリ (1バイト: u8)
        let mut category_buf = [0u8; 1];
        cursor
            .read_exact(&mut category_buf)
            .map_err(|_| "Failed to read category")?;
        let category = MessageType::from_bytes(&category_buf[0])?;

        // ルームID (4バイト: i32)
        let mut room_buf = [0u8; 4];
        cursor
            .read_exact(&mut room_buf)
            .map_err(|_| "Failed to read room")?;
        let room = i32::from_be_bytes(room_buf);

        // ユーザー名の長さ (1バイト: u8)
        let mut author_len_buf = [0u8; 1];
        cursor
            .read_exact(&mut author_len_buf)
            .map_err(|_| "Failed to read author length")?;
        let author_len = author_len_buf[0] as usize;

        // ユーザー名 (可変長)
        let mut author_buf = vec![0u8; author_len];
        cursor
            .read_exact(&mut author_buf)
            .map_err(|_| "Failed to read author")?;
        let author = String::from_utf8(author_buf).map_err(|_| "Invalid UTF-8 in author")?;

        // メッセージ本文の長さ (2バイト: u16)
        let mut message_len_buf = [0u8; 2];
        cursor
            .read_exact(&mut message_len_buf)
            .map_err(|_| "Failed to read message length")?;
        let message_len = u16::from_be_bytes(message_len_buf) as usize;

        // メッセージ本文 (可変長)
        let mut content = vec![0u8; message_len];
        cursor
            .read_exact(&mut content)
            .map_err(|_| "Failed to read message")?;

        // チェックサム (1バイト: 最後に検証可能)
        let mut checksum_buf = [0u8; 1];
        cursor
            .read_exact(&mut checksum_buf)
            .map_err(|_| "Failed to read checksum")?;
        let _checksum = checksum_buf[0];
        // チェックサムの検証ロジックを追加する場合はここで計算

        Ok(Self {
            author,
            room,
            category,
            content,
        })
    }
}

pub struct ExitMessage {}
impl BinarySerializable for ExitMessage {
    fn to_bytes(&self) -> Vec<u8> {
        vec![0x02]
    }
}

// impl BinaryDeserializable for ExitMessage {
//     fn from_bytes(data: &[u8]) -> Result<Self, String>
//     where
//         Self: Sized
//     {
//         if data.len() != 1 {
//             return Err("Invalid data length".to_string());
//         }
//         if data[0] != 0x02 {
//             return Err("Invalid data".to_string());
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let message = TextMessage {
            sender: "Alice".to_string(),
            room: 42,
            category: MessageType::Chat,
            content: "Hello, world!".to_string(),
        };

        let message = TextMessage {
            sender: "Alice".to_string(),
            room: 42,
            category: MessageType::Chat,
            content: "Hello, world!".to_string(),
        };

        let bytes = message.to_bytes();

        let decoded: TextMessage = TextMessage::from_bytes(&bytes).unwrap();

        assert_eq!(message, decoded);
    }
}
