//! Google Contacts API implementation

use crate::{GoogleService, ServiceInfo, Auth, Result, GoogleError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Google Contacts service
pub struct Contacts {
    client: Client,
    auth: Auth,
    base_url: String,
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// Contact ID
    pub id: Option<String>,
    
    /// Contact metadata
    pub metadata: Option<ContactMetadata>,
    
    /// Names associated with the contact
    #[serde(default)]
    pub names: Vec<Name>,
    
    /// Email addresses
    #[serde(default)]
    pub email_addresses: Vec<EmailAddress>,
    
    /// Phone numbers
    #[serde(default)]
    pub phone_numbers: Vec<PhoneNumber>,
    
    /// Addresses
    #[serde(default)]
    pub addresses: Vec<Address>,
    
    /// Organizations
    #[serde(default)]
    pub organizations: Vec<Organization>,
}

/// Contact metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactMetadata {
    /// Resource name
    #[serde(rename = "resourceName")]
    pub resource_name: Option<String>,
    
    /// Creation timestamp
    #[serde(rename = "createTime")]
    pub create_time: Option<String>,
    
    /// Last update timestamp
    #[serde(rename = "updateTime")]
    pub update_time: Option<String>,
}

/// Name information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name {
    /// Metadata
    pub metadata: Option<FieldMetadata>,
    
    /// Display name
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    
    /// Family name
    #[serde(rename = "familyName")]
    pub family_name: Option<String>,
    
    /// Given name
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
}

/// Email address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    /// Metadata
    pub metadata: Option<FieldMetadata>,
    
    /// Email address value
    pub value: String,
    
    /// Email address type (home, work, etc.)
    #[serde(rename = "type")]
    pub email_type: Option<String>,
}

/// Phone number
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneNumber {
    /// Metadata
    pub metadata: Option<FieldMetadata>,
    
    /// Phone number value
    pub value: String,
    
    /// Phone number type (home, work, mobile, etc.)
    #[serde(rename = "type")]
    pub phone_type: Option<String>,
}

/// Address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    /// Metadata
    pub metadata: Option<FieldMetadata>,
    
    /// Address type (home, work, etc.)
    #[serde(rename = "type")]
    pub address_type: Option<String>,
    
    /// Formatted address
    #[serde(rename = "formattedValue")]
    pub formatted_value: Option<String>,
    
    /// Street address
    #[serde(rename = "streetAddress")]
    pub street_address: Option<String>,
    
    /// City
    pub city: Option<String>,
    
    /// Region (state/province)
    pub region: Option<String>,
    
    /// Postal code
    #[serde(rename = "postalCode")]
    pub postal_code: Option<String>,
    
    /// Country
    pub country: Option<String>,
}

/// Organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// Metadata
    pub metadata: Option<FieldMetadata>,
    
    /// Organization name
    pub name: Option<String>,
    
    /// Job title
    #[serde(rename = "title")]
    pub job_title: Option<String>,
    
    /// Department
    pub department: Option<String>,
}

/// Field metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMetadata {
    /// Primary field
    pub primary: Option<bool>,
    
    /// Verified field
    pub verified: Option<bool>,
}

impl Contacts {
    /// Create a new Contacts service
    pub fn new(auth: Auth) -> Self {
        Self {
            client: Client::new(),
            auth,
            base_url: "https://people.googleapis.com/v1".to_string(),
        }
    }

    /// List contacts
    pub async fn list_contacts(&self, page_size: Option<u32>) -> Result<Vec<Contact>> {
        let mut url = format!("{}/people/me/connections", self.base_url);
        url.push_str("?personFields=names,emailAddresses,phoneNumbers,addresses,organizations");
        
        if let Some(size) = page_size {
            url.push_str(&format!("&pageSize={}", size));
        }
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth.authorization_header())
            .send()
            .await?;

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            let empty_vec = Vec::new();
            let connections = json.get("connections").and_then(|v| v.as_array()).unwrap_or(&empty_vec);
            
            let contacts: Vec<Contact> = connections
                .iter()
                .filter_map(|item| serde_json::from_value(item.clone()).ok())
                .collect();
                
            Ok(contacts)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Contacts API error {}: {}", status, error_text)))
        }
    }

    /// Get a contact by resource name
    pub async fn get_contact(&self, resource_name: &str) -> Result<Contact> {
        let url = format!("{}/{}", self.base_url, resource_name);
        let url = format!("{}?personFields=names,emailAddresses,phoneNumbers,addresses,organizations", url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth.authorization_header())
            .send()
            .await?;

        if response.status().is_success() {
            let contact: Contact = response.json().await?;
            Ok(contact)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Contacts API error {}: {}", status, error_text)))
        }
    }

    /// Create a new contact
    pub async fn create_contact(&self, contact: &Contact) -> Result<Contact> {
        let url = format!("{}/people:createContact", self.base_url);
        
        let request_body = serde_json::json!({
            "names": contact.names,
            "emailAddresses": contact.email_addresses,
            "phoneNumbers": contact.phone_numbers,
            "addresses": contact.addresses,
            "organizations": contact.organizations
        });

        let response = self.client
            .post(&url)
            .header("Authorization", self.auth.authorization_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let created_contact: Contact = response.json().await?;
            Ok(created_contact)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Contacts API error {}: {}", status, error_text)))
        }
    }

    /// Update a contact
    pub async fn update_contact(&self, resource_name: &str, contact: &Contact) -> Result<Contact> {
        let url = format!("{}/{}:updateContact", self.base_url, resource_name);
        
        let request_body = serde_json::json!({
            "names": contact.names,
            "emailAddresses": contact.email_addresses,
            "phoneNumbers": contact.phone_numbers,
            "addresses": contact.addresses,
            "organizations": contact.organizations
        });

        let response = self.client
            .patch(&url)
            .header("Authorization", self.auth.authorization_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let updated_contact: Contact = response.json().await?;
            Ok(updated_contact)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Contacts API error {}: {}", status, error_text)))
        }
    }

    /// Delete a contact
    pub async fn delete_contact(&self, resource_name: &str) -> Result<()> {
        let url = format!("{}/{}:deleteContact", self.base_url, resource_name);
        
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
            Err(GoogleError::Service(format!("Contacts API error {}: {}", status, error_text)))
        }
    }
}

#[async_trait]
impl GoogleService for Contacts {
    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            name: "Google Contacts".to_string(),
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
    fn test_contacts_creation() {
        let auth = Auth::new("access_token".to_string(), None, None);
        let contacts = Contacts::new(auth);
        assert_eq!(contacts.service_info().name, "Google Contacts");
    }

    #[test]
    fn test_contact_structures() {
        let contact = Contact {
            id: Some("contact123".to_string()),
            metadata: None,
            names: vec![Name {
                metadata: None,
                display_name: Some("John Doe".to_string()),
                family_name: Some("Doe".to_string()),
                given_name: Some("John".to_string()),
            }],
            email_addresses: vec![EmailAddress {
                metadata: None,
                value: "john@example.com".to_string(),
                email_type: Some("work".to_string()),
            }],
            phone_numbers: vec![],
            addresses: vec![],
            organizations: vec![],
        };
        
        assert_eq!(contact.names[0].display_name, Some("John Doe".to_string()));
        assert_eq!(contact.email_addresses[0].value, "john@example.com");
    }

    #[tokio::test]
    async fn test_list_contacts_success() {
        let (base_url, _handle) = spawn_http_server(
            "200 OK",
            r#"{"connections":[{"id":"c1","names":[{"displayName":"A"}]}]}"#,
        );
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut contacts = Contacts::new(auth);
        contacts.base_url = base_url;
        let list = contacts.list_contacts(Some(5)).await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_list_contacts_error() {
        let (base_url, _handle) = spawn_http_server("500 Internal Server Error", r#"{"error":"bad"}"#);
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut contacts = Contacts::new(auth);
        contacts.base_url = base_url;
        let result = contacts.list_contacts(None).await;
        assert!(matches!(result, Err(GoogleError::Service(_))));
    }

    #[tokio::test]
    async fn test_get_contact_success() {
        let (base_url, _handle) = spawn_http_server(
            "200 OK",
            r#"{"id":"c1","names":[{"displayName":"A"}]}"#,
        );
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut contacts = Contacts::new(auth);
        contacts.base_url = base_url;
        let contact = contacts.get_contact("people/c1").await.unwrap();
        assert_eq!(contact.id, Some("c1".to_string()));
    }

    #[tokio::test]
    async fn test_create_contact_success() {
        let (base_url, _handle) = spawn_http_server(
            "200 OK",
            r#"{"id":"c2","names":[{"displayName":"B"}]}"#,
        );
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut contacts = Contacts::new(auth);
        contacts.base_url = base_url;
        let contact = Contact {
            id: None,
            metadata: None,
            names: vec![Name {
                metadata: None,
                display_name: Some("B".to_string()),
                family_name: None,
                given_name: None,
            }],
            email_addresses: vec![],
            phone_numbers: vec![],
            addresses: vec![],
            organizations: vec![],
        };
        let created = contacts.create_contact(&contact).await.unwrap();
        assert_eq!(created.id, Some("c2".to_string()));
    }

    #[tokio::test]
    async fn test_update_contact_success() {
        let (base_url, _handle) = spawn_http_server(
            "200 OK",
            r#"{"id":"c3","names":[{"displayName":"C"}]}"#,
        );
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut contacts = Contacts::new(auth);
        contacts.base_url = base_url;
        let contact = Contact {
            id: Some("c3".to_string()),
            metadata: None,
            names: vec![Name {
                metadata: None,
                display_name: Some("C".to_string()),
                family_name: None,
                given_name: None,
            }],
            email_addresses: vec![],
            phone_numbers: vec![],
            addresses: vec![],
            organizations: vec![],
        };
        let updated = contacts.update_contact("people/c3", &contact).await.unwrap();
        assert_eq!(updated.id, Some("c3".to_string()));
    }

    #[tokio::test]
    async fn test_delete_contact_success() {
        let (base_url, _handle) = spawn_http_server("204 No Content", "");
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut contacts = Contacts::new(auth);
        contacts.base_url = base_url;
        contacts.delete_contact("people/c3").await.unwrap();
    }
}
