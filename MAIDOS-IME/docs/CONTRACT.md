# MAIDOS-IME v2.0 - FFI Contract

## 1. Purpose

This document defines the Foreign Function Interface (FFI) contracts between the three
layers of MAIDOS-IME: C++ TSF DLL, Rust engine DLL, and C# manager.

## 2. General FFI Conventions

- **ABI**: All exported functions use `extern "C"` / C calling convention.
- **Strings**: UTF-8 encoded, null-terminated (`*const c_char` / `*mut c_char`).
- **Memory Ownership**: Caller allocates output buffers; Rust fills them. Alternatively,
  Rust allocates via `CString` and caller frees via `ime_free_string`.
- **Error Handling**: Functions return `i32` status codes. 0 = success, negative = error.
- **Thread Safety**: All functions are safe to call from any thread. Internal locking
  is handled by the Rust engine.

## 3. C++ TSF to Rust Engine Contract

### 3.1 Initialization

```c
// Initialize the engine. Must be called once before any other function.
// Returns 0 on success, -1 on failure.
int32_t ime_init(const char* config_path);

// Shutdown the engine. Frees all resources.
void ime_shutdown(void);
```

### 3.2 Input Processing

```c
// Process a keystroke. Returns candidate count (>= 0) or error (< 0).
// composition_buf: receives the current composition string (UTF-8).
// buf_len: size of composition_buf in bytes.
int32_t ime_process_key(uint32_t vkey, uint32_t modifiers,
                        char* composition_buf, int32_t buf_len);

// Get candidate at index. Returns 0 on success.
// candidate_buf: receives the candidate string (UTF-8).
int32_t ime_get_candidate(int32_t index, char* candidate_buf, int32_t buf_len);

// Commit candidate at index. Returns 0 on success.
// commit_buf: receives the committed string (UTF-8).
int32_t ime_commit(int32_t index, char* commit_buf, int32_t buf_len);

// Cancel current composition. Returns 0 on success.
int32_t ime_cancel(void);
```

### 3.3 Scheme Management

```c
// Get current scheme ID. Returns scheme enum value.
int32_t ime_get_scheme(void);

// Set active scheme. Returns 0 on success.
// scheme: 0=Bopomofo, 1=Cangjie, 2=Wubi, 3=Pinyin, 4=English, 5=Japanese
int32_t ime_set_scheme(int32_t scheme);
```

## 4. Rust Engine to C# Manager Contract

### 4.1 Dictionary Operations

```c
// Reload dictionaries from disk. Returns 0 on success.
int32_t ime_reload_dictionaries(const char* dict_dir);

// Get engine version string. Caller must free with ime_free_string.
char* ime_get_version(void);

// Free a string allocated by the Rust engine.
void ime_free_string(char* ptr);
```

### 4.2 User Dictionary

```c
// Add entry to user dictionary. Returns 0 on success.
int32_t ime_user_dict_add(const char* key, const char* value, int32_t scheme);

// Remove entry from user dictionary. Returns 0 on success.
int32_t ime_user_dict_remove(const char* key, int32_t scheme);

// Export user dictionary to file. Returns 0 on success.
int32_t ime_user_dict_export(const char* file_path);

// Import user dictionary from file. Returns 0 on success.
int32_t ime_user_dict_import(const char* file_path);
```

### 4.3 LLM Configuration

```c
// Enable or disable LLM-assisted selection. Returns 0 on success.
int32_t ime_set_llm_enabled(int32_t enabled);

// Set LLM timeout in milliseconds. Returns 0 on success.
int32_t ime_set_llm_timeout(int32_t timeout_ms);
```

## 5. Error Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| -1 | General failure |
| -2 | Not initialized |
| -3 | Buffer too small |
| -4 | Invalid scheme |
| -5 | Dictionary load error |
| -6 | LLM communication error |

## 6. C# P/Invoke Declaration Example

```csharp
[DllImport("maidOS_ime.dll", CallingConvention = CallingConvention.Cdecl)]
private static extern int ime_init(
    [MarshalAs(UnmanagedType.LPUTF8Str)] string configPath);
```

## 7. References

- ARCHITECTURE.md - Layer overview
- SPEC-MAIDOS-IME-v2.0.md - Full API specification
