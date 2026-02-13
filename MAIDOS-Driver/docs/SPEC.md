# SPEC — MAIDOS-Driver v0.2.2

## Overview

This specification document maps all product features to their corresponding test implementations, defines interface contracts, non-functional requirements, and failure modes for MAIDOS-Driver.

**Product**: Windows Driver Management Engine
**Architecture**: Rust cdylib (maidOS_driver.dll) + C# WPF UI via P/Invoke FFI
**Version**: 0.2.2
**Test Coverage**: 30 tests (29 + 1), 0 ignored

---

## Feature → Test Mapping

### Module M1: Hardware Detection

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-001 | WMI device enumeration | `core::detect::hardware` | `tests/hardware_detection_test.rs` | `test_hardware_detection_basic` |
| F-002 | SetupDI device properties | `core::detect::hardware` | `src/core/detect/hardware.rs` | `test_device_property_extraction` |
| F-003 | PCI VEN/DEV parsing | `platform::windows::wmi_queries` | `src/platform/windows/wmi_queries.rs` | `test_pci_id_parsing` |
| F-004 | USB VID/PID parsing | `platform::windows::wmi_queries` | `src/platform/windows/wmi_queries.rs` | `test_usb_id_parsing` |
| F-005 | Device status detection | `core::detect::hardware` | `src/core/detect/hardware.rs` | `test_device_status_flags` |

### Module M2: Driver Matching and Updates

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-006 | TSV database loading | `database::driver_database` | `src/database/driver_database.rs` | `test_load_database` |
| F-007 | TSV database querying | `database::driver_database` | `src/database/driver_database.rs` | `test_query_by_device_id` |
| F-008 | Driver version comparison | `core::update::checker` | `src/core/update/checker.rs` | `test_version_comparison` |
| F-009 | VEN&DEV extraction | `core::update::checker` | `src/core/update/checker.rs` | `test_ven_dev_extraction` |
| F-010 | Update check (single device) | `core::update::checker` | `src/core/update/checker.rs` | `test_check_driver_update` |
| F-011 | Update check (all devices) | `core::update::checker` | `src/core/update/checker.rs` | `test_check_all_updates` |
| F-012 | Windows Update fallback | `core::update::checker` | `src/core/update/checker.rs` | `test_windows_update_fallback` |
| F-013 | BITS download with HTTPS | `core::download::downloader` | `src/core/download/downloader.rs` | `test_https_enforcement` |
| F-014 | SHA-256 verification | `core::download::downloader` | `src/core/download/downloader.rs` | `test_checksum_verification` |
| F-015 | Driver installation (pnputil) | `core::install::installer` | `src/core/install/installer.rs` | `test_driver_installation` |

### Module M3: Backup and Restore

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-016 | Driver backup to ZIP | `core::backup::manager` | `src/core/backup/manager.rs` | `test_backup_drivers` |
| F-017 | Backup entry serialization | `core::backup::manager` | `src/core/backup/manager.rs` | `test_backup_entry_serde` |
| F-018 | Driver restoration | `core::restore::manager` | `src/core/restore/manager.rs` | `test_restore_drivers` |
| F-019 | Selective restore | `core::restore::manager` | `src/core/restore/manager.rs` | `test_selective_restore` |

### Module M4: Signature Verification

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-020 | Authenticode verification | `core::verify::signature_verifier` | `src/core/verify/signature_verifier.rs` | `test_verify_valid_signature` |
| F-021 | Unsigned driver detection | `core::verify::signature_verifier` | `src/core/verify/signature_verifier.rs` | `test_verify_unsigned_driver` |
| F-022 | Tampered file detection | `core::verify::signature_verifier` | `src/core/verify/signature_verifier.rs` | `test_verify_tampered_file` |

### Module M5: Device Diagnostics

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-023 | Device problem code detection | `core::diagnose::device_diagnostics` | `src/core/diagnose/device_diagnostics.rs` | `test_diagnose_problem_code` |
| F-024 | IRQ conflict detection | `core::diagnose::device_diagnostics` | `src/core/diagnose/device_diagnostics.rs` | `test_diagnose_irq_conflict` |
| F-025 | Driver rollback | `core::install::rollback_handler` | `src/core/install/rollback_handler.rs` | `test_rollback_driver` |

### Module M6: AI Driver Recommendation

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-026 | Hardware identification | `ai::hardware_identifier` | `src/ai/hardware_identifier.rs` | `test_hardware_identification` |
| F-027 | Driver recommendation | `ai::driver_recommender` | `src/ai/driver_recommender.rs` | `test_driver_recommendation` |
| F-028 | Unknown device handling | `ai::hardware_identifier` | `src/ai/hardware_identifier.rs` | `test_unknown_device_fallback` |

### Module M7: Audit and Compliance

| Feature ID | Feature | Module | Test File | Test Function |
|-----------|---------|--------|-----------|---------------|
| F-029 | Audit log generation | `core::audit` | `src/core/audit.rs` | `test_audit_success` |
| F-030 | Audit log failure tracking | `core::audit` | `src/core/audit.rs` | `test_audit_failure` |

---

## Interface Contracts

### FFI API (P/Invoke C ABI)

**Hardware Detection**
```c
// Scan all devices, returns device count or -1 on error
int32_t scan_all_devices_c(CDeviceInfo** devices_ptr);

// Free device info array
void free_device_info(CDeviceInfo* devices, int32_t count);
```

**Driver Updates**
```c
// Check single device update
int32_t check_driver_update_c(const char* device_id, const char* update_server, CUpdateInfo* info_ptr);

// Check all device updates
int32_t check_all_updates_c(CUpdateInfo** updates_ptr);

// Download driver update
int64_t download_update_c(const char* download_url, const char* save_path);

// Apply driver update
int32_t apply_update_c(const char* inf_path, const char* device_id);

// Free update info structures
void free_update_info(CUpdateInfo* info);
void free_update_info_array(CUpdateInfo* updates, int32_t count);
```

**Backup & Restore**
```c
// Backup drivers to ZIP
int32_t backup_drivers_c(const char* backup_path, CBackupEntry** entries_ptr);

// Free backup entries
void free_backup_entries(CBackupEntry* entries, int32_t count);
```

**Diagnostics & Rollback**
```c
// Diagnose device issues
int32_t diagnose_device_c(const char* device_id, CDiagnosticInfo* info_ptr);

// Rollback driver
int32_t rollback_driver_c(const char* device_id, const char* backup_path);

// Free diagnostic info
void free_diagnostic_info(CDiagnosticInfo* info);
```

**Error Handling**
```c
// Get last error message
char* get_last_error();

// Free error string
void free_string(char* s);
```

### FFI Data Structures

**CDeviceInfo** (C# ↔ Rust)
- `id: *mut c_char` - Device hardware ID
- `name: *mut c_char` - Friendly name
- `vendor: *mut c_char` - Manufacturer
- `version: *mut c_char` - Driver version
- `status: *mut c_char` - Device status

**CUpdateInfo** (C# ↔ Rust)
- `device_id: *mut c_char` - Device identifier
- `current_version: *mut c_char` - Installed version
- `latest_version: *mut c_char` - Available version
- `update_available: i32` - Boolean flag (0/1)
- `status: *mut c_char` - Update status
- `download_url: *mut c_char` - Download URL from TSV

**CDiagnosticInfo** (C# ↔ Rust)
- `device_id: *mut c_char` - Device identifier
- `problem_code: i32` - Windows CM_PROB code
- `problem_description: *mut c_char` - Human-readable description
- `irq: i32` - IRQ number
- `status: *mut c_char` - Diagnostic status

### State Transitions

**Device Status States**
- `OK` - Device functioning normally
- `PROBLEM` - Device has issue (see problem_code)
- `DISABLED` - Device manually disabled
- `NOT_PRESENT` - Device not connected
- `UNKNOWN` - Status cannot be determined

**Update Status States**
- `NO_UPDATE` - No update available
- `UPDATE_AVAILABLE` - TSV database match found
- `WINDOWS_UPDATE` - Fallback to Windows Update
- `DOWNLOADING` - BITS download in progress
- `DOWNLOADED` - Package downloaded, SHA-256 verified
- `INSTALLING` - pnputil executing
- `INSTALLED` - Update applied successfully
- `FAILED` - Update failed (see last_error)

---

## NFR Mapping

### Performance SLO

| NFR | SLI | SLO Target | Test |
|-----|-----|------------|------|
| NFR-001 | Application startup time | ≤ 8 seconds | AC-020 |
| NFR-002 | Hardware scan time | ≤ 3 seconds for typical system (30 devices) | `test_hardware_detection_basic` |
| NFR-003 | Database query latency | ≤ 100ms per device | `test_query_by_device_id` |
| NFR-004 | Memory footprint | ≤ 200 MB (stable state) | Manual validation |

### Reliability SLO

| NFR | SLI | SLO Target | Test |
|-----|-----|------------|------|
| NFR-005 | Driver signature verification coverage | 100% | AC-011 |
| NFR-006 | SHA-256 download verification | 100% | AC-008 |
| NFR-007 | Backup/restore fidelity | Zero data loss | AC-012, AC-013 |
| NFR-008 | FFI string encoding fidelity | 100% UTF-8 correctness | AC-014 |

### Security SLO

| NFR | SLI | SLO Target | Test |
|-----|-----|------------|------|
| NFR-009 | HTTPS enforcement rate | 100% (reject HTTP) | AC-017 |
| NFR-010 | Admin privilege validation | 100% for install operations | AC-016 |
| NFR-011 | Unsigned driver rejection rate | 100% (when policy enabled) | `test_verify_unsigned_driver` |
| NFR-012 | Memory leak prevention | Zero leaks in FFI boundary | AC-015 |

### Compatibility SLO

| NFR | SLI | SLO Target | Test |
|-----|-----|------------|------|
| NFR-013 | Device detection coverage | ≥ 95% on supported hardware | PG-1 (PRD) |
| NFR-014 | Driver database match rate | ≥ 90% | PG-2 (PRD) |
| NFR-015 | Windows version support | Windows 10 1809+ and Windows 11 | Manual validation |
| NFR-016 | Offline operation capability | Backup/restore without internet | AC-019 |

---

## Failure Modes

### Critical Failures

| Mode | Impact | Mitigation | Test |
|------|--------|------------|------|
| FM-001: WMI COM initialization failure | Cannot enumerate devices | Retry with STA thread, log error, graceful UI message | AC-018 |
| FM-002: SetupDI API failure | Incomplete device list | Log warning, return partial results | `test_device_property_extraction` |
| FM-003: TSV database corruption | Cannot match drivers | Fallback to Windows Update API | AC-010 |
| FM-004: BITS download failure | Update unavailable | Retry with exponential backoff, log error | AC-019 |
| FM-005: SHA-256 mismatch | Corrupted download | Reject file, delete partial download, log failure | AC-008 |
| FM-006: pnputil execution failure | Driver not installed | Rollback operation, restore previous state | `test_rollback_driver` |
| FM-007: Backup ZIP creation failure | Cannot create backup | Check disk space, permissions, log error | `test_backup_drivers` |
| FM-008: Memory exhaustion | Crash or OOM | Limit device list size, stream ZIP contents | AC-015 |

### Non-Critical Failures

| Mode | Impact | Mitigation | Test |
|------|--------|------------|------|
| FM-009: Unknown device | No vendor/device name | Display raw VEN/DEV ID, enable AI fallback | `test_unknown_device_fallback` |
| FM-010: Version parsing failure | Cannot compare versions | Treat as "unknown version", skip update | `test_version_comparison` |
| FM-011: FFI string encoding error | Garbled UI text | Use UTF-8 with replacement chars, log warning | AC-014 |
| FM-012: Unsigned driver detected | Security warning | Display signature status, allow user override with warning | AC-011 |
| FM-013: Windows Update API timeout | Slow update check | Display progress indicator, allow cancellation | AC-019 |
| FM-014: Audit log write failure | Compliance gap | Queue logs in memory, retry write, alert admin | `test_audit_failure` |

### Edge Cases

| Mode | Impact | Mitigation | Test |
|------|--------|------------|------|
| FM-015: Empty device list | Confusing UI | Show "No devices detected" message, check admin rights | `test_hardware_detection_basic` |
| FM-016: Duplicate VEN/DEV entries in TSV | Ambiguous match | Select first match, log warning | `test_query_by_device_id` |
| FM-017: Download URL is HTTP (not HTTPS) | Security violation | Reject download, return error code 1 | AC-017 |
| FM-018: Device removed during update | Installation failure | Detect device absence, abort gracefully | Manual validation |
| FM-019: Insufficient admin privileges | Cannot install drivers | Detect privilege level, prompt UAC elevation | AC-016 |
| FM-020: Concurrent FFI calls | Race condition | Use `Mutex<String>` for last_error, thread-safe WMI | AC-018 |

---

## Acceptance Criteria Cross-Reference

All 20 acceptance criteria (AC-001 through AC-020) defined in [AC_MATRIX.md](./AC_MATRIX.md) are covered by the features and tests listed above. Key mappings:

- **AC-001 to AC-005**: Hardware detection (F-001 to F-005)
- **AC-006 to AC-010**: Driver updates (F-006 to F-012)
- **AC-011**: Signature verification (F-020 to F-022)
- **AC-012 to AC-013**: Backup and restore (F-016 to F-019)
- **AC-014 to AC-015**: FFI contracts (interface tests)
- **AC-016 to AC-020**: Non-functional requirements (NFR-001 to NFR-016)

---

## Test Execution Summary

**Total Tests**: 30 (29 unit tests + 1 integration test)
**Passing**: 30
**Failing**: 0
**Ignored**: 0
**Coverage**: All modules covered (M1-M7)

**Test Command**:
```bash
cargo test --release
```

**Evidence Location**:
- Test reports: `evidence/test_reports/`
- Proof manifest: `proof/manifest.json`
- Gate reports: `qc/gate{1-4}_report.log`

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-02-13 | ZGWC_acegod818 | Initial SPEC document for v0.2.2 |

---

**Signature**: ZGWC_acegod818 <wocao@maidos.dev>
**Compliance**: CodeQC v3.0 C gate-checked, 0 errors, 0 warnings
