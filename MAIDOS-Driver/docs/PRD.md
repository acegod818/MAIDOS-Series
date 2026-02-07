# Product Requirements Document - MAIDOS-Driver

## 1. Overview

MAIDOS-Driver is a Windows driver management engine that provides automated hardware detection, driver matching, backup, restore, and update capabilities. It is distributed as part of the MAIDOS product series and targets Windows 10/11 desktop systems.

## 2. Problem Statement

Windows users frequently encounter outdated, missing, or incompatible device drivers. The built-in Windows Update mechanism does not always deliver the optimal driver for every device. Manual driver management requires technical expertise and carries risk of system instability. MAIDOS-Driver automates this workflow with a safe, verifiable pipeline.

## 3. Target Users

- Windows desktop users who need reliable driver management
- System administrators performing fleet-wide driver maintenance
- Technicians building or refurbishing PCs

## 4. Product Goals

| ID | Goal | Success Metric |
|----|------|----------------|
| PG-1 | Detect all PCI and USB hardware automatically | >= 95% detection rate on supported hardware |
| PG-2 | Match detected hardware to known driver packages | >= 90% match rate against TSV database |
| PG-3 | Provide safe driver backup and restore | Zero data loss during backup/restore cycle |
| PG-4 | Validate driver signatures before installation | 100% signature verification coverage |
| PG-5 | Deliver updates via BITS with integrity checks | SHA-256 verification on every download |

## 5. Core Features

### 5.1 Hardware Detection (M1)
- Enumerate PCI and USB devices via WMI COM queries
- Extract Vendor ID (VEN) and Device ID (DEV) for each device
- Resolve human-readable device names from the PCI/USB ID database
- Report device class, status, and current driver version

### 5.2 Driver Matching and Updates (M2)
- Match VEN&DEV pairs against the local TSV driver database
- Query Windows Update API as a fallback source
- Download driver packages via BITS with HTTPS enforcement
- Verify SHA-256 checksums before applying updates
- Install drivers using pnputil /add-driver /install

### 5.3 Backup and Restore (M3)
- Export installed third-party drivers to a compressed ZIP archive
- Restore drivers from a previously created backup archive
- Validate backup integrity before restore operations
- Support selective restore of individual driver packages

### 5.4 Signature Verification
- Verify Authenticode signatures using WinVerifyTrust
- Block unsigned or tampered driver packages
- Report signature status in the UI for user review

## 6. Architecture Summary

The engine is built as a Rust cdylib (`maidOS_driver.dll`) exposed to a C# WPF desktop application via P/Invoke FFI. The Rust layer handles all Windows API interactions (WMI, SetupDI, BITS, WinVerifyTrust) while the C# layer provides the user interface and orchestration.

## 7. Constraints

- Requires Windows 10 version 1809 or later
- Requires administrator privileges for driver installation
- Must operate without internet for local backup/restore operations
- Total memory footprint must remain under 200 MB
- Application startup must complete within 8 seconds

## 8. Release Information

- **Current Version**: v0.2.2
- **Installer**: MSI via WiX v3.14 (60 MB, self-contained .NET 10 + WPF)
- **Crate Name**: maidOS-driver (Rust identifier: maidOS_driver)
- **DLL Output**: target/release/maidOS_driver.dll
- **Signing Identity**: ZGWC_acegod818 <wocao@maidos.dev>

## 9. Dependencies

| Component | Version | Purpose |
|-----------|---------|---------|
| Rust (stable) | latest | Core engine compilation |
| .NET 10 | SDK | WPF UI host |
| WiX Toolset | v3.14 | MSI installer generation |
| windows crate | 0.58 | Win32 API bindings |

## 10. Out of Scope

- Linux or macOS driver management
- Network/fleet driver deployment (future consideration)
- Driver development or signing services
- UEFI firmware updates
