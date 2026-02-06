# MAIDOS-Driver -- Non-Functional Requirements

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved

---

## NFR-001: Scan Performance

| Attribute | Value |
|:----------|:------|
| Requirement | Hardware scan completes in < 5 seconds for up to 500 devices |
| Measurement | Wall-clock time from `scan_all_devices_c()` call to return |
| Rationale | Users expect near-instant feedback when scanning hardware |
| Test | Benchmark test on a system with 100+ devices |
| AC Reference | AC-003 |

## NFR-002: Memory Usage

| Attribute | Value |
|:----------|:------|
| Requirement | Resident memory usage < 50 MB during normal operation |
| Measurement | Peak working set size measured via Task Manager or `tasklist` |
| Rationale | The tool should not compete for resources with other applications |
| Test | Monitor memory during full scan + update check cycle |

## NFR-003: Administrative Privileges

| Attribute | Value |
|:----------|:------|
| Requirement | Driver install, update, and rollback operations require administrator privileges |
| Measurement | UAC prompt appears; non-admin invocations return a clear error |
| Rationale | Windows driver installation is a privileged operation |
| Test | Run install_driver_c without elevation, verify error code |
| Note | Scan and diagnose operations do NOT require admin |

## NFR-004: Driver Signature Verification

| Attribute | Value |
|:----------|:------|
| Requirement | All downloaded drivers must pass digital signature verification |
| Measurement | WinVerifyTrust check before driver installation |
| Rationale | Prevent installation of tampered or unsigned drivers |
| Module | `src/core/verify/signature_verifier.rs` |
| API | `WinVerifyTrust` via `windows` crate |

## NFR-005: Crash-Free Error Handling

| Attribute | Value |
|:----------|:------|
| Requirement | No operation causes a crash; all errors return codes with human-readable messages |
| Measurement | All FFI functions return -1 on failure; `get_last_error()` provides details |
| Rationale | A driver management tool must never leave the system in an inconsistent state |
| Test | Error injection: null pointers, invalid paths, missing devices |
| AC Reference | AC-006, AC-012, AC-015, AC-020 |

## NFR-006: Clean Build

| Attribute | Value |
|:----------|:------|
| Requirement | `cargo build --release` and `dotnet build` produce 0 errors and 0 warnings |
| Measurement | `cargo clippy -- -D warnings` passes; `dotnet build -warnaserror` passes |
| Rationale | Warnings often indicate latent bugs, especially in unsafe FFI code |
| Test | CI build gate |

## NFR-007: Test Coverage

| Attribute | Value |
|:----------|:------|
| Requirement | Rust code coverage >= 60%; C# code coverage >= 50% |
| Measurement | `cargo tarpaulin` for Rust; `dotnet test --collect:"XPlat Code Coverage"` for C# |
| Rationale | Critical paths (FFI, device enumeration, error handling) must be tested |
| Current | 29+ Rust tests passing across all modules |

## NFR-008: Platform Compatibility

| Attribute | Value |
|:----------|:------|
| Requirement | Runs on Windows 10 21H2 (build 19044) and later, including Windows 11 |
| Measurement | Successful scan + install on both Windows 10 and Windows 11 test machines |
| Rationale | Cover the majority of enterprise and consumer Windows installations |
| Note | Specific pnputil flags vary by OS version; the tool auto-detects |
