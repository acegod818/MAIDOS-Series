# MAIDOS-IME v2.0 - Non-Functional Requirements

## 1. Purpose

This document defines the non-functional requirements (NFRs) that govern performance,
reliability, security, and quality standards for MAIDOS-IME.

## 2. Performance

### NFR-001: Key-to-Candidate Latency

- **Requirement**: From keystroke receipt to candidate list display shall be under 50 ms.
- **Measurement**: Instrumented timer in Rust engine, p95 over 1000-keystroke test run.
- **Rationale**: Users perceive input lag above 100 ms; 50 ms provides comfortable margin.

### NFR-002: AI Inference Latency

- **Requirement**: LLM-assisted candidate re-ranking shall complete within 2 seconds.
- **Timeout**: If Ollama response exceeds 2 s, fall back to dictionary-only ranking.
- **Measurement**: Round-trip time from Rust engine to Ollama and back.
- **Rationale**: AI is supplementary; it must not block the primary input flow.

### NFR-003: Memory Usage

- **Requirement**: Resident memory shall remain below 80 MB during normal operation.
- **Measurement**: Peak working set measured via Windows Performance Monitor over 30 min.
- **Includes**: Rust engine, loaded dictionaries, candidate buffers.
- **Excludes**: Ollama server process (managed externally).

### NFR-004: CPU Usage at Idle

- **Requirement**: CPU usage shall remain below 1% when no input is active.
- **Measurement**: Average CPU over 60 s idle period via Performance Monitor.
- **Rationale**: IME runs continuously; idle overhead must be negligible.

## 3. Compatibility

### NFR-005: Operating System Support

- **Requirement**: Fully supported on Windows 10 version 1903+ and Windows 11.
- **TSF Version**: Text Services Framework 2.0 and above.
- **Architecture**: x86-64 only (ARM64 deferred to future release).
- **Validation**: Installation and basic input tests on both OS versions.

## 4. Reliability

### NFR-006: Sustained Input Stability

- **Requirement**: The engine shall process 1000 consecutive characters without crash,
  hang, or memory leak exceeding 5 MB growth.
- **Test**: Automated input replay of 1000-character mixed-scheme corpus.
- **Pass Criteria**: Zero unhandled exceptions, memory delta under 5 MB.

## 5. Security

### NFR-007: LLM Communication Isolation

- **Requirement**: All LLM inference requests shall be sent to localhost (127.0.0.1) only.
- **Enforcement**: Hardcoded localhost binding in Rust HTTP client; no configurable endpoint.
- **Rationale**: User input text must never leave the local machine.
- **Audit**: Code review gate verifying no external network calls in engine.

## 6. Quality

### NFR-008: Test Coverage

- **Rust Engine**: Line coverage shall be at minimum 70%, measured by `cargo-tarpaulin`.
- **C# Manager**: Line coverage shall be at minimum 50%, measured by `coverlet`.
- **C++ TSF Layer**: Manual test coverage via integration tests (COM not easily unit-tested).
- **Gate**: Coverage check enforced in CI pipeline; build fails below threshold.

## 7. Traceability

| NFR | Related FR | Verified By |
|-----|-----------|-------------|
| NFR-001 | FR-002, FR-003 | Performance benchmark (G3) |
| NFR-002 | FR-007 | Timeout integration test (G2) |
| NFR-003 | All | Memory profiling (G3) |
| NFR-004 | FR-008 | Idle CPU test (G3) |
| NFR-005 | FR-008 | OS compatibility matrix (G4) |
| NFR-006 | All | Stress test (G3) |
| NFR-007 | FR-007 | Security code review (G1) |
| NFR-008 | All | CI coverage gate (G1) |
