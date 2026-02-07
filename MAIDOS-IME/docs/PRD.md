# MAIDOS-IME -- Product Requirements Document

| Field       | Value          |
|-------------|----------------|
| Product     | MAIDOS-IME     |
| Version     | 0.2.0          |
| Owner       | MAIDOS Team    |
| Status      | Draft          |

## Purpose

AI-powered Input Method Engine for Windows, combining a Rust core with
a C++ TSF (Text Services Framework) front-end to deliver fast keystroke
processing, intelligent candidate generation, and LLM-assisted completion.

## Feature Requirements

| ID     | Feature              | Description                                       | Priority |
|--------|----------------------|---------------------------------------------------|----------|
| FR-001 | Keystroke Processing | Capture and decode raw keystrokes via TSF          | P0       |
| FR-002 | Candidate Generation | Generate ranked candidate list from dictionary+AI  | P0       |
| FR-003 | Dictionary Mgmt      | Load/save user dictionary; CRUD entries             | P1       |
| FR-004 | AI Completion        | LLM-backed sentence completion via maidos-llm      | P1       |
| FR-005 | TSF Integration      | Register as Windows TSF text service               | P0       |

## Acceptance Criteria Summary

- FR-001: keystroke round-trip < 20 ms
- FR-002: candidate list returns within 50 ms
- FR-003: dictionary persists across restarts
- FR-004: AI suggestions appear within 200 ms
- FR-005: activates/deactivates cleanly in any Win32/UWP app

## Out of Scope

- macOS / Linux IME support
- Voice input
- Handwriting recognition

*MAIDOS-IME PRD v0.2.0 -- CodeQC Gate C Compliant*
