# MAIDOS-Driver -- Backup and Disaster Recovery

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved

---

## 1. Purpose

This document defines the backup strategy and disaster recovery procedures for systems managed
by MAIDOS-Driver. The focus is on preserving driver state so that a system can be restored to
a known-good configuration after a failed update, hardware change, or OS reinstallation.

## 2. Backup Strategy

### 2.1 Driver Backup

| Aspect | Details |
|:-------|:--------|
| Tool | `pnputil /export-driver * {backup_dir}` via `backup_drivers_c()` |
| Scope | All OEM (third-party) drivers in the Windows driver store |
| Output | One subdirectory per driver containing INF file and associated binaries |
| Trigger | Manual (user clicks "Backup") or pre-update automatic backup |
| Storage | User-specified directory (local disk, network share, or external drive) |
| AC Reference | AC-013, AC-014, AC-015 |

### 2.2 Pre-Operation Backups

Before any destructive operation (install, update, rollback), MAIDOS-Driver creates:

1. **System Restore Point**: Via `Checkpoint-Computer` PowerShell cmdlet (AC-005).
2. **Driver-specific backup**: The current driver is exported to a temporary directory before
   replacement. This enables targeted rollback without a full system restore.

### 2.3 Backup Validation

| Check | Method |
|:------|:-------|
| Disk space | `check_disk_space()` verifies MIN_FREE_SPACE_BYTES before backup (AC-015) |
| File integrity | Backup entry count matches expected OEM driver count |
| INF presence | Each backup subdirectory must contain at least one `.inf` file |
| Size verification | `CBackupEntry.size_bytes` is non-zero for every exported driver |

## 3. Disaster Recovery Scenarios

### 3.1 Failed Driver Update

| Step | Action |
|:-----|:-------|
| 1 | User observes device malfunction after update |
| 2 | Launch MAIDOS-Driver, navigate to device in scan list |
| 3 | Click "Rollback" to invoke `rollback_driver_c()` |
| 4a | If pre-update backup exists: driver restored from backup via pnputil |
| 4b | If no backup: user guided to System Restore via `rstrui.exe` |
| 5 | Verify device status returns to Normal (problem_code = 0) |

### 3.2 OS Reinstallation

| Step | Action |
|:-----|:-------|
| 1 | Before reinstall: run full backup to external drive |
| 2 | Reinstall Windows |
| 3 | Install MAIDOS-Driver |
| 4 | Navigate to Backup page, select external drive backup directory |
| 5 | Use pnputil to restore drivers: `pnputil /add-driver {dir}\*.inf /install /subdirs` |
| 6 | Run full scan to verify all devices are functioning |

### 3.3 Corrupted Driver Store

| Step | Action |
|:-----|:-------|
| 1 | Scan shows multiple devices with Problem Code 31 or 39 |
| 2 | Run System Restore to the most recent MAIDOS-created restore point |
| 3 | If restore point unavailable: restore drivers from last known backup |
| 4 | Run full scan and diagnostics to verify recovery |

## 4. Recovery Time Objectives

| Scenario | RTO | Method |
|:---------|:----|:-------|
| Single driver rollback | < 2 minutes | `rollback_driver_c()` from backup |
| Full driver restore from backup | < 10 minutes | Batch `pnputil /add-driver` |
| System Restore | < 15 minutes | Windows System Restore (OS-dependent) |

## 5. Retention Policy

| Item | Recommended Retention |
|:-----|:----------------------|
| Driver backups | Keep at least the last 2 backup sets |
| System restore points | Managed by Windows (disk space dependent) |
| Audit log | Retain indefinitely for compliance; archive monthly |

## 6. Best Practices

1. Run a full driver backup before any major OS update or hardware change.
2. Store backups on a separate physical drive or network share.
3. Verify backup integrity by checking the backup summary table after export.
4. Test restore on a non-production system before relying on backups for fleet deployment.
5. Review the audit log after any disaster recovery to confirm all operations succeeded.
