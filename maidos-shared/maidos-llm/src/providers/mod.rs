//! LLM Provider implementations
//!
//! <impl>
//! WHAT: Provider implementations for cloud and local LLM backends
//! WHY: Support multiple LLM backends with unified interface
//! HOW: Each provider implements LlmProvider trait, organized by category
//! TEST: Per-provider unit tests with mocked HTTP
//! </impl>
//!
//! # Provider Categories
//!
//! ## Cloud Providers (10)
//! - OpenAI (GPT-4, GPT-4o)
//! - Anthropic (Claude)
//! - Google (Gemini)
//! - DeepSeek (Chat, Coder, Reasoner)
//! - Groq (LPU-accelerated)
//! - Mistral (European, Vision via Pixtral)
//! - Azure OpenAI (Microsoft-hosted)
//! - Cohere (Command R, RAG)
//! - Together AI (Open-source models)
//! - Replicate (Async predictions)
//!
//! ## Local Providers (3)
//! - Ollama (local model runner)
//! - LM Studio (local inference)
//! - vLLM (high-throughput serving)

pub mod cloud;
pub mod local;

// Re-export cloud providers
pub use cloud::OpenAiProvider;
pub use cloud::AnthropicProvider;
pub use cloud::GoogleProvider;
pub use cloud::DeepSeekProvider;
pub use cloud::GroqProvider;
pub use cloud::MistralProvider;
pub use cloud::AzureOpenAiProvider;
pub use cloud::CohereProvider;
pub use cloud::TogetherProvider;
pub use cloud::ReplicateProvider;
pub use cloud::CloudProviderType;

// Re-export local providers
pub use local::OllamaProvider;
pub use local::LmStudioProvider;
pub use local::VllmProvider;
pub use local::LocalProviderType;

use crate::error::{LlmError, Result};
use crate::provider::LlmProvider;
use std::sync::Arc;

/// Provider category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderCategory {
    /// Cloud-based providers (require API key)
    Cloud,
    /// Local/self-hosted providers
    Local,
}

/// Unified provider type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    // Cloud providers
    OpenAi,
    Anthropic,
    Google,
    DeepSeek,
    Groq,
    Mistral,
    AzureOpenAi,
    Cohere,
    Together,
    Replicate,
    // Local providers
    Ollama,
    LmStudio,
    Vllm,
}

impl ProviderType {
    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            // Cloud
            "openai" | "open_ai" | "gpt" => Some(Self::OpenAi),
            "anthropic" | "claude" => Some(Self::Anthropic),
            "google" | "gemini" => Some(Self::Google),
            "deepseek" | "deep_seek" => Some(Self::DeepSeek),
            "groq" => Some(Self::Groq),
            "mistral" => Some(Self::Mistral),
            "azure_openai" | "azure" | "azureopenai" => Some(Self::AzureOpenAi),
            "cohere" | "command" => Some(Self::Cohere),
            "together" | "together_ai" | "togetherai" => Some(Self::Together),
            "replicate" => Some(Self::Replicate),
            // Local
            "ollama" | "local" => Some(Self::Ollama),
            "lmstudio" | "lm_studio" | "lm-studio" => Some(Self::LmStudio),
            "vllm" | "v-llm" => Some(Self::Vllm),
            _ => None,
        }
    }

    /// Get provider category
    pub fn category(&self) -> ProviderCategory {
        match self {
            Self::OpenAi | Self::Anthropic | Self::Google | Self::DeepSeek | 
            Self::Groq | Self::Mistral | Self::AzureOpenAi | Self::Cohere |
            Self::Together | Self::Replicate => {
                ProviderCategory::Cloud
            }
            Self::Ollama | Self::LmStudio | Self::Vllm => ProviderCategory::Local,
        }
    }

    /// Get provider name
    pub fn name(&self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Google => "google",
            Self::DeepSeek => "deepseek",
            Self::Groq => "groq",
            Self::Mistral => "mistral",
            Self::AzureOpenAi => "azure_openai",
            Self::Cohere => "cohere",
            Self::Together => "together",
            Self::Replicate => "replicate",
            Self::Ollama => "ollama",
            Self::LmStudio => "lmstudio",
            Self::Vllm => "vllm",
        }
    }

    /// Get default model for this provider
    pub fn default_model(&self) -> &'static str {
        match self {
            Self::OpenAi => "gpt-4o",
            Self::Anthropic => "claude-sonnet-4-20250514",
            Self::Google => "gemini-1.5-flash",
            Self::DeepSeek => "deepseek-chat",
            Self::Groq => "llama-3.3-70b-versatile",
            Self::Mistral => "mistral-large-latest",
            Self::AzureOpenAi => "gpt-4o",
            Self::Cohere => "command-r-plus",
            Self::Together => "meta-llama/Llama-3.3-70B-Instruct-Turbo",
            Self::Replicate => "meta/llama-2-70b-chat",
            Self::Ollama => "llama3.2",
            Self::LmStudio => "local-model",
            Self::Vllm => "served-model",
        }
    }

    /// Check if this provider requires an API key
    pub fn requires_api_key(&self) -> bool {
        self.category() == ProviderCategory::Cloud
    }

    /// All provider types
    pub fn all() -> &'static [Self] {
        &[
            Self::OpenAi,
            Self::Anthropic,
            Self::Google,
            Self::DeepSeek,
            Self::Groq,
            Self::Mistral,
            Self::AzureOpenAi,
            Self::Cohere,
            Self::Together,
            Self::Replicate,
            Self::Ollama,
            Self::LmStudio,
            Self::Vllm,
        ]
    }

    /// All cloud provider types
    pub fn cloud_providers() -> &'static [Self] {
        &[
            Self::OpenAi,
            Self::Anthropic,
            Self::Google,
            Self::DeepSeek,
            Self::Groq,
            Self::Mistral,
            Self::AzureOpenAi,
            Self::Cohere,
            Self::Together,
            Self::Replicate,
        ]
    }

    /// All local provider types
    pub fn local_providers() -> &'static [Self] {
        &[Self::Ollama, Self::LmStudio, Self::Vllm]
    }
}

/// Create a provider from type and configuration
pub fn create_provider(
    provider_type: ProviderType,
    api_key: Option<String>,
    base_url: Option<String>,
) -> Result<Arc<dyn LlmProvider>> {
    match provider_type {
        // Cloud providers
        ProviderType::OpenAi => {
            let key = api_key.ok_or_else(|| LlmError::Config("OpenAI requires API key".into()))?;
            Ok(Arc::new(OpenAiProvider::new(key, base_url)))
        }
        ProviderType::Anthropic => {
            let key = api_key.ok_or_else(|| LlmError::Config("Anthropic requires API key".into()))?;
            Ok(Arc::new(AnthropicProvider::new(key, base_url)))
        }
        ProviderType::Google => {
            let key = api_key.ok_or_else(|| LlmError::Config("Google requires API key".into()))?;
            Ok(Arc::new(GoogleProvider::new(key, base_url)))
        }
        ProviderType::DeepSeek => {
            let key = api_key.ok_or_else(|| LlmError::Config("DeepSeek requires API key".into()))?;
            Ok(Arc::new(DeepSeekProvider::new(key, base_url)))
        }
        ProviderType::Groq => {
            let key = api_key.ok_or_else(|| LlmError::Config("Groq requires API key".into()))?;
            Ok(Arc::new(GroqProvider::new(key, base_url)))
        }
        ProviderType::Mistral => {
            let key = api_key.ok_or_else(|| LlmError::Config("Mistral requires API key".into()))?;
            Ok(Arc::new(MistralProvider::new(key, base_url)))
        }
        ProviderType::AzureOpenAi => {
            let key = api_key.ok_or_else(|| LlmError::Config("Azure OpenAI requires API key".into()))?;
            // Azure requires endpoint and deployment_id, use base_url as endpoint
            let endpoint = base_url.ok_or_else(|| LlmError::Config("Azure OpenAI requires endpoint URL".into()))?;
            Ok(Arc::new(AzureOpenAiProvider::new(key, endpoint, "default", None)))
        }
        ProviderType::Cohere => {
            let key = api_key.ok_or_else(|| LlmError::Config("Cohere requires API key".into()))?;
            Ok(Arc::new(CohereProvider::new(key, base_url)))
        }
        ProviderType::Together => {
            let key = api_key.ok_or_else(|| LlmError::Config("Together AI requires API key".into()))?;
            Ok(Arc::new(TogetherProvider::new(key, base_url)))
        }
        ProviderType::Replicate => {
            let key = api_key.ok_or_else(|| LlmError::Config("Replicate requires API token".into()))?;
            Ok(Arc::new(ReplicateProvider::new(key, base_url)))
        }
        // Local providers
        ProviderType::Ollama => Ok(Arc::new(OllamaProvider::new(base_url))),
        ProviderType::LmStudio => Ok(Arc::new(LmStudioProvider::new(base_url))),
        ProviderType::Vllm => Ok(Arc::new(VllmProvider::new(base_url))),
    }
}

/// Create a cloud provider
pub fn create_cloud_provider(
    provider_type: CloudProviderType,
    api_key: String,
    base_url: Option<String>,
) -> Result<Arc<dyn LlmProvider>> {
    match provider_type {
        CloudProviderType::OpenAi => Ok(Arc::new(OpenAiProvider::new(api_key, base_url))),
        CloudProviderType::Anthropic => Ok(Arc::new(AnthropicProvider::new(api_key, base_url))),
        CloudProviderType::Google => Ok(Arc::new(GoogleProvider::new(api_key, base_url))),
        CloudProviderType::DeepSeek => Ok(Arc::new(DeepSeekProvider::new(api_key, base_url))),
        CloudProviderType::Groq => Ok(Arc::new(GroqProvider::new(api_key, base_url))),
        CloudProviderType::Mistral => Ok(Arc::new(MistralProvider::new(api_key, base_url))),
        CloudProviderType::AzureOpenAi => {
            let endpoint = base_url.ok_or_else(|| LlmError::Config("Azure requires endpoint".into()))?;
            Ok(Arc::new(AzureOpenAiProvider::new(api_key, endpoint, "default", None)))
        }
        CloudProviderType::Cohere => Ok(Arc::new(CohereProvider::new(api_key, base_url))),
        CloudProviderType::Together => Ok(Arc::new(TogetherProvider::new(api_key, base_url))),
        CloudProviderType::Replicate => Ok(Arc::new(ReplicateProvider::new(api_key, base_url))),
    }
}

/// Create a local provider
pub fn create_local_provider(
    provider_type: LocalProviderType,
    base_url: Option<String>,
) -> Arc<dyn LlmProvider> {
    match provider_type {
        LocalProviderType::Ollama => Arc::new(OllamaProvider::new(base_url)),
        LocalProviderType::LmStudio => Arc::new(LmStudioProvider::new(base_url)),
        LocalProviderType::Vllm => Arc::new(VllmProvider::new(base_url)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_from_str() {
        // Cloud
        assert_eq!(ProviderType::parse("openai"), Some(ProviderType::OpenAi));
        assert_eq!(ProviderType::parse("claude"), Some(ProviderType::Anthropic));
        assert_eq!(ProviderType::parse("gemini"), Some(ProviderType::Google));
        assert_eq!(ProviderType::parse("deepseek"), Some(ProviderType::DeepSeek));
        assert_eq!(ProviderType::parse("groq"), Some(ProviderType::Groq));
        assert_eq!(ProviderType::parse("mistral"), Some(ProviderType::Mistral));
        assert_eq!(ProviderType::parse("azure"), Some(ProviderType::AzureOpenAi));
        assert_eq!(ProviderType::parse("cohere"), Some(ProviderType::Cohere));
        assert_eq!(ProviderType::parse("together"), Some(ProviderType::Together));
        assert_eq!(ProviderType::parse("replicate"), Some(ProviderType::Replicate));
        // Local
        assert_eq!(ProviderType::parse("ollama"), Some(ProviderType::Ollama));
        assert_eq!(ProviderType::parse("lmstudio"), Some(ProviderType::LmStudio));
        assert_eq!(ProviderType::parse("vllm"), Some(ProviderType::Vllm));
        // Unknown
        assert_eq!(ProviderType::parse("unknown"), None);
    }

    #[test]
    fn test_provider_category() {
        assert_eq!(ProviderType::OpenAi.category(), ProviderCategory::Cloud);
        assert_eq!(ProviderType::Anthropic.category(), ProviderCategory::Cloud);
        assert_eq!(ProviderType::Google.category(), ProviderCategory::Cloud);
        assert_eq!(ProviderType::DeepSeek.category(), ProviderCategory::Cloud);
        assert_eq!(ProviderType::Groq.category(), ProviderCategory::Cloud);
        assert_eq!(ProviderType::Mistral.category(), ProviderCategory::Cloud);
        assert_eq!(ProviderType::AzureOpenAi.category(), ProviderCategory::Cloud);
        assert_eq!(ProviderType::Cohere.category(), ProviderCategory::Cloud);
        assert_eq!(ProviderType::Together.category(), ProviderCategory::Cloud);
        assert_eq!(ProviderType::Replicate.category(), ProviderCategory::Cloud);
        
        assert_eq!(ProviderType::Ollama.category(), ProviderCategory::Local);
        assert_eq!(ProviderType::LmStudio.category(), ProviderCategory::Local);
        assert_eq!(ProviderType::Vllm.category(), ProviderCategory::Local);
    }

    #[test]
    fn test_requires_api_key() {
        assert!(ProviderType::OpenAi.requires_api_key());
        assert!(ProviderType::Anthropic.requires_api_key());
        assert!(ProviderType::Google.requires_api_key());
        assert!(ProviderType::DeepSeek.requires_api_key());
        assert!(ProviderType::Groq.requires_api_key());
        assert!(ProviderType::Mistral.requires_api_key());
        assert!(ProviderType::AzureOpenAi.requires_api_key());
        assert!(ProviderType::Cohere.requires_api_key());
        assert!(ProviderType::Together.requires_api_key());
        assert!(ProviderType::Replicate.requires_api_key());
        
        assert!(!ProviderType::Ollama.requires_api_key());
        assert!(!ProviderType::LmStudio.requires_api_key());
        assert!(!ProviderType::Vllm.requires_api_key());
    }

    #[test]
    fn test_default_models() {
        assert!(ProviderType::OpenAi.default_model().contains("gpt"));
        assert!(ProviderType::Anthropic.default_model().contains("claude"));
        assert!(ProviderType::Google.default_model().contains("gemini"));
        assert!(ProviderType::DeepSeek.default_model().contains("deepseek"));
        assert!(ProviderType::Groq.default_model().contains("llama"));
        assert!(ProviderType::Mistral.default_model().contains("mistral"));
        assert!(ProviderType::AzureOpenAi.default_model().contains("gpt"));
        assert!(ProviderType::Cohere.default_model().contains("command"));
        assert!(ProviderType::Together.default_model().contains("Llama"));
        assert!(ProviderType::Replicate.default_model().contains("llama"));
        assert!(ProviderType::Ollama.default_model().contains("llama"));
    }

    #[test]
    fn test_create_provider_missing_key() {
        assert!(create_provider(ProviderType::OpenAi, None, None).is_err());
        assert!(create_provider(ProviderType::Anthropic, None, None).is_err());
        assert!(create_provider(ProviderType::Google, None, None).is_err());
        assert!(create_provider(ProviderType::DeepSeek, None, None).is_err());
        assert!(create_provider(ProviderType::Groq, None, None).is_err());
        assert!(create_provider(ProviderType::Mistral, None, None).is_err());
        assert!(create_provider(ProviderType::Cohere, None, None).is_err());
        assert!(create_provider(ProviderType::Together, None, None).is_err());
        assert!(create_provider(ProviderType::Replicate, None, None).is_err());
    }

    #[test]
    fn test_create_local_provider_no_key() {
        assert!(create_provider(ProviderType::Ollama, None, None).is_ok());
        assert!(create_provider(ProviderType::LmStudio, None, None).is_ok());
        assert!(create_provider(ProviderType::Vllm, None, None).is_ok());
    }

    #[test]
    fn test_all_providers() {
        assert_eq!(ProviderType::all().len(), 13);
        assert_eq!(ProviderType::cloud_providers().len(), 10);
        assert_eq!(ProviderType::local_providers().len(), 3);
    }

    #[test]
    fn test_provider_names() {
        assert_eq!(ProviderType::OpenAi.name(), "openai");
        assert_eq!(ProviderType::Google.name(), "google");
        assert_eq!(ProviderType::Mistral.name(), "mistral");
        assert_eq!(ProviderType::AzureOpenAi.name(), "azure_openai");
        assert_eq!(ProviderType::Cohere.name(), "cohere");
        assert_eq!(ProviderType::Together.name(), "together");
        assert_eq!(ProviderType::Replicate.name(), "replicate");
        assert_eq!(ProviderType::Ollama.name(), "ollama");
        assert_eq!(ProviderType::Vllm.name(), "vllm");
    }
}
