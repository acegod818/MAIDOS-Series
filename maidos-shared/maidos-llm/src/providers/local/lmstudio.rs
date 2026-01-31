//! LM Studio API Provider
//!
//! <impl>
//! WHAT: LM Studio local inference server (OpenAI-compatible)
//! WHY: Support local model inference through LM Studio
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

const DEFAULT_BASE_URL: &str = "http://localhost:1234/v1";

/// LM Studio Provider (OpenAI-compatible local inference)
pub struct LmStudioProvider {
    client: Client,
    base_url: String,
    info: ProviderInfo,
}

impl LmStudioProvider {
    /// Create new LM Studio provider
    pub fn new(base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Self {
            client: Client::new(),
            base_url: base_url.clone(),
            info: ProviderInfo {
                name: "LM Studio".to_string(),
                version: "v1".to_string(),
                models: vec![
                    // LM Studio dynamically loads models, these are common examples
                    ModelInfo {
                        id: "local-model".to_string(),
                        name: "Local Model".to_string(),
                        context_window: 4096,
                        max_output_tokens: Some(2048),
                        supports_vision: false,
                    },
                ],
                base_url,
                supports_streaming: true,
                supports_vision: false, // Depends on loaded model
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
            model: resp.model.unwrap_or_else(|| "local-model".to_string()),
            id: resp.id,
        })
    }
}

#[async_trait]
impl LlmProvider for LmStudioProvider {
    fn info(&self) -> &ProviderInfo {
        &self.info
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_request(&request);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::ConnectionFailed(format!("LM Studio not running: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            return Err(LlmError::ProviderError {
                code: status.to_string(),
                message: body,
            });
        }

        let api_response: OpenAiCompatResponse = response.json().await?;
        self.parse_response(api_response)
    }

    async fn complete_stream(&self, _request: CompletionRequest) -> Result<CompletionStream> {
        Err(LlmError::Provider("Streaming not yet implemented for LM Studio".to_string()))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LlmError::ConnectionFailed(format!("LM Studio not running: {}", e)))?;

        if !response.status().is_success() {
            return Ok(self.info.models.clone());
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelData>,
        }

        #[derive(Deserialize)]
        struct ModelData {
            id: String,
        }

        let models_resp: ModelsResponse = response.json().await?;
        
        Ok(models_resp.data.into_iter().map(|m| ModelInfo {
            id: m.id.clone(),
            name: m.id,
            context_window: 4096, // LM Studio doesn't expose this
            max_output_tokens: Some(2048),
            supports_vision: false,
        }).collect())
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/models", self.base_url);
        let response = self.client.get(&url).send().await;
        Ok(response.map(|r| r.status().is_success()).unwrap_or(false))
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
    model: Option<String>,
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
        let provider = LmStudioProvider::new(None);
        let info = provider.info();
        assert_eq!(info.name, "LM Studio");
    }

    #[test]
    fn test_build_request() {
        let provider = LmStudioProvider::new(None);
        let request = CompletionRequest::new("local-model")
            .system("You are helpful.")
            .message(Message::user("Hello"));

        let api_req = provider.build_request(&request);
        assert_eq!(api_req.model, "local-model");
        assert_eq!(api_req.messages.len(), 2);
    }

    #[test]
    fn test_custom_base_url() {
        let provider = LmStudioProvider::new(Some("http://192.168.1.100:1234/v1".to_string()));
        assert!(provider.base_url.contains("192.168.1.100"));
    }

    #[test]
    fn test_default_base_url() {
        let provider = LmStudioProvider::new(None);
        assert!(provider.base_url.contains("localhost:1234"));
    }
}
