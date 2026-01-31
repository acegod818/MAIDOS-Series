//! Google OAuth2 implementation

use crate::{GoogleService, ServiceInfo, Result, GoogleError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// OAuth2 client for Google services
pub struct OAuth2Client {
    client: Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    base_url: String,
}

/// Token response from Google OAuth2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// Access token
    pub access_token: String,
    
    /// Token type (usually "Bearer")
    pub token_type: String,
    
    /// Expires in seconds
    pub expires_in: u32,
    
    /// Refresh token (optional)
    pub refresh_token: Option<String>,
    
    /// Scope of the token
    pub scope: Option<String>,
}

/// Authentication state
#[derive(Debug, Clone)]
pub struct Auth {
    /// Access token
    pub access_token: String,
    
    /// Refresh token
    pub refresh_token: Option<String>,
    
    /// Expiration timestamp
    pub expires_at: Option<std::time::SystemTime>,
}

impl OAuth2Client {
    /// Create a new OAuth2 client
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: Client::new(),
            client_id,
            client_secret,
            redirect_uri: "http://localhost:8080/callback".to_string(),
            base_url: "https://accounts.google.com/o/oauth2".to_string(),
        }
    }

    /// Get the client ID
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Generate authorization URL
    pub fn authorize_url(&self, scope: &str) -> String {
        format!(
            "{}/auth?client_id={}&redirect_uri={}&scope={}&response_type=code&access_type=offline",
            self.base_url,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(scope)
        )
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse> {
        let url = format!("{}/token", self.base_url);
        
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
        ];

        let response = self.client
            .post(&url)
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: TokenResponse = response.json().await?;
            Ok(token_response)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::OAuth2(format!("OAuth2 error {}: {}", status, error_text)))
        }
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let url = format!("{}/token", self.base_url);
        
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let response = self.client
            .post(&url)
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: TokenResponse = response.json().await?;
            Ok(token_response)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::OAuth2(format!("Refresh token error {}: {}", status, error_text)))
        }
    }
}

#[async_trait]
impl GoogleService for OAuth2Client {
    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            name: "Google OAuth2".to_string(),
            version: "2.0".to_string(),
            base_url: self.base_url.clone(),
        }
    }
    
    async fn refresh_auth(&mut self) -> Result<()> {
        // OAuth2 client doesn't need to refresh itself
        Ok(())
    }
}

impl Auth {
    /// Create new authentication state
    pub fn new(access_token: String, refresh_token: Option<String>, expires_in: Option<u32>) -> Self {
        let expires_at = expires_in.map(|secs| {
            std::time::SystemTime::now() + std::time::Duration::from_secs(secs as u64)
        });
        
        Self {
            access_token,
            refresh_token,
            expires_at,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            std::time::SystemTime::now() > expires_at
        } else {
            false
        }
    }

    /// Get authorization header value
    pub fn authorization_header(&self) -> String {
        format!("Bearer {}", self.access_token)
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
    fn test_oauth_client_creation() {
        let oauth = OAuth2Client::new("client_id".to_string(), "client_secret".to_string());
        assert_eq!(oauth.client_id(), "client_id");
    }

    #[test]
    fn test_authorize_url() {
        let oauth = OAuth2Client::new("client_id".to_string(), "client_secret".to_string());
        let url = oauth.authorize_url("https://www.googleapis.com/auth/calendar");
        assert!(url.contains("client_id"));
        assert!(url.contains("calendar"));
    }

    #[test]
    fn test_auth_creation() {
        let auth = Auth::new("access_token".to_string(), Some("refresh_token".to_string()), Some(3600));
        assert_eq!(auth.access_token, "access_token");
        assert_eq!(auth.refresh_token, Some("refresh_token".to_string()));
    }

    #[test]
    fn test_auth_header_and_expiry() {
        let auth = Auth::new("token".to_string(), None, Some(0));
        assert!(auth.authorization_header().contains("Bearer token"));
        assert!(auth.is_expired());

        let auth = Auth::new("token".to_string(), None, None);
        assert!(!auth.is_expired());
    }

    #[tokio::test]
    async fn test_exchange_code_success() {
        let (base_url, _handle) = spawn_http_server(
            "200 OK",
            r#"{"access_token":"a","token_type":"Bearer","expires_in":3600}"#,
        );
        let mut oauth = OAuth2Client::new("id".to_string(), "secret".to_string());
        oauth.base_url = base_url;
        let token = oauth.exchange_code("code").await.unwrap();
        assert_eq!(token.access_token, "a");
        assert_eq!(token.token_type, "Bearer");
    }

    #[tokio::test]
    async fn test_refresh_token_success() {
        let (base_url, _handle) = spawn_http_server(
            "200 OK",
            r#"{"access_token":"b","token_type":"Bearer","expires_in":3600,"refresh_token":"r"}"#,
        );
        let mut oauth = OAuth2Client::new("id".to_string(), "secret".to_string());
        oauth.base_url = base_url;
        let token = oauth.refresh_token("r").await.unwrap();
        assert_eq!(token.access_token, "b");
        assert_eq!(token.refresh_token, Some("r".to_string()));
    }
}
