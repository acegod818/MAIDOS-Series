//! Error handling module
//!
//! Defines error types used in MAIDOS Forge

use thiserror::Error;

/// Forge error type
#[derive(Error, Debug)]
pub enum ForgeError {
    /// Compilation failed error
    #[error("Compilation failed: {message}")]
    CompilationFailed { message: String },

    /// Toolchain not found error
    #[error("Toolchain not found: {toolchain}")]
    ToolchainNotFound { toolchain: String },

    /// Input validation failed error
    #[error("Input validation failed: {message}")]
    ValidationError { message: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Filesystem error
    #[error("Filesystem error: {message}")]
    FileSystemError { message: String },

    /// Network error
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Internal error
    #[error("Internal error: {source}")]
    InternalError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>
    },

    /// Dependency error
    #[error("Dependency error: {message}")]
    DependencyError { message: String },

    /// Parse error
    #[error("Parse error: {message}")]
    ParseError { message: String },

    /// Timeout error
    #[error("Operation timed out: {message}")]
    TimeoutError { message: String },

    /// Unsupported language
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// Parser error
    #[error("Parser error: {0}")]
    ParserError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Compilation error
    #[error("Compilation error: {0}")]
    Compilation(String),

    /// Toolchain error
    #[error("Toolchain error: {0}")]
    Toolchain(String),
}

/// Configuration error type
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    /// Configuration format error
    #[error("Configuration format error: {message}")]
    FormatError { message: String },

    /// Missing required configuration key
    #[error("Missing required configuration key: {key}")]
    MissingKey { key: String },
}

/// Plugin error type
#[derive(Error, Debug)]
pub enum PluginError {
    /// Plugin not found
    #[error("Plugin not found: {name}")]
    PluginNotFound { name: String },

    /// Plugin load failed
    #[error("Plugin load failed: {message}")]
    LoadFailed { message: String },

    /// Plugin version incompatible
    #[error("Plugin version incompatible: {plugin_version} != {required_version}")]
    VersionMismatch {
        plugin_version: String,
        required_version: String
    },

    /// Plugin interface mismatch
    #[error("Plugin interface mismatch: {message}")]
    InterfaceMismatch { message: String },
}

impl From<ConfigError> for crate::compiler::CompilerError {
    fn from(error: ConfigError) -> Self {
        crate::compiler::CompilerError::ValidationError {
            message: error.to_string()
        }
    }
}

impl From<std::io::Error> for crate::compiler::CompilerError {
    fn from(error: std::io::Error) -> Self {
        crate::compiler::CompilerError::InternalError {
            source: Box::new(error)
        }
    }
}

impl From<serde_json::Error> for crate::compiler::CompilerError {
    fn from(error: serde_json::Error) -> Self {
        crate::compiler::CompilerError::ValidationError {
            message: error.to_string()
        }
    }
}

/// Result type aliases
pub type CompilerResult<T> = std::result::Result<T, crate::compiler::CompilerError>;
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;
pub type PluginResult<T> = std::result::Result<T, PluginError>;
pub type Result<T> = std::result::Result<T, ForgeError>;
