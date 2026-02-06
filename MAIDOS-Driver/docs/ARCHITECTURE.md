# MAIDOS-Driver -- Architecture Document

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved

---

## 1. Four-Layer Architecture

```
+--------------------------------------------------------------+
|                Layer 1: UI (C# WPF, .NET 10.0)               |
|  MainWindow.xaml.cs                                          |
|  Pages: Scan | Install | Update | Backup | Diagnose | Audit  |
+-------------------------------+------------------------------+
                                |
                          P/Invoke (16 DllImport)
                                |
+-------------------------------v------------------------------+
|           Layer 2: Service (C# HardwareDetectionService)     |
|  - Marshals managed types to/from C structs                  |
|  - UTF-8 string conversion (PtrToStringUTF8)                |
|  - Memory lifecycle management (AllocHGlobal/FreeCoTaskMem)  |
+-------------------------------+------------------------------+
                                |
                          extern "C" FFI
                                |
+-------------------------------v------------------------------+
|           Layer 3: Rust Engine (maidOS_driver.dll)           |
|  src/ffi.rs          - 16 #[no_mangle] extern "C" exports   |
|  src/core/detect/    - SetupDI hardware enumeration          |
|  src/core/install/   - pnputil driver installation           |
|  src/core/update/    - TSV DB + WU update checking           |
|  src/core/backup/    - pnputil driver export                 |
|  src/core/restore/   - pnputil driver restore                |
|  src/core/diagnose/  - CM_Get_DevNode_Status diagnostics     |
|  src/core/download/  - BITS/HTTPS driver download            |
|  src/core/verify/    - WinVerifyTrust signature check        |
|  src/core/audit.rs   - [MAIDOS-AUDIT] logging                |
|  src/database/       - TSV driver database                   |
|  src/platform/       - WMI, Registry, Service helpers        |
+-------------------------------+------------------------------+
                                |
                          Windows API calls
                                |
+-------------------------------v------------------------------+
|           Layer 4: Windows API                               |
|  SetupDiGetClassDevsW / SetupDiEnumDeviceInfo               |
|  CM_Locate_DevNodeA / CM_Get_DevNode_Status                 |
|  RegQueryValueExW (driver version from registry)            |
|  pnputil.exe (/add-driver, /export-driver, /remove-device)  |
|  BITS (Background Intelligent Transfer Service)              |
|  WinVerifyTrust (driver signature verification)              |
|  WMI (Win32_PnPAllocatedResource for IRQ)                   |
|  Checkpoint-Computer (system restore points)                 |
+--------------------------------------------------------------+
```

## 2. Data Flow: Scan to Update Cycle

```
User clicks "Check Updates"
  --> C# CheckAllUpdates() --> P/Invoke --> check_all_updates_c()
  --> Rust: scan_all_devices() via SetupDI enumeration
  --> Rust: load_driver_database() from TSV (VEN&DEV matching)
      |
      +--> Match found in DB --> return UpdateInfo with download_url
      +--> No DB match --> check_windows_update_available() via COM API
  --> User clicks "Download & Install"
  --> C# DownloadUpdate() --> download_update_c() --> BITS transfer
  --> C# ApplyUpdate() --> apply_update_c() --> pnputil /add-driver /install
  --> audit.log: [MAIDOS-AUDIT] timestamp APPLY_UPDATE device_id SUCCESS
```

## 3. Module Overview

| Module | Purpose |
|:-------|:--------|
| `core::detect::hardware` | SetupDI device enumeration and status query |
| `core::detect::unknown_devices` | Unknown device identification |
| `core::install::installer` | pnputil driver installation + restore point |
| `core::install::rollback_handler` | Driver rollback via backup or system restore |
| `core::update::checker` | TSV DB + Windows Update query, download, apply |
| `core::backup::manager` | pnputil /export-driver batch backup |
| `core::restore::manager` | pnputil /add-driver batch restore |
| `core::download::downloader` | BITS download with trusted source whitelist |
| `core::verify::signature_verifier` | WinVerifyTrust digital signature check |
| `core::diagnose::device_diagnostics` | CM_Get_DevNode_Status + IRQ query |
| `core::audit` | `[MAIDOS-AUDIT]` structured logging |
| `database::driver_database` | TSV driver database (VEN&DEV matching) |
| `platform::windows::wmi_queries` | WMI COM queries (STA thread) |
| `platform::windows::registry_manager` | Registry read/write helpers |

## 4. Key Design Decisions

- **Rust for native layer**: Memory safety for FFI, direct Windows API access via `windows` crate.
- **P/Invoke over COM Interop**: Simpler, more performant for cdylib functions.
- **UTF-8 throughout**: Rust CString (UTF-8) matched by C# `PtrToStringUTF8`.
- **pnputil over SetupAPI for install**: More reliable, handles driver store correctly.
- **TSV database over SQLite**: Simple, offline-capable, easy to update.
- **WMI via STA thread**: Avoids MTA deadlock issues with COM apartment model.
