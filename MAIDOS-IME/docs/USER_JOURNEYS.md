# MAIDOS-IME -- User Journeys

| Field   | Value        |
|---------|--------------|
| Product | MAIDOS-IME   |
| Version | 0.2.0        |

---

## J-001: Basic Typing

**Actor:** End user
**Trigger:** User switches to MAIDOS-IME and starts typing.
1. User presses language toggle hotkey
2. MAIDOS-IME activates via TSF
3. User types phonetic keys; candidate window appears
4. User presses number key or Enter to commit
**Outcome:** Selected text inserted into application

---

## J-002: AI-Assisted Completion

**Actor:** End user
**Trigger:** User pauses mid-sentence.
1. User types partial sentence
2. After 300 ms idle, maidos-llm generates completion
3. Ghost text appears in candidate window
4. User presses Tab to accept or continues typing to dismiss
**Outcome:** Full sentence committed with fewer keystrokes

---

## J-003: Dictionary Management

**Actor:** Power user
**Trigger:** User wants to add a custom phrase.
1. User opens IME settings tray menu
2. Selects "Add phrase"; enters reading and phrase
3. Dictionary file updated on disk
**Outcome:** Custom phrase appears in future candidate lists

---

## J-004: First-Time Setup

**Actor:** New user
**Trigger:** User installs MAIDOS-IME.
1. Installer registers TSF COM DLL
2. User opens Settings > Language > Preferred keyboards
3. Selects MAIDOS-IME; first activation triggers index build
**Outcome:** IME ready for use within 5 seconds

*MAIDOS-IME USER_JOURNEYS v0.2.0 -- CodeQC Gate C Compliant*
