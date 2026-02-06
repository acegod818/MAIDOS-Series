//! High-performance compiler interface module
//!
//! Compliant with MAIDOS Forge specification v2.1
//! Supports 87 languages x 12 platform cross-compilation

use std::time::Instant;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Compiler error type
#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Compilation failed: {message}")]
    CompilationFailed { message: String },

    #[error("Toolchain not found: {toolchain}")]
    ToolchainNotFound { toolchain: String },

    #[error("Input validation failed: {message}")]
    ValidationError { message: String },

    #[error("Internal error: {source}")]
    InternalError { source: Box<dyn std::error::Error + Send + Sync> }
}

/// Compilation configuration
#[derive(Debug, Clone)]
pub struct CompileConfig {
    /// Target platform
    pub target: String,

    /// Compilation mode (debug/release)
    pub mode: CompileMode,

    /// Whether to enable incremental compilation
    pub incremental: bool,

    /// Custom compilation flags
    pub custom_flags: Vec<String>,

    /// Output directory
    pub output_dir: String,
}

/// Compilation mode
#[derive(Debug, Clone)]
pub enum CompileMode {
    Debug,
    Release,
    Custom(String),
}

impl CompileMode {
    pub fn as_str(&self) -> &str {
        match self {
            CompileMode::Debug => "debug",
            CompileMode::Release => "release",
            CompileMode::Custom(s) => s.as_str(),
        }
    }
}

/// Compilation result
#[derive(Debug, Serialize, Deserialize)]
pub struct CompileResult {
    /// Whether compilation succeeded
    pub success: bool,

    /// Error message (if any)
    pub error: Option<String>,

    /// Generated artifact file paths
    pub artifacts: Vec<String>,

    /// Compilation duration (milliseconds)
    pub duration_ms: u128,

    /// Compilation logs
    pub logs: Vec<String>,

    /// Warning messages
    pub warnings: Vec<String>,
}

impl CompileResult {
    /// Create a successful compilation result
    pub fn success(artifacts: Vec<String>, duration_ms: u128, logs: Vec<String>, warnings: Vec<String>) -> Self {
        Self {
            success: true,
            error: None,
            artifacts,
            duration_ms,
            logs,
            warnings,
        }
    }
    
    /// Create a failed compilation result
    pub fn failure(error: String, duration_ms: u128, logs: Vec<String>) -> Self {
        Self {
            success: false,
            error: Some(error),
            artifacts: vec![],
            duration_ms,
            logs,
            warnings: vec![],
        }
    }
}

/// Language adapter interface
#[async_trait::async_trait]
pub trait LanguageAdapter: Send + Sync {
    /// Get the language name
    fn language_name(&self) -> &'static str;

    /// Get supported file extensions
    fn supported_extensions(&self) -> &[&'static str];

    /// Validate whether the toolchain is available
    async fn validate_toolchain(&self) -> Result<bool, CompilerError>;

    /// Compile source code
    async fn compile(&self, source_files: Vec<String>, config: CompileConfig) -> Result<CompileResult, CompilerError>;

    /// Extract interface description
    async fn extract_interface(&self, artifact_path: &str) -> Result<Option<String>, CompilerError>;

    /// Generate glue code
    async fn generate_glue(&self, interface: &str, target_language: &str) -> Result<String, CompilerError>;
}

/// Compiler core
pub struct CompilerCore {
    adapters: std::collections::HashMap<String, Box<dyn LanguageAdapter>>,
}

impl CompilerCore {
    /// Create a new compiler instance
    pub fn new() -> Self {
        Self {
            adapters: std::collections::HashMap::new(),
        }
    }
    
    /// Register a language adapter
    pub fn register_adapter(&mut self, adapter: Box<dyn LanguageAdapter>) {
        let language = adapter.language_name().to_string();
        self.adapters.insert(language, adapter);
    }
    
    /// Get a language adapter
    pub fn get_adapter(&self, language: &str) -> Option<&dyn LanguageAdapter> {
        self.adapters.get(language).map(|boxed| boxed.as_ref())
    }
    
    /// Compile source code for the specified language
    #[tracing::instrument(skip(self, source_files))]
    pub async fn compile_source(
        &self, 
        language: &str, 
        source_files: Vec<String>, 
        config: CompileConfig
    ) -> Result<CompileResult, CompilerError> {
        // [MAIDOS-AUDIT] Begin compilation request
        tracing::info!("[MAIDOS-AUDIT] Starting compilation of {} source files", language);

        let start_time = Instant::now();
        let mut logs = vec![format!("Starting compilation of {} source files", language)];

        // Check if the language adapter exists
        let adapter = self.adapters.get(language)
            .ok_or_else(|| CompilerError::CompilationFailed {
                message: format!("Unsupported language: {}", language)
            })?;

        // Validate toolchain
        logs.push("Validating toolchain...".to_string());
        if !adapter.validate_toolchain().await? {
            return Err(CompilerError::ToolchainNotFound {
                toolchain: language.to_string()
            });
        }

        // Execute compilation
        logs.push("Executing compilation...".to_string());
        let result = adapter.compile(source_files, config).await?;

        let duration = start_time.elapsed().as_millis();
        tracing::info!("[MAIDOS-AUDIT] Compilation completed in {} ms", duration);
        
        Ok(result)
    }
}

impl Default for CompilerCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct MockAdapter;
    
    #[async_trait::async_trait]
    impl LanguageAdapter for MockAdapter {
        fn language_name(&self) -> &'static str {
            "mock"
        }
        
        fn supported_extensions(&self) -> &[&'static str] {
            &[".mock"]
        }
        
        async fn validate_toolchain(&self) -> Result<bool, CompilerError> {
            Ok(true)
        }
        
        async fn compile(&self, _source_files: Vec<String>, _config: CompileConfig) -> Result<CompileResult, CompilerError> {
            Ok(CompileResult::success(
                vec!["output/mock.o".to_string()], 
                10, 
                vec!["Compilation succeeded".to_string()],
                vec![]
            ))
        }
        
        async fn extract_interface(&self, _artifact_path: &str) -> Result<Option<String>, CompilerError> {
            Ok(Some("interface".to_string()))
        }
        
        async fn generate_glue(&self, _interface: &str, _target_language: &str) -> Result<String, CompilerError> {
            Ok("glue code".to_string())
        }
    }
    
    #[tokio::test]
    async fn test_compiler_core() {
        let mut compiler = CompilerCore::new();
        compiler.register_adapter(Box::new(MockAdapter));
        
        let config = CompileConfig {
            target: "x86_64-unknown-linux-gnu".to_string(),
            mode: CompileMode::Debug,
            incremental: true,
            custom_flags: vec![],
            output_dir: "output".to_string(),
        };
        
        let result = compiler
            .compile_source("mock", vec!["test.mock".to_string()], config)
            .await
            .expect("Compilation should succeed");
        
        assert!(result.success);
        assert_eq!(result.artifacts.len(), 1);
    }
    
    #[tokio::test]
    async fn test_unsupported_language() {
        let compiler = CompilerCore::new();
        
        let config = CompileConfig {
            target: "x86_64-unknown-linux-gnu".to_string(),
            mode: CompileMode::Debug,
            incremental: true,
            custom_flags: vec![],
            output_dir: "output".to_string(),
        };
        
        let result = compiler
            .compile_source("unsupported", vec!["test.unsupported".to_string()], config)
            .await;
        
        assert!(result.is_err());
    }
}