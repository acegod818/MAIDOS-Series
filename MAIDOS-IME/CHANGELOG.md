# Changelog

All notable changes to MAIDOS-IME will be documented in this file.

## [0.2.0] - 2026-02-06

### Added
- **Pinyin input scheme (AC-003)**: Table-driven with ~400 syllables, ~2800 character entries, `include_str!` compile-time embedding
- **User learning (AC-010)**: Persistent per-scheme learning at `%LOCALAPPDATA%\MAIDOS\IME\user_{scheme}.json`, write-through cache, frequency boosting on selection
- **FFI: `ime_learn`**: Record user selection and persist to disk immediately
- **FFI: `ime_clear_learned`**: Clear all learned data for a scheme
- **Real Ollama provider**: HTTP POST to `/api/generate` with TCP availability check (replaces mock)

### Changed
- All input schemes unified to table-driven architecture (`include_str!` + HashMap lookup + user learning merge)
- PinyinScheme: from external file loading (parser: None = empty) to builtin table (always works)
- OllamaProvider: from hardcoded mock response to real HTTP POST
- `is_available()`: from always-true to actual TCP connect check
- All Chinese comments/strings translated to English
- FFI count: 11 -> 13 exports

### Fixed
- Pinyin `new_default()` returning empty candidates (now uses builtin table)
- Clippy warnings resolved (0 errors, 0 warnings)

## [0.1.0] - 2026-02-04

### Added
- Initial project structure: Rust workspace (maidos-core, maidos-llm, maidos-config)
- 9 input schemes: Bopomofo, Pinyin, Cangjie, Quick, Wubi, Handwriting, Voice, English, Japanese
- Bopomofo table (94 KB, 7000+ entries) with fallback minimal table
- Cangjie table (42 KB) with fallback
- Wubi table (17 KB) with fallback
- English dictionary (79 KB, 5000+ words)
- Japanese Kana-to-Kanji dictionary (174 KB, 3000+ terms)
- Traditional <-> Simplified conversion (500+ character pairs)
- 11 FFI exports (C ABI)
- C++ TSF integration layer
- C# AI manager layer
- Ollama LLM client (HTTP + local inference)
- Language detection (CJK/ASCII/Japanese)
- SPEC v2.0 document (AC-001~AC-017)
