//! Discord API integration

use crate::{SocialPlatform, PlatformInfo, SocialMessage, Result, SocialError};
use async_trait::async_trait;
use reqwest::Client;
use tracing::info;

/// Discord API client
pub struct Discord {
    client: Client,
    token: String,
    base_url: String,
}

impl Discord {
    /// Create a new Discord client
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
            base_url: "https://discord.com/api/v10".to_string(),
        }
    }

    /// Get the bot token
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Send a message to a channel
    pub async fn send_message(&self, channel_id: &str, content: &str) -> Result<()> {
        info!("[MAIDOS-AUDIT] Discord: Sending message to channel {}", channel_id);
        let url = format!("{}/channels/{}/messages", self.base_url, channel_id);
        
        let request_body = serde_json::json!({
            "content": content
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            info!("[MAIDOS-AUDIT] Discord: Message sent successfully to {}", channel_id);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            info!("[MAIDOS-AUDIT] Discord: Failed to send message to {}: {}", channel_id, status);
            Err(SocialError::Platform(format!("Discord API error {}: {}", status, error_text)))
        }
    }

    /// Send a rich message with embeds
    pub async fn send_rich_message(&self, channel_id: &str, message: &SocialMessage) -> Result<()> {
        info!("[MAIDOS-AUDIT] Discord: Sending rich message to channel {}", channel_id);
        let url = format!("{}/channels/{}/messages", self.base_url, channel_id);
        
        let mut embeds = Vec::new();
        for embed in &message.embeds {
            embeds.push(serde_json::json!({
                "title": embed.title,
                "description": embed.description,
                "url": embed.url,
                "color": embed.color,
                "footer": embed.footer.as_ref().map(|text| serde_json::json!({"text": text}))
            }));
        }

        let request_body = serde_json::json!({
            "content": message.content,
            "embeds": embeds
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            info!("[MAIDOS-AUDIT] Discord: Rich message sent successfully to {}", channel_id);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            info!("[MAIDOS-AUDIT] Discord: Failed to send rich message to {}: {}", channel_id, status);
            Err(SocialError::Platform(format!("Discord API error {}: {}", status, error_text)))
        }
    }
}

#[async_trait]
impl SocialPlatform for Discord {
    async fn send_message(&self, target: &str, message: &str) -> Result<()> {
        self.send_message(target, message).await
    }

    async fn send_structured_message(&self, target: &str, message: &SocialMessage) -> Result<()> {
        self.send_rich_message(target, message).await
    }

    fn platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            name: "Discord".to_string(),
            version: "10".to_string(),
            rate_limit: 50, // 50 requests per minute
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
    fn test_discord_creation() {
        let discord = Discord::new("test-token".to_string());
        assert_eq!(discord.token(), "test-token");
    }

    #[test]
    fn test_platform_info() {
        let discord = Discord::new("test-token".to_string());
        let info = discord.platform_info();
        assert_eq!(info.name, "Discord");
        assert_eq!(info.version, "10");
        assert_eq!(info.rate_limit, 50);
    }

    #[tokio::test]
    async fn test_send_message_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"id":"1"}"#);
        let mut discord = Discord::new("test-token".to_string());
        discord.base_url = base_url;
        discord.send_message("chan", "hi").await.unwrap();
    }

    #[tokio::test]
    async fn test_send_message_error() {
        let (base_url, _handle) = spawn_http_server("500 Internal Server Error", r#"{"error":"bad"}"#);
        let mut discord = Discord::new("test-token".to_string());
        discord.base_url = base_url;
        let result = discord.send_message("chan", "hi").await;
        assert!(matches!(result, Err(SocialError::Platform(_))));
    }

    #[tokio::test]
    async fn test_send_rich_message_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"id":"1"}"#);
        let mut discord = Discord::new("test-token".to_string());
        discord.base_url = base_url;
        let message = SocialMessage {
            content: "hello".to_string(),
            attachments: vec![],
            embeds: vec![crate::SocialEmbed {
                title: Some("title".to_string()),
                description: Some("desc".to_string()),
                url: None,
                color: Some(123),
                footer: Some("footer".to_string()),
            }],
        };
        discord.send_rich_message("chan", &message).await.unwrap();
    }
}
