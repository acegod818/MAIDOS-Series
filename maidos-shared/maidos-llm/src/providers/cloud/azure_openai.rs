//! Azure OpenAI API Provider
//!
//! <impl>
//! WHAT: Azure OpenAI API implementation (Microsoft-hosted OpenAI models)
//! WHY: Enterprise Azure deployments with regional data residency
//! HOW: OpenAI-compatible REST API with Azure-specific auth and endpoints
//! TEST: Request formatting, deployment routing, error handling
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

const DEFAULT_API_VERSION: &str = "2024-06-01";

/// Azure OpenAI Provider (Microsoft-hosted OpenAI models)
pub struct AzureOpenAiProvider {
    client: Client,
    api_key: String,
    endpoint: String,
    deployment_id: String,
    api_version: String,
    info: ProviderInfo,
}

impl AzureOpenAiProvider {
    /// Create new Azure OpenAI provider
    ///
    /// # Arguments
    /// * `api_key` - Azure OpenAI API key
    /// * `endpoint` - Azure resource endpoint (e.g., https://your-resource.openai.azure.com)
    /// * `deployment_id` - Deployment name configured in Azure
    /// * `api_version` - API version (default: 2024-06-01)
    pub fn new(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
        deployment_id: impl Into<String>,
        api_version: Option<String>,
    ) -> Self {
        let endpoint = endpoint.into();
        let deployment_id = deployment_id.into();
        let api_version = api_version.unwrap_or_else(|| DEFAULT_API_VERSION.to_string());

        Self {
            client: Client::new(),
            api_key: api_key.into(),
            endpoint: endpoint.clone(),
            deployment_id: deployment_id.clone(),
            api_version,
            info: ProviderInfo {
                name: "Azure OpenAI".to_string(),
                version: "v1".to_string(),
                models: vec![
                    ModelInfo {
                        id: "gpt-4o".to_string(),
                        name: "GPT-4o".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(16384),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "gpt-4o-mini".to_string(),
                        name: "GPT-4o Mini".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(16384),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "gpt-4-turbo".to_string(),
                        name: "GPT-4 Turbo".to_string(),
                        context_window: 128_000,
                        max_output_tokens: Some(4096),
                        supports_vision: true,
                    },
                    ModelInfo {
                        id: "gpt-4".to_string(),
                        name: "GPT-4".to_string(),
                        context_window: 8_192,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "gpt-35-turbo".to_string(),
                        name: "GPT-3.5 Turbo".to_string(),
                        context_window: 16_385,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                ],
                base_url: format!(
                    "{}/openai/deployments/{}",
                    endpoint.trim_end_matches('/'),
                    deployment_id
                ),
                supports_streaming: true,
                supports_vision: true,
                supports_tools: true,
            },
        }
    }

    /// Get the current deployment ID
    pub fn deployment_id(&self) -> &str {
        &self.deployment_id
    }

    /// Get the Azure endpoint
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Get the API version
    pub fn api_version(&self) -> &str {
        &self.api_version
    }

    fn build_url(&self, path: &str) -> String {
        format!(
            "{}/openai/deployments/{}/{}?api-version={}",
            self.endpoint.trim_end_matches('/'),
            self.deployment_id,
            path,
            self.api_version
        )
    }

    fn build_request(&self, req: &CompletionRequest) -> AzureOpenAiRequest {
        let mut messages = Vec::new();

        if let Some(system) = &req.system {
            messages.push(AzureMessage {
                role: "system".to_string(),
                content: system.clone(),
                name: None,
            });
        }

        for msg in &req.messages {
            messages.push(AzureMessage {
                role: msg.role.as_str().to_string(),
                content: msg.text().to_string(),
                name: msg.name.clone(),
            });
        }

        AzureOpenAiRequest {
            messages,
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            top_p: req.top_p,
            stop: if req.stop.as_ref().is_none_or(|v| v.is_empty()) { None } else { req.stop.clone() },
            stream: Some(false),
        }
    }

    fn parse_response(&self, resp: AzureOpenAiResponse) -> Result<CompletionResponse> {
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
impl LlmProvider for AzureOpenAiProvider {
    fn info(&self) -> &ProviderInfo {
        &self.info
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let url = self.build_url("chat/completions");
        let body = self.build_request(&request);

        let response = self
            .client
            .post(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if status.as_u16() == 401 {
                return Err(LlmError::Auth("Invalid Azure OpenAI API key".to_string()));
            }
            if status.as_u16() == 404 {
                return Err(LlmError::InvalidRequest(format!(
                    "Deployment '{}' not found. Check your deployment_id configuration.",
                    self.deployment_id
                )));
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

        let api_response: AzureOpenAiResponse = response.json().await?;
        self.parse_response(api_response)
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        let mut req = request;
        req.stream = true;
        let url = self.build_url("chat/completions");
        let body = self.build_request(&req);

        let response = self
            .client
            .post(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            if status.as_u16() == 401 {
                return Err(LlmError::Auth("Invalid Azure OpenAI API key".to_string()));
            }
            if status.as_u16() == 404 {
                return Err(LlmError::InvalidRequest(format!(
                    "Deployment '{}' not found",
                    self.deployment_id
                )));
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
                        if let Ok(chunk) = serde_json::from_str::<AzureStreamChunk>(data) {
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
        // Azure deployments are configured, return static list
        Ok(self.info.models.clone())
    }

    async fn health_check(&self) -> Result<bool> {
        // Azure doesn't have a simple health endpoint, try a minimal request
        let url = format!(
            "{}/openai/deployments?api-version={}",
            self.endpoint.trim_end_matches('/'),
            self.api_version
        );
        let response = self
            .client
            .get(&url)
            .header("api-key", &self.api_key)
            .send()
            .await?;
        Ok(response.status().is_success())
    }
}

// Azure OpenAI API types

#[derive(Debug, Serialize)]
struct AzureOpenAiRequest {
    messages: Vec<AzureMessage>,
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
struct AzureMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AzureOpenAiResponse {
    id: Option<String>,
    model: String,
    choices: Vec<AzureChoice>,
    usage: Option<AzureUsage>,
}

#[derive(Debug, Deserialize)]
struct AzureChoice {
    message: AzureMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AzureUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// Streaming response structures (OpenAI-compatible)
#[derive(Debug, Deserialize)]
struct AzureStreamChunk {
    choices: Vec<AzureStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct AzureStreamChoice {
    delta: AzureDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AzureDelta {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let provider = AzureOpenAiProvider::new(
            "test-key",
            "https://test.openai.azure.com",
            "gpt-4o-deployment",
            None,
        );
        let info = provider.info();
        assert_eq!(info.name, "Azure OpenAI");
        assert!(!info.models.is_empty());
        assert!(info.supports_vision);
    }

    #[test]
    fn test_build_url() {
        let provider = AzureOpenAiProvider::new(
            "key",
            "https://myresource.openai.azure.com",
            "my-deployment",
            None,
        );
        let url = provider.build_url("chat/completions");
        assert!(url.contains("myresource.openai.azure.com"));
        assert!(url.contains("my-deployment"));
        assert!(url.contains("api-version"));
    }

    #[test]
    fn test_deployment_id() {
        let provider = AzureOpenAiProvider::new(
            "key",
            "https://test.openai.azure.com",
            "custom-deployment",
            None,
        );
        assert_eq!(provider.deployment_id(), "custom-deployment");
    }

    #[test]
    fn test_custom_api_version() {
        let provider = AzureOpenAiProvider::new(
            "key",
            "https://test.openai.azure.com",
            "deployment",
            Some("2024-02-15-preview".to_string()),
        );
        assert_eq!(provider.api_version(), "2024-02-15-preview");
    }

    #[test]
    fn test_default_api_version() {
        let provider = AzureOpenAiProvider::new(
            "key",
            "https://test.openai.azure.com",
            "deployment",
            None,
        );
        assert_eq!(provider.api_version(), DEFAULT_API_VERSION);
    }

    #[test]
    fn test_build_request() {
        let provider = AzureOpenAiProvider::new(
            "key",
            "https://test.openai.azure.com",
            "deployment",
            None,
        );
        let request = CompletionRequest::new("gpt-4o")
            .system("You are helpful.")
            .message(Message::user("Hello"));

        let api_req = provider.build_request(&request);
        assert_eq!(api_req.messages.len(), 2);
    }

    #[test]
    fn test_endpoint_trailing_slash_handled() {
        let provider = AzureOpenAiProvider::new(
            "key",
            "https://test.openai.azure.com/",
            "deployment",
            None,
        );
        let url = provider.build_url("chat/completions");
        assert!(!url.contains("//openai"));
    }

    #[test]
    fn test_vision_models() {
        let provider = AzureOpenAiProvider::new(
            "key",
            "https://test.openai.azure.com",
            "deployment",
            None,
        );
        let info = provider.info();
        let vision_models: Vec<_> = info.models.iter().filter(|m| m.supports_vision).collect();
        assert!(vision_models.len() >= 3); // gpt-4o, gpt-4o-mini, gpt-4-turbo
    }
}
