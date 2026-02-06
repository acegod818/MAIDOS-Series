//! C++ language adapter implementation
//!
//! Compliant with MAIDOS Forge specification v2.1

use std::process::Command;
use async_trait::async_trait;
use tracing::{info, error};

use crate::compiler::{LanguageAdapter, CompileConfig, CompileResult, CompileMode, CompilerError};

/// C++ language adapter
pub struct CppAdapter;

#[async_trait]
impl LanguageAdapter for CppAdapter {
    fn language_name(&self) -> &'static str {
        "cpp"
    }
    
    fn supported_extensions(&self) -> &[&'static str] {
        &[".cpp", ".cc", ".cxx", ".hpp", ".h"]
    }
    
    async fn validate_toolchain(&self) -> Result<bool, CompilerError> {
        // [MAIDOS-AUDIT] Validate C++ toolchain
        info!("[MAIDOS-AUDIT] Validating C++ toolchain");
        
        let output = Command::new("g++")
            .arg("--version")
            .output();
            
        match output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                info!("G++ compiler detected: {}", version);
                Ok(true)
            }
            _ => {
                error!("G++ compiler not found, please install the G++ toolchain");
                Ok(false)
            }
        }
    }
    
    async fn compile(&self, source_files: Vec<String>, config: CompileConfig) -> Result<CompileResult, CompilerError> {
        // [MAIDOS-AUDIT] Start C++ compilation
        info!("[MAIDOS-AUDIT] Starting C++ source file compilation");
        
        let mut logs = vec![];
        let mut warnings = vec![];
        
        // Ensure at least one source file is provided
        if source_files.is_empty() {
            return Err(CompilerError::CompilationFailed {
                message: "No source files provided".to_string()
            });
        }
        
        let main_file = &source_files[0];
        logs.push(format!("Compiling main file: {}", main_file));
        
        // Build G++ command
        let mut cmd = Command::new("g++");
        
        // Add source files
        for file in &source_files {
            cmd.arg(file);
        }
        
        // Set output file
        let output_file = format!("{}/app", config.output_dir);
        cmd.arg("-o").arg(&output_file);
        
        // Set parameters based on compile mode
        match config.mode {
            CompileMode::Debug => {
                cmd.arg("-g"); // Enable debug info
            }
            CompileMode::Release => {
                cmd.arg("-O2"); // Enable optimizations
            }
            CompileMode::Custom(_) => {
                // Custom mode: no additional parameters
            }
        }
        
        // Add custom flags
        for flag in &config.custom_flags {
            cmd.arg(flag);
        }
        
        logs.push("Running G++ compilation...".to_string());
        
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
            let artifacts = vec![output_file];
            Ok(CompileResult::success(artifacts, 0, logs, warnings))
        } else {
            // Compilation failed
            Err(CompilerError::CompilationFailed {
                message: stderr.to_string()
            })
        }
    }
    
    async fn extract_interface(&self, artifact_path: &str) -> Result<Option<String>, CompilerError> {
        // For C++, we can use nm or objdump to extract the symbol table
        info!("[MAIDOS-AUDIT] Extracting C++ interface: {}", artifact_path);

        // This is a simplified implementation
        Ok(Some("C++ interface definition".to_string()))
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
    async fn test_cpp_adapter_creation() {
        let adapter = CppAdapter;
        assert_eq!(adapter.language_name(), "cpp");
        assert_eq!(adapter.supported_extensions(), &[".cpp", ".cc", ".cxx", ".hpp", ".h"]);
    }
    
    #[tokio::test]
    async fn test_cpp_compilation_failure() {
        let adapter = CppAdapter;
        let config = CompileConfig {
            target: "x86_64-unknown-linux-gnu".to_string(),
            mode: CompileMode::Debug,
            incremental: true,
            custom_flags: vec![],
            output_dir: "output".to_string(),
        };
        
        // Test with a nonexistent file
        let result = adapter.compile(vec!["nonexistent.cpp".to_string()], config).await;
        assert!(result.is_err());
    }
}