# Acceptance Criteria Matrix - MAIDOS-CodeQC

## Overview

This document defines acceptance criteria (AC) for all major features of MAIDOS-CodeQC. Each AC is uniquely identified and maps to functional requirements in PRD.md.

---

## AC-001: Scan Mode - Basic Execution

**Feature**: FR-001 (Quick Scan)
**Priority**: P0

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-001-01 | CLI accepts directory path as argument | Manual test: `codeqc scan ./src` | PASS |
| AC-001-02 | Scans all supported file types (`.ts`, `.js`, `.py`, `.rs`, `.go`) | Unit test with fixture files | PASS |
| AC-001-03 | Ignores unsupported file types without error | Test with `.txt`, `.md` files | PASS |
| AC-001-04 | Completes scan of 10k LOC in < 5 seconds | Performance test on benchmark repo | PASS |
| AC-001-05 | Exit code 0 when no violations, non-zero when violations found | Integration test | PASS |

---

## AC-002: Scan Mode - Rule Coverage

**Feature**: FR-002 (42 Rules Implementation)
**Priority**: P0

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-002-01 | Detects all 28 redlines (R01-R28) | 28 unit tests with positive cases | PASS |
| AC-002-02 | Detects all 14 prohibitions (P01-P14) | 14 unit tests with positive cases | PASS |
| AC-002-03 | Zero false negatives on R01-R12 (critical security) | Regression test suite | PASS |
| AC-002-04 | False positive rate < 10% on redlines | Manual review of 100 real-world scans | PASS |
| AC-002-05 | Skips test/spec/mock files to avoid self-detection | Test with `*.test.ts`, `*.spec.py` | PASS |

---

## AC-003: Scan Mode - Configuration

**Feature**: FR-003 (Config File Support)
**Priority**: P1

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-003-01 | Auto-discovers `.codeqcrc.yml` in project root | Integration test | PASS |
| AC-003-02 | Config file overrides default thresholds | Test with custom P05 maxFunctionLines | PASS |
| AC-003-03 | Config file supports exclude patterns | Test with `excludePatterns: ["**/*.test.ts"]` | PASS |
| AC-003-04 | Invalid YAML produces clear error message | Test with malformed YAML | PASS |
| AC-003-05 | CLI flags override config file settings | Test `-l B` vs `level: D` in config | PASS |

---

## AC-004: Scan Mode - Soft Configuration

**Feature**: FR-004 (Category Filtering)
**Priority**: P1

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-004-01 | `--only-security` flag filters to security rules only | Test reports only R-rules, no P-rules | PASS |
| AC-004-02 | `--only-structure` flag filters to structure rules only | Test reports only structure P-rules | PASS |
| AC-004-03 | `--only-quality` flag filters to quality rules only | Test reports only quality P-rules | PASS |
| AC-004-04 | `-C s,t,q` syntax allows multiple categories | Test combination filters | PASS |
| AC-004-05 | Default is all categories enabled | Test without flags | PASS |

---

## AC-005: Reporters - Console Output

**Feature**: FR-005 (Console Reporter)
**Priority**: P0

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-005-01 | Redlines displayed in red color | Visual test in terminal | PASS |
| AC-005-02 | Prohibitions displayed in yellow color | Visual test in terminal | PASS |
| AC-005-03 | Each violation shows file:line:column format | Unit test on reporter output | PASS |
| AC-005-04 | Summary shows total violations by category | Test with mock analysis result | PASS |
| AC-005-05 | Output is readable without ANSI colors (CI mode) | Test with `NO_COLOR=1` environment variable | PASS |

---

## AC-006: Reporters - JSON Output

**Feature**: FR-006 (JSON Reporter)
**Priority**: P1

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-006-01 | Outputs valid JSON (parsable by `JSON.parse`) | Unit test with schema validation | PASS |
| AC-006-02 | JSON includes all violation fields (rule, file, line, message) | Schema test | PASS |
| AC-006-03 | JSON schema is stable across versions | Regression test | PASS |
| AC-006-04 | Large reports (10k+ violations) produce valid JSON | Stress test | PASS |
| AC-006-05 | `-o` flag writes JSON to file | Integration test | PASS |

---

## AC-007: Reporters - HTML Output

**Feature**: FR-007 (HTML Reporter)
**Priority**: P1

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-007-01 | Generates standalone HTML file (no external dependencies) | Test file opens in browser offline | PASS |
| AC-007-02 | HTML includes embedded CSS (no CDN links) | Test with network disabled | PASS |
| AC-007-03 | HTML shows violation summary table | Visual test in browser | PASS |
| AC-007-04 | HTML shows top 10 offending files | Visual test | PASS |
| AC-007-05 | File size < 2 MB for typical project | Test with 10k LOC fixture | PASS |

---

## AC-008: Pipeline Mode - 10-Step Execution

**Feature**: FR-008 (Pipeline Mode)
**Priority**: P0

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-008-01 | Executes all 10 steps in sequence | Integration test with mock project | PASS |
| AC-008-02 | Fails fast on fatal step failure (build, test, redlines) | Test with broken build | PASS |
| AC-008-03 | Non-fatal steps (lint, coverage) produce warnings, not errors | Test with low coverage | PASS |
| AC-008-04 | Generates evidence files in `./evidence/` directory | File system test | PASS |
| AC-008-05 | Completes in < 5 minutes for 10k LOC project | Performance test | PASS |

---

## AC-009: Pipeline Mode - External Command Integration

**Feature**: FR-009 (Build/Test/Lint Detection)
**Priority**: P1

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-009-01 | Auto-detects `npm run build` from `package.json` | Test with standard Node.js project | PASS |
| AC-009-02 | Auto-detects `npm test` from `package.json` | Test with Vitest project | PASS |
| AC-009-03 | Auto-detects `npm run lint` from `package.json` | Test with ESLint project | PASS |
| AC-009-04 | Accepts manual command via `--build-cmd` flag | Test with custom build script | PASS |
| AC-009-05 | Skips external commands with `--no-auto` flag | Test pure static analysis | PASS |

---

## AC-010: Pipeline Mode - Proof Pack Generation

**Feature**: FR-010 (Proof Pack LV1-LV9)
**Priority**: P1

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-010-01 | Generates proof pack ZIP file | File system test | PASS |
| AC-010-02 | Proof pack includes SHA-256 checksum | Test ZIP contents | PASS |
| AC-010-03 | Proof pack includes Merkle root for evidence files | Cryptographic test | PASS |
| AC-010-04 | LV4: Nonce is UUID v4 format | Regex validation test | PASS |
| AC-010-05 | LV5: Hash is reproducible for same input | Determinism test | PASS |

---

## AC-011: Pipeline Mode - Gate Execution

**Feature**: FR-011 (G1-G4 Gates)
**Priority**: P1

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-011-01 | G1 gate checks interface synchronization | Test with SPEC.md function list | PASS |
| AC-011-02 | G2 gate checks spec coverage (checkbox count) | Test with SPEC.md checklist | PASS |
| AC-011-03 | G3 gate checks integration tests (placeholder in v0.3.5) | Skipped (future) | SKIP |
| AC-011-04 | G4 gate generates final acceptance proof | File system test | PASS |
| AC-011-05 | Gate failure blocks pipeline progression | Test with incomplete SPEC.md | PASS |

---

## AC-012: Serve Mode - API Server

**Feature**: FR-012 (Serve Mode API)
**Priority**: P2

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-012-01 | HTTP server starts on specified port | Integration test with `--port 3000` | PASS |
| AC-012-02 | `GET /health` returns HTTP 200 with status JSON | cURL test | PASS |
| AC-012-03 | `POST /api/scan` accepts scan request JSON | Integration test | PASS |
| AC-012-04 | `POST /api/scan` returns scan results JSON | Integration test | PASS |
| AC-012-05 | Server handles 10 concurrent requests | Load test with `ab` (Apache Bench) | PASS |

---

## AC-013: Serve Mode - Dashboard

**Feature**: FR-013 (Web Dashboard)
**Priority**: P2

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-013-01 | Dashboard accessible at `http://localhost:<port>/` | Browser test | PASS |
| AC-013-02 | Dashboard loads in < 2 seconds | Performance test | PASS |
| AC-013-03 | Dashboard shows violation count by category | Visual test | PASS |
| AC-013-04 | WebSocket connection updates in real-time | Test with mock scan trigger | PASS |
| AC-013-05 | Dashboard works in Chrome, Firefox, Safari | Cross-browser test | PASS |

---

## AC-014: Multi-Language Support

**Feature**: FR-014 (5 Core Languages)
**Priority**: P0

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-014-01 | TypeScript files (`.ts`, `.tsx`) are parsed with Tree-sitter | Unit test | PASS |
| AC-014-02 | JavaScript files (`.js`, `.jsx`, `.mjs`, `.cjs`) are parsed | Unit test | PASS |
| AC-014-03 | Python files (`.py`) are parsed | Unit test | PASS |
| AC-014-04 | Rust files (`.rs`) are parsed | Unit test | PASS |
| AC-014-05 | Go files (`.go`) are parsed | Unit test | PASS |

---

## AC-015: CI/CD Integration

**Feature**: FR-015 (CI Mode)
**Priority**: P1

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-015-01 | `--ci` flag exits with code 1 on violations | Integration test | PASS |
| AC-015-02 | `--ci` flag exits with code 0 when clean | Integration test | PASS |
| AC-015-03 | GitHub Actions workflow file example provided | Documentation test | PASS |
| AC-015-04 | GitLab CI workflow file example provided | Documentation test | PASS |
| AC-015-05 | CI mode outputs machine-readable logs (no colors) | Test with `CI=1` env var | PASS |

---

## AC-016: Performance - Memory Constraints

**Feature**: NFR-002 (Memory Usage)
**Priority**: P0

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-016-01 | Idle memory < 50 MB | Memory profiler test | PASS |
| AC-016-02 | Peak memory < 512 MB when scanning 100k LOC | Memory profiler test | PASS |
| AC-016-03 | Serve mode idle < 128 MB | Memory profiler test | PASS |
| AC-016-04 | Serve mode peak < 768 MB with 10 concurrent scans | Load test with profiler | PASS |
| AC-016-05 | No memory leaks over 1000 scans | Long-running stress test | PASS |

---

## AC-017: Security - Input Validation

**Feature**: NFR-004 (Security Requirements)
**Priority**: P0

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-017-01 | Directory traversal attacks blocked (e.g., `../../etc/passwd`) | Security test | PASS |
| AC-017-02 | Malformed YAML config rejected with clear error | Fuzz test | PASS |
| AC-017-03 | Regex timeout prevents ReDoS attacks (1 second limit) | Security test with pathological regex | PASS |
| AC-017-04 | No arbitrary code execution via config file | Static analysis + manual review | PASS |
| AC-017-05 | Source code never logged in production mode | Log audit test | PASS |

---

## AC-018: Reliability - False Negative Prevention

**Feature**: NFR-003 (Reliability Requirements)
**Priority**: P0

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-018-01 | R01 (hardcoded credentials): 0% false negatives | Regression test with 50 known samples | PASS |
| AC-018-02 | R02 (SQL injection): 0% false negatives | Regression test with 30 known samples | PASS |
| AC-018-03 | R07 (disable security): 0% false negatives | Regression test with 20 known samples | PASS |
| AC-018-04 | R10 (plaintext transmission): 0% false negatives | Regression test with 25 known samples | PASS |
| AC-018-05 | R05 (ignore errors): < 5% false negatives | Manual review of real-world code | PASS |

---

## AC-019: Usability - Error Messages

**Feature**: NFR-005 (Usability Requirements)
**Priority**: P1

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-019-01 | Error messages do not show raw stack traces | Integration test with invalid input | PASS |
| AC-019-02 | File not found error shows full path | Test with nonexistent path | PASS |
| AC-019-03 | Invalid config error shows line number | Test with malformed YAML | PASS |
| AC-019-04 | Help text (`--help`) includes usage examples | Manual review | PASS |
| AC-019-05 | All CLI flags have short and long forms | Manual review | PASS |

---

## AC-020: Extensibility - Plugin System

**Feature**: NFR-011 (Extensibility Requirements)
**Priority**: P2

| ID | Criteria | Test Method | Status |
|----|----------|-------------|--------|
| AC-020-01 | Custom language support via plugin API | Unit test with mock plugin | PASS |
| AC-020-02 | Custom reporter via plugin interface | Unit test with mock reporter | PASS |
| AC-020-03 | Plugin registration via config file | Integration test | PASS |
| AC-020-04 | Missing plugin produces warning, not crash | Test with invalid plugin path | PASS |
| AC-020-05 | Plugin API documented in API.md | Documentation review | PASS |

---

## Summary Table

| Feature Area | Total AC | Pass | Fail | Skip | Coverage |
|--------------|----------|------|------|------|----------|
| Scan Mode | 15 | 15 | 0 | 0 | 100% |
| Reporters | 15 | 15 | 0 | 0 | 100% |
| Pipeline Mode | 15 | 14 | 0 | 1 | 93% |
| Serve Mode | 9 | 9 | 0 | 0 | 100% |
| Multi-Language | 5 | 5 | 0 | 0 | 100% |
| CI/CD | 5 | 5 | 0 | 0 | 100% |
| Performance | 5 | 5 | 0 | 0 | 100% |
| Security | 5 | 5 | 0 | 0 | 100% |
| Reliability | 5 | 5 | 0 | 0 | 100% |
| Usability | 5 | 5 | 0 | 0 | 100% |
| Extensibility | 5 | 5 | 0 | 0 | 100% |
| **TOTAL** | **89** | **88** | **0** | **1** | **99%** |

---

*MAIDOS-CodeQC AC Matrix v0.3.5 -- CodeQC Gate C Compliant*
