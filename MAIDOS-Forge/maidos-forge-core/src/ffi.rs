//! MAIDOS-Forge FFI export layer
//!
//! Provides C-compatible interface for C# BuildOrchestrator to call Rust core functions.
//! Follows the SharedCore opaque handle + error code pattern.
//!
//! [MAIDOS-AUDIT] 6 feature exports + 4 memory management = 10 FFI functions

#![allow(dead_code)] // FFI exports are used externally

use std::ffi::{c_char, CStr, CString};
use std::path::Path;
use std::ptr;
use std::sync::Mutex;

use crate::parser::{TreeSitterParser, Parser, ParseResult};
use crate::checker::{Checker, RustChecker, CChecker, CheckResult};

// =====================================================================
// Global error storage (thread-safe) -- aligned with SharedCore pattern
// =====================================================================

static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn set_last_error(msg: String) {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = Some(msg);
    }
}

// =====================================================================
// FFI error codes -- aligned with SharedCore ConfigErrorCode style
// =====================================================================

/// FFI error codes
#[repr(C)]
pub enum ForgeErrorCode {
    Ok = 0,
    NullPointer = 1,
    InvalidUtf8 = 2,
    UnsupportedLanguage = 3,
    ParseFailed = 4,
    CheckFailed = 5,
    IoError = 6,
    SerializationError = 7,
    InternalError = 8,
}

// =====================================================================
// Helper: C string conversion
// =====================================================================

unsafe fn ptr_to_str<'a>(ptr: *const c_char) -> Result<&'a str, ForgeErrorCode> {
    if ptr.is_null() {
        return Err(ForgeErrorCode::NullPointer);
    }
    CStr::from_ptr(ptr)
        .to_str()
        .map_err(|_| ForgeErrorCode::InvalidUtf8)
}

fn string_to_c(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

// =====================================================================
// FFI export (1): Parse source code -> JSON AST
// =====================================================================

/// Parse a source file and return a JSON-formatted AST.
///
/// # Parameters
/// - `language`: Language name ("rust", "c", "cpp")
/// - `source_path`: Source file path (UTF-8)
///
/// # Returns
/// - JSON string pointer (on success); must be freed with `forge_free_string`
/// - null (on failure); use `forge_last_error` to retrieve the error message
///
/// # Safety
/// - `language` / `source_path` may be null; if non-null, they must point to valid NUL-terminated C strings.
/// - The memory pointed to must remain readable and valid for the duration of this function call.
#[no_mangle]
pub unsafe extern "C" fn forge_parse_source(
    language: *const c_char,
    source_path: *const c_char,
) -> *mut c_char {
    let lang = match ptr_to_str(language) {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid language parameter".to_string());
            return ptr::null_mut();
        }
    };

    let path = match ptr_to_str(source_path) {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid source_path parameter".to_string());
            return ptr::null_mut();
        }
    };

    let mut parser = match TreeSitterParser::new(lang) {
        Ok(p) => p,
        Err(e) => {
            set_last_error(format!("Parser creation failed: {}", e));
            return ptr::null_mut();
        }
    };

    match parser.parse(Path::new(path)) {
        Ok(result) => match serde_json::to_string(&result) {
            Ok(json) => string_to_c(json),
            Err(e) => {
                set_last_error(format!("JSON serialization failed: {}", e));
                ptr::null_mut()
            }
        },
        Err(e) => {
            set_last_error(format!("Parse failed: {}", e));
            ptr::null_mut()
        }
    }
}

// =====================================================================
// FFI export (2): Syntax check -> JSON diagnostics
// =====================================================================

/// Check a source file and return JSON-formatted diagnostics.
///
/// # Parameters
/// - `language`: Language name ("rust", "c")
/// - `source_path`: Source file path (UTF-8)
///
/// # Returns
/// - JSON string pointer (on success); must be freed with `forge_free_string`
/// - null (on failure)
///
/// # Safety
/// - `language` / `source_path` may be null; if non-null, they must point to valid NUL-terminated C strings.
/// - The memory pointed to must remain readable and valid for the duration of this function call.
#[no_mangle]
pub unsafe extern "C" fn forge_check_syntax(
    language: *const c_char,
    source_path: *const c_char,
) -> *mut c_char {
    let lang = match ptr_to_str(language) {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid language parameter".to_string());
            return ptr::null_mut();
        }
    };

    let path_str = match ptr_to_str(source_path) {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid source_path parameter".to_string());
            return ptr::null_mut();
        }
    };

    let path = Path::new(path_str);

    // Parse first
    let mut parser = match TreeSitterParser::new(lang) {
        Ok(p) => p,
        Err(e) => {
            set_last_error(format!("Parser creation failed: {}", e));
            return ptr::null_mut();
        }
    };

    let parse_result = match parser.parse(path) {
        Ok(r) => r,
        Err(e) => {
            set_last_error(format!("Parse failed: {}", e));
            return ptr::null_mut();
        }
    };

    // Then check
    let check_result: Result<CheckResult, _> = match lang {
        "rust" => RustChecker::new().check(&parse_result, path),
        "c" => CChecker::new().check(&parse_result, path),
        "cpp" => {
            // C++ uses the C checker (basic functionality is the same)
            CChecker::new().check(&parse_result, path)
        }
        _ => {
            set_last_error(format!("No checker for language: {}", lang));
            return ptr::null_mut();
        }
    };

    match check_result {
        Ok(result) => match serde_json::to_string(&result) {
            Ok(json) => string_to_c(json),
            Err(e) => {
                set_last_error(format!("JSON serialization failed: {}", e));
                ptr::null_mut()
            }
        },
        Err(e) => {
            set_last_error(format!("Check failed: {}", e));
            ptr::null_mut()
        }
    }
}

// =====================================================================
// FFI export (3): Get supported language list
// =====================================================================

/// Return a JSON array of supported parsing languages.
///
/// # Returns
/// - JSON string `["rust","c","cpp"]`; must be freed with `forge_free_string`
#[no_mangle]
pub extern "C" fn forge_supported_languages() -> *mut c_char {
    let languages = vec!["rust", "c", "cpp"];
    match serde_json::to_string(&languages) {
        Ok(json) => string_to_c(json),
        Err(e) => {
            set_last_error(format!("Serialization failed: {}", e));
            ptr::null_mut()
        }
    }
}

// =====================================================================
// FFI export (4): Get version information
// =====================================================================

/// Return the Forge Core version string.
///
/// # Returns
/// - Version string pointer; must be freed with `forge_free_string`
#[no_mangle]
pub extern "C" fn forge_version() -> *mut c_char {
    let version_info = serde_json::json!({
        "version": crate::VERSION,
        "codename": crate::CODENAME,
        "tree_sitter": true,
        "languages": ["rust", "c", "cpp"]
    });

    match serde_json::to_string(&version_info) {
        Ok(json) => string_to_c(json),
        Err(e) => {
            set_last_error(format!("Serialization failed: {}", e));
            ptr::null_mut()
        }
    }
}

// =====================================================================
// FFI export (5): Batch-parse multiple files
// =====================================================================

/// Batch-parse multiple source files.
///
/// # Parameters
/// - `language`: Language name
/// - `paths_json`: JSON array of file paths `["file1.rs","file2.rs"]`
///
/// # Returns
/// - JSON string `[ParseResult, ...]`; must be freed with `forge_free_string`
/// - null (on failure)
///
/// # Safety
/// - `language` / `paths_json` may be null; if non-null, they must point to valid NUL-terminated C strings.
/// - The memory pointed to must remain readable and valid for the duration of this function call.
#[no_mangle]
pub unsafe extern "C" fn forge_parse_batch(
    language: *const c_char,
    paths_json: *const c_char,
) -> *mut c_char {
    let lang = match ptr_to_str(language) {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid language parameter".to_string());
            return ptr::null_mut();
        }
    };

    let json_str = match ptr_to_str(paths_json) {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid paths_json parameter".to_string());
            return ptr::null_mut();
        }
    };

    let paths: Vec<String> = match serde_json::from_str(json_str) {
        Ok(p) => p,
        Err(e) => {
            set_last_error(format!("Invalid JSON paths: {}", e));
            return ptr::null_mut();
        }
    };

    let mut parser = match TreeSitterParser::new(lang) {
        Ok(p) => p,
        Err(e) => {
            set_last_error(format!("Parser creation failed: {}", e));
            return ptr::null_mut();
        }
    };

    let mut results: Vec<ParseResult> = Vec::with_capacity(paths.len());
    for path in &paths {
        match parser.parse(Path::new(path)) {
            Ok(result) => results.push(result),
            Err(e) => {
                results.push(ParseResult {
                    success: false,
                    error: Some(format!("Parse failed for {}: {}", path, e)),
                    tree: None,
                    duration_ms: 0,
                });
            }
        }
    }

    match serde_json::to_string(&results) {
        Ok(json) => string_to_c(json),
        Err(e) => {
            set_last_error(format!("JSON serialization failed: {}", e));
            ptr::null_mut()
        }
    }
}

// =====================================================================
// FFI export (6): Incremental parse (with previous tree hash)
// =====================================================================

/// Incrementally parse a source file (compares file_hash to decide whether to skip).
///
/// # Parameters
/// - `language`: Language name
/// - `source_path`: Source file path
/// - `prev_hash`: Previous file_hash (may be null to indicate first parse)
///
/// # Returns
/// - JSON string ParseResult; must be freed with `forge_free_string`
///
/// # Safety
/// - `language` / `source_path` may be null; if non-null, they must point to valid NUL-terminated C strings.
/// - `prev_hash` may be null; if non-null, it must point to a valid NUL-terminated C string.
/// - The memory pointed to must remain readable and valid for the duration of this function call.
#[no_mangle]
pub unsafe extern "C" fn forge_parse_incremental(
    language: *const c_char,
    source_path: *const c_char,
    prev_hash: *const c_char,
) -> *mut c_char {
    let lang = match ptr_to_str(language) {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid language parameter".to_string());
            return ptr::null_mut();
        }
    };

    let path_str = match ptr_to_str(source_path) {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid source_path parameter".to_string());
            return ptr::null_mut();
        }
    };

    let path = Path::new(path_str);

    let mut parser = match TreeSitterParser::new(lang) {
        Ok(p) => p,
        Err(e) => {
            set_last_error(format!("Parser creation failed: {}", e));
            return ptr::null_mut();
        }
    };

    // Try reading the file to compute its hash; skip if it matches the previous hash
    if !prev_hash.is_null() {
        if let Ok(old_hash) = ptr_to_str(prev_hash) {
            if let Ok(source_code) = std::fs::read_to_string(path) {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                source_code.hash(&mut hasher);
                let current_hash = format!("{:x}", hasher.finish());

                if current_hash == old_hash {
                    // Unchanged; return an empty success result
                    let result = ParseResult {
                        success: true,
                        error: None,
                        tree: None,
                        duration_ms: 0,
                    };
                    return match serde_json::to_string(&result) {
                        Ok(json) => string_to_c(json),
                        Err(_) => ptr::null_mut(),
                    };
                }
            }
        }
    }

    // Need to re-parse
    match parser.parse(path) {
        Ok(result) => match serde_json::to_string(&result) {
            Ok(json) => string_to_c(json),
            Err(e) => {
                set_last_error(format!("JSON serialization failed: {}", e));
                ptr::null_mut()
            }
        },
        Err(e) => {
            set_last_error(format!("Parse failed: {}", e));
            ptr::null_mut()
        }
    }
}

// =====================================================================
// Memory management FFI (4 functions)
// =====================================================================

/// Get the last error message.
///
/// # Returns
/// - Error string pointer; must be freed with `forge_free_string`
/// - null if no error
#[no_mangle]
pub extern "C" fn forge_last_error() -> *mut c_char {
    if let Ok(guard) = LAST_ERROR.lock() {
        if let Some(ref msg) = *guard {
            return string_to_c(msg.clone());
        }
    }
    ptr::null_mut()
}

/// Free a string returned by FFI.
///
/// # Safety
/// - `s` must be a pointer returned by a forge_* function, or null
#[no_mangle]
pub unsafe extern "C" fn forge_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Initialize Forge Core (sets up logging, etc.).
///
/// # Returns
/// - 0 on success, -1 on failure
#[no_mangle]
pub extern "C" fn forge_init() -> i32 {
    // Avoid duplicate tracing initialization
    // Use try_init for safe handling
    let _ = tracing_subscriber::fmt::try_init();
    tracing::info!("[MAIDOS-AUDIT] Forge Core FFI initialized v{}", crate::VERSION);
    0
}

/// Clear the last error.
#[no_mangle]
pub extern "C" fn forge_clear_error() {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = None;
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_ffi_parse_rust_source() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn main() {{ println!(\"Hello\"); }}").unwrap();

        unsafe {
            let lang = CString::new("rust").unwrap();
            let path = CString::new(temp_file.path().to_str().unwrap()).unwrap();

            let result = forge_parse_source(lang.as_ptr(), path.as_ptr());
            assert!(!result.is_null(), "Parse should succeed");

            let json = CStr::from_ptr(result).to_str().unwrap();
            assert!(json.contains("\"success\":true"));

            forge_free_string(result);
        }
    }

    #[test]
    fn test_ffi_parse_c_source() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "int main() {{ return 0; }}").unwrap();

        unsafe {
            let lang = CString::new("c").unwrap();
            let path = CString::new(temp_file.path().to_str().unwrap()).unwrap();

            let result = forge_parse_source(lang.as_ptr(), path.as_ptr());
            assert!(!result.is_null());

            forge_free_string(result);
        }
    }

    #[test]
    fn test_ffi_check_syntax() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "#include <stdio.h>\nint main() {{ gets(buf); return 0; }}").unwrap();

        unsafe {
            let lang = CString::new("c").unwrap();
            let path = CString::new(temp_file.path().to_str().unwrap()).unwrap();

            let result = forge_check_syntax(lang.as_ptr(), path.as_ptr());
            assert!(!result.is_null());

            let json = CStr::from_ptr(result).to_str().unwrap();
            assert!(json.contains("\"success\""));

            forge_free_string(result);
        }
    }

    #[test]
    fn test_ffi_unsupported_language() {
        let temp_file = NamedTempFile::new().unwrap();

        unsafe {
            let lang = CString::new("brainfuck").unwrap();
            let path = CString::new(temp_file.path().to_str().unwrap()).unwrap();

            let result = forge_parse_source(lang.as_ptr(), path.as_ptr());
            assert!(result.is_null());

            let err = forge_last_error();
            assert!(!err.is_null());
            forge_free_string(err);
        }
    }

    #[test]
    fn test_ffi_null_safety() {
        unsafe {
            let result = forge_parse_source(ptr::null(), ptr::null());
            assert!(result.is_null());

            forge_free_string(ptr::null_mut());

            forge_clear_error();
            let err = forge_last_error();
            assert!(err.is_null());
        }
    }

    #[test]
    fn test_ffi_supported_languages() {
        let result = forge_supported_languages();
        assert!(!result.is_null());

        unsafe {
            let json = CStr::from_ptr(result).to_str().unwrap();
            assert!(json.contains("rust"));
            assert!(json.contains("c"));
            assert!(json.contains("cpp"));
            forge_free_string(result);
        }
    }

    #[test]
    fn test_ffi_version() {
        let result = forge_version();
        assert!(!result.is_null());

        unsafe {
            let json = CStr::from_ptr(result).to_str().unwrap();
            assert!(json.contains("2.1.0"));
            assert!(json.contains("Forge"));
            forge_free_string(result);
        }
    }

    #[test]
    fn test_ffi_init() {
        let code = forge_init();
        assert_eq!(code, 0);
    }

    #[test]
    fn test_ffi_incremental_parse_skip() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn main() {{}}").unwrap();

        unsafe {
            let lang = CString::new("rust").unwrap();
            let path = CString::new(temp_file.path().to_str().unwrap()).unwrap();

            // First parse, obtain hash
            let result1 = forge_parse_source(lang.as_ptr(), path.as_ptr());
            assert!(!result1.is_null());
            let json1 = CStr::from_ptr(result1).to_str().unwrap().to_string();

            // Extract file_hash from JSON
            let parsed: serde_json::Value = serde_json::from_str(&json1).unwrap();
            let hash = parsed["tree"]["file_hash"].as_str().unwrap();
            let hash_c = CString::new(hash).unwrap();

            forge_free_string(result1);

            // Incremental parse (same hash -> skip)
            let result2 = forge_parse_incremental(
                lang.as_ptr(),
                path.as_ptr(),
                hash_c.as_ptr(),
            );
            assert!(!result2.is_null());
            let json2 = CStr::from_ptr(result2).to_str().unwrap();
            let parsed2: serde_json::Value = serde_json::from_str(json2).unwrap();
            assert!(parsed2["success"].as_bool().unwrap());
            assert!(parsed2["tree"].is_null()); // tree is null indicating skip
            assert_eq!(parsed2["duration_ms"].as_u64().unwrap(), 0);

            forge_free_string(result2);
        }
    }
}
