//! C FFI exports for maidos-auth
//!
//! <impl>
//! WHAT: C-compatible API for cross-language binding
//! WHY: Allow C#/other languages to use maidos-auth via native interop
//! HOW: extern "C" functions with raw pointers, error codes
//! TEST: FFI functions tested via integration tests
//! </impl>

#![allow(dead_code)]

use crate::capability::{Capability, CapabilitySet};
use crate::error::AuthError;
use crate::token::CapabilityToken;
use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::sync::Mutex;
use std::time::Duration;

/// Error codes for FFI
#[repr(C)]
pub enum AuthErrorCode {
    Ok = 0,
    TokenExpired = 1,
    InvalidSignature = 2,
    MalformedToken = 3,
    MissingCapability = 4,
    NoSecretKey = 5,
    NullPointer = 6,
    InvalidUtf8 = 7,
    InvalidToken = 8,
    Internal = 9,
}

impl From<&AuthError> for AuthErrorCode {
    fn from(err: &AuthError) -> Self {
        match err {
            AuthError::TokenExpired => AuthErrorCode::TokenExpired,
            AuthError::InvalidSignature => AuthErrorCode::InvalidSignature,
            AuthError::MalformedToken(_) => AuthErrorCode::MalformedToken,
            AuthError::InvalidToken(_) => AuthErrorCode::InvalidToken,
            AuthError::MissingCapability(_) => AuthErrorCode::MissingCapability,
            AuthError::NoSecretKey => AuthErrorCode::NoSecretKey,
            AuthError::ConfigError(_) => AuthErrorCode::MalformedToken,
            AuthError::SerializationError(_) => AuthErrorCode::MalformedToken,
            AuthError::Internal(_) => AuthErrorCode::Internal,
        }
    }
}

static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn set_last_error(msg: String) {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = Some(msg);
    }
}

/// Get last error message
#[no_mangle]
pub extern "C" fn maidos_auth_last_error() -> *const c_char {
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

/// Create a new token
///
/// # Safety
/// - `secret` must be a valid pointer to `secret_len` bytes
/// - Caller must free returned string with `maidos_auth_free_string`
#[no_mangle]
pub unsafe extern "C" fn maidos_auth_create_token(
    capabilities: u32,
    ttl_secs: u64,
    secret: *const u8,
    secret_len: usize,
) -> *mut c_char {
    if secret.is_null() {
        set_last_error("Secret is null".to_string());
        return ptr::null_mut();
    }

    let secret_slice = std::slice::from_raw_parts(secret, secret_len);
    let caps = CapabilitySet::from_u32(capabilities);

    match CapabilityToken::new(caps, Duration::from_secs(ttl_secs), secret_slice) {
        Ok(token) => match CString::new(token.as_str()) {
            Ok(cs) => cs.into_raw(),
            Err(_) => {
                set_last_error("Token contains null byte".to_string());
                ptr::null_mut()
            }
        },
        Err(e) => {
            set_last_error(e.to_string());
            ptr::null_mut()
        }
    }
}

/// Verify a token and return capabilities bitmask
///
/// # Safety
/// - `token` must be a valid null-terminated UTF-8 string
/// - `secret` must be a valid pointer to `secret_len` bytes
/// - Returns 0 on error, check `maidos_auth_last_error`
#[no_mangle]
pub unsafe extern "C" fn maidos_auth_verify_token(
    token: *const c_char,
    secret: *const u8,
    secret_len: usize,
) -> u32 {
    if token.is_null() || secret.is_null() {
        set_last_error("Null pointer".to_string());
        return 0;
    }

    let token_str = match CStr::from_ptr(token).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid UTF-8 in token".to_string());
            return 0;
        }
    };

    let secret_slice = std::slice::from_raw_parts(secret, secret_len);

    match CapabilityToken::verify(token_str, secret_slice) {
        Ok(verified) => verified.capabilities().as_u32(),
        Err(e) => {
            set_last_error(e.to_string());
            0
        }
    }
}

/// Check if token has a specific capability
///
/// # Safety
/// - `token` must be a valid null-terminated UTF-8 string
/// - `secret` must be a valid pointer to `secret_len` bytes
#[no_mangle]
pub unsafe extern "C" fn maidos_auth_token_has_capability(
    token: *const c_char,
    secret: *const u8,
    secret_len: usize,
    capability: u32,
) -> bool {
    if token.is_null() || secret.is_null() {
        return false;
    }

    let token_str = match CStr::from_ptr(token).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let secret_slice = std::slice::from_raw_parts(secret, secret_len);

    match CapabilityToken::verify(token_str, secret_slice) {
        Ok(verified) => {
            let caps = verified.capabilities();
            (caps.as_u32() & capability) != 0
        }
        Err(_) => false,
    }
}

/// Get capability value by name
///
/// # Safety
/// - `name` must be a valid null-terminated UTF-8 string
/// - Returns 0 if not found
#[no_mangle]
pub unsafe extern "C" fn maidos_auth_capability_from_name(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }

    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    Capability::from_name(name_str).map(|c| c as u32).unwrap_or(0)
}

/// Free a string returned by maidos_auth functions
#[no_mangle]
pub unsafe extern "C" fn maidos_auth_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_ffi_create_and_verify() {
        let secret = b"test-secret-32-bytes-long!!!!!!";
        let caps = (Capability::LlmChat as u32) | (Capability::FileRead as u32);

        unsafe {
            let token = maidos_auth_create_token(caps, 3600, secret.as_ptr(), secret.len());
            assert!(!token.is_null());

            let verified_caps = maidos_auth_verify_token(token, secret.as_ptr(), secret.len());
            assert_eq!(verified_caps, caps);

            maidos_auth_free_string(token);
        }
    }

    #[test]
    fn test_ffi_capability_from_name() {
        unsafe {
            let name = CString::new("llm.chat").unwrap();
            let cap = maidos_auth_capability_from_name(name.as_ptr());
            assert_eq!(cap, Capability::LlmChat as u32);

            let invalid = CString::new("invalid").unwrap();
            let cap = maidos_auth_capability_from_name(invalid.as_ptr());
            assert_eq!(cap, 0);
        }
    }

    #[test]
    fn test_ffi_token_has_capability() {
        let secret = b"test-secret-32-bytes-long!!!!!!";
        let caps = (Capability::LlmChat as u32) | (Capability::FileRead as u32);

        unsafe {
            let token = maidos_auth_create_token(caps, 3600, secret.as_ptr(), secret.len());
            assert!(!token.is_null());

            let has = maidos_auth_token_has_capability(
                token,
                secret.as_ptr(),
                secret.len(),
                Capability::FileRead as u32,
            );
            assert!(has);

            let missing = maidos_auth_token_has_capability(
                token,
                secret.as_ptr(),
                secret.len(),
                Capability::ShellExec as u32,
            );
            assert!(!missing);

            maidos_auth_free_string(token);
        }
    }

    #[test]
    fn test_ffi_verify_invalid_token() {
        let secret = b"test-secret-32-bytes-long!!!!!!";
        unsafe {
            let bad = CString::new("invalid.token").unwrap();
            let caps = maidos_auth_verify_token(bad.as_ptr(), secret.as_ptr(), secret.len());
            assert_eq!(caps, 0);

            let err = maidos_auth_last_error();
            assert!(!err.is_null());
        }
    }

    #[test]
    fn test_ffi_null_safety() {
        unsafe {
            let caps = maidos_auth_verify_token(ptr::null(), ptr::null(), 0);
            assert_eq!(caps, 0);
            let cap = maidos_auth_capability_from_name(ptr::null());
            assert_eq!(cap, 0);
        }
    }

    #[test]
    fn test_ffi_invalid_utf8() {
        unsafe {
            let invalid = [0xffu8, 0u8];
            let caps = maidos_auth_verify_token(
                invalid.as_ptr() as *const c_char,
                b"secret".as_ptr(),
                6,
            );
            assert_eq!(caps, 0);

            let cap = maidos_auth_capability_from_name(invalid.as_ptr() as *const c_char);
            assert_eq!(cap, 0);
        }
    }

    #[test]
    fn test_ffi_create_token_null_secret() {
        unsafe {
            let token = maidos_auth_create_token(0, 1, ptr::null(), 0);
            assert!(token.is_null());
            let err = maidos_auth_last_error();
            assert!(!err.is_null());
        }
    }

    #[test]
    fn test_ffi_token_has_capability_nulls() {
        unsafe {
            let has = maidos_auth_token_has_capability(ptr::null(), ptr::null(), 0, 0);
            assert!(!has);
        }
    }

    #[test]
    fn test_auth_error_code_mapping() {
        let err = AuthError::TokenExpired;
        assert!(matches!(AuthErrorCode::from(&err), AuthErrorCode::TokenExpired));

        let err = AuthError::InvalidSignature;
        assert!(matches!(AuthErrorCode::from(&err), AuthErrorCode::InvalidSignature));

        let err = AuthError::MalformedToken("x".to_string());
        assert!(matches!(AuthErrorCode::from(&err), AuthErrorCode::MalformedToken));

        let err = AuthError::InvalidToken("x".to_string());
        assert!(matches!(AuthErrorCode::from(&err), AuthErrorCode::InvalidToken));

        let err = AuthError::MissingCapability(Capability::FileRead);
        assert!(matches!(AuthErrorCode::from(&err), AuthErrorCode::MissingCapability));

        let err = AuthError::NoSecretKey;
        assert!(matches!(AuthErrorCode::from(&err), AuthErrorCode::NoSecretKey));

        let err = AuthError::SerializationError("x".to_string());
        assert!(matches!(AuthErrorCode::from(&err), AuthErrorCode::MalformedToken));

        let err = AuthError::Internal("x".to_string());
        assert!(matches!(AuthErrorCode::from(&err), AuthErrorCode::Internal));
    }

    #[test]
    fn test_token_has_capability_invalid_token() {
        let secret = b"test-secret-32-bytes-long!!!!!!";
        unsafe {
            let bad = CString::new("invalid.token").unwrap();
            let has = maidos_auth_token_has_capability(
                bad.as_ptr(),
                secret.as_ptr(),
                secret.len(),
                Capability::LlmChat as u32,
            );
            assert!(!has);
        }
    }
}
