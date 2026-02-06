//! Pinyin parser
//!
//! This module provides full pinyin-to-Chinese-character conversion.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use crate::Result;

/// Dictionary entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictEntry {
    /// Word
    pub word: String,
    /// Frequency
    pub frequency: u32,
    /// Pronunciation
    pub pronunciation: String,
    /// Tags
    pub tags: Vec<String>,
}

/// Dictionary data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dictionary {
    /// Entry map (pronunciation -> entry list)
    pub entries: HashMap<String, Vec<DictEntry>>,
    /// Version
    version: String,
    /// Creation time
    created_at: String,
    /// Update time
    updated_at: String,
}

impl Dictionary {
    /// Load dictionary from file
    pub fn load_from_file(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let dict: Dictionary = serde_json::from_reader(reader)?;
        Ok(dict)
    }

    /// Look up entries
    pub fn lookup(&self, pronunciation: &str) -> Option<&Vec<DictEntry>> {
        self.entries.get(pronunciation)
    }

    /// Get all entries
    pub fn get_all_entries(&self) -> &HashMap<String, Vec<DictEntry>> {
        &self.entries
    }

    /// Add an entry
    pub fn add_entry(&mut self, pronunciation: String, entry: DictEntry) {
        self.entries.entry(pronunciation).or_default().push(entry);
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Pinyin parse result
#[derive(Debug, Clone)]
pub struct PinyinParseResult {
    /// Candidate phrases
    pub candidates: Vec<String>,
    /// Frequency for each candidate
    pub frequencies: Vec<u32>,
}

/// Pinyin parser
pub struct PinyinParser {
    dictionary: Dictionary,
    /// Cache for intermediate results
    cache: HashMap<String, PinyinParseResult>,
}

impl PinyinParser {
    /// Create a new pinyin parser
    pub fn new(dictionary_path: &str) -> Result<Self> {
        let dictionary = Dictionary::load_from_file(dictionary_path)?;
        Ok(Self {
            dictionary,
            cache: HashMap::new(),
        })
    }

    /// Parse a single pinyin syllable
    pub fn parse_single_pinyin(&self, pinyin: &str) -> Vec<DictEntry> {
        if let Some(entries) = self.dictionary.lookup(pinyin) {
            // Sort by frequency
            let mut sorted_entries = entries.clone();
            sorted_entries.sort_by(|a, b| b.frequency.cmp(&a.frequency));
            sorted_entries
        } else {
            Vec::new()
        }
    }

    /// Parse continuous pinyin (e.g. "nihao")
    pub fn parse_continuous_pinyin(&mut self, pinyin_sequence: &str) -> PinyinParseResult {
        // Check cache
        if let Some(cached_result) = self.cache.get(pinyin_sequence) {
            return cached_result.clone();
        }

        // Decompose pinyin sequence into possible combinations
        let candidates = self.generate_candidates(pinyin_sequence);

        // Create result and cache it
        let result = PinyinParseResult {
            candidates: candidates.iter().map(|c| c.word.clone()).collect(),
            frequencies: candidates.iter().map(|c| c.frequency).collect(),
        };

        self.cache.insert(pinyin_sequence.to_string(), result.clone());
        result
    }

    /// Generate candidates (simplified implementation)
    fn generate_candidates(&self, pinyin_sequence: &str) -> Vec<DictEntry> {
        // This is a simplified implementation; real-world usage would be more complex.
        // We look up the full pinyin sequence and possible partial matches.

        let mut candidates = Vec::new();

        // 1. Look up exact match
        if let Some(entries) = self.dictionary.lookup(pinyin_sequence) {
            candidates.extend(entries.iter().cloned());
        }

        // 2. If no exact match, try splitting the pinyin sequence
        if candidates.is_empty() {
            // Simple splitting strategy: try different split points
            for i in 1..pinyin_sequence.len() {
                let left = &pinyin_sequence[..i];
                let right = &pinyin_sequence[i..];

                if let (Some(left_entries), Some(right_entries)) =
                    (self.dictionary.lookup(left), self.dictionary.lookup(right)) {
                    // Combine candidates from left and right parts
                    for left_entry in left_entries {
                        for right_entry in right_entries {
                            let combined_word = format!("{}{}", left_entry.word, right_entry.word);
                            let combined_freq = left_entry.frequency.min(right_entry.frequency);

                            candidates.push(DictEntry {
                                word: combined_word,
                                frequency: combined_freq,
                                pronunciation: format!("{} {}", left_entry.pronunciation, right_entry.pronunciation),
                                tags: vec!["combined".to_string()],
                            });
                        }
                    }
                }
            }
        }

        // 3. Sort by frequency and deduplicate
        candidates.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        candidates.dedup_by(|a, b| a.word == b.word);

        // Return at most 20 candidates
        candidates.truncate(20);

        candidates
    }

    /// Get dictionary reference
    pub fn get_dictionary(&self) -> &Dictionary {
        &self.dictionary
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_dictionary_loading() {
        // Should have a test dictionary file here
        // Skipping for now since test file has not been created yet
    }

    #[test]
    fn test_pinyin_parser_creation() {
        // Test parser creation (requires a valid dictionary file path)
        // let parser = PinyinParser::new("../../dicts/pinyin.dict.json");
        // assert!(parser.is_ok());
    }
}