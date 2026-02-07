# MAIDOS-IME -- State Model

| Field   | Value        |
|---------|--------------|
| Product | MAIDOS-IME   |
| Version | 0.2.0        |

## States

| State      | Description                                  |
|------------|----------------------------------------------|
| Idle       | IME active but no composition in progress    |
| Composing  | User is typing; raw buffer accumulating      |
| Candidates | Candidate window visible; awaiting selection |
| Committed  | Selected text committed to application       |

## Transitions

```
            key press
  Idle -----------------> Composing
   ^                         |
   |                         | buffer updated
   |                         v
   |                     Candidates
   |                      /        |           select   /       \ Esc / backspace
   |                   v         v
  Committed <------          Composing (or Idle if empty)
      |
      | (auto)
      v
    Idle
```

## Transition Rules

| From       | To         | Trigger                           |
|------------|------------|-----------------------------------|
| Idle       | Composing  | Printable key pressed             |
| Composing  | Candidates | Buffer non-empty + lookup done    |
| Candidates | Committed  | User selects candidate (num/Enter)|
| Candidates | Composing  | Backspace (shorten buffer)        |
| Candidates | Idle       | Escape pressed                    |
| Committed  | Idle       | Automatic after commit            |
| Composing  | Idle       | All chars deleted                 |

## Error States

- If Rust FFI returns error code, transition to Idle and log error.
- If TSF deactivates mid-composition, discard buffer silently.

*MAIDOS-IME STATE_MODEL v0.2.0 -- CodeQC Gate C Compliant*
