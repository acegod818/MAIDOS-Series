use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Simple P2P message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub from: String,
    pub to: String,
    pub kind: MessageKind,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub signature: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageKind {
    Text,
    Data,
    Control,
    Heartbeat,
}

impl Message {
    pub fn new_text(from: &str, to: &str, text: &str) -> Self {
        Self {
            id: rand::random(),
            from: from.to_string(),
            to: to.to_string(),
            kind: MessageKind::Text,
            payload: text.as_bytes().to_vec(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            signature: None,
        }
    }

    pub fn new_data(from: &str, to: &str, data: Vec<u8>) -> Self {
        Self {
            id: rand::random(),
            from: from.to_string(),
            to: to.to_string(),
            kind: MessageKind::Data,
            payload: data,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            signature: None,
        }
    }

    pub fn heartbeat(from: &str) -> Self {
        Self {
            id: rand::random(),
            from: from.to_string(),
            to: "broadcast".to_string(),
            kind: MessageKind::Heartbeat,
            payload: Vec::new(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            signature: None,
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_text_roundtrip() {
        let msg = Message::new_text("a", "b", "hello");
        let data = msg.serialize().unwrap();
        let decoded = Message::deserialize(&data).unwrap();
        assert_eq!(decoded.from, "a");
        assert_eq!(decoded.to, "b");
        assert!(matches!(decoded.kind, MessageKind::Text));
    }

    #[test]
    fn test_message_data_roundtrip() {
        let msg = Message::new_data("a", "b", vec![1, 2, 3]);
        let data = msg.serialize().unwrap();
        let decoded = Message::deserialize(&data).unwrap();
        assert!(matches!(decoded.kind, MessageKind::Data));
        assert_eq!(decoded.payload, vec![1, 2, 3]);
    }

    #[test]
    fn test_message_heartbeat() {
        let msg = Message::heartbeat("node");
        assert!(matches!(msg.kind, MessageKind::Heartbeat));
        assert_eq!(msg.to, "broadcast");
        assert!(msg.payload.is_empty());
    }
}
