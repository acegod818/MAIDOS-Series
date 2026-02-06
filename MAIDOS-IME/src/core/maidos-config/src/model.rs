//! Model configuration management
//!
//! This module provides model configuration management functionality.

use crate::{ConfigError, Result};
use std::path::Path;

/// Model configuration manager
pub struct ModelConfigManager;

/// Model information
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ModelInfo {
    /// Model name
    pub name: String,
    /// Model path
    pub path: String,
    /// Model size (in MB)
    pub size_mb: u32,
    /// Whether enabled
    pub enabled: bool,
    /// Whether this is the default model
    pub is_default: bool,
}

impl ModelConfigManager {
    /// Get default model configuration
    pub fn get_default_model() -> ModelInfo {
        ModelInfo {
            name: "llama3.2:3b-q4_0".to_string(),
            path: "models/llama3.2:3b-q4_0.gguf".to_string(),
            size_mb: 2048,
            enabled: true,
            is_default: true,
        }
    }

    /// Load model configuration from file
    pub fn load_model_config<P: AsRef<Path>>(path: P) -> Result<Vec<ModelInfo>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Save model configuration to file
    pub fn save_model_config<P: AsRef<Path>>(models: &[ModelInfo], path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(models)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        std::fs::write(path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))
    }

    /// Validate model configuration
    pub fn validate_model_config(models: &[ModelInfo]) -> Result<()> {
        // Check that at least one model is enabled
        if !models.iter().any(|m| m.enabled) {
            return Err(ConfigError::ParseError("At least one enabled model is required".to_string()));
        }

        // Check that exactly one default model exists
        let default_count = models.iter().filter(|m| m.is_default).count();
        if default_count != 1 {
            return Err(ConfigError::ParseError("Exactly one default model is required".to_string()));
        }

        // Check that model names are unique
        let mut names = std::collections::HashSet::new();
        for model in models {
            if !names.insert(&model.name) {
                return Err(ConfigError::ParseError(format!("Duplicate model name: {}", model.name)));
            }
        }

        Ok(())
    }

    /// Get default model from list
    pub fn get_default_model_from_list(models: &[ModelInfo]) -> Option<&ModelInfo> {
        models.iter().find(|m| m.is_default)
    }

    /// Get list of enabled models
    pub fn get_enabled_models(models: &[ModelInfo]) -> Vec<&ModelInfo> {
        models.iter().filter(|m| m.enabled).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_default_model() {
        let model = ModelConfigManager::get_default_model();
        assert_eq!(model.name, "llama3.2:3b-q4_0");
        assert_eq!(model.size_mb, 2048);
        assert!(model.enabled);
        assert!(model.is_default);
    }

    #[test]
    fn test_validate_model_config() {
        let models = vec![
            ModelInfo {
                name: "model1".to_string(),
                path: "path1".to_string(),
                size_mb: 100,
                enabled: true,
                is_default: true,
            },
            ModelInfo {
                name: "model2".to_string(),
                path: "path2".to_string(),
                size_mb: 200,
                enabled: false,
                is_default: false,
            },
        ];

        assert!(ModelConfigManager::validate_model_config(&models).is_ok());
    }

    #[test]
    fn test_validate_model_config_no_enabled() {
        let models = vec![
            ModelInfo {
                name: "model1".to_string(),
                path: "path1".to_string(),
                size_mb: 100,
                enabled: false,
                is_default: true,
            },
        ];

        assert!(ModelConfigManager::validate_model_config(&models).is_err());
    }

    #[test]
    fn test_validate_model_config_duplicate_names() {
        let models = vec![
            ModelInfo {
                name: "model1".to_string(),
                path: "path1".to_string(),
                size_mb: 100,
                enabled: true,
                is_default: true,
            },
            ModelInfo {
                name: "model1".to_string(), // Duplicate name
                path: "path2".to_string(),
                size_mb: 200,
                enabled: false,
                is_default: false,
            },
        ];

        assert!(ModelConfigManager::validate_model_config(&models).is_err());
    }

    #[test]
    fn test_save_and_load_model_config() {
        let models = vec![
            ModelInfo {
                name: "test_model".to_string(),
                path: "test_path".to_string(),
                size_mb: 500,
                enabled: true,
                is_default: true,
            },
        ];

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path();

        // Save model config
        assert!(ModelConfigManager::save_model_config(&models, path).is_ok());

        // Load model config
        let loaded_models = ModelConfigManager::load_model_config(path);
        assert!(loaded_models.is_ok());

        let loaded_models = loaded_models.expect("Failed to load models");
        assert_eq!(loaded_models.len(), 1);
        assert_eq!(loaded_models[0].name, "test_model");
        assert_eq!(loaded_models[0].size_mb, 500);
    }

    #[test]
    fn test_get_default_model_from_list() {
        let models = vec![
            ModelInfo {
                name: "model1".to_string(),
                path: "path1".to_string(),
                size_mb: 100,
                enabled: false,
                is_default: false,
            },
            ModelInfo {
                name: "model2".to_string(),
                path: "path2".to_string(),
                size_mb: 200,
                enabled: true,
                is_default: true,
            },
        ];

        let default_model = ModelConfigManager::get_default_model_from_list(&models);
        assert!(default_model.is_some());
        assert_eq!(default_model.expect("Failed to get default model").name, "model2");
    }

    #[test]
    fn test_get_enabled_models() {
        let models = vec![
            ModelInfo {
                name: "model1".to_string(),
                path: "path1".to_string(),
                size_mb: 100,
                enabled: true,
                is_default: false,
            },
            ModelInfo {
                name: "model2".to_string(),
                path: "path2".to_string(),
                size_mb: 200,
                enabled: false,
                is_default: true,
            },
            ModelInfo {
                name: "model3".to_string(),
                path: "path3".to_string(),
                size_mb: 300,
                enabled: true,
                is_default: false,
            },
        ];

        let enabled_models = ModelConfigManager::get_enabled_models(&models);
        assert_eq!(enabled_models.len(), 2);
        assert_eq!(enabled_models[0].name, "model1");
        assert_eq!(enabled_models[1].name, "model3");
    }
}