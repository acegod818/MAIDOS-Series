# MAIDOS-Driver -- FFI Contract

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved

---

## 1. Overview

The Rust engine (`maidOS_driver.dll`) exports 16 functions with `extern "C"` calling convention.
The C# Service layer calls these via `[DllImport("maidOS_driver.dll")]`. All functions are
defined in `src/ffi.rs` with `#[no_mangle]`.

## 2. General Conventions

- **Strings**: `*const c_char` (null-terminated UTF-8). C# must use `PtrToStringUTF8` / `StringToCoTaskMemUTF8`.
- **Return values**: Non-negative integer on success (typically element count), -1 on failure.
- **Error details**: Call `get_last_error()` after any function returns -1.
- **Memory ownership**: Rust allocates, caller must free via the corresponding `free_*` function.
- **Thread safety**: Functions are not thread-safe. Call from a single thread or synchronize externally.

## 3. Exported Functions

### 3.1 Device Scanning

| # | Function | Signature | Description |
|:--|:---------|:----------|:------------|
| 1 | `scan_all_devices_c` | `(out_ptr: *mut *mut CDeviceInfo) -> i32` | Enumerates all devices. Returns device count. |
| 2 | `free_device_info` | `(ptr: *mut CDeviceInfo, count: i32)` | Frees the CDeviceInfo array allocated by scan. |

### 3.2 Error Handling

| # | Function | Signature | Description |
|:--|:---------|:----------|:------------|
| 3 | `get_last_error` | `() -> *mut c_char` | Returns the last error message as a UTF-8 string. |
| 4 | `free_string` | `(ptr: *mut c_char)` | Frees a string allocated by Rust. |

### 3.3 Driver Installation

| # | Function | Signature | Description |
|:--|:---------|:----------|:------------|
| 5 | `install_driver_c` | `(inf_path: *const c_char) -> i32` | Installs driver from INF. Returns 0 on success. |

### 3.4 Driver Backup

| # | Function | Signature | Description |
|:--|:---------|:----------|:------------|
| 6 | `backup_drivers_c` | `(dir: *const c_char, out_ptr: *mut *mut CBackupEntry) -> i32` | Exports all OEM drivers. Returns entry count. |
| 7 | `free_backup_entries` | `(ptr: *mut CBackupEntry, count: i32)` | Frees the CBackupEntry array. |

### 3.5 Driver Update

| # | Function | Signature | Description |
|:--|:---------|:----------|:------------|
| 8 | `check_driver_update_c` | `(hw_id: *const c_char, server: *const c_char, out: *mut CUpdateInfo) -> i32` | Checks update for one device. Returns 0 on success. |
| 9 | `free_update_info` | `(ptr: *mut CUpdateInfo)` | Frees a single CUpdateInfo. |
| 10 | `download_update_c` | `(url: *const c_char, dest: *const c_char) -> i64` | Downloads driver package. Returns byte count or -1. |
| 11 | `apply_update_c` | `(inf_path: *const c_char, hw_id: *const c_char) -> i32` | Applies downloaded update. Returns 0 on success. |
| 12 | `check_all_updates_c` | `(out_ptr: *mut *mut CUpdateInfo) -> i32` | Batch checks all devices. Returns updatable count. |
| 13 | `free_update_info_array` | `(ptr: *mut CUpdateInfo, count: i32)` | Frees the CUpdateInfo array. |

### 3.6 Diagnostics and Rollback

| # | Function | Signature | Description |
|:--|:---------|:----------|:------------|
| 14 | `diagnose_device_c` | `(dev_id: *const c_char, out: *mut CDiagnosticInfo) -> i32` | Diagnoses device. Returns 0 on success. |
| 15 | `free_diagnostic_info` | `(ptr: *mut CDiagnosticInfo)` | Frees diagnostic info strings. |
| 16 | `rollback_driver_c` | `(dev_id: *const c_char, backup_dir: *const c_char) -> i32` | Rolls back driver. Returns 0 on success. |

## 4. C Struct Definitions

```
CDeviceInfo { name: *mut c_char, vendor: *mut c_char, driver_version: *mut c_char,
              hardware_id: *mut c_char, status: *mut c_char }

CBackupEntry { driver_name: *mut c_char, inf_path: *mut c_char, size_bytes: u64 }

CUpdateInfo  { device_name: *mut c_char, hardware_id: *mut c_char,
               current_version: *mut c_char, latest_version: *mut c_char,
               download_url: *mut c_char, update_available: i32 }

CDiagnosticInfo { device_id: *mut c_char, status: *mut c_char,
                  problem_code: i32, description: *mut c_char, irq: i32 }
```

## 5. Memory Lifecycle Rules

1. Every `scan_*` / `backup_*` / `check_*` call that outputs a pointer **must** be paired with its `free_*` call.
2. `get_last_error()` returns a freshly allocated string; caller **must** call `free_string()`.
3. Passing a null pointer to any function returns -1 with an appropriate error message.
4. Double-free is undefined behavior. Callers must null their pointer after freeing.
