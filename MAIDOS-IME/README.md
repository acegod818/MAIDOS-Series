# MAIDOS-IME

Multi-language input method engine with AI-assisted character selection, supporting Bopomofo, Pinyin, Cangjie, Quick, Wubi, English, and Japanese.

> **Spec**: See [`SPEC-MAIDOS-IME-v2.0.md`](SPEC-MAIDOS-IME-v2.0.md) for the full AC-001~AC-017 specification.

## Architecture

```
C++ TSF (COM)  -->  Rust cdylib (FFI)  -->  C# AI Layer (optional)
  (Windows)         (maidos_core.dll)        (Ollama / local LLM)
```

- **Rust core** (`src/core/maidos-core/`): input scheme processing, charset conversion, user learning
- **Rust LLM** (`src/core/maidos-llm/`): Ollama HTTP client, local inference
- **Rust config** (`src/core/maidos-config/`): configuration, user dictionary management
- **C++ TSF** (`src/MAIDOS.IME.Core/`): Windows Text Services Framework COM integration
- **C# AI** (`src/MAIDOS.IME.AI/`): .NET AI manager layer
- **FFI layer** (`src/core/maidos-core/src/ffi.rs`): 13 `extern "C"` functions exported from the DLL

## Features

| Feature | Implementation | Status |
|---------|---------------|--------|
| Bopomofo input | Table-driven, 7000+ entries, `include_str!` | Production |
| Pinyin input | Table-driven, ~400 syllables, ~2800 chars | Production |
| Cangjie input | Table-driven, `include_str!` | Production |
| Quick input | Derived from Cangjie (first+last code) | Production |
| Wubi 86 input | Table-driven, `include_str!` | Production |
| English input | Prefix match, 5000+ words | Production |
| Japanese input | Romaji -> Kana -> Kanji, 3000+ terms | Production |
| Traditional <-> Simplified | 500+ character pairs + extended table | Production |
| User learning | Persistent per-scheme, write-through cache | Production |
| AI character selection | Ollama HTTP POST / local LLM | Production |
| Language detection | Character range analysis (CJK/ASCII/JP) | Production |
| Handwriting / Voice | Returns platform error (requires hardware) | Stub |

## Getting Started

### Prerequisites

- **Rust** 1.70+ (stable MSVC toolchain)
- **Windows 10/11** (target platform)
- **Visual Studio 2022** (for C++ TSF project, optional)
- **.NET 8.0 SDK** (for C# AI layer, optional)

### Build

```bash
# Build the native Rust DLL
cargo build --release
# Output: target/release/maidos_core.dll

# Run tests (83 tests)
cargo test

# Lint
cargo clippy
```

### Usage (FFI)

The DLL exports 13 C-compatible functions:

| Export | Purpose |
|--------|---------|
| `ime_process_input` | Process input through a scheme, return candidates JSON |
| `ime_get_candidates` | Get candidate list for input |
| `ime_convert_charset` | Traditional <-> Simplified conversion |
| `ime_supported_schemes` | Return JSON list of 9 supported schemes |
| `ime_detect_language` | Detect input language (chinese/english/japanese) |
| `ime_version` | Return version JSON |
| `ime_pinyin_lookup` | Direct Pinyin scheme query |
| `ime_init` | Initialize IME core |
| `ime_learn` | Record user selection (persisted to disk) |
| `ime_clear_learned` | Clear all learned data for a scheme |
| `ime_last_error` | Get last error message |
| `ime_free_string` | Free FFI-returned string |
| `ime_clear_error` | Clear last error |

### User Learning

When a user selects a candidate, call `ime_learn(scheme, input_code, character)`.
The selection is persisted immediately to `%LOCALAPPDATA%\MAIDOS\IME\user_{scheme}.json`.
On subsequent lookups, learned entries are merged with builtin candidates (higher frequency = higher priority).

### Input Schemes

All Chinese input schemes use the same table-driven architecture:

```
JSON table (include_str!) --> HashMap<code, Vec<{char, freq}>> --> lookup + merge user learning
```

| Scheme | Data File | Size |
|--------|-----------|------|
| Bopomofo | `bopomofo_table.json` | 94 KB |
| Pinyin | `pinyin_table.json` | 29 KB |
| Cangjie | `cangjie_table.json` | 42 KB |
| Quick | Derived from Cangjie | - |
| Wubi | `wubi_table.json` | 17 KB |
| English | `english_common.json` | 79 KB |
| Japanese | `japanese_kana2kanji.json` | 174 KB |

## Project Structure

```
MAIDOS-IME/
├── Cargo.toml                          # Workspace root
├── src/
│   ├── core/
│   │   ├── maidos-core/               # Rust cdylib + rlib
│   │   │   ├── src/
│   │   │   │   ├── lib.rs            # Crate root
│   │   │   │   ├── ffi.rs            # 13 FFI exports (C ABI)
│   │   │   │   ├── schemes.rs        # All input scheme implementations
│   │   │   │   ├── user_learning.rs   # Persistent user learning
│   │   │   │   ├── converter.rs       # Traditional <-> Simplified
│   │   │   │   ├── pinyin_parser.rs   # Advanced Pinyin parser
│   │   │   │   ├── english.rs         # English prefix matching
│   │   │   │   ├── japanese.rs        # Romaji -> Kana -> Kanji
│   │   │   │   ├── ai.rs             # AI character selection
│   │   │   │   ├── ime_engine.rs      # Engine orchestration
│   │   │   │   └── dictionary.rs      # Dictionary management
│   │   │   └── Cargo.toml
│   │   ├── maidos-llm/                # LLM client library
│   │   │   ├── src/
│   │   │   │   ├── client.rs          # Ollama HTTP POST client
│   │   │   │   ├── providers.rs       # LLM provider trait + Ollama impl
│   │   │   │   ├── local.rs           # Local LLM inference
│   │   │   │   └── models.rs          # Model configuration
│   │   │   └── Cargo.toml
│   │   ├── maidos-config/             # Configuration management
│   │   │   ├── src/
│   │   │   │   ├── config.rs          # TOML config read/write
│   │   │   │   ├── dict.rs            # User dictionary
│   │   │   │   └── model.rs           # Model config
│   │   │   └── Cargo.toml
│   │   └── data/                      # Builtin dictionaries (JSON)
│   ├── MAIDOS.IME.Core/               # C++ TSF implementation
│   ├── MAIDOS.IME.AI/                 # C# AI manager
│   └── platform/                      # Platform-specific code
├── platform/                          # Cross-platform directories
├── tests/                             # Integration tests
├── installer/                         # WiX MSI installer
├── SPEC-MAIDOS-IME-v2.0.md           # Full specification
└── LICENSE                            # MIT
```

## TSF Registration (Windows)

To use as a system-wide input method, the C++ TSF DLL must be registered:

1. Build `MAIDOS.IME.Core.dll` (C++ COM DLL)
2. Register COM: `regsvr32 MAIDOS.IME.Core.dll`
3. The DLL registers itself as a TSF Text Input Processor under `HKLM\SOFTWARE\Microsoft\CTF\TIP\`
4. Users can then enable it in Settings > Time & Language > Language > Keyboard

The MSI installer handles registration automatically.

## License

MIT
