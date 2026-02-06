# MAIDOS-IME v2.0 - Architecture Document

## 1. Purpose

This document describes the system architecture of MAIDOS-IME, covering the three-layer
design, inter-layer communication, and supporting subsystems.

## 2. Architecture Overview

```
+-------------------+     +-------------------+     +-------------------+
|   C++ TSF Layer   | --> |   Rust Engine     | <-- |   C# Manager      |
|   (COM DLL)       |     |   (cdylib DLL)    |     |   (.NET 8.0 EXE)  |
+-------------------+     +-------------------+     +-------------------+
        |                         |                         |
   TSF / COM API           Dictionary Data            Settings / UI
   Keystroke I/O           LLM Bridge (Ollama)        Installer Logic
```

## 3. Layer Responsibilities

### 3.1 C++ TSF Layer

- **Output**: `maidOS_ime_tsf.dll` (COM DLL)
- **Responsibilities**:
  - Implements ITfTextInputProcessor and related TSF interfaces.
  - Registers as a Windows input method via COM.
  - Captures keystrokes and forwards them to the Rust engine via FFI.
  - Receives candidate lists from the Rust engine and renders the candidate window.
  - Commits selected text back to the target application via TSF.

### 3.2 Rust Core Engine

- **Output**: `maidOS_ime.dll` (cdylib)
- **Responsibilities**:
  - Input scheme logic for all six input methods.
  - Dictionary loading, indexing, and candidate lookup.
  - Bopomofo key-to-syllable mapping and syllable-to-character resolution.
  - Pinyin segmentation and lookup.
  - Traditional/Simplified character conversion.
  - English prefix trie search.
  - Japanese romaji-to-kana conversion and kanji lookup.
  - LLM bridge: formats context, calls Ollama HTTP API, parses re-ranked results.
  - Exposes `extern "C"` FFI surface for both C++ and C# callers.

### 3.3 C# Manager Layer

- **Output**: `MAIDOS.IME.Manager.exe` (.NET 8.0)
- **Responsibilities**:
  - User-facing settings UI (scheme selection, dictionary paths, LLM toggle).
  - Dictionary update and download management.
  - COM DLL registration/unregistration helper.
  - User dictionary import/export.
  - Calls Rust engine via P/Invoke for dictionary operations.

## 4. LLM Integration Layer

- **Runtime**: Ollama server running locally on port 11434.
- **Model**: Configurable (default: lightweight model suitable for character ranking).
- **Protocol**: HTTP POST to `http://127.0.0.1:11434/api/generate`.
- **Timeout**: 2-second hard timeout with graceful fallback to dictionary ranking.
- **Data Sent**: Surrounding text context (up to 200 characters) and top-10 candidates.
- **Data Received**: Re-ordered candidate list with confidence scores.

## 5. Dictionary Data Layer

- **Storage**: JSON files in `{install_dir}/data/dictionaries/`.
- **Cache**: Binary index files generated on first load, stored alongside JSON sources.
- **Update**: Managed by C# Manager; downloads from MAIDOS repository.
- **User Dictionary**: Per-user file in `%AppData%\MAIDOS\IME\user_dict.json`.

## 6. FFI Boundaries

| Boundary | Caller | Callee | ABI | Details |
|----------|--------|--------|-----|---------|
| TSF-to-Engine | C++ DLL | Rust DLL | C ABI | Key events in, candidates out |
| Manager-to-Engine | C# EXE | Rust DLL | C ABI (P/Invoke) | Config and dictionary ops |

See CONTRACT.md for detailed FFI function signatures.

## 7. Deployment Topology

Single-machine deployment. All components install to `C:\Program Files\MAIDOS\IME\`.
COM DLL is registered system-wide. Rust DLL and dictionaries are co-located.
Ollama is a user-managed dependency (not bundled).

## 8. References

- CONTRACT.md - FFI contract details
- STATE_MODEL.md - Engine state machine
- SPEC-MAIDOS-IME-v2.0.md - Full specification
