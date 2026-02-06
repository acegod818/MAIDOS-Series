//! MAIDOS Forge language adapter standard interface
//!
//! Defines the standard interface that all language adapters must implement

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use crate::error::Result;

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
    /// Preprocessor defines
    pub defines: HashMap<String, String>,
    /// Include paths
    pub include_paths: Vec<String>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            profile: "debug".to_string(),
            target_platform: "native".to_string(),
            verbose: false,
            extra_args: Vec::new(),
            defines: HashMap::new(),
            include_paths: Vec::new(),
        }
    }
}

/// Compile result
#[derive(Debug, Clone)]
pub struct CompileResult {
    /// Whether compilation succeeded
    pub success: bool,
    /// Output file paths
    pub outputs: Vec<std::path::PathBuf>,
    /// Compilation duration
    pub duration: Duration,
    /// Error message (if any)
    pub error: Option<String>,
    /// Log messages
    pub logs: Vec<String>,
    /// Warning messages
    pub warnings: Vec<String>,
}

impl CompileResult {
    /// Create a successful compile result.
    pub fn success(outputs: Vec<std::path::PathBuf>, logs: Vec<String>, duration: Duration) -> Self {
        Self {
            success: true,
            outputs,
            duration,
            error: None,
            logs,
            warnings: Vec::new(),
        }
    }

    /// Create a failed compile result.
    pub fn failure(error: String, logs: Vec<String>, duration: Duration) -> Self {
        Self {
            success: false,
            outputs: Vec::new(),
            duration,
            error: Some(error),
            logs,
            warnings: Vec::new(),
        }
    }
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
    /// Supported features
    pub features: Vec<String>,
}

/// Language adapter trait
#[async_trait]
pub trait LanguageAdapter: Send + Sync {
    /// Get the language ID.
    fn language_id(&self) -> &str;

    /// Get the language name.
    fn language_name(&self) -> &str;

    /// Get supported file extensions.
    fn extensions(&self) -> &[&str];

    /// Compile source code.
    async fn compile(
        &self,
        source_files: &[std::path::PathBuf],
        output_dir: &Path,
        options: &CompileOptions,
    ) -> Result<CompileResult>;

    /// Check whether the toolchain is available.
    async fn check_toolchain(&self) -> Result<bool>;

    /// Get toolchain information.
    async fn toolchain_info(&self) -> Result<ToolchainInfo>;

    /// Parse source code and generate an abstract syntax tree.
    async fn parse(&self, source_file: &Path) -> Result<crate::parser::ParseResult>;

    /// Check source code for errors and warnings.
    async fn check(&self, source_file: &Path) -> Result<crate::checker::CheckResult>;

    /// Extract interface definitions.
    async fn extract_interface(&self, artifact_path: &Path) -> Result<InterfaceDescription>;

    /// Generate glue code.
    async fn generate_glue(&self, interface: &InterfaceDescription, target_language: &str) -> Result<GlueCodeResult>;
}

/// Interface description
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InterfaceDescription {
    /// Version
    pub version: String,
    /// Module information
    pub module: InterfaceModule,
    /// Language information
    pub language: InterfaceLanguage,
    /// Exported functions
    pub exports: Vec<ExportedFunction>,
}

/// Interface module
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InterfaceModule {
    /// Name
    pub name: String,
    /// Version
    pub version: String,
}

/// Interface language
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InterfaceLanguage {
    /// Name
    pub name: String,
    /// ABI
    pub abi: String,
}

/// Exported function
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportedFunction {
    /// Name
    pub name: String,
    /// Return type
    pub return_type: String,
    /// Parameters
    pub parameters: Vec<FunctionParameter>,
}

/// Function parameter
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionParameter {
    /// Name
    pub name: String,
    /// Type
    pub type_name: String,
}

/// Glue code result
#[derive(Debug, Clone)]
pub struct GlueCodeResult {
    /// Whether generation succeeded
    pub success: bool,
    /// Generated code
    pub code: Option<String>,
    /// Filename
    pub filename: Option<String>,
    /// Target language
    pub target_language: Option<String>,
    /// Error message
    pub error: Option<String>,
}

impl GlueCodeResult {
    /// Create a successful glue code result.
    pub fn success(code: String, filename: String, target_language: String) -> Self {
        Self {
            success: true,
            code: Some(code),
            filename: Some(filename),
            target_language: Some(target_language),
            error: None,
        }
    }

    /// Create a failed glue code result.
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            code: None,
            filename: None,
            target_language: None,
            error: Some(error),
        }
    }
}