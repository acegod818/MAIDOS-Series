//! Configuration management module
//!
//! Handles MAIDOS Forge configuration options

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod module;

/// MAIDOS Forge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeConfig {
    /// Compilation target platforms
    pub targets: Vec<String>,

    /// Default compilation mode
    pub default_mode: CompileMode,

    /// Whether to enable incremental compilation
    pub incremental: bool,

    /// Custom environment variables
    pub env_vars: HashMap<String, String>,

    /// Path configuration
    pub paths: PathConfig,

    /// Logging configuration
    pub logging: LogConfig,
}

/// Compilation mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompileMode {
    /// Debug mode
    Debug,

    /// Release mode
    Release,

    /// Custom mode
    Custom(String),
}

/// Path configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    /// Source code path
    pub source: String,

    /// Output path
    pub output: String,

    /// Temporary file path
    pub temp: String,

    /// Cache path
    pub cache: String,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Whether logging is enabled
    pub enabled: bool,

    /// Log level
    pub level: LogLevel,

    /// Log file path
    pub file: Option<String>,
}

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace level logging
    Trace,

    /// Debug level logging
    Debug,

    /// Info level logging
    Info,

    /// Warning level logging
    Warn,

    /// Error level logging
    Error,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            targets: vec![
                "x86_64-unknown-linux-gnu".to_string(),
                "x86_64-pc-windows-msvc".to_string(),
                "aarch64-apple-darwin".to_string(),
            ],
            default_mode: CompileMode::Debug,
            incremental: true,
            env_vars: HashMap::new(),
            paths: PathConfig {
                source: "./src".to_string(),
                output: "./dist".to_string(),
                temp: "./tmp".to_string(),
                cache: "./cache".to_string(),
            },
            logging: LogConfig {
                enabled: true,
                level: LogLevel::Info,
                file: None,
            },
        }
    }
}

impl ForgeConfig {
    /// Create a new configuration instance
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set compilation targets
    pub fn with_targets(mut self, targets: Vec<String>) -> Self {
        self.targets = targets;
        self
    }
    
    /// Set default compilation mode
    pub fn with_default_mode(mut self, mode: CompileMode) -> Self {
        self.default_mode = mode;
        self
    }
    
    /// Enable or disable incremental compilation
    pub fn with_incremental(mut self, incremental: bool) -> Self {
        self.incremental = incremental;
        self
    }
    
    /// Add an environment variable
    pub fn with_env_var(mut self, key: String, value: String) -> Self {
        self.env_vars.insert(key, value);
        self
    }
    
    /// Set source code path
    pub fn with_source_path(mut self, path: String) -> Self {
        self.paths.source = path;
        self
    }
    
    /// Set output path
    pub fn with_output_path(mut self, path: String) -> Self {
        self.paths.output = path;
        self
    }
    
    /// Load configuration from file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // [MAIDOS-AUDIT] Load configuration file
        tracing::info!("[MAIDOS-AUDIT] Loading configuration file from {}", path);

        // This should implement the logic to load configuration from file
        // For simplicity, we return the default configuration
        Ok(Self::default())
    }
    
    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // [MAIDOS-AUDIT] Save configuration file
        tracing::info!("[MAIDOS-AUDIT] Saving configuration to {}", path);

        // This should implement the logic to save configuration to file
        Ok(())
    }
}