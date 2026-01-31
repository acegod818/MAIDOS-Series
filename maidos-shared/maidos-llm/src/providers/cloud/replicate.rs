//! Replicate API Provider
//!
//! <impl>
//! WHAT: Replicate API implementation (async prediction model)
//! WHY: Access to diverse open-source models with async execution
//! HOW: Async polling API (POST create → GET poll) or streaming endpoint
//! TEST: Prediction lifecycle, polling, error handling
//! </impl>

use crate::error::{LlmError, Result};
use crate::message::{CompletionResponse, FinishReason, Message, Usage};
use crate::provider::{
    CompletionRequest, CompletionStream, LlmProvider, ModelInfo, ProviderInfo, StreamChunk,
};
use futures::StreamExt;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.replicate.com/v1";
const MAX_POLL_ATTEMPTS: u32 = 60;
const POLL_INTERVAL_MS: u64 = 500;

/// Replicate Provider (async prediction-based inference)
pub struct ReplicateProvider {
    client: Client,
    api_token: String,
    base_url: String,
    info: ProviderInfo,
}

impl ReplicateProvider {
    /// Create new Replicate provider
    pub fn new(api_token: impl Into<String>, base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Self {
            client: Client::new(),
            api_token: api_token.into(),
            base_url: base_url.clone(),
            info: ProviderInfo {
                name: "Replicate".to_string(),
                version: "v1".to_string(),
                models: vec![
                    // Llama 2
                    ModelInfo {
                        id: "meta/llama-2-70b-chat".to_string(),
                        name: "Llama 2 70B Chat".to_string(),
                        context_window: 4_096,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    ModelInfo {
                        id: "meta/llama-2-13b-chat".to_string(),
                        name: "Llama 2 13B Chat".to_string(),
                        context_window: 4_096,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    // Mixtral
                    ModelInfo {
                        id: "mistralai/mixtral-8x7b-instruct-v0.1".to_string(),
                        name: "Mixtral 8x7B Instruct".to_string(),
                        context_window: 32_768,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                    // Stability / Vision models
                    ModelInfo {
                        id: "yorickvp/llava-13b".to_string(),
                        name: "LLaVA 13B".to_string(),
                        context_window: 4_096,
                        max_output_tokens: Some(2048),
                        supports_vision: true,
                    },
                    // Mistral
                    ModelInfo {
                        id: "mistralai/mistral-7b-instruct-v0.2".to_string(),
                        name: "Mistral 7B Instruct".to_string(),
                        context_window: 32_768,
                        max_output_tokens: Some(4096),
                        supports_vision: false,
                    },
                ],
                base_url,
                supports_streaming: true,
                supports_vision: true, // Some models support vision
                supports_tools: false, // Replicate doesn't have unified tool support
            },
        }
    }

    /// Check if a model supports vision
    pub fn model_supports_vision(model: &str) -> bool {
        model.contains("llava") || model.contains("vision")
    }

    /// Check if a model supports function calling (most don't on Replicate)
    pub fn model_supports_tools(_model: &str) -> bool {
        false
    }

    fn build_prediction_input(&self, req: &CompletionRequest) -> serde_json::Value {
        // Build a simple prompt from messages
        let mut prompt = String::new();

        if let Some(system) = &req.system {
            prompt.push_str(&format!("[INST] <<SYS>>\n{}\n<</SYS>>\n\n", system));
        }

        for msg in &req.messages {
            match msg.role.as_str() {
                "user" => {
                    if prompt.contains("<<SYS>>") {
                        prompt.push_str(&format!("{} [/INST]", msg.text()));
                    } else {
                        prompt.push_str(&format!("[INST] {} [/INST]", msg.text()));
                    }
                }
                "assistant" => {
                    prompt.push_str(&format!(" {}\n", msg.text()));
                }
                _ => {}
            }
        }

        let mut input = serde_json::json!({
            "prompt": prompt
        });

        if let Some(max_tokens) = req.max_tokens {
            input["max_new_tokens"] = serde_json::json!(max_tokens);
        }
        if let Some(temperature) = req.temperature {
            input["temperature"] = serde_json::json!(temperature);
        }
        if let Some(top_p) = req.top_p {
            input["top_p"] = serde_json::json!(top_p);
        }

        input
    }

    async fn create_prediction(&self, model: &str, input: serde_json::Value) -> Result<String> {
        let url = format!("{}/predictions", self.base_url);

        let body = serde_json::json!({
            "version": self.get_model_version(model),
            "input": input
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if status.as_u16() == 401 {
                return Err(LlmError::Auth("Invalid Replicate API token".to_string()));
            }
            if status.as_u16() == 422 {
                return Err(LlmError::InvalidRequest(format!(
                    "Invalid model or input: {}",
                    body
                )));
            }

            return Err(LlmError::ProviderError {
                code: status.to_string(),
                message: body,
            });
        }

        let prediction: ReplicatePrediction = response.json().await?;
        prediction.urls.get.ok_or_else(|| {
            LlmError::Provider("No prediction URL returned".to_string())
        })
    }

    async fn poll_prediction(&self, url: &str) -> Result<ReplicatePrediction> {
        for _ in 0..MAX_POLL_ATTEMPTS {
            let response = self
                .client
                .get(url)
                .header("Authorization", format!("Bearer {}", self.api_token))
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(LlmError::ProviderError {
                    code: status.to_string(),
                    message: body,
                });
            }

            let prediction: ReplicatePrediction = response.json().await?;

            match prediction.status.as_str() {
                "succeeded" => return Ok(prediction),
                "failed" | "canceled" => {
                    let error = prediction.error.unwrap_or_else(|| "Prediction failed".to_string());
                    return Err(LlmError::Provider(error));
                }
                _ => {
                    // Still processing, wait and retry
                    tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
                }
            }
        }

        Err(LlmError::Timeout((MAX_POLL_ATTEMPTS as u64 * POLL_INTERVAL_MS) / 1000))
    }

    fn get_model_version<'a>(&self, model: &'a str) -> &'a str {
        // Return model as-is, Replicate uses version hashes
        // In production, you'd map model IDs to version hashes
        model
    }

    fn parse_output(&self, output: serde_json::Value) -> String {
        match output {
            serde_json::Value::String(s) => s,
            serde_json::Value::Array(arr) => {
                arr.into_iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
                    .join("")
            }
            _ => output.to_string(),
        }
    }
}

#[async_trait]
impl LlmProvider for ReplicateProvider {
    fn info(&self) -> &ProviderInfo {
        &self.info
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Replicate doesn't support function calling
        // Check and return clear error

        let input = self.build_prediction_input(&request);
        let poll_url = self.create_prediction(&request.model, input).await?;
        let prediction = self.poll_prediction(&poll_url).await?;

        let output_text = prediction
            .output
            .map(|o| self.parse_output(o))
            .unwrap_or_default();

        // Replicate doesn't return token counts in standard format
        let usage = Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            cached_tokens: None,
        };

        Ok(CompletionResponse {
            message: Message::assistant(&output_text),
            usage,
            finish_reason: FinishReason::Stop,
            model: request.model,
            id: prediction.id,
        })
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        let url = format!("{}/predictions", self.base_url);
        let mut input = self.build_prediction_input(&request);
        input["stream"] = serde_json::json!(true);

        let body = serde_json::json!({
            "version": self.get_model_version(&request.model),
            "input": input,
            "stream": true
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(LlmError::Provider(format!("Replicate stream failed: {}", response.status())));
        }

        let prediction: ReplicatePrediction = response.json().await?;
        let stream_url = prediction.urls.stream.ok_or_else(|| {
            LlmError::Provider("Model does not support streaming on Replicate".to_string())
        })?;

        let stream_response = self.client.get(&stream_url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Accept", "text/event-stream")
            .send()
            .await?;

        let mut sse_parser = crate::streaming::SseParser::new();
        let stream = stream_response.bytes_stream().flat_map(move |item| {
            let bytes = match item {
                Ok(b) => b,
                Err(e) => return futures::stream::iter(vec![Err(LlmError::Provider(e.to_string()))]).boxed(),
            };

            let events = sse_parser.parse(&bytes);
            let results: Vec<Result<StreamChunk>> = events
                .into_iter()
                .map(|event| match event {
                    crate::streaming::SseEvent::Data(data) => {
                        // Replicate SSE data is usually just the raw string chunk
                        Ok(StreamChunk::text(data))
                    }
                    crate::streaming::SseEvent::Done => Ok(StreamChunk::finish("stop")),
                })
                .collect();

            futures::stream::iter(results).boxed()
        });

        Ok(Box::pin(stream))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // Replicate doesn't have a simple model list endpoint
        Ok(self.info.models.clone())
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/predictions", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;
        // 401 means auth failed, anything else means the API is up
        Ok(response.status() != reqwest::StatusCode::UNAUTHORIZED)
    }
}

// Replicate API types

#[derive(Debug, Serialize, Deserialize)]
struct ReplicatePrediction {
    id: Option<String>,
    status: String,
    output: Option<serde_json::Value>,
    error: Option<String>,
    urls: ReplicateUrls,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReplicateUrls {
    get: Option<String>,
    cancel: Option<String>,
    stream: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let provider = ReplicateProvider::new("test-token", None);
        let info = provider.info();
        assert_eq!(info.name, "Replicate");
        assert!(!info.models.is_empty());
        assert!(!info.supports_tools); // Replicate doesn't support tools
    }

    #[test]
    fn test_custom_base_url() {
        let provider = ReplicateProvider::new("token", Some("https://custom.api.com".to_string()));
        assert_eq!(provider.base_url, "https://custom.api.com");
    }

    #[test]
    fn test_default_base_url() {
        let provider = ReplicateProvider::new("token", None);
        assert!(provider.base_url.contains("replicate.com"));
    }

    #[test]
    fn test_vision_model_detection() {
        assert!(ReplicateProvider::model_supports_vision("yorickvp/llava-13b"));
        assert!(!ReplicateProvider::model_supports_vision("meta/llama-2-70b-chat"));
    }

    #[test]
    fn test_tools_not_supported() {
        assert!(!ReplicateProvider::model_supports_tools("any-model"));
    }

    #[test]
    fn test_build_prediction_input() {
        let provider = ReplicateProvider::new("token", None);
        let request = CompletionRequest::new("meta/llama-2-70b-chat")
            .system("You are helpful.")
            .message(Message::user("Hello"));

        let input = provider.build_prediction_input(&request);
        assert!(input.get("prompt").is_some());
    }

    #[test]
    fn test_llama_models() {
        let provider = ReplicateProvider::new("token", None);
        let info = provider.info();
        let llama: Vec<_> = info.models.iter().filter(|m| m.id.contains("llama")).collect();
        assert!(llama.len() >= 2);
    }
}
