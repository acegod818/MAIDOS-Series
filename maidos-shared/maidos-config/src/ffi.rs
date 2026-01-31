//! C FFI exports for maidos-config
//!
//! <impl>
//! WHAT: C-compatible API for cross-language binding (P/Invoke, etc.)
//! WHY: Allow C#/other languages to use maidos-config via native interop
//! HOW: extern "C" functions with raw pointers, JSON for complex types
//! TEST: FFI functions tested via integration tests
//! </impl>

#![allow(dead_code)]  // FFI exports are used externally

use crate::error::ConfigError;
use crate::loader::load_config;
use crate::schema::MaidosConfigSchema;
use std::ffi::{c_char, CStr, CString};
use std::path::Path;
use std::ptr;
use std::sync::Mutex;

/// Opaque handle to configuration
pub struct ConfigHandle {
    schema: MaidosConfigSchema,
    path: Option<std::path::PathBuf>,
}

/// Error codes for FFI
#[repr(C)]
pub enum ConfigErrorCode {
    Ok = 0,
    FileNotFound = 1,
    ReadError = 2,
    ParseError = 3,
    MissingKey = 4,
    TypeMismatch = 5,
    EnvVarNotSet = 6,
    ValidationError = 7,
    WatcherError = 8,
    NullPointer = 9,
    InvalidUtf8 = 10,
}

impl From<&ConfigError> for ConfigErrorCode {
    fn from(err: &ConfigError) -> Self {
        match err {
            ConfigError::FileNotFound(_) => ConfigErrorCode::FileNotFound,
            ConfigError::ReadError(_) => ConfigErrorCode::ReadError,
            ConfigError::ParseError(_) => ConfigErrorCode::ParseError,
            ConfigError::MissingKey(_) => ConfigErrorCode::MissingKey,
            ConfigError::TypeMismatch { .. } => ConfigErrorCode::TypeMismatch,
            ConfigError::EnvVarNotSet(_) => ConfigErrorCode::EnvVarNotSet,
            ConfigError::ValidationError(_) => ConfigErrorCode::ValidationError,
            ConfigError::WatcherError(_) => ConfigErrorCode::WatcherError,
        }
    }
}

// Thread-safe storage for last error message
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn set_last_error(msg: String) {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = Some(msg);
    }
}

/// Load configuration from file
///
/// # Safety
/// - `path` must be a valid null-terminated UTF-8 string
/// - Returns null on error, check `maidos_config_last_error` for details
#[no_mangle]
pub unsafe extern "C" fn maidos_config_load(path: *const c_char) -> *mut ConfigHandle {
    if path.is_null() {
        set_last_error("Path is null".to_string());
        return ptr::null_mut();
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid UTF-8 in path".to_string());
            return ptr::null_mut();
        }
    };

    match load_config(Path::new(path_str)) {
        Ok(schema) => {
            let handle = Box::new(ConfigHandle {
                schema,
                path: Some(std::path::PathBuf::from(path_str)),
            });
            Box::into_raw(handle)
        }
        Err(e) => {
            set_last_error(e.to_string());
            ptr::null_mut()
        }
    }
}

/// Free configuration handle
///
/// # Safety
/// - `handle` must be a valid pointer from `maidos_config_load` or null
#[no_mangle]
pub unsafe extern "C" fn maidos_config_free(handle: *mut ConfigHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Get last error message
///
/// # Safety
/// - Returns a pointer to internal string, valid until next error
/// - Caller must NOT free this string
#[no_mangle]
pub extern "C" fn maidos_config_last_error() -> *const c_char {
    static mut ERROR_CSTRING: Option<CString> = None;

    if let Ok(guard) = LAST_ERROR.lock() {
        if let Some(ref msg) = *guard {
            unsafe {
                ERROR_CSTRING = CString::new(msg.as_str()).ok();
                if let Some(ref cs) = ERROR_CSTRING {
                    return cs.as_ptr();
                }
            }
        }
    }
    ptr::null()
}

/// Get configuration as JSON string
///
/// # Safety
/// - `handle` must be a valid pointer from `maidos_config_load`
/// - Caller must free returned string with `maidos_config_free_string`
#[no_mangle]
pub unsafe extern "C" fn maidos_config_to_json(handle: *const ConfigHandle) -> *mut c_char {
    if handle.is_null() {
        set_last_error("Handle is null".to_string());
        return ptr::null_mut();
    }

    let handle = &*handle;
    match serde_json::to_string(&handle.schema) {
        Ok(json) => match CString::new(json) {
            Ok(cs) => cs.into_raw(),
            Err(_) => {
                set_last_error("JSON contains null byte".to_string());
                ptr::null_mut()
            }
        },
        Err(e) => {
            set_last_error(format!("JSON serialization failed: {}", e));
            ptr::null_mut()
        }
    }
}

/// Get a string value from config by dot-separated key
///
/// # Safety
/// - `handle` must be a valid pointer
/// - `key` must be a valid null-terminated UTF-8 string
/// - Caller must free returned string with `maidos_config_free_string`
#[no_mangle]
pub unsafe extern "C" fn maidos_config_get_string(
    handle: *const ConfigHandle,
    key: *const c_char,
) -> *mut c_char {
    if handle.is_null() || key.is_null() {
        set_last_error("Null pointer".to_string());
        return ptr::null_mut();
    }

    let handle = &*handle;
    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid UTF-8 in key".to_string());
            return ptr::null_mut();
        }
    };

    // Simple dot-notation lookup for common keys
    let value: Option<String> = match key_str {
        "maidos.version" => Some(handle.schema.maidos.version.clone()),
        "llm.default_provider" => Some(handle.schema.llm.default_provider.clone()),
        "bus.endpoint" => Some(handle.schema.bus.endpoint.clone()),
        _ => {
            // For provider-specific keys like "llm.providers.anthropic.model"
            if key_str.starts_with("llm.providers.") {
                let parts: Vec<&str> = key_str.split('.').collect();
                if parts.len() >= 4 {
                    let provider_name = parts[2];
                    let field = parts[3];
                    if let Some(provider) = handle.schema.llm.providers.get(provider_name) {
                        match field {
                            "api_key" => provider.api_key.clone(),
                            "endpoint" => provider.endpoint.clone(),
                            "model" => provider.model.clone(),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    };

    match value {
        Some(v) => match CString::new(v) {
            Ok(cs) => cs.into_raw(),
            Err(_) => {
                set_last_error("Value contains null byte".to_string());
                ptr::null_mut()
            }
        },
        None => {
            set_last_error(format!("Key not found: {}", key_str));
            ptr::null_mut()
        }
    }
}

/// Get a float value from config by dot-separated key
///
/// # Safety
/// - `handle` must be a valid pointer
/// - `key` must be a valid null-terminated UTF-8 string
/// - Returns 0.0 on error, check `maidos_config_last_error`
#[no_mangle]
pub unsafe extern "C" fn maidos_config_get_f64(
    handle: *const ConfigHandle,
    key: *const c_char,
) -> f64 {
    if handle.is_null() || key.is_null() {
        set_last_error("Null pointer".to_string());
        return 0.0;
    }

    let handle = &*handle;
    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid UTF-8 in key".to_string());
            return 0.0;
        }
    };

    match key_str {
        "llm.budget_daily" => handle.schema.llm.budget_daily,
        "llm.budget_monthly" => handle.schema.llm.budget_monthly,
        _ => {
            set_last_error(format!("Key not found or not a float: {}", key_str));
            0.0
        }
    }
}

/// Get an integer value from config by dot-separated key
///
/// # Safety
/// - `handle` must be a valid pointer
/// - `key` must be a valid null-terminated UTF-8 string
/// - Returns 0 on error, check `maidos_config_last_error`
#[no_mangle]
pub unsafe extern "C" fn maidos_config_get_u64(
    handle: *const ConfigHandle,
    key: *const c_char,
) -> u64 {
    if handle.is_null() || key.is_null() {
        set_last_error("Null pointer".to_string());
        return 0;
    }

    let handle = &*handle;
    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid UTF-8 in key".to_string());
            return 0;
        }
    };

    match key_str {
        "auth.token_ttl" => handle.schema.auth.token_ttl,
        "bus.buffer_size" => handle.schema.bus.buffer_size as u64,
        "bus.reconnect_ms" => handle.schema.bus.reconnect_ms,
        _ => {
            // Check provider timeout
            if key_str.starts_with("llm.providers.") && key_str.ends_with(".timeout_secs") {
                let parts: Vec<&str> = key_str.split('.').collect();
                if parts.len() >= 4 {
                    let provider_name = parts[2];
                    if let Some(provider) = handle.schema.llm.providers.get(provider_name) {
                        return provider.timeout_secs;
                    }
                }
            }
            set_last_error(format!("Key not found or not an integer: {}", key_str));
            0
        }
    }
}

/// Free a string returned by maidos_config functions
///
/// # Safety
/// - `s` must be a pointer from `maidos_config_get_string` or `maidos_config_to_json`, or null
#[no_mangle]
pub unsafe extern "C" fn maidos_config_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_ffi_load_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        fs::write(
            &config_path,
            r#"
[maidos]
version = "1.0"

[llm]
default_provider = "test_provider"
budget_daily = 25.5

[bus]
endpoint = "tcp://127.0.0.1:5555"

[auth]
token_ttl = 3600
"#,
        )
        .unwrap();

        unsafe {
            let path_cstr = CString::new(config_path.to_str().unwrap()).unwrap();
            let handle = maidos_config_load(path_cstr.as_ptr());
            assert!(!handle.is_null());

            // Test string getter
            let key = CString::new("llm.default_provider").unwrap();
            let value = maidos_config_get_string(handle, key.as_ptr());
            assert!(!value.is_null());
            let value_str = CStr::from_ptr(value).to_str().unwrap();
            assert_eq!(value_str, "test_provider");
            maidos_config_free_string(value);

            // Test float getter
            let key = CString::new("llm.budget_daily").unwrap();
            let value = maidos_config_get_f64(handle, key.as_ptr());
            assert!((value - 25.5).abs() < 0.001);

            // Test integer getter
            let key = CString::new("auth.token_ttl").unwrap();
            let value = maidos_config_get_u64(handle, key.as_ptr());
            assert_eq!(value, 3600);

            // Test JSON export
            let json = maidos_config_to_json(handle);
            assert!(!json.is_null());
            let json_str = CStr::from_ptr(json).to_str().unwrap();
            assert!(json_str.contains("test_provider"));
            maidos_config_free_string(json);

            maidos_config_free(handle);
        }
    }

    #[test]
    fn test_ffi_null_safety() {
        unsafe {
            // Null path
            let handle = maidos_config_load(ptr::null());
            assert!(handle.is_null());

            // Null handle
            let key = CString::new("test").unwrap();
            let value = maidos_config_get_string(ptr::null(), key.as_ptr());
            assert!(value.is_null());

            // Free null is safe
            maidos_config_free(ptr::null_mut());
            maidos_config_free_string(ptr::null_mut());
        }
    }

    #[test]
    fn test_ffi_provider_keys_and_errors() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        fs::write(
            &config_path,
            r#"
[maidos]
version = "1.0"

[llm]
default_provider = "openai"
budget_daily = 5.0

[llm.providers.openai]
api_key = "key"
model = "gpt-4o"
timeout_secs = 15
"#,
        )
        .unwrap();

        unsafe {
            let path_cstr = CString::new(config_path.to_str().unwrap()).unwrap();
            let handle = maidos_config_load(path_cstr.as_ptr());
            assert!(!handle.is_null());

            let key = CString::new("llm.providers.openai.model").unwrap();
            let value = maidos_config_get_string(handle, key.as_ptr());
            assert!(!value.is_null());
            maidos_config_free_string(value);

            let key = CString::new("llm.providers.openai.timeout_secs").unwrap();
            let timeout = maidos_config_get_u64(handle, key.as_ptr());
            assert_eq!(timeout, 15);

            let missing = CString::new("llm.providers.openai.unknown").unwrap();
            let value = maidos_config_get_string(handle, missing.as_ptr());
            assert!(value.is_null());
            let err = maidos_config_last_error();
            assert!(!err.is_null());

            maidos_config_free(handle);
        }
    }

    #[test]
    fn test_ffi_invalid_utf8_inputs() {
        unsafe {
            let invalid = [0xffu8, 0u8];
            let handle = maidos_config_load(invalid.as_ptr() as *const c_char);
            assert!(handle.is_null());

            let key = invalid.as_ptr() as *const c_char;
            let value = maidos_config_get_string(ptr::null(), key);
            assert!(value.is_null());
        }
    }

    #[test]
    fn test_ffi_get_f64_missing_key() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        fs::write(
            &config_path,
            r#"
[maidos]
version = "1.0"

[llm]
default_provider = "openai"
budget_daily = 5.0
"#,
        )
        .unwrap();

        unsafe {
            let path_cstr = CString::new(config_path.to_str().unwrap()).unwrap();
            let handle = maidos_config_load(path_cstr.as_ptr());
            assert!(!handle.is_null());

            let key = CString::new("llm.unknown").unwrap();
            let value = maidos_config_get_f64(handle, key.as_ptr());
            assert_eq!(value, 0.0);
            let err = maidos_config_last_error();
            assert!(!err.is_null());

            maidos_config_free(handle);
        }
    }

    #[test]
    fn test_ffi_basic_string_and_u64_keys() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        fs::write(
            &config_path,
            r#"
[maidos]
version = "1.2"

[llm]
default_provider = "openai"

[llm.providers.openai]
api_key = "key"
endpoint = "http://localhost:1234"
timeout_secs = 30

[bus]
endpoint = "tcp://127.0.0.1:9999"
buffer_size = 512
reconnect_ms = 1500
"#,
        )
        .unwrap();

        unsafe {
            let path_cstr = CString::new(config_path.to_str().unwrap()).unwrap();
            let handle = maidos_config_load(path_cstr.as_ptr());
            assert!(!handle.is_null());

            let key = CString::new("maidos.version").unwrap();
            let value = maidos_config_get_string(handle, key.as_ptr());
            assert!(!value.is_null());
            maidos_config_free_string(value);

            let key = CString::new("bus.endpoint").unwrap();
            let value = maidos_config_get_string(handle, key.as_ptr());
            assert!(!value.is_null());
            maidos_config_free_string(value);

            let key = CString::new("llm.providers.openai.api_key").unwrap();
            let value = maidos_config_get_string(handle, key.as_ptr());
            assert!(!value.is_null());
            maidos_config_free_string(value);

            let key = CString::new("llm.providers.openai.endpoint").unwrap();
            let value = maidos_config_get_string(handle, key.as_ptr());
            assert!(!value.is_null());
            maidos_config_free_string(value);

            let key = CString::new("bus.buffer_size").unwrap();
            let value = maidos_config_get_u64(handle, key.as_ptr());
            assert_eq!(value, 512);

            let key = CString::new("bus.reconnect_ms").unwrap();
            let value = maidos_config_get_u64(handle, key.as_ptr());
            assert_eq!(value, 1500);

            maidos_config_free(handle);
        }
    }

    #[test]
    fn test_ffi_to_json_null_handle() {
        unsafe {
            let json = maidos_config_to_json(ptr::null());
            assert!(json.is_null());
            let err = maidos_config_last_error();
            assert!(!err.is_null());
        }
    }

    #[test]
    fn test_ffi_get_string_invalid_utf8_key() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        fs::write(
            &config_path,
            r#"
[maidos]
version = "1.0"

[llm]
default_provider = "openai"
"#,
        )
        .unwrap();

        unsafe {
            let path_cstr = CString::new(config_path.to_str().unwrap()).unwrap();
            let handle = maidos_config_load(path_cstr.as_ptr());
            assert!(!handle.is_null());

            let invalid = [0xffu8, 0u8];
            let value = maidos_config_get_string(handle, invalid.as_ptr() as *const c_char);
            assert!(value.is_null());

            maidos_config_free(handle);
        }
    }

    #[test]
    fn test_ffi_get_u64_missing_key() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        fs::write(
            &config_path,
            r#"
[maidos]
version = "1.0"
"#,
        )
        .unwrap();

        unsafe {
            let path_cstr = CString::new(config_path.to_str().unwrap()).unwrap();
            let handle = maidos_config_load(path_cstr.as_ptr());
            assert!(!handle.is_null());

            let key = CString::new("llm.providers.openai.timeout_secs").unwrap();
            let value = maidos_config_get_u64(handle, key.as_ptr());
            assert_eq!(value, 0);

            maidos_config_free(handle);
        }
    }

    #[test]
    fn test_ffi_get_f64_invalid_utf8_key() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        fs::write(
            &config_path,
            r#"
[maidos]
version = "1.0"

[llm]
default_provider = "openai"
budget_daily = 5.0
"#,
        )
        .unwrap();

        unsafe {
            let path_cstr = CString::new(config_path.to_str().unwrap()).unwrap();
            let handle = maidos_config_load(path_cstr.as_ptr());
            assert!(!handle.is_null());

            let invalid = [0xffu8, 0u8];
            let value = maidos_config_get_f64(handle, invalid.as_ptr() as *const c_char);
            assert_eq!(value, 0.0);

            maidos_config_free(handle);
        }
    }

    #[test]
    fn test_ffi_get_u64_invalid_utf8_key() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        fs::write(
            &config_path,
            r#"
[maidos]
version = "1.0"
"#,
        )
        .unwrap();

        unsafe {
            let path_cstr = CString::new(config_path.to_str().unwrap()).unwrap();
            let handle = maidos_config_load(path_cstr.as_ptr());
            assert!(!handle.is_null());

            let invalid = [0xffu8, 0u8];
            let value = maidos_config_get_u64(handle, invalid.as_ptr() as *const c_char);
            assert_eq!(value, 0);

            maidos_config_free(handle);
        }
    }
}
