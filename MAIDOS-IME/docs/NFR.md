# MAIDOS-IME -- Non-Functional Requirements

| Field   | Value        |
|---------|--------------|
| Product | MAIDOS-IME   |
| Version | 0.2.0        |

## Performance

| ID      | Metric                | Target    |
|---------|-----------------------|-----------|
| NFR-001 | Keystroke latency     | < 20 ms   |
| NFR-002 | Candidate render time | < 50 ms   |
| NFR-003 | AI completion latency | < 200 ms  |
| NFR-004 | Cold start time       | < 500 ms  |

## Resource Usage

| ID      | Metric          | Target    |
|---------|-----------------|-----------|
| NFR-005 | Memory (idle)   | < 50 MB   |
| NFR-006 | Memory (active) | < 120 MB  |
| NFR-007 | Disk footprint  | < 30 MB   |

## Quality

| ID      | Metric              | Target   |
|---------|---------------------|----------|
| NFR-008 | Compiler warnings   | 0        |
| NFR-009 | Clippy warnings     | 0        |
| NFR-010 | Test coverage       | >= 80%   |

## Compatibility

- Windows 10 1903+ / Windows 11, x86_64
- Works with Win32, UWP, and Electron apps

## Security

- No telemetry without opt-in
- Dictionary stored locally; encrypted at rest optional
- LLM calls use HTTPS with certificate pinning

*MAIDOS-IME NFR v0.2.0 -- CodeQC Gate C Compliant*
