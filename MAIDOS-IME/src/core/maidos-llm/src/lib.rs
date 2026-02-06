//! MAIDOS-LLM module
//!
//! This module provides AI model interaction functionality, including:
//! - Local Ollama model invocation
//! - Cloud API calls (future extension)
//! - Model configuration management

pub mod client;
pub mod models;
pub mod providers;
pub mod local;

/// LLM provider type
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderType {
    Ollama,
    OpenAI,
    Anthropic,
    // More providers can be added in the future
}

/// LLM message role
#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    System,
    User,
    Assistant,
}

/// LLM message
#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// LLM request
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub provider: ProviderType,
    pub model: String,
    pub system: Option<String>,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

impl Default for LlmRequest {
    fn default() -> Self {
        Self {
            provider: ProviderType::Ollama,
            model: "llama3.2:3b-q4_0".to_string(),
            system: None,
            messages: vec![],
            max_tokens: Some(10),
            temperature: Some(0.7),
        }
    }
}

/// LLM response
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub usage: Option<Usage>,
}

/// Usage statistics
#[derive(Debug, Clone)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// MAIDOS-LLM error type
#[derive(thiserror::Error, Debug)]
pub enum LlmError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Model error: {0}")]
    ModelError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Config error: {0}")]
    ConfigError(String),
}

/// MAIDOS-LLM result type
pub type Result<T> = std::result::Result<T, LlmError>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}