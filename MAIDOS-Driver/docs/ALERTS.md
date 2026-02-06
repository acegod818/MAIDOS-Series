# MAIDOS-Driver -- Alert Conditions

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved

---

## 1. Purpose

This document defines the alert conditions that MAIDOS-Driver monitors and reports to the user.
Alerts are surfaced through the WPF UI and recorded in the audit log at
`C:\ProgramData\MAIDOS\DriverManager\audit.log`.

## 2. Alert Severity Levels

| Level | Icon | Description |
|:------|:-----|:------------|
| Critical | Red | Operation failed; user action required immediately |
| Warning | Yellow | Potential issue detected; user should review |
| Info | Blue | Informational; no action required |

## 3. Alert Definitions

### ALERT-001: Driver Install Failure

| Attribute | Value |
|:----------|:------|
| Severity | Critical |
| Trigger | `install_driver_c()` returns -1 |
| Message | "Driver installation failed: {error_message}" |
| User Action | Check INF file validity; verify admin privileges; review error code |
| Audit Entry | `[MAIDOS-AUDIT] INSTALL {device} FAILURE {error_code}` |

### ALERT-002: Insufficient Disk Space

| Attribute | Value |
|:----------|:------|
| Severity | Critical |
| Trigger | Free disk space < MIN_FREE_SPACE_BYTES before backup or download |
| Message | "Insufficient disk space for operation. Required: {N} MB, Available: {M} MB" |
| User Action | Free disk space or select a different target directory |
| AC Reference | AC-015 |

### ALERT-003: Unsigned Driver Detected

| Attribute | Value |
|:----------|:------|
| Severity | Warning |
| Trigger | WinVerifyTrust returns non-zero for a downloaded driver package |
| Message | "Driver signature verification failed. The driver may be unsigned or tampered." |
| User Action | Obtain the driver from the official vendor; do not install unsigned drivers |
| NFR Reference | NFR-004 |

### ALERT-004: Device Problem Code Detected

| Attribute | Value |
|:----------|:------|
| Severity | Warning |
| Trigger | CM_Get_DevNode_Status returns a non-zero problem code during scan |
| Message | "Device '{name}' has problem code {N}: {description}" |
| User Action | Run diagnostics; consider reinstalling or updating the driver |
| AC Reference | AC-002, AC-016 |

### ALERT-005: Download Failure

| Attribute | Value |
|:----------|:------|
| Severity | Critical |
| Trigger | `download_update_c()` returns -1 (BITS failure or network error) |
| Message | "Driver download failed: {error_message}" |
| User Action | Check network connectivity; verify BITS service is running |

### ALERT-006: IRQ Conflict Detected

| Attribute | Value |
|:----------|:------|
| Severity | Warning |
| Trigger | Two or more devices share the same IRQ number (WMI query result) |
| Message | "IRQ conflict on IRQ {N}: devices '{A}' and '{B}' share the same interrupt" |
| User Action | Disable one device, update BIOS settings, or reassign resources |
| AC Reference | AC-017 |

### ALERT-007: Rollback Unavailable

| Attribute | Value |
|:----------|:------|
| Severity | Info |
| Trigger | `rollback_driver_c()` finds no previous driver version or backup |
| Message | "No previous driver version available for rollback." |
| User Action | Create a backup before updating in the future |
| AC Reference | AC-012 |

### ALERT-008: Database Load Failure

| Attribute | Value |
|:----------|:------|
| Severity | Warning |
| Trigger | TSV database file missing or malformed at startup |
| Message | "Driver database not found or corrupted. Update checks will use Windows Update only." |
| User Action | Reinstall or replace `data\drivers.tsv` |

## 4. Alert Routing

All alerts are:
1. Displayed in the WPF UI as a dialog or status bar notification.
2. Written to the audit log with severity, timestamp, and details.
3. Available for review on the Audit page with filtering by severity level.
