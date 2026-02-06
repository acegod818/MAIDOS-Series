//! Dictionary module
//!
//! This module provides dictionary loading and lookup functionality.

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

/// Pinyin parser
pub struct PinyinParser {
    dictionary: Dictionary,
}

impl PinyinParser {
    /// Create a new pinyin parser
    pub fn new(dictionary_path: &str) -> Result<Self> {
        let dictionary = Dictionary::load_from_file(dictionary_path)?;
        Ok(Self { dictionary })
    }

    /// Parse pinyin and get candidate words
    pub fn parse_pinyin(&self, pinyin: &str) -> Vec<DictEntry> {
        if let Some(entries) = self.dictionary.lookup(pinyin) {
            // Sort by frequency
            let mut sorted_entries = entries.clone();
            sorted_entries.sort_by(|a, b| b.frequency.cmp(&a.frequency));
            sorted_entries
        } else {
            Vec::new()
        }
    }

    /// Get dictionary reference
    pub fn get_dictionary(&self) -> &Dictionary {
        &self.dictionary
    }
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_dictionary_loading() {
        // Should have a test dictionary file here
        // Skipping for now since test file has not been created yet
    }
}