# Non-Functional Requirements - MAIDOS-Driver

## 1. Overview

This document specifies the non-functional requirements (NFRs) for MAIDOS-Driver, covering performance, reliability, security, usability, and operational characteristics.

## 2. Performance Requirements

### 2.1 Startup Time

| Metric | Target | Measurement |
|--------|--------|-------------|
| Application cold start | < 8 seconds | From process launch to UI ready |
| DLL load time | < 500 ms | P/Invoke first-call overhead |
| Hardware detection | < 10 seconds | Full PCI + USB enumeration |

### 2.2 Memory Usage

| Metric | Target | Condition |
|--------|--------|-----------|
| Idle memory | < 135 MB | Application loaded, no active operations |
| Peak memory during scan | < 200 MB | Full hardware detection + database lookup |
| Peak memory during backup | < 250 MB | ZIP compression of driver store |

### 2.3 Throughput

| Operation | Target |
|-----------|--------|
| TSV database lookup | < 50 ms per device |
| Driver download (BITS) | Limited by network bandwidth, not engine |
| Backup compression | > 20 MB/s on SSD storage |
| Restore extraction | > 30 MB/s on SSD storage |

## 3. Reliability Requirements

### 3.1 Crash Recovery
- The engine must not crash on malformed WMI responses
- WMI WBEM_S_NO_MORE_DATA (0x00040005) must be handled as normal termination, not an error
- All FFI functions must catch panics at the boundary and return error codes
- A failed driver installation must not leave the system in an inconsistent state

### 3.2 Data Integrity
- Backup ZIP archives include SHA-256 checksums for each contained file
- Restore operations validate checksums before extracting
- Downloaded driver packages are verified against published SHA-256 hashes
- TSV database corruption is detected on load and reported to the user

### 3.3 Availability
- The application must function offline for local operations (backup, restore, scan)
- Network-dependent operations (download, update check) must gracefully degrade
- Timeout for WMI queries: 30 seconds per query
- Timeout for BITS downloads: configurable, default 10 minutes per package

## 4. Security Requirements

### 4.1 Privilege Management
- The application detects whether it is running with administrator privileges at startup
- Driver installation operations are gated behind an admin privilege check
- Non-admin users can view hardware information and check for updates
- UAC elevation prompt is triggered when admin operations are requested

### 4.2 Driver Signature Verification
- All driver packages must pass WinVerifyTrust Authenticode verification before installation
- Unsigned drivers are flagged and blocked by default
- Users may override the block with an explicit acknowledgment (logged)
- Signature verification results are displayed in the UI

### 4.3 Network Security
- All download URLs must use HTTPS; HTTP URLs are rejected
- BITS downloads enforce the trusted URL whitelist
- No telemetry or user data is transmitted from the application
- The TSV database is loaded from local storage only

### 4.4 Input Validation
- All FFI string inputs are validated for null pointers and UTF-8 correctness
- File paths are sanitized to prevent directory traversal attacks
- JSON payloads are validated against expected schemas before processing

## 5. Usability Requirements

### 5.1 User Interface
- Device list must display within 2 seconds after scan completion
- Progress indicators for all long-running operations (download, backup, restore)
- Error messages must be human-readable, not raw error codes
- One-click update workflow for devices with available drivers

### 5.2 Accessibility
- UI supports Windows High Contrast themes
- All interactive elements accessible via keyboard navigation
- Screen reader compatible labels on all controls

## 6. Compatibility Requirements

| Requirement | Specification |
|-------------|---------------|
| Operating System | Windows 10 v1809+ and Windows 11 |
| Architecture | x86-64 only |
| .NET Runtime | .NET 10 (bundled in MSI) |
| Display | Minimum 1024x768 resolution |
| Disk Space | 200 MB for installation |

## 7. Maintainability Requirements

- Rust code must pass cargo clippy -- -D warnings with zero warnings
- All public FFI functions must have doc comments
- Unit test coverage target: 80% of Rust engine logic
- Modular architecture: each Windows API interaction isolated in its own module

## 8. Installability Requirements

- Single MSI installer, self-contained (no external runtime downloads)
- Silent install support: msiexec /i MAIDOS-Driver.msi /quiet
- Clean uninstall: removes all files and registry entries
- Upgrade path: in-place upgrade preserving user settings and backups
- Installer size: < 70 MB

## 9. Logging and Diagnostics

- Engine logs written to %ProgramData%\MAIDOS\DriverManager\logs- Log rotation: maximum 10 MB per file, 5 files retained
- Debug-level logging available via environment variable MAIDOS_LOG=debug
- All FFI calls logged with timestamp, function name, and result code
