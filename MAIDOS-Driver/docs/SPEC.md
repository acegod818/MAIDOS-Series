# MAIDOS-Driver -- Technical Specification Summary

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved
**Full Specification**: `SPEC-MAIDOS-Driver-v2.0.md` (project root)

---

## 1. Purpose

This document provides a condensed reference to the full technical specification defined in
`SPEC-MAIDOS-Driver-v2.0.md`. That specification is the authoritative source for all
functional requirements, acceptance criteria, FFI interface definitions, and architecture
decisions. This summary exists for quick onboarding and cross-referencing.

## 2. Product Definition

MAIDOS-Driver is a Windows device driver lifecycle management tool. It enumerates hardware
devices, installs, updates, rolls back, backs up, and diagnoses drivers through a four-layer
architecture: C# WPF UI, C# Service (P/Invoke bridge), Rust cdylib DLL, and Windows API.

## 3. Functional Requirements Summary

| ID | Name | Key API |
|:---|:-----|:--------|
| FR-001 | Hardware Scan | `scan_all_devices_c()` via SetupDI |
| FR-002 | Driver Install | `install_driver_c()` via pnputil |
| FR-003 | Driver Update | `check_all_updates_c()`, `download_update_c()`, `apply_update_c()` |
| FR-004 | Driver Rollback | `rollback_driver_c()` via backup restore or system rollback |
| FR-005 | Driver Backup | `backup_drivers_c()` via pnputil /export-driver |
| FR-006 | Diagnostics | `diagnose_device_c()` via CM_Get_DevNode_Status + WMI IRQ |
| FR-007 | Audit Logging | `[MAIDOS-AUDIT]` structured log entries |

## 4. Acceptance Criteria Summary

- **20 acceptance criteria** (AC-001 through AC-020) mapped across 7 functional requirements.
- Full matrix available in `docs/AC_MATRIX.md`.
- Test types: Unit, Integration, Benchmark.

## 5. FFI Interface

- **16 exported functions** defined in `src/ffi.rs`.
- All functions use `extern "C"` calling convention with `#[no_mangle]`.
- String parameters: `*const c_char` (UTF-8 null-terminated).
- Return convention: count on success (>= 0), -1 on failure.
- Memory ownership: caller must invoke corresponding `free_*` function.
- Full contract in `docs/CONTRACT.md`.

## 6. Non-Functional Requirements

| NFR | Target |
|:----|:-------|
| NFR-001 | Scan < 5s for 500 devices |
| NFR-002 | Memory < 50 MB |
| NFR-003 | Admin required for install/update/rollback |
| NFR-004 | Digital signature verification via WinVerifyTrust |
| NFR-005 | All errors handled gracefully, no crashes |
| NFR-006 | 0 errors, 0 warnings on build |
| NFR-007 | Rust >= 60% coverage, C# >= 50% coverage |
| NFR-008 | Windows 10 21H2+ and Windows 11 |

## 7. Technology Stack

| Layer | Technology | Version |
|:------|:-----------|:--------|
| UI | C# WPF | .NET 10.0 |
| Service | C# P/Invoke | .NET 10.0 |
| Engine | Rust cdylib | 1.70+, windows crate 0.58.0 |
| Installer | WiX Toolset | v3.14 |
| OS | Windows | 10 21H2+ / 11 |

## 8. Cross-References

| Document | Location |
|:---------|:---------|
| Full Specification | `SPEC-MAIDOS-Driver-v2.0.md` |
| Architecture | `docs/ARCHITECTURE.md` |
| FFI Contract | `docs/CONTRACT.md` |
| Acceptance Matrix | `docs/AC_MATRIX.md` |
| ADR Records | `docs/ADR.md` |
| Deployment Guide | `docs/DEPLOY.md` |
