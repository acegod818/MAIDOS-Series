# Alerting Guide - MAIDOS-Driver

## 1. Overview

MAIDOS-Driver generates alerts and warnings to inform users about conditions requiring attention. Alerts are presented through the UI and logged to the engine log files. This document catalogs all alert types, their severity levels, trigger conditions, and recommended responses.

## 2. Alert Severity Levels

| Level | Display | Meaning |
|-------|---------|---------|
| **Critical** | Red banner with dialog | Operation blocked, immediate attention required |
| **Warning** | Yellow banner | Operation may proceed but with risks |
| **Info** | Blue notification | Informational, no action required |

## 3. Alert Catalog

### 3.1 Driver Signature Alerts

#### ALERT-SIG-001: Unsigned Driver Detected
- **Severity**: Critical
- **Trigger**: verify_driver_signature() returns non-zero for a driver package
- **Message**: Driver package [name] is not digitally signed. Installation blocked.
- **Response**: Do not install the driver. Obtain a signed version from the manufacturer.

#### ALERT-SIG-002: Expired Certificate
- **Severity**: Warning
- **Trigger**: WinVerifyTrust returns certificate-expired status
- **Message**: Driver [name] has an expired certificate. Proceed with caution.
- **Response**: Verify the driver source. Consider obtaining an updated version.

#### ALERT-SIG-003: Tampered Driver File
- **Severity**: Critical
- **Trigger**: WinVerifyTrust detects signature/content mismatch
- **Message**: Driver file [name] appears to be tampered. Installation blocked.
- **Response**: Delete the file immediately. Re-download from the official source.

### 3.2 Driver Mismatch Alerts

#### ALERT-DRV-001: Unknown Device
- **Severity**: Info
- **Trigger**: Device detected but no VEN/DEV match in TSV database or Windows Update
- **Message**: Device [hardware_id] has no known driver. Manual installation may be needed.
- **Response**: Check the device manufacturer website for drivers.

#### ALERT-DRV-002: Outdated Driver Version
- **Severity**: Warning
- **Trigger**: Installed driver version is older than the database version
- **Message**: Device [name] has driver version [current] but [available] is available.
- **Response**: Consider updating to the newer version via the Update tab.

#### ALERT-DRV-003: Driver Downgrade Detected
- **Severity**: Warning
- **Trigger**: User attempts to install a driver with a lower version than currently installed
- **Message**: The selected driver [version] is older than the installed driver [version].
- **Response**: Confirm the downgrade is intentional. Downgrades may cause instability.

### 3.3 Download Alerts

#### ALERT-DL-001: SHA-256 Checksum Failure
- **Severity**: Critical
- **Trigger**: Downloaded file SHA-256 does not match expected value
- **Message**: Downloaded package failed integrity check. The file may be corrupted or tampered.
- **Response**: Retry the download. If persistent, report the issue.

#### ALERT-DL-002: HTTP URL Rejected
- **Severity**: Critical
- **Trigger**: A download URL uses HTTP instead of HTTPS
- **Message**: Download blocked: URL does not use HTTPS. Insecure downloads are not permitted.
- **Response**: Verify the URL in the driver database. Only HTTPS URLs are accepted.

#### ALERT-DL-003: BITS Transfer Stalled
- **Severity**: Warning
- **Trigger**: BITS reports a transient error or transfer stall exceeding 60 seconds
- **Message**: Download stalled. Retrying automatically (attempt [n] of 3).
- **Response**: Check network connectivity. Downloads will retry automatically.

#### ALERT-DL-004: Download Timeout
- **Severity**: Warning
- **Trigger**: BITS transfer exceeds the configured timeout (default 10 minutes)
- **Message**: Download timed out for [package]. Check your network connection.
- **Response**: Retry manually or check network/proxy settings.

### 3.4 System Alerts

#### ALERT-SYS-001: Administrator Privileges Required
- **Severity**: Warning
- **Trigger**: User attempts a driver installation without admin rights
- **Message**: Administrator privileges are required to install drivers.
- **Response**: Right-click the application and select Run as administrator.

#### ALERT-SYS-002: WMI Service Unavailable
- **Severity**: Critical
- **Trigger**: WMI COM initialization fails or winmgmt service is not running
- **Message**: Windows Management Instrumentation service is not available.
- **Response**: Start the WMI service: net start winmgmt. See RUNBOOK.md section 3.1.

#### ALERT-SYS-003: Database File Missing
- **Severity**: Warning
- **Trigger**: drivers.tsv not found at either expected path
- **Message**: Driver database not found. Update checking will be limited to Windows Update.
- **Response**: Reinstall the application or restore the database file.

#### ALERT-SYS-004: High Memory Usage
- **Severity**: Warning
- **Trigger**: Process working set exceeds 200 MB
- **Message**: Application memory usage is elevated. Consider restarting the application.
- **Response**: Restart the application. If persistent, report as a potential memory leak.

### 3.5 Backup and Restore Alerts

#### ALERT-BKP-001: Backup Integrity Failure
- **Severity**: Critical
- **Trigger**: Restore validation finds checksum mismatch in backup ZIP
- **Message**: Backup file is corrupted. Cannot safely restore drivers from this backup.
- **Response**: Use a different backup file. Create a new backup if possible.

#### ALERT-BKP-002: Partial Restore
- **Severity**: Warning
- **Trigger**: Some drivers in a restore operation failed to install
- **Message**: [n] of [total] drivers could not be restored. See details for failures.
- **Response**: Review failures and install failed drivers manually.

## 4. Alert Routing

All alerts are:
1. Displayed in the application UI as banner notifications or modal dialogs
2. Written to the engine log file at %ProgramData%\MAIDOS\DriverManager\logs\
3. Critical alerts also surface in the Windows Event Log (Application source)

## 5. Log Format

```
[YYYY-MM-DD HH:MM:SS] [LEVEL] [ALERT-CODE] Message details
```

Example entries:
```
[2026-02-06 14:30:22] [CRITICAL] [ALERT-SIG-001] Unsigned driver blocked: C:	emp\driver.inf
[2026-02-06 14:30:23] [WARN] [ALERT-DRV-002] Outdated driver: PCI\VEN_10EC&DEV_8168
[2026-02-06 14:31:05] [INFO] [ALERT-DRV-001] No driver match: USB\VID_1234&PID_5678
```
