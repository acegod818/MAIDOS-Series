//! High-performance parser module
//! Implements incremental parsing using Tree-sitter

use crate::error::ForgeError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Parse result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// Whether parsing succeeded
    pub success: bool,
    /// Error message (if any)
    pub error: Option<String>,
    /// Parsed syntax tree
    pub tree: Option<SyntaxTree>,
    /// Parse duration
    pub duration_ms: u64,
}

/// Syntax tree representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxTree {
    /// Root node
    pub root: SyntaxNode,
    /// File hash (used for incremental parsing)
    pub file_hash: String,
    /// Language type
    pub language: String,
}

/// Syntax node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxNode {
    /// Node type
    pub kind: String,
    /// Text content
    pub text: String,
    /// Child nodes
    pub children: Vec<SyntaxNode>,
    /// Line number
    pub line: usize,
    /// Column number
    pub column: usize,
}

/// Parser trait
pub trait Parser: Send + Sync {
    /// Parse a source code file
    fn parse(&mut self, source_path: &Path) -> Result<ParseResult, ForgeError>;

    /// Incremental parse (based on a previous syntax tree)
    fn parse_incremental(&mut self, source_path: &Path, previous_tree: Option<&SyntaxTree>) -> Result<ParseResult, ForgeError>;

    /// Supported language
    fn language(&self) -> &str;
}

/// Tree-sitter parser implementation
pub struct TreeSitterParser {
    language: String,
    parser: tree_sitter::Parser,
}

impl TreeSitterParser {
    /// Create a new parser instance
    pub fn new(language: &str) -> Result<Self, ForgeError> {
        let mut parser = tree_sitter::Parser::new();
        
        // Set the parser for the corresponding language
        let language_obj = match language {
            "rust" => tree_sitter_rust::language(),
            "c" => tree_sitter_c::language(),
            "cpp" => tree_sitter_cpp::language(),
            _ => return Err(ForgeError::UnsupportedLanguage(language.to_string())),
        };
        
        parser
            .set_language(language_obj)
            .map_err(|e| ForgeError::ParserError(format!("Failed to set language: {}", e)))?;
        
        Ok(Self {
            language: language.to_string(),
            parser,
        })
    }
}

impl Parser for TreeSitterParser {
    fn parse(&mut self, source_path: &Path) -> Result<ParseResult, ForgeError> {
        let start_time = std::time::Instant::now();
        
        // Read source file
        let source_code = std::fs::read_to_string(source_path)
            .map_err(ForgeError::IoError)?;

        // Calculate file hash
        let file_hash = calculate_hash(&source_code);

        // Parse source code
        let tree = self.parser.parse(&source_code, None)
            .ok_or_else(|| ForgeError::ParserError("Failed to parse source code".to_string()))?;
        
        let duration = start_time.elapsed().as_millis() as u64;
        
        Ok(ParseResult {
            success: true,
            error: None,
            tree: Some(SyntaxTree {
                root: convert_node(tree.root_node(), &source_code)?,
                file_hash,
                language: self.language.clone(),
            }),
            duration_ms: duration,
        })
    }
    
    fn parse_incremental(&mut self, source_path: &Path, previous_tree: Option<&SyntaxTree>) -> Result<ParseResult, ForgeError> {
        let start_time = std::time::Instant::now();
        
        // Read source file
        let source_code = std::fs::read_to_string(source_path)
            .map_err(ForgeError::IoError)?;

        // Calculate file hash
        let file_hash = calculate_hash(&source_code);

        // Check if re-parsing is needed
        if let Some(prev_tree) = previous_tree {
            if prev_tree.file_hash == file_hash {
                // File unchanged, return previous result directly
                return Ok(ParseResult {
                    success: true,
                    error: None,
                    tree: Some(prev_tree.clone()),
                    duration_ms: 0,
                });
            }
        }
        
        // Perform incremental parsing based on the previous tree
        let old_tree = previous_tree.and_then(|_t| {
            // This should convert prev_tree to tree_sitter::Tree
            // For simplicity, we return None for now
            None
        });
        
        let tree = self.parser.parse(&source_code, old_tree.as_ref())
            .ok_or_else(|| ForgeError::ParserError("Failed to parse source code".to_string()))?;
        
        let duration = start_time.elapsed().as_millis() as u64;
        
        Ok(ParseResult {
            success: true,
            error: None,
            tree: Some(SyntaxTree {
                root: convert_node(tree.root_node(), &source_code)?,
                file_hash,
                language: self.language.clone(),
            }),
            duration_ms: duration,
        })
    }
    
    fn language(&self) -> &str {
        &self.language
    }
}

/// Convert a Tree-sitter node to the internal representation
fn convert_node(node: tree_sitter::Node, source: &str) -> Result<SyntaxNode, ForgeError> {
    let text = node
        .utf8_text(source.as_bytes())
        .map_err(|e| ForgeError::ParserError(format!("Failed to get node text: {}", e)))?
        .to_string();
    
    let mut children = Vec::new();
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            children.push(convert_node(child, source)?);
        }
    }
    
    let start_position = node.start_position();
    
    Ok(SyntaxNode {
        kind: node.kind().to_string(),
        text,
        children,
        line: start_position.row + 1,
        column: start_position.column + 1,
    })
}

/// Calculate a simple hash of a string
fn calculate_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_rust_parser() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "fn main() {{ println!(\"Hello\"); }}")?;
        
        let mut parser = TreeSitterParser::new("rust")?;
        let result = parser.parse(temp_file.path())?;
        
        assert!(result.success);
        assert!(result.tree.is_some());
        assert_eq!(result.tree.as_ref().expect("Tree should exist").language, "rust");
        Ok(())
    }
    
    #[test]
    fn test_c_parser() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "int main() {{ return 0; }}")?;
        
        let mut parser = TreeSitterParser::new("c")?;
        let result = parser.parse(temp_file.path())?;
        
        assert!(result.success);
        assert!(result.tree.is_some());
        assert_eq!(result.tree.as_ref().expect("Tree should exist").language, "c");
        Ok(())
    }
}
