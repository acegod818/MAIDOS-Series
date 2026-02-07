# MAIDOS-Forge Service Level Objectives

| Field   | Value                                     |
|---------|-------------------------------------------|
| Product | MAIDOS-Forge                              |
| Version | 3.0                                       |
| Type    | Local CLI tool -- SLOs apply per-invocation|

## 1. Overview

These SLOs define the performance and reliability targets for MAIDOS-Forge as a local CLI tool. Measurements are taken on a reference machine (8-core CPU, 16 GB RAM, SSD) unless otherwise noted.

## 2. Parser Latency

| Metric                  | Target           |
|-------------------------|------------------|
| P50 parse time          | < 3 ms           |
| P99 parse time          | < 10 ms          |
| Scope                   | Single file, < 10,000 lines |

**Measurement**: Time from `ILanguagePlugin.ExtractInterfaceAsync()` call to return, excluding disk I/O for file read.

**Exclusions**: Files exceeding 10,000 lines are not covered by this SLO. Performance degrades linearly with file size beyond this threshold.

## 3. Compilation Overhead

| Metric                          | Target           |
|---------------------------------|------------------|
| Forge overhead above native     | < 500 ms         |
| Scope                           | Per-module build  |

**Definition**: Total Forge invocation time minus the time spent inside the native compiler process. This measures the cost of Forge's orchestration, plugin dispatch, interface extraction, and glue generation.

**Measurement**: `forge build --timings` reports Forge overhead separately from compiler time.

## 4. Memory Usage

| Metric                          | Target           |
|---------------------------------|------------------|
| Resident memory (single file)   | < 100 MB         |
| Resident memory (full project)  | < 500 MB         |
| Scope                           | Compilation phase |

**Measurement**: Peak RSS during a build operation.

**Notes**: Memory usage scales with `parallel_jobs`. The single-file target assumes `parallel_jobs = 1`. The full-project target assumes default parallelism.

## 5. Reliability

| Metric                          | Target                        |
|---------------------------------|-------------------------------|
| Crash rate on valid input       | 0 (zero crashes)              |
| Graceful error on invalid input | 100% (always returns error)   |
| Exit code correctness           | 100% (correct code for cause) |

**Definitions**:
- **Valid input**: Source files that compile successfully with the native compiler, paired with a valid `forge.toml`.
- **Crash**: Unhandled exception, segfault, or process abort. A non-zero exit code with a structured error message is NOT a crash.
- **Graceful error**: Forge exits with a documented exit code (see RUNBOOK.md) and a human-readable error message on stderr.

## 6. Startup Time

| Metric                  | Target           |
|-------------------------|------------------|
| CLI cold start          | < 2 s            |
| CLI warm start          | < 500 ms         |

**Definitions**:
- **Cold start**: First invocation after system boot, no OS file cache.
- **Warm start**: Subsequent invocation with OS file cache populated.

**Measurement**: Time from process start to the first line of output (e.g., "Forge 3.0.x -- checking toolchains...").

## 7. Plugin Load Time

| Metric                          | Target           |
|---------------------------------|------------------|
| Per-plugin load time            | < 200 ms         |
| Total plugin discovery          | < 1 s (up to 97) |

**Measurement**: Time to scan `plugins/` directory, load assemblies, and call `GetCapabilities()` on each.

## 8. How to Measure

### Built-in Timing

```bash
forge build --timings
```

Reports:
- Total wall time
- Per-module compile time
- Plugin load time
- Forge overhead

### External Profiling

```bash
# Linux
/usr/bin/time -v forge build

# Windows (PowerShell)
Measure-Command { forge build }
```

### Memory Profiling

```bash
# Linux
/usr/bin/time -v forge build 2>&1 | grep "Maximum resident"

# Windows (PowerShell)
(Get-Process forge | Select-Object -Property PeakWorkingSet64).PeakWorkingSet64 / 1MB
```

## 9. SLO Violation Handling

Since Forge is a local CLI tool, SLO violations are not paged to an on-call team. Instead:

1. **Detection**: Users observe slow builds or high memory usage.
2. **Diagnosis**: Run `forge build --timings --verbose` and file a bug report with the output.
3. **Resolution**: Performance regressions are treated as P1 bugs in the issue tracker.
4. **CI Gate**: The test suite includes benchmark tests that fail if SLO thresholds are exceeded on the CI runner.
