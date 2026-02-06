# MAIDOS-CodeQC -- Alert Definitions

> **Version**: 1.0
> **Date**: 2026-02-07
> **Product**: MAIDOS-CodeQC v2.6.1

---

## 1. Alert Categories

| Category | Prefix | Severity Range |
|:---------|:-------|:---------------|
| Scan Failure | ALT-SCAN | Critical, Warning |
| Plugin Error | ALT-PLUG | Critical, Warning |
| Configuration | ALT-CONF | Warning, Info |
| Performance | ALT-PERF | Warning |
| Resource | ALT-RES | Warning, Critical |

---

## 2. Alert Definitions

### ALT-SCAN-001: Scan Aborted

- **Severity**: Critical
- **Condition**: Scan exits with code 2 (tool error), not code 1 (violations found).
- **Cause**: I/O failure, permission denied on target directory, or engine panic.
- **Action**: Check the error message in stderr. Verify target path exists and is readable.
  Review logs at `CODEQC_LOG=debug` level for stack traces.

### ALT-SCAN-002: Zero Files Scanned

- **Severity**: Warning
- **Condition**: Scan completes but reports 0 files analyzed.
- **Cause**: Exclude patterns too broad, no plugins match any file extensions, or
  empty target directory.
- **Action**: Review `.codeqc.toml` exclude list. Run `maidos-codeqc plugins list`
  to verify installed plugins cover the project's file types.

### ALT-PLUG-001: Plugin Load Failure

- **Severity**: Warning
- **Condition**: One or more plugins fail to load at startup.
- **Cause**: Binary incompatibility, missing system library, or API version mismatch.
- **Action**: Run `maidos-codeqc plugins check`. Re-download the plugin for the
  correct platform. Ensure the plugin's `api_version` matches the core.

### ALT-PLUG-002: Plugin Panic During Analysis

- **Severity**: Critical
- **Condition**: A plugin panics while analyzing a file.
- **Cause**: Bug in the plugin's analysis logic triggered by unexpected input.
- **Action**: The core catches the panic and continues scanning other files. File an
  issue with the plugin maintainer including the file that triggered the panic.
  Use `CODEQC_LOG=debug` to capture the panic backtrace.

### ALT-CONF-001: Invalid Configuration File

- **Severity**: Warning
- **Condition**: `.codeqc.toml` fails schema validation.
- **Cause**: Syntax error, unknown key, or invalid value type.
- **Action**: Run `maidos-codeqc config validate` for a detailed error report.
  Fix the TOML syntax or remove unrecognized keys.

### ALT-CONF-002: Missing Configuration File

- **Severity**: Info
- **Condition**: No `.codeqc.toml` found in the project root.
- **Cause**: First-time usage or config file not committed to version control.
- **Action**: The engine uses built-in defaults. Create a config file with
  `maidos-codeqc config init` if customization is needed.

### ALT-PERF-001: Scan Duration Exceeded SLO

- **Severity**: Warning
- **Condition**: Scan time exceeds the p95 target for the codebase size.
- **Cause**: Large codebase without exclusions, slow disk, or too many plugins.
- **Action**: Add `vendor/`, `node_modules/`, `target/` to exclude list.
  Review plugin count. Consider running with `--parallel` tuning.

### ALT-RES-001: Memory Usage Exceeded Threshold

- **Severity**: Warning
- **Condition**: Peak RSS exceeds 200 MB during a scan.
- **Cause**: Very large files, many concurrent threads, or plugin memory leak.
- **Action**: Review `CODEQC_LOG=debug` output for peak RSS. Exclude large
  generated files. Limit thread count with `--jobs N`.

---

## 3. Escalation

| Severity | Response Time | Escalation |
|:---------|:-------------|:-----------|
| Critical | Investigate immediately | Block release if in CI pipeline |
| Warning | Investigate within 24 hours | Add to next sprint if recurring |
| Info | Log and review weekly | No immediate action required |

---

*Alert conditions are checked automatically when `--strict` mode is enabled in CI.*
