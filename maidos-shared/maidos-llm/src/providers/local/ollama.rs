//! Ollama Local LLM Provider
//!
//! <impl>
//! WHAT: Ollama API implementation for local LLM inference
//! WHY: Support local/offline LLM usage without API keys
//! HOW: reqwest HTTP client to local Ollama server
//! TEST: Request formatting, response parsing, health check
//! </impl>

use crate::error::{LlmError, Result};
use crate::message::{CompletionResponse, Content, FinishReason, Message, Role, Usage};
use crate::provider::{
    CompletionRequest, CompletionStream, LlmProvider, ModelInfo, ProviderInfo, StreamChunk,
};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

const DEFAULT_BASE_URL: &str = "http://localhost:11434";

/// Ollama Provider for local LLM
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    info: ProviderInfo,
}

impl OllamaProvider {
    /// Create new Ollama provider
    pub fn new(base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Self {
            client: Client::new(),
            base_url: base_url.clone(),
            info: ProviderInfo {
                name: "Ollama".to_string(),
                version: "local".to_string(),
                models: vec![
                    ModelInfo {
                        id: "llama3.2".to_string(),
                        name: "Llama 3.2".to_string(),
                        context_window: 128000,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "llama3.2-vision".to_string(),
                        name: "Llama 3.2 Vision".to_string(),
                        context_window: 128000,
                        max_output_tokens: Some(4096),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "mistral".to_string(),
                        name: "Mistral".to_string(),
                        context_window: 32768,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "codellama".to_string(),
                        name: "Code Llama".to_string(),
                        context_window: 16384,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "qwen2.5-coder".to_string(),
                        name: "Qwen 2.5 Coder".to_string(),
                        context_window: 32768,
                        max_output_tokens: Some(8192),
                        supports_vision: false,
                    },
                ],
                base_url,
                supports_streaming: true,
                supports_vision: true,
                supports_tools: false,
            },
        }
    }

    /// Build Ollama request body
    fn build_request(&self, req: &CompletionRequest) -> OllamaRequest {
        let mut messages = Vec::new();

        // Add system message if provided
        if let Some(system) = &req.system {
            messages.push(OllamaMessage {
                role: "system".to_string(),
                content: system.clone(),
                images: None,
            });
        }

        // Add conversation messages
        for msg in &req.messages {
            let (content, images) = Self::extract_content_and_images(msg);
            messages.push(OllamaMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "user".to_string(),
                },
                content,
                images,
            });
        }

        OllamaRequest {
            model: req.model.clone(),
            messages,
            stream: Some(req.stream),
            options: Some(OllamaOptions {
                temperature: req.temperature,
                top_p: req.top_p,
                num_predict: req.max_tokens.map(|n| n as i32),
                stop: req.stop.clone(),
            }),
        }
    }

    /// Extract text content and images from message
    fn extract_content_and_images(msg: &Message) -> (String, Option<Vec<String>>) {
        let mut text_parts = Vec::new();
        let mut images = Vec::new();

        for content in &msg.content {
            match content {
                Content::Text { text } => text_parts.push(text.clone()),
                Content::Image { base64: Some(b64), .. } => {
                    images.push(b64.clone());
                }
                _ => {}
            }
        }

        let images = if images.is_empty() { None } else { Some(images) };
        (text_parts.join("\n"), images)
    }

    /// Parse Ollama response
    fn parse_response(&self, resp: OllamaResponse) -> Result<CompletionResponse> {
        let text = resp.message.content;

        // Ollama doesn't provide detailed usage, estimate from response
        let completion_tokens = (text.len() / 4) as u32; // rough estimate
        let prompt_tokens = resp.prompt_eval_count.unwrap_or(0);

        Ok(CompletionResponse {
            message: Message::assistant(text),
            usage: Usage {
                prompt_tokens,
                completion_tokens: resp.eval_count.unwrap_or(completion_tokens),
                total_tokens: prompt_tokens + resp.eval_count.unwrap_or(completion_tokens),
                cached_tokens: None,
            },
            finish_reason: if resp.done {
                FinishReason::Stop
            } else {
                FinishReason::Unknown
            },
            model: resp.model,
            id: None,
        })
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn info(&self) -> &ProviderInfo {
        &self.info
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let body = self.build_request(&request);
        debug!("Ollama request: {:?}", body.model);

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    LlmError::ConnectionFailed(format!(
                        "Cannot connect to Ollama at {}. Is Ollama running?",
                        self.base_url
                    ))
                } else {
                    LlmError::from(e)
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(Self::parse_error(status.as_u16(), &error_body));
        }

        let resp: OllamaResponse = response.json().await?;
        self.parse_response(resp)
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        let mut req = request;
        req.stream = true;
        let body = self.build_request(&req);

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    LlmError::ConnectionFailed(format!(
                        "Cannot connect to Ollama at {}",
                        self.base_url
                    ))
                } else {
                    LlmError::from(e)
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(Self::parse_error(status.as_u16(), &error_body));
        }

        let stream = response.bytes_stream();
        let mut accumulated = String::new();

        let mapped = stream.map(move |chunk_result| {
            let chunk_result: std::result::Result<bytes::Bytes, reqwest::Error> = chunk_result;
            match chunk_result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    // Ollama sends newline-delimited JSON
                    for line in text.lines() {
                        if let Ok(chunk) = serde_json::from_str::<OllamaStreamChunk>(line) {
                            let delta = chunk.message.content;
                            accumulated.push_str(&delta);
                            return Ok(StreamChunk {
                                delta,
                                accumulated: accumulated.clone(),
                                is_final: chunk.done,
                                usage: if chunk.done {
                                    Some(Usage::new(
                                        chunk.prompt_eval_count.unwrap_or(0),
                                        chunk.eval_count.unwrap_or(0),
                                    ))
                                } else {
                                    None
                                },
                            });
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
            }
        });

        Ok(Box::pin(mapped))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await?;

        if !response.status().is_success() {
            // Fall back to static list
            return Ok(self.info.models.clone());
        }

        let resp: OllamaTagsResponse = response.json().await?;
        let models = resp
            .models
            .into_iter()
            .map(|m| ModelInfo {
                id: m.name.clone(),
                name: m.name,
                context_window: 4096, // Ollama doesn't report this
                max_output_tokens: Some(4096),
                supports_vision: false,
            })
            .collect();

        Ok(models)
    }

    async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl OllamaProvider {
    fn parse_error(status: u16, body: &str) -> LlmError {
        if let Ok(err) = serde_json::from_str::<OllamaErrorResponse>(body) {
            LlmError::ProviderError {
                code: status.to_string(),
                message: err.error,
            }
        } else {
            LlmError::ProviderError {
                code: status.to_string(),
                message: body.to_string(),
            }
        }
    }
}

// ============================================================================
// Ollama API Types
// ============================================================================

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    model: String,
    message: OllamaResponseMessage,
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaStreamChunk {
    message: OllamaResponseMessage,
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OllamaErrorResponse {
    error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let provider = OllamaProvider::new(None);
        let info = provider.info();
        assert_eq!(info.name, "Ollama");
        assert!(info.supports_streaming);
        assert_eq!(info.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn test_custom_base_url() {
        let provider = OllamaProvider::new(Some("http://remote:11434".to_string()));
        assert_eq!(provider.info().base_url, "http://remote:11434");
    }

    #[test]
    fn test_build_request() {
        let provider = OllamaProvider::new(None);
        let req = CompletionRequest::new("llama3.2")
            .system("You are helpful")
            .message(Message::user("Hello"))
            .max_tokens(100)
            .temperature(0.5);

        let ollama_req = provider.build_request(&req);
        assert_eq!(ollama_req.model, "llama3.2");
        assert_eq!(ollama_req.messages.len(), 2); // system + user
        assert!(ollama_req.options.is_some());
    }

    #[test]
    fn test_extract_content_text_only() {
        let msg = Message::user("Hello world");
        let (content, images) = OllamaProvider::extract_content_and_images(&msg);
        assert_eq!(content, "Hello world");
        assert!(images.is_none());
    }

    #[test]
    fn test_extract_content_with_image() {
        let msg = Message::with_content(
            Role::User,
            vec![
                Content::text("Look at this:"),
                Content::image_base64("abc123", "image/png"),
            ],
        );
        let (content, images) = OllamaProvider::extract_content_and_images(&msg);
        assert_eq!(content, "Look at this:");
        assert_eq!(images, Some(vec!["abc123".to_string()]));
    }

    #[test]
    fn test_no_api_key_required() {
        // Ollama works without API key
        let provider = OllamaProvider::new(None);
        assert!(provider.info().name == "Ollama");
    }
}
