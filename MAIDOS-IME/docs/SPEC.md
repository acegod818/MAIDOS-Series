# MAIDOS-IME v2.0 - Technical Specification Summary

## 1. Purpose

This document provides a high-level technical specification summary for MAIDOS-IME.
The full specification is maintained in SPEC-MAIDOS-IME-v2.0.md; this summary captures
the key technical decisions and constraints for quick reference.

## 2. System Boundary

MAIDOS-IME operates as a Windows Text Services Framework (TSF) input processor. It
intercepts keystrokes at the OS level, processes them through a Rust-based engine, and
commits composed text back to the target application via TSF.

## 3. Technology Stack

| Layer | Language | Output | Role |
|-------|----------|--------|------|
| TSF Hook | C++ | COM DLL | Windows TSF integration, keystroke interception |
| Core Engine | Rust | cdylib DLL | Input processing, dictionary lookup, LLM bridge |
| Manager | C# (.NET 8.0) | EXE / Class Library | Settings UI, dictionary management, installer |

## 4. Key Technical Constraints

- **TSF Requirement**: Windows mandates COM-based DLL for input method registration;
  Rust cannot directly implement COM TSF interfaces, necessitating the C++ layer.
- **FFI Boundary**: C++ TSF DLL calls Rust engine via C-ABI (`extern "C"` functions).
  Rust engine exposes a flat API surface; no COM on the Rust side.
- **LLM Locality**: All LLM inference runs via Ollama on localhost:11434. No external
  network access is permitted from the engine.
- **Dictionary Format**: JSON-based dictionary files for human readability and easy
  community contribution. Binary cache generated on first load for performance.

## 5. Data Flow

```
Keystroke -> [C++ TSF COM DLL] -> FFI call -> [Rust Engine]
                                                  |
                                          Dictionary Lookup
                                                  |
                                          (optional) Ollama LLM
                                                  |
                                          Candidate List
                                                  |
[Application] <- TSF commit <- [C++ TSF COM DLL] <- FFI return
```

## 6. Dictionary Data

| Dictionary | Format | Size (approx.) | Entries |
|------------|--------|-----------------|---------|
| Bopomofo | JSON | 8 MB | ~80,000 |
| Pinyin | JSON | 12 MB | ~120,000 |
| Cangjie | JSON | 6 MB | ~60,000 |
| Wubi | JSON | 5 MB | ~50,000 |
| English | JSON | 3 MB | ~100,000 |
| Japanese | JSON | 15 MB | ~150,000 |
| Trad-Simp Map | JSON | 1 MB | ~8,000 |

## 7. Functional Requirements Reference

See PRD.md for FR-001 through FR-008 definitions.
See AC_MATRIX.md for acceptance criteria mapping.

## 8. Non-Functional Requirements Reference

See NFR.md for NFR-001 through NFR-008 definitions.

## 9. Full Specification

The complete specification including detailed API signatures, data schemas, error
handling, and edge case behavior is maintained in:

**SPEC-MAIDOS-IME-v2.0.md** (canonical source of truth)

This summary document is derived from the full specification and shall be updated
whenever the canonical spec is revised.
