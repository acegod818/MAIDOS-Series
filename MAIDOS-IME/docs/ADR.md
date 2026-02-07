# MAIDOS-IME -- Architecture Decision Records

| Field   | Value        |
|---------|--------------|
| Product | MAIDOS-IME   |
| Version | 0.2.0        |

---

## ADR-001: Rust for Core Engine

**Status:** Accepted
**Context:** IME core needs sub-20 ms latency with memory safety.
**Decision:** Use Rust for the core pipeline (maidos-core crate).
**Consequences:**
- (+) Memory safety without GC pauses
- (+) Easy FFI export via cdylib
- (-) C++ TSF layer still required for COM interop

---

## ADR-002: C++ TSF for Windows Integration

**Status:** Accepted
**Context:** Windows TSF is COM-based; Rust COM support is immature.
**Decision:** Implement TSF front-end in C++ as a COM DLL calling Rust FFI.
**Consequences:**
- (+) Full TSF API coverage
- (+) Proven pattern (Microsoft samples)
- (-) Two-language build (cmake + cargo)

---

## ADR-003: AI Completion via maidos-llm

**Status:** Accepted
**Context:** Sentence completion improves UX but must not block typing.
**Decision:** Use maidos-llm crate for async LLM inference with 200 ms timeout.
**Consequences:**
- (+) Non-blocking; falls back to dictionary on timeout
- (+) Shared crate reusable across MAIDOS products
- (-) Requires model download or API key setup

*MAIDOS-IME ADR v0.2.0 -- CodeQC Gate C Compliant*
