# MAIDOS-IME v2.0 - Operations Runbook

## 1. Purpose

This runbook provides step-by-step procedures for installing, configuring, and
troubleshooting MAIDOS-IME in production environments.

## 2. Installation

### 2.1 Standard Installation

1. Obtain `MAIDOS-IME-v2.0.msi` from the official distribution channel.
2. Right-click the MSI and select "Run as administrator".
3. Follow the installer wizard. Default install path: `C:\Program Files\MAIDOS\IME\`.
4. After completion, open Windows Settings > Time & Language > Language.
5. Add MAIDOS-IME as a keyboard input method for your language.

### 2.2 Silent Installation

```cmd
msiexec /i MAIDOS-IME-v2.0.msi /quiet /norestart
```

## 3. TSF Registration Verification

```cmd
reg query "HKLM\SOFTWARE\Microsoft\CTF\TIP" /s | findstr MAIDOS
```

If MAIDOS entries appear, registration is correct. If not, re-register manually:

```cmd
regsvr32 "C:\Program Files\MAIDOS\IME\maidOS_ime_tsf.dll"
```

## 4. Ollama Setup (Optional)

1. Install Ollama from https://ollama.ai.
2. Pull a lightweight model: `ollama pull qwen2:1.5b` (or preferred model).
3. Verify Ollama is running: `curl http://127.0.0.1:11434/api/tags`.
4. Enable AI in MAIDOS-IME Manager: Settings > AI Assistance > Enable.

## 5. Common Troubleshooting

### 5.1 IME Does Not Appear in Windows Settings

- **Cause**: COM DLL not registered or registration corrupted.
- **Fix**: Run `regsvr32 "C:\Program Files\MAIDOS\IME\maidOS_ime_tsf.dll"` as admin.
- **Verify**: Check registry as described in Section 3.

### 5.2 No Candidates Appear When Typing

- **Cause**: Dictionary files missing or corrupted.
- **Fix**: Open MAIDOS-IME Manager > Dictionaries > Rebuild Cache.
- **Verify**: Check that JSON files exist in `C:\Program Files\MAIDOS\IME\data\dictionaries\`.

### 5.3 AI Selection Not Working

- **Cause**: Ollama not running or wrong port.
- **Fix**: Ensure Ollama is running (`ollama serve`). Check port 11434 is accessible.
- **Note**: AI is optional. The IME functions fully without it using dictionary ranking.

### 5.4 High Memory Usage

- **Cause**: Large dictionaries loaded simultaneously.
- **Fix**: In Manager, disable input schemes you do not use. This unloads their dictionaries.
- **Expected**: Normal operation should remain below 80 MB (NFR-003).

### 5.5 Input Lag

- **Cause**: AI timeout too long or dictionary cache not built.
- **Fix**: Rebuild dictionary cache. Reduce LLM timeout (Settings > AI > Timeout).
- **Expected**: Non-AI input should be under 50 ms (NFR-001).

## 6. Log Locations

| Log | Path |
|-----|------|
| Engine log | `%LocalAppData%\MAIDOS\IME\logs\engine.log` |
| TSF layer log | `%LocalAppData%\MAIDOS\IME\logs\tsf.log` |
| Manager log | `%LocalAppData%\MAIDOS\IME\logs\manager.log` |

Log rotation: Files rotate at 10 MB, keeping 3 archives.

## 7. Health Check

Run the built-in diagnostic from the Manager:

```
MAIDOS.IME.Manager.exe --diag
```

This checks: COM registration, dictionary integrity, Rust DLL load, and Ollama
connectivity. Output is written to stdout and to the manager log.

## 8. References

- DEPLOY.md - Deployment procedures
- ALERTS.md - Alert conditions and responses
- SLO.md - Service level objectives
