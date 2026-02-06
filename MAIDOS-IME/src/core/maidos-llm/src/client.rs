//! LLM client implementation
//!
//! This module provides client implementations for interacting with different LLM providers.

use crate::{LlmRequest, LlmResponse, LlmError, Result, ProviderType};

const OLLAMA_GENERATE_PATH: &str = "/api/generate";

fn resolve_ollama_base_url() -> String {
    // Soft-config via env vars (portable + system).
    // Prefer MAIDOS_* (our portable convention), then OLLAMA_HOST (Ollama convention).
    if let (Ok(host), Ok(port)) = (std::env::var("MAIDOS_OLLAMA_HOST"), std::env::var("MAIDOS_OLLAMA_PORT")) {
        let host = host.trim();
        let port = port.trim();
        if !host.is_empty() && !port.is_empty() {
            return format!("http://{}:{}", host, port);
        }
    }

    if let Ok(ollama_host) = std::env::var("OLLAMA_HOST") {
        let v = ollama_host.trim();
        if !v.is_empty() {
            if v.starts_with("http://") || v.starts_with("https://") {
                return v.to_string();
            }
            return format!("http://{}", v);
        }
    }

    // Default: local Ollama install.
    "http://127.0.0.1:11434".to_string()
}

/// LLM client
pub struct LlmClient {
    base_url: String,
}

impl LlmClient {
    /// Create a new LLM client
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    /// Create an LLM client from configuration
    pub fn from_config(config: &crate::models::ModelConfig) -> Result<Self> {
        let base_url = match config.provider.as_str() {
            "ollama" => {
                let base = resolve_ollama_base_url();
                format!("{}{}", base.trim_end_matches('/'), OLLAMA_GENERATE_PATH)
            }
            "local" => "local".to_string(), // Local model marker
            _ => return Err(LlmError::ConfigError("Unsupported provider".to_string())),
        };

        Ok(Self::new(base_url))
    }

    /// Send a request to the LLM
    pub async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse> {
        // If local model, use local LLM implementation
        if self.base_url == "local" {
            let local_llm = crate::local::LocalLlm::new();
            return local_llm.complete(request).await;
        }

        // Otherwise use HTTP API
        match request.provider {
            ProviderType::Ollama => self.call_ollama(request).await,
            _ => Err(LlmError::ConfigError("Unsupported provider".to_string())),
        }
    }

    /// Call Ollama API
    async fn call_ollama(&self, request: &LlmRequest) -> Result<LlmResponse> {
        use serde::{Deserialize, Serialize};
        use reqwest::Client;

        #[derive(Serialize)]
        struct OllamaRequest {
            model: String,
            prompt: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            system: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            options: Option<serde_json::Value>,
        }

        // Build prompt text
        let prompt = request.messages.iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let ollama_request = OllamaRequest {
            model: request.model.clone(),
            prompt,
            system: request.system.clone(),
            options: None,
        };

        let client = Client::new();
        let response = client
            .post(&self.base_url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(LlmError::ModelError(format!("HTTP error: {}", response.status())));
        }

        #[derive(Deserialize)]
        struct OllamaResponse {
            response: String,
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| LlmError::SerializationError(e.to_string()))?;

        Ok(LlmResponse {
            content: ollama_response.response,
            usage: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Message, ProviderType};

    #[tokio::test]
    async fn test_ollama_client_creation() {
        let base = "http://127.0.0.1:11434";
        let url = format!("{}{}", base, OLLAMA_GENERATE_PATH);
        let client = LlmClient::new(url.clone());
        assert_eq!(client.base_url, url);
    }

    #[tokio::test]
    async fn test_local_client() {
        let client = LlmClient::new("local");
        assert_eq!(client.base_url, "local");
        
        // Test local model
        let request = LlmRequest {
            provider: ProviderType::Ollama,
            model: "local".to_string(),
            system: None,
            messages: vec![Message::user("你好")],
            max_tokens: Some(10),
            temperature: Some(0.7),
        };

        let response = client.complete(&request).await;
        assert!(response.is_ok());
        assert!(!response.unwrap().content.is_empty());
    }

    #[test]
    fn test_from_config() {
        // Test Ollama config
        let config = crate::models::ModelConfig {
            provider: "ollama".to_string(),
            model: "llama3.2:3b-q4_0".to_string(),
            max_tokens: 10,
        };
        
        let client = LlmClient::from_config(&config);
        assert!(client.is_ok());
        let base = resolve_ollama_base_url();
        let expected = format!("{}{}", base.trim_end_matches('/'), OLLAMA_GENERATE_PATH);
        assert_eq!(client.unwrap().base_url, expected);

        // Test local config
        let local_config = crate::models::ModelConfig {
            provider: "local".to_string(),
            model: "local".to_string(),
            max_tokens: 10,
        };
        
        let local_client = LlmClient::from_config(&local_config);
        assert!(local_client.is_ok());
        assert_eq!(local_client.unwrap().base_url, "local");
    }
}
