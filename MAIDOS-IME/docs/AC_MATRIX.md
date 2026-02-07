# MAIDOS-IME -- Acceptance Criteria Matrix

| Field   | Value        |
|---------|--------------|
| Product | MAIDOS-IME   |
| Version | 0.2.0        |

## Matrix

| AC-ID  | FR     | Criterion                                            | Gate |
|--------|--------|------------------------------------------------------|------|
| AC-001 | FR-001 | Keystroke event decoded within 20 ms                 | G1   |
| AC-002 | FR-001 | All printable ASCII keys mapped correctly             | G1   |
| AC-003 | FR-002 | Candidate list returns >= 1 result for known input    | G2   |
| AC-004 | FR-002 | Candidates ranked by frequency descending             | G2   |
| AC-005 | FR-002 | Empty input returns empty candidate list              | G1   |
| AC-006 | FR-003 | Dictionary loads from disk on init                    | G2   |
| AC-007 | FR-003 | New word persists after add + restart                 | G2   |
| AC-008 | FR-003 | Delete word removes from dictionary file              | G2   |
| AC-009 | FR-004 | AI completion returns suggestion for partial sentence | G3   |
| AC-010 | FR-004 | AI timeout falls back to dictionary-only results      | G3   |
| AC-011 | FR-004 | AI disabled flag skips LLM call entirely              | G2   |
| AC-012 | FR-005 | TSF registers with Windows input service manager      | G3   |
| AC-013 | FR-005 | Activate/Deactivate cycle completes without error     | G3   |
| AC-014 | FR-005 | Composition renders inline in target application      | G4   |
| AC-015 | FR-005 | IME works in Notepad, Chrome, VS Code                 | G4   |

## Gate Legend

- **G1** -- Unit tests
- **G2** -- Integration tests
- **G3** -- System / E2E tests
- **G4** -- Manual / smoke validation

*MAIDOS-IME AC_MATRIX v0.2.0 -- CodeQC Gate C Compliant*
