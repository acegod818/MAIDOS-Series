# MAIDOS-CodeQC -- Operations Runbook

> **Version**: 1.0
> **Date**: 2026-02-07
> **Product**: MAIDOS-CodeQC v2.6.1

---

## 1. Common Operations

### 1.1 Run a Full Scan

```bash
maidos-codeqc scan . --gate G4 --format console
```

### 1.2 Validate Configuration

```bash
maidos-codeqc config validate
```

Expected output: `Configuration valid.` or a list of issues with line references.

### 1.3 List Installed Plugins

```bash
maidos-codeqc plugins list
```

Output columns: Name, Version, API Version, Supported Extensions, Status.

### 1.4 Check Plugin Compatibility

```bash
maidos-codeqc plugins check
```

Reports any plugins with mismatched API versions that will be skipped during scans.

## 2. Troubleshooting

### 2.1 Scan Produces No Output

| Check | Command |
|:------|:--------|
| Target directory exists | `ls <target>` |
| Files match plugin extensions | `maidos-codeqc plugins list` for supported types |
| Config does not exclude everything | Review `[scan].exclude` in `.codeqc.toml` |

### 2.2 Plugin Fails to Load

Symptom: Warning message `Failed to load plugin: <name>`.

| Cause | Resolution |
|:------|:-----------|
| Wrong platform binary | Download the correct .dll/.so/.dylib for your OS |
| API version mismatch | Update the plugin to match core API version |
| Missing dependencies | Check `ldd` (Linux) or `dumpbin /dependents` (Windows) |
| Corrupted download | Re-download and verify checksum |

### 2.3 Scan Is Slow

| Cause | Resolution |
|:------|:-----------|
| Large codebase without exclusions | Add `vendor/`, `node_modules/`, `target/` to exclude |
| Too many plugins loaded | Disable unused plugins in config |
| Disk I/O bottleneck | Run on SSD; consider `--parallel` flag tuning |

### 2.4 False Positives

1. Identify the rule ID from the violation output (e.g., `SYS-003`).
2. Suppress inline: add `// codeqc:ignore SYS-003` above the line.
3. Suppress globally: add the rule to `[rules.disabled]` in `.codeqc.toml`.
4. Report the false positive to the plugin maintainer with a minimal reproducer.

## 3. Log Levels

```bash
CODEQC_LOG=debug maidos-codeqc scan .    # Full debug output
CODEQC_LOG=info maidos-codeqc scan .     # Default
CODEQC_LOG=warn maidos-codeqc scan .     # Warnings and errors only
```

## 4. Health Check

Verify the installation is functional:

```bash
maidos-codeqc self-check
```

This validates: binary integrity, plugin directory access, config schema availability,
and report template presence. Exit code 0 means all checks pass.

## 5. Maintenance Schedule

| Task | Frequency | Command |
|:-----|:----------|:--------|
| Update plugins | Monthly | `maidos-codeqc plugins update` |
| Review suppressed rules | Quarterly | Audit `.codeqc.toml` disabled list |
| Upgrade core binary | Per release | See DEPLOY.md upgrade procedure |

---

*For alert conditions and escalation, see ALERTS.md.*
