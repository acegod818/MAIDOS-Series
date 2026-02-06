//! Code checker module
//! Implements code quality checks and error detection

use crate::error::ForgeError;
use crate::parser::{ParseResult, SyntaxNode};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Whether the check succeeded
    pub success: bool,
    /// Error list
    pub errors: Vec<Diagnostic>,
    /// Warning list
    pub warnings: Vec<Diagnostic>,
    /// Check duration
    pub duration_ms: u64,
}

/// Diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Diagnostic kind
    pub kind: DiagnosticKind,
    /// Error code
    pub code: String,
    /// Message
    pub message: String,
    /// Location information
    pub location: Location,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Diagnostic kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticKind {
    /// Error
    Error,
    /// Warning
    Warning,
    /// Hint
    Hint,
}

/// Location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// Line number
    pub line: usize,
    /// Column number
    pub column: usize,
    /// File path
    pub file: String,
}

/// Checker trait
pub trait Checker: Send + Sync {
    /// Check parse results
    fn check(&self, parse_result: &ParseResult, file_path: &Path) -> Result<CheckResult, ForgeError>;

    /// Supported language
    fn language(&self) -> &'static str;
}

/// Rust language checker implementation
pub struct RustChecker;

impl RustChecker {
    /// Create a new Rust checker instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker for RustChecker {
    fn check(&self, parse_result: &ParseResult, file_path: &Path) -> Result<CheckResult, ForgeError> {
        let start_time = std::time::Instant::now();
        
        let errors = Vec::new();
        let mut warnings = Vec::new();
        
        if let Some(tree) = &parse_result.tree {
            // Check for unused variables
            self.check_unused_variables(&tree.root, file_path, &mut warnings);

            // Check for potential memory safety issues
            self.check_memory_safety(&tree.root, file_path, &mut warnings);

            // Check for deprecated API usage
            self.check_deprecated_apis(&tree.root, file_path, &mut warnings);
        }
        
        let duration = start_time.elapsed().as_millis() as u64;
        
        Ok(CheckResult {
            success: errors.is_empty(),
            errors,
            warnings,
            duration_ms: duration,
        })
    }
    
    fn language(&self) -> &'static str {
        "rust"
    }
}

impl RustChecker {
    /// Check for unused variables
    fn check_unused_variables(&self, node: &SyntaxNode, file_path: &Path, warnings: &mut Vec<Diagnostic>) {
        if node.kind == "let_declaration" {
            // Check if it contains "_"
            if !node.text.contains("_") {
                // Check if the variable is used
                let var_name = self.extract_variable_name(node);
                if !self.is_variable_used(var_name.as_str(), node) {
                    warnings.push(Diagnostic {
                        kind: DiagnosticKind::Warning,
                        code: "unused_variable".to_string(),
                        message: format!("Variable `{}` is never used", var_name),
                        location: Location {
                            line: node.line,
                            column: node.column,
                            file: file_path.to_string_lossy().to_string(),
                        },
                        suggestion: Some("Consider prefixing with `_` or removing the variable".to_string()),
                    });
                }
            }
        }
        
        // Recursively check child nodes
        for child in &node.children {
            self.check_unused_variables(child, file_path, warnings);
        }
    }

    /// Check for potential memory safety issues
    fn check_memory_safety(&self, node: &SyntaxNode, file_path: &Path, warnings: &mut Vec<Diagnostic>) {
        // Check for unsafe code blocks
        if node.kind == "unsafe_block" {
            warnings.push(Diagnostic {
                kind: DiagnosticKind::Warning,
                code: "unsafe_code".to_string(),
                message: "Usage of unsafe code detected".to_string(),
                location: Location {
                    line: node.line,
                    column: node.column,
                    file: file_path.to_string_lossy().to_string(),
                },
                suggestion: Some("Ensure unsafe code is properly reviewed and documented".to_string()),
            });
        }
        
        // Recursively check child nodes
        for child in &node.children {
            self.check_memory_safety(child, file_path, warnings);
        }
    }

    /// Check for deprecated API usage
    fn check_deprecated_apis(&self, node: &SyntaxNode, file_path: &Path, warnings: &mut Vec<Diagnostic>) {
        // Check for known deprecated APIs
        let deprecated_apis = [
            ("std::sync::atomic::spin_loop_hint", "std::hint::spin_loop"),
        ];
        
        for (old_api, new_api) in &deprecated_apis {
            if node.text.contains(old_api) {
                warnings.push(Diagnostic {
                    kind: DiagnosticKind::Warning,
                    code: "deprecated_api".to_string(),
                    message: format!("Usage of deprecated API `{}`", old_api),
                    location: Location {
                        line: node.line,
                        column: node.column,
                        file: file_path.to_string_lossy().to_string(),
                    },
                    suggestion: Some(format!("Use `{}` instead", new_api)),
                });
            }
        }
        
        // Recursively check child nodes
        for child in &node.children {
            self.check_deprecated_apis(child, file_path, warnings);
        }
    }

    /// Extract variable name
    fn extract_variable_name(&self, node: &SyntaxNode) -> String {
        // Simplified implementation; actual logic would be more complex
        node.text.clone()
    }

    /// Check if a variable is used
    fn is_variable_used(&self, var_name: &str, node: &SyntaxNode) -> bool {
        // Simplified implementation; actual logic would be more complex
        node.text.contains(var_name) && !node.text.starts_with("let ")
    }
}

/// C language checker implementation
pub struct CChecker;

impl CChecker {
    /// Create a new C checker instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for CChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker for CChecker {
    fn check(&self, parse_result: &ParseResult, file_path: &Path) -> Result<CheckResult, ForgeError> {
        let start_time = std::time::Instant::now();
        
        let errors = Vec::new();
        let mut warnings = Vec::new();
        
        if let Some(tree) = &parse_result.tree {
            // Check for memory leak risks
            self.check_memory_leaks(&tree.root, file_path, &mut warnings);

            // Check for buffer overflow risks
            self.check_buffer_overflows(&tree.root, file_path, &mut warnings);
        }
        
        let duration = start_time.elapsed().as_millis() as u64;
        
        Ok(CheckResult {
            success: errors.is_empty(),
            errors,
            warnings,
            duration_ms: duration,
        })
    }
    
    fn language(&self) -> &'static str {
        "c"
    }
}

impl CChecker {
    /// Check for memory leak risks
    fn check_memory_leaks(&self, node: &SyntaxNode, file_path: &Path, warnings: &mut Vec<Diagnostic>) {
        // Check for malloc/free pairs
        if node.text.contains("malloc") && !node.text.contains("free") {
            warnings.push(Diagnostic {
                kind: DiagnosticKind::Warning,
                code: "potential_memory_leak".to_string(),
                message: "Potential memory leak: malloc without corresponding free".to_string(),
                location: Location {
                    line: node.line,
                    column: node.column,
                    file: file_path.to_string_lossy().to_string(),
                },
                suggestion: Some("Ensure every malloc has a corresponding free".to_string()),
            });
        }
        
        // Recursively check child nodes
        for child in &node.children {
            self.check_memory_leaks(child, file_path, warnings);
        }
    }

    /// Check for buffer overflow risks
    fn check_buffer_overflows(&self, node: &SyntaxNode, file_path: &Path, warnings: &mut Vec<Diagnostic>) {
        // Check for dangerous functions
        let dangerous_functions = ["strcpy", "strcat", "sprintf", "gets"];
        for func in &dangerous_functions {
            if node.text.contains(func) {
                warnings.push(Diagnostic {
                    kind: DiagnosticKind::Warning,
                    code: "buffer_overflow_risk".to_string(),
                    message: format!("Usage of dangerous function `{}` may cause buffer overflow", func),
                    location: Location {
                        line: node.line,
                        column: node.column,
                        file: file_path.to_string_lossy().to_string(),
                    },
                    suggestion: Some("Consider using safer alternatives like strncpy, snprintf, etc.".to_string()),
                });
            }
        }
        
        // Recursively check child nodes
        for child in &node.children {
            self.check_buffer_overflows(child, file_path, warnings);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{TreeSitterParser, Parser};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_rust_checker() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "fn main() {{ let x = 5; println!(\"Hello\"); }}")?;
        
        let mut parser = TreeSitterParser::new("rust")?;
        let parse_result = parser.parse(temp_file.path())?;
        
        let checker = RustChecker::new();
        let check_result = checker.check(&parse_result, temp_file.path())?;
        
        // Should detect unused variable warnings
        assert!(!check_result.warnings.is_empty());
        Ok(())
    }
    
    #[test]
    fn test_c_checker() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "#include <stdio.h>\nint main() {{ char buf[10]; gets(buf); return 0; }}")?;
        
        let mut parser = TreeSitterParser::new("c")?;
        let parse_result = parser.parse(temp_file.path())?;
        
        let checker = CChecker::new();
        let check_result = checker.check(&parse_result, temp_file.path())?;
        
        // Should detect buffer overflow risk warnings
        assert!(!check_result.warnings.is_empty());
        Ok(())
    }
}