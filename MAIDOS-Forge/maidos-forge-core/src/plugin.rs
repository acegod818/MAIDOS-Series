//! MAIDOS Forge language plugin system
//!
//! Provides a plugin architecture for cross-language compilation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::process::Command;

use crate::config::module::ModuleConfig;
use crate::error::{ForgeError, Result};

/// Language plugin trait
#[async_trait::async_trait]
pub trait LanguagePlugin: Send + Sync + Debug {
    /// Get the language ID.
    fn language_id(&self) -> &str;

    /// Get the language name.
    fn language_name(&self) -> &str;

    /// Get supported file extensions.
    fn extensions(&self) -> &[&str];

    /// Compile a module.
    async fn compile(
        &self,
        module: &ModuleConfig,
        output_dir: &std::path::Path,
        options: &CompileOptions,
    ) -> Result<CompileResult>;

    /// Check whether the toolchain is available.
    async fn check_toolchain(&self) -> Result<bool>;

    /// Get toolchain information.
    async fn toolchain_info(&self) -> Result<ToolchainInfo>;
}

/// Compile options
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// Build profile (debug/release)
    pub profile: String,
    /// Target platform
    pub target_platform: String,
    /// Whether to enable verbose output
    pub verbose: bool,
    /// Extra arguments
    pub extra_args: Vec<String>,
}

/// Compile result
#[derive(Debug, Clone)]
pub struct CompileResult {
    /// Whether compilation succeeded
    pub success: bool,
    /// Output file paths
    pub outputs: Vec<std::path::PathBuf>,
    /// Compilation duration
    pub duration: std::time::Duration,
    /// Error message (if any)
    pub error: Option<String>,
    /// Log messages
    pub logs: Vec<String>,
}

/// Toolchain information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolchainInfo {
    /// Toolchain name
    pub name: String,
    /// Version
    pub version: String,
    /// Supported target platforms
    pub targets: Vec<String>,
}

/// Language definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LanguageDefinition {
    /// Language ID
    pub id: String,
    /// Language name
    pub name: String,
    /// Display name
    pub display_name: String,
    /// Language category
    pub category: LanguageCategory,
    /// Supported file extensions
    pub extensions: Vec<String>,
    /// Supported toolchains
    pub toolchains: Vec<String>,
    /// Output file types
    pub output_types: Vec<String>,
    /// Description
    pub description: String,
    /// Whether this is a built-in language
    pub is_builtin: bool,
}

/// Language category
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum LanguageCategory {
    /// Systems programming language
    System,
    /// Managed language
    Managed,
    /// Scripting language
    Scripting,
    /// Web/frontend language
    Web,
    /// Functional language
    Functional,
    /// Mobile language
    Mobile,
    /// Concurrent language
    Concurrent,
    /// Scientific computing language
    Scientific,
    /// Hardware description language
    Hardware,
    /// Smart contract language
    Blockchain,
    /// Formal verification language
    Verification,
    /// Modern language
    Modern,
    /// Configuration language
    Configuration,
    /// Logic/query language
    Logic,
    /// Other language
    Other,
}

/// Language plugin manager
#[derive(Debug)]
pub struct PluginManager {
    /// Registered plugins
    plugins: HashMap<String, Arc<dyn LanguagePlugin>>,
    /// Language definitions
    language_definitions: HashMap<String, LanguageDefinition>,
}

impl PluginManager {
    /// Create a new plugin manager.
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            language_definitions: HashMap::new(),
        }
    }

    /// Register a plugin.
    pub fn register_plugin<P: LanguagePlugin + 'static>(&mut self, plugin: P) {
        let id = plugin.language_id().to_string();
        self.plugins.insert(id, Arc::new(plugin));
    }

    /// Get a plugin by language ID.
    pub fn get_plugin(&self, language_id: &str) -> Option<Arc<dyn LanguagePlugin>> {
        self.plugins.get(language_id).cloned()
    }

    /// Get all registered language IDs.
    pub fn registered_languages(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Add a language definition.
    pub fn add_language_definition(&mut self, definition: LanguageDefinition) {
        self.language_definitions
            .insert(definition.id.clone(), definition);
    }

    /// Get a language definition by ID.
    pub fn get_language_definition(&self, language_id: &str) -> Option<&LanguageDefinition> {
        self.language_definitions.get(language_id)
    }

    /// Get all language definitions.
    pub fn get_all_language_definitions(&self) -> Vec<&LanguageDefinition> {
        self.language_definitions.values().collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic language plugin implementation
#[derive(Debug)]
pub struct GenericLanguagePlugin {
    /// Language definition
    definition: LanguageDefinition,
    /// Toolchain command
    toolchain_cmd: String,
}

impl GenericLanguagePlugin {
    /// Create a new generic language plugin.
    pub fn new(definition: LanguageDefinition, toolchain_cmd: String) -> Self {
        Self {
            definition,
            toolchain_cmd,
        }
    }
}

#[async_trait::async_trait]
impl LanguagePlugin for GenericLanguagePlugin {
    fn language_id(&self) -> &str {
        &self.definition.id
    }

    fn language_name(&self) -> &str {
        &self.definition.name
    }

    fn extensions(&self) -> &[&str] {
        // We need to return a static lifetime reference here.
        // For simplicity, we temporarily return an empty array.
        // In the actual implementation, this should be obtained from the definition.
        &[]
    }

    async fn compile(
        &self,
        module: &ModuleConfig,
        output_dir: &std::path::Path,
        options: &CompileOptions,
    ) -> Result<CompileResult> {
        // This is a simplified implementation; the actual one varies by language
        let start_time = std::time::Instant::now();

        // Build the compile command
        let mut cmd = Command::new(&self.toolchain_cmd);
        cmd.arg("build")
            .arg("--output")
            .arg(output_dir)
            .arg("--source")
            .arg(&module.src);

        // Add extra arguments
        for arg in &options.extra_args {
            cmd.arg(arg);
        }

        // Execute the compile command
        let output = cmd.output().await.map_err(|e| {
            ForgeError::Compilation(format!(
                "Failed to execute compile command '{}': {}",
                self.toolchain_cmd, e
            ))
        })?;

        let duration = start_time.elapsed();

        // Check compilation result
        if output.status.success() {
            // Simplified handling; in practice, output files should be parsed
            let outputs = vec![output_dir.join(format!("{}.out", module.name))];
            
            Ok(CompileResult {
                success: true,
                outputs,
                duration,
                error: None,
                logs: vec![String::from_utf8_lossy(&output.stdout).to_string()],
            })
        } else {
            Ok(CompileResult {
                success: false,
                outputs: vec![],
                duration,
                error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
                logs: vec![String::from_utf8_lossy(&output.stdout).to_string()],
            })
        }
    }

    async fn check_toolchain(&self) -> Result<bool> {
        let output = Command::new(&self.toolchain_cmd)
            .arg("--version")
            .output()
            .await;

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    async fn toolchain_info(&self) -> Result<ToolchainInfo> {
        let output = Command::new(&self.toolchain_cmd)
            .arg("--version")
            .output()
            .await
            .map_err(|e| {
                ForgeError::Toolchain(format!(
                    "Failed to get toolchain info for '{}': {}",
                    self.toolchain_cmd, e
                ))
            })?;

        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stdout);
            let version_lines: Vec<&str> = version_output.lines().collect();
            let version = if !version_lines.is_empty() {
                version_lines[0].to_string()
            } else {
                "unknown".to_string()
            };

            Ok(ToolchainInfo {
                name: self.toolchain_cmd.clone(),
                version,
                targets: vec!["native".to_string()], // Simplified handling
            })
        } else {
            Err(ForgeError::Toolchain(
                "Unable to retrieve toolchain information".to_string(),
            ))
        }
    }
}

// Add async-trait macro to lib.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager() {
        let manager = PluginManager::new();
        assert_eq!(manager.registered_languages().len(), 0);
    }

    #[tokio::test]
    async fn test_generic_plugin_creation() {
        let definition = LanguageDefinition {
            id: "test".to_string(),
            name: "Test Language".to_string(),
            display_name: "Test Language".to_string(),
            category: LanguageCategory::Other,
            extensions: vec![".test".to_string()],
            toolchains: vec!["testc".to_string()],
            output_types: vec![".out".to_string()],
            description: "Test language for unit testing".to_string(),
            is_builtin: true,
        };

        let plugin = GenericLanguagePlugin::new(definition, "testc".to_string());
        assert_eq!(plugin.language_id(), "test");
    }
}