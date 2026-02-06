# MAIDOS-Driver

Windows hardware driver management tool with real-time device enumeration, driver update, backup, rollback, and diagnostics.

> **Spec**: See [`SPEC-MAIDOS-Driver-v2.0.md`](SPEC-MAIDOS-Driver-v2.0.md) for the full AC-001~AC-020 specification.

## Architecture

```
C# WPF UI  -->  C# Service (P/Invoke)  -->  Rust cdylib  -->  Windows API
  (App)         (HardwareDetectionService)   (maidOS_driver.dll)  (SetupDI / PnP / CM)
```

- **Rust core** (`src/core/`, `src/platform/`): native Windows API calls via the `windows` crate 0.58
- **FFI layer** (`src/ffi.rs`): 16 `extern "C"` functions exported from the DLL
- **C# service** (`src/MAIDOS.Driver.Service/`): managed wrapper with P/Invoke declarations
- **C# UI** (`src/MAIDOS.Driver.App/`): WPF desktop application

## Features

| Feature | Backend | Status |
|---------|---------|--------|
| Hardware scan | `SetupDiGetClassDevsW` + `SetupDiEnumDeviceInfo` | Production |
| Driver version | `SetupDiOpenDevRegKey` + `RegQueryValueExW` | Production |
| Device status | `CM_Get_DevNode_Status` | Production |
| Driver install | `pnputil /add-driver` | Production |
| Driver backup | `pnputil /export-driver` | Production |
| Driver rollback | `pnputil /add-driver` (from backup) | Production |
| Device diagnostics | `CM_Locate_DevNodeA` + `CM_Get_DevNode_Status` | Production |
| Update check | DriverDatabase TSV + Windows Update API | Production |
| Driver download | BITS transfer, HTTPS-only, trusted whitelist | Production |
| Driver apply | `pnputil /add-driver /install` | Production |
| Audit logging | File-based `[MAIDOS-AUDIT]` trail | Production |
| System restore | `Checkpoint-Computer` before install | Production |
| Disk space check | 500 MB minimum before backup | Production |
| IRQ detection | WMI `Win32_PnPAllocatedResource` | Production |
| Windows service mgmt | `OpenSCManagerW` / `CreateServiceW` | Production |
| Registry operations | `RegOpenKeyExW` / `RegSetValueExW` | Production |

## Getting Started

### Prerequisites

- **Rust** 1.70+ (stable MSVC toolchain)
- **.NET 10 SDK** (for C# WPF projects)
- **Windows 10/11** (target platform)
- **Administrator privileges** (required for hardware enumeration)

### Build

```bash
# Build the native Rust DLL
cargo build --release
# Output: target/release/maidOS_driver.dll

# Run tests (30 tests)
cargo test

# Lint
cargo clippy

# Build the C# WPF application
dotnet publish src/MAIDOS.Driver.App/MAIDOS.Driver.App.csproj \
  -c Release -r win-x64 --self-contained -o publish

# Copy the DLL and driver database into publish/
cp target/release/maidOS_driver.dll publish/
cp -r data publish/data
```

### Usage (GUI)

1. **Run as Administrator**: `publish/MAIDOS.Driver.App.exe`
2. **Scan**: Click "Scan Hardware" to enumerate all devices via SetupDI
3. **Update**: Click "Check Updates" to query the driver database + Windows Update
   - Devices with a DB match will auto-download via BITS and install via `pnputil`
   - Devices with Windows Update only will open `ms-settings:windowsupdate`
4. **Backup**: Click "Backup Drivers" to export OEM drivers via `pnputil /export-driver`
5. **Rollback**: Select a device and click "Rollback" to restore from backup
6. **Diagnostics**: Click "Diagnose" to query device problem codes and IRQ allocation

### Usage (CLI)

```bash
# Scan all hardware devices
cargo run --release -- scan

# Check for driver updates
cargo run --release -- check-updates

# Diagnose a specific device
cargo run --release -- diagnose "PCI\VEN_10DE&DEV_2684"
```

### Driver Database

The file `data/drivers.tsv` contains known drivers with official download URLs.
TSV format: `driver_id  name  version  manufacturer  device_ids  download_url  checksum  score`

To add a new driver, append a row with the VEN&DEV device ID. The update checker
matches devices by `PCI\VEN_XXXX&DEV_XXXX` prefix.

### Audit Log

All operations are logged to `%ProgramData%\MAIDOS\DriverManager\audit.log`:

```
[MAIDOS-AUDIT] 2026-02-06 15:04:01 SCAN ALL SUCCESS 376 devices
[MAIDOS-AUDIT] 2026-02-06 15:05:04 DOWNLOAD https://... SUCCESS 91321 bytes, trusted=true
```

### MSI Installer

Requires [WiX Toolset v3.14](https://wixtoolset.org/):

```bash
cd installer
heat dir ../publish -cg PublishFiles -dr INSTALLFOLDER -srd -ag -sfrag -out HeatOutput.wxs
candle Product.wxs HeatOutput.wxs -dVersion=0.2.2.0
light Product.wixobj HeatOutput.wixobj -ext WixUIExtension -b ../publish -o MAIDOS-Driver-0.2.2.msi
```

## Project Structure

```
MAIDOS-Driver/
├── Cargo.toml
├── src/
│   ├── lib.rs                        # Crate root
│   ├── main.rs                       # CLI binary
│   ├── ffi.rs                        # 16 FFI exports (C ABI)
│   ├── core/
│   │   ├── detect/                   # SetupDI hardware enumeration
│   │   ├── install/                  # pnputil driver install + rollback
│   │   ├── backup/                   # pnputil driver export + disk space check
│   │   ├── restore/                  # Driver restore from backup
│   │   ├── update/                   # DB + WU update check, BITS download, pnputil apply
│   │   ├── diagnose/                 # CM_Get_DevNode_Status + IRQ diagnostics
│   │   ├── driver_match/             # VEN&DEV device-to-driver matching
│   │   ├── download/                 # BITS downloader with trusted whitelist
│   │   ├── verify/                   # Driver signature verification
│   │   └── audit.rs                  # File-based audit logging
│   ├── platform/windows/
│   │   ├── service_controller.rs     # SCM service management
│   │   ├── registry_manager.rs       # Registry read/write
│   │   ├── wmi_queries.rs            # WMI COM queries
│   │   └── system_info.rs            # OS info + elevation check
│   ├── MAIDOS.Driver.Service/        # C# P/Invoke service layer
│   └── MAIDOS.Driver.App/            # C# WPF application
├── data/
│   └── drivers.tsv                   # Driver database (27 records)
├── installer/
│   └── Product.wxs                   # WiX MSI installer definition
├── tests/
│   └── hardware_detection_test.rs    # Integration test
├── SPEC-MAIDOS-Driver-v2.0.md       # Full specification (AC-001~AC-020)
├── CHANGELOG.md                      # Version history
└── LICENSE                           # MIT
```

## FFI Interface

The DLL exports 16 C-compatible functions consumed by the C# service via `[DllImport]`:

| Export | Purpose |
|--------|---------|
| `init_driver_system` | Initialize logging + subsystems |
| `scan_devices` | Enumerate all hardware devices |
| `free_devices` | Free device array memory |
| `get_device_count` | Return cached device count |
| `install_driver_c` | Install driver from INF path |
| `uninstall_driver_c` | Remove device driver |
| `backup_drivers_c` | Export OEM drivers to directory |
| `free_backup_entries` | Free backup array memory |
| `restore_drivers_c` | Reinstall drivers from backup |
| `check_driver_update_c` | Check single device for updates |
| `check_all_updates_c` | Batch update check |
| `free_update_infos` | Free update array memory |
| `download_update_c` | Download driver package |
| `apply_update_c` | Apply downloaded update |
| `diagnose_device_c` | Run device diagnostics |
| `free_diagnostic_info` | Free diagnostic struct memory |
| `rollback_driver_c` | Rollback device to backup driver |

## License

MIT
