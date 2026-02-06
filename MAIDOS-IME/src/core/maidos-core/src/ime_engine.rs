//! IME engine implementation
//!
//! This module provides the core functionality of the IME engine, including:
//! - Input processing
//! - Candidate selection
//! - AI character selection
//! - Context understanding

use crate::Result;
use maidos_llm::{client::LlmClient, ProviderType, LlmRequest, Message};
use maidos_config::MaidosConfig;
use crate::converter::CharsetConverter;
use maidos_config::Charset;

/// IME engine
pub struct ImeEngine {
    /// LLM client
    llm_client: LlmClient,
    /// Configuration
    pub config: MaidosConfig,
}

impl ImeEngine {
    /// Create a new IME engine
    pub fn new(config: MaidosConfig) -> Result<Self> {
        let model_config: maidos_llm::models::ModelConfig = config.model.clone().into();
        let llm_client = LlmClient::from_config(&model_config)?;
        Ok(Self { llm_client, config })
    }

    /// Create an IME engine from a config file
    pub fn from_config_file(config_path: &str) -> Result<Self> {
        let config = MaidosConfig::load(config_path)?;
        Self::new(config)
    }

    /// Select a character
    pub async fn select_character(&self, context: &str, candidates: &[char]) -> Result<char> {
        // If AI selection is disabled, use a simple selection strategy
        if !self.config.features.ai_selection {
            return Ok(*candidates.first().unwrap_or(&'\0'));
        }

        // Build prompt
        let prompt = format!(
            "Select the most appropriate character based on the context.\n\nContext: {}\nCandidates: {:?}\n\nPlease reply with only the selected character, no other content.",
            context, candidates
        );

        // Build request
        let request = LlmRequest {
            provider: ProviderType::Ollama,
            model: self.config.model.model.clone(),
            system: Some("You are an intelligent IME assistant that helps users select the most appropriate character.".to_string()),
            messages: vec![Message::user(prompt)],
            max_tokens: Some(self.config.model.max_tokens),
            temperature: Some(0.3), // Use lower temperature for more consistent results
        };

        // Send request
        let response = self.llm_client.complete(&request).await?;

        // Parse response
        self.parse_character_selection(&response.content, candidates)
    }

    /// Parse character selection response
    fn parse_character_selection(&self, response: &str, candidates: &[char]) -> Result<char> {
        let trimmed = response.trim();

        // If response is empty, return the first candidate
        if trimmed.is_empty() {
            return Ok(*candidates.first().unwrap_or(&'\0'));
        }

        // Find the first matching character in the response
        for ch in trimmed.chars() {
            if candidates.contains(&ch) {
                return Ok(ch);
            }
        }

        // If no matching character found, return the first candidate
        Ok(*candidates.first().unwrap_or(&'\0'))
    }

    /// Process input
    pub async fn process_input(&self, input: &str, _context: &str) -> Result<String> {
        // Based on configured input scheme, use SchemeFactory to get the corresponding Scheme and process input
        let scheme_name = &self.config.ime.default_scheme;
        let scheme_type = match scheme_name.as_str() {
            "bopomofo" => crate::schemes::SchemeType::Bopomofo,
            "pinyin" => crate::schemes::SchemeType::Pinyin,
            "cangjie" => crate::schemes::SchemeType::Cangjie,
            "quick" => crate::schemes::SchemeType::Quick,
            "wubi" => crate::schemes::SchemeType::Wubi,
            "english" => crate::schemes::SchemeType::English,
            "japanese" => crate::schemes::SchemeType::Japanese,
            _ => crate::schemes::SchemeType::Bopomofo,
        };

        let scheme = crate::schemes::SchemeFactory::create_scheme_simple(&scheme_type);
        let candidates = scheme.process_input(input)?;

        if candidates.is_empty() {
            return Ok(input.to_string());
        }

        // Return the highest-frequency candidate
        Ok(candidates[0].character.to_string())
    }

    /// Process cross-input (supports combinations of different input schemes and charsets)
    pub async fn process_cross_input(&self, input: &str, _context: &str, _scheme: &str, charset: &Charset) -> Result<String> {
        // First process the input
        let processed_input = self.process_input(input, _context).await?;

        // Convert based on configured charset
        let converted_input = CharsetConverter::convert(
            &processed_input,
            &self.config.ime.charset,
            charset
        );

        Ok(converted_input)
    }

    /// Get cross-input candidates (supports combinations of different input schemes and charsets)
    pub async fn get_cross_candidates(&self, input: &str, scheme: &str, charset: &Charset) -> Result<Vec<char>> {
        // Get candidates based on input scheme
        let scheme_type = match scheme {
            "bopomofo" => crate::schemes::SchemeType::Bopomofo,
            "pinyin" => crate::schemes::SchemeType::Pinyin,
            "cangjie" => crate::schemes::SchemeType::Cangjie,
            "quick" => crate::schemes::SchemeType::Quick,
            "wubi" => crate::schemes::SchemeType::Wubi,
            "english" => crate::schemes::SchemeType::English,
            "japanese" => crate::schemes::SchemeType::Japanese,
            _ => crate::schemes::SchemeType::Bopomofo,
        };

        let scheme_impl = crate::schemes::SchemeFactory::create_scheme_simple(&scheme_type);
        let candidates = scheme_impl.get_candidates(input)?;

        // Extract characters
        let chars: Vec<char> = candidates.iter().map(|c| c.character).collect();

        // Convert candidates based on charset
        let converted = CharsetConverter::convert_candidates(
            &chars,
            &self.config.ime.charset,
            charset,
        );

        Ok(converted)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ime_engine_creation() {
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

        let _engine = ImeEngine::new(config);
        // Note: This test will fail without an actual Ollama service running
        // But in a real environment, this should work correctly
        // assert!(engine.is_ok());
    }

    #[test]
    fn test_parse_character_selection() {
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

        // Create a mock IME engine (without LLM client)
        let engine = ImeEngine {
            llm_client: maidos_llm::client::LlmClient::new("http://localhost:11434"),
            config,
        };

        let candidates = vec!['\u{4F60}', '\u{597D}', '\u{4E16}', '\u{754C}'];

        // Test correct character selection
        let result = engine.parse_character_selection("\u{4F60}", &candidates);
        assert!(result.is_ok());
        assert_eq!(result.expect("Failed to get result"), '\u{4F60}');

        // Test character not in candidate list
        let result = engine.parse_character_selection("a", &candidates);
        assert!(result.is_ok());
        assert_eq!(result.expect("Failed to get result"), '\u{4F60}'); // Should return the first candidate

        // Test empty response
        let result = engine.parse_character_selection("", &candidates);
        assert!(result.is_ok());
        assert_eq!(result.expect("Failed to get result"), '\u{4F60}'); // Should return the first candidate
    }
}
