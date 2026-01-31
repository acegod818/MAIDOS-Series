//! Error types for maidos-config
//!
//! <impl>
//! WHAT: Unified error enum for all config operations
//! WHY: Single error type simplifies FFI and downstream handling
//! HOW: thiserror derive with specific variants for each failure mode
//! TEST: Each variant tested in integration tests
//! </impl>

use std::path::PathBuf;
use thiserror::Error;

/// All possible errors from maidos-config operations
#[derive(Error, Debug)]
pub enum ConfigError {
    /// File not found at specified path
    #[error("Config file not found: {0}")]
    FileNotFound(PathBuf),

    /// Failed to read file contents
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    /// TOML parsing failed
    #[error("Invalid TOML syntax: {0}")]
    ParseError(#[from] toml::de::Error),

    /// Required key missing from config
    #[error("Missing required key: {0}")]
    MissingKey(String),

    /// Type mismatch when accessing value
    #[error("Type mismatch for key '{key}': expected {expected}")]
    TypeMismatch { key: String, expected: String },

    /// Environment variable referenced but not set
    #[error("Environment variable not set: {0}")]
    EnvVarNotSet(String),

    /// Schema validation failed
    #[error("Validation failed: {0}")]
    ValidationError(String),

    /// File watcher setup failed
    #[error("Failed to setup file watcher: {0}")]
    WatcherError(String),
}

/// Result type alias for config operations
pub type Result<T> = std::result::Result<T, ConfigError>;
