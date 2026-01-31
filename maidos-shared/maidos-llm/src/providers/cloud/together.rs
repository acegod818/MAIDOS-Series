//! Together AI API Provider
//!
//! <impl>
//! WHAT: Together AI API implementation (OpenAI-compatible for open-source models)
//! WHY: Access to open-source models like Llama, Mixtral, Qwen with OpenAI compatibility
//! HOW: OpenAI-compatible REST API
//! TEST: Request formatting, response parsing, model support
//! </impl>

use crate::error::{LlmError, Result};
use crate::message::{CompletionResponse, FinishReason, Message, Usage};
use crate::provider::{
    CompletionRequest, CompletionStream, LlmProvider, ModelInfo, ProviderInfo, StreamChunk,
};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_BASE_URL: &str = "https://api.together.xyz/v1";

/// Together AI Provider (OpenAI-compatible API for open-source models)
pub struct TogetherProvider {
    client: Client,
    api_key: String,
    base_url: String,
    info: ProviderInfo,
}

impl TogetherProvider {
    /// Create new Together AI provider
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: base_url.clone(),
            info: ProviderInfo {
                name: "Together AI".to_string(),
                version: "v1".to_string(),
                models: vec![
                    // Llama 3.3
                    ModelInfo {
                        id: "meta-llama/Llama-3.3-70B-Instruct-Turbo".to_string(),
                        name: "Llama 3.3 70B Instruct Turbo".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8192),
                        supports_vision: false,
                    },
                    // Llama 3.2 Vision
                    ModelInfo {
                        id: "meta-llama/Llama-3.2-90B-Vision-Instruct-Turbo".to_string(),
                        name: "Llama 3.2 90B Vision".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8192),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "meta-llama/Llama-3.2-11B-Vision-Instruct-Turbo".to_string(),
                        name: "Llama 3.2 11B Vision".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8192),
                        supports_vision: true,
                    },
                    // Mixtral
                    ModelInfo {
                        id: "mistralai/Mixtral-8x22B-Instruct-v0.1".to_string(),
                        name: "Mixtral 8x22B Instruct".to_string(),
                        context_window: 65_536,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "mistralai/Mixtral-8x7B-Instruct-v0.1".to_string(),
                        name: "Mixtral 8x7B Instruct".to_string(),
                        context_window: 32_768,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    // Qwen
                    ModelInfo {
                        id: "Qwen/Qwen2.5-72B-Instruct-Turbo".to_string(),
                        name: "Qwen 2.5 72B Instruct".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8192),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "Qwen/Qwen2-72B-Instruct".to_string(),
                        name: "Qwen 2 72B Instruct".to_string(),
                        context_window: 32_768,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    // DeepSeek
                    ModelInfo {
                        id: "deepseek-ai/deepseek-coder-33b-instruct".to_string(),
                        name: "DeepSeek Coder 33B".to_string(),
                        context_window: 16_384,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                ],
                base_url,
                supports_streaming: true,
                supports_vision: true, // Some models support vision
                supports_tools: true,
            },
        }
    }

    /// Check if a model supports vision
    pub fn model_supports_vision(model: &str) -> bool {
        model.to_lowercase().contains("vision")
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
            "eos" => FinishReason::Stop,
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
impl LlmProvider for TogetherProvider {
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
                return Err(LlmError::Auth("Invalid Together AI API key".to_string()));
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

    async fn complete_stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        let mut req = request;
        req.stream = true;
        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_request(&req);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            if status.as_u16() == 401 {
                return Err(LlmError::Auth("Invalid Together AI API key".to_string()));
            }
            if status.as_u16() == 429 {
                return Err(LlmError::RateLimited { retry_after_secs: 10 });
            }
            return Err(LlmError::ProviderError {
                code: status.to_string(),
                message: error_body,
            });
        }

        let stream = response.bytes_stream();
        let mut accumulated = String::new();

        let mapped = stream.map(move |chunk_result| match chunk_result {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                for line in text.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            return Ok(StreamChunk {
                                delta: String::new(),
                                accumulated: accumulated.clone(),
                                is_final: true,
                                usage: None,
                            });
                        }
                        if let Ok(chunk) = serde_json::from_str::<TogetherStreamChunk>(data) {
                            if let Some(choice) = chunk.choices.first() {
                                if let Some(content) = &choice.delta.content {
                                    accumulated.push_str(content);
                                    return Ok(StreamChunk {
                                        delta: content.clone(),
                                        accumulated: accumulated.clone(),
                                        is_final: choice.finish_reason.is_some(),
                                        usage: None,
                                    });
                                }
                            }
                        }
                    }
                }
                Ok(StreamChunk {
                    delta: String::new(),
                    accumulated: accumulated.clone(),
                    is_final: false,
                    usage: None,
                })
            }
            Err(e) => Err(LlmError::Network(e.to_string())),
        });

        Ok(Box::pin(mapped))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(self.info.models.clone());
        }

        // Together returns a list of models
        let api_response: TogetherModelsResponse = response.json().await?;
        Ok(api_response.data.into_iter().take(20).map(|m| ModelInfo {
            id: m.id.clone(),
            name: m.display_name.unwrap_or_else(|| m.id.clone()),
            context_window: m.context_length.unwrap_or(4096) as u32,
            max_output_tokens: Some(4096),
            supports_vision: Self::model_supports_vision(&m.id),
        }).collect())
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

// OpenAI-compatible API types (shared with other providers)

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

#[derive(Debug, Deserialize)]
struct TogetherModelsResponse {
    data: Vec<TogetherModel>,
}

#[derive(Debug, Deserialize)]
struct TogetherModel {
    id: String,
    display_name: Option<String>,
    context_length: Option<i64>,
}

// Streaming response structures (OpenAI-compatible)
#[derive(Debug, Deserialize)]
struct TogetherStreamChunk {
    choices: Vec<TogetherStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct TogetherStreamChoice {
    delta: TogetherDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TogetherDelta {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let provider = TogetherProvider::new("test-key", None);
        let info = provider.info();
        assert_eq!(info.name, "Together AI");
        assert!(!info.models.is_empty());
        assert!(info.supports_tools);
    }

    #[test]
    fn test_build_request() {
        let provider = TogetherProvider::new("test-key", None);
        let request = CompletionRequest::new("meta-llama/Llama-3.3-70B-Instruct-Turbo")
            .system("You are helpful.")
            .message(Message::user("Hello"));

        let api_req = provider.build_request(&request);
        assert!(api_req.model.contains("Llama"));
        assert_eq!(api_req.messages.len(), 2);
    }

    #[test]
    fn test_custom_base_url() {
        let provider = TogetherProvider::new("key", Some("https://custom.api.com".to_string()));
        assert_eq!(provider.base_url, "https://custom.api.com");
    }

    #[test]
    fn test_default_base_url() {
        let provider = TogetherProvider::new("key", None);
        assert!(provider.base_url.contains("together.xyz"));
    }

    #[test]
    fn test_vision_model_detection() {
        assert!(TogetherProvider::model_supports_vision("meta-llama/Llama-3.2-90B-Vision-Instruct-Turbo"));
        assert!(TogetherProvider::model_supports_vision("meta-llama/Llama-3.2-11B-Vision-Instruct-Turbo"));
        assert!(!TogetherProvider::model_supports_vision("meta-llama/Llama-3.3-70B-Instruct-Turbo"));
    }

    #[test]
    fn test_llama_models() {
        let provider = TogetherProvider::new("key", None);
        let info = provider.info();
        let llama: Vec<_> = info.models.iter().filter(|m| m.id.contains("llama") || m.id.contains("Llama")).collect();
        assert!(llama.len() >= 3);
    }

    #[test]
    fn test_mixtral_models() {
        let provider = TogetherProvider::new("key", None);
        let info = provider.info();
        let mixtral: Vec<_> = info.models.iter().filter(|m| m.id.contains("Mixtral")).collect();
        assert!(!mixtral.is_empty());
    }
}
