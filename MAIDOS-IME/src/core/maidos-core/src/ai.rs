//! AI feature implementation
//!
//! This module provides AI-related features, including:
//! - Context understanding
//! - Long sentence input
//! - Auto-correction
//! - Smart suggestions

use crate::Result;
use maidos_llm::{client::LlmClient, ProviderType, LlmRequest, Message};
use maidos_config::MaidosConfig;

/// AI feature manager
pub struct AiManager {
    /// LLM client
    llm_client: LlmClient,
    /// Configuration
    config: MaidosConfig,
}

impl AiManager {
    /// Create a new AI manager
    pub fn new(config: MaidosConfig) -> Result<Self> {
        let model_config: maidos_llm::models::ModelConfig = config.model.clone().into();
        let llm_client = LlmClient::from_config(&model_config)?;
        Ok(Self { llm_client, config })
    }

    /// Create an AI manager from a config file
    pub fn from_config_file(config_path: &str) -> Result<Self> {
        let config = MaidosConfig::load(config_path)?;
        Self::new(config)
    }

    /// Context-based character selection
    pub async fn context_based_selection(&self, context: &str, candidates: &[String]) -> Result<String> {
        // If context understanding is disabled, return the first candidate
        if !self.config.features.context_understanding {
            return Ok(candidates.first().cloned().unwrap_or_else(|| "".to_string()));
        }

        // Build prompt
        let prompt = format!(
            "Select the most appropriate option based on the context.\n\nContext: {}\nCandidates: {:?}\n\nPlease reply with only the selected option index (starting from 0), no other content.",
            context, candidates
        );

        // Build request
        let request = LlmRequest {
            provider: ProviderType::Ollama,
            model: self.config.model.model.clone(),
            system: Some("You are an intelligent assistant that helps users select the most appropriate option based on context.".to_string()),
            messages: vec![Message::user(prompt)],
            max_tokens: Some(self.config.model.max_tokens),
            temperature: Some(0.3), // Use lower temperature for more consistent results
        };

        // Send request
        let response = self.llm_client.complete(&request).await?;

        // Parse response
        self.parse_index_selection(&response.content, candidates.len())
            .map(|index| candidates[index].clone())
    }

    /// Parse index selection response
    fn parse_index_selection(&self, response: &str, max_index: usize) -> Result<usize> {
        let trimmed = response.trim();

        // If response is empty, return the first option
        if trimmed.is_empty() {
            return Ok(0);
        }

        // Try to parse as a number
        if let Ok(index) = trimmed.parse::<usize>() {
            if index < max_index {
                return Ok(index);
            }
        }

        // If unable to parse as a valid index, return the first option
        Ok(0)
    }

    /// Long sentence input processing
    pub async fn long_sentence_input(&self, sentence: &str) -> Result<Vec<String>> {
        // Build prompt
        let prompt = format!(
            "Break the following sentence into appropriate phrases, one per line:\n\n{}",
            sentence
        );

        // Build request
        let request = LlmRequest {
            provider: ProviderType::Ollama,
            model: self.config.model.model.clone(),
            system: Some("You are a language expert who specializes in breaking long sentences into appropriate phrases.".to_string()),
            messages: vec![Message::user(prompt)],
            max_tokens: Some(self.config.model.max_tokens * 20), // Long sentences may need more tokens
            temperature: Some(0.5), // Use moderate temperature for balanced results
        };

        // Send request
        let response = self.llm_client.complete(&request).await?;

        // Parse response into a list of phrases
        let phrases: Vec<String> = response.content
            .lines()
            .map(|line: &str| line.trim().to_string())
            .filter(|line: &String| !line.is_empty())
            .collect();

        Ok(phrases)
    }

    /// Auto-correction
    pub async fn auto_correct(&self, text: &str) -> Result<String> {
        // If auto-correction is disabled, return the original text
        if !self.config.features.auto_correction {
            return Ok(text.to_string());
        }

        // Build prompt
        let prompt = format!(
            "Correct the spelling and grammar errors in the following text:\n\n{}",
            text
        );

        // Build request
        let request = LlmRequest {
            provider: ProviderType::Ollama,
            model: self.config.model.model.clone(),
            system: Some("You are a text proofreading expert who specializes in correcting spelling and grammar errors.".to_string()),
            messages: vec![Message::user(prompt)],
            max_tokens: Some(self.config.model.max_tokens * 10), // Correction may need more tokens
            temperature: Some(0.2), // Use lower temperature for more accurate results
        };

        // Send request
        let response = self.llm_client.complete(&request).await?;

        // Return corrected text
        Ok(response.content.trim().to_string())
    }

    /// Smart suggestions
    pub async fn smart_suggestions(&self, text: &str) -> Result<Vec<String>> {
        // If smart suggestions is disabled, return empty list
        if !self.config.features.smart_suggestions {
            return Ok(vec![]);
        }

        // Build prompt
        let prompt = format!(
            "Provide several possible continuation suggestions based on the following text:\n\n{}",
            text
        );

        // Build request
        let request = LlmRequest {
            provider: ProviderType::Ollama,
            model: self.config.model.model.clone(),
            system: Some("You are a text prediction expert who specializes in providing text continuation suggestions.".to_string()),
            messages: vec![Message::user(prompt)],
            max_tokens: Some(self.config.model.max_tokens * 5), // Suggestions may need more tokens
            temperature: Some(0.7), // Use higher temperature for more diverse suggestions
        };

        // Send request
        let response = self.llm_client.complete(&request).await?;

        // Parse response into a list of suggestions
        let suggestions: Vec<String> = response.content
            .lines()
            .map(|line: &str| line.trim().to_string())
            .filter(|line: &String| !line.is_empty())
            .take(5) // Return at most 5 suggestions
            .collect();

        Ok(suggestions)
    }

    /// Language detection
    pub async fn detect_language(&self, text: &str) -> Result<String> {
        // Build prompt
        let prompt = format!(
            "Detect the language of the following text:\n\n{}",
            text
        );

        // Build request
        let request = LlmRequest {
            provider: ProviderType::Ollama,
            model: self.config.model.model.clone(),
            system: Some("You are a language detection expert who can identify multiple languages.".to_string()),
            messages: vec![Message::user(prompt)],
            max_tokens: Some(self.config.model.max_tokens),
            temperature: Some(0.1), // Use very low temperature for consistent results
        };

        // Send request
        let response = self.llm_client.complete(&request).await?;

        // Return detected language
        Ok(response.content.trim().to_string())
    }

    /// Text summarization
    pub async fn summarize_text(&self, text: &str) -> Result<String> {
        // Build prompt
        let prompt = format!(
            "Provide a brief summary for the following text:\n\n{}",
            text
        );

        // Build request
        let request = LlmRequest {
            provider: ProviderType::Ollama,
            model: self.config.model.model.clone(),
            system: Some("You are a summarization expert who provides concise and clear text summaries.".to_string()),
            messages: vec![Message::user(prompt)],
            max_tokens: Some(self.config.model.max_tokens * 3), // Summaries may need more tokens
            temperature: Some(0.4), // Use moderate temperature for balanced results
        };

        // Send request
        let response = self.llm_client.complete(&request).await?;

        // Return summary
        Ok(response.content.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_manager_creation() {
        // Create a simple config for testing
        let config = MaidosConfig {
            model: maidos_config::ModelConfig {
                provider: "ollama".to_string(),
                model: "llama3.2:3b-q4_0".to_string(),
                max_tokens: 10,
            },
            ime: maidos_config::ImeConfig {
                default_scheme: "bopomofo".to_string(),
                charset: maidos_config::Charset::Traditional,
                enabled_schemes: vec!["bopomofo".to_string(), "pinyin".to_string()],
            },
            security: maidos_config::SecurityConfig {
                data_collection: false,
                privacy_first: true,
            },
            features: maidos_config::FeaturesConfig {
                ai_selection: true,
                context_understanding: true,
                auto_correction: true,
                smart_suggestions: true,
            },
        };

        let _manager = AiManager::new(config);
        // Note: This test will fail without an actual Ollama service running
        // But in a real environment, this should work correctly
        // assert!(manager.is_ok());
    }

    #[test]
    fn test_parse_index_selection() {
        let config = MaidosConfig {
            model: maidos_config::ModelConfig {
                provider: "ollama".to_string(),
                model: "llama3.2:3b-q4_0".to_string(),
                max_tokens: 10,
            },
            ime: maidos_config::ImeConfig {
                default_scheme: "bopomofo".to_string(),
                charset: maidos_config::Charset::Traditional,
                enabled_schemes: vec!["bopomofo".to_string(), "pinyin".to_string()],
            },
            security: maidos_config::SecurityConfig {
                data_collection: false,
                privacy_first: true,
            },
            features: maidos_config::FeaturesConfig {
                ai_selection: true,
                context_understanding: true,
                auto_correction: true,
                smart_suggestions: true,
            },
        };

        // Create a mock AI manager (without LLM client)
        let manager = AiManager {
            llm_client: maidos_llm::client::LlmClient::new("http://localhost:11434"),
            config,
        };

        let candidates = ["Option1".to_string(), "Option2".to_string(), "Option3".to_string()];

        // Test correct index selection
        let result = manager.parse_index_selection("1", candidates.len());
        assert!(result.is_ok());
        assert_eq!(result.expect("Failed to get result"), 1);

        // Test invalid index
        let result = manager.parse_index_selection("5", candidates.len());
        assert!(result.is_ok());
        assert_eq!(result.expect("Failed to get result"), 0); // Should return the default first option

        // Test non-numeric response
        let result = manager.parse_index_selection("abc", candidates.len());
        assert!(result.is_ok());
        assert_eq!(result.expect("Failed to get result"), 0); // Should return the default first option

        // Test empty response
        let result = manager.parse_index_selection("", candidates.len());
        assert!(result.is_ok());
        assert_eq!(result.expect("Failed to get result"), 0); // Should return the default first option
    }
}
