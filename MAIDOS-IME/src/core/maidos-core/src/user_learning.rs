//! Persistent user learning for input schemes
//!
//! Stores user selection history per scheme. When a user selects a candidate,
//! its frequency is boosted. The table is loaded once on first access and
//! saved immediately on every update (write-through cache).
//!
//! Storage: `%LOCALAPPDATA%\MAIDOS\IME\user_{scheme_name}.json`

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, Mutex};
use serde::{Deserialize, Serialize};

/// Per-character entry in the user learning table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEntry {
    pub char: String,
    pub freq: u32,
}

/// User learning table for one scheme
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserTable {
    pub entries: HashMap<String, Vec<UserEntry>>,
}

// ---------------------------------------------------------------------------
// Global in-memory cache (one UserTable per scheme name)
// ---------------------------------------------------------------------------

fn global_cache() -> &'static Mutex<HashMap<String, UserTable>> {
    static CACHE: OnceLock<Mutex<HashMap<String, UserTable>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Get the storage directory for user learning data
fn user_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("MAIDOS")
        .join("IME")
}

/// Get the file path for a scheme's user table
fn user_table_path(scheme_name: &str) -> PathBuf {
    user_data_dir().join(format!("user_{}.json", scheme_name))
}

impl UserTable {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Load from disk (returns empty table if file doesn't exist or is corrupt)
    pub fn load_from_disk(path: &PathBuf) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::new(),
        }
    }

    /// Save to disk (creates parent directories if needed)
    pub fn save_to_disk(&self, path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string(self)
            .map_err(std::io::Error::other)?;
        std::fs::write(path, content)
    }

    /// Record that the user selected `character` when they typed `input_code`.
    /// Each selection boosts the frequency by 100 (caps at 9999).
    pub fn learn(&mut self, input_code: &str, character: &str) {
        let entries = self.entries.entry(input_code.to_lowercase()).or_default();
        if let Some(entry) = entries.iter_mut().find(|e| e.char == character) {
            entry.freq = (entry.freq + 100).min(9999);
        } else {
            // New entry starts at 1100 (higher than most builtin entries)
            entries.push(UserEntry {
                char: character.to_string(),
                freq: 1100,
            });
        }
    }

    /// Get user-learned entries for a given input code
    pub fn lookup(&self, input_code: &str) -> Vec<&UserEntry> {
        self.entries.get(&input_code.to_lowercase())
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Get all entries (for merging into scheme lookup)
    pub fn all_entries(&self) -> &HashMap<String, Vec<UserEntry>> {
        &self.entries
    }
}

// ---------------------------------------------------------------------------
// Public API (thread-safe, globally accessible)
// ---------------------------------------------------------------------------

/// Get a clone of the user table for a scheme (loads from disk on first access)
pub fn get_user_table(scheme_name: &str) -> UserTable {
    let mut cache = global_cache().lock().unwrap_or_else(|e| e.into_inner());
    if !cache.contains_key(scheme_name) {
        let path = user_table_path(scheme_name);
        cache.insert(scheme_name.to_string(), UserTable::load_from_disk(&path));
    }
    cache.get(scheme_name).cloned().unwrap_or_default()
}

/// Lookup user-learned entries for a scheme + input code
pub fn lookup_user_entries(scheme_name: &str, input_code: &str) -> Vec<UserEntry> {
    let table = get_user_table(scheme_name);
    table.lookup(input_code).into_iter().cloned().collect()
}

/// Record a user selection and save to disk immediately
pub fn learn_and_save(scheme_name: &str, input_code: &str, character: &str) -> Result<(), String> {
    let mut cache = global_cache().lock().unwrap_or_else(|e| e.into_inner());
    let path = user_table_path(scheme_name);

    let table = cache.entry(scheme_name.to_string())
        .or_insert_with(|| UserTable::load_from_disk(&path));

    table.learn(input_code, character);
    table.save_to_disk(&path).map_err(|e| e.to_string())
}

/// Clear all learned data for a scheme (and delete the file)
pub fn clear_user_table(scheme_name: &str) -> Result<(), String> {
    let mut cache = global_cache().lock().unwrap_or_else(|e| e.into_inner());
    cache.insert(scheme_name.to_string(), UserTable::new());
    let path = user_table_path(scheme_name);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Merge user-learned entries into a candidate list.
/// User entries override builtin entries of the same character (higher freq wins).
/// Call this at the end of each scheme's lookup() method.
pub fn merge_user_learned(
    results: &mut Vec<crate::schemes::Candidate>,
    scheme_name: &str,
    input_code: &str,
) {
    let entries = lookup_user_entries(scheme_name, input_code);
    for ue in entries {
        if let Some(ch) = ue.char.chars().next() {
            // If the character already exists in results, boost its frequency
            if let Some(existing) = results.iter_mut().find(|c| c.character == ch) {
                if ue.freq > existing.frequency {
                    existing.frequency = ue.freq;
                }
            } else {
                results.push(crate::schemes::Candidate {
                    character: ch,
                    frequency: ue.freq,
                    pronunciation: input_code.to_string(),
                });
            }
        }
    }
    // Re-sort after merging
    results.sort_by(|a, b| b.frequency.cmp(&a.frequency));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_user_table_learn_and_lookup() {
        let mut table = UserTable::new();

        // First selection: "ni" → "泥"
        table.learn("ni", "泥");
        let results = table.lookup("ni");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].char, "泥");
        assert_eq!(results[0].freq, 1100);

        // Second selection: "ni" → "泥" again (boost)
        table.learn("ni", "泥");
        let results = table.lookup("ni");
        assert_eq!(results[0].freq, 1200);

        // Different character same input
        table.learn("ni", "你");
        let results = table.lookup("ni");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_user_table_persistence() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test_user.json");

        // Write
        let mut table = UserTable::new();
        table.learn("shi", "是");
        table.learn("shi", "是");
        table.save_to_disk(&path).unwrap();

        // Read back
        let loaded = UserTable::load_from_disk(&path);
        let results = loaded.lookup("shi");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].char, "是");
        assert_eq!(results[0].freq, 1200);
    }

    #[test]
    fn test_freq_cap() {
        let mut table = UserTable::new();
        for _ in 0..200 {
            table.learn("de", "的");
        }
        let results = table.lookup("de");
        assert_eq!(results[0].freq, 9999); // capped
    }

    #[test]
    fn test_case_insensitive() {
        let mut table = UserTable::new();
        table.learn("NI", "你");
        let results = table.lookup("ni");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].char, "你");
    }
}
