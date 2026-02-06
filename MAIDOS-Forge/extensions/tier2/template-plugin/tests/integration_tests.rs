//! Integration tests for the Template Language Plugin
//!
//! These tests verify that the plugin works correctly with the MAIDOS Forge core.

use template_language_plugin::create_adapter;
use maidos_forge_core::languages::adapter::LanguageAdapter;
use tempfile::TempDir;

#[tokio::test]
async fn test_plugin_registration() {
    let adapter = create_adapter();
    
    assert_eq!(adapter.language_id(), "template");
    assert_eq!(adapter.language_name(), "Template Language");
    assert_eq!(adapter.extensions(), &[".tmpl", ".template"]);
}

#[tokio::test]
async fn test_compilation_process() {
    let adapter = create_adapter();
    let temp_dir = TempDir::new().unwrap();
    
    // Create dummy source files
    let source_files = vec![
        temp_dir.path().join("test1.tmpl"),
        temp_dir.path().join("test2.template"),
    ];
    
    let options = maidos_forge_core::languages::adapter::CompileOptions::default();
    
    let result = adapter.compile(&source_files, temp_dir.path(), &options).await;
    
    assert!(result.is_ok());
    let compile_result = result.unwrap();
    assert!(compile_result.success);
    assert!(compile_result.logs.len() > 0);
}

#[tokio::test]
async fn test_toolchain_availability() {
    let adapter = create_adapter();
    let result = adapter.check_toolchain().await;
    
    assert!(result.is_ok());
    // In the template, toolchain check always returns true
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_interface_generation() {
    let adapter = create_adapter();
    let temp_dir = TempDir::new().unwrap();
    let artifact_path = temp_dir.path().join("test_artifact");
    
    let result = adapter.extract_interface(&artifact_path).await;
    
    assert!(result.is_ok());
    let interface = result.unwrap();
    assert_eq!(interface.version, "1.0");
    assert_eq!(interface.language.name, "template");
}

#[tokio::test]
async fn test_glue_code_generation() {
    let adapter = create_adapter();
    let interface = maidos_forge_core::languages::adapter::InterfaceDescription {
        version: "1.0".to_string(),
        module: maidos_forge_core::languages::adapter::InterfaceModule {
            name: "test_module".to_string(),
            version: "1.0.0".to_string(),
        },
        language: maidos_forge_core::languages::adapter::InterfaceLanguage {
            name: "template".to_string(),
            abi: "template".to_string(),
        },
        exports: vec![],
    };
    
    // Test supported languages
    let rust_result = adapter.generate_glue(&interface, "rust").await;
    assert!(rust_result.is_ok());
    let rust_glue = rust_result.unwrap();
    assert!(rust_glue.success);
    assert!(rust_glue.code.is_some());
    assert_eq!(rust_glue.filename.unwrap(), "test_module_glue.rs");
    
    let c_result = adapter.generate_glue(&interface, "c").await;
    assert!(c_result.is_ok());
    let c_glue = c_result.unwrap();
    assert!(c_glue.success);
    assert!(c_glue.code.is_some());
    assert_eq!(c_glue.filename.unwrap(), "test_module_glue.c");
    
    // Test unsupported language
    let unsupported_result = adapter.generate_glue(&interface, "unsupported").await;
    assert!(unsupported_result.is_ok());
    let unsupported_glue = unsupported_result.unwrap();
    assert!(!unsupported_glue.success);
    assert!(unsupported_glue.error.is_some());
}