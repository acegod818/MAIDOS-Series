//! Google Calendar API implementation

use crate::{GoogleService, ServiceInfo, Auth, Result, GoogleError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Google Calendar service
pub struct Calendar {
    client: Client,
    auth: Auth,
    base_url: String,
}

/// Calendar event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    /// Event ID
    pub id: Option<String>,
    
    /// Event summary/title
    pub summary: String,
    
    /// Event description
    pub description: Option<String>,
    
    /// Start time
    pub start: EventDateTime,
    
    /// End time
    pub end: EventDateTime,
    
    /// Location
    pub location: Option<String>,
    
    /// Attendees
    #[serde(default)]
    pub attendees: Vec<EventAttendee>,
}

/// Event date/time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDateTime {
    /// Date-time in RFC3339 format
    #[serde(rename = "dateTime")]
    pub date_time: Option<DateTime<Utc>>,
    
    /// Date in YYYY-MM-DD format
    pub date: Option<String>,
    
    /// Timezone
    pub timezone: Option<String>,
}

/// Event attendee
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAttendee {
    /// Email address
    pub email: String,
    
    /// Display name
    pub display_name: Option<String>,
}

/// Calendar list entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarListEntry {
    /// Calendar ID
    pub id: String,
    
    /// Calendar summary
    pub summary: String,
    
    /// Primary calendar
    pub primary: Option<bool>,
}

impl Calendar {
    /// Create a new Calendar service
    pub fn new(auth: Auth) -> Self {
        Self {
            client: Client::new(),
            auth,
            base_url: "https://www.googleapis.com/calendar/v3".to_string(),
        }
    }

    /// List calendars
    pub async fn list_calendars(&self) -> Result<Vec<CalendarListEntry>> {
        let url = format!("{}/users/me/calendarList", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth.authorization_header())
            .send()
            .await?;

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            let empty_vec = Vec::new();
            let items = json.get("items").and_then(|v| v.as_array()).unwrap_or(&empty_vec);
            
            let calendars: Vec<CalendarListEntry> = items
                .iter()
                .filter_map(|item| serde_json::from_value(item.clone()).ok())
                .collect();
                
            Ok(calendars)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Calendar API error {}: {}", status, error_text)))
        }
    }

    /// Create a new event
    pub async fn create_event(&self, calendar_id: &str, event: &CalendarEvent) -> Result<CalendarEvent> {
        let url = format!("{}/calendars/{}/events", self.base_url, calendar_id);
        
        let response = self.client
            .post(&url)
            .header("Authorization", self.auth.authorization_header())
            .header("Content-Type", "application/json")
            .json(event)
            .send()
            .await?;

        if response.status().is_success() {
            let created_event: CalendarEvent = response.json().await?;
            Ok(created_event)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Calendar API error {}: {}", status, error_text)))
        }
    }

    /// Get an event
    pub async fn get_event(&self, calendar_id: &str, event_id: &str) -> Result<CalendarEvent> {
        let url = format!("{}/calendars/{}/events/{}", self.base_url, calendar_id, event_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth.authorization_header())
            .send()
            .await?;

        if response.status().is_success() {
            let event: CalendarEvent = response.json().await?;
            Ok(event)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Calendar API error {}: {}", status, error_text)))
        }
    }

    /// Update an event
    pub async fn update_event(&self, calendar_id: &str, event_id: &str, event: &CalendarEvent) -> Result<CalendarEvent> {
        let url = format!("{}/calendars/{}/events/{}", self.base_url, calendar_id, event_id);
        
        let response = self.client
            .put(&url)
            .header("Authorization", self.auth.authorization_header())
            .header("Content-Type", "application/json")
            .json(event)
            .send()
            .await?;

        if response.status().is_success() {
            let updated_event: CalendarEvent = response.json().await?;
            Ok(updated_event)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Service(format!("Calendar API error {}: {}", status, error_text)))
        }
    }

    /// Delete an event
    pub async fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<()> {
        let url = format!("{}/calendars/{}/events/{}", self.base_url, calendar_id, event_id);
        
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
            Err(GoogleError::Service(format!("Calendar API error {}: {}", status, error_text)))
        }
    }
}

#[async_trait]
impl GoogleService for Calendar {
    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            name: "Google Calendar".to_string(),
            version: "v3".to_string(),
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
    use chrono::TimeZone;
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
    fn test_calendar_creation() {
        let auth = Auth::new("access_token".to_string(), None, None);
        let calendar = Calendar::new(auth);
        assert_eq!(calendar.service_info().name, "Google Calendar");
    }

    #[test]
    fn test_event_datetime_serialization() {
        let dt = Utc.with_ymd_and_hms(2023, 10, 15, 14, 30, 0).unwrap();
        let event_dt = EventDateTime {
            date_time: Some(dt),
            date: None,
            timezone: Some("UTC".to_string()),
        };
        
        let json = serde_json::to_string(&event_dt).unwrap();
        assert!(json.contains("dateTime"));
    }

    #[tokio::test]
    async fn test_list_calendars_success() {
        let (base_url, _handle) = spawn_http_server(
            "200 OK",
            r#"{"items":[{"id":"1","summary":"S","primary":true}]}"#,
        );
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut calendar = Calendar::new(auth);
        calendar.base_url = base_url;
        let calendars = calendar.list_calendars().await.unwrap();
        assert_eq!(calendars.len(), 1);
        assert_eq!(calendars[0].id, "1");
    }

    #[tokio::test]
    async fn test_list_calendars_error() {
        let (base_url, _handle) = spawn_http_server("500 Internal Server Error", r#"{"error":"bad"}"#);
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut calendar = Calendar::new(auth);
        calendar.base_url = base_url;
        let result = calendar.list_calendars().await;
        assert!(matches!(result, Err(GoogleError::Service(_))));
    }

    #[tokio::test]
    async fn test_create_event_success() {
        let (base_url, _handle) = spawn_http_server(
            "200 OK",
            r#"{"id":"e1","summary":"S","start":{"dateTime":"2023-01-01T00:00:00Z"},"end":{"dateTime":"2023-01-01T01:00:00Z"}}"#,
        );
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut calendar = Calendar::new(auth);
        calendar.base_url = base_url;
        let event = CalendarEvent {
            id: None,
            summary: "S".to_string(),
            description: None,
            start: EventDateTime {
                date_time: Some(Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap()),
                date: None,
                timezone: None,
            },
            end: EventDateTime {
                date_time: Some(Utc.with_ymd_and_hms(2023, 1, 1, 1, 0, 0).unwrap()),
                date: None,
                timezone: None,
            },
            location: None,
            attendees: vec![],
        };
        let created = calendar.create_event("cal", &event).await.unwrap();
        assert_eq!(created.id, Some("e1".to_string()));
    }

    #[tokio::test]
    async fn test_delete_event_success() {
        let (base_url, _handle) = spawn_http_server("204 No Content", "");
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut calendar = Calendar::new(auth);
        calendar.base_url = base_url;
        calendar.delete_event("cal", "e1").await.unwrap();
    }

    #[tokio::test]
    async fn test_get_event_success() {
        let (base_url, _handle) = spawn_http_server(
            "200 OK",
            r#"{"id":"e2","summary":"S","start":{"dateTime":"2023-01-01T00:00:00Z"},"end":{"dateTime":"2023-01-01T01:00:00Z"}}"#,
        );
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut calendar = Calendar::new(auth);
        calendar.base_url = base_url;
        let event = calendar.get_event("cal", "e2").await.unwrap();
        assert_eq!(event.id, Some("e2".to_string()));
    }

    #[tokio::test]
    async fn test_update_event_success() {
        let (base_url, _handle) = spawn_http_server(
            "200 OK",
            r#"{"id":"e3","summary":"S","start":{"dateTime":"2023-01-01T00:00:00Z"},"end":{"dateTime":"2023-01-01T01:00:00Z"}}"#,
        );
        let auth = Auth::new("access_token".to_string(), None, None);
        let mut calendar = Calendar::new(auth);
        calendar.base_url = base_url;
        let event = CalendarEvent {
            id: Some("e3".to_string()),
            summary: "S".to_string(),
            description: None,
            start: EventDateTime {
                date_time: Some(Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap()),
                date: None,
                timezone: None,
            },
            end: EventDateTime {
                date_time: Some(Utc.with_ymd_and_hms(2023, 1, 1, 1, 0, 0).unwrap()),
                date: None,
                timezone: None,
            },
            location: None,
            attendees: vec![],
        };
        let updated = calendar.update_event("cal", "e3", &event).await.unwrap();
        assert_eq!(updated.id, Some("e3".to_string()));
    }
}
