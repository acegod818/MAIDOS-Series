# MAIDOS-IME v2.0 - Deployment Document

## 1. Purpose

This document describes the deployment procedures for MAIDOS-IME, including COM DLL
registration, file distribution, and system requirements.

## 2. Prerequisites

| Requirement | Details |
|-------------|---------|
| OS | Windows 10 version 1903+ or Windows 11 |
| Architecture | x86-64 |
| .NET Runtime | .NET 8.0 Desktop Runtime (bundled or pre-installed) |
| Privileges | Administrator (required for COM registration) |
| Disk Space | 120 MB minimum |
| Optional | Ollama (for AI-assisted selection) |

## 3. Installation Package

The installer is built as a WiX v3.14 MSI package containing:

| Component | Target Path |
|-----------|-------------|
| `maidOS_ime_tsf.dll` | `C:\Program Files\MAIDOS\IME\` |
| `maidOS_ime.dll` | `C:\Program Files\MAIDOS\IME\` |
| `MAIDOS.IME.Manager.exe` | `C:\Program Files\MAIDOS\IME\` |
| Dictionary JSON files | `C:\Program Files\MAIDOS\IME\data\dictionaries\` |
| Default config | `C:\Program Files\MAIDOS\IME\config\default.json` |

## 4. COM DLL Registration

The TSF COM DLL must be registered system-wide for the IME to appear in Windows input
method settings.

### 4.1 Registration (during install)

```cmd
regsvr32 /s "C:\Program Files\MAIDOS\IME\maidOS_ime_tsf.dll"
```

### 4.2 Unregistration (during uninstall)

```cmd
regsvr32 /u /s "C:\Program Files\MAIDOS\IME\maidOS_ime_tsf.dll"
```

### 4.3 Registry Entries

Registration creates entries under:
- `HKLM\SOFTWARE\Microsoft\CTF\TIP\{CLSID}` - TSF Text Input Processor registration.
- `HKCR\CLSID\{CLSID}` - Standard COM class registration.

## 5. Dictionary File Distribution

- Dictionary JSON files are bundled in the MSI.
- Binary index caches are generated on first launch (1-3 second one-time cost).
- Dictionary updates are downloaded by the C# Manager from the MAIDOS repository.
- Updated dictionaries are placed in `%ProgramData%\MAIDOS\IME\dictionaries\`.

## 6. User Data Directories

| Purpose | Path |
|---------|------|
| User dictionary | `%AppData%\MAIDOS\IME\user_dict.json` |
| User settings | `%AppData%\MAIDOS\IME\settings.json` |
| Dictionary cache | `%LocalAppData%\MAIDOS\IME\cache\` |
| Logs | `%LocalAppData%\MAIDOS\IME\logs\` |

## 7. Upgrade Procedure

1. MSI upgrade (major upgrade): Unregisters old COM DLL, removes old files, installs
   new version, re-registers COM DLL.
2. Dictionary-only update: C# Manager downloads new JSON files and triggers cache rebuild.

## 8. Uninstallation

1. MSI uninstall unregisters the COM DLL.
2. Program files are removed.
3. User data in `%AppData%` and `%LocalAppData%` is preserved (user choice in UI).

## 9. Verification

After installation, verify:
- IME appears in Windows Settings > Time & Language > Language > Keyboard.
- Typing in Notepad with the IME active produces candidate windows.
- Manager application launches from Start Menu.

## 10. References

- RUNBOOK.md - Operational procedures and troubleshooting
- ARCHITECTURE.md - System component overview
