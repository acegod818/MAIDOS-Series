//! Twitter/X API integration

use crate::{SocialPlatform, PlatformInfo, SocialMessage, Result, SocialError};
use async_trait::async_trait;
use reqwest::Client;

/// Twitter API client
pub struct Twitter {
    client: Client,
    consumer_key: String,
    consumer_secret: String,
    access_token: String,
    access_token_secret: String,
    base_url: String,
}

impl Twitter {
    /// Create a new Twitter client
    pub fn new(
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        access_token_secret: String,
    ) -> Self {
        Self {
            client: Client::new(),
            consumer_key,
            consumer_secret,
            access_token,
            access_token_secret,
            base_url: "https://api.twitter.com/2".to_string(),
        }
    }

    /// Post a tweet
    pub async fn post_tweet(&self, content: &str) -> Result<String> {
        let url = format!("{}/tweets", self.base_url);
        
        let request_body = serde_json::json!({
            "text": content
        });

        let response = self.client
            .post(&url)
            .header("Authorization", self.build_bearer_token())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;
        
        if let Some(data) = response_json.get("data") {
            if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                Ok(id.to_string())
            } else {
                Err(SocialError::Platform("Tweet posted but no ID returned".to_string()))
            }
        } else {
            let error = response_json.get("errors")
                .and_then(|errs| errs.as_array())
                .and_then(|errs| errs.first())
                .and_then(|err| err.get("detail"))
                .and_then(|detail| detail.as_str())
                .unwrap_or("Unknown error");
            Err(SocialError::Platform(format!("Twitter API error: {}", error)))
        }
    }

    /// Build bearer token for authentication
    fn build_bearer_token(&self) -> String {
        // Touch credentials to avoid dead-code warnings; real OAuth 1.0a can be added later.
        let _ = (&self.consumer_key, &self.consumer_secret, &self.access_token_secret);
        // In a real implementation, this would use OAuth 1.0a
        // For simplicity, we're returning a placeholder
        format!("Bearer {}", self.access_token)
    }
}

#[async_trait]
impl SocialPlatform for Twitter {
    async fn send_message(&self, _target: &str, message: &str) -> Result<()> {
        self.post_tweet(message).await?;
        Ok(())
    }

    async fn send_structured_message(&self, _target: &str, message: &SocialMessage) -> Result<()> {
        // Twitter doesn't support rich messages in the same way
        // We'll just post the content as a tweet
        self.post_tweet(&message.content).await?;
        Ok(())
    }

    fn platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            name: "Twitter".to_string(),
            version: "2".to_string(),
            rate_limit: 300, // 300 requests per 15 minutes
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
    fn test_twitter_creation() {
        let _twitter = Twitter::new(
            "consumer_key".to_string(),
            "consumer_secret".to_string(),
            "access_token".to_string(),
            "access_token_secret".to_string(),
        );
        // Just test that creation works without panicking
    }

    #[test]
    fn test_platform_info() {
        let twitter = Twitter::new(
            "consumer_key".to_string(),
            "consumer_secret".to_string(),
            "access_token".to_string(),
            "access_token_secret".to_string(),
        );
        let info = twitter.platform_info();
        assert_eq!(info.name, "Twitter");
        assert_eq!(info.version, "2");
        assert_eq!(info.rate_limit, 300);
    }

    #[tokio::test]
    async fn test_post_tweet_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"data":{"id":"123"}}"#);
        let mut twitter = Twitter::new(
            "consumer_key".to_string(),
            "consumer_secret".to_string(),
            "access_token".to_string(),
            "access_token_secret".to_string(),
        );
        twitter.base_url = base_url;
        let id = twitter.post_tweet("hello").await.unwrap();
        assert_eq!(id, "123");
    }

    #[tokio::test]
    async fn test_post_tweet_error() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"errors":[{"detail":"bad"}]}"#);
        let mut twitter = Twitter::new(
            "consumer_key".to_string(),
            "consumer_secret".to_string(),
            "access_token".to_string(),
            "access_token_secret".to_string(),
        );
        twitter.base_url = base_url;
        let result = twitter.post_tweet("hello").await;
        assert!(matches!(result, Err(SocialError::Platform(_))));
    }

    #[test]
    fn test_bearer_token() {
        let twitter = Twitter::new(
            "consumer_key".to_string(),
            "consumer_secret".to_string(),
            "access_token".to_string(),
            "access_token_secret".to_string(),
        );
        let token = twitter.build_bearer_token();
        assert!(token.starts_with("Bearer "));
    }
}
