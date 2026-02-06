# MAIDOS-Driver -- Product Requirements Document

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved

---

## 1. Product Overview

MAIDOS-Driver is a Windows device driver lifecycle management tool that provides comprehensive
hardware detection, driver installation, updating, rollback, backup, and diagnostics through
a native desktop application. It bridges a C# WPF front-end with a Rust native engine via FFI,
delivering both safety and performance for low-level Windows API operations.

## 2. Core Value Proposition

- **One-click hardware inventory**: Enumerate every device with name, vendor, driver version, and status.
- **Full driver lifecycle**: Install, update, rollback, backup, and restore drivers from a single UI.
- **Proactive diagnostics**: Detect problem codes, missing drivers, and IRQ conflicts automatically.
- **Audit trail**: Every operation is logged for IT compliance and troubleshooting.
- **Safe by design**: Rust FFI layer eliminates memory-safety bugs; signature verification blocks tampered drivers.

## 3. Target Users

| Persona | Description | Primary Use Case |
|:--------|:------------|:-----------------|
| IT Administrator | Manages fleet of Windows workstations | Batch driver updates, backup before OS migration |
| Power User | Enthusiast who builds or upgrades PCs | Install latest GPU/audio drivers, diagnose hardware issues |
| System Integrator | Assembles custom PC configurations | Verify all drivers installed, backup golden image drivers |

## 4. Competitive Analysis

| Competitor | Strengths | MAIDOS-Driver Advantage |
|:-----------|:----------|:------------------------|
| Windows Device Manager | Built-in, zero install | No batch operations, no backup, no audit log |
| Driver Booster (IObit) | Large driver database | Closed source, bundleware, no FFI safety layer |
| Snappy Driver Installer | Offline driver packs | Complex UI, no rollback, no audit log |
| Intel DSA / NVIDIA GFE | Vendor-specific accuracy | Single-vendor only; MAIDOS covers all device classes |

## 5. Core Features

| ID | Feature | Description |
|:---|:--------|:------------|
| FR-001 | Hardware Scan | Enumerate all devices via SetupDI with name, vendor, version, status |
| FR-002 | Driver Install | Install from INF with automatic system restore point creation |
| FR-003 | Driver Update | Check TSV database and Windows Update, download via BITS, apply via pnputil |
| FR-004 | Driver Rollback | Revert to previous driver version or restore from backup |
| FR-005 | Driver Backup | Export all OEM drivers to a specified directory using pnputil |
| FR-006 | Diagnostics | Detect problem codes (CM_Get_DevNode_Status) and IRQ conflicts |
| FR-007 | Audit Logging | Record all operations with timestamp, device, operation, and result |

## 6. Architecture Summary

```
C# WPF UI (.NET 10.0)  -->  C# Service (P/Invoke, 16 DllImport)
    -->  Rust cdylib (maidOS_driver.dll, 16 exported functions)
    -->  Windows API (SetupDI, PnP, CM, WMI, BITS, pnputil)
```

## 7. Non-Goals

- Not a driver development tool (no WDK/WDF/KMDF interaction).
- Not a hardware benchmark tool.
- Not a remote management tool (local machine only).
- Not cross-platform (Windows 10/11 only).
- Not a driver signing tool.

## 8. Success Metrics

| Metric | Target |
|:-------|:-------|
| Device scan time | < 5 seconds for up to 500 devices |
| Memory footprint | < 50 MB resident |
| Driver install time | < 30 seconds per driver |
| Backup completion | < 60 seconds for all OEM drivers |
| Crash-free rate | 100% (all errors handled gracefully) |

## 9. Dependencies

| Dependency | Version | Purpose |
|:-----------|:--------|:--------|
| Rust toolchain | 1.70+ | Native engine (cdylib) |
| windows crate | 0.58.0 | SetupDI, CM, WMI bindings |
| .NET SDK | 10.0 | WPF UI and Service layer |
| MSVC | VS 2022+ | C/C++ link toolchain |
| WiX Toolset | v3.14 | MSI installer packaging |
