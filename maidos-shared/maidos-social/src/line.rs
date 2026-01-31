//! LINE API integration

use crate::{SocialPlatform, PlatformInfo, SocialMessage, Result, SocialError};
use async_trait::async_trait;
use reqwest::Client;

/// LINE API client
pub struct Line {
    client: Client,
    access_token: String,
    base_url: String,
}

impl Line {
    /// Create a new LINE client
    pub fn new(access_token: String) -> Self {
        Self {
            client: Client::new(),
            access_token,
            base_url: "https://api.line.me/v2/bot".to_string(),
        }
    }

    /// Get the access token
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    /// Send a message to a user or group
    pub async fn send_message(&self, to: &str, content: &str) -> Result<()> {
        let url = format!("{}/message/push", self.base_url);
        
        let request_body = serde_json::json!({
            "to": to,
            "messages": [
                {
                    "type": "text",
                    "text": content
                }
            ]
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(SocialError::Platform(format!("LINE API error {}: {}", status, error_text)))
        }
    }

    /// Send a rich message with multiple message types
    pub async fn send_rich_message(&self, to: &str, message: &SocialMessage) -> Result<()> {
        let url = format!("{}/message/push", self.base_url);
        
        let mut messages = vec![serde_json::json!({
            "type": "text",
            "text": message.content
        })];

        // Add image attachments
        for attachment in &message.attachments {
            messages.push(serde_json::json!({
                "type": "image",
                "originalContentUrl": attachment,
                "previewImageUrl": attachment
            }));
        }

        let request_body = serde_json::json!({
            "to": to,
            "messages": messages
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(SocialError::Platform(format!("LINE API error {}: {}", status, error_text)))
        }
    }
}

#[async_trait]
impl SocialPlatform for Line {
    async fn send_message(&self, target: &str, message: &str) -> Result<()> {
        self.send_message(target, message).await
    }

    async fn send_structured_message(&self, target: &str, message: &SocialMessage) -> Result<()> {
        self.send_rich_message(target, message).await
    }

    fn platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            name: "LINE".to_string(),
            version: "2".to_string(),
            rate_limit: 1000, // 1000 requests per minute
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    fn spawn_http_server(status: &str, body: &str) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let status = status.to_string();
        let body = body.to_string();

        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buffer = [0u8; 1024];
                let _ = stream.read(&mut buffer);
                let response = format!(
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    status,
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });

        (format!("http://{}", addr), handle)
    }

    #[test]
    fn test_line_creation() {
        let line = Line::new("test-token".to_string());
        assert_eq!(line.access_token(), "test-token");
    }

    #[test]
    fn test_platform_info() {
        let line = Line::new("test-token".to_string());
        let info = line.platform_info();
        assert_eq!(info.name, "LINE");
        assert_eq!(info.version, "2");
        assert_eq!(info.rate_limit, 1000);
    }

    #[tokio::test]
    async fn test_send_message_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{}"#);
        let mut line = Line::new("test-token".to_string());
        line.base_url = base_url;
        line.send_message("user", "hi").await.unwrap();
    }

    #[tokio::test]
    async fn test_send_message_error() {
        let (base_url, _handle) = spawn_http_server("500 Internal Server Error", r#"{"message":"bad"}"#);
        let mut line = Line::new("test-token".to_string());
        line.base_url = base_url;
        let result = line.send_message("user", "hi").await;
        assert!(matches!(result, Err(SocialError::Platform(_))));
    }

    #[tokio::test]
    async fn test_send_rich_message_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{}"#);
        let mut line = Line::new("test-token".to_string());
        line.base_url = base_url;
        let message = SocialMessage {
            content: "hello".to_string(),
            attachments: vec!["https://example.com/a.png".to_string()],
            embeds: vec![],
        };
        line.send_rich_message("user", &message).await.unwrap();
    }
}
