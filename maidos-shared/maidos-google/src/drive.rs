//! Google Drive API implementation

use crate::{GoogleService, ServiceInfo, Auth, Result, GoogleError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Google Drive service
pub struct Drive {
    client: Client,
    auth: Auth,
    base_url: String,
}

/// Drive file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveFile {
    /// File ID
    pub id: Option<String>,
    
    /// File name
    pub name: Option<String>,
    
    /// MIME type
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    
    /// File size in bytes
    pub size: Option<String>,
    
    /// Creation time
    #[serde(rename = "createdTime")]
    pub created_time: Option<String>,
    
    /// Last modified time
    #[serde(rename = "modifiedTime")]
    pub modified_time: Option<String>,
    
    /// Parents folder IDs
    pub parents: Option<Vec<String>>,
    
    /// File permissions
    pub permissions: Option<Vec<Permission>>,
    
    /// File properties
    pub properties: Option<std::collections::HashMap<String, String>>,
}

/// File permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Permission ID
    pub id: Option<String>,
    
    /// Permission type (user, group, domain, anyone)
    #[serde(rename = "type")]
    pub permission_type: String,
    
    /// Permission role (owner, organizer, fileOrganizer, writer, commenter, reader)
    pub role: String,
    
    /// Email address of the user or group
    pub email_address: Option<String>,
}

/// File list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileList {
    /// List of files
    pub files: Vec<DriveFile>,
    
    /// Next page token
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

impl Drive {
    /// Create a new Drive service
    pub fn new(auth: Auth) -> Self {
        Self {
            client: Client::new(),
            auth,
            base_url: "https://www.googleapis.com/drive/v3".to_string(),
        }
    }

    /// List files
    pub async fn list_files(&self, query: Option<&str>, page_size: Option<u32>) -> Result<FileList> {
        let mut url = format!("{}/files", self.base_url);
        
        let mut params = Vec::new();
        if let Some(q) = query {
            params.push(format!("q={}", urlencoding::encode(q)));
        }
        if let Some(size) = page_size {
            params.push(format!("pageSize={}", size));
        }
        
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth.authorization_header())
            .send()
            .await?;

        if response.status().is_success() {
            let file_list: FileList = response.json().await?;
            Ok(file_list)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Drive API error {}: {}", status, error_text)))
        }
    }

    /// Get file metadata
    pub async fn get_file(&self, file_id: &str) -> Result<DriveFile> {
        let url = format!("{}/files/{}", self.base_url, file_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth.authorization_header())
            .send()
            .await?;

        if response.status().is_success() {
            let file: DriveFile = response.json().await?;
            Ok(file)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Drive API error {}: {}", status, error_text)))
        }
    }

    /// Create a new file
    pub async fn create_file(&self, file: &DriveFile, content: Option<&[u8]>) -> Result<DriveFile> {
        let url = format!("{}/files", self.base_url);
        
        let response = if let Some(file_content) = content {
            // Upload file with content
            let mut multipart = reqwest::multipart::Form::new();
            
            // Add metadata as JSON
            let metadata_json = serde_json::to_string(&file)?;
            multipart = multipart.text("metadata", metadata_json);
            
            // Add file content
            let part = reqwest::multipart::Part::bytes(file_content.to_vec())
                .file_name(file.name.clone().unwrap_or("file".to_string()));
            multipart = multipart.part("file", part);
            
            self.client
                .post(&url)
                .header("Authorization", self.auth.authorization_header())
                .query(&[("uploadType", "multipart")])
                .multipart(multipart)
                .send()
                .await?
        } else {
            // Create file metadata only
            self.client
                .post(&url)
                .header("Authorization", self.auth.authorization_header())
                .header("Content-Type", "application/json")
                .json(file)
                .send()
                .await?
        };

        if response.status().is_success() {
            let created_file: DriveFile = response.json().await?;
            Ok(created_file)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Drive API error {}: {}", status, error_text)))
        }
    }

    /// Update file metadata
    pub async fn update_file(&self, file_id: &str, file: &DriveFile) -> Result<DriveFile> {
        let url = format!("{}/files/{}", self.base_url, file_id);
        
        let response = self.client
            .patch(&url)
            .header("Authorization", self.auth.authorization_header())
            .header("Content-Type", "application/json")
            .json(file)
            .send()
            .await?;

        if response.status().is_success() {
            let updated_file: DriveFile = response.json().await?;
            Ok(updated_file)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Drive API error {}: {}", status, error_text)))
        }
    }

    /// Download file content
    pub async fn download_file(&self, file_id: &str) -> Result<Vec<u8>> {
        let url = format!("{}/files/{}?alt=media", self.base_url, file_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth.authorization_header())
            .send()
            .await?;

        if response.status().is_success() {
            let content = response.bytes().await?.to_vec();
            Ok(content)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Drive API error {}: {}", status, error_text)))
        }
    }

    /// Delete a file
    pub async fn delete_file(&self, file_id: &str) -> Result<()> {
        let url = format!("{}/files/{}", self.base_url, file_id);
        
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
            Err(GoogleError::Service(format!("Drive API error {}: {}", status, error_text)))
        }
    }

    /// Share a file
    pub async fn share_file(&self, file_id: &str, permission: &Permission) -> Result<Permission> {
        let url = format!("{}/files/{}/permissions", self.base_url, file_id);
        
        let response = self.client
            .post(&url)
            .header("Authorization", self.auth.authorization_header())
            .header("Content-Type", "application/json")
            .json(permission)
            .send()
            .await?;

        if response.status().is_success() {
            let created_permission: Permission = response.json().await?;
            Ok(created_permission)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Drive API error {}: {}", status, error_text)))
        }
    }
}

#[async_trait]
impl GoogleService for Drive {
    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            name: "Google Drive".to_string(),
            version: "v3".to_string(),
            base_url: self.base_url.clone(),
        }
    }
    
    async fn refresh_auth(&mut self) -> Result<()> {
        // No logic here because the existing logic was broken and we don't have
        // the client credentials required for a real refresh here.
        // The calling application should handle refresh via OAuth2Client.
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
    fn test_drive_creation() {
        let auth = Auth::new("access_token".to_string(), None, None);
        let drive = Drive::new(auth);
        assert_eq!(drive.service_info().name, "Google Drive");
    }

    #[test]
    fn test_drive_file_structure() {
        let file = DriveFile {
            id: Some("file123".to_string()),
            name: Some("test.txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            size: Some("1024".to_string()),
            created_time: Some("2023-10-15T14:30:00Z".to_string()),
            modified_time: Some("2023-10-15T14:30:00Z".to_string()),
            parents: Some(vec!["folder123".to_string()]),
            permissions: None,
            properties: None,
        };
        
        assert_eq!(file.name, Some("test.txt".to_string()));
        assert_eq!(file.mime_type, Some("text/plain".to_string()));
    }

    #[tokio::test]
    async fn test_list_files_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"files":[],"nextPageToken":null}"#);
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut drive = Drive::new(auth);
        drive.base_url = base_url;
        let list = drive.list_files(Some("name contains 'x'"), Some(10)).await.unwrap();
        assert!(list.files.is_empty());
    }

    #[tokio::test]
    async fn test_list_files_error() {
        let (base_url, _handle) = spawn_http_server("500 Internal Server Error", r#"{"error":"bad"}"#);
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut drive = Drive::new(auth);
        drive.base_url = base_url;
        let result = drive.list_files(None, None).await;
        assert!(matches!(result, Err(GoogleError::Service(_))));
    }

    #[tokio::test]
    async fn test_get_file_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"id":"f1","name":"n1"}"#);
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut drive = Drive::new(auth);
        drive.base_url = base_url;
        let file = drive.get_file("f1").await.unwrap();
        assert_eq!(file.id, Some("f1".to_string()));
    }

    #[tokio::test]
    async fn test_create_file_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"id":"f2","name":"n2"}"#);
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut drive = Drive::new(auth);
        drive.base_url = base_url;
        let file = DriveFile {
            id: None,
            name: Some("n2".to_string()),
            mime_type: None,
            size: None,
            created_time: None,
            modified_time: None,
            parents: None,
            permissions: None,
            properties: None,
        };
        let created = drive.create_file(&file, None).await.unwrap();
        assert_eq!(created.name, Some("n2".to_string()));
    }

    #[tokio::test]
    async fn test_delete_file_success() {
        let (base_url, _handle) = spawn_http_server("204 No Content", "");
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut drive = Drive::new(auth);
        drive.base_url = base_url;
        drive.delete_file("f1").await.unwrap();
    }

    #[tokio::test]
    async fn test_update_file_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", r#"{"id":"f1","name":"n1"}"#);
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut drive = Drive::new(auth);
        drive.base_url = base_url;
        let file = DriveFile {
            id: Some("f1".to_string()),
            name: Some("n1".to_string()),
            mime_type: None,
            size: None,
            created_time: None,
            modified_time: None,
            parents: None,
            permissions: None,
            properties: None,
        };
        let updated = drive.update_file("f1", &file).await.unwrap();
        assert_eq!(updated.id, Some("f1".to_string()));
    }

    #[tokio::test]
    async fn test_download_file_success() {
        let (base_url, _handle) = spawn_http_server("200 OK", "abc");
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut drive = Drive::new(auth);
        drive.base_url = base_url;
        let content = drive.download_file("f1").await.unwrap();
        assert_eq!(content, b"abc");
    }

    #[tokio::test]
    async fn test_share_file_success() {
        let (base_url, _handle) = spawn_http_server(
            "200 OK",
            r#"{"id":"p1","type":"user","role":"reader"}"#,
        );
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut drive = Drive::new(auth);
        drive.base_url = base_url;
        let permission = Permission {
            id: None,
            permission_type: "user".to_string(),
            role: "reader".to_string(),
            email_address: Some("a@example.com".to_string()),
        };
        let created = drive.share_file("f1", &permission).await.unwrap();
        assert_eq!(created.role, "reader");
    }
}
