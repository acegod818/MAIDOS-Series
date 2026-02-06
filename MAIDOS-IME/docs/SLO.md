# MAIDOS-IME v2.0 - Service Level Objectives

## 1. Purpose

This document defines the Service Level Objectives (SLOs) for MAIDOS-IME. These targets
guide development priorities and provide measurable quality benchmarks.

## 2. SLO Definitions

### SLO-001: Input Latency (Dictionary Path)

- **Objective**: 95th percentile key-to-candidate latency shall not exceed 50 ms.
- **Measurement Window**: Rolling 5-minute window during active input.
- **Data Source**: Instrumented timer in Rust engine (`ime_process_key` duration).
- **Related NFR**: NFR-001.
- **Exclusions**: First keystroke after dictionary load (cold cache).

### SLO-002: Input Latency (AI Path)

- **Objective**: 95th percentile end-to-end latency with AI re-ranking shall not
  exceed 2000 ms.
- **Measurement Window**: Per AI-assisted candidate generation event.
- **Data Source**: Round-trip timer from Rust engine Ollama call.
- **Related NFR**: NFR-002.
- **Fallback**: If exceeded, engine reverts to dictionary-only ranking transparently.

### SLO-003: Availability (Session Stability)

- **Objective**: The IME shall remain functional for 99.9% of active input sessions
  without requiring restart or re-registration.
- **Definition of Failure**: An event where the user must manually re-enable the IME
  in Windows Settings or restart the Manager.
- **Measurement**: Failure count per 1000 input sessions in telemetry (opt-in, local).
- **Related NFR**: NFR-006.

### SLO-004: Memory Stability

- **Objective**: Memory growth over a 60-minute active session shall not exceed 10 MB
  above baseline.
- **Baseline**: Working set measured 30 seconds after engine initialization.
- **Measurement**: Peak working set delta at session end.
- **Related NFR**: NFR-003.

### SLO-005: Dictionary Load Time

- **Objective**: Warm dictionary load (from binary cache) shall complete within 500 ms.
- **Measurement**: Time from `ime_init` call to first successful `ime_process_key`.
- **Cold Load**: First load after install or dictionary update may take up to 3 seconds
  for index generation. This is not covered by this SLO.

### SLO-006: AI Accuracy

- **Objective**: AI-assisted candidate selection shall promote the contextually correct
  candidate to the top-3 position in at least 85% of test corpus queries.
- **Measurement**: Evaluated against standard 500-query test corpus.
- **Scope**: Measured during release QA, not in production.

## 3. SLO Summary Table

| SLO | Metric | Target | Window |
|-----|--------|--------|--------|
| SLO-001 | Dictionary input p95 latency | < 50 ms | Rolling 5 min |
| SLO-002 | AI input p95 latency | < 2000 ms | Per event |
| SLO-003 | Session stability | 99.9% | Per 1000 sessions |
| SLO-004 | Memory growth | < 10 MB / 60 min | Per session |
| SLO-005 | Warm load time | < 500 ms | Per init |
| SLO-006 | AI accuracy (top-3) | >= 85% | QA corpus |

## 4. Error Budget

For SLO-003 (99.9% session stability): out of every 1000 sessions, the error budget
allows at most 1 session failure. Exceeding this budget triggers a stability-focused
development sprint.

## 5. Monitoring

- Engine performance metrics are written to structured log files.
- C# Manager aggregates metrics and displays them in Settings > Performance.
- No external telemetry is transmitted; all data remains local.

## 6. References

- NFR.md - Non-functional requirements
- ALERTS.md - Alert conditions derived from SLO breaches
