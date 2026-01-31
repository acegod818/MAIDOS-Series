//! Event types and serialization
//!
//! <impl>
//! WHAT: Define Event struct for message passing, with topic-based routing
//! WHY: Unified message format across all bus operations
//! HOW: Serde + MessagePack for efficient binary serialization
//! TEST: Roundtrip serialization, topic validation
//! </impl>

use crate::error::{BusError, Result};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum topic length
const MAX_TOPIC_LENGTH: usize = 256;

/// Maximum payload size (1MB)
const MAX_PAYLOAD_SIZE: usize = 1024 * 1024;

/// Event message for bus transport
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    /// Topic for routing (e.g., "maidos.config.changed")
    pub topic: String,
    /// Unique event ID
    pub id: u64,
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
    /// Sender identifier
    pub source: String,
    /// Binary payload (MessagePack encoded application data)
    pub payload: Vec<u8>,
}

impl Event {
    /// Create a new event with auto-generated ID and timestamp
    ///
    /// # Arguments
    /// * `topic` - Routing topic (dot-separated, e.g., "maidos.config")
    /// * `source` - Sender identifier
    /// * `payload` - Raw payload bytes
    ///
    /// # Errors
    /// Returns error if topic or payload validation fails
    pub fn new(topic: impl Into<String>, source: impl Into<String>, payload: Vec<u8>) -> Result<Self> {
        let topic = topic.into();
        let source = source.into();

        // Validate topic
        if topic.is_empty() {
            return Err(BusError::InvalidTopic("topic cannot be empty".to_string()));
        }
        if topic.len() > MAX_TOPIC_LENGTH {
            return Err(BusError::InvalidTopic(format!(
                "topic exceeds {} chars",
                MAX_TOPIC_LENGTH
            )));
        }
        if !topic.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-') {
            return Err(BusError::InvalidTopic(
                "topic must contain only alphanumeric, dot, underscore, or hyphen".to_string(),
            ));
        }

        // Validate payload size
        if payload.len() > MAX_PAYLOAD_SIZE {
            return Err(BusError::Serialization(format!(
                "payload exceeds {} bytes",
                MAX_PAYLOAD_SIZE
            )));
        }

        let id = generate_id();
        let timestamp = current_timestamp();

        Ok(Self {
            topic,
            id,
            timestamp,
            source,
            payload,
        })
    }

    /// Create event with typed payload (auto-serialized)
    pub fn with_data<T: Serialize>(
        topic: impl Into<String>,
        source: impl Into<String>,
        data: &T,
    ) -> Result<Self> {
        let payload = rmp_serde::to_vec(data)?;
        Self::new(topic, source, payload)
    }

    /// Deserialize payload to typed data
    pub fn data<'a, T: Deserialize<'a>>(&'a self) -> Result<T> {
        rmp_serde::from_slice(&self.payload).map_err(BusError::from)
    }

    /// Serialize event to MessagePack bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self).map_err(BusError::from)
    }

    /// Deserialize event from MessagePack bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(bytes).map_err(BusError::from)
    }

    /// Check if event matches a topic pattern
    /// Supports wildcard: "maidos.*" matches "maidos.config", "maidos.auth"
    pub fn matches_topic(&self, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        if let Some(prefix) = pattern.strip_suffix(".*") {
            return self.topic.starts_with(prefix)
                && self.topic[prefix.len()..].starts_with('.');
        }
        self.topic == pattern
    }
}

/// Generate unique event ID
fn generate_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    
    let time_part = current_timestamp() << 20;
    let seq_part = COUNTER.fetch_add(1, Ordering::SeqCst) & 0xFFFFF;
    time_part | seq_part
}

/// Get current Unix timestamp in milliseconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Wire format header for framing
#[derive(Debug, Clone, Copy)]
pub(crate) struct FrameHeader {
    pub length: u32,
}

impl FrameHeader {
    pub const SIZE: usize = 4;

    pub fn new(length: u32) -> Self {
        Self { length }
    }

    pub fn to_bytes(self) -> [u8; 4] {
        self.length.to_be_bytes()
    }

    pub fn from_bytes(bytes: &[u8; 4]) -> Self {
        Self {
            length: u32::from_be_bytes(*bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::new("maidos.config", "test-source", vec![1, 2, 3]).unwrap();
        assert_eq!(event.topic, "maidos.config");
        assert_eq!(event.source, "test-source");
        assert_eq!(event.payload, vec![1, 2, 3]);
        assert!(event.id > 0);
        assert!(event.timestamp > 0);
    }

    #[test]
    fn test_event_with_typed_data() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let event = Event::with_data("maidos.test", "source", &data).unwrap();
        let decoded: TestData = event.data().unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_event_roundtrip() {
        let original = Event::new("test.topic", "source", vec![1, 2, 3, 4]).unwrap();
        let bytes = original.to_bytes().unwrap();
        let decoded = Event::from_bytes(&bytes).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_topic_validation_empty() {
        let result = Event::new("", "source", vec![]);
        assert!(matches!(result, Err(BusError::InvalidTopic(_))));
    }

    #[test]
    fn test_topic_validation_too_long() {
        let long_topic = "a".repeat(MAX_TOPIC_LENGTH + 1);
        let result = Event::new(long_topic, "source", vec![]);
        assert!(matches!(result, Err(BusError::InvalidTopic(_))));
    }

    #[test]
    fn test_topic_validation_invalid_chars() {
        let result = Event::new("topic/with/slash", "source", vec![]);
        assert!(matches!(result, Err(BusError::InvalidTopic(_))));
    }

    #[test]
    fn test_topic_matching_exact() {
        let event = Event::new("maidos.config", "src", vec![]).unwrap();
        assert!(event.matches_topic("maidos.config"));
        assert!(!event.matches_topic("maidos.auth"));
    }

    #[test]
    fn test_topic_matching_wildcard() {
        let event = Event::new("maidos.config.changed", "src", vec![]).unwrap();
        assert!(event.matches_topic("maidos.*"));
        assert!(event.matches_topic("*"));
        assert!(!event.matches_topic("other.*"));
    }

    #[test]
    fn test_frame_header_roundtrip() {
        let header = FrameHeader::new(12345);
        let bytes = header.to_bytes();
        let decoded = FrameHeader::from_bytes(&bytes);
        assert_eq!(header.length, decoded.length);
    }

    #[test]
    fn test_unique_ids() {
        let e1 = Event::new("test", "src", vec![]).unwrap();
        let e2 = Event::new("test", "src", vec![]).unwrap();
        assert_ne!(e1.id, e2.id);
    }
}
