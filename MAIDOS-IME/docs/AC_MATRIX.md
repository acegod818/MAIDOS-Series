# MAIDOS-IME v2.0 - Acceptance Criteria Matrix

## 1. Purpose

This document maps all acceptance criteria (AC-001 through AC-017) to their parent
functional requirements, test type, and pass criteria.

## 2. Matrix

| AC | FR | Title | Test Type | Pass Criteria |
|----|-----|-------|-----------|---------------|
| AC-001 | FR-001 | List available schemes | Unit | API returns all 6 schemes with correct metadata |
| AC-002 | FR-001 | Switch active scheme | Integration | Scheme transitions correctly; composition buffer cleared |
| AC-003 | FR-001 | Persist scheme preference | Integration | After restart, last-used scheme is restored from config |
| AC-004 | FR-002 | Bopomofo key mapping | Unit | All 37 Zhuyin symbols map correctly from keyboard layout |
| AC-005 | FR-002 | Bopomofo candidate generation | Unit | Input "ㄇㄚ" returns candidates including common characters |
| AC-006 | FR-003 | Pinyin syllable segmentation | Unit | "nihao" segments to ["ni", "hao"] correctly |
| AC-007 | FR-003 | Pinyin candidate lookup | Unit | Segmented syllables return ranked character candidates |
| AC-008 | FR-004 | Traditional to Simplified | Unit | Conversion of 100-char test string matches reference output |
| AC-009 | FR-004 | Simplified to Traditional | Unit | Reverse conversion of 100-char test string matches reference |
| AC-010 | FR-005 | English prefix lookup | Unit | Prefix "prog" returns ["program", "progress", ...] from dictionary |
| AC-011 | FR-005 | English Tab completion | Integration | Tab key commits the top suggestion into the application |
| AC-012 | FR-006 | Romaji to Hiragana | Unit | "ka" converts to hiragana; all basic kana covered |
| AC-013 | FR-006 | Kanji candidate lookup | Unit | Hiragana input yields ranked kanji candidates from dictionary |
| AC-014 | FR-007 | LLM candidate re-ranking | Integration | With Ollama running, candidate order changes based on context |
| AC-015 | FR-007 | LLM timeout fallback | Integration | When Ollama exceeds 2 s, original ranking is returned |
| AC-016 | FR-007 | LLM unavailable fallback | Integration | When Ollama is offline, input proceeds without error |
| AC-017 | FR-008 | TSF COM registration | System | DLL registers via regsvr32; IME appears in Windows settings |

## 3. Test Type Definitions

| Type | Description | Gate |
|------|-------------|------|
| Unit | Isolated function-level test in Rust or C# | G1 |
| Integration | Cross-layer test (C++ TSF + Rust, or Rust + C#) | G2 |
| System | Full end-to-end test on Windows with real TSF | G4 |

## 4. Coverage Summary

- **FR-001** (Scheme Management): 3 ACs (AC-001, AC-002, AC-003)
- **FR-002** (Bopomofo): 2 ACs (AC-004, AC-005)
- **FR-003** (Pinyin): 2 ACs (AC-006, AC-007)
- **FR-004** (Conversion): 2 ACs (AC-008, AC-009)
- **FR-005** (English): 2 ACs (AC-010, AC-011)
- **FR-006** (Japanese): 2 ACs (AC-012, AC-013)
- **FR-007** (LLM): 3 ACs (AC-014, AC-015, AC-016)
- **FR-008** (TSF): 1 AC (AC-017)

**Total**: 17 acceptance criteria covering 8 functional requirements.

## 5. Notes

- All unit tests (G1) must pass before integration tests (G2) are executed.
- System tests (G4) require a Windows 10/11 environment with TSF enabled.
- LLM-related ACs (AC-014 through AC-016) require Ollama installed for positive tests;
  negative tests (timeout, unavailable) run without Ollama.
