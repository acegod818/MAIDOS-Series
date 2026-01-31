//! C FFI exports for maidos-llm
//!
//! <impl>
//! WHAT: C-compatible FFI for P/Invoke from C#/.NET
//! WHY: Cross-language integration with MAIDOS applications
//! HOW: Blocking wrappers around async API, opaque pointers
//! TEST: FFI handle creation, completion, cleanup
//! </impl>

use crate::error::LlmError;
use crate::message::Message;
use crate::provider::{CompletionRequest, LlmProvider};
use crate::providers::{create_provider, ProviderType};
use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Opaque LLM handle
pub struct LlmHandle {
    provider: Arc<dyn LlmProvider>,
    runtime: Runtime,
}

/// FFI error codes for detailed error handling
#[repr(C)]
pub enum FfiErrorCode {
    /// No error
    None = 0,
    /// Invalid arguments
    InvalidArguments = 1,
    /// Authentication error
    Auth = 2,
    /// Network error
    Network = 3,
    /// Rate limited
    RateLimited = 4,
    /// Invalid request
    InvalidRequest = 5,
    /// Provider error
    Provider = 6,
    /// Vision not supported
    VisionNotSupported = 7,
    /// Tools not supported
    ToolsNotSupported = 8,
    /// Budget exceeded
    BudgetExceeded = 9,
    /// Parse error
    ParseError = 10,
    /// Unknown error
    Unknown = 99,
}

/// Detailed error information for FFI
#[repr(C)]
pub struct FfiErrorDetails {
    /// Error code
    pub code: FfiErrorCode,
    /// Error type name
    pub error_type: *mut c_char,
    /// Human-readable message
    pub message: *mut c_char,
    /// Suggestion for resolution (may be null)
    pub suggestion: *mut c_char,
    /// Retry after seconds (for rate limiting, 0 if not applicable)
    pub retry_after_secs: u32,
    /// Is this a capability error (can be resolved by switching provider)
    pub is_capability_error: bool,
}

impl FfiErrorDetails {
    fn from_llm_error(e: &LlmError) -> Self {
        let (code, error_type, suggestion) = match e {
            LlmError::Auth(_) => (FfiErrorCode::Auth, "Auth", None),
            LlmError::Network(_) => (FfiErrorCode::Network, "Network", None),
            LlmError::RateLimited { retry_after_secs } => {
                (FfiErrorCode::RateLimited, "RateLimited", Some(format!("Wait {} seconds", retry_after_secs)))
            }
            LlmError::InvalidRequest(_) => (FfiErrorCode::InvalidRequest, "InvalidRequest", None),
            LlmError::Provider(_) => (FfiErrorCode::Provider, "Provider", None),
            LlmError::ProviderError { .. } => (FfiErrorCode::Provider, "ProviderError", None),
            LlmError::VisionNotSupported { suggestion, .. } => {
                (FfiErrorCode::VisionNotSupported, "VisionNotSupported", Some(suggestion.clone()))
            }
            LlmError::ToolsNotSupported { suggestion, .. } => {
                (FfiErrorCode::ToolsNotSupported, "ToolsNotSupported", Some(suggestion.clone()))
            }
            LlmError::BudgetExceeded { .. } => (FfiErrorCode::BudgetExceeded, "BudgetExceeded", None),
            LlmError::ParseError(_) => (FfiErrorCode::ParseError, "ParseError", None),
            _ => (FfiErrorCode::Unknown, "Unknown", None),
        };

        let retry_after = if let LlmError::RateLimited { retry_after_secs } = e {
            *retry_after_secs as u32
        } else {
            0
        };

        Self {
            code,
            error_type: CString::new(error_type).map(|s| s.into_raw()).unwrap_or(ptr::null_mut()),
            message: CString::new(e.to_string()).map(|s| s.into_raw()).unwrap_or(ptr::null_mut()),
            suggestion: suggestion
                .and_then(|s| CString::new(s).ok())
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut()),
            retry_after_secs: retry_after,
            is_capability_error: e.is_capability_error(),
        }
    }

    fn null() -> Self {
        Self {
            code: FfiErrorCode::None,
            error_type: ptr::null_mut(),
            message: ptr::null_mut(),
            suggestion: ptr::null_mut(),
            retry_after_secs: 0,
            is_capability_error: false,
        }
    }
}

/// Completion result for FFI
#[repr(C)]
pub struct FfiCompletionResult {
    /// Success flag
    pub success: bool,
    /// Response text (null on error)
    pub text: *mut c_char,
    /// Error message (null on success) - simple string for backward compatibility
    pub error: *mut c_char,
    /// Detailed error information (code=None on success)
    pub error_details: FfiErrorDetails,
    /// Prompt tokens used
    pub prompt_tokens: u32,
    /// Completion tokens used
    pub completion_tokens: u32,
    /// Model used
    pub model: *mut c_char,
}

// ============================================================================
// Provider Management
// ============================================================================

/// Create an LLM provider
///
/// # Arguments
/// * `provider_name` - Provider name: "openai", "anthropic", "ollama"
/// * `api_key` - API key (can be null for ollama)
/// * `base_url` - Custom base URL (can be null for defaults)
///
/// # Returns
/// Handle to the provider, or null on failure
///
/// # Safety
/// Strings must be valid UTF-8 or null
#[no_mangle]
pub unsafe extern "C" fn maidos_llm_create(
    provider_name: *const c_char,
    api_key: *const c_char,
    base_url: *const c_char,
) -> *mut LlmHandle {
    if provider_name.is_null() {
        return ptr::null_mut();
    }

    let provider_str = match CStr::from_ptr(provider_name).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let provider_type = match ProviderType::parse(provider_str) {
        Some(t) => t,
        None => return ptr::null_mut(),
    };

    let api_key = if api_key.is_null() {
        None
    } else {
        CStr::from_ptr(api_key).to_str().ok().map(|s| s.to_string())
    };

    let base_url = if base_url.is_null() {
        None
    } else {
        CStr::from_ptr(base_url).to_str().ok().map(|s| s.to_string())
    };

    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    let provider = match create_provider(provider_type, api_key, base_url) {
        Ok(p) => p,
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(LlmHandle { provider, runtime }))
}

/// Get provider name
///
/// # Safety
/// Handle must be valid
#[no_mangle]
pub unsafe extern "C" fn maidos_llm_provider_name(handle: *const LlmHandle) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }

    let h = &*handle;
    CString::new(h.provider.name())
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Check if provider is healthy
///
/// # Safety
/// Handle must be valid
#[no_mangle]
pub unsafe extern "C" fn maidos_llm_health_check(handle: *mut LlmHandle) -> bool {
    if handle.is_null() {
        return false;
    }

    let h = &*handle;
    h.runtime
        .block_on(h.provider.health_check())
        .unwrap_or(false)
}

// ============================================================================
// Completion
// ============================================================================

/// Complete a chat request
///
/// # Arguments
/// * `handle` - Provider handle
/// * `model` - Model name
/// * `system` - System prompt (can be null)
/// * `user_message` - User message
/// * `max_tokens` - Maximum tokens (0 for default)
/// * `temperature` - Temperature (negative for default)
///
/// # Returns
/// Completion result (must be freed with maidos_llm_result_free)
///
/// # Safety
/// All pointers must be valid or null where allowed
#[no_mangle]
pub unsafe extern "C" fn maidos_llm_complete(
    handle: *mut LlmHandle,
    model: *const c_char,
    system: *const c_char,
    user_message: *const c_char,
    max_tokens: u32,
    temperature: f32,
) -> *mut FfiCompletionResult {
    let make_error = |msg: &str| -> *mut FfiCompletionResult {
        Box::into_raw(Box::new(FfiCompletionResult {
            success: false,
            text: ptr::null_mut(),
            error: CString::new(msg).map(|s| s.into_raw()).unwrap_or(ptr::null_mut()),
            error_details: FfiErrorDetails {
                code: FfiErrorCode::InvalidArguments,
                error_type: CString::new("InvalidArguments").map(|s| s.into_raw()).unwrap_or(ptr::null_mut()),
                message: CString::new(msg).map(|s| s.into_raw()).unwrap_or(ptr::null_mut()),
                suggestion: ptr::null_mut(),
                retry_after_secs: 0,
                is_capability_error: false,
            },
            prompt_tokens: 0,
            completion_tokens: 0,
            model: ptr::null_mut(),
        }))
    };

    let make_llm_error = |e: &LlmError| -> *mut FfiCompletionResult {
        Box::into_raw(Box::new(FfiCompletionResult {
            success: false,
            text: ptr::null_mut(),
            error: CString::new(e.to_string()).map(|s| s.into_raw()).unwrap_or(ptr::null_mut()),
            error_details: FfiErrorDetails::from_llm_error(e),
            prompt_tokens: 0,
            completion_tokens: 0,
            model: ptr::null_mut(),
        }))
    };

    if handle.is_null() || model.is_null() || user_message.is_null() {
        return make_error("Invalid arguments");
    }

    let h = &*handle;

    let model_str = match CStr::from_ptr(model).to_str() {
        Ok(s) => s,
        Err(_) => return make_error("Invalid model string"),
    };

    let user_str = match CStr::from_ptr(user_message).to_str() {
        Ok(s) => s,
        Err(_) => return make_error("Invalid message string"),
    };

    let mut request = CompletionRequest::new(model_str).message(Message::user(user_str));

    if !system.is_null() {
        if let Ok(s) = CStr::from_ptr(system).to_str() {
            request = request.system(s);
        }
    }

    if max_tokens > 0 {
        request = request.max_tokens(max_tokens);
    }

    if temperature >= 0.0 {
        request = request.temperature(temperature);
    }

    match h.runtime.block_on(h.provider.complete(request)) {
        Ok(response) => {
            let text = CString::new(response.message.text())
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut());

            let model_str = CString::new(response.model)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut());

            Box::into_raw(Box::new(FfiCompletionResult {
                success: true,
                text,
                error: ptr::null_mut(),
                error_details: FfiErrorDetails::null(),
                prompt_tokens: response.usage.prompt_tokens,
                completion_tokens: response.usage.completion_tokens,
                model: model_str,
            }))
        }
        Err(e) => make_llm_error(&e),
    }
}

/// Free a completion result
///
/// # Safety
/// Result must be valid or null
#[no_mangle]
pub unsafe extern "C" fn maidos_llm_result_free(result: *mut FfiCompletionResult) {
    if !result.is_null() {
        let r = Box::from_raw(result);
        if !r.text.is_null() {
            let _ = CString::from_raw(r.text);
        }
        if !r.error.is_null() {
            let _ = CString::from_raw(r.error);
        }
        if !r.model.is_null() {
            let _ = CString::from_raw(r.model);
        }
        // Free error details strings
        if !r.error_details.error_type.is_null() {
            let _ = CString::from_raw(r.error_details.error_type);
        }
        if !r.error_details.message.is_null() {
            let _ = CString::from_raw(r.error_details.message);
        }
        if !r.error_details.suggestion.is_null() {
            let _ = CString::from_raw(r.error_details.suggestion);
        }
    }
}

/// Destroy an LLM handle
///
/// # Safety
/// Handle must be valid or null
#[no_mangle]
pub unsafe extern "C" fn maidos_llm_destroy(handle: *mut LlmHandle) {
    if !handle.is_null() {
        let _ = Box::from_raw(handle);
    }
}

/// Free a string returned by FFI functions
///
/// # Safety
/// String must have been allocated by this library
#[no_mangle]
pub unsafe extern "C" fn maidos_llm_string_free(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_create_ollama_provider() {
        unsafe {
            let provider = CString::new("ollama").unwrap();
            let handle = maidos_llm_create(provider.as_ptr(), ptr::null(), ptr::null());
            assert!(!handle.is_null());

            let name = maidos_llm_provider_name(handle);
            assert!(!name.is_null());
            let name_str = CStr::from_ptr(name).to_str().unwrap();
            assert_eq!(name_str, "Ollama");

            maidos_llm_string_free(name);
            maidos_llm_destroy(handle);
        }
    }

    #[test]
    fn test_create_invalid_provider() {
        unsafe {
            let provider = CString::new("invalid").unwrap();
            let handle = maidos_llm_create(provider.as_ptr(), ptr::null(), ptr::null());
            assert!(handle.is_null());
        }
    }

    #[test]
    fn test_create_openai_without_key() {
        unsafe {
            let provider = CString::new("openai").unwrap();
            let handle = maidos_llm_create(provider.as_ptr(), ptr::null(), ptr::null());
            // Should fail without API key
            assert!(handle.is_null());
        }
    }

    #[test]
    fn test_null_safety() {
        unsafe {
            // Null handle
            assert_eq!(maidos_llm_provider_name(ptr::null()), ptr::null_mut());
            assert!(!maidos_llm_health_check(ptr::null_mut()));

            // Null destroy should not crash
            maidos_llm_destroy(ptr::null_mut());
            maidos_llm_result_free(ptr::null_mut());
            maidos_llm_string_free(ptr::null_mut());
        }
    }

    #[test]
    fn test_complete_null_args() {
        unsafe {
            let result = maidos_llm_complete(
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                0,
                -1.0,
            );
            assert!(!result.is_null());
            let r = &*result;
            assert!(!r.success);
            assert!(!r.error.is_null());
            maidos_llm_result_free(result);
        }
    }

    #[test]
    fn test_error_details_structure() {
        unsafe {
            let result = maidos_llm_complete(
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                0,
                -1.0,
            );
            assert!(!result.is_null());
            let r = &*result;
            assert!(!r.success);
            
            // Check error details
            assert!(!matches!(r.error_details.code, FfiErrorCode::None));
            assert!(!r.error_details.error_type.is_null());
            assert!(!r.error_details.message.is_null());
            assert!(!r.error_details.is_capability_error);
            
            maidos_llm_result_free(result);
        }
    }

    #[test]
    fn test_ffi_error_details_from_llm_error() {
        // Test VisionNotSupported
        let err = LlmError::vision_not_supported("TestProvider");
        let details = FfiErrorDetails::from_llm_error(&err);
        assert!(matches!(details.code, FfiErrorCode::VisionNotSupported));
        assert!(details.is_capability_error);
        
        // Test ToolsNotSupported
        let err = LlmError::tools_not_supported("TestProvider");
        let details = FfiErrorDetails::from_llm_error(&err);
        assert!(matches!(details.code, FfiErrorCode::ToolsNotSupported));
        assert!(details.is_capability_error);
        
        // Test RateLimited
        let err = LlmError::RateLimited { retry_after_secs: 30 };
        let details = FfiErrorDetails::from_llm_error(&err);
        assert!(matches!(details.code, FfiErrorCode::RateLimited));
        assert_eq!(details.retry_after_secs, 30);
        assert!(!details.is_capability_error);
        
        // Test Auth
        let err = LlmError::Auth("Invalid key".to_string());
        let details = FfiErrorDetails::from_llm_error(&err);
        assert!(matches!(details.code, FfiErrorCode::Auth));
        assert!(!details.is_capability_error);

        // Test BudgetExceeded
        let err = LlmError::BudgetExceeded("limit".to_string());
        let details = FfiErrorDetails::from_llm_error(&err);
        assert!(matches!(details.code, FfiErrorCode::BudgetExceeded));

        // Test ParseError
        let err = LlmError::ParseError("bad json".to_string());
        let details = FfiErrorDetails::from_llm_error(&err);
        assert!(matches!(details.code, FfiErrorCode::ParseError));

        // Test InvalidRequest
        let err = LlmError::InvalidRequest("bad request".to_string());
        let details = FfiErrorDetails::from_llm_error(&err);
        assert!(matches!(details.code, FfiErrorCode::InvalidRequest));

        // Test Provider
        let err = LlmError::Provider("generic".to_string());
        let details = FfiErrorDetails::from_llm_error(&err);
        assert!(matches!(details.code, FfiErrorCode::Provider));

        // Test ProviderError
        let err = LlmError::ProviderError {
            code: "E123".to_string(),
            message: "bad".to_string(),
        };
        let details = FfiErrorDetails::from_llm_error(&err);
        assert!(matches!(details.code, FfiErrorCode::Provider));
    }

    #[test]
    fn test_ffi_error_details_null() {
        let details = FfiErrorDetails::null();
        assert!(matches!(details.code, FfiErrorCode::None));
        assert!(details.error_type.is_null());
        assert!(details.message.is_null());
        assert!(details.suggestion.is_null());
        assert_eq!(details.retry_after_secs, 0);
        assert!(!details.is_capability_error);
    }

    #[test]
    fn test_complete_with_provider_error() {
        unsafe {
            let provider = CString::new("ollama").unwrap();
            let base_url = CString::new("http://127.0.0.1:0").unwrap();
            let handle = maidos_llm_create(provider.as_ptr(), ptr::null(), base_url.as_ptr());
            assert!(!handle.is_null());

            let model = CString::new("llama3").unwrap();
            let user = CString::new("hi").unwrap();
            let result = maidos_llm_complete(
                handle,
                model.as_ptr(),
                ptr::null(),
                user.as_ptr(),
                16,
                0.2,
            );
            assert!(!result.is_null());
            let r = &*result;
            assert!(!r.success);
            assert!(!r.error.is_null());
            assert!(!r.error_details.message.is_null());

            maidos_llm_result_free(result);
            maidos_llm_destroy(handle);
        }
    }

    #[test]
    fn test_health_check_invalid_server() {
        unsafe {
            let provider = CString::new("ollama").unwrap();
            let base_url = CString::new("http://127.0.0.1:0").unwrap();
            let handle = maidos_llm_create(provider.as_ptr(), ptr::null(), base_url.as_ptr());
            assert!(!handle.is_null());

            let ok = maidos_llm_health_check(handle);
            assert!(!ok);

            maidos_llm_destroy(handle);
        }
    }

    #[test]
    fn test_complete_with_system_prompt() {
        unsafe {
            let provider = CString::new("ollama").unwrap();
            let base_url = CString::new("http://127.0.0.1:0").unwrap();
            let handle = maidos_llm_create(provider.as_ptr(), ptr::null(), base_url.as_ptr());
            assert!(!handle.is_null());

            let model = CString::new("llama3").unwrap();
            let system = CString::new("system").unwrap();
            let user = CString::new("hi").unwrap();
            let result = maidos_llm_complete(
                handle,
                model.as_ptr(),
                system.as_ptr(),
                user.as_ptr(),
                0,
                -1.0,
            );
            assert!(!result.is_null());
            maidos_llm_result_free(result);
            maidos_llm_destroy(handle);
        }
    }

    #[test]
    fn test_create_invalid_utf8_provider() {
        unsafe {
            let bytes = [0xffu8, 0x00u8];
            let provider = CStr::from_bytes_with_nul_unchecked(&bytes);
            let handle = maidos_llm_create(provider.as_ptr(), ptr::null(), ptr::null());
            assert!(handle.is_null());
        }
    }

    #[test]
    fn test_complete_invalid_utf8_model_and_user() {
        unsafe {
            let provider = CString::new("ollama").unwrap();
            let base_url = CString::new("http://127.0.0.1:0").unwrap();
            let handle = maidos_llm_create(provider.as_ptr(), ptr::null(), base_url.as_ptr());
            assert!(!handle.is_null());

            let invalid_bytes = [0xffu8, 0x00u8];
            let invalid = CStr::from_bytes_with_nul_unchecked(&invalid_bytes);
            let ok_user = CString::new("ok").unwrap();
            let ok_model = CString::new("model").unwrap();

            let result = maidos_llm_complete(
                handle,
                invalid.as_ptr(),
                ptr::null(),
                ok_user.as_ptr(),
                0,
                -1.0,
            );
            assert!(!result.is_null());
            maidos_llm_result_free(result);

            let result = maidos_llm_complete(
                handle,
                ok_model.as_ptr(),
                ptr::null(),
                invalid.as_ptr(),
                0,
                -1.0,
            );
            assert!(!result.is_null());
            maidos_llm_result_free(result);

            maidos_llm_destroy(handle);
        }
    }
}
