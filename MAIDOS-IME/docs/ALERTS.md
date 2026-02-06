# MAIDOS-IME v2.0 - Alert Conditions

## 1. Purpose

This document defines the alert conditions that the MAIDOS-IME system monitors. When
triggered, these alerts surface in the Manager UI and are logged for troubleshooting.

## 2. Alert Severity Levels

| Level | Description | User Impact |
|-------|-------------|-------------|
| Critical | IME cannot function | Input method unavailable |
| Warning | Degraded functionality | Some features unavailable |
| Info | Operational notice | No immediate impact |

## 3. Alert Definitions

### ALT-001: COM Registration Failure

- **Severity**: Critical
- **Condition**: TSF COM DLL is not registered or registry entries are missing.
- **Detection**: Manager startup check queries `HKLM\SOFTWARE\Microsoft\CTF\TIP` for
  MAIDOS-IME CLSID.
- **Impact**: IME does not appear in Windows input methods. No input processing.
- **Response**: Re-register via `regsvr32` as administrator. See RUNBOOK.md Section 5.1.

### ALT-002: Dictionary Load Failure

- **Severity**: Critical
- **Condition**: One or more required dictionary JSON files are missing, unreadable,
  or fail JSON parsing.
- **Detection**: `ime_init` returns error code -5. Engine logs detail the failing file.
- **Impact**: Affected input schemes produce no candidates.
- **Response**: Reinstall dictionaries via Manager > Dictionaries > Repair. If files are
  corrupted, re-download from MAIDOS repository.

### ALT-003: Rust Engine DLL Load Failure

- **Severity**: Critical
- **Condition**: C++ TSF layer cannot load `maidOS_ime.dll` via `LoadLibrary`.
- **Detection**: TSF layer logs `DLL_LOAD_FAILED` with Windows error code.
- **Impact**: IME is registered but non-functional. Keystrokes pass through unprocessed.
- **Response**: Verify DLL exists at install path. Check for missing VC++ Runtime
  dependencies. Reinstall if necessary.

### ALT-004: Ollama Unreachable

- **Severity**: Warning
- **Condition**: HTTP connection to `127.0.0.1:11434` fails or times out.
- **Detection**: Rust engine HTTP client receives connection refused or timeout.
- **Impact**: AI-assisted candidate selection is unavailable. Dictionary-only ranking
  is used as fallback. Core input functionality is not affected.
- **Response**: Verify Ollama is running (`ollama serve`). Check no firewall blocks
  localhost port 11434. This alert auto-clears when Ollama becomes reachable.

### ALT-005: AI Latency Exceeds SLO

- **Severity**: Warning
- **Condition**: More than 10% of AI requests in a 5-minute window exceed the 2-second
  timeout threshold.
- **Detection**: Rust engine tracks timeout rate in rolling window.
- **Impact**: Frequent fallback to dictionary-only ranking. AI benefit is diminished.
- **Response**: Consider using a smaller Ollama model. Check system resource availability.
  Increase timeout if acceptable (Settings > AI > Timeout).

### ALT-006: Memory Threshold Exceeded

- **Severity**: Warning
- **Condition**: Engine working set exceeds 80 MB (NFR-003 threshold).
- **Detection**: Manager periodically queries process working set via Win32 API.
- **Impact**: Potential system resource pressure on low-memory machines.
- **Response**: Disable unused input schemes to reduce loaded dictionaries. Restart the
  IME to clear any accumulated buffers. Report if persistent (possible memory leak).

### ALT-007: Dictionary Cache Stale

- **Severity**: Info
- **Condition**: Binary cache file timestamp is older than the source JSON dictionary.
- **Detection**: Engine compares file timestamps during `ime_init`.
- **Impact**: Slightly slower first-query performance until cache is rebuilt.
- **Response**: Automatic. Engine rebuilds cache on detection. No user action required.

## 4. Alert Delivery

- All alerts are written to log files (see RUNBOOK.md Section 6).
- Critical and Warning alerts display a notification in the Manager system tray icon.
- The Manager Settings > Status page shows current alert state.

## 5. References

- SLO.md - Service level objectives that trigger alerts
- RUNBOOK.md - Response procedures for each alert
