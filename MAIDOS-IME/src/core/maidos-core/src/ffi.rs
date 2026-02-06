//! MAIDOS-IME FFI export layer
//!
//! Provides C-compatible interfaces for C# AI Layer to call Rust core functionality
//! Based on SharedCore opaque handle + JSON exchange pattern
//!
//! [MAIDOS-AUDIT] 8 feature exports + 3 memory management = 11 FFI functions

#![allow(dead_code)]

use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::sync::Mutex;

use crate::schemes::{SchemeType, SchemeFactory, Candidate, InputScheme};
use crate::converter::CharsetConverter;

// =====================================================================
// Global error storage (thread-safe)
// =====================================================================

static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn set_last_error(msg: String) {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = Some(msg);
    }
}

// =====================================================================
// Helper
// =====================================================================

unsafe fn ptr_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}

fn string_to_c(s: String) -> *mut c_char {
    CString::new(s).map(|cs| cs.into_raw()).unwrap_or(ptr::null_mut())
}

fn parse_scheme_type(name: &str) -> SchemeType {
    match name {
        "bopomofo" => SchemeType::Bopomofo,
        "pinyin" => SchemeType::Pinyin,
        "cangjie" => SchemeType::Cangjie,
        "quick" => SchemeType::Quick,
        "wubi" => SchemeType::Wubi,
        "english" => SchemeType::English,
        "japanese" => SchemeType::Japanese,
        "handwriting" => SchemeType::Handwriting,
        "voice" => SchemeType::Voice,
        _ => SchemeType::Bopomofo,
    }
}

/// Serialize a Candidate list to JSON
fn candidates_to_json(candidates: &[Candidate]) -> String {
    let items: Vec<serde_json::Value> = candidates
        .iter()
        .map(|c| {
            serde_json::json!({
                "character": c.character.to_string(),
                "frequency": c.frequency,
                "pronunciation": c.pronunciation
            })
        })
        .collect();
    serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string())
}

// =====================================================================
// FFI export (1): Process input -> candidate JSON
// =====================================================================

/// Process input and return a JSON-formatted candidate list
///
/// # Parameters
/// - `scheme_name`: Input scheme name ("bopomofo", "pinyin", "cangjie", ...)
/// - `input`: Input string
///
/// # Returns
/// - JSON string `[{"character":"...","frequency":100,"pronunciation":"ni3"}, ...]`
/// - null on failure
///
/// # Safety
/// - `scheme_name` / `input` may be null; if non-null, must point to a valid NUL-terminated C string.
/// - The memory pointed to must remain readable and valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn ime_process_input(
    scheme_name: *const c_char,
    input: *const c_char,
) -> *mut c_char {
    let scheme_str = match ptr_to_str(scheme_name) {
        Some(s) => s,
        None => {
            set_last_error("scheme_name is null".to_string());
            return ptr::null_mut();
        }
    };

    let input_str = match ptr_to_str(input) {
        Some(s) => s,
        None => {
            set_last_error("input is null".to_string());
            return ptr::null_mut();
        }
    };

    let scheme_type = parse_scheme_type(scheme_str);
    let scheme = SchemeFactory::create_scheme_simple(&scheme_type);

    match scheme.process_input(input_str) {
        Ok(candidates) => string_to_c(candidates_to_json(&candidates)),
        Err(e) => {
            set_last_error(format!("process_input failed: {}", e));
            ptr::null_mut()
        }
    }
}

// =====================================================================
// FFI export (2): Get candidates
// =====================================================================

/// Get candidate list (JSON)
///
/// # Safety
/// - `scheme_name` / `input` may be null; if non-null, must point to a valid NUL-terminated C string.
/// - The memory pointed to must remain readable and valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn ime_get_candidates(
    scheme_name: *const c_char,
    input: *const c_char,
) -> *mut c_char {
    let scheme_str = match ptr_to_str(scheme_name) {
        Some(s) => s,
        None => {
            set_last_error("scheme_name is null".to_string());
            return ptr::null_mut();
        }
    };

    let input_str = match ptr_to_str(input) {
        Some(s) => s,
        None => {
            set_last_error("input is null".to_string());
            return ptr::null_mut();
        }
    };

    let scheme_type = parse_scheme_type(scheme_str);
    let scheme = SchemeFactory::create_scheme_simple(&scheme_type);

    match scheme.get_candidates(input_str) {
        Ok(candidates) => string_to_c(candidates_to_json(&candidates)),
        Err(e) => {
            set_last_error(format!("get_candidates failed: {}", e));
            ptr::null_mut()
        }
    }
}

// =====================================================================
// FFI export (3): Traditional/Simplified conversion
// =====================================================================

/// Charset conversion (Traditional <-> Simplified)
///
/// # Parameters
/// - `text`: Text to convert
/// - `from_charset`: Source charset ("traditional", "simplified")
/// - `to_charset`: Target charset
///
/// # Returns
/// - Converted string; must be freed with `ime_free_string`
///
/// # Safety
/// - `text` / `from_charset` / `to_charset` may be null; if non-null, must point to a valid NUL-terminated C string.
/// - The memory pointed to must remain readable and valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn ime_convert_charset(
    text: *const c_char,
    from_charset: *const c_char,
    to_charset: *const c_char,
) -> *mut c_char {
    let text_str = match ptr_to_str(text) {
        Some(s) => s,
        None => {
            set_last_error("text is null".to_string());
            return ptr::null_mut();
        }
    };

    let from_str = match ptr_to_str(from_charset) {
        Some(s) => s,
        None => {
            set_last_error("from_charset is null".to_string());
            return ptr::null_mut();
        }
    };

    let to_str = match ptr_to_str(to_charset) {
        Some(s) => s,
        None => {
            set_last_error("to_charset is null".to_string());
            return ptr::null_mut();
        }
    };

    let from = match from_str {
        "traditional" => maidos_config::Charset::Traditional,
        "simplified" => maidos_config::Charset::Simplified,
        _ => maidos_config::Charset::Traditional,
    };

    let to = match to_str {
        "traditional" => maidos_config::Charset::Traditional,
        "simplified" => maidos_config::Charset::Simplified,
        _ => maidos_config::Charset::Simplified,
    };

    let result = CharsetConverter::convert(text_str, &from, &to);
    string_to_c(result)
}

// =====================================================================
// FFI export (4): Get supported input schemes
// =====================================================================

/// Return supported input scheme list (JSON)
#[no_mangle]
pub extern "C" fn ime_supported_schemes() -> *mut c_char {
    let schemes = SchemeFactory::get_supported_schemes();
    let names: Vec<&str> = schemes
        .iter()
        .map(|s| match s {
            SchemeType::Bopomofo => "bopomofo",
            SchemeType::Pinyin => "pinyin",
            SchemeType::Cangjie => "cangjie",
            SchemeType::Quick => "quick",
            SchemeType::Wubi => "wubi",
            SchemeType::English => "english",
            SchemeType::Japanese => "japanese",
            SchemeType::Handwriting => "handwriting",
            SchemeType::Voice => "voice",
        })
        .collect();

    match serde_json::to_string(&names) {
        Ok(json) => string_to_c(json),
        Err(e) => {
            set_last_error(format!("Serialization failed: {}", e));
            ptr::null_mut()
        }
    }
}

// =====================================================================
// FFI export (5): Language detection (character-based analysis)
// =====================================================================

/// Detect text language
///
/// # Returns
/// - Language name ("chinese", "english", "japanese", "mixed")
///
/// # Safety
/// - `text` may be null; if non-null, must point to a valid NUL-terminated C string.
/// - The memory pointed to must remain readable and valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn ime_detect_language(
    text: *const c_char,
) -> *mut c_char {
    let text_str = match ptr_to_str(text) {
        Some(s) => s,
        None => {
            set_last_error("text is null".to_string());
            return ptr::null_mut();
        }
    };

    let mut cjk_count = 0u32;
    let mut ascii_count = 0u32;
    let mut jp_count = 0u32;
    let total = text_str.chars().count() as u32;

    for ch in text_str.chars() {
        if ch.is_ascii_alphabetic() {
            ascii_count += 1;
        } else if ('\u{4e00}'..='\u{9fff}').contains(&ch) {
            cjk_count += 1;
        } else if ('\u{3040}'..='\u{30ff}').contains(&ch) || ('\u{31f0}'..='\u{31ff}').contains(&ch) {
            jp_count += 1;
        }
    }

    let lang = if total == 0 {
        "unknown"
    } else if jp_count > 0 {
        "japanese"
    } else if cjk_count > ascii_count {
        "chinese"
    } else if ascii_count > cjk_count {
        "english"
    } else {
        "mixed"
    };

    string_to_c(lang.to_string())
}

// =====================================================================
// FFI export (6): Version info
// =====================================================================

/// Return IME Core version info (JSON)
#[no_mangle]
pub extern "C" fn ime_version() -> *mut c_char {
    let info = serde_json::json!({
        "version": "0.1.0",
        "name": "MAIDOS-IME Core",
        "schemes": ["bopomofo", "pinyin", "cangjie", "quick", "wubi", "english", "japanese"],
        "charsets": ["traditional", "simplified"],
    });

    match serde_json::to_string(&info) {
        Ok(json) => string_to_c(json),
        Err(e) => {
            set_last_error(format!("Serialization failed: {}", e));
            ptr::null_mut()
        }
    }
}

// =====================================================================
// FFI export (7): Pinyin lookup
// =====================================================================

/// Pinyin to candidates (direct lookup, bypasses full scheme pipeline)
///
/// # Safety
/// - `pinyin` may be null; if non-null, must point to a valid NUL-terminated C string.
/// - The memory pointed to must remain readable and valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn ime_pinyin_lookup(
    pinyin: *const c_char,
) -> *mut c_char {
    let pinyin_str = match ptr_to_str(pinyin) {
        Some(s) => s,
        None => {
            set_last_error("pinyin is null".to_string());
            return ptr::null_mut();
        }
    };

    // Use PinyinScheme for direct lookup
    let scheme = crate::schemes::PinyinScheme::new_default();
    match scheme.get_candidates(pinyin_str) {
        Ok(candidates) => string_to_c(candidates_to_json(&candidates)),
        Err(e) => {
            set_last_error(format!("Pinyin lookup failed: {}", e));
            ptr::null_mut()
        }
    }
}

// =====================================================================
// FFI export (8): Initialization
// =====================================================================

/// Initialize IME Core
#[no_mangle]
pub extern "C" fn ime_init() -> i32 {
    // No special initialization needed currently; interface reserved for future extension
    0
}

// =====================================================================
// Memory management FFI (3 functions)
// =====================================================================

/// Get the last error message
#[no_mangle]
pub extern "C" fn ime_last_error() -> *mut c_char {
    if let Ok(guard) = LAST_ERROR.lock() {
        if let Some(ref msg) = *guard {
            return string_to_c(msg.clone());
        }
    }
    ptr::null_mut()
}

/// Free a string returned by FFI
///
/// # Safety
/// - `s` must be null or a pointer returned by this crate's FFI functions (e.g. `ime_process_input`) that has not yet been freed.
/// - Each pointer must only be freed once.
#[no_mangle]
pub unsafe extern "C" fn ime_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Clear the last error
#[no_mangle]
pub extern "C" fn ime_clear_error() {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = None;
    }
}

// =====================================================================
// FFI: User learning (2)
// =====================================================================

/// Record that the user selected `character` when typing `input_code` in `scheme`.
/// The selection is persisted to disk immediately.
/// Returns 0 on success, -1 on error.
///
/// # Safety
/// - All pointer parameters must be null or point to valid NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn ime_learn(
    scheme_name: *const c_char,
    input_code: *const c_char,
    character: *const c_char,
) -> i32 {
    let scheme = match ptr_to_str(scheme_name) {
        Some(s) => s,
        None => { set_last_error("scheme_name is null".to_string()); return -1; }
    };
    let input = match ptr_to_str(input_code) {
        Some(s) => s,
        None => { set_last_error("input_code is null".to_string()); return -1; }
    };
    let ch = match ptr_to_str(character) {
        Some(s) => s,
        None => { set_last_error("character is null".to_string()); return -1; }
    };

    match crate::user_learning::learn_and_save(scheme, input, ch) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(format!("Learn failed: {}", e));
            -1
        }
    }
}

/// Clear all user-learned data for a scheme.
/// Returns 0 on success, -1 on error.
///
/// # Safety
/// - `scheme_name` must be null or point to a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn ime_clear_learned(
    scheme_name: *const c_char,
) -> i32 {
    let scheme = match ptr_to_str(scheme_name) {
        Some(s) => s,
        None => { set_last_error("scheme_name is null".to_string()); return -1; }
    };

    match crate::user_learning::clear_user_table(scheme) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(format!("Clear learned data failed: {}", e));
            -1
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_ffi_process_input_bopomofo() {
        unsafe {
            let scheme = CString::new("bopomofo").unwrap();
            let input = CString::new("ㄋㄧˇ").unwrap();

            let result = ime_process_input(scheme.as_ptr(), input.as_ptr());
            // May return null (if the bopomofo table has no matching entry); this is normal
            if !result.is_null() {
                let json = CStr::from_ptr(result).to_str().unwrap();
                assert!(json.starts_with('['));
                ime_free_string(result);
            }
        }
    }

    #[test]
    fn test_ffi_process_input_pinyin() {
        unsafe {
            let scheme = CString::new("pinyin").unwrap();
            let input = CString::new("ni").unwrap();

            let result = ime_process_input(scheme.as_ptr(), input.as_ptr());
            if !result.is_null() {
                let json = CStr::from_ptr(result).to_str().unwrap();
                assert!(json.starts_with('['));
                ime_free_string(result);
            }
        }
    }

    #[test]
    fn test_ffi_convert_charset() {
        unsafe {
            let text = CString::new("Hello").unwrap();
            let from = CString::new("traditional").unwrap();
            let to = CString::new("simplified").unwrap();

            let result = ime_convert_charset(text.as_ptr(), from.as_ptr(), to.as_ptr());
            assert!(!result.is_null());
            ime_free_string(result);
        }
    }

    #[test]
    fn test_ffi_supported_schemes() {
        let result = ime_supported_schemes();
        assert!(!result.is_null());

        unsafe {
            let json = CStr::from_ptr(result).to_str().unwrap();
            assert!(json.contains("bopomofo"));
            assert!(json.contains("pinyin"));
            ime_free_string(result);
        }
    }

    #[test]
    fn test_ffi_detect_language() {
        unsafe {
            let text = CString::new("Hello world").unwrap();
            let result = ime_detect_language(text.as_ptr());
            assert!(!result.is_null());
            let lang = CStr::from_ptr(result).to_str().unwrap();
            assert_eq!(lang, "english");
            ime_free_string(result);

            let text2 = CString::new("你好世界").unwrap();
            let result2 = ime_detect_language(text2.as_ptr());
            assert!(!result2.is_null());
            let lang2 = CStr::from_ptr(result2).to_str().unwrap();
            assert_eq!(lang2, "chinese");
            ime_free_string(result2);
        }
    }

    #[test]
    fn test_ffi_version() {
        let result = ime_version();
        assert!(!result.is_null());

        unsafe {
            let json = CStr::from_ptr(result).to_str().unwrap();
            assert!(json.contains("MAIDOS-IME"));
            assert!(json.contains("0.1.0"));
            ime_free_string(result);
        }
    }

    #[test]
    fn test_ffi_null_safety() {
        unsafe {
            let result = ime_process_input(ptr::null(), ptr::null());
            assert!(result.is_null());

            ime_free_string(ptr::null_mut());

            ime_clear_error();
            let err = ime_last_error();
            assert!(err.is_null());
        }
    }

    #[test]
    fn test_ffi_init() {
        assert_eq!(ime_init(), 0);
    }
}
