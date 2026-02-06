//! Module configuration
//!
//! Defines configuration options for compilation modules

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    /// Module name
    pub name: String,

    /// Source code path
    pub src: String,

    /// Dependencies
    pub dependencies: Vec<String>,

    /// Compilation options
    pub options: HashMap<String, String>,
}

impl ModuleConfig {
    /// Create a new module configuration
    pub fn new(name: String, src: String) -> Self {
        Self {
            name,
            src,
            dependencies: Vec::new(),
            options: HashMap::new(),
        }
    }
    
    /// Add a dependency
    pub fn with_dependency(mut self, dependency: String) -> Self {
        self.dependencies.push(dependency);
        self
    }
    
    /// Set an option
    pub fn with_option(mut self, key: String, value: String) -> Self {
        self.options.insert(key, value);
        self
    }
}