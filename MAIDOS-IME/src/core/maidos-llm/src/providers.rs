//! LLM provider implementations
//!
//! Concrete provider implementations for different LLM backends.

use crate::{LlmRequest, LlmResponse, LlmError, Result, ProviderType};
use async_trait::async_trait;

/// LLM provider trait
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a completion request to the LLM
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse>;

    /// Get the provider type
    fn provider_type(&self) -> ProviderType;

    /// Check if the provider is available
    fn is_available(&self) -> bool;
}

/// Ollama provider — real HTTP POST to local Ollama API
pub struct OllamaProvider {
    base_url: String,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    /// Extract host:port from base_url for TCP availability check
    fn host_port(&self) -> Option<String> {
        let url = self.base_url.trim_end_matches('/');
        // Strip http:// or https://
        let stripped = url
            .strip_prefix("http://")
            .or_else(|| url.strip_prefix("https://"))
            .unwrap_or(url);
        // Take up to the first '/' (path)
        let authority = stripped.split('/').next().unwrap_or(stripped);
        if authority.is_empty() {
            None
        } else {
            Some(authority.to_string())
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse> {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize)]
        struct OllamaRequest {
            model: String,
            prompt: String,
            stream: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            system: Option<String>,
        }

        #[derive(Deserialize)]
        struct OllamaResponse {
            response: String,
        }

        let prompt = request.messages.iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let ollama_req = OllamaRequest {
            model: request.model.clone(),
            prompt,
            stream: false,
            system: request.system.clone(),
        };

        let url = format!("{}/api/generate", self.base_url.trim_end_matches('/'));

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        let response = client
            .post(&url)
            .json(&ollama_req)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(LlmError::ModelError(format!("HTTP error: {}", response.status())));
        }

        let ollama_resp: OllamaResponse = response
            .json()
            .await
            .map_err(|e| LlmError::SerializationError(e.to_string()))?;

        Ok(LlmResponse {
            content: ollama_resp.response,
            usage: None,
        })
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Ollama
    }

    fn is_available(&self) -> bool {
        // TCP connect check with 1s timeout
        if let Some(addr) = self.host_port() {
            use std::net::{TcpStream, ToSocketAddrs};
            if let Ok(mut addrs) = addr.to_socket_addrs() {
                if let Some(sock_addr) = addrs.next() {
                    return TcpStream::connect_timeout(&sock_addr, std::time::Duration::from_secs(1)).is_ok();
                }
            }
        }
        false
    }
}

/// Provider factory
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create a provider instance by type
    pub fn create_provider(provider_type: &ProviderType) -> Result<Box<dyn LlmProvider>> {
        match provider_type {
            ProviderType::Ollama => {
                Ok(Box::new(OllamaProvider::new("http://localhost:11434")))
            },
            _ => Err(LlmError::ConfigError("Unsupported provider type".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_provider_creation() {
        let provider = OllamaProvider::new("http://localhost:11434");
        assert_eq!(provider.provider_type(), ProviderType::Ollama);
        // is_available depends on whether Ollama is running locally — just test it doesn't panic
        let _ = provider.is_available();
    }

    #[tokio::test]
    async fn test_provider_factory() {
        let provider = ProviderFactory::create_provider(&ProviderType::Ollama);
        assert!(provider.is_ok());

        let provider = provider.expect("Failed to create provider");
        assert_eq!(provider.provider_type(), ProviderType::Ollama);
    }
}