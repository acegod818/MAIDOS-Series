# API Contract - MAIDOS-Driver

## 1. Overview

This document defines the FFI contract between the Rust cdylib engine (maidOS_driver.dll) and the C# WPF application. All functions use the extern "C" calling convention and UTF-8 string encoding.

## 2. String Marshalling Convention

| Direction | Rust Side | C# Side |
|-----------|-----------|---------|
| Rust to C# | CString::into_raw() | Marshal.PtrToStringUTF8(ptr) |
| C# to Rust | CStr::from_ptr(ptr) | Marshal.StringToCoTaskMemUTF8(str) |
| Free returned strings | driver_free_string(ptr) | Called after reading |

All returned string pointers must be freed by the caller using driver_free_string to prevent memory leaks.

## 3. FFI Function Signatures

### 3.1 Hardware Detection

```rust
/// Detect all PCI and USB hardware devices.
/// Returns JSON array of DeviceInfo structs as a C string.
/// Caller must free the returned pointer with driver_free_string.
#[no_mangle]
pub extern "C" fn detect_hardware() -> *mut c_char
```

Return JSON schema:
```json
[
  {
    "device_id": "PCI\VEN_8086&DEV_1901",
    "vendor_id": "8086",
    "device_name": "Intel Xeon E3-1200 v5",
    "driver_version": "10.1.1.44",
    "device_class": "System",
    "status": "OK"
  }
]
```

### 3.2 Driver Update Check

```rust
/// Check for available driver updates for the given devices.
#[no_mangle]
pub extern "C" fn check_all_updates(devices_json: *const c_char) -> *mut c_char
```

### 3.3 Driver Download

```rust
/// Download a driver update package via BITS.
/// Returns 0 on success, non-zero error code on failure.
#[no_mangle]
pub extern "C" fn download_update(url: *const c_char, dest_path: *const c_char) -> i32
```

Error codes:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Invalid URL or not HTTPS |
| 2 | BITS transfer failed |
| 3 | SHA-256 checksum mismatch |
| 4 | Destination path not writable |

### 3.4 Driver Installation

```rust
/// Apply a downloaded driver update using pnputil.
#[no_mangle]
pub extern "C" fn apply_update(inf_path: *const c_char) -> i32
```

### 3.5 Backup Drivers

```rust
/// Backup all third-party drivers to a ZIP archive.
#[no_mangle]
pub extern "C" fn backup_drivers(output_path: *const c_char) -> i32
```

### 3.6 Restore Drivers

```rust
/// Restore drivers from a previously created backup archive.
#[no_mangle]
pub extern "C" fn restore_drivers(backup_path: *const c_char) -> i32
```

### 3.7 Signature Verification

```rust
/// Verify the Authenticode signature of a driver file.
#[no_mangle]
pub extern "C" fn verify_driver_signature(file_path: *const c_char) -> i32
```

### 3.8 Memory Management

```rust
/// Free a string previously returned by any driver function.
#[no_mangle]
pub extern "C" fn driver_free_string(ptr: *mut c_char)
```

### 3.9 Version Query

```rust
/// Return the engine version string.
#[no_mangle]
pub extern "C" fn driver_version() -> *mut c_char
```

## 4. C# P/Invoke Declarations

```csharp
internal static class DriverNative
{
    private const string DLL = "maidOS_driver.dll";

    [DllImport(DLL, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr detect_hardware();

    [DllImport(DLL, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr check_all_updates(IntPtr devices_json);

    [DllImport(DLL, CallingConvention = CallingConvention.Cdecl)]
    public static extern int download_update(IntPtr url, IntPtr dest_path);

    [DllImport(DLL, CallingConvention = CallingConvention.Cdecl)]
    public static extern int apply_update(IntPtr inf_path);

    [DllImport(DLL, CallingConvention = CallingConvention.Cdecl)]
    public static extern int backup_drivers(IntPtr output_path);

    [DllImport(DLL, CallingConvention = CallingConvention.Cdecl)]
    public static extern int restore_drivers(IntPtr backup_path);

    [DllImport(DLL, CallingConvention = CallingConvention.Cdecl)]
    public static extern int verify_driver_signature(IntPtr file_path);

    [DllImport(DLL, CallingConvention = CallingConvention.Cdecl)]
    public static extern void driver_free_string(IntPtr ptr);

    [DllImport(DLL, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr driver_version();
}
```

## 5. Thread Safety

All FFI functions are safe to call from any thread. The Rust engine internally manages STA thread affinity for WMI COM operations.

## 6. Error Handling

Functions returning i32 use 0 for success and positive integers for error codes. Functions returning *mut c_char return a null pointer on catastrophic failure. The C# layer must check for null before calling PtrToStringUTF8.
