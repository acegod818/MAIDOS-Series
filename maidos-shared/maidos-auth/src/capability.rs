//! Capability definitions
//!
//! <impl>
//! WHAT: Enum of all capabilities that can be granted via tokens
//! WHY: Fine-grained permission control for MAIDOS ecosystem
//! HOW: Bitflag-style enum for efficient storage and checking
//! TEST: Serialization/deserialization, bitwise operations
//! </impl>

use serde::{Deserialize, Serialize};

/// Capabilities that can be granted to tokens
///
/// Each capability represents a specific permission that can be
/// granted or denied independently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum Capability {
    // ========== LLM Operations ==========
    /// Basic chat completion
    LlmChat = 1 << 0,
    /// Vision/image input
    LlmVision = 1 << 1,
    /// Function/tool calling
    LlmFunction = 1 << 2,
    /// Streaming responses
    LlmStream = 1 << 3,

    // ========== File System ==========
    /// Read files
    FileRead = 1 << 4,
    /// Write files
    FileWrite = 1 << 5,
    /// Execute files
    FileExecute = 1 << 6,

    // ========== System ==========
    /// Execute shell commands
    ShellExec = 1 << 7,
    /// Access environment variables
    EnvAccess = 1 << 8,

    // ========== Input Control ==========
    /// Control mouse
    MouseControl = 1 << 9,
    /// Control keyboard
    KeyboardControl = 1 << 10,

    // ========== Network ==========
    /// Make HTTP requests
    HttpRequest = 1 << 11,
    /// WebSocket connections
    WebSocket = 1 << 12,

    // ========== Sensitive ==========
    /// Capture screen
    ScreenCapture = 1 << 13,
    /// Record audio
    AudioRecord = 1 << 14,
    /// Access clipboard
    Clipboard = 1 << 15,

    // ========== Event Bus ==========
    /// Publish events
    EventPublish = 1 << 16,
    /// Subscribe to events
    EventSubscribe = 1 << 17,
}

impl Capability {
    /// Get all capabilities as a slice
    pub fn all() -> &'static [Capability] {
        use Capability::*;
        &[
            LlmChat,
            LlmVision,
            LlmFunction,
            LlmStream,
            FileRead,
            FileWrite,
            FileExecute,
            ShellExec,
            EnvAccess,
            MouseControl,
            KeyboardControl,
            HttpRequest,
            WebSocket,
            ScreenCapture,
            AudioRecord,
            Clipboard,
            EventPublish,
            EventSubscribe,
        ]
    }

    /// Get capability name as string
    pub fn name(&self) -> &'static str {
        match self {
            Capability::LlmChat => "llm.chat",
            Capability::LlmVision => "llm.vision",
            Capability::LlmFunction => "llm.function",
            Capability::LlmStream => "llm.stream",
            Capability::FileRead => "file.read",
            Capability::FileWrite => "file.write",
            Capability::FileExecute => "file.execute",
            Capability::ShellExec => "shell.exec",
            Capability::EnvAccess => "env.access",
            Capability::MouseControl => "input.mouse",
            Capability::KeyboardControl => "input.keyboard",
            Capability::HttpRequest => "net.http",
            Capability::WebSocket => "net.websocket",
            Capability::ScreenCapture => "sensitive.screen",
            Capability::AudioRecord => "sensitive.audio",
            Capability::Clipboard => "sensitive.clipboard",
            Capability::EventPublish => "bus.publish",
            Capability::EventSubscribe => "bus.subscribe",
        }
    }

    /// Parse capability from name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "llm.chat" => Some(Capability::LlmChat),
            "llm.vision" => Some(Capability::LlmVision),
            "llm.function" => Some(Capability::LlmFunction),
            "llm.stream" => Some(Capability::LlmStream),
            "file.read" => Some(Capability::FileRead),
            "file.write" => Some(Capability::FileWrite),
            "file.execute" => Some(Capability::FileExecute),
            "shell.exec" => Some(Capability::ShellExec),
            "env.access" => Some(Capability::EnvAccess),
            "input.mouse" => Some(Capability::MouseControl),
            "input.keyboard" => Some(Capability::KeyboardControl),
            "net.http" => Some(Capability::HttpRequest),
            "net.websocket" => Some(Capability::WebSocket),
            "sensitive.screen" => Some(Capability::ScreenCapture),
            "sensitive.audio" => Some(Capability::AudioRecord),
            "sensitive.clipboard" => Some(Capability::Clipboard),
            "bus.publish" => Some(Capability::EventPublish),
            "bus.subscribe" => Some(Capability::EventSubscribe),
            _ => None,
        }
    }
}

/// A set of capabilities stored as a bitmask
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CapabilitySet(u32);

impl CapabilitySet {
    /// Create an empty capability set
    pub fn empty() -> Self {
        Self(0)
    }

    /// Create a capability set with all capabilities
    pub fn all() -> Self {
        let mut set = Self::empty();
        for cap in Capability::all() {
            set.grant(*cap);
        }
        set
    }

    /// Grant a capability
    pub fn grant(&mut self, cap: Capability) {
        self.0 |= cap as u32;
    }

    /// Revoke a capability
    pub fn revoke(&mut self, cap: Capability) {
        self.0 &= !(cap as u32);
    }

    /// Check if a capability is granted
    pub fn has(&self, cap: Capability) -> bool {
        (self.0 & (cap as u32)) != 0
    }

    /// Check if all given capabilities are granted
    pub fn has_all(&self, caps: &[Capability]) -> bool {
        caps.iter().all(|c| self.has(*c))
    }

    /// Check if any of the given capabilities is granted
    pub fn has_any(&self, caps: &[Capability]) -> bool {
        caps.iter().any(|c| self.has(*c))
    }

    /// Get the raw bitmask value
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// Create from raw bitmask value
    pub fn from_u32(value: u32) -> Self {
        Self(value)
    }

    /// Iterate over granted capabilities
    pub fn iter(&self) -> impl Iterator<Item = Capability> + '_ {
        Capability::all().iter().filter(|c| self.has(**c)).copied()
    }
}

impl FromIterator<Capability> for CapabilitySet {
    fn from_iter<T: IntoIterator<Item = Capability>>(iter: T) -> Self {
        let mut set = Self::empty();
        for cap in iter {
            set.grant(cap);
        }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_set_empty() {
        let set = CapabilitySet::empty();
        assert!(!set.has(Capability::LlmChat));
        assert_eq!(set.as_u32(), 0);
    }

    #[test]
    fn test_capability_set_grant_revoke() {
        let mut set = CapabilitySet::empty();
        set.grant(Capability::LlmChat);
        assert!(set.has(Capability::LlmChat));

        set.revoke(Capability::LlmChat);
        assert!(!set.has(Capability::LlmChat));
    }

    #[test]
    fn test_capability_set_multiple() {
        let mut set = CapabilitySet::empty();
        set.grant(Capability::LlmChat);
        set.grant(Capability::FileRead);
        set.grant(Capability::HttpRequest);

        assert!(set.has(Capability::LlmChat));
        assert!(set.has(Capability::FileRead));
        assert!(set.has(Capability::HttpRequest));
        assert!(!set.has(Capability::ShellExec));
    }

    #[test]
    fn test_capability_set_has_all() {
        let mut set = CapabilitySet::empty();
        set.grant(Capability::LlmChat);
        set.grant(Capability::LlmVision);

        assert!(set.has_all(&[Capability::LlmChat, Capability::LlmVision]));
        assert!(!set.has_all(&[Capability::LlmChat, Capability::ShellExec]));
    }

    #[test]
    fn test_capability_set_has_any() {
        let mut set = CapabilitySet::empty();
        set.grant(Capability::LlmChat);

        assert!(set.has_any(&[Capability::LlmChat, Capability::ShellExec]));
        assert!(!set.has_any(&[Capability::FileWrite, Capability::ShellExec]));
    }

    #[test]
    fn test_capability_set_from_iter() {
        let set: CapabilitySet = vec![Capability::LlmChat, Capability::FileRead]
            .into_iter()
            .collect();

        assert!(set.has(Capability::LlmChat));
        assert!(set.has(Capability::FileRead));
        assert!(!set.has(Capability::ShellExec));
    }

    #[test]
    fn test_capability_name_roundtrip() {
        for cap in Capability::all() {
            let name = cap.name();
            let parsed = Capability::from_name(name);
            assert_eq!(parsed, Some(*cap));
        }
    }

    #[test]
    fn test_capability_set_iter() {
        let mut set = CapabilitySet::empty();
        set.grant(Capability::LlmChat);
        set.grant(Capability::FileRead);

        let caps: Vec<_> = set.iter().collect();
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&Capability::LlmChat));
        assert!(caps.contains(&Capability::FileRead));
    }

    #[test]
    fn test_capability_serialization() {
        let set: CapabilitySet = vec![Capability::LlmChat, Capability::FileRead]
            .into_iter()
            .collect();

        let json = serde_json::to_string(&set).unwrap();
        let deserialized: CapabilitySet = serde_json::from_str(&json).unwrap();

        assert_eq!(set, deserialized);
    }
}
