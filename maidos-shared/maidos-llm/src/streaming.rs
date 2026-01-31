//! Streaming response handling for LLM providers
//!
//! This module provides a unified interface for handling streaming responses
//! from various LLM providers using Server-Sent Events (SSE).
//!
//! # Example
//! ```ignore
//! let mut stream = provider.chat_stream(request).await?;
//! while let Some(chunk) = stream.next().await {
//!     match chunk {
//!         Ok(c) => print!("{}", c.delta),
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```

use crate::error::{LlmError, Result};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Token usage information (sent in final chunk)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct StreamUsage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,
    /// Number of tokens in the completion
    pub completion_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
}

impl StreamUsage {
    /// Create new usage info
    pub fn new(prompt: u32, completion: u32) -> Self {
        Self {
            prompt_tokens: prompt,
            completion_tokens: completion,
            total_tokens: prompt + completion,
        }
    }
}

/// A single chunk from a streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Incremental text content
    pub delta: String,

    /// Finish reason (only in final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,

    /// Token usage (only in final chunk, if provider supports)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<StreamUsage>,

    /// Tool call delta (for function calling)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<ToolCallDelta>,
}

impl StreamChunk {
    /// Create a text-only chunk
    pub fn text(delta: impl Into<String>) -> Self {
        Self {
            delta: delta.into(),
            finish_reason: None,
            usage: None,
            tool_call: None,
        }
    }

    /// Create a final chunk with finish reason
    pub fn finish(reason: impl Into<String>) -> Self {
        Self {
            delta: String::new(),
            finish_reason: Some(reason.into()),
            usage: None,
            tool_call: None,
        }
    }

    /// Create a final chunk with usage info
    pub fn finish_with_usage(reason: impl Into<String>, usage: StreamUsage) -> Self {
        Self {
            delta: String::new(),
            finish_reason: Some(reason.into()),
            usage: Some(usage),
            tool_call: None,
        }
    }

    /// Check if this is the final chunk
    pub fn is_final(&self) -> bool {
        self.finish_reason.is_some()
    }
}

/// Tool call information in streaming mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDelta {
    /// Tool call index (for parallel calls)
    pub index: u32,
    /// Tool call ID (first chunk only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Function name (first chunk only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Arguments delta (incremental JSON)
    pub arguments_delta: String,
}

/// Unified streaming response trait
///
/// All providers implement this trait for consistent streaming behavior.
pub trait StreamingResponse: Send + Unpin {
    /// Get the next chunk from the stream
    ///
    /// Returns `None` when the stream is exhausted.
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<StreamChunk>>>;

    /// Cancel the stream
    ///
    /// This should clean up any resources and stop receiving data.
    fn cancel(&mut self);

    /// Check if the stream has been cancelled
    fn is_cancelled(&self) -> bool;
}

/// Boxed streaming response for dynamic dispatch
pub type BoxedStream = Pin<Box<dyn StreamingResponse>>;

/// SSE (Server-Sent Events) parser for streaming responses
///
/// Handles the common SSE format used by most LLM providers:
/// ```text
/// data: {"content": "Hello"}
/// data: {"content": " world"}
/// data: [DONE]
/// ```
#[derive(Debug)]
pub struct SseParser {
    buffer: String,
    done_marker: String,
}

impl SseParser {
    /// Create a new SSE parser with default [DONE] marker
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            done_marker: "[DONE]".to_string(),
        }
    }

    /// Create with custom done marker
    pub fn with_done_marker(marker: impl Into<String>) -> Self {
        Self {
            buffer: String::new(),
            done_marker: marker.into(),
        }
    }

    /// Parse incoming bytes and extract complete SSE events
    ///
    /// Returns a vector of data payloads (without "data: " prefix)
    pub fn parse(&mut self, bytes: &[u8]) -> Vec<SseEvent> {
        let text = String::from_utf8_lossy(bytes);
        self.buffer.push_str(&text);

        let mut events = Vec::new();

        // Split by double newline (SSE event boundary)
        while let Some(pos) = self.buffer.find("\n\n") {
            let event_text = self.buffer[..pos].to_string();
            self.buffer = self.buffer[pos + 2..].to_string();

            if let Some(event) = self.parse_event(&event_text) {
                events.push(event);
            }
        }

        // Also handle single newline for providers that use it
        while let Some(pos) = self.buffer.find('\n') {
            let line = self.buffer[..pos].to_string();
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                self.buffer = self.buffer[pos + 1..].to_string();
                continue;
            }

            // Check for complete data line
            if trimmed.starts_with("data:") {
                self.buffer = self.buffer[pos + 1..].to_string();
                if let Some(event) = self.parse_data_line(trimmed) {
                    events.push(event);
                }
            } else {
                break; // Wait for more data
            }
        }

        events
    }

    /// Parse a single SSE event block
    fn parse_event(&self, text: &str) -> Option<SseEvent> {
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("data:") {
                return self.parse_data_line(line);
            }
        }
        None
    }

    /// Parse a data: line
    fn parse_data_line(&self, line: &str) -> Option<SseEvent> {
        let data = line.strip_prefix("data:")?.trim();

        if data == self.done_marker {
            return Some(SseEvent::Done);
        }

        if data.is_empty() {
            return None;
        }

        Some(SseEvent::Data(data.to_string()))
    }

    /// Clear the internal buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Default for SseParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed SSE event
#[derive(Debug, Clone, PartialEq)]
pub enum SseEvent {
    /// Data event with JSON payload
    Data(String),
    /// Stream complete marker
    Done,
}

/// Stream state for tracking cancellation and completion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    /// Stream is active and receiving data
    Active,
    /// Stream completed normally
    Completed,
    /// Stream was cancelled
    Cancelled,
    /// Stream encountered an error
    Error,
}

/// OpenAI-compatible streaming response chunk
#[derive(Debug, Deserialize)]
pub struct OpenAiStreamChunk {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<u64>,
    pub model: Option<String>,
    pub choices: Vec<OpenAiStreamChoice>,
    #[serde(default)]
    pub usage: Option<OpenAiStreamUsage>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiStreamChoice {
    pub index: u32,
    pub delta: OpenAiDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct OpenAiDelta {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<OpenAiToolCallDelta>>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiToolCallDelta {
    pub index: u32,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub call_type: Option<String>,
    pub function: Option<OpenAiFunctionDelta>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiFunctionDelta {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiStreamUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl From<OpenAiStreamUsage> for StreamUsage {
    fn from(u: OpenAiStreamUsage) -> Self {
        Self {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }
    }
}

/// Convert OpenAI stream chunk to unified StreamChunk
impl TryFrom<OpenAiStreamChunk> for StreamChunk {
    type Error = LlmError;

    fn try_from(chunk: OpenAiStreamChunk) -> Result<Self> {
        let choice = chunk.choices.into_iter().next();

        match choice {
            Some(c) => {
                let delta = c.delta.content.unwrap_or_default();
                let tool_call = c.delta.tool_calls.and_then(|calls| {
                    calls.into_iter().next().map(|tc| ToolCallDelta {
                        index: tc.index,
                        id: tc.id,
                        name: tc.function.as_ref().and_then(|f| f.name.clone()),
                        arguments_delta: tc
                            .function
                            .and_then(|f| f.arguments)
                            .unwrap_or_default(),
                    })
                });

                Ok(StreamChunk {
                    delta,
                    finish_reason: c.finish_reason,
                    usage: chunk.usage.map(Into::into),
                    tool_call,
                })
            }
            None => {
                // Empty choice - might be usage-only final chunk
                if chunk.usage.is_some() {
                    Ok(StreamChunk {
                        delta: String::new(),
                        finish_reason: Some("stop".to_string()),
                        usage: chunk.usage.map(Into::into),
                        tool_call: None,
                    })
                } else {
                    Err(LlmError::ParseError("Empty choices in stream chunk".to_string()))
                }
            }
        }
    }
}

/// Anthropic streaming event types
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: AnthropicMessageStart },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: u32,
        content_block: AnthropicContentBlock,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: u32,
        delta: AnthropicDelta,
    },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: u32 },
    #[serde(rename = "message_delta")]
    MessageDelta { delta: AnthropicMessageDelta },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error { error: AnthropicErrorDetail },
}

#[derive(Debug, Deserialize)]
pub struct AnthropicMessageStart {
    pub id: String,
    pub model: String,
    pub usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },
}

#[derive(Debug, Deserialize)]
pub struct AnthropicMessageDelta {
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Option<AnthropicOutputUsage>,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicOutputUsage {
    pub output_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

/// Convert Anthropic stream event to unified StreamChunk
impl TryFrom<AnthropicStreamEvent> for Option<StreamChunk> {
    type Error = LlmError;

    fn try_from(event: AnthropicStreamEvent) -> Result<Self> {
        match event {
            AnthropicStreamEvent::ContentBlockDelta { delta, .. } => match delta {
                AnthropicDelta::TextDelta { text } => Ok(Some(StreamChunk::text(text))),
                AnthropicDelta::InputJsonDelta { partial_json } => {
                    Ok(Some(StreamChunk {
                        delta: String::new(),
                        finish_reason: None,
                        usage: None,
                        tool_call: Some(ToolCallDelta {
                            index: 0,
                            id: None,
                            name: None,
                            arguments_delta: partial_json,
                        }),
                    }))
                }
            },
            AnthropicStreamEvent::MessageDelta { delta } => {
                if let Some(reason) = delta.stop_reason {
                    let usage = delta.usage.map(|u| StreamUsage {
                        prompt_tokens: 0,
                        completion_tokens: u.output_tokens,
                        total_tokens: u.output_tokens,
                    });
                    Ok(Some(StreamChunk {
                        delta: String::new(),
                        finish_reason: Some(reason),
                        usage,
                        tool_call: None,
                    }))
                } else {
                    Ok(None)
                }
            }
            AnthropicStreamEvent::MessageStop => Ok(Some(StreamChunk::finish("end_turn"))),
            AnthropicStreamEvent::Error { error } => {
                Err(LlmError::Provider(format!("{}: {}", error.error_type, error.message)))
            }
            _ => Ok(None), // Ignore ping, message_start, etc.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_chunk_text() {
        let chunk = StreamChunk::text("Hello");
        assert_eq!(chunk.delta, "Hello");
        assert!(!chunk.is_final());
    }

    #[test]
    fn test_stream_chunk_finish() {
        let chunk = StreamChunk::finish("stop");
        assert!(chunk.delta.is_empty());
        assert!(chunk.is_final());
        assert_eq!(chunk.finish_reason, Some("stop".to_string()));
    }

    #[test]
    fn test_stream_chunk_with_usage() {
        let usage = StreamUsage::new(100, 50);
        let chunk = StreamChunk::finish_with_usage("stop", usage.clone());
        assert!(chunk.is_final());
        assert_eq!(chunk.usage, Some(usage));
    }

    #[test]
    fn test_stream_usage_total() {
        let usage = StreamUsage::new(100, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_sse_parser_basic() {
        let mut parser = SseParser::new();
        let events = parser.parse(b"data: {\"text\": \"hello\"}\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], SseEvent::Data("{\"text\": \"hello\"}".to_string()));
    }

    #[test]
    fn test_sse_parser_done() {
        let mut parser = SseParser::new();
        let events = parser.parse(b"data: [DONE]\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], SseEvent::Done);
    }

    #[test]
    fn test_sse_parser_multiple_events() {
        let mut parser = SseParser::new();
        let data = b"data: {\"a\": 1}\n\ndata: {\"b\": 2}\n\n";
        let events = parser.parse(data);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_sse_parser_partial() {
        let mut parser = SseParser::new();

        // First partial chunk
        let events1 = parser.parse(b"data: {\"par");
        assert!(events1.is_empty());

        // Complete the event
        let events2 = parser.parse(b"tial\": true}\n\n");
        assert_eq!(events2.len(), 1);
        assert_eq!(
            events2[0],
            SseEvent::Data("{\"partial\": true}".to_string())
        );
    }

    #[test]
    fn test_sse_parser_custom_done_marker() {
        let mut parser = SseParser::with_done_marker("DONE");
        let events = parser.parse(b"data: DONE\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], SseEvent::Done);
    }

    #[test]
    fn test_sse_parser_empty_data() {
        let mut parser = SseParser::new();
        let events = parser.parse(b"data: \n\n");
        assert!(events.is_empty());
    }

    #[test]
    fn test_openai_chunk_conversion() {
        let chunk = OpenAiStreamChunk {
            id: Some("chatcmpl-123".to_string()),
            object: Some("chat.completion.chunk".to_string()),
            created: Some(1234567890),
            model: Some("gpt-4".to_string()),
            choices: vec![OpenAiStreamChoice {
                index: 0,
                delta: OpenAiDelta {
                    role: None,
                    content: Some("Hello".to_string()),
                    tool_calls: None,
                },
                finish_reason: None,
            }],
            usage: None,
        };

        let stream_chunk: StreamChunk = chunk.try_into().expect("conversion failed");
        assert_eq!(stream_chunk.delta, "Hello");
        assert!(!stream_chunk.is_final());
    }

    #[test]
    fn test_openai_chunk_with_finish() {
        let chunk = OpenAiStreamChunk {
            id: Some("chatcmpl-123".to_string()),
            object: Some("chat.completion.chunk".to_string()),
            created: Some(1234567890),
            model: Some("gpt-4".to_string()),
            choices: vec![OpenAiStreamChoice {
                index: 0,
                delta: OpenAiDelta::default(),
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(OpenAiStreamUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
        };

        let stream_chunk: StreamChunk = chunk.try_into().expect("conversion failed");
        assert!(stream_chunk.is_final());
        assert_eq!(stream_chunk.finish_reason, Some("stop".to_string()));
        assert!(stream_chunk.usage.is_some());
    }

    #[test]
    fn test_anthropic_text_delta() {
        let event = AnthropicStreamEvent::ContentBlockDelta {
            index: 0,
            delta: AnthropicDelta::TextDelta {
                text: "Hello".to_string(),
            },
        };

        let chunk: Option<StreamChunk> = event.try_into().expect("conversion failed");
        assert!(chunk.is_some());
        let chunk = chunk.unwrap();
        assert_eq!(chunk.delta, "Hello");
    }

    #[test]
    fn test_anthropic_message_stop() {
        let event = AnthropicStreamEvent::MessageStop;
        let chunk: Option<StreamChunk> = event.try_into().expect("conversion failed");
        assert!(chunk.is_some());
        assert!(chunk.unwrap().is_final());
    }

    #[test]
    fn test_anthropic_ping_ignored() {
        let event = AnthropicStreamEvent::Ping;
        let chunk: Option<StreamChunk> = event.try_into().expect("conversion failed");
        assert!(chunk.is_none());
    }

    #[test]
    fn test_stream_state() {
        assert_eq!(StreamState::Active, StreamState::Active);
        assert_ne!(StreamState::Active, StreamState::Completed);
    }

    #[test]
    fn test_tool_call_delta() {
        let chunk = StreamChunk {
            delta: String::new(),
            finish_reason: None,
            usage: None,
            tool_call: Some(ToolCallDelta {
                index: 0,
                id: Some("call_123".to_string()),
                name: Some("get_weather".to_string()),
                arguments_delta: "{\"loc".to_string(),
            }),
        };

        assert!(chunk.tool_call.is_some());
        let tc = chunk.tool_call.unwrap();
        assert_eq!(tc.name, Some("get_weather".to_string()));
    }
}
