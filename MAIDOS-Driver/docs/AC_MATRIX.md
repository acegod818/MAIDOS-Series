# Acceptance Criteria Matrix - MAIDOS-Driver

## Overview

This matrix defines the 20 acceptance criteria (AC-001 through AC-020) for MAIDOS-Driver. Each criterion is independently testable and mapped to a product requirement. All 20 criteria have been implemented and verified.

## Acceptance Criteria

### AC-001: WMI Hardware Enumeration
- **Given**: The application is running with standard user or admin privileges
- **When**: detect_hardware() is called
- **Then**: A JSON array of all PnP devices is returned with VEN/DEV IDs
- **Status**: PASS

### AC-002: SetupDI Device Properties
- **Given**: WMI has returned a list of device instances
- **When**: SetupDI queries each device for registry properties
- **Then**: Device class, friendly name, and driver version are populated
- **Status**: PASS

### AC-003: PCI Vendor/Device ID Parsing
- **Given**: A hardware ID string like PCI\VEN_8086&DEV_1901
- **When**: The ID is parsed by the PCI/USB module
- **Then**: Vendor ID 8086 and Device ID 1901 are extracted correctly
- **Status**: PASS

### AC-004: USB Vendor/Device ID Parsing
- **Given**: A hardware ID string like USB\VID_046D&PID_C52B
- **When**: The ID is parsed by the PCI/USB module
- **Then**: Vendor ID 046D and Product ID C52B are extracted correctly
- **Status**: PASS

### AC-005: TSV Database Loading
- **Given**: A valid drivers.tsv exists at the expected path
- **When**: The database module initializes
- **Then**: All rows are parsed and indexed by VEN&DEV pair
- **Status**: PASS

### AC-006: TSV Database Matching
- **Given**: A device with VEN_8086&DEV_1901 and a TSV entry for the same pair
- **When**: check_all_updates() processes the device
- **Then**: The matching driver entry with version and download URL is returned
- **Status**: PASS

### AC-007: Update Check with Download URL
- **Given**: A TSV entry with a non-empty download_url field
- **When**: The update check matches this entry
- **Then**: The returned UpdateInfo contains the download_url
- **Status**: PASS

### AC-008: BITS Download with SHA-256 Verification
- **Given**: A valid HTTPS download URL and expected SHA-256 hash
- **When**: download_update() is called
- **Then**: The file is downloaded via BITS and SHA-256 matches
- **Status**: PASS

### AC-009: Driver Installation via pnputil
- **Given**: A valid driver package with .inf file, running as admin
- **When**: apply_update() is called with the .inf path
- **Then**: pnputil /add-driver /install succeeds
- **Status**: PASS

### AC-010: Windows Update Fallback
- **Given**: A device with no TSV database match
- **When**: check_all_updates() processes the device
- **Then**: The source field is set to "windows_update" in the result
- **Status**: PASS

### AC-011: Authenticode Signature Verification
- **Given**: A signed driver file (.sys or .cat)
- **When**: verify_driver_signature() is called
- **Then**: Returns 0 for valid signatures, non-zero for invalid
- **Status**: PASS

### AC-012: Driver Backup to ZIP
- **Given**: The system has third-party drivers installed
- **When**: backup_drivers() is called with an output path
- **Then**: A ZIP file is created containing all third-party driver files
- **Status**: PASS

### AC-013: Driver Restore from ZIP
- **Given**: A valid backup ZIP created by AC-012
- **When**: restore_drivers() is called with the backup path
- **Then**: All drivers in the archive are installed successfully
- **Status**: PASS

### AC-014: FFI String Marshalling
- **Given**: A string containing ASCII and extended characters
- **When**: Passed through C# StringToCoTaskMemUTF8 to Rust CStr
- **Then**: The string is received intact without encoding corruption
- **Status**: PASS

### AC-015: Memory Leak Prevention
- **Given**: Multiple sequential calls to detect_hardware()
- **When**: Each returned pointer is freed via driver_free_string()
- **Then**: Process memory does not grow unboundedly
- **Status**: PASS

### AC-016: Admin Privilege Detection
- **Given**: The application is launched with or without admin rights
- **When**: The privilege check runs at startup
- **Then**: Admin-only operations are gated appropriately
- **Status**: PASS

### AC-017: HTTPS Enforcement
- **Given**: A download URL using HTTP (not HTTPS)
- **When**: download_update() is called with the HTTP URL
- **Then**: The operation is rejected with error code 1
- **Status**: PASS

### AC-018: WMI STA Thread Safety
- **Given**: The engine is called from a C# managed thread
- **When**: WMI queries execute internally
- **Then**: COM is initialized on a dedicated STA thread, no MTA deadlock
- **Status**: PASS

### AC-019: Graceful Offline Handling
- **Given**: The system has no internet connectivity
- **When**: download_update() or online update check is attempted
- **Then**: A descriptive error is returned, no crash or hang
- **Status**: PASS

### AC-020: Startup Performance
- **Given**: A clean boot of the application on reference hardware
- **When**: The process is launched and UI becomes responsive
- **Then**: Total startup time is under 8 seconds
- **Status**: PASS

## Summary Matrix

| AC | Title | Module | Status |
|----|-------|--------|--------|
| AC-001 | WMI Hardware Enumeration | wmi.rs | PASS |
| AC-002 | SetupDI Device Properties | setupdi.rs | PASS |
| AC-003 | PCI VEN/DEV Parsing | pci_usb.rs | PASS |
| AC-004 | USB VID/PID Parsing | pci_usb.rs | PASS |
| AC-005 | TSV Database Loading | database.rs | PASS |
| AC-006 | TSV Database Matching | database.rs | PASS |
| AC-007 | Update Check with URL | update.rs | PASS |
| AC-008 | BITS Download + SHA-256 | bits.rs | PASS |
| AC-009 | pnputil Installation | update.rs | PASS |
| AC-010 | Windows Update Fallback | update.rs | PASS |
| AC-011 | Authenticode Verification | trust.rs | PASS |
| AC-012 | Driver Backup to ZIP | backup.rs | PASS |
| AC-013 | Driver Restore from ZIP | backup.rs | PASS |
| AC-014 | FFI String Marshalling | ffi.rs | PASS |
| AC-015 | Memory Leak Prevention | ffi.rs | PASS |
| AC-016 | Admin Privilege Detection | ffi.rs | PASS |
| AC-017 | HTTPS Enforcement | bits.rs | PASS |
| AC-018 | WMI STA Thread Safety | wmi.rs | PASS |
| AC-019 | Graceful Offline Handling | bits.rs | PASS |
| AC-020 | Startup Performance | lib.rs | PASS |

## Test Coverage

- **Total acceptance criteria**: 20
- **Passing**: 20
- **Failing**: 0
- **Unit tests**: 29 + 1 (30 total, 0 ignored)
