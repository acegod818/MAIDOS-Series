# MAIDOS-CodeQC -- Deployment Guide

> **Version**: 1.0
> **Date**: 2026-02-07
> **Product**: MAIDOS-CodeQC v2.6.1

---

## 1. Installation Methods

### 1.1 Cargo Install (from source)

```bash
cargo install maidos-codeqc --version 2.6.1
```

This builds from source and places the binary in `~/.cargo/bin/`.

### 1.2 Pre-built Binary Download

Download the appropriate archive from the MAIDOS releases page:

| Platform | Archive |
|:---------|:--------|
| Windows x64 | `maidos-codeqc-2.6.1-x86_64-pc-windows-msvc.zip` |
| Linux x64 | `maidos-codeqc-2.6.1-x86_64-unknown-linux-gnu.tar.gz` |
| macOS ARM64 | `maidos-codeqc-2.6.1-aarch64-apple-darwin.tar.gz` |
| macOS x64 | `maidos-codeqc-2.6.1-x86_64-apple-darwin.tar.gz` |

Extract and place `maidos-codeqc` (or `maidos-codeqc.exe`) on your PATH.

### 1.3 Docker

```bash
docker pull ghcr.io/maidos/codeqc:2.6.1
docker run --rm -v $(pwd):/workspace ghcr.io/maidos/codeqc:2.6.1 scan /workspace
```

## 2. Plugin Distribution

Plugins are distributed as shared libraries matching the host platform:

```
~/.codeqc/plugins/
  codeqc-plugin-web.dll        (Windows)
  codeqc-plugin-web.so         (Linux)
  codeqc-plugin-web.dylib      (macOS)
```

Install plugins by placing them in `~/.codeqc/plugins/` or a project-local
`plugins/` directory. The core engine scans these directories at startup.

## 3. Configuration

Create `.codeqc.toml` in the project root:

```toml
[scan]
target = "src/"
exclude = ["vendor/", "generated/"]

[gate]
minimum = "G2"

[plugins]
directory = "plugins/"

[report]
format = "json"
output = "qc-report.json"
```

## 4. CI/CD Integration

### GitHub Actions

```yaml
- name: Run MAIDOS-CodeQC
  run: |
    maidos-codeqc scan . --gate G2 --format json --output qc-report.json
  continue-on-error: false

- name: Upload QC Report
  uses: actions/upload-artifact@v4
  with:
    name: qc-report
    path: qc-report.json
```

### GitLab CI

```yaml
code-quality:
  stage: test
  script:
    - maidos-codeqc scan . --gate G2 --format json --output qc-report.json
  artifacts:
    paths:
      - qc-report.json
```

## 5. Upgrade Procedure

1. Download or build the new version.
2. Replace the binary on PATH.
3. Run `maidos-codeqc --version` to confirm.
4. Check plugin compatibility: `maidos-codeqc plugins check`.
5. Update plugins if the API version has changed.

---

*For operational procedures after deployment, see RUNBOOK.md.*
