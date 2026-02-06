//! Template Language Plugin for MAIDOS Forge
//!
//! This is a template implementation that developers can use as a starting point
//! for creating their own language plugins.

use async_trait::async_trait;
use std::path::Path;
use maidos_forge_core::languages::adapter::{
    LanguageAdapter, CompileOptions, CompileResult, ToolchainInfo, 
    InterfaceDescription, GlueCodeResult
};
use maidos_forge_core::error::Result;
use maidos_forge_core::parser::ParseResult;
use maidos_forge_core::checker::CheckResult;

/// Template Language Adapter
///
/// This is a template implementation of the LanguageAdapter trait.
/// Developers should provide actual logic for their specific language
/// by extending this base.
pub struct TemplateLanguageAdapter;

#[async_trait]
impl LanguageAdapter for TemplateLanguageAdapter {
    /// Get the language ID
    fn language_id(&self) -> &str {
        "template"
    }

    /// Get the language name
    fn language_name(&self) -> &str {
        "Template Language"
    }

    /// Get supported file extensions
    fn extensions(&self) -> &[&str] {
        &[".tmpl", ".template"]
    }

    /// Compile source files
    async fn compile(
        &self,
        source_files: &[std::path::PathBuf],
        output_dir: &Path,
        options: &CompileOptions,
    ) -> Result<CompileResult> {
        // [TEMPLATE] Implementation point: Add actual compilation logic here.
        // For this template, we simulate a successful compilation without output.
        
        let start_time = std::time::Instant::now();
        
        let logs = vec![
            format!("Compiling {} template files", source_files.len()),
            "Using template compiler v1.0".to_string(),
        ];
        
        // Implementation Guideline:
        // 1. Validate source files
        // 2. Invoke the language-specific compiler
        // 3. Handle compilation errors
        // 4. Return the compiled artifacts
        
        let outputs = vec![]; // No outputs for template base
        
        let duration = start_time.elapsed();
        
        Ok(CompileResult {
            success: true,
            outputs,
            duration,
            error: None,
            logs,
            warnings: vec![],
        })
    }

    /// Check if toolchain is available
    async fn check_toolchain(&self) -> Result<bool> {
        // [TEMPLATE] Implementation point: Check for required compiler/tools.
        // Returns true for template validation.
        
        Ok(true)
    }

    /// Get toolchain information
    async fn toolchain_info(&self) -> Result<ToolchainInfo> {
        // [TEMPLATE] Implementation point: Retrieve actual toolchain info.
        
        Ok(ToolchainInfo {
            name: "template-compiler".to_string(),
            version: "1.0.0".to_string(),
            targets: vec!["native".to_string()],
            features: vec!["basic-compilation".to_string()],
        })
    }

    /// Parse source file
    async fn parse(&self, source_file: &Path) -> Result<ParseResult> {
        // [TEMPLATE] Implementation point: Implement AST generation logic.
        
        Ok(ParseResult {
            success: true,
            error: None,
            tree: None,
            duration_ms: 0,
        })
    }

    /// Check source file for errors
    async fn check(&self, source_file: &Path) -> Result<CheckResult> {
        // [TEMPLATE] Implementation point: Implement static analysis.
        
        Ok(CheckResult {
            success: true,
            errors: vec![],
            warnings: vec![],
            duration_ms: 0,
        })
    }

    /// Extract interface from compiled artifact
    async fn extract_interface(&self, artifact_path: &Path) -> Result<InterfaceDescription> {
        // [TEMPLATE] Implementation point: Extract function signatures.
        
        Ok(InterfaceDescription {
            version: "1.0".to_string(),
            module: maidos_forge_core::languages::adapter::InterfaceModule {
                name: "template_module".to_string(),
                version: "1.0.0".to_string(),
            },
            language: maidos_forge_core::languages::adapter::InterfaceLanguage {
                name: "template".to_string(),
                abi: "template".to_string(),
            },
            exports: vec![],
        })
    }

    /// Generate glue code for interfacing with other languages
    async fn generate_glue(&self, interface: &InterfaceDescription, target_language: &str) -> Result<GlueCodeResult> {
        // [TEMPLATE] Implementation point: Generate wrapper code.
        
        match target_language {
            "rust" | "c" | "javascript" => {
                let code = format!("// Generated glue code for {}\n// Template implementation: Add logic here", interface.module.name);
                Ok(GlueCodeResult::success(
                    code, 
                    format!("{}_glue.{}", interface.module.name, get_extension(target_language)), 
                    target_language.to_string()
                ))
            },
            _ => Ok(GlueCodeResult::failure(format!("Unsupported target language: {}", target_language))),
        }
    }
}

/// Helper function to get file extension for a target language
fn get_extension(target_language: &str) -> &str {
    match target_language {
        "rust" => "rs",
        "c" => "c",
        "javascript" => "js",
        _ => "txt",
    }
}

/// Create a new instance of the template language adapter
pub fn create_adapter() -> TemplateLanguageAdapter {
    TemplateLanguageAdapter
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_template_adapter_creation() {
        let adapter = TemplateLanguageAdapter;
        assert_eq!(adapter.language_id(), "template");
        assert_eq!(adapter.language_name(), "Template Language");
    }

    #[tokio::test]
    async fn test_compile_base() {
        let adapter = TemplateLanguageAdapter;
        let temp_dir = TempDir::new().unwrap();
        let source_files = vec![PathBuf::from("test.tmpl")];
        let options = CompileOptions::default();
        
        let result = adapter.compile(&source_files, temp_dir.path(), &options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_toolchain_check_base() {
        let adapter = TemplateLanguageAdapter;
        let result = adapter.check_toolchain().await;
        assert!(result.is_ok());
    }
}