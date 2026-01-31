//! Groq API Provider
//!
//! <impl>
//! WHAT: Groq API implementation (OpenAI-compatible with LPU acceleration)
//! WHY: Ultra-fast inference with specialized hardware
//! HOW: OpenAI-compatible REST API
//! TEST: Request formatting, response parsing, error handling
//! </impl>

use crate::error::{LlmError, Result};
use crate::message::{CompletionResponse, FinishReason, Message, Usage};
use crate::provider::{
    CompletionRequest, CompletionStream, LlmProvider, ModelInfo, ProviderInfo,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_BASE_URL: &str = "https://api.groq.com/openai/v1";

/// Groq Provider (OpenAI-compatible API with LPU acceleration)
pub struct GroqProvider {
    client: Client,
    api_key: String,
    base_url: String,
    info: ProviderInfo,
}

impl GroqProvider {
    /// Create new Groq provider
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: base_url.clone(),
            info: ProviderInfo {
                name: "Groq".to_string(),
                version: "v1".to_string(),
                models: vec![
                    ModelInfo {
                        id: "llama-3.3-70b-versatile".to_string(),
                        name: "Llama 3.3 70B Versatile".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(32768),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "llama-3.1-70b-versatile".to_string(),
                        name: "Llama 3.1 70B Versatile".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8000),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "llama-3.1-8b-instant".to_string(),
                        name: "Llama 3.1 8B Instant".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8000),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "mixtral-8x7b-32768".to_string(),
                        name: "Mixtral 8x7B".to_string(),
                        context_window: 32_768,
                        max_output_tokens: Some(32768),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "gemma2-9b-it".to_string(),
                        name: "Gemma 2 9B".to_string(),
                        context_window: 8_192,
                        max_output_tokens: Some(8192),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "llama-3.2-11b-vision-preview".to_string(),
                        name: "Llama 3.2 11B Vision".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8000),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "llama-3.2-90b-vision-preview".to_string(),
                        name: "Llama 3.2 90B Vision".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8000),
                        supports_vision: true,
                    },
                ],
                base_url,
                supports_streaming: true,
                supports_vision: true,
                supports_tools: true,
            },
        }
    }

    fn build_request(&self, req: &CompletionRequest) -> OpenAiCompatRequest {
        let mut messages = Vec::new();

        if let Some(system) = &req.system {
            messages.push(OpenAiMessage {
                role: "system".to_string(),
                content: system.clone(),
                name: None,
            });
        }

        for msg in &req.messages {
            messages.push(OpenAiMessage {
                role: msg.role.as_str().to_string(),
                content: msg.text().to_string(),
                name: msg.name.clone(),
            });
        }

        OpenAiCompatRequest {
            model: req.model.clone(),
            messages,
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            top_p: req.top_p,
            stop: if req.stop.as_ref().is_none_or(|v| v.is_empty()) { None } else { req.stop.clone() },
            stream: Some(false),
        }
    }

    fn parse_response(&self, resp: OpenAiCompatResponse) -> Result<CompletionResponse> {
        let choice = resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| LlmError::ParseError("No choices in response".to_string()))?;

        let finish_reason = choice.finish_reason.map(|r| match r.as_str() {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "content_filter" => FinishReason::ContentFilter,
            "tool_calls" => FinishReason::ToolUse,
            _ => FinishReason::Unknown,
        }).unwrap_or(FinishReason::Unknown);

        let usage = resp.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            cached_tokens: None,
        }).unwrap_or_default();

        Ok(CompletionResponse {
            message: Message::assistant(&choice.message.content),
            usage,
            finish_reason,
            model: resp.model,
            id: resp.id,
        })
    }
}

#[async_trait]
impl LlmProvider for GroqProvider {
    fn info(&self) -> &ProviderInfo {
        &self.info
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_request(&request);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if status.as_u16() == 401 {
                return Err(LlmError::Auth("Invalid Groq API key".to_string()));
            }
            if status.as_u16() == 429 {
                return Err(LlmError::RateLimited { retry_after_secs: 10 });
            }
            if status.as_u16() == 400 {
                return Err(LlmError::InvalidRequest(body));
            }

            return Err(LlmError::ProviderError {
                code: status.to_string(),
                message: body,
            });
        }

        let api_response: OpenAiCompatResponse = response.json().await?;
        self.parse_response(api_response)
    }

    async fn complete_stream(&self, _request: CompletionRequest) -> Result<CompletionStream> {
        Err(LlmError::Provider("Streaming not yet implemented for Groq".to_string()))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(self.info.models.clone())
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;
        Ok(response.status().is_success())
    }
}

// OpenAI-compatible API types

#[derive(Debug, Serialize)]
struct OpenAiCompatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiCompatResponse {
    id: Option<String>,
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let provider = GroqProvider::new("test-key", None);
        let info = provider.info();
        assert_eq!(info.name, "Groq");
        assert!(!info.models.is_empty());
    }

    #[test]
    fn test_build_request() {
        let provider = GroqProvider::new("test-key", None);
        let request = CompletionRequest::new("llama-3.3-70b-versatile")
            .system("You are helpful.")
            .message(Message::user("Hello"));

        let api_req = provider.build_request(&request);
        assert_eq!(api_req.model, "llama-3.3-70b-versatile");
        assert_eq!(api_req.messages.len(), 2);
    }

    #[test]
    fn test_custom_base_url() {
        let provider = GroqProvider::new("key", Some("https://custom.api.com".to_string()));
        assert_eq!(provider.base_url, "https://custom.api.com");
    }

    #[test]
    fn test_default_base_url() {
        let provider = GroqProvider::new("key", None);
        assert!(provider.base_url.contains("groq.com"));
    }

    #[test]
    fn test_vision_models() {
        let provider = GroqProvider::new("key", None);
        let info = provider.info();
        let vision_models: Vec<_> = info.models.iter().filter(|m| m.supports_vision).collect();
        assert!(!vision_models.is_empty());
    }

    #[test]
    fn test_llama_models() {
        let provider = GroqProvider::new("key", None);
        let info = provider.info();
        let llama_models: Vec<_> = info.models.iter().filter(|m| m.id.contains("llama")).collect();
        assert!(llama_models.len() >= 4);
    }
}
