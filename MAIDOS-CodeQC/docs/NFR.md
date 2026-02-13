# Non-Functional Requirements - MAIDOS-CodeQC

## 1. Overview

This document specifies the non-functional requirements (NFRs) for MAIDOS-CodeQC, a TypeScript/Node.js CLI tool implementing Code-QC v3.5 standards with 42 rules (28 redlines + 14 prohibitions) and three operational modes: scan, pipeline, and serve.

## 2. Performance Requirements

### 2.1 Scan Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Scan throughput | 10,000 lines < 5 seconds | Single-threaded scan on typical developer laptop |
| File discovery | < 1 second | For projects with < 10,000 files |
| Regex rule execution | < 10 ms per rule per file | For files < 1000 lines |
| AST parsing (Tree-sitter) | < 100 ms per file | For TypeScript files < 5000 lines |

### 2.2 Memory Usage

| Metric | Target | Condition |
|--------|--------|-----------|
| Idle memory | < 50 MB | CLI loaded, no active scan |
| Peak memory during scan | < 512 MB | Scanning 100,000 lines of code |
| Serve mode idle | < 128 MB | API server running, no active requests |
| Serve mode peak | < 768 MB | 10 concurrent scan requests |

### 2.3 Pipeline Mode

| Operation | Target |
|-----------|--------|
| Full 10-step pipeline | < 5 minutes for typical project (10k LOC) |
| Evidence collection | < 2 seconds per step |
| Proof pack generation | < 3 seconds (ZIP + SHA256 + Merkle) |
| Gate execution (G1-G4) | < 10 seconds total |

### 2.4 Serve Mode

| Metric | Target |
|--------|--------|
| API cold start | < 3 seconds |
| Dashboard page load | < 2 seconds |
| Scan API response | < 10 seconds for 5k LOC |
| WebSocket update latency | < 100 ms |

## 3. Reliability Requirements

### 3.1 False Negative Rate (Security Rules)

- **R01-R12 (Critical Security Redlines)**: 0% false negatives on known patterns
- **R13-R28 (Extended Redlines)**: < 5% false negatives
- **P01-P14 (Prohibitions)**: < 10% false negatives acceptable (quality-focused)

### 3.2 False Positive Rate

- **Redlines (R01-R28)**: < 10% false positives
- **Prohibitions (P01-P14)**: < 20% false positives
- **Test/Spec/Mock file exclusion**: Must skip files matching `test|spec|mock` patterns

### 3.3 Error Handling

- All CLI commands must exit with non-zero code on failure
- Malformed input files must not crash the scanner
- Invalid config files must produce clear error messages
- Network failures in serve mode must return HTTP 500 with error details

### 3.4 Data Integrity

- Pipeline evidence files must include SHA-256 checksums
- Proof pack Merkle root must be verifiable
- Nonce (LV4 anti-replay) must be cryptographically random (UUID v4)
- Report JSON schema must be stable across versions

## 4. Security Requirements

### 4.1 Input Validation

- All file paths must be sanitized against directory traversal
- Config file YAML parsing must reject untrusted schemas
- Regex patterns must have execution timeout (1 second per match)
- No arbitrary code execution via config files

### 4.2 Self-Detection Avoidance

- CodeQC must not flag its own test cases
- Checker logic must skip files with `test/spec/mock` in path
- Regex definition lines must be excluded from regex rule matching
- Comment-only violations must be ignored

### 4.3 Privacy

- No telemetry or analytics transmission
- No network calls in scan/pipeline modes
- Serve mode API accepts only local connections by default
- Source code never logged in production mode

## 5. Usability Requirements

### 5.1 CLI Interface

- All commands support `--help` with clear usage examples
- Error messages must be actionable (not raw stack traces)
- Progress indicators for long-running scans (> 5 seconds)
- Color-coded output: red (errors), yellow (warnings), green (success)

### 5.2 Configuration

- `.codeqcrc.yml` config file must be auto-discovered in project root
- Soft configuration via CLI flags: `--only-security`, `--only-structure`, `--only-quality`
- Category filters: `-C s,t,q` for selective analysis
- All config keys must have sensible defaults

### 5.3 Reports

- Console reporter: human-readable with file:line references
- JSON reporter: machine-parsable with stable schema
- HTML reporter: standalone file with embedded CSS (no external dependencies)
- Report must include: total violations, by-category breakdown, top offending files

## 6. Compatibility Requirements

| Requirement | Specification |
|-------------|---------------|
| Node.js Version | >= 18.0.0 |
| Operating Systems | Windows 10+, macOS 11+, Linux (Ubuntu 20.04+) |
| TypeScript Compiler | >= 5.3.3 |
| Module System | ESM + CJS dual export support |
| Package Format | npm tarball, standalone ZIP |

## 7. Maintainability Requirements

- All TypeScript code must pass `tsc --noEmit` with zero errors
- All code must pass ESLint with zero warnings
- Unit test coverage target: 80% for core engine logic
- All public API functions must have JSDoc comments
- Vitest test suite must complete in < 30 seconds

## 8. Installability Requirements

- Single command install: `npm install @maidos/codeqc`
- ZIP package: self-contained with `npm install` required
- Global binary install: `npm install -g` exposes `codeqc` and `maidos-codeqc` commands
- No post-install scripts or native dependencies

## 9. Scalability Requirements

### 9.1 Codebase Size

- Must handle projects up to 1,000,000 lines of code
- Must handle projects with up to 50,000 files
- Must handle files up to 50,000 lines (with degraded performance acceptable)

### 9.2 Concurrency (Serve Mode)

- Support 10 concurrent scan requests
- WebSocket connections: up to 100 simultaneous clients
- Dashboard: refresh every 3 seconds under load

## 10. Logging and Diagnostics

- No logging in default mode (clean console output)
- Debug mode: `CODEQC_DEBUG=1` environment variable
- Debug logs written to stderr, not stdout
- Log format: `[TIMESTAMP] [LEVEL] [MODULE] message`
- Log rotation not required (short-lived CLI processes)

## 11. Extensibility Requirements

- Plugin system for custom language support
- Custom rule injection via config file
- Reporter plugin interface for custom output formats
- Language detection via file extension mapping (configurable)

## 12. Internationalization

- Error messages: English only (v0.3.5)
- CLI help text: English only
- HTML report: UTF-8 encoding with Chinese character support
- Tutorial documentation: Chinese (TUTORIAL.md)

## 13. Compliance Requirements

- **Code-QC v3.5 Standards**: Full implementation of 42/42 rules (100%)
- **CodeQC Gate C**: All 14 C-layer spec documents present
- **MIT License**: No GPL dependencies
- **Zero-trust philosophy**: No external API calls for rule checking (regex + AST only, no LLM)

---

*MAIDOS-CodeQC NFR v0.3.5 -- CodeQC Gate C Compliant*
