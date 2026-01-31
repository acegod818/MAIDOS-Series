//! Cloud LLM Providers
//!
//! <impl>
//! WHAT: Cloud-based LLM provider implementations
//! WHY: Unified interface for commercial cloud LLM APIs
//! HOW: Each provider implements LlmProvider trait with HTTP clients
//! TEST: Per-provider unit tests with mocked HTTP
//! </impl>

pub mod openai;
pub mod anthropic;
pub mod google;
pub mod deepseek;
pub mod groq;
pub mod mistral;
pub mod azure_openai;
pub mod cohere;
pub mod together;
pub mod replicate;

pub use openai::OpenAiProvider;
pub use anthropic::AnthropicProvider;
pub use google::GoogleProvider;
pub use deepseek::DeepSeekProvider;
pub use groq::GroqProvider;
pub use mistral::MistralProvider;
pub use azure_openai::AzureOpenAiProvider;
pub use cohere::CohereProvider;
pub use together::TogetherProvider;
pub use replicate::ReplicateProvider;

/// Cloud provider category marker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudProviderType {
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
}

impl CloudProviderType {
    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
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
            _ => None,
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
        }
    }

    /// Get default model
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
        }
    }

    /// All cloud provider types
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
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_provider_from_str() {
        assert_eq!(CloudProviderType::parse("openai"), Some(CloudProviderType::OpenAi));
        assert_eq!(CloudProviderType::parse("claude"), Some(CloudProviderType::Anthropic));
        assert_eq!(CloudProviderType::parse("gemini"), Some(CloudProviderType::Google));
        assert_eq!(CloudProviderType::parse("deepseek"), Some(CloudProviderType::DeepSeek));
        assert_eq!(CloudProviderType::parse("groq"), Some(CloudProviderType::Groq));
        assert_eq!(CloudProviderType::parse("mistral"), Some(CloudProviderType::Mistral));
        assert_eq!(CloudProviderType::parse("azure"), Some(CloudProviderType::AzureOpenAi));
        assert_eq!(CloudProviderType::parse("cohere"), Some(CloudProviderType::Cohere));
        assert_eq!(CloudProviderType::parse("together"), Some(CloudProviderType::Together));
        assert_eq!(CloudProviderType::parse("replicate"), Some(CloudProviderType::Replicate));
        assert_eq!(CloudProviderType::parse("unknown"), None);
    }

    #[test]
    fn test_all_cloud_providers() {
        assert_eq!(CloudProviderType::all().len(), 10);
    }
}
