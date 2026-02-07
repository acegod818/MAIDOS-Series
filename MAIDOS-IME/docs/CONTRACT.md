# MAIDOS-IME -- API Contract

| Field   | Value        |
|---------|--------------|
| Product | MAIDOS-IME   |
| Version | 0.2.0        |

## Rust FFI Exports (maidos-core cdylib)

```rust
#[no_mangle] pub extern "C" fn ime_init() -> i32;
#[no_mangle] pub extern "C" fn ime_process_keystroke(key_code: u32) -> *mut c_char;
#[no_mangle] pub extern "C" fn ime_get_candidates(buf: *mut c_char, len: usize) -> i32;
#[no_mangle] pub extern "C" fn ime_select_candidate(index: u32) -> i32;
#[no_mangle] pub extern "C" fn ime_shutdown() -> i32;
#[no_mangle] pub extern "C" fn ime_free_string(ptr: *mut c_char);
```

## TSF COM Interfaces Implemented

| Interface                | Purpose                        |
|--------------------------|--------------------------------|
| ITfTextInputProcessor    | Activate / Deactivate          |
| ITfKeyEventSink          | OnKeyDown / OnKeyUp            |
| ITfCompositionSink       | Composition start / end        |
| ITfCandidateListUIElement| Candidate window rendering     |

## Return Codes

| Code | Meaning          |
|------|------------------|
| 0    | Success          |
| -1   | Not initialised  |
| -2   | Invalid input    |
| -3   | Dictionary error |
| -4   | LLM timeout      |

## String Encoding

All FFI strings are UTF-8 `CString`. Caller must use `ime_free_string`.

*MAIDOS-IME CONTRACT v0.2.0 -- CodeQC Gate C Compliant*
