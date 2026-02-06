//! Go language adapter implementation
//!
//! Compliant with MAIDOS Forge specification v2.1

use std::process::Command;
use async_trait::async_trait;
use tracing::{info, error};

use crate::compiler::{LanguageAdapter, CompileConfig, CompileResult, CompileMode, CompilerError};

/// Go language adapter
pub struct GoAdapter;

#[async_trait]
impl LanguageAdapter for GoAdapter {
    fn language_name(&self) -> &'static str {
        "go"
    }
    
    fn supported_extensions(&self) -> &[&'static str] {
        &[".go"]
    }
    
    async fn validate_toolchain(&self) -> Result<bool, CompilerError> {
        // [MAIDOS-AUDIT] Validate Go toolchain
        info!("[MAIDOS-AUDIT] Validating Go toolchain");
        
        let output = Command::new("go")
            .arg("version")
            .output();
            
        match output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                info!("Go detected: {}", version);
                Ok(true)
            }
            _ => {
                error!("Go not found, please install Go");
                Ok(false)
            }
        }
    }
    
    async fn compile(&self, source_files: Vec<String>, config: CompileConfig) -> Result<CompileResult, CompilerError> {
        // [MAIDOS-AUDIT] Start Go compilation
        info!("[MAIDOS-AUDIT] Starting Go source file compilation");
        
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
        
        // Build Go command
        let mut cmd = Command::new("go");
        cmd.arg("build");
        
        // Set output file
        let output_file = format!("{}/app", config.output_dir);
        cmd.arg("-o").arg(&output_file);
        
        // Set parameters based on compile mode
        match config.mode {
            CompileMode::Debug => {
                cmd.arg("-gcflags"); // Enable debug info
                cmd.args(["-N", "-l"]);
            }
            CompileMode::Release => {
                cmd.arg("-ldflags"); // Enable optimizations (strip debug + DWARF)
                cmd.args(["-s", "-w"]);
            }
            CompileMode::Custom(_) => {
                // Custom mode: no additional parameters
            }
        }
        
        // Add source files
        for file in &source_files {
            cmd.arg(file);
        }
        
        // Add custom flags
        for flag in &config.custom_flags {
            cmd.arg(flag);
        }
        
        logs.push("Running Go compilation...".to_string());
        
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
            // 編譯失敗
            Err(CompilerError::CompilationFailed {
                message: stderr.to_string()
            })
        }
    }
    
    async fn extract_interface(&self, artifact_path: &str) -> Result<Option<String>, CompilerError> {
        // 對於Go語言，我們可以使用go doc來提取接口
        info!("[MAIDOS-AUDIT] 提取Go接口: {}", artifact_path);
        
        // 這是一個簡化的實現
        Ok(Some("Go接口定義".to_string()))
    }
    
    async fn generate_glue(&self, interface: &str, target_language: &str) -> Result<String, CompilerError> {
        // 生成膠水代碼
        info!("[MAIDOS-AUDIT] 生成{}語言的膠水代碼", target_language);
        
        // 這是一個簡化的實現
        Ok(format!("// {}語言的膠水代碼\n{}", target_language, interface))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_go_adapter_creation() {
        let adapter = GoAdapter;
        assert_eq!(adapter.language_name(), "go");
        assert_eq!(adapter.supported_extensions(), &[".go"]);
    }
    
    #[tokio::test]
    async fn test_go_compilation_failure() {
        let adapter = GoAdapter;
        let config = CompileConfig {
            target: "x86_64-unknown-linux-gnu".to_string(),
            mode: CompileMode::Debug,
            incremental: true,
            custom_flags: vec![],
            output_dir: "output".to_string(),
        };
        
        // 使用不存在的文件進行測試
        let result = adapter.compile(vec!["nonexistent.go".to_string()], config).await;
        assert!(result.is_err());
    }
}