# MAIDOS-CodeQC -- Product Specification

| Field         | Value                                              |
|---------------|----------------------------------------------------|
| Product       | MAIDOS-CodeQC                                      |
| Version       | v3.0                                               |
| Type          | TypeScript / Node.js CLI + Web UI                  |
| Description   | Code quality assurance pipeline tool for MAIDOS products |
| Entry Points  | `codeqc.cmd` (Windows), `codeqc.sh` (Unix)        |
| Build System  | tsup (via `npm run build`)                         |
| Test Runner   | vitest (via `npm test`)                            |
| License       | Proprietary -- MAIDOS Project                      |

## Purpose

MAIDOS-CodeQC is the centralized quality assurance pipeline that all MAIDOS series products must pass before release. It implements a 4-gate QC model (G1 through G4) covering spec compliance, build verification, test verification, and proof pack generation.

## Core Features

| Feature                  | Description                                          |
|--------------------------|------------------------------------------------------|
| 4-Gate QC Pipeline       | G1 (Spec), G2 (Build), G3 (Test), G4 (Proof)        |
| Spec Compliance Checking | Validates docs/, qc/, and manifest structure         |
| Build Verification       | Runs build commands and checks output artifacts      |
| Test Verification        | Executes unit/integration/e2e suites                 |
| Proof Pack Generation    | Collects evidence, timestamps, and produces manifest |
| Evidence Collection      | Gathers logs, screenshots, and test outputs          |
| Web UI Dashboard         | Browser-based QC status and history viewer           |

## Gate Definitions

| Gate | Name       | Purpose                                       |
|------|------------|-----------------------------------------------|
| G1   | Spec Gate  | All required docs exist and are well-formed    |
| G2   | Build Gate | Project compiles/bundles without errors        |
| G3   | Test Gate  | All test suites pass (unit, integration, e2e)  |
| G4   | Proof Gate | Evidence pack generated with valid manifest    |

## Technology Stack

| Component    | Technology          |
|--------------|---------------------|
| Language     | TypeScript 5.x      |
| Runtime      | Node.js 20+         |
| Bundler      | tsup                 |
| Test Runner  | vitest               |
| Config       | tsconfig.json        |
| Package Mgr  | npm                  |
| Web UI       | Built-in dashboard   |

## File Structure

```
MAIDOS-CodeQC/
  codeqc.cmd          # Windows entry point
  codeqc.sh           # Unix entry point
  package.json         # Dependencies and scripts
  tsconfig.json        # TypeScript configuration
  tsup.config.ts       # Bundle configuration
  vitest.config.ts     # Test configuration
  src/                 # Source code
  web-ui/              # Dashboard web interface
  docs/                # Compliance documentation
  qc/                  # QC gate scripts
```

## Version History

| Version | Date       | Notes                              |
|---------|------------|------------------------------------|
| v1.0    | 2025-06    | Initial 4-gate pipeline            |
| v2.0    | 2025-10    | Web UI dashboard, evidence system  |
| v2.6.1  | 2026-01    | Stabilized gate logic              |
| v3.0    | 2026-02    | Full compliance docs, proof packs  |

*MAIDOS-CodeQC SPEC v3.0 -- CodeQC Gate C Compliant*
