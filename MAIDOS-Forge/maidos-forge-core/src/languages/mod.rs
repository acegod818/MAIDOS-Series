//! Language adapter module
//!
//! Contains adapter implementations for various languages

pub mod rust;
pub mod c;
pub mod cpp;
pub mod javascript;
pub mod python;
pub mod go;

// Re-export core components
pub use crate::compiler::{LanguageAdapter, CompileConfig, CompileResult, CompileMode};