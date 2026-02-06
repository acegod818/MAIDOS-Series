# MAIDOS-Driver -- Acceptance Criteria Matrix

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved

---

## AC Matrix

| AC | FR | Description | Test Type | Pass Criteria |
|:---|:---|:------------|:----------|:--------------|
| AC-001 | FR-001 | Scan lists N devices with non-empty name and vendor | Integration | Device count > 0; every entry has non-empty name |
| AC-002 | FR-001 | Problem devices show problem code in status field | Integration | Devices with CM problem code display "Problem (Code N)" |
| AC-003 | FR-001 | Scan completes in < 5 seconds for <= 500 devices | Benchmark | Wall-clock time < 5000 ms |
| AC-004 | FR-002 | Valid INF + admin installs driver successfully | Integration | pnputil returns success; Device Manager shows new driver |
| AC-005 | FR-002 | System restore point created before installation | Integration | Checkpoint-Computer creates restore point before pnputil call |
| AC-006 | FR-002 | Invalid INF returns error code, no crash | Unit | Return value = -1; get_last_error() provides message |
| AC-007 | FR-003 | Device with update returns UpdateInfo with versions | Integration | UpdateInfo.update_available = true; current_version != latest_version |
| AC-008 | FR-003 | Valid URL downloads file, returns byte count | Integration | Return value > 0; file exists at target path |
| AC-009 | FR-003 | Downloaded INF applied, device updated | Integration | pnputil installs; driver version changes |
| AC-010 | FR-003 | Batch check returns all updatable devices | Integration | Array length matches devices with available updates |
| AC-011 | FR-004 | Device with previous version rolls back successfully | Integration | Driver version reverts to previous |
| AC-012 | FR-004 | No previous version returns "no rollback available" | Unit | Return value = -1; error message describes no previous version |
| AC-013 | FR-005 | Backup creates INF files and returns BackupInfo array | Integration | Backup directory contains INF files; array length > 0 |
| AC-014 | FR-005 | Non-existent backup directory auto-created | Unit | Directory created via create_dir_all; no error |
| AC-015 | FR-005 | Insufficient disk space returns error, no partial files | Unit | Return value = -1; no partial files in target directory |
| AC-016 | FR-006 | Problem code 28 returns correct description | Unit | description contains "driver is not installed" |
| AC-017 | FR-006 | IRQ conflict detected and reported | Integration | Diagnostic output lists conflicting IRQ number and device names |
| AC-018 | FR-006 | All devices normal returns "Running normally" | Unit | Status field = "Running normally" when no problem code |
| AC-019 | FR-007 | Every operation writes audit log entry | Unit | Log file contains `[MAIDOS-AUDIT]` entry after each operation |
| AC-020 | FR-007 | Failed operations log error code and reason | Unit | Log entry includes error code and human-readable reason |

## Summary by FR

| FR | AC Count | Rust Test Location |
|:---|:---------|:-------------------|
| FR-001 Hardware Scan | 3 (AC-001 to AC-003) | `src/core/detect/hardware.rs`, `tests/hardware_detection_test.rs` |
| FR-002 Driver Install | 3 (AC-004 to AC-006) | `src/core/install/installer.rs`, `src/ffi.rs` |
| FR-003 Driver Update | 4 (AC-007 to AC-010) | `src/core/update/checker.rs`, `src/core/download/downloader.rs` |
| FR-004 Driver Rollback | 2 (AC-011 to AC-012) | `src/core/restore/manager.rs`, `src/core/install/rollback_handler.rs` |
| FR-005 Driver Backup | 3 (AC-013 to AC-015) | `src/core/backup/manager.rs` |
| FR-006 Diagnostics | 3 (AC-016 to AC-018) | `src/core/diagnose/device_diagnostics.rs` |
| FR-007 Audit Logging | 2 (AC-019 to AC-020) | `src/core/audit.rs` |

**Total**: 20 acceptance criteria across 7 functional requirements.
