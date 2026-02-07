# MAIDOS-IME -- Architecture Document

| Field   | Value        |
|---------|--------------|
| Product | MAIDOS-IME   |
| Version | 0.2.0        |

## High-Level Diagram

```
 Windows App (any)
       |
 [ C++ TSF Text Service ]   <-- COM DLL registered with Windows
       | FFI (cdylib)
 [ maidos-core  (Rust) ]    <-- keystroke pipeline, candidate engine
       |
 [ maidos-llm   (Rust) ]    <-- AI completion backend
       |
 [ maidos-config (Rust) ]   <-- user prefs, dictionary paths
```

## Component Responsibilities

| Component     | Language | Role                                          |
|---------------|----------|-----------------------------------------------|
| TSF Front-End | C++      | COM text service; captures keystrokes, renders |
| maidos-core   | Rust     | Pipeline: decode -> lookup -> rank -> emit     |
| maidos-llm    | Rust     | LLM inference adapter (local/remote)           |
| maidos-config | Rust     | Config files, dictionary I/O, user prefs       |

## Data Flow

1. User presses key -> TSF captures `ITfKeyEventSink`
2. TSF calls Rust FFI `process_keystroke(key_code)`
3. maidos-core queries dictionary + maidos-llm
4. Candidate list returned to TSF -> rendered in candidate window
5. User selects candidate -> TSF commits text to application

## Build Artefacts

- `maidos_ime.dll` -- C++ TSF COM DLL
- `maidos_core.dll` -- Rust cdylib consumed by TSF DLL
- Dictionary: `%APPDATA%\MAIDOS\IME\user_dict.db`

*MAIDOS-IME ARCHITECTURE v0.2.0 -- CodeQC Gate C Compliant*
