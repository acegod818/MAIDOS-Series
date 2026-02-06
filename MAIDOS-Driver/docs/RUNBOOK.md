# MAIDOS-Driver -- Operations Runbook

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved

---

## 1. Installation

1. Obtain `MAIDOS-Driver.msi` from the official distribution channel.
2. Right-click the MSI and select "Install", or run from an elevated command prompt:
   ```
   msiexec /i MAIDOS-Driver.msi /qn
   ```
3. Accept the UAC prompt. Installation takes approximately 10 seconds.
4. Verify installation: launch MAIDOS-Driver from the Start menu. The scan page should appear.
5. Confirm the Rust engine loaded: click "Scan". If devices appear, the DLL is functional.

## 2. First Run Checklist

- [ ] Application launches without error.
- [ ] Hardware scan returns a non-empty device list.
- [ ] Audit log file exists at `C:\ProgramData\MAIDOS\DriverManager\audit.log`.
- [ ] Driver database loaded from `data\drivers.tsv` in the installation directory.

## 3. Common Operations

### 3.1 Full Hardware Scan

Click "Scan" on the main page. Expected completion time: < 5 seconds for typical systems
(< 500 devices). Results display device name, vendor, driver version, and status.

### 3.2 Driver Update Workflow

1. Click "Check Updates" to query the TSV database and Windows Update.
2. Review the list of devices with available updates.
3. For TSV-matched updates: click "Download & Install" for automatic BITS download and pnputil install.
4. For Windows Update-only matches: click the link to open `ms-settings:windowsupdate`.

### 3.3 Driver Backup

1. Navigate to the Backup page. Select a target directory.
2. Click "Backup". The tool runs `pnputil /export-driver *` to the selected directory.
3. Verify the backup summary table shows all exported INF files.

## 4. Troubleshooting

### 4.1 Application Fails to Launch

| Symptom | Cause | Resolution |
|:--------|:------|:-----------|
| "maidOS_driver.dll not found" | DLL missing from install directory | Reinstall MSI or manually copy DLL |
| Crash on startup | Corrupted .NET runtime | Run `dotnet --info` to verify; reinstall MSI |
| No UAC prompt, operations fail | Not running as administrator | Right-click, "Run as administrator" |

### 4.2 Scan Returns Zero Devices

| Symptom | Cause | Resolution |
|:--------|:------|:-----------|
| Empty device list | SetupDI API failure | Check Windows Event Log for setupapi errors |
| "Access denied" error | Insufficient permissions for WMI | Run as administrator |
| Partial device list | WMI STA thread timeout | Restart application; check WMI service |

### 4.3 Driver Install Fails

| Symptom | Cause | Resolution |
|:--------|:------|:-----------|
| "Invalid INF" error | Malformed or unsigned INF file | Verify INF with `pnputil /add-driver` manually |
| "Access denied" | Not elevated | Restart with admin privileges |
| Signature verification failed | Unsigned or tampered driver | Obtain driver from official vendor |

### 4.4 Download Fails

| Symptom | Cause | Resolution |
|:--------|:------|:-----------|
| BITS error | Network unavailable or proxy blocking | Check network; verify BITS service is running |
| URL not in whitelist | Untrusted download source | Add source to trusted whitelist or use manual download |
| SHA-256 mismatch | Corrupted download | Delete and retry download |

## 5. Log Locations

| Log | Path |
|:----|:-----|
| Audit log | `C:\ProgramData\MAIDOS\DriverManager\audit.log` |
| Windows Event Log | `eventvwr.msc` > Application > Source: MAIDOS-Driver |
| Rust panic log | stderr (visible when launched from command line) |

## 6. Maintenance

- **Database update**: Replace `data\drivers.tsv` with the latest version and restart.
- **Log rotation**: Audit log grows indefinitely; archive or truncate periodically.
- **MSI upgrade**: Install new MSI over existing installation; settings are preserved.
