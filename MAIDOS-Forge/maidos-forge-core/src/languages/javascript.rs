//! JavaScript語言適配器實現
//! 
//! 符合MAIDOS Forge規格v2.1要求

use std::process::Command;
use async_trait::async_trait;
use tracing::{info, error};

use crate::compiler::{LanguageAdapter, CompileConfig, CompileResult, CompilerError};

/// JavaScript語言適配器
pub struct JavaScriptAdapter;

#[async_trait]
impl LanguageAdapter for JavaScriptAdapter {
    fn language_name(&self) -> &'static str {
        "javascript"
    }
    
    fn supported_extensions(&self) -> &[&'static str] {
        &[".js", ".mjs", ".cjs"]
    }
    
    async fn validate_toolchain(&self) -> Result<bool, CompilerError> {
        // [MAIDOS-AUDIT] 驗證Node.js工具鏈
        info!("[MAIDOS-AUDIT] 驗證Node.js工具鏈");
        
        let output = Command::new("node")
            .arg("--version")
            .output();
            
        match output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                info!("檢測到Node.js: {}", version);
                Ok(true)
            }
            _ => {
                error!("未找到Node.js，請安裝Node.js");
                Ok(false)
            }
        }
    }
    
    async fn compile(&self, source_files: Vec<String>, config: CompileConfig) -> Result<CompileResult, CompilerError> {
        // [MAIDOS-AUDIT] 開始JavaScript處理
        info!("[MAIDOS-AUDIT] 開始處理JavaScript源文件");
        
        let mut logs = vec![];
        let mut warnings = vec![];
        
        // 確保至少有一個源文件
        if source_files.is_empty() {
            return Err(CompilerError::CompilationFailed {
                message: "沒有提供源文件".to_string()
            });
        }
        
        let main_file = &source_files[0];
        logs.push(format!("處理主文件: {}", main_file));
        
        // 對於JavaScript，我們可以使用Node.js直接運行或者使用打包工具
        // 這裡我們假設使用Node.js直接運行
        let mut cmd = Command::new("node");
        
        // 添加源文件
        cmd.arg(main_file);
        
        // 添加自定義標誌
        for flag in &config.custom_flags {
            cmd.arg(flag);
        }
        
        logs.push("執行Node.js...".to_string());
        
        // 執行命令
        let output = tokio::process::Command::from(cmd)
            .output()
            .await
            .map_err(|e| CompilerError::InternalError {
                source: Box::new(e)
            })?;
        
        // 處理結果
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        logs.extend(stdout.lines().map(|s| s.to_string()));
        warnings.extend(stderr.lines().map(|s| s.to_string()));
        
        if output.status.success() {
            // 成功執行
            let artifacts = vec![format!("{}/app.js", config.output_dir)];
            Ok(CompileResult::success(artifacts, 0, logs, warnings))
        } else {
            // 執行失敗
            Err(CompilerError::CompilationFailed {
                message: stderr.to_string()
            })
        }
    }
    
    async fn extract_interface(&self, artifact_path: &str) -> Result<Option<String>, CompilerError> {
        // 對於JavaScript，我們可以解析文件來提取接口
        info!("[MAIDOS-AUDIT] 提取JavaScript接口: {}", artifact_path);
        
        // 這是一個簡化的實現
        Ok(Some("JavaScript接口定義".to_string()))
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
    use crate::compiler::CompileMode;
    
    #[tokio::test]
    async fn test_javascript_adapter_creation() {
        let adapter = JavaScriptAdapter;
        assert_eq!(adapter.language_name(), "javascript");
        assert_eq!(adapter.supported_extensions(), &[".js", ".mjs", ".cjs"]);
    }
    
    #[tokio::test]
    async fn test_javascript_execution_failure() {
        let adapter = JavaScriptAdapter;
        let config = CompileConfig {
            target: "x86_64-unknown-linux-gnu".to_string(),
            mode: CompileMode::Debug,
            incremental: true,
            custom_flags: vec![],
            output_dir: "output".to_string(),
        };
        
        // 使用不存在的文件進行測試
        let result = adapter.compile(vec!["nonexistent.js".to_string()], config).await;
        assert!(result.is_err());
    }
}