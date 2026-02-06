//! MAIDOS configuration module
//!
//! This module provides configuration management functionality, including:
//! - Reading and parsing configuration files
//! - Managing user dictionaries
//! - Managing model settings

pub mod config;
pub mod dict;
pub mod model;

use std::path::PathBuf;

/// MAIDOS configuration structure
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MaidosConfig {
    /// Model configuration
    pub model: ModelConfig,
    /// IME configuration
    pub ime: ImeConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Feature toggles
    pub features: FeaturesConfig,
}

/// Model configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ModelConfig {
    /// Provider
    pub provider: String,
    /// Model name
    pub model: String,
    /// Maximum token count
    pub max_tokens: u32,
}

/// IME configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ImeConfig {
    /// Default input scheme
    pub default_scheme: String,
    /// Charset (Simplified or Traditional)
    pub charset: Charset,
    /// List of enabled input schemes
    pub enabled_schemes: Vec<String>,
}

/// Charset type
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Charset {
    /// Simplified Chinese
    #[serde(alias = "simplified")]
    Simplified,
    /// Traditional Chinese
    #[serde(alias = "traditional")]
    Traditional,
}

/// Security configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SecurityConfig {
    /// Whether to collect data
    pub data_collection: bool,
    /// Whether to prioritize privacy
    pub privacy_first: bool,
}

/// Feature toggle configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FeaturesConfig {
    /// AI character selection
    pub ai_selection: bool,
    /// Context understanding
    pub context_understanding: bool,
    /// Auto-correction
    pub auto_correction: bool,
    /// Smart suggestions
    pub smart_suggestions: bool,
}

impl MaidosConfig {
    /// Load configuration file
    pub fn load(config_path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(config_path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Save configuration file
    pub fn save(&self, config_path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        std::fs::write(config_path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))
    }

    /// Get default configuration path
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("maidos")
            .join("maidos.toml")
    }
}

/// Configuration error type
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Configuration result type
pub type Result<T> = std::result::Result<T, ConfigError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MaidosConfig {
            model: ModelConfig {
                provider: "ollama".to_string(),
                model: "llama3.2:3b-q4_0".to_string(),
                max_tokens: 10,
            },
            ime: ImeConfig {
                default_scheme: "bopomofo".to_string(),
                charset: Charset::Traditional,
                enabled_schemes: vec!["bopomofo".to_string(), "pinyin".to_string()],
            },
            security: SecurityConfig {
                data_collection: false,
                privacy_first: true,
            },
            features: FeaturesConfig {
                ai_selection: true,
                context_understanding: true,
                auto_correction: true,
                smart_suggestions: true,
            },
        };

        assert_eq!(config.model.provider, "ollama");
        assert_eq!(config.ime.default_scheme, "bopomofo");
        assert_eq!(config.ime.charset, Charset::Traditional);
        assert_eq!(config.ime.enabled_schemes, vec!["bopomofo".to_string(), "pinyin".to_string()]);
    }
}
