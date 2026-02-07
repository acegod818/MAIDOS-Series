# MAIDOS-IME -- Service Level Objectives

| Field   | Value        |
|---------|--------------|
| Product | MAIDOS-IME   |
| Version | 0.2.0        |

## Objectives

| SLO-ID  | Metric                | Target   | Window | Measurement            |
|---------|-----------------------|----------|--------|------------------------|
| SLO-001 | Keystroke latency p99 | < 20 ms  | 1 hour | Internal timer in core |
| SLO-002 | Candidate render p95  | < 50 ms  | 1 hour | TSF callback timing    |
| SLO-003 | Crash-free sessions   | > 99.9%  | 7 days | Crash dump count       |
| SLO-004 | Memory usage (idle)   | < 50 MB  | 1 hour | Working set sample     |

## Measurement Method

- Keystroke latency: timestamp at OnKeyDown entry vs candidate list return
- Candidate render: timestamp at Rust return vs TSF paint complete
- Crash-free: Windows Error Reporting / internal crash handler
- Memory: periodic GetProcessMemoryInfo sampling

## Breach Response

| SLO     | Response                                           |
|---------|----------------------------------------------------|
| SLO-001 | Profile hot path; check dictionary index integrity |
| SLO-002 | Reduce candidate count; check GDI handle leaks     |
| SLO-003 | Analyse minidump; hotfix release within 48 h       |
| SLO-004 | Heap profiler; check for leaked COM references      |

*MAIDOS-IME SLO v0.2.0 -- CodeQC Gate C Compliant*
