//! MAIDOS Forge core library
//!
//! Provides core functionality for the cross-language compilation framework
//! Compliant with MAIDOS Forge specification v2.1


/// Core compiler module
pub mod compiler;

/// Language adapter module
pub mod languages;

/// Configuration management module
pub mod config;

/// Error handling module
pub mod error;

/// Parser module
pub mod parser;

/// Code checker module
pub mod checker;

/// Dependency management module
pub mod dependency;

/// Filesystem operations module
pub mod fs;

/// Plugin system module
pub mod plugin;

/// Scheduler module
pub mod scheduler;

/// FFI export module -- C-compatible interface for C# P/Invoke calls
pub mod ffi;

// Re-export main components for convenience
pub use compiler::{CompilerCore, LanguageAdapter, CompileConfig, CompileResult, CompileMode, CompilerError};
pub use parser::{Parser, TreeSitterParser, ParseResult};
pub use checker::{Checker, RustChecker, CChecker, CheckResult};

/// MAIDOS Forge version information
pub const VERSION: &str = "2.1.0";
pub const CODENAME: &str = "Forge";

/// Initialize MAIDOS Forge core
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging system
    tracing_subscriber::fmt::init();
    
    // [MAIDOS-AUDIT] System initialization
    tracing::info!("[MAIDOS-AUDIT] MAIDOS Forge core initialized v{}", VERSION);
    
    Ok(())
}