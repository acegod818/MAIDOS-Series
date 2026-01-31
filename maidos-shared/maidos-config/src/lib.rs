//! MAIDOS Unified Configuration System
//!
//! Provides TOML-based configuration with:
//! - Environment variable expansion (`${VAR}` syntax)
//! - Schema validation
//! - Hot-reload support
//! - C FFI for cross-language bindings
//!
//! # Example
//!
//! ```ignore
//! use maidos_config::MaidosConfig;
//! use std::path::Path;
//!
//! // Load configuration
//! let config = MaidosConfig::load(Path::new("maidos.toml"))?;
//!
//! // Access typed values
//! println!("Default provider: {}", config.llm().default_provider);
//! println!("Daily budget: ${}", config.llm().budget_daily);
//!
//! // Watch for changes
//! let handle = config.watch(|new_config| {
//!     println!("Config reloaded!");
//! })?;
//!
//! // Stop watching when done
//! handle.stop();
//! ```
//!
//! # Configuration File Format
//!
//! ```toml
//! [maidos]
//! version = "1.0"
//!
//! [llm]
//! default_provider = "anthropic"
//! budget_daily = 10.0
//! budget_monthly = 100.0
//!
//! [llm.providers.anthropic]
//! api_key = "${ANTHROPIC_API_KEY}"
//! model = "claude-sonnet-4-20250514"
//!
//! [bus]
//! endpoint = "tcp://127.0.0.1:5555"
//!
//! [auth]
//! token_ttl = 3600
//! ```

mod error;
mod ffi;
mod loader;
mod schema;
#[cfg(feature = "watcher")]
mod watcher;

pub use error::{ConfigError, Result};
pub use schema::{
    AuthSection, BusSection, LlmSection, MaidosConfigSchema, MaidosSection, ProviderConfig,
};
#[cfg(feature = "watcher")]
pub use watcher::{WatchConfig, WatchHandle};

use std::path::Path;
use std::sync::{Arc, RwLock};
use tracing::info;

/// Main configuration interface
///
/// Thread-safe wrapper around configuration schema with reload support.
#[derive(Clone)]
pub struct MaidosConfig {
    inner: Arc<RwLock<MaidosConfigSchema>>,
    path: Option<std::path::PathBuf>,
}

impl MaidosConfig {
    /// Load configuration from a TOML file
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// * `Ok(MaidosConfig)` - Loaded configuration
    /// * `Err(ConfigError)` - If loading fails
    pub fn load(path: &Path) -> Result<Self> {
        let schema = loader::load_config(path)?;
        info!("Configuration loaded from {}", path.display());
        Ok(Self {
            inner: Arc::new(RwLock::new(schema)),
            path: Some(path.to_path_buf()),
        })
    }

    /// Create configuration from a TOML string
    ///
    /// Useful for testing or embedded configurations.
    /// 
    /// Consider using `str::parse()` instead for a more idiomatic API.
    pub fn from_toml(toml: &str) -> Result<Self> {
        let schema = loader::load_config_str(toml)?;
        Ok(Self {
            inner: Arc::new(RwLock::new(schema)),
            path: None,
        })
    }

    /// Create configuration with defaults
    pub fn default_config() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MaidosConfigSchema::default())),
            path: None,
        }
    }
}

impl std::str::FromStr for MaidosConfig {
    type Err = ConfigError;

    fn from_str(toml: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_toml(toml)
    }
}

impl MaidosConfig {
    /// Reload configuration from the original file
    ///
    /// # Returns
    /// * `Ok(())` - Configuration reloaded successfully
    /// * `Err(ConfigError)` - If reload fails (original config preserved)
    pub fn reload(&self) -> Result<()> {
        let path = self.path.as_ref().ok_or_else(|| {
            ConfigError::ValidationError("No file path for reload".to_string())
        })?;

        let new_schema = loader::load_config(path)?;

        let mut guard = self.inner.write().map_err(|_| {
            ConfigError::ValidationError("Lock poisoned".to_string())
        })?;
        *guard = new_schema;

        info!("Configuration reloaded from {}", path.display());
        Ok(())
    }

    /// Watch configuration file for changes and reload automatically
    ///
    /// # Arguments
    /// * `callback` - Function called after successful reload
    ///
    /// # Returns
    /// * `Ok(WatchHandle)` - Handle to stop watching
    /// * `Err(ConfigError)` - If watcher setup fails
    #[cfg(feature = "watcher")]
    pub fn watch<F>(&self, callback: F) -> Result<WatchHandle>
    where
        F: Fn(&MaidosConfig) + Send + 'static,
    {
        let path = self.path.as_ref().ok_or_else(|| {
            ConfigError::ValidationError("No file path for watching".to_string())
        })?;

        let config_clone = self.clone();
        watcher::watch_file(
            path,
            move |_| {
                if config_clone.reload().is_ok() {
                    callback(&config_clone);
                }
            },
            None,
        )
    }

    /// Watch with custom configuration
    #[cfg(feature = "watcher")]
    pub fn watch_with_config<F>(
        &self,
        callback: F,
        watch_config: WatchConfig,
    ) -> Result<WatchHandle>
    where
        F: Fn(&MaidosConfig) + Send + 'static,
    {
        let path = self.path.as_ref().ok_or_else(|| {
            ConfigError::ValidationError("No file path for watching".to_string())
        })?;

        let config_clone = self.clone();
        watcher::watch_file(
            path,
            move |_| {
                if config_clone.reload().is_ok() {
                    callback(&config_clone);
                }
            },
            Some(watch_config),
        )
    }

    // ========== Accessors ==========

    /// Acquire read lock on config.
    /// 
    /// # Panics
    /// Panics if the lock is poisoned (a thread panicked while holding the lock).
    #[inline]
    fn read_lock(&self) -> std::sync::RwLockReadGuard<'_, MaidosConfigSchema> {
        self.inner.read().expect("config lock poisoned")
    }

    /// Get MAIDOS metadata section
    pub fn maidos(&self) -> MaidosSection {
        self.read_lock().maidos.clone()
    }

    /// Get LLM configuration section
    pub fn llm(&self) -> LlmSection {
        self.read_lock().llm.clone()
    }

    /// Get event bus configuration section
    pub fn bus(&self) -> BusSection {
        self.read_lock().bus.clone()
    }

    /// Get authentication configuration section
    pub fn auth(&self) -> AuthSection {
        self.read_lock().auth.clone()
    }

    /// Get full schema (cloned)
    pub fn schema(&self) -> MaidosConfigSchema {
        self.read_lock().clone()
    }

    /// Get a specific provider configuration
    pub fn provider(&self, name: &str) -> Option<ProviderConfig> {
        self.read_lock().llm.providers.get(name).cloned()
    }

    /// Get the default provider configuration
    pub fn default_provider(&self) -> Option<ProviderConfig> {
        let guard = self.read_lock();
        guard.llm.providers.get(&guard.llm.default_provider).cloned()
    }
}

impl std::fmt::Debug for MaidosConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaidosConfig")
            .field("path", &self.path)
            .field("schema", &*self.read_lock())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_from_str() {
        let toml = r#"
[maidos]
version = "1.0"

[llm]
default_provider = "test"

[bus]
endpoint = "tcp://127.0.0.1:5555"

[auth]
token_ttl = 1800
"#;
        let config = MaidosConfig::from_str(toml).unwrap();
        assert_eq!(config.llm().default_provider, "test");
        assert_eq!(config.auth().token_ttl, 1800);
    }

    #[test]
    fn test_default_config() {
        let config = MaidosConfig::default_config();
        assert_eq!(config.maidos().version, "1.0");
        assert_eq!(config.llm().default_provider, "ollama");
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let config = MaidosConfig::default_config();
        let mut handles = vec![];

        for _ in 0..10 {
            let c = config.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    let _ = c.llm();
                    let _ = c.bus();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
    }
}
