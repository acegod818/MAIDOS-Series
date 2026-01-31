//! Local LLM Providers
//!
//! <impl>
//! WHAT: Local/self-hosted LLM provider implementations
//! WHY: Unified interface for local model inference servers
//! HOW: Each provider implements LlmProvider trait with HTTP clients
//! TEST: Per-provider unit tests with mocked HTTP
//! </impl>

pub mod ollama;
pub mod lmstudio;
pub mod vllm;

pub use ollama::OllamaProvider;
pub use lmstudio::LmStudioProvider;
pub use vllm::VllmProvider;

/// Local provider category marker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalProviderType {
    Ollama,
    LmStudio,
    Vllm,
}

impl LocalProviderType {
    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ollama" => Some(Self::Ollama),
            "lmstudio" | "lm_studio" | "lm-studio" => Some(Self::LmStudio),
            "vllm" | "v-llm" => Some(Self::Vllm),
            _ => None,
        }
    }

    /// Get provider name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::LmStudio => "lmstudio",
            Self::Vllm => "vllm",
        }
    }

    /// Get default base URL
    pub fn default_url(&self) -> &'static str {
        match self {
            Self::Ollama => "http://localhost:11434",
            Self::LmStudio => "http://localhost:1234/v1",
            Self::Vllm => "http://localhost:8000/v1",
        }
    }

    /// All local provider types
    pub fn all() -> &'static [Self] {
        &[Self::Ollama, Self::LmStudio, Self::Vllm]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_provider_from_str() {
        assert_eq!(LocalProviderType::parse("ollama"), Some(LocalProviderType::Ollama));
        assert_eq!(LocalProviderType::parse("lmstudio"), Some(LocalProviderType::LmStudio));
        assert_eq!(LocalProviderType::parse("lm-studio"), Some(LocalProviderType::LmStudio));
        assert_eq!(LocalProviderType::parse("vllm"), Some(LocalProviderType::Vllm));
        assert_eq!(LocalProviderType::parse("unknown"), None);
    }

    #[test]
    fn test_all_local_providers() {
        assert_eq!(LocalProviderType::all().len(), 3);
    }

    #[test]
    fn test_default_urls() {
        assert!(LocalProviderType::Ollama.default_url().contains("11434"));
        assert!(LocalProviderType::LmStudio.default_url().contains("1234"));
        assert!(LocalProviderType::Vllm.default_url().contains("8000"));
    }
}
