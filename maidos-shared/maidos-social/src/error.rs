//! Social media integration errors

use thiserror::Error;

/// Social media integration errors
#[derive(Error, Debug)]
pub enum SocialError {
    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization error
    #[error("JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimit,

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Platform-specific error
    #[error("Platform error: {0}")]
    Platform(String),
}

/// Result type for social operations
pub type Result<T> = std::result::Result<T, SocialError>;