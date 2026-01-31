//! Google API integration errors

use thiserror::Error;

/// Google API integration errors
#[derive(Error, Debug)]
pub enum GoogleError {
    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization error
    #[error("JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// OAuth2 error
    #[error("OAuth2 error: {0}")]
    OAuth2(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Service-specific error
    #[error("Service error: {0}")]
    Service(String),
}

/// Result type for Google operations
pub type Result<T> = std::result::Result<T, GoogleError>;