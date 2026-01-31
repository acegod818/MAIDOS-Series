//! Error types for maidos-bus
//!
//! <impl>
//! WHAT: Define BusError enum for all bus operations
//! WHY: Type-safe error handling with detailed context
//! HOW: thiserror derive for automatic Error impl
//! TEST: Error construction and display
//! </impl>

use std::io;
use thiserror::Error;

/// Result type alias for bus operations
pub type Result<T> = std::result::Result<T, BusError>;

/// Errors that can occur during bus operations
#[derive(Debug, Error)]
pub enum BusError {
    /// IO error during socket operations
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Connection error
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Channel closed
    #[error("Channel closed")]
    ChannelClosed,

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Timeout during operation
    #[error("Operation timed out")]
    Timeout,

    /// Bus already started
    #[error("Bus already running")]
    AlreadyRunning,

    /// Bus not running
    #[error("Bus not running")]
    NotRunning,

    /// Topic validation error
    #[error("Invalid topic: {0}")]
    InvalidTopic(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
}

impl From<rmp_serde::encode::Error> for BusError {
    fn from(e: rmp_serde::encode::Error) -> Self {
        BusError::Serialization(e.to_string())
    }
}

impl From<rmp_serde::decode::Error> for BusError {
    fn from(e: rmp_serde::decode::Error) -> Self {
        BusError::Deserialization(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BusError::ConnectionFailed("host unreachable".to_string());
        assert!(err.to_string().contains("host unreachable"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let bus_err: BusError = io_err.into();
        assert!(matches!(bus_err, BusError::Io(_)));
    }

    #[test]
    fn test_result_type() {
        let ok_result: Result<i32> = Ok(42);
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: Result<i32> = Err(BusError::Timeout);
        assert!(matches!(err_result.unwrap_err(), BusError::Timeout));
    }
}
