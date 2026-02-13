# User Journeys - MAIDOS-CodeQC

## Overview

This document describes typical user workflows for MAIDOS-CodeQC, covering developers, CI/CD pipelines, and team leads. Each journey is identified with a J-XXX code and includes preconditions, steps, and expected outcomes.

---

## J-001: Developer Runs Quick Scan on Local Project

**Actor**: Software Developer
**Goal**: Check code quality violations before committing
**Preconditions**:
- Node.js 18+ installed
- MAIDOS-CodeQC installed (`npm install @maidos/codeqc`)
- Working directory is a code project

**Steps**:
1. Developer opens terminal in project root
2. Runs: `npx maidos-codeqc scan ./src`
3. Tool scans all supported files in `./src` directory
4. Console displays violations grouped by severity (redlines in red, prohibitions in yellow)
5. Developer reviews violations with file:line references
6. Developer fixes critical redlines (R01-R28)
7. Re-runs scan to verify fixes
8. Scan shows zero redlines, commits code

**Expected Outcome**:
- Scan completes in < 5 seconds for typical project (10k LOC)
- Clear, actionable error messages with file locations
- Exit code 0 if no critical violations, non-zero otherwise
- Developer understands what to fix without consulting documentation

**Success Metrics**:
- < 10% false positives on security rules
- Developer fixes violations in < 15 minutes
- No crashes or hangs on malformed input

---

## J-002: CI/CD Pipeline Integration

**Actor**: DevOps Engineer / CI System
**Goal**: Enforce code quality gates in automated pipeline
**Preconditions**:
- GitHub Actions or GitLab CI configured
- Project has `.codeqcrc.yml` config file

**Steps**:
1. Developer pushes code to feature branch
2. CI pipeline triggers on push event
3. CI job runs: `npm install @maidos/codeqc`
4. CI job runs: `npx maidos-codeqc scan --ci ./src`
5. CodeQC scans project and exits with code 1 if redlines found
6. CI pipeline fails, blocking merge
7. Developer receives notification with link to scan logs
8. Developer fixes violations and pushes again
9. CI re-runs, scan passes (exit code 0)
10. Pipeline proceeds to next stage (build, test, deploy)

**Expected Outcome**:
- Zero false negatives on critical security rules (R01-R12)
- CI pipeline fails fast (< 30 seconds for scan step)
- Clear failure reason in CI logs (e.g., "R01: Hardcoded credentials found in auth.ts:42")
- No manual intervention required

**Success Metrics**:
- CI integration adds < 1 minute to pipeline duration
- 100% detection of critical security violations
- < 5% false positive rate causing unnecessary pipeline failures

---

## J-003: Team Lead Reviews HTML Report

**Actor**: Engineering Team Lead
**Goal**: Understand codebase health and prioritize tech debt
**Preconditions**:
- CodeQC scan completed with HTML output
- Team lead has web browser

**Steps**:
1. Team lead runs: `npx maidos-codeqc scan -r html -o report.html ./src`
2. Tool generates standalone HTML file with embedded CSS
3. Team lead opens `report.html` in browser
4. Dashboard shows:
   - Total violation count by category (Security/Structure/Quality)
   - Top 10 offending files
   - Trend graph (if historical data available)
   - Detailed violation list with code snippets
5. Team lead filters by category (Security only)
6. Identifies 3 high-priority files with R01 (hardcoded credentials)
7. Assigns tickets to developers for remediation
8. Saves report for weekly review meeting

**Expected Outcome**:
- HTML report is self-contained (no external dependencies)
- Report loads in < 2 seconds
- Visualizations clearly show problem areas
- Report is shareable via email or wiki

**Success Metrics**:
- Team lead can prioritize work in < 10 minutes
- Report format is consistent across versions
- HTML file size < 2 MB for typical project

---

## J-004: Developer Uses Config File for Custom Rules

**Actor**: Software Developer
**Goal**: Customize rule thresholds for project-specific needs
**Preconditions**:
- Developer understands project coding standards
- MAIDOS-CodeQC supports config files

**Steps**:
1. Developer creates `.codeqcrc.yml` in project root
2. Configures custom thresholds:
   ```yaml
   level: D
   categories:
     - security
     - structure
   thresholds:
     maxFunctionLines: 100  # Allow longer functions
     maxTodos: 20           # Allow more TODOs
   excludePatterns:
     - "**/*.test.ts"
     - "**/fixtures/**"
   ```
3. Runs: `npx maidos-codeqc scan ./src`
4. Tool auto-discovers config file
5. Scan applies custom thresholds (P05 now allows 100-line functions)
6. Scan skips test files and fixtures as configured
7. Developer commits `.codeqcrc.yml` for team-wide consistency

**Expected Outcome**:
- Config file is auto-discovered without CLI flag
- Custom thresholds override defaults
- Exclude patterns work correctly
- Config syntax errors produce clear error messages

**Success Metrics**:
- Config file format is documented and intuitive
- All team members use same config (committed to repo)
- False positives reduced by 50% with tuned thresholds

---

## J-005: Project Manager Runs Pipeline for Product Release

**Actor**: Project Manager / Release Engineer
**Goal**: Execute full 10-step quality pipeline before shipping
**Preconditions**:
- Project has build/test/lint scripts configured
- Evidence directory exists for proof collection

**Steps**:
1. Release engineer runs: `npx maidos-codeqc pipeline . --grade E`
2. Pipeline auto-detects `npm run build`, `npm test`, `npm run lint` from `package.json`
3. Pipeline executes 10 steps sequentially:
   - Step 1: Anti-counterfeiting check (LV1-5 for grade E)
   - Step 2: Anti-fraud scan (R13-R18 redlines)
   - Step 3: Build check (runs `npm run build`, fails if exit code ≠ 0)
   - Step 4: Lint check (runs `npm run lint`)
   - Step 5: Test check (runs `npm test`, requires 100% pass)
   - Step 6: Coverage check (parses coverage report, warns if < 80%)
   - Step 7: Full redline scan (R01-R28, fails on any violation)
   - Step 8: G1 interface sync (checks API contracts)
   - Step 9: G2 spec coverage (validates SPEC.md checklist)
   - Step 10: G4 final acceptance (generates proof pack)
4. Pipeline generates evidence files in `./evidence/` directory
5. Proof pack ZIP created with SHA-256 + Merkle root
6. Pipeline outputs waveform report (Y/X/Z axes)
7. All steps pass, exit code 0
8. Release engineer archives proof pack for audit trail

**Expected Outcome**:
- Pipeline completes in < 5 minutes for 10k LOC project
- Fail-fast on critical failures (build, test, security redlines)
- Evidence directory contains 10+ log files with timestamps
- Proof pack is cryptographically verifiable (LV5 hash)

**Success Metrics**:
- 100% reproducibility (same code = same proof hash)
- Zero undetected critical violations
- Proof pack accepted by external auditors

---

## J-006: Security Auditor Runs Security-Only Scan

**Actor**: Security Auditor
**Goal**: Verify codebase has no security vulnerabilities
**Preconditions**:
- Auditor receives codebase ZIP from development team
- Auditor has CodeQC installed

**Steps**:
1. Auditor extracts codebase to `/tmp/audit-project`
2. Runs: `npx maidos-codeqc scan --only-security /tmp/audit-project`
3. Tool scans only security-related rules (R01-R28, subset of P-rules)
4. Console shows 2 violations:
   - R01: Hardcoded API key in `config.ts:15`
   - R10: HTTP URL in `api.ts:88` (should be HTTPS)
5. Auditor generates JSON report: `npx maidos-codeqc scan --only-security -r json -o audit.json /tmp/audit-project`
6. Sends `audit.json` back to development team with rejection notice
7. Development team fixes violations
8. Auditor re-scans, zero violations
9. Auditor approves release

**Expected Outcome**:
- `--only-security` flag filters to security rules only
- JSON report is machine-parsable for tracking systems
- Zero false negatives on known CVE patterns
- Clear remediation guidance in report

**Success Metrics**:
- Auditor completes review in < 1 hour for 50k LOC
- No manual code inspection required for common vulnerabilities
- 95% agreement with manual security review

---

## J-007: Open Source Maintainer Integrates CodeQC Badge

**Actor**: Open Source Maintainer
**Goal**: Show code quality status in GitHub README
**Preconditions**:
- Project hosted on GitHub
- GitHub Actions configured with CodeQC

**Steps**:
1. Maintainer adds GitHub Actions workflow:
   ```yaml
   name: Code Quality
   on: [push, pull_request]
   jobs:
     codeqc:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4
         - uses: actions/setup-node@v4
         - run: npm install @maidos/codeqc
         - run: npx maidos-codeqc scan --ci ./src
   ```
2. Pushes workflow to main branch
3. GitHub Actions runs scan on every commit
4. Maintainer adds badge to README.md:
   ```markdown
   ![Code Quality](https://github.com/user/repo/actions/workflows/codeqc.yml/badge.svg)
   ```
5. Badge shows green checkmark when scan passes
6. Contributors see badge before submitting PRs
7. Quality improves over time as contributors fix violations

**Expected Outcome**:
- Badge accurately reflects latest scan status
- CI integration is zero-config for standard Node.js projects
- Contributors understand quality expectations before contributing

**Success Metrics**:
- < 5 minutes to add workflow file and badge
- 100% uptime for CI scans
- 30% reduction in PR rejections due to code quality

---

## J-008: Polyglot Developer Scans Multi-Language Project

**Actor**: Full-Stack Developer
**Goal**: Scan project with TypeScript, Python, and Rust code
**Preconditions**:
- Project contains `.ts`, `.py`, `.rs` files
- Tree-sitter grammars installed for all languages

**Steps**:
1. Developer runs: `npx maidos-codeqc scan ./`
2. Tool auto-detects file types by extension
3. Loads appropriate Tree-sitter parsers:
   - `tree-sitter-typescript` for `.ts` files
   - `tree-sitter-python` for `.py` files
   - `tree-sitter-rust` for `.rs` files
4. Runs AST-based rules (P05, P06, P10) using language-specific parsers
5. Runs regex-based rules (R01, R02, R07, R10) on all files
6. Aggregates violations across all languages
7. Console shows violations grouped by file, with language labels

**Expected Outcome**:
- All 5 core languages supported (TS/JS/Python/Rust/Go)
- Language detection is automatic (no flags required)
- Rules behave consistently across languages
- Missing parsers produce clear warnings (not crashes)

**Success Metrics**:
- 90% rule coverage parity across all 5 languages
- Scan completes in < 10 seconds for 5k LOC mixed codebase
- Zero crashes on unsupported file types

---

## J-009: Data Engineer Configures Serve Mode Dashboard

**Actor**: Data Engineer / DevOps
**Goal**: Set up real-time code quality dashboard for team
**Preconditions**:
- Server with Node.js 18+ available
- Team has access to internal network

**Steps**:
1. Data engineer runs: `npx maidos-codeqc serve --port 3000 --path /var/repos/`
2. Tool starts HTTP server on port 3000
3. Serves web dashboard at `http://localhost:3000/`
4. WebSocket connection established for real-time updates
5. Dashboard shows:
   - Current scan status (idle/scanning)
   - Last scan results (violation counts by category)
   - Historical trend graph (if data persisted)
6. Team members bookmark dashboard URL
7. CI/CD triggers scan via API: `POST /api/scan`
8. Dashboard updates in real-time (< 1 second latency)
9. Team monitors quality metrics during sprint

**Expected Outcome**:
- Serve mode starts in < 3 seconds
- Dashboard accessible from any browser on network
- WebSocket updates have < 100 ms latency
- API endpoint returns JSON results for programmatic access

**Success Metrics**:
- 100% uptime during business hours
- < 2 second page load time on dashboard
- 10+ concurrent users supported without degradation

---

## Summary Table

| ID | Journey | Actor | Duration | Frequency |
|----|---------|-------|----------|-----------|
| J-001 | Quick Scan | Developer | 5 min | Daily |
| J-002 | CI/CD Integration | DevOps | 30 sec | Per commit |
| J-003 | HTML Report Review | Team Lead | 10 min | Weekly |
| J-004 | Config File Setup | Developer | 15 min | One-time |
| J-005 | Pipeline Execution | Release Engineer | 5 min | Per release |
| J-006 | Security Audit | Security Auditor | 1 hour | Per release |
| J-007 | Badge Integration | OSS Maintainer | 5 min | One-time |
| J-008 | Multi-Language Scan | Full-Stack Dev | 10 min | Daily |
| J-009 | Dashboard Setup | DevOps | 30 min | One-time |

---

*MAIDOS-CodeQC User Journeys v0.3.5 -- CodeQC Gate C Compliant*
