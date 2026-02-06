//! English input scheme
//!
//! Prefix auto-completion + frequency sorting

use std::collections::HashMap;
use crate::Result;
use crate::schemes::{Candidate, InputScheme, SchemeType};

/// English word entry
#[derive(Debug, Clone, serde::Deserialize)]
struct EnglishWord {
    word: String,
    freq: u32,
}

/// English dictionary
#[derive(Debug, Clone, serde::Deserialize)]
struct EnglishDict {
    words: Vec<EnglishWord>,
}

/// English input scheme
pub struct EnglishScheme {
    words: Vec<EnglishWord>,
    user_words: HashMap<String, u32>,
}

impl EnglishScheme {
    pub fn new() -> Self {
        let words = Self::load_dict();
        Self {
            words,
            user_words: HashMap::new(),
        }
    }

    fn load_dict() -> Vec<EnglishWord> {
        let data = include_str!("../../data/english_common.json");
        match serde_json::from_str::<EnglishDict>(data) {
            Ok(dict) => dict.words,
            Err(_) => Self::build_minimal_dict(),
        }
    }

    fn build_minimal_dict() -> Vec<EnglishWord> {
        let words = vec![
            ("the", 10000), ("be", 9900), ("to", 9800), ("of", 9700),
            ("and", 9600), ("a", 9500), ("in", 9400), ("that", 9300),
            ("have", 9200), ("i", 9100), ("it", 9000), ("for", 8900),
            ("not", 8800), ("on", 8700), ("with", 8600), ("he", 8500),
            ("as", 8400), ("you", 8300), ("do", 8200), ("at", 8100),
            ("this", 8000), ("but", 7900), ("his", 7800), ("by", 7700),
            ("from", 7600), ("they", 7500), ("we", 7400), ("her", 7300),
            ("she", 7200), ("or", 7100), ("an", 7000), ("will", 6900),
            ("my", 6800), ("one", 6700), ("all", 6600), ("would", 6500),
            ("there", 6400), ("their", 6300), ("what", 6200), ("so", 6100),
            ("up", 6000), ("out", 5900), ("if", 5800), ("about", 5700),
            ("who", 5600), ("get", 5500), ("which", 5400), ("go", 5300),
            ("me", 5200), ("when", 5100), ("make", 5000), ("can", 4900),
            ("like", 4800), ("time", 4700), ("no", 4600), ("just", 4500),
            ("him", 4400), ("know", 4300), ("take", 4200), ("people", 4100),
            ("into", 4000), ("year", 3900), ("your", 3800), ("good", 3700),
            ("some", 3600), ("could", 3500), ("them", 3400), ("see", 3300),
            ("other", 3200), ("than", 3100), ("then", 3000), ("now", 2900),
            ("look", 2800), ("only", 2700), ("come", 2600), ("its", 2500),
            ("over", 2400), ("think", 2300), ("also", 2200), ("back", 2100),
            ("after", 2000), ("use", 1900), ("two", 1800), ("how", 1700),
            ("our", 1600), ("work", 1500), ("first", 1400), ("well", 1300),
            ("way", 1200), ("even", 1100), ("new", 1000), ("want", 900),
            ("because", 800), ("any", 700), ("these", 600), ("give", 500),
            ("day", 400), ("most", 300), ("us", 200),
        ];
        words.into_iter()
            .map(|(w, f)| EnglishWord { word: w.to_string(), freq: f })
            .collect()
    }

    fn lookup(&self, prefix: &str) -> Vec<Candidate> {
        let lower = prefix.to_lowercase();
        let mut results = Vec::new();

        // Prefix match
        for w in &self.words {
            if w.word.starts_with(&lower) {
                if let Some(ch) = w.word.chars().next() {
                    results.push(Candidate {
                        character: ch,
                        frequency: w.freq,
                        pronunciation: w.word.clone(),
                    });
                }
            }
        }

        // User words
        for (word, &freq) in &self.user_words {
            if word.starts_with(&lower) {
                if let Some(ch) = word.chars().next() {
                    results.push(Candidate {
                        character: ch,
                        frequency: freq,
                        pronunciation: word.clone(),
                    });
                }
            }
        }

        results.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        results.truncate(20);
        results
    }
}

impl Default for EnglishScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl InputScheme for EnglishScheme {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::English
    }

    fn process_input(&self, input: &str) -> Result<Vec<Candidate>> {
        Ok(self.lookup(input))
    }

    fn get_candidates(&self, input: &str) -> Result<Vec<Candidate>> {
        Ok(self.lookup(input))
    }

    fn add_user_word(&mut self, word: &str, frequency: u32) -> Result<()> {
        self.user_words.insert(word.to_lowercase(), frequency);
        Ok(())
    }

    fn remove_user_word(&mut self, word: &str) -> Result<()> {
        self.user_words.remove(&word.to_lowercase());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_prefix_match() {
        let scheme = EnglishScheme::new();
        let candidates = scheme.process_input("th").unwrap();
        assert!(!candidates.is_empty());
        // "the" should be the top result
        assert_eq!(candidates[0].pronunciation, "the");
    }

    #[test]
    fn test_english_no_match() {
        let scheme = EnglishScheme::new();
        let candidates = scheme.process_input("zzzzxxx").unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_english_user_word() {
        let mut scheme = EnglishScheme::new();
        scheme.add_user_word("testword", 5000).unwrap();
        let candidates = scheme.process_input("testw").unwrap();
        assert!(!candidates.is_empty());
        scheme.remove_user_word("testword").unwrap();
    }

    #[test]
    fn test_english_case_insensitive() {
        let scheme = EnglishScheme::new();
        let candidates = scheme.process_input("TH").unwrap();
        assert!(!candidates.is_empty());
    }
}
