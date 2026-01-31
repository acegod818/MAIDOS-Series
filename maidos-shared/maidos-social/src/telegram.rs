//! Telegram API integration

use crate::{SocialPlatform, PlatformInfo, SocialMessage, Result, SocialError};
use async_trait::async_trait;
use reqwest::Client;

/// Telegram API client
pub struct Telegram {
    client: Client,
    token: String,
    base_url: String,
}

impl Telegram {
    /// Create a new Telegram client
    pub fn new(token: String) -> Self {
        let base_url = format!("https://api.telegram.org/bot{}", token);
        Self {
            client: Client::new(),
            token,
            base_url,
        }
    }

    /// Get the bot token
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Send a message to a chat
    pub async fn send_message(&self, chat_id: &str, content: &str) -> Result<()> {
        let url = format!("{}/sendMessage", self.base_url);
        
        let request_body = serde_json::json!({
            "chat_id": chat_id,
            "text": content
        });

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;
        
        if response_json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
            Ok(())
        } else {
            let error = response_json.get("description").and_then(|v| v.as_str()).unwrap_or("Unknown error");
            Err(SocialError::Platform(format!("Telegram API error: {}", error)))
        }
    }

    /// Send a photo message
    pub async fn send_photo(&self, chat_id: &str, photo_url: &str, caption: Option<&str>) -> Result<()> {
        let url = format!("{}/sendPhoto", self.base_url);
        
        let mut request_body = serde_json::json!({
            "chat_id": chat_id,
            "photo": photo_url
        });

        if let Some(caption_text) = caption {
            request_body["caption"] = serde_json::Value::String(caption_text.to_string());
        }

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;
        
        if response_json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
            Ok(())
        } else {
            let error = response_json.get("description").and_then(|v| v.as_str()).unwrap_or("Unknown error");
            Err(SocialError::Platform(format!("Telegram API error: {}", error)))
        }
    }
}

#[async_trait]
impl SocialPlatform for Telegram {
    async fn send_message(&self, target: &str, message: &str) -> Result<()> {
        self.send_message(target, message).await
    }

    async fn send_structured_message(&self, target: &str, message: &SocialMessage) -> Result<()> {
        // For Telegram, we'll send the content as a message and attachments as photos
        self.send_message(target, &message.content).await?;
        
        // Send each attachment as a photo
        for attachment in &message.attachments {
            self.send_photo(target, attachment, None).await?;
        }
        
        Ok(())
    }

    fn platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            name: "Telegram".to_string(),
            version: "1".to_string(),
            rate_limit: 30, // 30 messages per second
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
    fn test_telegram_creation() {
        let telegram = Telegram::new("test-token".to_string());
        assert_eq!(telegram.token(), "test-token");
    }

    #[test]
    fn test_platform_info() {
        let telegram = Telegram::new("test-token".to_string());
        let info = telegram.platform_info();
        assert_eq!(info.name, "Telegram");
        assert_eq!(info.version, "1");
        assert_eq!(info.rate_limit, 30);
    }

    #[tokio::test]
    async fn test_send_message_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"ok":true}"#);
        let mut telegram = Telegram::new("test-token".to_string());
        telegram.base_url = base_url;
        telegram.send_message("chat", "hi").await.unwrap();
    }

    #[tokio::test]
    async fn test_send_message_error() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"ok":false,"description":"bad"}"#);
        let mut telegram = Telegram::new("test-token".to_string());
        telegram.base_url = base_url;
        let result = telegram.send_message("chat", "hi").await;
        assert!(matches!(result, Err(SocialError::Platform(_))));
    }

    #[tokio::test]
    async fn test_send_photo_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"ok":true}"#);
        let mut telegram = Telegram::new("test-token".to_string());
        telegram.base_url = base_url;
        telegram.send_photo("chat", "https://example.com/a.png", Some("cap")).await.unwrap();
    }

    #[tokio::test]
    async fn test_send_structured_message() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"ok":true}"#);
        let mut telegram = Telegram::new("test-token".to_string());
        telegram.base_url = base_url;
        let message = SocialMessage {
            content: "hello".to_string(),
            attachments: vec![],
            embeds: vec![],
        };
        telegram.send_structured_message("chat", &message).await.unwrap();
    }
}
