//! Local LLM implementation
//!
//! Provides lightweight local inference without external service dependencies.

use crate::{LlmRequest, LlmResponse, Result};
use std::collections::HashMap;

/// Simple rule-based local LLM implementation
pub struct LocalLlm {
    patterns: HashMap<&'static str, &'static str>,
}

impl Default for LocalLlm {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalLlm {
    /// Create a new local LLM
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        patterns.insert("hello", "hello");
        patterns.insert("world", "world");
        patterns.insert("test", "test");
        
        Self { patterns }
    }

    /// Simple rule-based inference
    pub fn simple_inference(&self, prompt: &str) -> String {
        // Match by keyword
        for (keyword, response) in &self.patterns {
            if prompt.contains(keyword) {
                return response.to_string();
            }
        }

        // Default response
        "ok".to_string()
    }

    /// Process LLM request
    pub async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse> {
        // Build prompt
        let prompt = request.messages.iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        // Perform simple inference
        let response_content = self.simple_inference(&prompt);

        Ok(LlmResponse {
            content: response_content,
            usage: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Message, ProviderType};

    #[tokio::test]
    async fn test_local_llm_basic() {
        let llm = LocalLlm::new();
        let request = LlmRequest {
            provider: ProviderType::Ollama,
            model: "local".to_string(),
            system: None,
            messages: vec![Message::user("hello")],
            max_tokens: Some(10),
            temperature: Some(0.7),
        };

        let response = llm.complete(&request).await;
        assert!(response.is_ok());
        assert!(!response.unwrap().content.is_empty());
    }

    #[test]
    fn test_simple_inference() {
        let llm = LocalLlm::new();
        
        // Test keyword matching
        assert_eq!(llm.simple_inference("hello"), "hello");
        assert_eq!(llm.simple_inference("world"), "world");
        assert_eq!(llm.simple_inference("test"), "test");
        
        // Test default response
        assert_eq!(llm.simple_inference("unknown"), "ok");
    }
}
