use crate::{BinaryDeserializable, BinarySerializable, TextMessage};
pub use message_pack::MessageType;
use message_pack::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn serialize_exit_message_string() -> String {
    String::from("exit")
}

#[wasm_bindgen]
pub fn serialize_exit_message() -> Vec<u8> {
    vec![0x04]
}

#[wasm_bindgen]
pub fn serialize_text_message(sender: String, room: i32, category: u8, content: String) -> Vec<u8> {
    let category = match category {
        0x01 => MessageType::Chat,
        0x02 => MessageType::Exit,
        0x03 => MessageType::FileTransfer,
        0x04 => MessageType::List,
        _ => MessageType::Unknown,
    };

    let message = TextMessage {
        sender,
        room,
        category,
        content,
    };

    message.to_bytes()
}

#[wasm_bindgen]
pub fn deserialize_text_message(data: &[u8]) -> Result<JsValue, JsValue> {
    let message = TextMessage::from_bytes(data)
        .map_err(|e| JsValue::from(format!("Failed to deserialize: {}", e)))?;

    // JavaScriptオブジェクトとして返す
    Ok(serde_wasm_bindgen::to_value(&message).unwrap())
}

#[wasm_bindgen]
pub fn serialize_list_message(sender: String, room: i32, target: Option<String>) -> Vec<u8> {
    let target = match target {
        None => "socket".to_string(),
        Some(target) if target == "" => "socket".to_string(),
        Some(target) => target.to_string(),
    };

    let message = ListMessage {
        sender,
        target,
        room,
        category: MessageType::List,
    };

    message.to_bytes()
}

#[wasm_bindgen]
pub fn convert_to_bytes(message_type: MessageType) -> u8 {
    message_type.to_bytes()
}

#[wasm_bindgen]
pub fn convert_from_bytes(data: u8) -> MessageType {
    MessageType::from_bytes(&data).expect("Failed to convert from bytes")
}
