//! MAIDOS Unified LLM Provider Interface
//!
//! A unified interface for multiple LLM providers including OpenAI,
//! Anthropic (Claude), and Ollama (local).
//!
//! <impl>
//! WHAT: Unified LLM provider abstraction layer
//! WHY: Enable provider switching without code changes
//! HOW: Trait-based design with async runtime, streaming support
//! TEST: Per-module tests + integration tests
//! </impl>
//!
//! # Example
//!
//! ```rust,no_run
//! use maidos_llm::{providers, CompletionRequest, Message, LlmProvider};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create provider
//!     let provider = providers::OpenAiProvider::new("your-api-key", None);
//!
//!     // Build request
//!     let request = CompletionRequest::new("gpt-4o")
//!         .system("You are a helpful assistant.")
//!         .message(Message::user("What is 2+2?"))
//!         .max_tokens(100);
//!
//!     // Get completion
//!     let response = provider.complete(request).await.unwrap();
//!     println!("{}", response.message.text());
//! }
//! ```

pub mod error;
pub mod ffi;
pub mod message;
pub mod provider;
pub mod providers;
pub mod router;
pub mod budget;
pub mod streaming;
pub mod tool;

// Re-exports for convenience
pub use error::{LlmError, Result};
pub use message::{CompletionResponse, Content, FinishReason, Message, Role, Usage};
pub use provider::{CompletionRequest, CompletionStream, LlmProvider, ModelInfo, ProviderInfo, StreamChunk};
pub use providers::{create_provider, ProviderType};
pub use router::{Router, RouterBuilder, RouterConfig, RoutingStrategy, ProviderConfig, ProviderHealth};
pub use budget::{BudgetController, BudgetBuilder, BudgetConfig, BudgetLimit, BudgetPeriod, BudgetStatus, ExceededAction, UsageStats};
pub use streaming::{
    StreamChunk as UnifiedStreamChunk, StreamUsage, StreamingResponse, BoxedStream,
    SseParser, SseEvent, StreamState, ToolCallDelta,
};
pub use tool::{
    MaidosTool, ToolParameter, ToolParameters, ProviderHints, ToProviderFormat,
    ToolCall, ToolResult, parse_openai_tool_call, parse_anthropic_tool_call,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_request() {
        let req = CompletionRequest::quick("gpt-4o", "Hello!");
        assert_eq!(req.model, "gpt-4o");
        assert_eq!(req.messages.len(), 1);
    }

    #[test]
    fn test_message_builder() {
        let msg = Message::user("Test").with_name("Alice");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.name, Some("Alice".to_string()));
    }

    #[test]
    fn test_provider_type_parsing() {
        assert!(ProviderType::parse("openai").is_some());
        assert!(ProviderType::parse("anthropic").is_some());
        assert!(ProviderType::parse("ollama").is_some());
        assert!(ProviderType::parse("unknown").is_none());
    }

    #[test]
    fn test_content_types() {
        let text = Content::text("Hello");
        assert!(text.is_text());
        assert_eq!(text.as_text(), Some("Hello"));

        let img = Content::image_url("https://example.com/img.png");
        assert!(!img.is_text());
    }

    #[test]
    fn test_usage_calculation() {
        let usage = Usage::new(100, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_multipart_message() {
        let msg = Message::with_content(
            Role::User,
            vec![
                Content::text("What's in this image?"),
                Content::image_url("https://example.com/cat.jpg"),
            ],
        );
        assert!(msg.has_image());
        assert_eq!(msg.text(), "What's in this image?");
    }
}
