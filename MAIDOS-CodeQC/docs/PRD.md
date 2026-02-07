# MAIDOS-CodeQC -- Product Requirements Document

| Field   | Value            |
|---------|------------------|
| Product | MAIDOS-CodeQC    |
| Version | 3.0              |
| Owner   | MAIDOS Team      |

## Purpose

Quality-control pipeline for MAIDOS. Enforces 4-gate (G1-G4) process.
TypeScript/Node.js CLI and web-UI dashboard.

## Feature Requirements

| ID     | Feature           | Description                                     | Priority |
|--------|-------------------|-------------------------------------------------|----------|
| FR-001 | 4-Gate Pipeline   | G1 build, G2 unit, G3 integration, G4 proof     | P0       |
| FR-002 | Spec Check        | Validate docs against CodeQC schema              | P0       |
| FR-003 | Proof Generation  | Generate evidence pack per product               | P1       |
| FR-004 | Web-UI Dashboard  | Real-time gate status per product                | P1       |
| FR-005 | CI Integration    | GitHub Actions / local runner support            | P1       |

## Acceptance Criteria Summary

- FR-001: all 4 gates execute in sequence; fail-fast on error
- FR-002: missing/malformed doc detected and reported
- FR-003: proof pack ZIP contains all evidence
- FR-004: dashboard renders within 2 s
- FR-005: exit code 0 on pass, non-zero on fail

*MAIDOS-CodeQC PRD v3.0 -- CodeQC Gate C Compliant*
