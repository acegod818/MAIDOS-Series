# MAIDOS-IME -- Runbook

| Field   | Value        |
|---------|--------------|
| Product | MAIDOS-IME   |
| Version | 0.2.0        |

## Setup

1. Install via MSI or manual registration (`regsvr32`)
2. Settings > Time & Language > Keyboard > Add MAIDOS-IME
3. Toggle with Win+Space

## Health Check

```bat
reg query "HKLM\SOFTWARE\Classes\CLSID\{MAIDOS-IME-CLSID}" /s
tasklist | findstr maidos
```

## Common Troubleshooting

| Symptom                  | Cause                | Fix                              |
|--------------------------|----------------------|----------------------------------|
| IME not in keyboard list | COM not registered   | `regsvr32 maidos_ime.dll` (admin)|
| No candidates appear     | Dictionary missing   | Re-run first-time index build    |
| AI suggestions missing   | Model not downloaded | Run `maidos-ime --download-model`|
| High latency (> 100 ms) | Debug build in use   | Switch to `--release` build      |
| Crash on activate        | DLL version mismatch | Rebuild both Rust + C++ DLLs     |

## Log Locations

- Rust core: `%APPDATA%\MAIDOS\IME\logs\core.log`
- TSF layer: `%APPDATA%\MAIDOS\IME\logs	sf.log`

## Emergency: Disable IME

```bat
regsvr32 /u /s maidos_ime.dll
```

*MAIDOS-IME RUNBOOK v0.2.0 -- CodeQC Gate C Compliant*
