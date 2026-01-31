//! Message types for LLM conversations
//!
//! <impl>
//! WHAT: Define Message, Role, Content structs for LLM chat
//! WHY: Unified message format across all providers
//! HOW: Serde serialization with provider-agnostic design
//! TEST: Construction, serialization, builder pattern
//! </impl>

use serde::{Deserialize, Serialize};

/// Role in a conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System prompt / instructions
    System,
    /// User message
    User,
    /// Assistant response
    Assistant,
    /// Tool/function call result
    Tool,
}

impl Role {
    /// Convert to OpenAI-compatible string
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        }
    }
}

/// Content type in a message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Content {
    /// Plain text content
    Text { text: String },
    /// Image content (base64 or URL)
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        base64: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        media_type: Option<String>,
    },
    /// Tool use request
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    /// Tool result
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

impl Content {
    /// Create text content
    pub fn text(s: impl Into<String>) -> Self {
        Content::Text { text: s.into() }
    }

    /// Create image content from URL
    pub fn image_url(url: impl Into<String>) -> Self {
        Content::Image {
            url: Some(url.into()),
            base64: None,
            media_type: None,
        }
    }

    /// Create image content from base64
    pub fn image_base64(data: impl Into<String>, media_type: impl Into<String>) -> Self {
        Content::Image {
            url: None,
            base64: Some(data.into()),
            media_type: Some(media_type.into()),
        }
    }

    /// Check if this is text content
    pub fn is_text(&self) -> bool {
        matches!(self, Content::Text { .. })
    }

    /// Extract text if this is text content
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Content::Text { text } => Some(text),
            _ => None,
        }
    }
}

/// A message in an LLM conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// Role of the message sender
    pub role: Role,
    /// Content of the message (can be multiple parts)
    pub content: Vec<Content>,
    /// Optional name for the participant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Message {
    /// Create a new message with single text content
    pub fn new(role: Role, text: impl Into<String>) -> Self {
        Self {
            role,
            content: vec![Content::text(text)],
            name: None,
        }
    }

    /// Create a system message
    pub fn system(text: impl Into<String>) -> Self {
        Self::new(Role::System, text)
    }

    /// Create a user message
    pub fn user(text: impl Into<String>) -> Self {
        Self::new(Role::User, text)
    }

    /// Create an assistant message
    pub fn assistant(text: impl Into<String>) -> Self {
        Self::new(Role::Assistant, text)
    }

    /// Create a message with multiple content parts
    pub fn with_content(role: Role, content: Vec<Content>) -> Self {
        Self {
            role,
            content,
            name: None,
        }
    }

    /// Add a name to this message
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add content to this message
    pub fn add_content(&mut self, content: Content) {
        self.content.push(content);
    }

    /// Get all text content concatenated
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|c| c.as_text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Check if message has any image content
    pub fn has_image(&self) -> bool {
        self.content.iter().any(|c| matches!(c, Content::Image { .. }))
    }
}

/// Token usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Usage {
    /// Tokens in the prompt
    pub prompt_tokens: u32,
    /// Tokens in the completion
    pub completion_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
    /// Cached tokens (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,
}

impl Usage {
    /// Create new usage stats
    pub fn new(prompt: u32, completion: u32) -> Self {
        Self {
            prompt_tokens: prompt,
            completion_tokens: completion,
            total_tokens: prompt + completion,
            cached_tokens: None,
        }
    }
}

/// Response from an LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// The generated message
    pub message: Message,
    /// Token usage
    pub usage: Usage,
    /// Finish reason
    pub finish_reason: FinishReason,
    /// Model used
    pub model: String,
    /// Provider-specific ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Reason for completion finishing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural end of generation
    Stop,
    /// Hit max tokens limit
    Length,
    /// Content was filtered
    ContentFilter,
    /// Tool/function call
    ToolUse,
    /// Unknown/other reason
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_as_str() {
        assert_eq!(Role::System.as_str(), "system");
        assert_eq!(Role::User.as_str(), "user");
        assert_eq!(Role::Assistant.as_str(), "assistant");
        assert_eq!(Role::Tool.as_str(), "tool");
    }

    #[test]
    fn test_content_text() {
        let content = Content::text("Hello");
        assert!(content.is_text());
        assert_eq!(content.as_text(), Some("Hello"));
    }

    #[test]
    fn test_content_image_url() {
        let content = Content::image_url("https://example.com/image.png");
        assert!(!content.is_text());
        assert!(matches!(content, Content::Image { url: Some(_), .. }));
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello, world!");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.text(), "Hello, world!");
    }

    #[test]
    fn test_message_with_name() {
        let msg = Message::user("Hi").with_name("Alice");
        assert_eq!(msg.name, Some("Alice".to_string()));
    }

    #[test]
    fn test_message_multipart() {
        let msg = Message::with_content(
            Role::User,
            vec![
                Content::text("Describe this image:"),
                Content::image_url("https://example.com/cat.jpg"),
            ],
        );
        assert!(msg.has_image());
        assert_eq!(msg.text(), "Describe this image:");
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::system("You are a helpful assistant.");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("system"));
        assert!(json.contains("helpful assistant"));
    }

    #[test]
    fn test_usage_new() {
        let usage = Usage::new(100, 50);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_content_tool_use() {
        let content = Content::ToolUse {
            id: "call_123".to_string(),
            name: "get_weather".to_string(),
            input: serde_json::json!({"city": "Tokyo"}),
        };
        assert!(!content.is_text());
    }

    #[test]
    fn test_content_tool_result() {
        let content = Content::ToolResult {
            tool_use_id: "call_123".to_string(),
            content: "25°C, sunny".to_string(),
            is_error: None,
        };
        assert!(!content.is_text());
    }
}
