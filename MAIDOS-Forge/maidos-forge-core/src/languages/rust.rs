//! Rust language adapter implementation
//!
//! Compliant with MAIDOS Forge specification v2.1

use std::process::Command;
use async_trait::async_trait;
use tracing::{info, error};

use crate::compiler::{LanguageAdapter, CompileConfig, CompileResult, CompileMode, CompilerError};

/// Rust language adapter
pub struct RustAdapter;

#[async_trait]
impl LanguageAdapter for RustAdapter {
    fn language_name(&self) -> &'static str {
        "rust"
    }
    
    fn supported_extensions(&self) -> &[&'static str] {
        &[".rs"]
    }
    
    async fn validate_toolchain(&self) -> Result<bool, CompilerError> {
        // [MAIDOS-AUDIT] Validate Rust toolchain
        info!("[MAIDOS-AUDIT] Validating Rust toolchain");
        
        let output = Command::new("rustc")
            .arg("--version")
            .output();
            
        match output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                info!("Rust compiler detected: {}", version);
                Ok(true)
            }
            _ => {
                error!("Rust compiler not found, please install the Rust toolchain");
                Ok(false)
            }
        }
    }
    
    async fn compile(&self, source_files: Vec<String>, config: CompileConfig) -> Result<CompileResult, CompilerError> {
        // [MAIDOS-AUDIT] Start Rust compilation
        info!("[MAIDOS-AUDIT] Starting Rust source file compilation");
        
        let mut logs = vec![];
        let mut warnings = vec![];
        
        // Ensure at least one entry file is provided
        if source_files.is_empty() {
            return Err(CompilerError::CompilationFailed {
                message: "No source files provided".to_string()
            });
        }
        
        let main_file = &source_files[0];
        logs.push(format!("Compiling main file: {}", main_file));
        
        // Build cargo command
        let mut cmd = Command::new("cargo");
        
        // Set parameters based on compile mode
        match config.mode {
            CompileMode::Debug => {
                cmd.arg("build");
            }
            CompileMode::Release => {
                cmd.arg("build").arg("--release");
            }
            CompileMode::Custom(_) => {
                // For custom mode, use the default debug build
                cmd.arg("build");
            }
        }
        
        // Add custom flags
        for flag in &config.custom_flags {
            cmd.arg(flag);
        }
        
        logs.push("Running Cargo build...".to_string());
        
        // Execute compilation command
        let output = tokio::process::Command::from(cmd)
            .output()
            .await
            .map_err(|e| CompilerError::InternalError {
                source: Box::new(e)
            })?;
        
        // Process compilation result
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        logs.extend(stdout.lines().map(|s| s.to_string()));
        warnings.extend(stderr.lines().map(|s| s.to_string()));
        
        if output.status.success() {
            // Compilation succeeded
            let artifacts = vec![format!("{}/target/debug/app", config.output_dir)];
            Ok(CompileResult::success(artifacts, 0, logs, warnings))
        } else {
            // Compilation failed
            Err(CompilerError::CompilationFailed {
                message: stderr.to_string()
            })
        }
    }
    
    async fn extract_interface(&self, artifact_path: &str) -> Result<Option<String>, CompilerError> {
        // For Rust, we can use rustdoc to extract the interface
        info!("[MAIDOS-AUDIT] Extracting Rust interface: {}", artifact_path);

        // This is a simplified implementation; a real one would be more complex
        Ok(Some("Rust interface definition".to_string()))
    }
    
    async fn generate_glue(&self, interface: &str, target_language: &str) -> Result<String, CompilerError> {
        // Generate glue code
        info!("[MAIDOS-AUDIT] Generating glue code for {} language", target_language);

        // This is a simplified implementation
        Ok(format!("// Glue code for {}\n{}", target_language, interface))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rust_adapter_creation() {
        let adapter = RustAdapter;
        assert_eq!(adapter.language_name(), "rust");
        assert_eq!(adapter.supported_extensions(), &[".rs"]);
    }
    
    #[tokio::test]
    #[ignore = "Rust tests require a specific cargo environment and may be unstable in the test environment"]
    async fn test_rust_compilation_failure() {
        let adapter = RustAdapter;
        let config = CompileConfig {
            target: "x86_64-unknown-linux-gnu".to_string(),
            mode: CompileMode::Debug,
            incremental: true,
            custom_flags: vec![],
            output_dir: "output".to_string(),
        };
        
        // Test with a nonexistent file
        let result = adapter.compile(vec!["nonexistent.rs".to_string()], config).await;
        
        // Since the Rust adapter uses cargo build, it checks if Cargo.toml exists.
        // If Cargo.toml is missing, cargo will fail, so this test should pass.
        assert!(result.is_err());
    }
}