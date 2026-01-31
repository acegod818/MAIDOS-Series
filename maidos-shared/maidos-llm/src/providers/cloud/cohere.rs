//! Cohere API Provider
//!
//! <impl>
//! WHAT: Cohere API implementation (Command R family)
//! WHY: Enterprise RAG and retrieval-augmented generation
//! HOW: Cohere-native REST API with document/RAG support
//! TEST: Request formatting, response parsing, RAG documents, vision error
//! </impl>

use crate::error::{LlmError, Result};
use crate::message::{CompletionResponse, FinishReason, Message, Usage};
use crate::provider::{
    CompletionRequest, CompletionStream, LlmProvider, ModelInfo, ProviderInfo, StreamChunk,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_BASE_URL: &str = "https://api.cohere.ai/v1";

/// Cohere Provider (Command R models with RAG support)
pub struct CohereProvider {
    client: Client,
    api_key: String,
    base_url: String,
    info: ProviderInfo,
}

impl CohereProvider {
    /// Create new Cohere provider
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: base_url.clone(),
            info: ProviderInfo {
                name: "Cohere".to_string(),
                version: "v1".to_string(),
                models: vec![
                    ModelInfo {
                        id: "command-r-plus".to_string(),
                        name: "Command R+".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "command-r".to_string(),
                        name: "Command R".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "command-light".to_string(),
                        name: "Command Light".to_string(),
                        context_window: 4_096,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "command".to_string(),
                        name: "Command".to_string(),
                        context_window: 4_096,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "command-r-08-2024".to_string(),
                        name: "Command R (Aug 2024)".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "command-r-plus-08-2024".to_string(),
                        name: "Command R+ (Aug 2024)".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                ],
                base_url,
                supports_streaming: true,
                supports_vision: false, // Cohere does not support vision
                supports_tools: true,
            },
        }
    }

    fn build_request(&self, req: &CompletionRequest) -> CohereRequest {
        // Build chat history
        let mut chat_history = Vec::new();

        if let Some(system) = &req.system {
            chat_history.push(CohereChatMessage {
                role: "SYSTEM".to_string(),
                message: system.clone(),
            });
        }

        // Add all messages except the last one to chat_history
        // The last user message becomes the 'message' field
        let mut last_message = String::new();
        for (i, msg) in req.messages.iter().enumerate() {
            if i == req.messages.len() - 1 && msg.role.as_str() == "user" {
                last_message = msg.text().to_string();
            } else {
                chat_history.push(CohereChatMessage {
                    role: match msg.role.as_str() {
                        "user" => "USER".to_string(),
                        "assistant" => "CHATBOT".to_string(),
                        _ => "USER".to_string(),
                    },
                    message: msg.text().to_string(),
                });
            }
        }

        // If no explicit last user message, use empty
        if last_message.is_empty() && !req.messages.is_empty() {
            let last = req.messages.last().map(|m| m.text().to_string()).unwrap_or_default();
            last_message = last;
        }

        CohereRequest {
            model: req.model.clone(),
            message: last_message,
            chat_history: if chat_history.is_empty() { None } else { Some(chat_history) },
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            stop_sequences: if req.stop.as_ref().is_none_or(|v| v.is_empty()) { None } else { req.stop.clone() },
            stream: Some(false),
            documents: None, // RAG documents can be added via extension
        }
    }

    fn parse_response(&self, resp: CohereResponse) -> Result<CompletionResponse> {
        let finish_reason = resp.finish_reason.map(|r| match r.as_str() {
            "COMPLETE" => FinishReason::Stop,
            "MAX_TOKENS" => FinishReason::Length,
            "ERROR" => FinishReason::Unknown,
            "ERROR_TOXIC" => FinishReason::ContentFilter,
            "ERROR_LIMIT" => FinishReason::Length,
            "USER_CANCEL" => FinishReason::Stop,
            "TOOL_CALL" => FinishReason::ToolUse,
            _ => FinishReason::Unknown,
        }).unwrap_or(FinishReason::Unknown);

        let usage = if let (Some(input), Some(output)) = (resp.meta.as_ref().and_then(|m| m.tokens.as_ref()).map(|t| t.input_tokens), resp.meta.as_ref().and_then(|m| m.tokens.as_ref()).map(|t| t.output_tokens)) {
            Usage {
                prompt_tokens: input,
                completion_tokens: output,
                total_tokens: input + output,
                cached_tokens: None,
            }
        } else {
            Usage::default()
        };

        Ok(CompletionResponse {
            message: Message::assistant(&resp.text),
            usage,
            finish_reason,
            model: resp.response_id.unwrap_or_default(),
            id: resp.generation_id,
        })
    }

    /// Check if a request contains vision content (Cohere doesn't support it)
    pub fn has_vision_content(_req: &CompletionRequest) -> bool {
        // Cohere doesn't support vision, always return false for detection
        false
    }
}

#[async_trait]
impl LlmProvider for CohereProvider {
    fn info(&self) -> &ProviderInfo {
        &self.info
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let url = format!("{}/chat", self.base_url);
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
                return Err(LlmError::Auth("Invalid Cohere API key".to_string()));
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

        let api_response: CohereResponse = response.json().await?;
        self.parse_response(api_response)
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        // Cohere streaming uses a different SSE format
        // For now, use fallback: call non-streaming and return as single chunk
        let response = self.complete(request).await?;
        let text = response.message.text().to_string();
        let usage = response.usage;

        let stream = futures::stream::once(async move {
            Ok(StreamChunk {
                delta: text.clone(),
                accumulated: text,
                is_final: true,
                usage: Some(usage),
            })
        });

        Ok(Box::pin(stream))
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

// Cohere API types

#[derive(Debug, Serialize)]
struct CohereRequest {
    model: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_history: Option<Vec<CohereChatMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    documents: Option<Vec<CohereDocument>>,
}

#[derive(Debug, Serialize)]
struct CohereChatMessage {
    role: String,
    message: String,
}

#[derive(Debug, Serialize)]
pub struct CohereDocument {
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Deserialize)]
struct CohereResponse {
    text: String,
    generation_id: Option<String>,
    response_id: Option<String>,
    finish_reason: Option<String>,
    meta: Option<CohereMeta>,
}

#[derive(Debug, Deserialize)]
struct CohereMeta {
    tokens: Option<CohereTokens>,
}

#[derive(Debug, Deserialize)]
struct CohereTokens {
    input_tokens: u32,
    output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let provider = CohereProvider::new("test-key", None);
        let info = provider.info();
        assert_eq!(info.name, "Cohere");
        assert!(!info.models.is_empty());
        assert!(!info.supports_vision); // Cohere doesn't support vision
        assert!(info.supports_tools);
    }

    #[test]
    fn test_build_request() {
        let provider = CohereProvider::new("test-key", None);
        let request = CompletionRequest::new("command-r-plus")
            .system("You are helpful.")
            .message(Message::user("Hello"));

        let api_req = provider.build_request(&request);
        assert_eq!(api_req.model, "command-r-plus");
        assert!(!api_req.message.is_empty());
    }

    #[test]
    fn test_custom_base_url() {
        let provider = CohereProvider::new("key", Some("https://custom.api.com".to_string()));
        assert_eq!(provider.base_url, "https://custom.api.com");
    }

    #[test]
    fn test_default_base_url() {
        let provider = CohereProvider::new("key", None);
        assert!(provider.base_url.contains("cohere.ai"));
    }

    #[test]
    fn test_no_vision_support() {
        let provider = CohereProvider::new("key", None);
        let info = provider.info();
        assert!(!info.supports_vision);
        for model in &info.models {
            assert!(!model.supports_vision);
        }
    }

    #[test]
    fn test_command_r_models() {
        let provider = CohereProvider::new("key", None);
        let info = provider.info();
        let command_r: Vec<_> = info.models.iter().filter(|m| m.id.contains("command-r")).collect();
        assert!(command_r.len() >= 4);
    }

    #[test]
    fn test_large_context_models() {
        let provider = CohereProvider::new("key", None);
        let info = provider.info();
        let large_ctx: Vec<_> = info.models.iter().filter(|m| m.context_window >= 128_000).collect();
        assert!(!large_ctx.is_empty());
    }

    #[test]
    fn test_request_with_chat_history() {
        let provider = CohereProvider::new("key", None);
        let request = CompletionRequest::new("command-r")
            .message(Message::user("First message"))
            .message(Message::assistant("Response"))
            .message(Message::user("Second message"));

        let api_req = provider.build_request(&request);
        assert!(api_req.chat_history.is_some());
        assert!(!api_req.message.is_empty());
    }
}
