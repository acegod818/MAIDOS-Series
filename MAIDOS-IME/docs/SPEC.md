# SPEC — MAIDOS-IME v0.3.0

## Overview

This specification document maps all product features to their corresponding test implementations, defines interface contracts, non-functional requirements, and failure modes for MAIDOS-IME.

**Product**: AI-Powered Input Method Engine
**Architecture**: Rust core (maidos-core) + Windows TSF backend + C# UI via FFI
**Version**: 0.3.0
**Test Coverage**: 22 test modules with comprehensive coverage

---

## Feature → Test Mapping

### Module M1: Pinyin Input Processing

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-001 | Pinyin syllable parsing | `maidos-core::pinyin_parser` | `src/core/maidos-core/src/pinyin_parser.rs` | `test_parse_single_pinyin` |
| F-002 | Multi-syllable segmentation | `maidos-core::pinyin_parser` | `src/core/maidos-core/src/pinyin_parser.rs` | `test_parse_multiple_pinyin` |
| F-003 | Pinyin→Chinese conversion | `maidos-core::pinyin_parser` | `tests/pinyin_parser_test.rs` | `test_pinyin_to_hanzi` |
| F-004 | Fuzzy pinyin matching | `maidos-core::pinyin_parser` | `src/core/maidos-core/src/pinyin_parser.rs` | `test_fuzzy_matching` |
| F-005 | Tone mark handling | `maidos-core::pinyin_parser` | `src/core/maidos-core/src/pinyin_parser.rs` | `test_tone_marks` |

### Module M2: Dictionary Management

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-006 | Dictionary loading from JSON | `maidos-core::dictionary` | `src/core/maidos-core/src/dictionary.rs` | `test_load_dictionary` |
| F-007 | Dictionary lookup by pronunciation | `maidos-core::dictionary` | `tests/dictionary_test.rs` | `test_lookup_pronunciation` |
| F-008 | Frequency-based candidate sorting | `maidos-core::dictionary` | `src/core/maidos-core/src/dictionary.rs` | `test_frequency_sorting` |
| F-009 | Custom dictionary merging | `maidos-core::dictionary` | `src/core/maidos-core/src/dictionary.rs` | `test_custom_dict_merge` |
| F-010 | Dictionary entry addition | `maidos-core::dictionary` | `tests/dictionary_test.rs` | `test_add_entry` |

### Module M3: AI-Powered Character Selection

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-011 | Context-aware prediction | `maidos-core::ai` | `src/core/maidos-core/src/ai.rs` | `test_context_prediction` |
| F-012 | LLM-based character ranking | `maidos-core::ai` | `src/core/maidos-core/src/ai.rs` | `test_llm_ranking` |
| F-013 | Auto-correction | `maidos-core::ime_engine` | `src/core/maidos-core/src/ime_engine.rs` | `test_auto_correction` |
| F-014 | Smart suggestions | `maidos-core::ime_engine` | `src/core/maidos-core/src/ime_engine.rs` | `test_smart_suggestions` |
| F-015 | Historical context tracking | `maidos-core::ai` | `src/core/maidos-core/src/ai.rs` | `test_context_history` |

### Module M4: LLM Integration

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-016 | Ollama local model support | `maidos-llm::local` | `src/core/maidos-llm/src/local.rs` | `test_ollama_connection` |
| F-017 | OpenAI API integration | `maidos-llm::client` | `src/core/maidos-llm/src/client.rs` | `test_openai_completion` |
| F-018 | Request/response serialization | `maidos-llm::models` | `src/core/maidos-llm/src/models.rs` | `test_request_serde` |
| F-019 | Multi-provider fallback | `maidos-llm::client` | `src/core/maidos-llm/src/client.rs` | `test_provider_fallback` |
| F-020 | Token limit enforcement | `maidos-llm::lib` | `src/core/maidos-llm/src/lib.rs` | `test_token_limits` |
| F-021 | Extended LLM coverage | `maidos-llm::tests` | `src/core/maidos-llm/src/tests/extended_coverage.rs` | `test_extended_scenarios` |

### Module M5: Multi-Language Support

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-022 | English input processing | `maidos-core::english` | `src/core/maidos-core/src/english.rs` | `test_english_input` |
| F-023 | English auto-completion | `maidos-core::english` | `src/core/maidos-core/src/english.rs` | `test_english_completion` |
| F-024 | Japanese Hiragana conversion | `maidos-core::japanese` | `src/core/maidos-core/src/japanese.rs` | `test_hiragana_conversion` |
| F-025 | Japanese Katakana conversion | `maidos-core::japanese` | `src/core/maidos-core/src/japanese.rs` | `test_katakana_conversion` |
| F-026 | Japanese Kanji lookup | `maidos-core::japanese` | `src/core/maidos-core/src/japanese.rs` | `test_kanji_lookup` |

### Module M6: Input Schemes

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-027 | Full Pinyin scheme | `maidos-core::schemes` | `src/core/maidos-core/src/schemes.rs` | `test_full_pinyin_scheme` |
| F-028 | Shuangpin scheme | `maidos-core::schemes` | `src/core/maidos-core/src/schemes.rs` | `test_shuangpin_scheme` |
| F-029 | Wubi scheme | `maidos-core::schemes` | `src/core/maidos-core/src/schemes.rs` | `test_wubi_scheme` |
| F-030 | Scheme switching | `maidos-core::schemes` | `src/core/maidos-core/src/schemes.rs` | `test_scheme_switching` |
| F-031 | Custom scheme loading | `maidos-core::schemes` | `src/core/maidos-core/src/schemes.rs` | `test_custom_scheme` |

### Module M7: User Learning

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-032 | User input history tracking | `maidos-core::user_learning` | `src/core/maidos-core/src/user_learning.rs` | `test_history_tracking` |
| F-033 | Personalized word frequency | `maidos-core::user_learning` | `src/core/maidos-core/src/user_learning.rs` | `test_personalized_frequency` |
| F-034 | Custom phrase learning | `maidos-core::user_learning` | `src/core/maidos-core/src/user_learning.rs` | `test_phrase_learning` |
| F-035 | User dictionary export | `maidos-core::user_learning` | `src/core/maidos-core/src/user_learning.rs` | `test_dict_export` |
| F-036 | Learning data synchronization | `maidos-core::user_learning` | `src/core/maidos-core/src/user_learning.rs` | `test_data_sync` |

### Module M8: Text Conversion

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-037 | Simplified ↔ Traditional Chinese | `maidos-core::converter` | `src/core/maidos-core/src/converter.rs` | `test_simplified_to_traditional` |
| F-038 | Character set filtering | `maidos-core::converter` | `src/core/maidos-core/src/converter.rs` | `test_charset_filtering` |
| F-039 | Full-width ↔ Half-width conversion | `maidos-core::converter` | `src/core/maidos-core/src/converter.rs` | `test_width_conversion` |

### Module M9: Configuration Management

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-040 | Config loading from TOML | `maidos-config::config` | `src/core/maidos-config/src/config.rs` | `test_load_config` |
| F-041 | Config validation | `maidos-config::config` | `src/core/maidos-config/src/config.rs` | `test_validate_config` |
| F-042 | Default config generation | `maidos-config::config` | `src/core/maidos-config/src/config.rs` | `test_default_config` |
| F-043 | Model configuration | `maidos-config::model` | `src/core/maidos-config/src/model.rs` | `test_model_config` |
| F-044 | Dictionary path configuration | `maidos-config::dict` | `src/core/maidos-config/src/dict.rs` | `test_dict_path_config` |

### Module M10: FFI Interface

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-045 | String marshalling (Rust ↔ C#) | `maidos-core::ffi` | `src/core/maidos-core/src/ffi.rs` | `test_string_marshalling` |
| F-046 | IME engine initialization | `maidos-core::ffi` | `src/core/maidos-core/src/ffi.rs` | `test_ime_init` |
| F-047 | Input processing via FFI | `maidos-core::ffi` | `src/core/maidos-core/src/ffi.rs` | `test_ffi_process_input` |
| F-048 | Candidate retrieval via FFI | `maidos-core::ffi` | `src/core/maidos-core/src/ffi.rs` | `test_ffi_get_candidates` |
| F-049 | Memory leak prevention | `maidos-core::ffi` | `src/core/maidos-core/src/ffi.rs` | `test_ffi_memory_safety` |

---

## Interface Contracts

### FFI API (P/Invoke C ABI)

**IME Engine Lifecycle**
```c
// Initialize IME engine with config path
void* ime_init(const char* config_path);

// Shutdown IME engine
void ime_shutdown(void* engine_ptr);
```

**Input Processing**
```c
// Process raw input string
int32_t ime_process_input(void* engine_ptr, const char* input, char** result_ptr);

// Get candidate list
int32_t ime_get_candidates(void* engine_ptr, char*** candidates_ptr);

// Select candidate by index
int32_t ime_select_candidate(void* engine_ptr, int32_t index);
```

**Configuration**
```c
// Set input scheme (pinyin/shuangpin/wubi)
int32_t ime_set_scheme(void* engine_ptr, const char* scheme_name);

// Enable/disable AI features
int32_t ime_set_ai_enabled(void* engine_ptr, int32_t enabled);
```

**Memory Management**
```c
// Free string returned from Rust
void ime_free_string(char* s);

// Free string array
void ime_free_string_array(char** arr, int32_t count);
```

### State Transitions

**IME Engine States**
- `UNINITIALIZED` → `ime_init()` → `READY`
- `READY` → `ime_process_input()` → `COMPOSING`
- `COMPOSING` → `ime_select_candidate()` → `COMMITTED`
- `COMMITTED` → `ime_process_input()` → `COMPOSING` (next input)
- `*` → `ime_shutdown()` → `UNINITIALIZED`

**Input Processing States**
- `EMPTY` - No input buffer
- `COMPOSING` - User typing, candidates available
- `SELECTING` - Candidate menu displayed
- `COMMITTED` - Text finalized and sent to application

**AI Feature States**
- `AI_DISABLED` - Use dictionary-only mode
- `AI_ENABLED` - Use LLM for ranking and suggestions
- `AI_FALLBACK` - LLM unavailable, fallback to dictionary

---

## NFR Mapping

### Performance SLO

| NFR | SLI | SLO Target | Test |
|-----|-----|------------|------|
| NFR-001 | Keystroke latency | ≤ 50ms (p95) | Manual validation + profiling |
| NFR-002 | Dictionary lookup time | ≤ 10ms per query | `test_lookup_pronunciation` |
| NFR-003 | LLM response time | ≤ 500ms (local), ≤ 2000ms (cloud) | `test_ollama_connection`, `test_openai_completion` |
| NFR-004 | Candidate generation time | ≤ 100ms for top 10 candidates | `test_pinyin_to_hanzi` |
| NFR-005 | Memory footprint | ≤ 150 MB (IME process) | Manual validation |
| NFR-006 | Config load time | ≤ 200ms | `test_load_config` |

### Accuracy SLO

| NFR | SLI | SLO Target | Test |
|-----|-----|------------|------|
| NFR-007 | Pinyin parsing accuracy | ≥ 99% for standard pinyin | `test_parse_single_pinyin` |
| NFR-008 | Top-1 candidate accuracy | ≥ 85% (with AI), ≥ 70% (dictionary-only) | Manual A/B testing |
| NFR-009 | Top-5 candidate coverage | ≥ 95% (correct char in top 5) | Manual evaluation |
| NFR-010 | Auto-correction accuracy | ≥ 90% for common typos | `test_auto_correction` |
| NFR-011 | Context prediction accuracy | ≥ 80% for 3-word history | `test_context_prediction` |

### Reliability SLO

| NFR | SLI | SLO Target | Test |
|-----|-----|------------|------|
| NFR-012 | Uptime (no crash) | ≥ 99.9% during 8-hour session | Stress testing |
| NFR-013 | LLM fallback success rate | 100% (graceful degradation) | `test_provider_fallback` |
| NFR-014 | FFI memory safety | Zero memory leaks | `test_ffi_memory_safety` |
| NFR-015 | Config validation coverage | 100% (reject invalid configs) | `test_validate_config` |

### Usability SLO

| NFR | SLI | SLO Target | Test |
|-----|-----|------------|------|
| NFR-016 | Scheme switching latency | ≤ 100ms | `test_scheme_switching` |
| NFR-017 | User dictionary learning effectiveness | ≥ 80% frequency boost for repeated phrases | `test_personalized_frequency` |
| NFR-018 | Multi-language switching | Seamless (no restart required) | Manual validation |

---

## Failure Modes

### Critical Failures

| Mode | Impact | Mitigation | Test |
|------|--------|------------|------|
| FM-001: Dictionary load failure | No candidate suggestions | Use fallback minimal dictionary, log error | `test_load_dictionary` |
| FM-002: LLM connection timeout | Slow AI features | Fallback to dictionary-only mode, cache recent results | `test_provider_fallback` |
| FM-003: Config parsing error | Cannot initialize IME | Use default config, log warning | `test_validate_config` |
| FM-004: FFI null pointer dereference | Crash | Validate pointers, return error codes | `test_ffi_memory_safety` |
| FM-005: Pinyin parser infinite loop | UI freeze | Implement timeout (500ms), return partial results | `test_parse_multiple_pinyin` |
| FM-006: User dictionary corruption | Lost custom phrases | Auto-backup before write, checksum validation | `test_dict_export` |

### Non-Critical Failures

| Mode | Impact | Mitigation | Test |
|------|--------|------------|------|
| FM-007: Invalid tone mark | Incorrect candidate | Treat as tone-less pinyin, log warning | `test_tone_marks` |
| FM-008: Unknown scheme name | Cannot switch scheme | Keep current scheme, show error message | `test_scheme_switching` |
| FM-009: LLM token limit exceeded | Truncated response | Trim context history, retry with shorter prompt | `test_token_limits` |
| FM-010: Frequency data inconsistency | Suboptimal ranking | Re-sort by global frequency, log anomaly | `test_frequency_sorting` |
| FM-011: Network error (cloud LLM) | No AI suggestions | Fallback to local model or dictionary-only | `test_openai_completion` |
| FM-012: Japanese conversion ambiguity | Multiple valid outputs | Present all candidates, let user choose | `test_kanji_lookup` |

### Edge Cases

| Mode | Impact | Mitigation | Test |
|------|--------|------------|------|
| FM-013: Empty input string | No candidates | Return empty candidate list | `test_ffi_process_input` |
| FM-014: Very long input (>100 chars) | Performance degradation | Process in chunks, warn user | `test_parse_multiple_pinyin` |
| FM-015: Unsupported character in pinyin | Parse failure | Skip invalid chars, parse remainder | `test_fuzzy_matching` |
| FM-016: Dictionary merge conflict | Duplicate entries | Use higher-frequency entry, log warning | `test_custom_dict_merge` |
| FM-017: LLM returns non-Chinese output | Unusable suggestions | Filter response, fallback to dictionary | `test_llm_ranking` |
| FM-018: Concurrent FFI calls | Race condition | Use thread-safe engine wrapper (Mutex) | `test_ffi_memory_safety` |
| FM-019: Charset conversion unsupported char | Missing output | Use replacement char (�), log warning | `test_charset_filtering` |
| FM-020: Historical context overflow | Memory growth | Limit history to 50 words, evict oldest | `test_context_history` |

---

## Acceptance Criteria Cross-Reference

All features are mapped to test functions in the Feature → Test Mapping section above. Key module groupings:

- **M1-M2**: Core input processing (F-001 to F-010)
- **M3-M4**: AI-powered features (F-011 to F-021)
- **M5-M6**: Multi-language and schemes (F-022 to F-031)
- **M7-M9**: User learning and config (F-032 to F-044)
- **M10**: FFI interface (F-045 to F-049)

---

## Test Execution Summary

**Total Tests**: 22 test modules with 100+ individual test functions
**Coverage**: All public API functions covered
**Test Types**:
- Unit tests: `src/core/maidos-core/src/*.rs` (inline `#[test]` modules)
- Integration tests: `tests/pinyin_parser_test.rs`, `tests/dictionary_test.rs`
- LLM integration tests: `src/core/maidos-llm/src/tests/extended_coverage.rs`

**Test Commands**:
```bash
# Run all tests
cargo test --workspace

# Run specific module
cargo test -p maidos-core
cargo test -p maidos-llm
cargo test -p maidos-config
```

**Evidence Location**:
- Test reports: `evidence/test_reports/`
- Proof manifest: `proof/manifest.json`
- CodeQC reports: `qc/gate{1-4}_report.log`

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-02-13 | ZGWC_acegod818 | Initial SPEC document for v0.3.0 |

---

**Signature**: ZGWC_acegod818 <wocao@maidos.dev>
**Compliance**: CodeQC v3.0 C gate-checked, 0 errors, 0 warnings
