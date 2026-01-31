//! Error types for maidos-llm
//!
//! <impl>
//! WHAT: Define LlmError enum for all LLM operations
//! WHY: Type-safe error handling with provider-specific context
//! HOW: thiserror derive for automatic Error impl
//! TEST: Error construction, display, and conversion
//! </impl>

use std::io;
use thiserror::Error;

/// Result type alias for LLM operations
pub type Result<T> = std::result::Result<T, LlmError>;

/// Errors that can occur during LLM operations
#[derive(Debug, Error)]
pub enum LlmError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Authentication error (invalid API key, etc.)
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Network/HTTP error
    #[error("Network error: {0}")]
    Network(String),

    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// API rate limit exceeded
    #[error("Rate limit exceeded: retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    /// Budget exceeded
    #[error("Budget exceeded: {0}")]
    BudgetExceeded(String),

    /// Invalid request (bad parameters, etc.)
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Provider error (generic)
    #[error("Provider error: {0}")]
    Provider(String),

    /// Provider returned an error with code
    #[error("Provider error [{code}]: {message}")]
    ProviderError { code: String, message: String },

    /// Response parsing error
    #[error("Failed to parse response: {0}")]
    ParseError(String),

    /// Model not found or not available
    #[error("Model not available: {0}")]
    ModelNotAvailable(String),

    /// Content was filtered/blocked
    #[error("Content filtered: {reason}")]
    ContentFiltered { reason: String },

    /// Context length exceeded
    #[error("Context length exceeded: {used} tokens used, {max} max")]
    ContextLengthExceeded { used: usize, max: usize },

    /// Timeout during operation
    #[error("Operation timed out after {0}ms")]
    Timeout(u64),

    /// Stream ended unexpectedly
    #[error("Stream ended unexpectedly")]
    StreamEnded,

    /// Provider not supported
    #[error("Provider not supported: {0}")]
    UnsupportedProvider(String),

    /// Vision/image input not supported by provider
    #[error("{provider} does not support vision/image input. {suggestion}")]
    VisionNotSupported {
        provider: String,
        suggestion: String,
    },

    /// Function calling not supported by provider
    #[error("{provider} does not support function calling. {suggestion}")]
    ToolsNotSupported {
        provider: String,
        suggestion: String,
    },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Capability not granted
    #[error("Capability not granted: {0}")]
    CapabilityDenied(String),
}

impl From<reqwest::Error> for LlmError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            LlmError::Timeout(30000) // default timeout assumption
        } else if e.is_connect() {
            LlmError::Network(format!("Connection failed: {}", e))
        } else {
            LlmError::Network(e.to_string())
        }
    }
}

impl From<serde_json::Error> for LlmError {
    fn from(e: serde_json::Error) -> Self {
        LlmError::ParseError(e.to_string())
    }
}

impl LlmError {
    /// Create a vision not supported error with helpful suggestion
    pub fn vision_not_supported(provider: &str) -> Self {
        Self::VisionNotSupported {
            provider: provider.to_string(),
            suggestion: "Try OpenAI (GPT-4o), Anthropic (Claude), Google (Gemini), or Mistral (Pixtral) for vision support.".to_string(),
        }
    }

    /// Create a vision not supported error with custom suggestion
    pub fn vision_not_supported_with_suggestion(provider: &str, suggestion: &str) -> Self {
        Self::VisionNotSupported {
            provider: provider.to_string(),
            suggestion: suggestion.to_string(),
        }
    }

    /// Create a tools not supported error with helpful suggestion
    pub fn tools_not_supported(provider: &str) -> Self {
        Self::ToolsNotSupported {
            provider: provider.to_string(),
            suggestion: "Try OpenAI, Anthropic, Google, Mistral, or Cohere for function calling support.".to_string(),
        }
    }

    /// Create a tools not supported error with custom suggestion
    pub fn tools_not_supported_with_suggestion(provider: &str, suggestion: &str) -> Self {
        Self::ToolsNotSupported {
            provider: provider.to_string(),
            suggestion: suggestion.to_string(),
        }
    }

    /// Check if this is a capability error (vision or tools not supported)
    pub fn is_capability_error(&self) -> bool {
        matches!(
            self,
            LlmError::VisionNotSupported { .. } | LlmError::ToolsNotSupported { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = LlmError::Auth("invalid key".to_string());
        assert!(err.to_string().contains("invalid key"));
    }

    #[test]
    fn test_rate_limited_display() {
        let err = LlmError::RateLimited { retry_after_secs: 60 };
        assert!(err.to_string().contains("60s"));
    }

    #[test]
    fn test_provider_error_display() {
        let err = LlmError::ProviderError {
            code: "invalid_model".to_string(),
            message: "Model does not exist".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("invalid_model"));
        assert!(s.contains("Model does not exist"));
    }

    #[test]
    fn test_context_length_display() {
        let err = LlmError::ContextLengthExceeded { used: 5000, max: 4096 };
        let s = err.to_string();
        assert!(s.contains("5000"));
        assert!(s.contains("4096"));
    }

    #[test]
    fn test_result_type() {
        let ok: Result<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: Result<i32> = Err(LlmError::Timeout(5000));
        assert!(matches!(err.unwrap_err(), LlmError::Timeout(5000)));
    }

    #[test]
    fn test_vision_not_supported() {
        let err = LlmError::vision_not_supported("DeepSeek");
        let s = err.to_string();
        assert!(s.contains("DeepSeek"));
        assert!(s.contains("vision"));
        assert!(s.contains("OpenAI"));
    }

    #[test]
    fn test_vision_not_supported_custom() {
        let err = LlmError::vision_not_supported_with_suggestion(
            "Cohere",
            "Use command-r-plus with vision enabled.",
        );
        let s = err.to_string();
        assert!(s.contains("Cohere"));
        assert!(s.contains("command-r-plus"));
    }

    #[test]
    fn test_tools_not_supported() {
        let err = LlmError::tools_not_supported("Replicate");
        let s = err.to_string();
        assert!(s.contains("Replicate"));
        assert!(s.contains("function calling"));
    }

    #[test]
    fn test_is_capability_error() {
        let vision_err = LlmError::vision_not_supported("Test");
        assert!(vision_err.is_capability_error());

        let tools_err = LlmError::tools_not_supported("Test");
        assert!(tools_err.is_capability_error());

        let other_err = LlmError::Auth("test".to_string());
        assert!(!other_err.is_capability_error());
    }
}
