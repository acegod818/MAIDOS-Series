//! Error types for maidos-auth
//!
//! <impl>
//! WHAT: Unified error enum for all auth operations
//! WHY: Single error type simplifies FFI and downstream handling
//! HOW: thiserror derive with specific variants for each failure mode
//! TEST: Each variant tested in integration tests
//! </impl>

use thiserror::Error;

/// All possible errors from maidos-auth operations
#[derive(Error, Debug)]
pub enum AuthError {
    /// Token has expired
    #[error("Token expired")]
    TokenExpired,

    /// Token signature is invalid
    #[error("Invalid token signature")]
    InvalidSignature,

    /// Token format is malformed
    #[error("Malformed token: {0}")]
    MalformedToken(String),

    /// Token is invalid or not found
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    /// Required capability not present in token
    #[error("Missing capability: {0:?}")]
    MissingCapability(crate::Capability),

    /// Secret key not configured
    #[error("Secret key not configured")]
    NoSecretKey,

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(#[from] maidos_config::ConfigError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias for auth operations
pub type Result<T> = std::result::Result<T, AuthError>;
