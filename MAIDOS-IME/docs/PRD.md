# MAIDOS-IME v2.0 - Product Requirements Document

## 1. Overview

MAIDOS-IME is an AI-assisted multilingual Input Method Editor for Windows. It provides
high-performance character input for Chinese (Traditional and Simplified), Japanese, and
English with integrated local LLM inference for intelligent candidate selection.

## 2. Target Users

| Segment | Description |
|---------|-------------|
| Primary | Native Chinese speakers (Traditional/Simplified) on Windows 10/11 |
| Secondary | Japanese speakers requiring romaji-to-kana/kanji input |
| Tertiary | Multilingual professionals switching between CJK and Latin scripts |

## 3. Problem Statement

Existing IMEs (Microsoft IME, Google Japanese Input, RIME) lack unified multilingual
support within a single engine, require cloud connectivity for smart predictions, and
offer limited customization. Users who need privacy-respecting AI assistance for
character disambiguation have no viable local-only option.

## 4. Competitive Analysis

| Feature | Microsoft IME | Google IME | RIME | MAIDOS-IME |
|---------|--------------|------------|------|------------|
| Bopomofo | Yes | No | Plugin | Yes |
| Pinyin | Yes | Yes | Yes | Yes |
| Cangjie | Yes | No | Plugin | Yes |
| Wubi | Yes | No | Plugin | Yes |
| Japanese | Separate IME | Separate | Plugin | Integrated |
| AI Assist | Cloud-only | Cloud-only | None | Local LLM |
| Privacy | Telemetry | Telemetry | Full | Full |
| Open Engine | No | No | Yes | Partial |

## 5. Product Goals

- G-001: Provide six input schemes (Bopomofo, Cangjie, Wubi, Pinyin, English, Japanese)
  in a single IME installation.
- G-002: Achieve key-to-candidate latency under 50 ms for dictionary lookups.
- G-003: Integrate Ollama-based local LLM for context-aware candidate ranking.
- G-004: Support Windows 10 1903+ and Windows 11 via TSF COM integration.
- G-005: Keep memory footprint below 80 MB during normal operation.

## 6. Functional Requirements Summary

| ID | Title | Priority |
|----|-------|----------|
| FR-001 | Input scheme management | P0 |
| FR-002 | Bopomofo input | P0 |
| FR-003 | Pinyin input | P0 |
| FR-004 | Traditional/Simplified conversion | P1 |
| FR-005 | English prefix completion | P1 |
| FR-006 | Japanese romaji input | P1 |
| FR-007 | LLM smart candidate selection | P2 |
| FR-008 | TSF integration | P0 |

## 7. Success Metrics

- 95th percentile input latency under 50 ms (non-AI path).
- AI-assisted selection accuracy above 85% on standard test corpus.
- Zero critical defects at GA release.
- Installation success rate above 99% on supported OS versions.

## 8. Out of Scope

- Mobile or macOS platform support.
- Cloud-based LLM inference.
- Handwriting or voice input recognition.

## 9. References

- SPEC-MAIDOS-IME-v2.0.md
- Microsoft TSF Documentation (MSDN)
- Ollama API Specification
