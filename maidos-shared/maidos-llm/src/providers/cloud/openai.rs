//! OpenAI API Provider
//!
//! <impl>
//! WHAT: OpenAI API implementation (GPT-4, GPT-4o, etc.)
//! WHY: Support OpenAI models through unified interface
//! HOW: reqwest HTTP client, JSON serialization, streaming SSE
//! TEST: Request formatting, response parsing, error handling
//! </impl>

use crate::error::{LlmError, Result};
use crate::message::{CompletionResponse, Content, FinishReason, Message, Usage};
use crate::provider::{
    CompletionRequest, CompletionStream, LlmProvider, ModelInfo, ProviderInfo, StreamChunk,
};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

/// OpenAI Provider
pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    info: ProviderInfo,
}

impl OpenAiProvider {
    /// Create new OpenAI provider
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: base_url.clone(),
            info: ProviderInfo {
                name: "OpenAI".to_string(),
                version: "v1".to_string(),
                models: vec![
                    ModelInfo {
                        id: "gpt-4o".to_string(),
                        name: "GPT-4o".to_string(),
                        context_window: 128000,
                        max_output_tokens: Some(16384),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "gpt-4o-mini".to_string(),
                        name: "GPT-4o Mini".to_string(),
                        context_window: 128000,
                        max_output_tokens: Some(16384),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "gpt-4-turbo".to_string(),
                        name: "GPT-4 Turbo".to_string(),
                        context_window: 128000,
                        max_output_tokens: Some(4096),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "gpt-3.5-turbo".to_string(),
                        name: "GPT-3.5 Turbo".to_string(),
                        context_window: 16385,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                ],
                base_url,
                supports_streaming: true,
                supports_vision: true,
                supports_tools: true,
            },
        }
    }

    /// Convert our Message to OpenAI format
    fn to_openai_message(msg: &Message) -> OpenAiMessage {
        let content = if msg.content.len() == 1 && msg.content[0].is_text() {
            OpenAiContent::Text(msg.content[0].as_text().unwrap_or("").to_string())
        } else {
            OpenAiContent::Parts(
                msg.content
                    .iter()
                    .map(|c| match c {
                        Content::Text { text } => OpenAiContentPart::Text { text: text.clone() },
                        Content::Image { url, base64, media_type } => {
                            if let Some(url) = url {
                                OpenAiContentPart::ImageUrl {
                                    image_url: ImageUrl {
                                        url: url.clone(),
                                        detail: None,
                                    },
                                }
                            } else if let Some(b64) = base64 {
                                let mt = media_type.as_deref().unwrap_or("image/png");
                                OpenAiContentPart::ImageUrl {
                                    image_url: ImageUrl {
                                        url: format!("data:{};base64,{}", mt, b64),
                                        detail: None,
                                    },
                                }
                            } else {
                                OpenAiContentPart::Text { text: "[invalid image]".to_string() }
                            }
                        }
                        _ => OpenAiContentPart::Text { text: "[unsupported content]".to_string() },
                    })
                    .collect(),
            )
        };

        OpenAiMessage {
            role: msg.role.as_str().to_string(),
            content,
            name: msg.name.clone(),
        }
    }

    /// Build OpenAI request body
    fn build_request(&self, req: &CompletionRequest) -> OpenAiRequest {
        let mut messages = Vec::new();

        // Add system message if provided
        if let Some(system) = &req.system {
            messages.push(OpenAiMessage {
                role: "system".to_string(),
                content: OpenAiContent::Text(system.clone()),
                name: None,
            });
        }

        // Add conversation messages
        for msg in &req.messages {
            messages.push(Self::to_openai_message(msg));
        }

        OpenAiRequest {
            model: req.model.clone(),
            messages,
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            top_p: req.top_p,
            stop: req.stop.clone(),
            stream: Some(req.stream),
        }
    }

    /// Parse OpenAI response
    fn parse_response(&self, resp: OpenAiResponse) -> Result<CompletionResponse> {
        let choice = resp.choices.into_iter().next().ok_or_else(|| {
            LlmError::ParseError("No choices in response".to_string())
        })?;

        let text = match choice.message.content {
            Some(OpenAiContent::Text(t)) => t,
            Some(OpenAiContent::Parts(parts)) => {
                parts
                    .iter()
                    .filter_map(|p| match p {
                        OpenAiContentPart::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            None => String::new(),
        };

        let finish_reason = match choice.finish_reason.as_deref() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("content_filter") => FinishReason::ContentFilter,
            Some("tool_calls") | Some("function_call") => FinishReason::ToolUse,
            _ => FinishReason::Unknown,
        };

        Ok(CompletionResponse {
            message: Message::assistant(text),
            usage: Usage {
                prompt_tokens: resp.usage.prompt_tokens,
                completion_tokens: resp.usage.completion_tokens,
                total_tokens: resp.usage.total_tokens,
                cached_tokens: None,
            },
            finish_reason,
            model: resp.model,
            id: Some(resp.id),
        })
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn info(&self) -> &ProviderInfo {
        &self.info
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let body = self.build_request(&request);
        debug!("OpenAI request: {:?}", body.model);

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
            return Err(Self::parse_error(status.as_u16(), &error_body));
        }

        let resp: OpenAiResponse = response.json().await?;
        self.parse_response(resp)
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
            return Err(Self::parse_error(status.as_u16(), &error_body));
        }

        let stream = response.bytes_stream();
        let mut accumulated = String::new();

        let mapped = stream.map(move |chunk_result| {
            match chunk_result {
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
                            if let Ok(chunk) = serde_json::from_str::<OpenAiStreamChunk>(data) {
                                if let Some(choice) = chunk.choices.first() {
                                    if let Some(content) = &choice.delta.content {
                                        accumulated.push_str(content);
                                        return Ok(StreamChunk {
                                            delta: content.clone(),
                                            accumulated: accumulated.clone(),
                                            is_final: false,
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
            }
        });

        Ok(Box::pin(mapped))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(self.info.models.clone())
    }

    async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

impl OpenAiProvider {
    fn parse_error(status: u16, body: &str) -> LlmError {
        if let Ok(err) = serde_json::from_str::<OpenAiErrorResponse>(body) {
            match status {
                401 => LlmError::Auth(err.error.message),
                429 => LlmError::RateLimited { retry_after_secs: 60 },
                400 => LlmError::InvalidRequest(err.error.message),
                _ => LlmError::ProviderError {
                    code: err.error.code.unwrap_or_else(|| status.to_string()),
                    message: err.error.message,
                },
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
// OpenAI API Types
// ============================================================================

#[derive(Debug, Serialize)]
struct OpenAiRequest {
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

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: OpenAiContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum OpenAiContent {
    Text(String),
    Parts(Vec<OpenAiContentPart>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum OpenAiContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Serialize, Deserialize)]
struct ImageUrl {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    id: String,
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseMessage {
    content: Option<OpenAiContent>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiDelta,
}

#[derive(Debug, Deserialize)]
struct OpenAiDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiErrorResponse {
    error: OpenAiError,
}

#[derive(Debug, Deserialize)]
struct OpenAiError {
    message: String,
    code: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Role;

    #[test]
    fn test_provider_info() {
        let provider = OpenAiProvider::new("test-key", None);
        let info = provider.info();
        assert_eq!(info.name, "OpenAI");
        assert!(info.supports_streaming);
        assert!(info.supports_vision);
    }

    #[test]
    fn test_build_request() {
        let provider = OpenAiProvider::new("test-key", None);
        let req = CompletionRequest::new("gpt-4o")
            .system("You are helpful")
            .message(Message::user("Hello"))
            .max_tokens(100)
            .temperature(0.5);

        let openai_req = provider.build_request(&req);
        assert_eq!(openai_req.model, "gpt-4o");
        assert_eq!(openai_req.messages.len(), 2); // system + user
        assert_eq!(openai_req.max_tokens, Some(100));
    }

    #[test]
    fn test_to_openai_message_text() {
        let msg = Message::user("Hello world");
        let openai_msg = OpenAiProvider::to_openai_message(&msg);
        assert_eq!(openai_msg.role, "user");
        assert!(matches!(openai_msg.content, OpenAiContent::Text(t) if t == "Hello world"));
    }

    #[test]
    fn test_to_openai_message_multipart() {
        let msg = Message::with_content(
            Role::User,
            vec![
                Content::text("Look at this:"),
                Content::image_url("https://example.com/img.png"),
            ],
        );
        let openai_msg = OpenAiProvider::to_openai_message(&msg);
        assert!(matches!(openai_msg.content, OpenAiContent::Parts(parts) if parts.len() == 2));
    }

    #[test]
    fn test_parse_error_401() {
        let body = r#"{"error":{"message":"Invalid API key","code":"invalid_api_key"}}"#;
        let err = OpenAiProvider::parse_error(401, body);
        assert!(matches!(err, LlmError::Auth(_)));
    }

    #[test]
    fn test_parse_error_429() {
        let body = r#"{"error":{"message":"Rate limit exceeded"}}"#;
        let err = OpenAiProvider::parse_error(429, body);
        assert!(matches!(err, LlmError::RateLimited { .. }));
    }

    #[test]
    fn test_custom_base_url() {
        let provider = OpenAiProvider::new("key", Some("https://custom.api.com".to_string()));
        assert_eq!(provider.info().base_url, "https://custom.api.com");
    }
}
