//! Anthropic API Provider
//!
//! <impl>
//! WHAT: Anthropic API implementation (Claude models)
//! WHY: Support Claude models through unified interface
//! HOW: reqwest HTTP client, JSON serialization, streaming SSE
//! TEST: Request formatting, response parsing, error handling
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

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const API_VERSION: &str = "2023-06-01";

/// Anthropic Provider
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
    info: ProviderInfo,
}

impl AnthropicProvider {
    /// Create new Anthropic provider
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: base_url.clone(),
            info: ProviderInfo {
                name: "Anthropic".to_string(),
                version: API_VERSION.to_string(),
                models: vec![
                    ModelInfo {
                        id: "claude-sonnet-4-20250514".to_string(),
                        name: "Claude Sonnet 4".to_string(),
                        context_window: 200000,
                        max_output_tokens: Some(64000),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "claude-3-5-sonnet-20241022".to_string(),
                        name: "Claude 3.5 Sonnet".to_string(),
                        context_window: 200000,
                        max_output_tokens: Some(8192),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "claude-3-5-haiku-20241022".to_string(),
                        name: "Claude 3.5 Haiku".to_string(),
                        context_window: 200000,
                        max_output_tokens: Some(8192),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "claude-3-opus-20240229".to_string(),
                        name: "Claude 3 Opus".to_string(),
                        context_window: 200000,
                        max_output_tokens: Some(4096),
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

    /// Convert our Content to Anthropic format
    fn to_anthropic_content(content: &Content) -> AnthropicContent {
        match content {
            Content::Text { text } => AnthropicContent::Text { text: text.clone() },
            Content::Image { url, base64, media_type } => {
                if let Some(b64) = base64 {
                    AnthropicContent::Image {
                        source: ImageSource {
                            source_type: "base64".to_string(),
                            media_type: media_type.clone().unwrap_or_else(|| "image/png".to_string()),
                            data: b64.clone(),
                        },
                    }
                } else if let Some(url) = url {
                    // Anthropic doesn't support URL directly, would need to fetch
                    AnthropicContent::Text {
                        text: format!("[Image URL: {}]", url),
                    }
                } else {
                    AnthropicContent::Text {
                        text: "[Invalid image]".to_string(),
                    }
                }
            }
            Content::ToolUse { id, name, input } => AnthropicContent::ToolUse {
                id: id.clone(),
                name: name.clone(),
                input: input.clone(),
            },
            Content::ToolResult { tool_use_id, content, is_error } => AnthropicContent::ToolResult {
                tool_use_id: tool_use_id.clone(),
                content: content.clone(),
                is_error: *is_error,
            },
        }
    }

    /// Convert our Message to Anthropic format
    fn to_anthropic_message(msg: &Message) -> AnthropicMessage {
        AnthropicMessage {
            role: match msg.role {
                Role::User | Role::Tool => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
                Role::System => "user".to_string(), // System handled separately
            },
            content: msg.content.iter().map(Self::to_anthropic_content).collect(),
        }
    }

    /// Build Anthropic request body
    fn build_request(&self, req: &CompletionRequest) -> AnthropicRequest {
        // Extract system prompt
        let system = req.system.clone().or_else(|| {
            req.messages
                .iter()
                .find(|m| m.role == Role::System)
                .map(|m| m.text())
        });

        // Filter out system messages (handled separately)
        let messages: Vec<AnthropicMessage> = req
            .messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(Self::to_anthropic_message)
            .collect();

        AnthropicRequest {
            model: req.model.clone(),
            messages,
            system,
            max_tokens: req.max_tokens.unwrap_or(4096),
            temperature: req.temperature,
            top_p: req.top_p,
            stop_sequences: req.stop.clone(),
            stream: Some(req.stream),
        }
    }

    /// Parse Anthropic response
    fn parse_response(&self, resp: AnthropicResponse) -> Result<CompletionResponse> {
        let text = resp
            .content
            .iter()
            .filter_map(|c| match c {
                AnthropicResponseContent::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let finish_reason = match resp.stop_reason.as_deref() {
            Some("end_turn") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("tool_use") => FinishReason::ToolUse,
            _ => FinishReason::Unknown,
        };

        Ok(CompletionResponse {
            message: Message::assistant(text),
            usage: Usage {
                prompt_tokens: resp.usage.input_tokens,
                completion_tokens: resp.usage.output_tokens,
                total_tokens: resp.usage.input_tokens + resp.usage.output_tokens,
                cached_tokens: resp.usage.cache_read_input_tokens,
            },
            finish_reason,
            model: resp.model,
            id: Some(resp.id),
        })
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn info(&self) -> &ProviderInfo {
        &self.info
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let body = self.build_request(&request);
        debug!("Anthropic request: {:?}", body.model);

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(Self::parse_error(status.as_u16(), &error_body));
        }

        let resp: AnthropicResponse = response.json().await?;
        self.parse_response(resp)
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        let mut req = request;
        req.stream = true;
        let body = self.build_request(&req);

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
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
                            if let Ok(event) = serde_json::from_str::<AnthropicStreamEvent>(data) {
                                match event {
                                    AnthropicStreamEvent::ContentBlockDelta { delta, .. } => {
                                        if let Some(text) = delta.text {
                                            accumulated.push_str(&text);
                                            return Ok(StreamChunk {
                                                delta: text,
                                                accumulated: accumulated.clone(),
                                                is_final: false,
                                                usage: None,
                                            });
                                        }
                                    }
                                    AnthropicStreamEvent::MessageStop => {
                                        return Ok(StreamChunk {
                                            delta: String::new(),
                                            accumulated: accumulated.clone(),
                                            is_final: true,
                                            usage: None,
                                        });
                                    }
                                    _ => {}
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
        // Anthropic doesn't have a dedicated health endpoint
        // We'll just verify the API key format
        Ok(!self.api_key.is_empty())
    }
}

impl AnthropicProvider {
    fn parse_error(status: u16, body: &str) -> LlmError {
        if let Ok(err) = serde_json::from_str::<AnthropicErrorResponse>(body) {
            match err.error.error_type.as_str() {
                "authentication_error" => LlmError::Auth(err.error.message),
                "rate_limit_error" => LlmError::RateLimited { retry_after_secs: 60 },
                "invalid_request_error" => LlmError::InvalidRequest(err.error.message),
                "overloaded_error" => LlmError::RateLimited { retry_after_secs: 30 },
                _ => LlmError::ProviderError {
                    code: err.error.error_type,
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
// Anthropic API Types
// ============================================================================

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum AnthropicContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

#[derive(Debug, Serialize)]
struct ImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    model: String,
    content: Vec<AnthropicResponseContent>,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AnthropicResponseContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
    #[serde(default)]
    cache_read_input_tokens: Option<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AnthropicStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: serde_json::Value },
    #[serde(rename = "content_block_start")]
    ContentBlockStart { index: u32 },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: u32, delta: StreamDelta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: u32 },
    #[serde(rename = "message_delta")]
    MessageDelta { delta: serde_json::Value },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicErrorResponse {
    error: AnthropicError,
}

#[derive(Debug, Deserialize)]
struct AnthropicError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let provider = AnthropicProvider::new("test-key", None);
        let info = provider.info();
        assert_eq!(info.name, "Anthropic");
        assert!(info.supports_streaming);
        assert!(info.supports_vision);
    }

    #[test]
    fn test_build_request_with_system() {
        let provider = AnthropicProvider::new("test-key", None);
        let req = CompletionRequest::new("claude-sonnet-4-20250514")
            .system("You are helpful")
            .message(Message::user("Hello"))
            .max_tokens(100);

        let anthropic_req = provider.build_request(&req);
        assert_eq!(anthropic_req.model, "claude-sonnet-4-20250514");
        assert_eq!(anthropic_req.system, Some("You are helpful".to_string()));
        assert_eq!(anthropic_req.messages.len(), 1); // Only user message
        assert_eq!(anthropic_req.max_tokens, 100);
    }

    #[test]
    fn test_to_anthropic_content_text() {
        let content = Content::text("Hello");
        let anthropic = AnthropicProvider::to_anthropic_content(&content);
        assert!(matches!(anthropic, AnthropicContent::Text { text } if text == "Hello"));
    }

    #[test]
    fn test_to_anthropic_content_image_base64() {
        let content = Content::image_base64("abc123", "image/png");
        let anthropic = AnthropicProvider::to_anthropic_content(&content);
        assert!(matches!(anthropic, AnthropicContent::Image { source } if source.data == "abc123"));
    }

    #[test]
    fn test_parse_error_auth() {
        let body = r#"{"type":"error","error":{"type":"authentication_error","message":"Invalid key"}}"#;
        let err = AnthropicProvider::parse_error(401, body);
        assert!(matches!(err, LlmError::Auth(_)));
    }

    #[test]
    fn test_parse_error_rate_limit() {
        let body = r#"{"type":"error","error":{"type":"rate_limit_error","message":"Too many requests"}}"#;
        let err = AnthropicProvider::parse_error(429, body);
        assert!(matches!(err, LlmError::RateLimited { .. }));
    }

    #[test]
    fn test_custom_base_url() {
        let provider = AnthropicProvider::new("key", Some("https://custom.api.com".to_string()));
        assert_eq!(provider.info().base_url, "https://custom.api.com");
    }
}
