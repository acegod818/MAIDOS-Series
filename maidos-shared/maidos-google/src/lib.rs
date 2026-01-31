//! MAIDOS Google API Integration
//!
//! Provides unified interfaces for Google services:
//! - OAuth2 authentication
//! - Google Calendar
//! - Gmail
//! - Google Contacts
//! - Google Drive
//!
//! # Example
//!
//! ```rust,no_run
//! use chrono::Utc;
//! use maidos_google::{Auth, Calendar, CalendarEvent, EventDateTime};
//!
//! async fn run() -> maidos_google::Result<()> {
//!     let auth = Auth::new("access_token".to_string(), None, None);
//!     let calendar = Calendar::new(auth);
//!
//!     let event = CalendarEvent {
//!         id: None,
//!         summary: "Demo".to_string(),
//!         description: None,
//!         start: EventDateTime {
//!             date_time: Some(Utc::now()),
//!             date: None,
//!             timezone: None,
//!         },
//!         end: EventDateTime {
//!             date_time: Some(Utc::now()),
//!             date: None,
//!             timezone: None,
//!         },
//!         location: None,
//!         attendees: vec![],
//!     };
//!
//!     let _created = calendar.create_event("primary", &event).await?;
//!     Ok(())
//! }
//! ```

mod oauth;
mod calendar;
mod gmail;
mod contacts;
mod drive;
mod error;

pub use oauth::{Auth, OAuth2Client, TokenResponse};
pub use calendar::{Calendar, CalendarEvent, EventDateTime, EventAttendee, CalendarListEntry};
pub use gmail::Gmail;
pub use contacts::Contacts;
pub use drive::Drive;
pub use error::{GoogleError, Result};

use async_trait::async_trait;

/// Common interface for Google services
#[async_trait]
pub trait GoogleService: Send + Sync {
    /// Get service information
    fn service_info(&self) -> ServiceInfo;
    
    /// Refresh authentication if needed
    async fn refresh_auth(&mut self) -> Result<()>;
}

/// Service information
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    
    /// API version
    pub version: String,
    
    /// Base URL for the service
    pub base_url: String,
}

/// Factory for creating Google service instances
pub struct GoogleFactory;

impl GoogleFactory {
    /// Create OAuth2 client
    pub fn create_oauth(
        client_id: String,
        client_secret: String,
    ) -> OAuth2Client {
        OAuth2Client::new(client_id, client_secret)
    }
    
    /// Create Calendar service
    pub fn create_calendar(auth: Auth) -> Calendar {
        Calendar::new(auth)
    }
    
    /// Create Gmail service
    pub fn create_gmail(auth: Auth) -> Gmail {
        Gmail::new(auth)
    }
    
    /// Create Contacts service
    pub fn create_contacts(auth: Auth) -> Contacts {
        Contacts::new(auth)
    }
    
    /// Create Drive service
    pub fn create_drive(auth: Auth) -> Drive {
        Drive::new(auth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_factory() {
        let oauth = GoogleFactory::create_oauth(
            "client_id".to_string(),
            "client_secret".to_string(),
        );
        
        // Just test that creation works without panicking
        assert_eq!(oauth.client_id(), "client_id");
    }
}
