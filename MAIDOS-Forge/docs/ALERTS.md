# MAIDOS-Forge Alerting and Monitoring

| Field   | Value                                                  |
|---------|--------------------------------------------------------|
| Product | MAIDOS-Forge                                           |
| Version | 3.0                                                    |
| Type    | Local CLI tool -- alerts are build-time checks, not ops|

## 1. Overview

MAIDOS-Forge is a local CLI tool, not a long-running service. "Alerting" means detecting problems at build time and reporting them clearly to the user via exit codes, stderr messages, and optional structured output.

## 2. Build Failure Alerts

### Detection

Any non-zero exit code indicates a build failure.

| Condition                | Exit Code | stderr Output                          |
|--------------------------|-----------|----------------------------------------|
| Compilation error        | 1         | Normalized error with file/line/col    |
| Toolchain missing        | 2         | Which language and expected binary     |
| Config error             | 3         | Parse error with line number           |
| Plugin error             | 4         | Plugin name and failure reason         |
| I/O error                | 5         | File path and OS error message         |

### CI Integration

In CI pipelines, check the exit code:

```yaml
# GitHub Actions example
- name: Forge Build
  run: forge build
  # GitHub Actions treats non-zero exit as failure by default
```

### Structured Error Output

Enable JSON error output for machine consumption:

```bash
forge build --output-format json 2> errors.json
```

JSON schema per error:

```json
{
  "file": "src/lib.rs",
  "line": 42,
  "col": 5,
  "severity": "error",
  "message": "type mismatch: expected i32, found String",
  "lang": "rust"
}
```

## 3. Performance Degradation Alerts

### Compilation Time Threshold

Use `--timings` with `--warn-slow`:

```bash
forge build --timings --warn-slow 30s
```

If total build time exceeds the threshold, Forge prints a warning to stderr:

```
Warning: build completed in 47s, exceeding threshold of 30s
```

Exit code remains 0 (build succeeded), but the warning is visible in CI logs.

### Per-Module Threshold

In `forge.toml`:

```toml
[build]
warn_slow_module_ms = 10000   # warn if any single module takes >10s
```

### CI Benchmark Gate

```bash
forge build --timings --fail-slow 60s
```

With `--fail-slow`, Forge exits with code 1 if the threshold is exceeded. Use this in CI to catch performance regressions.

## 4. Resource Alerts

### Memory Usage

Forge monitors its own RSS during builds (on supported platforms):

```bash
forge build --warn-memory 100
```

The value is in MB. If peak RSS exceeds the threshold during the build, a warning is emitted:

```
Warning: peak memory usage 142 MB exceeded threshold of 100 MB
```

### Disk Space

Before writing build artifacts, Forge checks available disk space. If less than 100 MB is available, the build aborts with exit code 5 and a clear message.

## 5. Plugin Alerts

### Plugin Load Failure

When a plugin DLL fails to load, Forge emits a warning per plugin:

```
Warning: failed to load plugin 'MaidOS.Forge.Plugin.Go.dll': missing dependency 'System.Text.Json 8.0'
```

The build continues with available plugins. If a module requires a failed plugin, compilation fails with exit code 4.

### Version Mismatch

Each plugin declares a `min_forge_version` in its capabilities. If the running Forge version is older, Forge emits:

```
Warning: plugin 'Go' requires Forge >= 3.1.0 but current is 3.0.0
```

The plugin is skipped.

### Toolchain Validation

At the start of each build, Forge calls `ValidateToolchainAsync()` on every plugin needed by the current project. Failures are reported immediately:

```
Error [exit 2]: toolchain validation failed for 'cpp':
  Expected: cl.exe (MSVC 19+) or g++ (GCC 12+)
  Found: none on PATH
```

## 6. Verbose Logging

### Enable Verbose Output

```bash
# Standard verbose
forge build --verbose

# Maximum detail (trace level)
forge build --log-level trace
```

### What Verbose Shows

| Level   | Additional Output                                        |
|---------|----------------------------------------------------------|
| info    | Build progress, module names, pass/fail (default)        |
| verbose | Plugin load details, toolchain paths, config resolution  |
| debug   | Exact compiler command lines, environment variables       |
| trace   | Raw compiler stdout/stderr, plugin method call timing    |

### Redirect Logs to File

```bash
forge build --verbose 2> forge.log
```

Logs go to stderr; build results go to stdout. This separation allows piping structured output while capturing diagnostics.

## 7. Monitoring in CI

### Recommended CI Setup

```yaml
steps:
  - name: Forge Check
    run: forge check
    # Validates toolchains and config before building

  - name: Forge Build
    run: forge build --timings --warn-slow 30s --warn-memory 200 --output-format json 2> build-report.json

  - name: Upload Diagnostics
    if: failure()
    uses: actions/upload-artifact@v4
    with:
      name: forge-diagnostics
      path: build-report.json
```

### Health Check Command

```bash
forge doctor
```

Outputs a full diagnostic report:
- Forge version
- Rust and .NET versions
- Loaded plugins and their toolchain status
- Config validation result
- Available disk space
