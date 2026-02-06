# MAIDOS-IME v2.0 - Architecture Decision Records

## 1. Purpose

This document records key architecture decisions for MAIDOS-IME, including context,
options considered, decision rationale, and consequences.

---

## ADR-001: Rust for Core Engine

- **Status**: Accepted
- **Date**: 2026-01-15
- **Context**: The IME engine processes every keystroke in real time. It must be fast,
  memory-safe, and capable of loading large dictionaries without GC pauses.
- **Options Considered**:
  1. C++ - Maximum performance but manual memory management risk.
  2. Rust - Near-C++ performance with compile-time memory safety.
  3. C# - Managed runtime but GC pauses unacceptable for keystroke processing.
- **Decision**: Rust, compiled as cdylib for FFI compatibility with both C++ and C#.
- **Rationale**: Rust provides zero-cost abstractions, no GC pauses, and strong safety
  guarantees. The `cdylib` output integrates cleanly with C++ via C ABI and C# via P/Invoke.
- **Consequences**: Team must maintain Rust expertise. Build toolchain adds complexity.
  FFI boundary requires careful lifetime and encoding management.

---

## ADR-002: C++ for TSF Layer

- **Status**: Accepted
- **Date**: 2026-01-15
- **Context**: Windows Text Services Framework requires a COM DLL implementing specific
  COM interfaces (ITfTextInputProcessor, ITfKeyEventSink, etc.).
- **Options Considered**:
  1. C++ with ATL/WTL - Native COM support, well-documented TSF examples.
  2. Rust with windows-rs - Possible but TSF COM interfaces are poorly supported.
  3. C# with COM interop - Heavy runtime overhead for a system-level hook.
- **Decision**: C++ with ATL for COM interface implementation.
- **Rationale**: TSF is a COM-first API. C++ with ATL provides the most straightforward
  and well-tested path. Microsoft sample code and documentation target C++.
- **Consequences**: Thin C++ layer (under 2000 LOC) that delegates all logic to Rust.
  C++ layer handles only TSF plumbing and candidate window rendering.

---

## ADR-003: Ollama for Local LLM Inference

- **Status**: Accepted
- **Date**: 2026-01-20
- **Context**: AI-assisted candidate selection requires an LLM. User privacy mandates
  that no input text leaves the local machine.
- **Options Considered**:
  1. Cloud API (OpenAI/Claude) - Best models but sends user input to external servers.
  2. Embedded ONNX model - No external dependency but limited model flexibility.
  3. Ollama local server - Wide model selection, simple HTTP API, user manages install.
- **Decision**: Ollama running on localhost:11434.
- **Rationale**: Ollama provides a clean HTTP API, supports multiple models, and keeps
  all inference local. Users can choose model size based on their hardware.
- **Consequences**: Ollama is an optional user-managed dependency. IME must function
  fully without it (graceful degradation). 2-second timeout prevents blocking.

---

## ADR-004: JSON for Dictionary Format

- **Status**: Accepted
- **Date**: 2026-01-22
- **Context**: MAIDOS-IME ships multiple dictionaries (Bopomofo, Pinyin, Cangjie, Wubi,
  English, Japanese). Community contributions to dictionaries are desired.
- **Options Considered**:
  1. SQLite - Fast queries but opaque to contributors, merge conflicts in binary.
  2. Binary format - Fastest load but not human-editable.
  3. JSON - Human-readable, diffable, easy to contribute to.
  4. TSV - Simple but limited structure for nested data (phrases, metadata).
- **Decision**: JSON as source format with binary index cache generated at first load.
- **Rationale**: JSON is universally readable, supports nested structures for phrase
  entries, and diffs cleanly in version control. Binary cache eliminates load-time penalty
  after first run.
- **Consequences**: First launch after dictionary update incurs a one-time indexing cost
  (estimated 1-3 seconds). JSON files are larger on disk than binary alternatives.
