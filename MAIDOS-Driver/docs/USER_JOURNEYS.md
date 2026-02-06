# MAIDOS-Driver -- User Journeys

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved

---

## J-001: First Scan -- Discover All Hardware Devices

**Actor**: IT Admin / Power User
**Goal**: View a complete inventory of all hardware devices and their driver status.

| Step | Action | System Response |
|:-----|:-------|:----------------|
| 1 | Launch MAIDOS-Driver | WPF UI loads, scan page displayed |
| 2 | Click "Scan" button | UI calls `ScanAllDevices()` via P/Invoke |
| 3 | Wait for results | Rust `scan_all_devices_c()` enumerates via SetupDI |
| 4 | View device list | Table shows name, vendor, driver version, status for each device |
| 5 | Identify problem devices | Devices with error codes shown with status "Problem (Code N)" |

**AC Coverage**: AC-001, AC-002, AC-003
**NFR**: NFR-001 (< 5s scan time)

---

## J-002: Install Driver from INF

**Actor**: IT Admin
**Goal**: Install a new driver from a downloaded INF file.

| Step | Action | System Response |
|:-----|:-------|:----------------|
| 1 | Navigate to Install page | UI shows file picker |
| 2 | Select INF file | Path validated locally |
| 3 | Click "Install" | System restore point created (AC-005) |
| 4 | Wait for installation | Rust calls `pnputil /add-driver /install` |
| 5 | View result | Success message or error with code and description |

**AC Coverage**: AC-004, AC-005, AC-006
**NFR**: NFR-003 (admin required)

---

## J-003: Check and Apply Driver Update

**Actor**: Power User
**Goal**: Find and install available driver updates.

| Step | Action | System Response |
|:-----|:-------|:----------------|
| 1 | Click "Check Updates" | `CheckAllUpdates()` queries TSV database + Windows Update |
| 2 | View update list | Table shows devices with available updates and download URLs |
| 3 | Select device to update | Highlight row with update_available = true |
| 4 | Click "Download & Install" | BITS downloads driver package; pnputil installs |
| 5 | View result | Updated version confirmed; audit log entry written |

**AC Coverage**: AC-007, AC-008, AC-009, AC-010
**NFR**: NFR-004 (signature verification)

---

## J-004: Rollback Problematic Driver

**Actor**: Power User
**Goal**: Revert a driver update that caused instability.

| Step | Action | System Response |
|:-----|:-------|:----------------|
| 1 | Select device from scan list | Device details shown |
| 2 | Click "Rollback" | Rust `rollback_driver_c()` called |
| 3a | If backup exists | Driver restored from backup via `pnputil /add-driver` |
| 3b | If no backup | `RollbackHandler` attempts system-level rollback |
| 4 | View result | Previous version confirmed or "No previous version available" |

**AC Coverage**: AC-011, AC-012
**NFR**: NFR-003 (admin required)

---

## J-005: Batch Backup All Drivers

**Actor**: IT Admin
**Goal**: Create a complete backup of all OEM drivers before OS migration.

| Step | Action | System Response |
|:-----|:-------|:----------------|
| 1 | Navigate to Backup page | UI shows directory picker |
| 2 | Select backup directory | Path validated; directory created if needed (AC-014) |
| 3 | Click "Backup" | Disk space checked (AC-015); `pnputil /export-driver *` executed |
| 4 | Wait for export | Progress shown; INF files enumerated |
| 5 | View backup summary | Table shows driver name, INF path, size for each exported package |

**AC Coverage**: AC-013, AC-014, AC-015
**NFR**: NFR-001 (< 60s backup time)

---

## J-006: Diagnose Problem Device

**Actor**: IT Admin
**Goal**: Identify why a device is not functioning correctly.

| Step | Action | System Response |
|:-----|:-------|:----------------|
| 1 | Select device marked with problem status | Device highlighted |
| 2 | Click "Diagnose" | Rust `diagnose_device_c()` calls CM_Locate_DevNode + CM_Get_DevNode_Status |
| 3 | View diagnostic results | Problem code, human-readable description, IRQ allocation shown |
| 4 | Check for IRQ conflicts | IRQ number queried via WMI; conflicts flagged if duplicate |
| 5 | Take corrective action | Reinstall driver, disable conflicting device, or update |

**AC Coverage**: AC-016, AC-017, AC-018
**NFR**: NFR-005 (crash-free handling of offline devices)

---

## J-007: View Audit Log

**Actor**: IT Admin
**Goal**: Review history of all driver operations for compliance.

| Step | Action | System Response |
|:-----|:-------|:----------------|
| 1 | Navigate to Audit page | UI displays log entries |
| 2 | View log entries | Each entry: `[MAIDOS-AUDIT] {timestamp} {operation} {device} {result}` |
| 3 | Filter by operation type | Show only INSTALL, UPDATE, BACKUP, ROLLBACK, or SCAN entries |
| 4 | Export log | Save audit.log from `%ProgramData%\MAIDOS\DriverManager\` |

**AC Coverage**: AC-019, AC-020
**Log Location**: `C:\ProgramData\MAIDOS\DriverManager\audit.log`
