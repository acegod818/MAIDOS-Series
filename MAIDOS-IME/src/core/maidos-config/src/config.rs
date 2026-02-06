//! Configuration management implementation
//!
//! This module provides config file loading, parsing, and saving functionality.

use crate::{MaidosConfig, ConfigError, Result};
use std::path::Path;

/// Configuration manager
pub struct ConfigManager;

impl ConfigManager {
    /// Load configuration file
    pub fn load_config<P: AsRef<Path>>(path: P) -> Result<MaidosConfig> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Save configuration file
    pub fn save_config<P: AsRef<Path>>(config: &MaidosConfig, path: P) -> Result<()> {
        let content = toml::to_string_pretty(config)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        std::fs::write(path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))
    }

    /// Create default configuration
    pub fn create_default_config() -> MaidosConfig {
        MaidosConfig {
            model: crate::ModelConfig {
                provider: "ollama".to_string(),
                model: "llama3.2:3b-q4_0".to_string(),
                max_tokens: 10,
            },
            ime: crate::ImeConfig {
                default_scheme: "bopomofo".to_string(),
                charset: crate::Charset::Traditional, // Default to Traditional Chinese
                enabled_schemes: vec!["bopomofo".to_string(), "pinyin".to_string()], // Default: enable Bopomofo and Pinyin
            },
            security: crate::SecurityConfig {
                data_collection: false,
                privacy_first: true,
            },
            features: crate::FeaturesConfig {
                ai_selection: true,
                context_understanding: true,
                auto_correction: true,
                smart_suggestions: true,
            },
        }
    }

    /// Validate configuration
    pub fn validate_config(config: &MaidosConfig) -> Result<()> {
        // Validate model provider
        if config.model.provider.is_empty() {
            return Err(ConfigError::ParseError("Model provider must not be empty".to_string()));
        }

        // Validate model name
        if config.model.model.is_empty() {
            return Err(ConfigError::ParseError("Model name must not be empty".to_string()));
        }

        // Validate input scheme
        if config.ime.default_scheme.is_empty() {
            return Err(ConfigError::ParseError("Default input scheme must not be empty".to_string()));
        }

        // Validate max tokens
        if config.model.max_tokens == 0 {
            return Err(ConfigError::ParseError("Max tokens must be greater than 0".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_create_default_config() {
        let config = ConfigManager::create_default_config();
        assert_eq!(config.model.provider, "ollama");
        assert_eq!(config.ime.default_scheme, "bopomofo");
        assert!(!config.security.data_collection);
        assert!(config.features.ai_selection);
    }

    #[test]
    fn test_validate_config() {
        let config = ConfigManager::create_default_config();
        assert!(ConfigManager::validate_config(&config).is_ok());
    }

    #[test]
    fn test_save_and_load_config() {
        let config = ConfigManager::create_default_config();
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path();

        // Save config
        assert!(ConfigManager::save_config(&config, path).is_ok());

        // Load config
        let loaded_config = ConfigManager::load_config(path);
        assert!(loaded_config.is_ok());

        let loaded_config = loaded_config.expect("Failed to load config");
        assert_eq!(config.model.provider, loaded_config.model.provider);
        assert_eq!(config.ime.default_scheme, loaded_config.ime.default_scheme);
    }
}