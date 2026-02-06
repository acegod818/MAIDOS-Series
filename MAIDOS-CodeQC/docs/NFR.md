# MAIDOS-CodeQC -- Non-Functional Requirements

> **Version**: 1.0
> **Date**: 2026-02-07
> **Product**: MAIDOS-CodeQC v2.6.1

---

## 1. Performance

| Requirement | Target | Measurement |
|:------------|:-------|:------------|
| NFR-PERF-001 | Scan 10,000 LOC in under 5 seconds | Wall-clock time, single-threaded baseline |
| NFR-PERF-002 | Scan 100,000 LOC in under 30 seconds | With parallel file processing enabled |
| NFR-PERF-003 | Plugin load time under 500ms per plugin | Cold start, measured from dlopen to ready |
| NFR-PERF-004 | Report generation under 1 second | For JSON and console; HTML under 3 seconds |

## 2. Memory

| Requirement | Target |
|:------------|:-------|
| NFR-MEM-001 | Peak memory usage below 200 MB for a 100K LOC scan |
| NFR-MEM-002 | Idle memory footprint below 30 MB (core + no plugins loaded) |
| NFR-MEM-003 | Per-plugin overhead below 15 MB |

## 3. Reliability

| Requirement | Description |
|:------------|:------------|
| NFR-REL-001 | Graceful degradation when a plugin fails to load |
| NFR-REL-002 | No data loss on interrupted scans; partial results preserved |
| NFR-REL-003 | Deterministic output: same input always produces same report |

## 4. Portability

| Requirement | Description |
|:------------|:------------|
| NFR-PORT-001 | Runs on Windows 10+, Ubuntu 20.04+, macOS 12+ |
| NFR-PORT-002 | Single static binary with no runtime dependencies beyond OS |
| NFR-PORT-003 | Plugin binaries are per-platform; distributed as shared libraries |

## 5. Extensibility

| Requirement | Description |
|:------------|:------------|
| NFR-EXT-001 | Plugin hot-loading: add or replace plugins without restarting the engine |
| NFR-EXT-002 | Custom rule definitions via TOML configuration files |
| NFR-EXT-003 | Plugin API is versioned; backward-compatible within a major version |

## 6. Security

| Requirement | Description |
|:------------|:------------|
| NFR-SEC-001 | Plugins are loaded from a trusted directory only |
| NFR-SEC-002 | No network access required for offline scanning |
| NFR-SEC-003 | Config files validated against schema before use |

## 7. Usability

| Requirement | Description |
|:------------|:------------|
| NFR-USE-001 | CLI follows POSIX conventions; `--help` on every subcommand |
| NFR-USE-002 | Exit codes: 0 = pass, 1 = violations found, 2 = tool error |
| NFR-USE-003 | Colored console output with `--no-color` fallback |

---

*All NFRs are verified through the benchmark suite in `tests/bench/`.*
