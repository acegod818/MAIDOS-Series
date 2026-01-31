//! MAIDOS Social Media API Integration
//!
//! Provides unified interfaces for popular social media platforms:
//! - Discord
//! - Slack
//! - Twitter/X
//! - LINE
//! - Telegram
//!
//! # Example
//!
//! ```rust,no_run
//! use maidos_social::Discord;
//!
//! async fn run() -> maidos_social::Result<()> {
//!     let token = "bot-token";
//!     let channel_id = "channel-id";
//!     let discord = Discord::new(token.to_string());
//!     discord.send_message(channel_id, "Hello from MAIDOS!").await?;
//!     Ok(())
//! }
//! ```

mod discord;
mod slack;
mod twitter;
mod line;
mod telegram;
mod error;

pub use discord::Discord;
pub use slack::Slack;
pub use twitter::Twitter;
pub use line::Line;
pub use telegram::Telegram;
pub use error::{SocialError, Result};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Generic social media message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMessage {
    /// Message content
    pub content: String,
    
    /// Optional attachments (URLs)
    #[serde(default)]
    pub attachments: Vec<String>,
    
    /// Optional embeds
    #[serde(default)]
    pub embeds: Vec<SocialEmbed>,
}

/// Embedded content for rich messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialEmbed {
    /// Title of the embed
    pub title: Option<String>,
    
    /// Description of the embed
    pub description: Option<String>,
    
    /// URL of the embed
    pub url: Option<String>,
    
    /// Color of the embed (in hex)
    pub color: Option<u32>,
    
    /// Footer text
    pub footer: Option<String>,
}

/// Common interface for all social media platforms
#[async_trait]
pub trait SocialPlatform: Send + Sync {
    /// Send a message to a channel/user
    async fn send_message(&self, target: &str, message: &str) -> Result<()>;
    
    /// Send a structured message with attachments/embeds
    async fn send_structured_message(&self, target: &str, message: &SocialMessage) -> Result<()>;
    
    /// Get platform-specific information
    fn platform_info(&self) -> PlatformInfo;
}

/// Platform information
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    /// Platform name
    pub name: String,
    
    /// API version
    pub version: String,
    
    /// Rate limits (requests per minute)
    pub rate_limit: u32,
}

/// Factory for creating social platform instances
pub struct SocialFactory;

impl SocialFactory {
    /// Create a Discord client
    pub fn create_discord(token: String) -> Discord {
        Discord::new(token)
    }
    
    /// Create a Slack client
    pub fn create_slack(token: String) -> Slack {
        Slack::new(token)
    }
    
    /// Create a Twitter client
    pub fn create_twitter(
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        access_token_secret: String,
    ) -> Twitter {
        Twitter::new(consumer_key, consumer_secret, access_token, access_token_secret)
    }
    
    /// Create a LINE client
    pub fn create_line(access_token: String) -> Line {
        Line::new(access_token)
    }
    
    /// Create a Telegram client
    pub fn create_telegram(token: String) -> Telegram {
        Telegram::new(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_social_message_creation() {
        let message = SocialMessage {
            content: "Hello World".to_string(),
            attachments: vec!["http://example.com/image.png".to_string()],
            embeds: vec![],
        };
        
        assert_eq!(message.content, "Hello World");
        assert_eq!(message.attachments.len(), 1);
    }

    #[test]
    fn test_social_factory() {
        let discord = SocialFactory::create_discord("test-token".to_string());
        let slack = SocialFactory::create_slack("test-token".to_string());
        let line = SocialFactory::create_line("test-token".to_string());
        let telegram = SocialFactory::create_telegram("test-token".to_string());
        
        // Just test that creation works without panicking
        assert_eq!(discord.token(), "test-token");
        assert_eq!(slack.token(), "test-token");
        assert_eq!(line.access_token(), "test-token");
        assert_eq!(telegram.token(), "test-token");
    }
}
