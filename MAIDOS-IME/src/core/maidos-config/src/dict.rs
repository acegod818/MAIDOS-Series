//! User dictionary management
//!
//! This module provides management for user-defined dictionaries.

use crate::{ConfigError, Result};
use std::collections::HashMap;
use std::path::Path;
#[allow(unused_imports)]
use chrono::{Utc};

/// User dictionary entry
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DictEntry {
    /// Word
    pub word: String,
    /// Frequency
    pub frequency: u32,
    /// Pronunciation (pinyin/bopomofo)
    pub pronunciation: String,
    /// Tags
    pub tags: Vec<String>,
}

/// User dictionary
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UserDict {
    /// Dictionary entries
    pub entries: HashMap<String, DictEntry>,
    /// Dictionary version
    pub version: String,
    /// Creation time
    pub created_at: String,
    /// Last updated time
    pub updated_at: String,
}

impl UserDict {
    /// Create a new empty dictionary
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Add a dictionary entry
    pub fn add_entry(&mut self, entry: DictEntry) -> Result<()> {
        self.entries.insert(entry.word.clone(), entry);
        self.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    /// Remove a dictionary entry
    pub fn remove_entry(&mut self, word: &str) -> Result<()> {
        if self.entries.remove(word).is_some() {
            self.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(ConfigError::IoError("Word does not exist".to_string()))
        }
    }

    /// Find a dictionary entry
    pub fn find_entry(&self, word: &str) -> Option<&DictEntry> {
        self.entries.get(word)
    }

    /// Load dictionary from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Save dictionary to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        std::fs::write(path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))
    }

    /// Get dictionary size
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Check if dictionary is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for UserDict {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_user_dict_creation() {
        let dict = UserDict::new();
        assert!(dict.is_empty());
        assert_eq!(dict.version, "1.0.0");
    }

    #[test]
    fn test_add_and_find_entry() {
        let mut dict = UserDict::new();
        let entry = DictEntry {
            word: "你好".to_string(),
            frequency: 100,
            pronunciation: "ni3 hao3".to_string(),
            tags: vec!["greeting".to_string()],
        };

        assert!(dict.add_entry(entry).is_ok());
        assert_eq!(dict.size(), 1);

        let found = dict.find_entry("你好");
        assert!(found.is_some());
        assert_eq!(found.expect("Failed to find entry").frequency, 100);
    }

    #[test]
    fn test_remove_entry() {
        let mut dict = UserDict::new();
        let entry = DictEntry {
            word: "世界".to_string(),
            frequency: 50,
            pronunciation: "shi4 jie4".to_string(),
            tags: vec!["noun".to_string()],
        };

        dict.add_entry(entry).expect("Failed to add entry");
        assert_eq!(dict.size(), 1);

        assert!(dict.remove_entry("世界").is_ok());
        assert_eq!(dict.size(), 0);
        assert!(dict.is_empty());
    }

    #[test]
    fn test_save_and_load_dict() {
        let mut dict = UserDict::new();
        let entry = DictEntry {
            word: "測試".to_string(),
            frequency: 10,
            pronunciation: "ce4 shi4".to_string(),
            tags: vec!["test".to_string()],
        };

        dict.add_entry(entry).expect("Failed to add entry");

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path();

        // Save dictionary
        assert!(dict.save_to_file(path).is_ok());

        // Load dictionary
        let loaded_dict = UserDict::load_from_file(path);
        assert!(loaded_dict.is_ok());

        let loaded_dict = loaded_dict.expect("Failed to load dict");
        assert_eq!(loaded_dict.size(), 1);
        assert!(loaded_dict.find_entry("測試").is_some());
    }
}