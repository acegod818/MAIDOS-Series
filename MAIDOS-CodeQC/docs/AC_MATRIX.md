# MAIDOS-CodeQC -- Acceptance Criteria Matrix

> **Version**: 1.0
> **Date**: 2026-02-07
> **Product**: MAIDOS-CodeQC v2.6.1

---

## Scanning (J-001)

| AC ID | Criterion | Verification |
|:------|:----------|:-------------|
| AC-001 | Engine discovers all source files recursively from the target path | Integration test with nested directory structure |
| AC-002 | Files are matched to plugins by extension mapping | Unit test with 10+ file types |
| AC-003 | Scan completes within 5 seconds for 10K LOC | Benchmark test |
| AC-004 | Violations include file path, line number, rule ID, and message | Output schema validation |
| AC-005 | Exit code 0 when all gates pass; 1 when any gate fails | CLI integration test |

## Reporting (J-004)

| AC ID | Criterion | Verification |
|:------|:----------|:-------------|
| AC-006 | JSON report conforms to `schemas/report.json` schema | JSON schema validation |
| AC-007 | HTML report renders correctly in Chrome, Firefox, Edge | Manual smoke test |
| AC-008 | Console output respects `--no-color` flag | CLI integration test |
| AC-009 | Report includes gate summary with pass/fail per gate | Output inspection |

## Plugin Loading (J-003)

| AC ID | Criterion | Verification |
|:------|:----------|:-------------|
| AC-010 | Plugins are loaded from the configured plugin directory | Integration test with mock plugin |
| AC-011 | Invalid plugin binaries produce a warning, not a crash | Fault injection test |
| AC-012 | `plugins list` shows name, version, and supported file types | CLI output test |
| AC-013 | Hot-loaded plugin is available on next scan without restart | Integration test |

## Rule Configuration (J-002)

| AC ID | Criterion | Verification |
|:------|:----------|:-------------|
| AC-014 | `.codeqc.toml` overrides default rule settings | Unit test with custom config |
| AC-015 | Invalid config file produces a descriptive error message | Error handling test |
| AC-016 | Disabled rules produce zero violations for their category | Scan test with disabled rules |
| AC-017 | Gate thresholds from config are enforced during evaluation | Threshold boundary test |

## Fake Implementation Cleanup (J-005)

| AC ID | Criterion | Verification |
|:------|:----------|:-------------|
| AC-018 | Detects `return true/false/null/0` stubs | Pattern matching unit test |
| AC-019 | Detects empty catch/except blocks | AST analysis test |
| AC-020 | Detects `todo!()`, `unimplemented!()`, `TODO` comments | Regex + AST test |

## CI/CD Integration (J-006)

| AC ID | Criterion | Verification |
|:------|:----------|:-------------|
| AC-021 | Non-zero exit code on gate failure in CI mode | CI simulation test |
| AC-022 | JSON output is machine-parseable for downstream tooling | Schema conformance test |
| AC-023 | Scan works with `--gate` flag to specify minimum gate level | CLI integration test |

---

*Traceability: each AC maps to a user journey (J-xxx) and is covered by at least one test.*
