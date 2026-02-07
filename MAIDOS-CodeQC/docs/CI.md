# MAIDOS-CodeQC -- CI/CD Integration

| Field     | Value              |
|-----------|--------------------|
| Product   | MAIDOS-CodeQC      |
| Version   | v3.0               |
| Type      | CI/CD Guide        |

## Overview

MAIDOS-CodeQC can be integrated into CI/CD pipelines to enforce quality gates automatically on every commit or pull request.

## GitHub Actions Example

```yaml
name: CodeQC Pipeline
on: [push, pull_request]

jobs:
  qc:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - name: Install CodeQC
        run: npm install -g maidos-codeqc
      - name: Run QC Pipeline
        run: codeqc run --json . > qc-result.json
      - name: Upload Proof Pack
        uses: actions/upload-artifact@v4
        with:
          name: qc-proof-pack
          path: proof/
```

## Exit Codes

| Code | Meaning                               |
|------|---------------------------------------|
| 0    | All gates passed                      |
| 1    | One or more gates failed              |
| 2    | Configuration error                   |
| 3    | Runtime error (crash)                 |

## JSON Output Schema

```json
{
  "product": "string",
  "version": "string",
  "timestamp": "ISO 8601",
  "passed": true,
  "gates": [
    {
      "id": "g1",
      "name": "Spec Gate",
      "passed": true,
      "duration_ms": 1200,
      "errors": []
    }
  ]
}
```

## CI Best Practices

| Practice                      | Description                                   |
|-------------------------------|-----------------------------------------------|
| Run on every PR               | Catch issues before merge                     |
| Archive proof packs           | Store evidence for audit trail                |
| Fail the build on QC failure  | Use exit codes to block merges                |
| Cache node_modules            | Speed up CI runs with dependency caching      |
| Use `--json` output           | Machine-readable results for CI integration   |

*MAIDOS-CodeQC CI v3.0 -- CodeQC Gate C Compliant*
