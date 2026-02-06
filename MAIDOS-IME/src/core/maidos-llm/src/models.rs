//! LLM model data structures

use serde::{Deserialize, Serialize};

/// Model configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelConfig {
    /// Provider
    pub provider: String,
    /// Model name
    pub model: String,
    /// Maximum token count
    pub max_tokens: u32,
}

impl From<maidos_config::ModelConfig> for ModelConfig {
    fn from(config: maidos_config::ModelConfig) -> Self {
        ModelConfig {
            provider: config.provider,
            model: config.model,
            max_tokens: config.max_tokens,
        }
    }
}

/// Model metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelMetadata {
    /// Model ID
    pub id: String,
    /// Model name
    pub name: String,
    /// Model description
    pub description: String,
    /// Model size (in MB)
    pub size_mb: u32,
    /// Whether local execution is supported
    pub local_supported: bool,
    /// Recommended use cases
    pub recommended_use: Vec<String>,
}

/// Model list response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelListResponse {
    /// Model list
    pub models: Vec<ModelMetadata>,
}

/// Model download request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelDownloadRequest {
    /// Model ID
    pub model_id: String,
    /// Target path
    pub target_path: Option<String>,
}

/// Model download response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelDownloadResponse {
    /// Whether successful
    pub success: bool,
    /// Message
    pub message: String,
    /// Download path
    pub download_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_config() {
        let config = ModelConfig {
            provider: "ollama".to_string(),
            model: "llama3.2:3b-q4_0".to_string(),
            max_tokens: 10,
        };

        assert_eq!(config.provider, "ollama");
        assert_eq!(config.model, "llama3.2:3b-q4_0");
        assert_eq!(config.max_tokens, 10);
    }

    #[test]
    fn test_model_metadata() {
        let metadata = ModelMetadata {
            id: "llama3.2:3b-q4_0".to_string(),
            name: "Llama 3.2 3B Q4".to_string(),
            description: "Quantized version of the Llama 3.2 3B model".to_string(),
            size_mb: 2048,
            local_supported: true,
            recommended_use: vec!["text generation".to_string(), "conversation".to_string()],
        };

        assert_eq!(metadata.id, "llama3.2:3b-q4_0");
        assert_eq!(metadata.name, "Llama 3.2 3B Q4");
        assert_eq!(metadata.size_mb, 2048);
        assert!(metadata.local_supported);
        assert_eq!(metadata.recommended_use.len(), 2);
    }
}