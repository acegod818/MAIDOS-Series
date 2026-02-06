# Changelog

All notable changes to MAIDOS-Driver will be documented in this file.

## [0.2.2] - 2026-02-06

### Added
- **Driver update flow (AC-007~009)**: DriverDatabase TSV lookup with VEN&DEV matching → BITS download (trusted whitelist, HTTPS-only, SHA256) → `pnputil /add-driver /install`
- **Audit logging (AC-019/020)**: file-based audit trail at `%ProgramData%\MAIDOS\DriverManager\audit.log`, format: `[MAIDOS-AUDIT] {timestamp} {op} {device} {result}`
- **System restore point (AC-005)**: `Checkpoint-Computer` before every driver install
- **Disk space check (AC-015)**: 500 MB minimum free space validation before backup
- **IRQ conflict detection (AC-017)**: WMI `Win32_PnPAllocatedResource` query for real IRQ allocations
- **Driver database** (`data/drivers.tsv`): 27 records covering NVIDIA, AMD, Intel, Realtek with official download URLs
- **WiX MSI installer** (`installer/`): v0.2.2 self-contained .NET 10 + WPF, 60 MB package
- C++ reference implementations (`src/MAIDOS.Driver.Native/`)
- Download and update verification test scripts

### Changed
- `UpdateInfo` struct: added `download_url` field across Rust FFI / C# Service / WPF UI
- Update checker: DB lookup first (with direct download URL), Windows Update API fallback
- WPF UI: auto-download+install for DB matches, opens `ms-settings:windowsupdate` for WU-only
- All user-facing strings translated to English

### Fixed
- C# FFI string encoding: `PtrToStringAnsi` → `PtrToStringUTF8` (fixes garbled text)
- App crash on non-admin: added `app.manifest` requiring admin elevation

## [0.2.0] - 2026-02-06

### Added
- Real hardware enumeration via `SetupDiGetClassDevsW` + `SetupDiEnumDeviceInfo` (376 devices detected on test system)
- Driver version reading via `SetupDiOpenDevRegKey` + `RegQueryValueExW`
- Device diagnostics via `CM_Locate_DevNodeA` + `CM_Get_DevNode_Status`
- Driver backup using `pnputil /export-driver`
- Driver restore from backup using `pnputil /add-driver`
- Driver rollback FFI (`rollback_driver_c`)
- Device diagnostics FFI (`diagnose_device_c`, `free_diagnostic_info`)
- CLI tool (`maidOS-driver-cli scan`) for command-line hardware scanning
- WPF UI buttons: hardware scan, driver install, backup, rollback, diagnostics, update check
- `.gitignore` with Rust + C# + Windows patterns
- MIT LICENSE
- Professional README with architecture diagram and FFI table
- SPEC v2.0 full specification document

### Changed
- Replaced all mock/stub implementations with real Windows API calls
- Hardware detection: from hardcoded "CPU0" to real SetupDI enumeration
- Driver install: from `mock_installation_process` to `pnputil /add-driver`
- Driver uninstall: from mock to `pnputil /remove-device`
- Backup manager: from fake data to `pnputil /export-driver`
- Restore manager: from empty stub to real `.inf` re-installation
- Rollback handler: from mock to `pnputil /add-driver` on backup `.inf` files
- Version display: from registry path string to real `DriverVersion` value
- Backup path: from hardcoded `C:\MAIDOS_Driver_Backup` to `%LOCALAPPDATA%\MAIDOS\DriverBackup`

### Removed
- ~950 MB of junk directories (temp_test_MIGRATED, simple_hardware_detection_DEPRECATED, MAIDOS_Driver_v1.0, dist_native, publish_new, docs)
- Dead modules: match, match_driver, download, verify, ai, database
- Empty stub: `wmi_queries.rs`
- 13 dev-only `.bat` scripts
- 26 dev-only `.md` files
- 5 `.obj` files and other build artifacts
- `Class1.cs` empty placeholder

### Fixed
- `CM_Get_DevNode_Status` type signature: use `CM_DEVNODE_STATUS_FLAGS` / `CM_PROB` instead of raw `u32`
- `SetupDiGetDeviceRegistryPropertyW` type: use `SETUP_DI_REGISTRY_PROPERTY` instead of raw `u32`
- `DIREG_DRV` already `u32`, removed erroneous `.0` accessor
- All clippy warnings resolved (0 errors, 0 warnings)
- All code formatted with `cargo fmt`

## [0.1.0] - 2026-01-30

### Added
- Initial project structure with Rust cdylib + C# WPF architecture
- 13 FFI function stubs
- Basic project scaffolding
