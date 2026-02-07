# Architecture Document - MAIDOS-Driver

## 1. System Overview

MAIDOS-Driver follows a layered architecture with a Rust native engine at the core, exposed to a C# WPF desktop application through a well-defined FFI boundary. This design isolates safety-critical driver operations in Rust while providing a rich Windows desktop user experience through WPF.

## 2. Layer Diagram

```
+--------------------------------------------------+
|              C# WPF Desktop UI                    |
|   (MainWindow, DeviceListView, UpdatePanel)       |
+--------------------------------------------------+
|            C# Service / ViewModel Layer           |
|   (DriverService, P/Invoke declarations)          |
+--------------------------------------------------+
|              FFI Boundary (extern "C")            |
|   (CString UTF-8, CoTaskMemUTF8 marshalling)     |
+--------------------------------------------------+
|           Rust cdylib: maidOS_driver.dll          |
|   +--------------------------------------------+ |
|   |  detect   |  update  | backup  | verify    | |
|   +--------------------------------------------+ |
|   |  wmi.rs   | bits.rs  | zip.rs  | trust.rs  | |
|   +--------------------------------------------+ |
|   |        Windows API Bindings (0.58)          | |
|   +--------------------------------------------+ |
+--------------------------------------------------+
|              Windows Kernel / Drivers             |
+--------------------------------------------------+
```

## 3. Component Details

### 3.1 Rust Engine (maidOS_driver.dll)

The Rust engine is compiled as a `cdylib` crate. It contains the following modules:

| Module | Responsibility | Windows API |
|--------|---------------|-------------|
| `wmi.rs` | Hardware enumeration via WMI COM | IWbemLocator, IWbemServices, IWbemClassObject |
| `setupdi.rs` | Device information via SetupDI | SetupDiGetClassDevs, SetupDiEnumDeviceInfo |
| `bits.rs` | Driver package downloads | BITS COM interfaces |
| `trust.rs` | Authenticode signature verification | WinVerifyTrust |
| `database.rs` | TSV driver database lookup | File I/O |
| `backup.rs` | Driver export/import to ZIP | zip crate + pnputil |
| `ffi.rs` | extern "C" function exports | CString marshalling |
| `pci_usb.rs` | VEN/DEV identifier parsing | N/A |

### 3.2 C# WPF Application

The C# layer provides:
- **MainWindow**: Primary application shell with navigation
- **DeviceListView**: DataGrid displaying detected hardware
- **UpdatePanel**: Driver update workflow with progress indicators
- **BackupView**: Backup creation and restore interface
- **DriverService**: P/Invoke wrapper calling into maidOS_driver.dll

### 3.3 FFI Boundary

All FFI functions use `extern "C"` calling convention. String parameters are passed as `*const c_char` (UTF-8 encoded). The C# side uses `PtrToStringUTF8` and `StringToCoTaskMemUTF8` for marshalling. Callers are responsible for freeing returned strings via `driver_free_string`.

## 4. Data Flow

### 4.1 Hardware Detection Flow
1. C# UI calls `detect_hardware()` via P/Invoke
2. Rust spawns an STA thread for WMI COM initialization
3. WMI queries `Win32_PnPEntity` for all PnP devices
4. SetupDI enriches device data with registry properties
5. Results serialized as JSON, returned as CString to C#
6. C# deserializes and populates the DeviceListView

### 4.2 Driver Update Flow
1. C# calls `check_all_updates(devices_json)` via P/Invoke
2. Rust parses device list, queries TSV database for VEN&DEV matches
3. Matches with download URLs are returned; unmatched fall back to Windows Update
4. C# calls `download_update(url, dest_path)` for each available update
5. Rust uses BITS to download with HTTPS enforcement and SHA-256 verification
6. C# calls `apply_update(inf_path)` which invokes pnputil

### 4.3 Backup/Restore Flow
1. C# calls `backup_drivers(output_path)` via P/Invoke
2. Rust enumerates third-party drivers via SetupDI
3. Driver files collected and compressed into a ZIP archive
4. Restore reverses: extract ZIP, verify contents, install via pnputil

## 5. Threading Model

- WMI operations execute on a dedicated STA thread to avoid MTA deadlock
- BITS downloads run asynchronously with progress callbacks
- The FFI boundary is synchronous; C# manages async dispatch via Task.Run
- UI thread is never blocked by engine operations

## 6. Security Model

- All driver packages verified via WinVerifyTrust before installation
- BITS downloads restricted to a trusted whitelist of HTTPS URLs
- SHA-256 checksums validated for every downloaded package
- Administrator privileges required and verified at startup
- No user data leaves the local machine

## 7. Database Architecture

The TSV driver database (`drivers.tsv`) maps hardware IDs to driver packages:

```
VEN_ID    DEV_ID    DRIVER_NAME    VERSION    DOWNLOAD_URL    SHA256
```

Database location priority:
1. `{exe_dir}/data/drivers.tsv` (portable mode)
2. `%ProgramData%\MAIDOS\DriverManager\drivers.tsv` (installed mode)

## 8. Build Artifacts

| Artifact | Path | Description |
|----------|------|-------------|
| maidOS_driver.dll | target/release/ | Rust cdylib engine |
| MAIDOS-Driver.exe | publish/ | C# WPF application |
| MAIDOS-Driver.msi | installer/out/ | WiX v3.14 installer |
