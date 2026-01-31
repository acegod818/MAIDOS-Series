//! vLLM API Provider
//!
//! <impl>
//! WHAT: vLLM high-throughput inference server (OpenAI-compatible)
//! WHY: Support high-performance local model serving
//! HOW: OpenAI-compatible REST API
//! TEST: Request formatting, response parsing, error handling
//! </impl>

use crate::error::{LlmError, Result};
use crate::message::{CompletionResponse, FinishReason, Message, Usage};
use crate::provider::{
    CompletionRequest, CompletionStream, LlmProvider, ModelInfo, ProviderInfo, StreamChunk,
};
use crate::streaming::OpenAiStreamChunk;
use async_trait::async_trait;
use futures::StreamExt;
use bytes::Bytes;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_BASE_URL: &str = "http://localhost:8000/v1";

/// vLLM Provider (OpenAI-compatible high-throughput inference)
pub struct VllmProvider {
    client: Client,
    base_url: String,
    info: ProviderInfo,
}

impl VllmProvider {
    /// Create new vLLM provider
    pub fn new(base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Self {
            client: Client::new(),
            base_url: base_url.clone(),
            info: ProviderInfo {
                name: "vLLM".to_string(),
                version: "v1".to_string(),
                models: vec![
                    // vLLM serves one model at a time, configured at startup
                    ModelInfo {
                        id: "served-model".to_string(),
                        name: "Served Model".to_string(),
                        context_window: 4096,
                        max_output_tokens: Some(2048),
                        supports_vision: false,
                    },
                ],
                base_url,
                supports_streaming: true,
                supports_vision: false, // Depends on served model
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
            model: resp.model.unwrap_or_else(|| "served-model".to_string()),
            id: resp.id,
        })
    }
}

#[async_trait]
impl LlmProvider for VllmProvider {
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
            .map_err(|e| LlmError::ConnectionFailed(format!("vLLM not running: {}", e)))?;

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

    async fn complete_stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        let url = format!("{}/chat/completions", self.base_url);
        let mut body = self.build_request(&request);
        body.stream = Some(true);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::ConnectionFailed(format!("vLLM streaming failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError {
                code: status.to_string(),
                message: body,
            });
        }

        let mut sse_parser = crate::streaming::SseParser::new();
        let stream = response.bytes_stream().flat_map(move |item: std::result::Result<Bytes, reqwest::Error>| {
            let bytes = match item {
                Ok(b) => b,
                Err(e) => return futures::stream::iter(vec![Err(LlmError::Provider(e.to_string()))]).boxed(),
            };

            let events = sse_parser.parse(&bytes);
            let results: Vec<Result<StreamChunk>> = events
                .into_iter()
                .filter_map(|event| match event {
                    crate::streaming::SseEvent::Data(data) => {
                        match serde_json::from_str::<OpenAiStreamChunk>(&data) {
                            Ok(chunk) => {
                                let choice = chunk.choices.into_iter().next();
                                if let Some(c) = choice {
                                    if let Some(reason) = c.finish_reason {
                                        Some(Ok::<StreamChunk, LlmError>(StreamChunk::finish(reason)))
                                    } else {
                                        Some(Ok::<StreamChunk, LlmError>(StreamChunk::text(c.delta.content.unwrap_or_default())))
                                    }
                                } else {
                                    None
                                }
                            }
                            Err(e) => Some(Err(LlmError::ParseError(e.to_string()))),
                        }
                    }
                    crate::streaming::SseEvent::Done => None,
                })
                .collect();

            futures::stream::iter(results).boxed()
        });

        Ok(Box::pin(stream))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LlmError::ConnectionFailed(format!("vLLM not running: {}", e)))?;

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
            #[serde(default)]
            max_model_len: Option<u32>,
        }

        let models_resp: ModelsResponse = response.json().await?;
        
        Ok(models_resp.data.into_iter().map(|m| ModelInfo {
            id: m.id.clone(),
            name: m.id,
            context_window: m.max_model_len.unwrap_or(4096),
            max_output_tokens: Some(2048),
            supports_vision: false,
        }).collect())
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await;
        
        // vLLM health endpoint returns 200 when ready
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
        let provider = VllmProvider::new(None);
        let info = provider.info();
        assert_eq!(info.name, "vLLM");
    }

    #[test]
    fn test_build_request() {
        let provider = VllmProvider::new(None);
        let request = CompletionRequest::new("served-model")
            .system("You are helpful.")
            .message(Message::user("Hello"));

        let api_req = provider.build_request(&request);
        assert_eq!(api_req.model, "served-model");
        assert_eq!(api_req.messages.len(), 2);
    }

    #[test]
    fn test_custom_base_url() {
        let provider = VllmProvider::new(Some("http://gpu-server:8000/v1".to_string()));
        assert!(provider.base_url.contains("gpu-server"));
    }

    #[test]
    fn test_default_base_url() {
        let provider = VllmProvider::new(None);
        assert!(provider.base_url.contains("localhost:8000"));
    }
}
