# MAIDOS-IME v2.0 - State Model

## 1. Purpose

This document defines the finite state machine governing the MAIDOS-IME engine's
behavior during input processing.

## 2. States

| State | ID | Description |
|-------|----|-------------|
| Idle | S0 | No active composition. Keystrokes pass through to application. |
| Composing | S1 | User is building an input sequence (phonetic, shape, or romaji). |
| Candidate Selection | S2 | Candidate list is displayed. User is choosing a candidate. |
| AI Processing | S3 | Ollama LLM is re-ranking candidates. Candidate list visible. |
| Scheme Switching | S4 | Scheme selector is open. Input is paused. |

## 3. State Transitions

```
          keystroke (input key)
  [S0 Idle] -----------------------> [S1 Composing]
      ^                                    |
      |  Esc / focus loss                  | completion trigger
      |                                    v
      |                          [S2 Candidate Selection]
      |                            |              |
      |  commit (number/Enter)     |              | LLM enabled
      |<---------------------------|              v
      |                                  [S3 AI Processing]
      |  commit after AI                       |
      |<---------------------------------------|
      |                                        | timeout (2s)
      |<---------------------------------------|
      |
      |  Ctrl+Shift+Space
      |----------------------------> [S4 Scheme Switching]
      |<---------------------------- scheme selected / Esc
```

## 4. Transition Table

| From | To | Trigger | Action |
|------|----|---------|--------|
| S0 | S1 | Input key pressed | Open composition buffer |
| S1 | S1 | Additional input key | Append to composition buffer |
| S1 | S2 | Space / completion trigger | Generate candidate list |
| S1 | S0 | Esc pressed | Discard composition buffer |
| S1 | S0 | Focus lost | Discard composition buffer |
| S2 | S0 | Number key / Enter | Commit selected candidate |
| S2 | S1 | Backspace | Return to composing, remove last input |
| S2 | S3 | LLM enabled and candidates ready | Send to Ollama for re-ranking |
| S2 | S0 | Esc pressed | Discard composition and candidates |
| S3 | S2 | LLM response received | Update candidate order, return to selection |
| S3 | S2 | LLM timeout (2 s) | Keep original order, return to selection |
| S3 | S2 | LLM error | Keep original order, return to selection |
| S0 | S4 | Ctrl+Shift+Space | Open scheme selector |
| S4 | S0 | Scheme selected | Apply new scheme, close selector |
| S4 | S0 | Esc pressed | Keep current scheme, close selector |

## 5. State Invariants

- **S0 (Idle)**: Composition buffer is empty. No candidate window visible.
- **S1 (Composing)**: Composition buffer contains at least one character. No candidates.
- **S2 (Candidate Selection)**: Candidate list has at least one entry. Buffer is frozen.
- **S3 (AI Processing)**: HTTP request to Ollama is in-flight. Timer running.
- **S4 (Scheme Switching)**: All input is intercepted by the scheme selector popup.

## 6. Error Recovery

- Any unhandled exception in S1-S3 transitions the engine to S0 (Idle) with buffer
  discarded. An error is logged but no crash propagates to the host application.
- If the C++ TSF layer detects the Rust engine is unresponsive (>5 s), it force-resets
  the engine state to S0.

## 7. References

- ARCHITECTURE.md - System layers
- CONTRACT.md - FFI functions that trigger transitions
