//! Filesystem operations module

use std::path::Path;

/// Filesystem operation result
#[derive(Debug)]
pub struct FsResult<T> {
    /// Whether the operation succeeded
    pub success: bool,
    /// Error message (if any)
    pub error: Option<String>,
    /// Result data
    pub data: Option<T>,
}

impl<T> FsResult<T> {
    /// Create a successful filesystem operation result.
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            error: None,
            data: Some(data),
        }
    }

    /// Create a failed filesystem operation result.
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            error: Some(error),
            data: None,
        }
    }
}

/// Read a file.
pub fn read_file(path: &Path) -> FsResult<String> {
    std::fs::read_to_string(path)
        .map(FsResult::success)
        .unwrap_or_else(|e| FsResult::failure(format!("Failed to read file: {}", e)))
}

/// Write a file.
pub fn write_file(path: &Path, content: &str) -> FsResult<()> {
    std::fs::write(path, content)
        .map(|_| FsResult::success(()))
        .unwrap_or_else(|e| FsResult::failure(format!("Failed to write file: {}", e)))
}