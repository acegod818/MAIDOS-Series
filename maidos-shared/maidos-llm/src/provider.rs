//! LLM Provider trait and request types
//!
//! <impl>
//! WHAT: Define LlmProvider trait for unified provider interface
//! WHY: Abstract away provider differences, enable provider switching
//! HOW: async_trait for async methods, builder pattern for requests
//! TEST: Request building, provider trait requirements
//! </impl>

use crate::error::Result;
use crate::message::{CompletionResponse, Message, Usage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use futures::Stream;

/// Streaming chunk from LLM
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Delta text content
    pub delta: String,
    /// Accumulated text so far
    pub accumulated: String,
    /// Is this the final chunk?
    pub is_final: bool,
    /// Usage (only on final chunk typically)
    pub usage: Option<Usage>,
}

impl StreamChunk {
    /// Create a text-only chunk
    pub fn text(delta: impl Into<String>) -> Self {
        Self {
            delta: delta.into(),
            accumulated: String::new(), // In MAIDOS v2.2, accumulated is handled by caller or stateful stream
            is_final: false,
            usage: None,
        }
    }

    /// Create a final chunk with finish reason (finish reason is mapped to is_final in this simple struct)
    pub fn finish(_reason: impl Into<String>) -> Self {
        Self {
            delta: String::new(),
            accumulated: String::new(),
            is_final: true,
            usage: None,
        }
    }
}

/// Stream type alias
pub type CompletionStream = Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>;

/// Request parameters for LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model identifier
    pub model: String,
    /// Conversation messages
    pub messages: Vec<Message>,
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Temperature (0.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Top-p sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// Enable streaming
    #[serde(skip)]
    pub stream: bool,
    /// System prompt (some providers handle separately)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

impl Default for CompletionRequest {
    fn default() -> Self {
        Self {
            model: String::new(),
            messages: Vec::new(),
            max_tokens: Some(4096),
            temperature: None,
            top_p: None,
            stop: None,
            stream: false,
            system: None,
        }
    }
}

impl CompletionRequest {
    /// Create a new request with model
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }

    /// Add a message
    pub fn message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    /// Add messages from iterator
    pub fn messages(mut self, messages: impl IntoIterator<Item = Message>) -> Self {
        self.messages.extend(messages);
        self
    }

    /// Set system prompt
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Set max tokens
    pub fn max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = Some(max);
        self
    }

    /// Set temperature
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp.clamp(0.0, 2.0));
        self
    }

    /// Set top-p
    pub fn top_p(mut self, p: f32) -> Self {
        self.top_p = Some(p.clamp(0.0, 1.0));
        self
    }

    /// Add stop sequence
    pub fn stop(mut self, stop: impl Into<String>) -> Self {
        self.stop.get_or_insert_with(Vec::new).push(stop.into());
        self
    }

    /// Enable streaming
    pub fn streaming(mut self) -> Self {
        self.stream = true;
        self
    }

    /// Quick builder: model + user message
    pub fn quick(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self::new(model).message(Message::user(prompt))
    }
}

/// Provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider name
    pub name: String,
    /// Provider version
    pub version: String,
    /// Available models
    pub models: Vec<ModelInfo>,
    /// Base API URL
    pub base_url: String,
    /// Whether streaming is supported
    pub supports_streaming: bool,
    /// Whether vision (images) is supported
    pub supports_vision: bool,
    /// Whether tool use is supported
    pub supports_tools: bool,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Context window size
    pub context_window: u32,
    /// Max output tokens
    pub max_output_tokens: Option<u32>,
    /// Supports vision
    pub supports_vision: bool,
}

/// LLM Provider trait
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get provider information
    fn info(&self) -> &ProviderInfo;

    /// Complete a chat request (non-streaming)
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    /// Complete a chat request (streaming)
    async fn complete_stream(&self, request: CompletionRequest) -> Result<CompletionStream>;

    /// List available models
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;

    /// Check if provider is healthy/reachable
    async fn health_check(&self) -> Result<bool>;

    /// Get the provider name
    fn name(&self) -> &str {
        &self.info().name
    }

    /// Check if a model is available
    fn supports_model(&self, model: &str) -> bool {
        self.info().models.iter().any(|m| m.id == model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder() {
        let req = CompletionRequest::new("gpt-4")
            .system("You are helpful")
            .message(Message::user("Hello"))
            .max_tokens(1000)
            .temperature(0.7);

        assert_eq!(req.model, "gpt-4");
        assert_eq!(req.system, Some("You are helpful".to_string()));
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.max_tokens, Some(1000));
        assert_eq!(req.temperature, Some(0.7));
    }

    #[test]
    fn test_request_quick() {
        let req = CompletionRequest::quick("claude-3", "What is 2+2?");
        assert_eq!(req.model, "claude-3");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].text(), "What is 2+2?");
    }

    #[test]
    fn test_temperature_clamp() {
        let req = CompletionRequest::new("test").temperature(5.0);
        assert_eq!(req.temperature, Some(2.0));

        let req = CompletionRequest::new("test").temperature(-1.0);
        assert_eq!(req.temperature, Some(0.0));
    }

    #[test]
    fn test_top_p_clamp() {
        let req = CompletionRequest::new("test").top_p(1.5);
        assert_eq!(req.top_p, Some(1.0));
    }

    #[test]
    fn test_stop_sequences() {
        let req = CompletionRequest::new("test")
            .stop("END")
            .stop("STOP");
        assert_eq!(req.stop, Some(vec!["END".to_string(), "STOP".to_string()]));
    }

    #[test]
    fn test_streaming_flag() {
        let req = CompletionRequest::new("test").streaming();
        assert!(req.stream);
    }

    #[test]
    fn test_provider_info() {
        let info = ProviderInfo {
            name: "test".to_string(),
            version: "1.0".to_string(),
            models: vec![ModelInfo {
                id: "model-1".to_string(),
                name: "Model One".to_string(),
                context_window: 4096,
                max_output_tokens: Some(4096),
                supports_vision: false,
            }],
            base_url: "https://api.test.com".to_string(),
            supports_streaming: true,
            supports_vision: false,
            supports_tools: false,
        };

        assert_eq!(info.models.len(), 1);
        assert!(info.supports_streaming);
    }
}
