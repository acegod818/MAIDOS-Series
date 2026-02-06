//! Python language adapter implementation
//!
//! Compliant with MAIDOS Forge specification v2.1

use std::process::Command;
use async_trait::async_trait;
use tracing::{info, error};

use crate::compiler::{LanguageAdapter, CompileConfig, CompileResult, CompilerError};

/// Python language adapter
pub struct PythonAdapter;

#[async_trait]
impl LanguageAdapter for PythonAdapter {
    fn language_name(&self) -> &'static str {
        "python"
    }
    
    fn supported_extensions(&self) -> &[&'static str] {
        &[".py"]
    }
    
    async fn validate_toolchain(&self) -> Result<bool, CompilerError> {
        // [MAIDOS-AUDIT] Validate Python toolchain
        info!("[MAIDOS-AUDIT] Validating Python toolchain");
        
        let output = Command::new("python")
            .arg("--version")
            .output();
            
        match output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                info!("Python detected: {}", version);
                Ok(true)
            }
            _ => {
                error!("Python not found, please install Python");
                Ok(false)
            }
        }
    }
    
    async fn compile(&self, source_files: Vec<String>, config: CompileConfig) -> Result<CompileResult, CompilerError> {
        // [MAIDOS-AUDIT] Start Python processing
        info!("[MAIDOS-AUDIT] Starting Python source file processing");
        
        let mut logs = vec![];
        let mut warnings = vec![];
        
        // Ensure at least one source file is provided
        if source_files.is_empty() {
            return Err(CompilerError::CompilationFailed {
                message: "No source files provided".to_string()
            });
        }
        
        let main_file = &source_files[0];
        logs.push(format!("Processing main file: {}", main_file));
        
        // For Python, we can run directly with the Python interpreter
        let mut cmd = Command::new("python");
        
        // Add source file
        cmd.arg(main_file);
        
        // Add custom flags
        for flag in &config.custom_flags {
            cmd.arg(flag);
        }
        
        logs.push("Running Python...".to_string());
        
        // Execute command
        let output = tokio::process::Command::from(cmd)
            .output()
            .await
            .map_err(|e| CompilerError::InternalError {
                source: Box::new(e)
            })?;
        
        // Process result
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        logs.extend(stdout.lines().map(|s| s.to_string()));
        warnings.extend(stderr.lines().map(|s| s.to_string()));
        
        if output.status.success() {
            // Execution succeeded
            let artifacts = vec![format!("{}/app.py", config.output_dir)];
            Ok(CompileResult::success(artifacts, 0, logs, warnings))
        } else {
            // Execution failed
            Err(CompilerError::CompilationFailed {
                message: stderr.to_string()
            })
        }
    }
    
    async fn extract_interface(&self, artifact_path: &str) -> Result<Option<String>, CompilerError> {
        // For Python, we can parse the file to extract the interface
        info!("[MAIDOS-AUDIT] Extracting Python interface: {}", artifact_path);

        // This is a simplified implementation
        Ok(Some("Python interface definition".to_string()))
    }
    
    async fn generate_glue(&self, interface: &str, target_language: &str) -> Result<String, CompilerError> {
        // Generate glue code
        info!("[MAIDOS-AUDIT] Generating glue code for {} language", target_language);

        // This is a simplified implementation
        Ok(format!("# Glue code for {}\n{}", target_language, interface))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::CompileMode;
    
    #[tokio::test]
    async fn test_python_adapter_creation() {
        let adapter = PythonAdapter;
        assert_eq!(adapter.language_name(), "python");
        assert_eq!(adapter.supported_extensions(), &[".py"]);
    }
    
    #[tokio::test]
    async fn test_python_execution_failure() {
        let adapter = PythonAdapter;
        let config = CompileConfig {
            target: "x86_64-unknown-linux-gnu".to_string(),
            mode: CompileMode::Debug,
            incremental: true,
            custom_flags: vec![],
            output_dir: "output".to_string(),
        };
        
        // Test with a nonexistent file
        let result = adapter.compile(vec!["nonexistent.py".to_string()], config).await;
        assert!(result.is_err());
    }
}
