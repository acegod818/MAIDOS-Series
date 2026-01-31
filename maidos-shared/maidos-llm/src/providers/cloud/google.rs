//! Google Gemini API Provider
//!
//! <impl>
//! WHAT: Google Gemini API implementation
//! WHY: Support Google's AI models through unified interface
//! HOW: reqwest HTTP client, Google AI Studio API
//! TEST: Request formatting, response parsing, error handling
//! </impl>

use crate::error::{LlmError, Result};
use crate::message::{CompletionResponse, Content, FinishReason, Message, Role, Usage};
use crate::provider::{
    CompletionRequest, CompletionStream, LlmProvider, ModelInfo, ProviderInfo,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Google Gemini Provider
pub struct GoogleProvider {
    client: Client,
    api_key: String,
    base_url: String,
    info: ProviderInfo,
}

impl GoogleProvider {
    /// Create new Google provider
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: base_url.clone(),
            info: ProviderInfo {
                name: "Google".to_string(),
                version: "v1beta".to_string(),
                models: vec![
                    ModelInfo {
                        id: "gemini-1.5-pro".to_string(),
                        name: "Gemini 1.5 Pro".to_string(),
                        context_window: 1_000_000,
                        max_output_tokens: Some(8192),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "gemini-1.5-flash".to_string(),
                        name: "Gemini 1.5 Flash".to_string(),
                        context_window: 1_000_000,
                        max_output_tokens: Some(8192),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "gemini-2.0-flash-exp".to_string(),
                        name: "Gemini 2.0 Flash".to_string(),
                        context_window: 1_000_000,
                        max_output_tokens: Some(8192),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "gemini-pro".to_string(),
                        name: "Gemini Pro".to_string(),
                        context_window: 32_000,
                        max_output_tokens: Some(2048),
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

    fn build_request(&self, req: &CompletionRequest) -> GeminiRequest {
        let mut contents = Vec::new();
        let mut system_instruction = None;

        // Handle system message
        if let Some(system) = &req.system {
            system_instruction = Some(GeminiContent {
                role: None,
                parts: vec![GeminiPart::Text { text: system.clone() }],
            });
        }

        // Convert messages
        for msg in &req.messages {
            let role = match msg.role {
                Role::User => "user",
                Role::Assistant => "model",
                Role::System => {
                    // Add to system instruction
                    if system_instruction.is_none() {
                        system_instruction = Some(GeminiContent {
                            role: None,
                            parts: vec![GeminiPart::Text { text: msg.text().to_string() }],
                        });
                    }
                    continue;
                }
                Role::Tool => "function",
            };

            let parts = self.convert_content(&msg.content);
            contents.push(GeminiContent {
                role: Some(role.to_string()),
                parts,
            });
        }

        GeminiRequest {
            contents,
            system_instruction,
            generation_config: Some(GeminiGenerationConfig {
                temperature: req.temperature,
                top_p: req.top_p,
                max_output_tokens: req.max_tokens,
                stop_sequences: if req.stop.as_ref().is_none_or(|v| v.is_empty()) {
                    None
                } else {
                    req.stop.clone()
                },
            }),
        }
    }

    fn convert_content(&self, content: &[Content]) -> Vec<GeminiPart> {
        content
            .iter()
            .map(|c| match c {
                Content::Text { text } => GeminiPart::Text { text: text.clone() },
                Content::Image { base64, media_type, .. } => {
                    if let Some(data) = base64 {
                        GeminiPart::InlineData {
                            inline_data: InlineData {
                                mime_type: media_type.clone().unwrap_or_else(|| "image/png".to_string()),
                                data: data.clone(),
                            },
                        }
                    } else {
                        GeminiPart::Text { text: "[image url not supported]".to_string() }
                    }
                }
                _ => GeminiPart::Text { text: "[unsupported content]".to_string() },
            })
            .collect()
    }

    fn parse_response(&self, resp: GeminiResponse, model: &str) -> Result<CompletionResponse> {
        let candidate = resp
            .candidates
            .into_iter()
            .next()
            .ok_or_else(|| LlmError::ParseError("No candidates in response".to_string()))?;

        let text: String = candidate
            .content
            .parts
            .iter()
            .filter_map(|p| match p {
                GeminiPart::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let finish_reason = candidate.finish_reason.map(|r| match r.as_str() {
            "STOP" => FinishReason::Stop,
            "MAX_TOKENS" => FinishReason::Length,
            "SAFETY" => FinishReason::ContentFilter,
            _ => FinishReason::Unknown,
        }).unwrap_or(FinishReason::Unknown);

        let usage = resp.usage_metadata.map(|u| Usage {
            prompt_tokens: u.prompt_token_count.unwrap_or(0),
            completion_tokens: u.candidates_token_count.unwrap_or(0),
            total_tokens: u.total_token_count.unwrap_or(0),
            cached_tokens: None,
        }).unwrap_or_default();

        Ok(CompletionResponse {
            message: Message::assistant(&text),
            usage,
            finish_reason,
            model: model.to_string(),
            id: None,
        })
    }
}

#[async_trait]
impl LlmProvider for GoogleProvider {
    fn info(&self) -> &ProviderInfo {
        &self.info
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let model = &request.model;
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, model, self.api_key
        );

        let body = self.build_request(&request);

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(LlmError::Auth("Invalid Google API key".to_string()));
            }
            if status.as_u16() == 429 {
                return Err(LlmError::RateLimited { retry_after_secs: 60 });
            }

            return Err(LlmError::ProviderError {
                code: status.to_string(),
                message: body,
            });
        }

        let gemini_response: GeminiResponse = response.json().await?;
        self.parse_response(gemini_response, model)
    }

    async fn complete_stream(&self, _request: CompletionRequest) -> Result<CompletionStream> {
        Err(LlmError::Provider("Streaming not yet implemented for Google".to_string()))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(self.info.models.clone())
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/models?key={}", self.base_url, self.api_key);
        let response = self.client.get(&url).send().await?;
        Ok(response.status().is_success())
    }
}

// Gemini API types

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GeminiPart {
    Text { text: String },
    #[serde(rename_all = "camelCase")]
    InlineData { inline_data: InlineData },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(default)]
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiCandidate {
    content: GeminiContent,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiUsageMetadata {
    prompt_token_count: Option<u32>,
    candidates_token_count: Option<u32>,
    total_token_count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let provider = GoogleProvider::new("test-key", None);
        let info = provider.info();
        assert_eq!(info.name, "Google");
        assert!(!info.models.is_empty());
    }

    #[test]
    fn test_build_request() {
        let provider = GoogleProvider::new("test-key", None);
        let request = CompletionRequest::new("gemini-1.5-flash")
            .system("You are helpful.")
            .message(Message::user("Hello"));

        let gemini_req = provider.build_request(&request);
        assert!(!gemini_req.contents.is_empty());
        assert!(gemini_req.system_instruction.is_some());
    }

    #[test]
    fn test_custom_base_url() {
        let provider = GoogleProvider::new("key", Some("https://custom.api.com".to_string()));
        assert_eq!(provider.base_url, "https://custom.api.com");
    }

    #[test]
    fn test_models_support_vision() {
        let provider = GoogleProvider::new("key", None);
        let info = provider.info();
        let vision_models: Vec<_> = info.models.iter().filter(|m| m.supports_vision).collect();
        assert!(!vision_models.is_empty());
    }
}
