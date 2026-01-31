//! Google Gmail API implementation

use crate::{GoogleService, ServiceInfo, Auth, Result, GoogleError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};

/// Google Gmail service
pub struct Gmail {
    client: Client,
    auth: Auth,
    base_url: String,
}

/// Email message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message ID
    pub id: Option<String>,
    
    /// Thread ID
    pub thread_id: Option<String>,
    
    /// Message label IDs
    #[serde(rename = "labelIds")]
    pub label_ids: Option<Vec<String>>,
    
    /// Message snippet
    pub snippet: Option<String>,
    
    /// Message payload
    pub payload: Option<MessagePart>,
    
    /// Message size estimate
    #[serde(rename = "sizeEstimate")]
    pub size_estimate: Option<u32>,
}

/// Message part (email body/attachment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePart {
    /// Part ID
    pub id: Option<String>,
    
    /// MIME type
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    
    /// Part filename
    pub filename: Option<String>,
    
    /// Part headers
    pub headers: Option<Vec<MessageHeader>>,
    
    /// Part body
    pub body: Option<MessageBody>,
    
    /// Nested parts
    pub parts: Option<Vec<MessagePart>>,
}

/// Message header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Header name
    pub name: String,
    
    /// Header value
    pub value: String,
}

/// Message body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBody {
    /// Body size
    pub size: Option<u32>,
    
    /// Base64-encoded body data
    pub data: Option<String>,
}

/// Email draft
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draft {
    /// Draft ID
    pub id: Option<String>,
    
    /// Message content
    pub message: Option<Message>,
}

/// Send email request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailRequest {
    /// Recipient email addresses
    pub to: Vec<String>,
    
    /// CC email addresses
    pub cc: Option<Vec<String>>,
    
    /// BCC email addresses
    pub bcc: Option<Vec<String>>,
    
    /// Subject line
    pub subject: String,
    
    /// Email body (plain text)
    pub body: String,
    
    /// HTML body (optional)
    pub html_body: Option<String>,
}

impl Gmail {
    /// Create a new Gmail service
    pub fn new(auth: Auth) -> Self {
        Self {
            client: Client::new(),
            auth,
            base_url: "https://www.googleapis.com/gmail/v1".to_string(),
        }
    }

    /// List messages
    pub async fn list_messages(&self, query: Option<&str>) -> Result<Vec<Message>> {
        let mut url = format!("{}/users/me/messages", self.base_url);
        
        if let Some(q) = query {
            url.push_str(&format!("?q={}", urlencoding::encode(q)));
        }
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth.authorization_header())
            .send()
            .await?;

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            let empty_vec = Vec::new();
            let messages_array = json.get("messages").and_then(|v| v.as_array()).unwrap_or(&empty_vec);
            
            let messages: Vec<Message> = messages_array
                .iter()
                .filter_map(|item| serde_json::from_value(item.clone()).ok())
                .collect();
                
            Ok(messages)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Gmail API error {}: {}", status, error_text)))
        }
    }

    /// Get a message by ID
    pub async fn get_message(&self, message_id: &str) -> Result<Message> {
        let url = format!("{}/users/me/messages/{}", self.base_url, message_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth.authorization_header())
            .send()
            .await?;

        if response.status().is_success() {
            let message: Message = response.json().await?;
            Ok(message)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Gmail API error {}: {}", status, error_text)))
        }
    }

    /// Send an email
    pub async fn send_email(&self, email: &SendEmailRequest) -> Result<String> {
        let url = format!("{}/users/me/messages/send", self.base_url);
        
        // Create RFC 2822 formatted email
        let mut email_content = format!("To: {}\r\n", email.to.join(", "));
        
        if let Some(cc) = &email.cc {
            email_content.push_str(&format!("Cc: {}\r\n", cc.join(", ")));
        }
        
        email_content.push_str(&format!("Subject: {}\r\n", email.subject));
        email_content.push_str("MIME-Version: 1.0\r\n");
        email_content.push_str("Content-Type: text/plain; charset=UTF-8\r\n\r\n");
        email_content.push_str(&email.body);
        
        // If HTML body is provided, create multipart email
        if let Some(html_body) = &email.html_body {
            email_content = format!("To: {}\r\n", email.to.join(", "));
            
            if let Some(cc) = &email.cc {
                email_content.push_str(&format!("Cc: {}\r\n", cc.join(", ")));
            }
            
            email_content.push_str(&format!("Subject: {}\r\n", email.subject));
            email_content.push_str("MIME-Version: 1.0\r\n");
            email_content.push_str("Content-Type: multipart/alternative; boundary=\"boundary\"\r\n\r\n");
            email_content.push_str("--boundary\r\n");
            email_content.push_str("Content-Type: text/plain; charset=UTF-8\r\n\r\n");
            email_content.push_str(&email.body);
            email_content.push_str("\r\n--boundary\r\n");
            email_content.push_str("Content-Type: text/html; charset=UTF-8\r\n\r\n");
            email_content.push_str(html_body);
            email_content.push_str("\r\n--boundary--\r\n");
        }
        
        // Base64 encode the email content
        let encoded_email = general_purpose::STANDARD.encode(email_content);
        
        let request_body = serde_json::json!({
            "raw": encoded_email
        });

        let response = self.client
            .post(&url)
            .header("Authorization", self.auth.authorization_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            let message_id = result.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            Ok(message_id)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Gmail API error {}: {}", status, error_text)))
        }
    }

    /// Create a draft
    pub async fn create_draft(&self, draft: &Draft) -> Result<Draft> {
        let url = format!("{}/users/me/drafts", self.base_url);
        
        let request_body = serde_json::json!({
            "message": draft.message
        });

        let response = self.client
            .post(&url)
            .header("Authorization", self.auth.authorization_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let created_draft: Draft = response.json().await?;
            Ok(created_draft)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Gmail API error {}: {}", status, error_text)))
        }
    }

    /// Delete a message
    pub async fn delete_message(&self, message_id: &str) -> Result<()> {
        let url = format!("{}/users/me/messages/{}", self.base_url, message_id);
        
        let response = self.client
            .delete(&url)
            .header("Authorization", self.auth.authorization_header())
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Gmail API error {}: {}", status, error_text)))
        }
    }
}

#[async_trait]
impl GoogleService for Gmail {
    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            name: "Gmail".to_string(),
            version: "v1".to_string(),
            base_url: self.base_url.clone(),
        }
    }
    
    async fn refresh_auth(&mut self) -> Result<()> {
        // In a real implementation, this would refresh the auth token if needed
        Ok(())
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
    fn test_gmail_creation() {
        let auth = Auth::new("access_token".to_string(), None, None);
        let gmail = Gmail::new(auth);
        assert_eq!(gmail.service_info().name, "Gmail");
    }

    #[test]
    fn test_send_email_request() {
        let request = SendEmailRequest {
            to: vec!["test@example.com".to_string()],
            cc: Some(vec!["cc@example.com".to_string()]),
            bcc: None,
            subject: "Test Subject".to_string(),
            body: "Test Body".to_string(),
            html_body: Some("<p>Test HTML Body</p>".to_string()),
        };
        
        assert_eq!(request.to[0], "test@example.com");
        assert_eq!(request.subject, "Test Subject");
    }

    #[tokio::test]
    async fn test_list_messages_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"messages":[{"id":"1"}]}"#);
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut gmail = Gmail::new(auth);
        gmail.base_url = base_url;
        let list = gmail.list_messages(Some("from:test")).await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_list_messages_error() {
        let (base_url, _handle) = spawn_http_server("500 Internal Server Error", r#"{"error":"bad"}"#);
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut gmail = Gmail::new(auth);
        gmail.base_url = base_url;
        let result = gmail.list_messages(None).await;
        assert!(matches!(result, Err(GoogleError::Service(_))));
    }

    #[tokio::test]
    async fn test_send_email_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"id":"m1"}"#);
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut gmail = Gmail::new(auth);
        gmail.base_url = base_url;
        let request = SendEmailRequest {
            to: vec!["test@example.com".to_string()],
            cc: None,
            bcc: None,
            subject: "Test".to_string(),
            body: "Body".to_string(),
            html_body: None,
        };
        let id = gmail.send_email(&request).await.unwrap();
        assert_eq!(id, "m1");
    }

    #[tokio::test]
    async fn test_get_message_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"id":"msg1","snippet":"hi"}"#);
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut gmail = Gmail::new(auth);
        gmail.base_url = base_url;
        let msg = gmail.get_message("msg1").await.unwrap();
        assert_eq!(msg.id, Some("msg1".to_string()));
    }

    #[tokio::test]
    async fn test_create_draft_success() {
        let (base_url, _handle) = spawn_http_server(
            "200 OK",
            r#"{"id":"d1","message":{"id":"m1"}}"#,
        );
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut gmail = Gmail::new(auth);
        gmail.base_url = base_url;
        let draft = Draft {
            id: None,
            message: Some(Message {
                id: None,
                thread_id: None,
                label_ids: None,
                snippet: None,
                payload: None,
                size_estimate: None,
            }),
        };
        let created = gmail.create_draft(&draft).await.unwrap();
        assert_eq!(created.id, Some("d1".to_string()));
    }

    #[tokio::test]
    async fn test_delete_message_success() {
        let (base_url, _handle) = spawn_http_server("204 No Content", "");
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut gmail = Gmail::new(auth);
        gmail.base_url = base_url;
        gmail.delete_message("msg1").await.unwrap();
    }
}
