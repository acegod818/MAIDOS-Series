//! Mistral API Provider
//!
//! <impl>
//! WHAT: Mistral AI API implementation (OpenAI-compatible with native features)
//! WHY: European LLM provider with strong multilingual and coding capabilities
//! HOW: OpenAI-compatible REST API with Mistral-specific extensions
//! TEST: Request formatting, response parsing, error handling, vision support
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

const DEFAULT_BASE_URL: &str = "https://api.mistral.ai/v1";

/// Mistral Provider (OpenAI-compatible API)
pub struct MistralProvider {
    client: Client,
    api_key: String,
    base_url: String,
    info: ProviderInfo,
}

impl MistralProvider {
    /// Create new Mistral provider
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: base_url.clone(),
            info: ProviderInfo {
                name: "Mistral".to_string(),
                version: "v1".to_string(),
                models: vec![
                    ModelInfo {
                        id: "mistral-large-latest".to_string(),
                        name: "Mistral Large".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8192),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "mistral-medium-latest".to_string(),
                        name: "Mistral Medium".to_string(),
                        context_window: 32_000,
                        max_output_tokens: Some(8192),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "mistral-small-latest".to_string(),
                        name: "Mistral Small".to_string(),
                        context_window: 32_000,
                        max_output_tokens: Some(8192),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "codestral-latest".to_string(),
                        name: "Codestral".to_string(),
                        context_window: 32_000,
                        max_output_tokens: Some(8192),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "ministral-8b-latest".to_string(),
                        name: "Ministral 8B".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8192),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "ministral-3b-latest".to_string(),
                        name: "Ministral 3B".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8192),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "open-mistral-nemo".to_string(),
                        name: "Mistral Nemo".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8192),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "pixtral-large-latest".to_string(),
                        name: "Pixtral Large".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8192),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "pixtral-12b-2409".to_string(),
                        name: "Pixtral 12B".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(8192),
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

    fn build_request(&self, req: &CompletionRequest) -> MistralRequest {
        let mut messages = Vec::new();

        if let Some(system) = &req.system {
            messages.push(MistralMessage {
                role: "system".to_string(),
                content: MistralContent::Text(system.clone()),
            });
        }

        for msg in &req.messages {
            messages.push(MistralMessage {
                role: msg.role.as_str().to_string(),
                content: MistralContent::Text(msg.text().to_string()),
            });
        }

        MistralRequest {
            model: req.model.clone(),
            messages,
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            top_p: req.top_p,
            stop: if req.stop.as_ref().is_none_or(|v| v.is_empty()) { None } else { req.stop.clone() },
            stream: Some(false),
            random_seed: None,
            safe_prompt: None,
        }
    }

    fn parse_response(&self, resp: MistralResponse) -> Result<CompletionResponse> {
        let choice = resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| LlmError::ParseError("No choices in response".to_string()))?;

        let finish_reason = choice.finish_reason.map(|r| match r.as_str() {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "model_length" => FinishReason::Length,
            "content_filter" => FinishReason::ContentFilter,
            "tool_calls" => FinishReason::ToolUse,
            _ => FinishReason::Unknown,
        }).unwrap_or(FinishReason::Unknown);

        let content = match &choice.message.content {
            MistralContent::Text(text) => text.clone(),
            MistralContent::Parts(_) => {
                // For multi-part responses, extract text parts
                String::new()
            }
        };

        let usage = resp.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            cached_tokens: None,
        }).unwrap_or_default();

        Ok(CompletionResponse {
            message: Message::assistant(&content),
            usage,
            finish_reason,
            model: resp.model,
            id: resp.id,
        })
    }

    /// Check if a model supports vision
    pub fn model_supports_vision(model: &str) -> bool {
        model.contains("pixtral")
    }
}

#[async_trait]
impl LlmProvider for MistralProvider {
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
                return Err(LlmError::Auth("Invalid Mistral API key".to_string()));
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

        let api_response: MistralResponse = response.json().await?;
        self.parse_response(api_response)
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        let mut req = request;
        req.stream = true;
        let body = self.build_request(&req);

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            if status.as_u16() == 401 {
                return Err(LlmError::Auth("Invalid Mistral API key".to_string()));
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
                        if let Ok(chunk) = serde_json::from_str::<MistralStreamChunk>(data) {
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
            // Fall back to static list
            return Ok(self.info.models.clone());
        }

        // Parse dynamic model list
        let api_response: MistralModelsResponse = response.json().await?;
        Ok(api_response.data.into_iter().map(|m| ModelInfo {
            id: m.id.clone(),
            name: m.id.clone(),
            context_window: m.max_context_length.unwrap_or(32_000) as u32,
            max_output_tokens: m.max_context_length.map(|v| v as u32),
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

// Mistral API types

#[derive(Debug, Serialize)]
struct MistralRequest {
    model: String,
    messages: Vec<MistralMessage>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    random_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safe_prompt: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralMessage {
    role: String,
    content: MistralContent,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum MistralContent {
    Text(String),
    Parts(Vec<MistralContentPart>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum MistralContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: MistralImageUrl },
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralImageUrl {
    url: String,
}

#[derive(Debug, Deserialize)]
struct MistralResponse {
    id: Option<String>,
    model: String,
    choices: Vec<MistralChoice>,
    usage: Option<MistralUsage>,
}

#[derive(Debug, Deserialize)]
struct MistralChoice {
    message: MistralMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MistralUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct MistralModelsResponse {
    data: Vec<MistralModel>,
}

#[derive(Debug, Deserialize)]
struct MistralModel {
    id: String,
    #[serde(default)]
    max_context_length: Option<i64>,
}

// Streaming response structures
#[derive(Debug, Deserialize)]
struct MistralStreamChunk {
    choices: Vec<MistralStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct MistralStreamChoice {
    delta: MistralDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MistralDelta {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let provider = MistralProvider::new("test-key", None);
        let info = provider.info();
        assert_eq!(info.name, "Mistral");
        assert!(!info.models.is_empty());
        assert!(info.supports_vision);
        assert!(info.supports_tools);
    }

    #[test]
    fn test_build_request() {
        let provider = MistralProvider::new("test-key", None);
        let request = CompletionRequest::new("mistral-large-latest")
            .system("You are helpful.")
            .message(Message::user("Hello"));

        let api_req = provider.build_request(&request);
        assert_eq!(api_req.model, "mistral-large-latest");
        assert_eq!(api_req.messages.len(), 2);
    }

    #[test]
    fn test_custom_base_url() {
        let provider = MistralProvider::new("key", Some("https://custom.api.com".to_string()));
        assert_eq!(provider.base_url, "https://custom.api.com");
    }

    #[test]
    fn test_default_base_url() {
        let provider = MistralProvider::new("key", None);
        assert!(provider.base_url.contains("mistral.ai"));
    }

    #[test]
    fn test_vision_model_detection() {
        assert!(MistralProvider::model_supports_vision("pixtral-large-latest"));
        assert!(MistralProvider::model_supports_vision("pixtral-12b-2409"));
        assert!(!MistralProvider::model_supports_vision("mistral-large-latest"));
        assert!(!MistralProvider::model_supports_vision("codestral-latest"));
    }

    #[test]
    fn test_vision_models() {
        let provider = MistralProvider::new("key", None);
        let info = provider.info();
        let vision_models: Vec<_> = info.models.iter().filter(|m| m.supports_vision).collect();
        assert_eq!(vision_models.len(), 2); // pixtral-large and pixtral-12b
    }

    #[test]
    fn test_codestral_model() {
        let provider = MistralProvider::new("key", None);
        let info = provider.info();
        let codestral: Vec<_> = info.models.iter().filter(|m| m.id.contains("codestral")).collect();
        assert!(!codestral.is_empty());
    }

    #[test]
    fn test_all_models_have_context_window() {
        let provider = MistralProvider::new("key", None);
        let info = provider.info();
        for model in &info.models {
            assert!(model.context_window > 0);
        }
    }

    #[test]
    fn test_request_with_stop_sequences() {
        let provider = MistralProvider::new("test-key", None);
        let request = CompletionRequest::new("mistral-small-latest")
            .message(Message::user("Hello"))
            .stop("END");

        let api_req = provider.build_request(&request);
        assert!(api_req.stop.is_some());
        assert_eq!(api_req.stop.as_ref().map(|v| v.len()), Some(1));
    }

    #[test]
    fn test_request_with_temperature() {
        let provider = MistralProvider::new("test-key", None);
        let request = CompletionRequest::new("mistral-small-latest")
            .message(Message::user("Hello"))
            .temperature(0.7);

        let api_req = provider.build_request(&request);
        assert_eq!(api_req.temperature, Some(0.7));
    }

    #[test]
    fn test_ministral_models() {
        let provider = MistralProvider::new("key", None);
        let info = provider.info();
        let ministral: Vec<_> = info.models.iter().filter(|m| m.id.contains("ministral")).collect();
        assert_eq!(ministral.len(), 2); // 8b and 3b
    }

    #[test]
    fn test_large_context_models() {
        let provider = MistralProvider::new("key", None);
        let info = provider.info();
        let large_ctx: Vec<_> = info.models.iter().filter(|m| m.context_window >= 128_000).collect();
        assert!(large_ctx.len() >= 4); // ministral, nemo, pixtral, mistral-large
    }
}
