//! Slack API integration

use crate::{SocialPlatform, PlatformInfo, SocialMessage, Result, SocialError};
use async_trait::async_trait;
use reqwest::Client;

/// Slack API client
pub struct Slack {
    client: Client,
    token: String,
    base_url: String,
}

impl Slack {
    /// Create a new Slack client
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
            base_url: "https://slack.com/api".to_string(),
        }
    }

    /// Get the bot token
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Send a message to a channel
    pub async fn send_message(&self, channel_id: &str, content: &str) -> Result<()> {
        let url = format!("{}/chat.postMessage", self.base_url);
        
        let request_body = serde_json::json!({
            "channel": channel_id,
            "text": content
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;
        
        if response_json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
            Ok(())
        } else {
            let error = response_json.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error");
            Err(SocialError::Platform(format!("Slack API error: {}", error)))
        }
    }

    /// Send a rich message with attachments
    pub async fn send_rich_message(&self, channel_id: &str, message: &SocialMessage) -> Result<()> {
        let url = format!("{}/chat.postMessage", self.base_url);
        
        let mut attachments = Vec::new();
        for attachment_url in &message.attachments {
            attachments.push(serde_json::json!({
                "image_url": attachment_url
            }));
        }

        // Convert embeds to Slack attachments
        for embed in &message.embeds {
            attachments.push(serde_json::json!({
                "title": embed.title,
                "text": embed.description,
                "color": format!("#{:06x}", embed.color.unwrap_or(0)),
                "footer": embed.footer
            }));
        }

        let request_body = serde_json::json!({
            "channel": channel_id,
            "text": message.content,
            "attachments": attachments
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;
        
        if response_json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
            Ok(())
        } else {
            let error = response_json.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error");
            Err(SocialError::Platform(format!("Slack API error: {}", error)))
        }
    }
}

#[async_trait]
impl SocialPlatform for Slack {
    async fn send_message(&self, target: &str, message: &str) -> Result<()> {
        self.send_message(target, message).await
    }

    async fn send_structured_message(&self, target: &str, message: &SocialMessage) -> Result<()> {
        self.send_rich_message(target, message).await
    }

    fn platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            name: "Slack".to_string(),
            version: "1".to_string(),
            rate_limit: 100, // 100 requests per minute
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
    fn test_slack_creation() {
        let slack = Slack::new("test-token".to_string());
        assert_eq!(slack.token(), "test-token");
    }

    #[test]
    fn test_platform_info() {
        let slack = Slack::new("test-token".to_string());
        let info = slack.platform_info();
        assert_eq!(info.name, "Slack");
        assert_eq!(info.version, "1");
        assert_eq!(info.rate_limit, 100);
    }

    #[tokio::test]
    async fn test_send_message_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"ok":true}"#);
        let mut slack = Slack::new("test-token".to_string());
        slack.base_url = base_url;
        slack.send_message("chan", "hi").await.unwrap();
    }

    #[tokio::test]
    async fn test_send_message_error() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"ok":false,"error":"bad_auth"}"#);
        let mut slack = Slack::new("test-token".to_string());
        slack.base_url = base_url;
        let result = slack.send_message("chan", "hi").await;
        assert!(matches!(result, Err(SocialError::Platform(_))));
    }

    #[tokio::test]
    async fn test_send_rich_message_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"ok":true}"#);
        let mut slack = Slack::new("test-token".to_string());
        slack.base_url = base_url;

        let message = SocialMessage {
            content: "hello".to_string(),
            attachments: vec!["https://example.com/a.png".to_string()],
            embeds: vec![crate::SocialEmbed {
                title: Some("title".to_string()),
                description: Some("desc".to_string()),
                url: None,
                color: Some(0xff00ff),
                footer: Some("footer".to_string()),
            }],
        };

        slack.send_rich_message("chan", &message).await.unwrap();
    }
}
