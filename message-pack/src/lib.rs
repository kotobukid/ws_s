use std::fmt::Debug;
use std::io::{Cursor, Read};

#[derive(Eq)]
pub enum MessageCategory {
    ChatMessage,
    Exit,
}

impl MessageCategory {
    pub fn to_bytes(&self) -> u8 {
        match self {
            MessageCategory::ChatMessage => 0x01,
            MessageCategory::Exit => 0x02,
        }
    }

    pub fn from_bytes(data: &u8) -> Result<Self, String> {
        match data {
            0x01 => Ok(MessageCategory::ChatMessage),
            0x02 => Ok(MessageCategory::Exit),
            _ => Err("Invalid message category".to_string()),
        }
    }
}

impl Debug for MessageCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageCategory::ChatMessage => write!(f, "ChatMessage"),
            MessageCategory::Exit => write!(f, "Exit"),
        }
    }
}

impl PartialEq<Self> for MessageCategory {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ChatMessage {
    pub author: String,
    pub room: i32,
    pub category: MessageCategory,
    pub message: String,
}

impl ChatMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
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
        let message_bytes = self.message.as_bytes();
        buffer.extend(&(message_bytes.len() as u16).to_be_bytes());

        // 本文のバイト列
        buffer.extend(message_bytes);

        // 最後に簡易的なチェックサム (すべてのバイトを合計して8ビットで切り捨て)
        let checksum: u8 = buffer.iter().fold(0, |acc, &x| acc.wrapping_add(x));
        buffer.push(checksum);

        buffer
    }
}

impl ChatMessage {
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        let mut cursor = Cursor::new(data);

        // カテゴリ (1バイト: u8)
        let mut category_buf = [0u8; 1];
        cursor
            .read_exact(&mut category_buf)
            .map_err(|_| "Failed to read category")?;
        let category = MessageCategory::from_bytes(&category_buf[0])?;

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
            author,
            room,
            category,
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let message = ChatMessage {
            author: "Alice".to_string(),
            room: 42,
            category: MessageCategory::ChatMessage,
            message: "Hello, world!".to_string(),
        };

        let message = ChatMessage {
            author: "Alice".to_string(),
            room: 42,
            category: MessageCategory::ChatMessage,
            message: "Hello, world!".to_string(),
        };

        let bytes = message.to_bytes();

        let decoded: ChatMessage = ChatMessage::from_bytes(&bytes).unwrap();

        assert_eq!(message, decoded);
    }
}
