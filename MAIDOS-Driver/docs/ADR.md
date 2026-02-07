# Architecture Decision Records - MAIDOS-Driver

## ADR-001: Rust cdylib as the Core Engine

### Status
Accepted

### Context
The driver management engine needs to interact with low-level Windows APIs (WMI COM, SetupDI, WinVerifyTrust, BITS) while maintaining memory safety and preventing undefined behavior. The UI layer uses C# WPF for its mature Windows desktop ecosystem.

### Decision
Build the core engine as a Rust cdylib (dynamic library) exposed to C# via P/Invoke FFI.

### Rationale
- Rust provides memory safety guarantees without a garbage collector
- The cdylib target produces a standard Windows DLL compatible with P/Invoke
- The windows crate (v0.58) provides ergonomic, type-safe bindings to Win32 APIs
- Panic safety at the FFI boundary prevents crashes from propagating to the host process
- Performance-critical device enumeration benefits from zero-cost abstractions

### Consequences
- FFI string marshalling requires careful UTF-8 handling (CString/CStr)
- C# must use PtrToStringUTF8 and StringToCoTaskMemUTF8, not ANSI variants
- Crate name maidOS-driver maps to identifier maidOS_driver in Rust code

---

## ADR-002: WMI COM via STA Thread

### Status
Accepted

### Context
WMI (Windows Management Instrumentation) is the standard API for hardware enumeration on Windows. WMI uses COM, which has threading model requirements that can cause deadlocks if not handled correctly.

### Decision
Execute all WMI COM operations on a dedicated STA (Single-Threaded Apartment) thread, using a semi-async query pattern.

### Rationale
- C# WPF runs on an STA thread; calling WMI from an MTA worker thread can deadlock
- Spawning a dedicated STA thread with CoInitializeEx(COINIT_APARTMENTTHREADED) avoids conflicts
- IWbemClassObject::Next returns WBEM_S_NO_MORE_DATA (0x00040005), which is a success HRESULT and cannot be detected by is_err() -- instead, the code checks for empty property names to detect enumeration end
- Semi-async pattern: IWbemServices::ExecQuery with WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY

### Consequences
- WMI operations have a small overhead from cross-thread marshalling
- The STA thread is managed internally by the Rust engine, transparent to C#
- All WMI error handling must account for success HRESULTs that indicate completion

---

## ADR-003: TSV Database for Driver Matching

### Status
Accepted

### Context
The engine needs a local database mapping hardware VEN/DEV IDs to driver packages, versions, and download URLs. Options considered: SQLite, JSON, CSV/TSV, embedded key-value store.

### Decision
Use a tab-separated values (TSV) file as the driver database format.

### Rationale
- TSV is human-readable and easily editable with any text editor
- No additional dependencies (no SQLite native library needed)
- Fast sequential scan performance for the expected dataset size (< 10,000 entries)
- Easy to distribute and update alongside the application
- Fields naturally align in columns when viewed in a text editor

### Consequences
- No query optimization (full scan per lookup); acceptable for < 10K entries
- Schema changes require updating the parser
- Database path resolution: {exe_dir}/data/drivers.tsv for portable, %ProgramData% for installed

---

## ADR-004: BITS for Driver Downloads

### Status
Accepted

### Context
Driver packages must be downloaded reliably from remote servers. Options considered: raw HTTP client (reqwest), WinHTTP, BITS (Background Intelligent Transfer Service).

### Decision
Use BITS (Background Intelligent Transfer Service) COM API for all driver downloads.

### Rationale
- BITS is a Windows system service designed for reliable background file transfer
- Automatic retry and resume on network interruption
- Bandwidth throttling respects user network activity
- System-level integration means downloads survive application restarts
- BITS is already trusted by Windows Update, reducing security concerns

### Consequences
- BITS COM initialization adds complexity to the download path
- Downloads are limited to the trusted URL whitelist for security
- All URLs must use HTTPS; HTTP is rejected at the engine level
- SHA-256 verification is performed after BITS transfer completes

---

## ADR-005: WinVerifyTrust for Signature Verification

### Status
Accepted

### Context
Driver packages must be verified for authenticity before installation. Windows provides multiple APIs for signature verification.

### Decision
Use WinVerifyTrust with WINTRUST_ACTION_GENERIC_VERIFY_V2 for Authenticode signature verification of all driver files.

### Rationale
- WinVerifyTrust is the official Windows API for Authenticode verification
- Validates the entire certificate chain including revocation checks
- Supports .sys, .dll, .cat, and .inf file types
- Consistent with how Windows itself verifies driver signatures

### Consequences
- Revocation checks require network access; offline mode skips revocation
- Catalog-signed drivers require the catalog file to be present
- Test-signed drivers are rejected unless test signing mode is enabled on the OS

---

## ADR-006: WiX v3.14 for MSI Installer

### Status
Accepted

### Context
The application needs a Windows installer that bundles the self-contained .NET 10 runtime, the WPF application, and the Rust DLL into a single MSI package.

### Decision
Use WiX Toolset v3.14 to produce the MSI installer.

### Rationale
- WiX is the industry-standard open-source tool for MSI creation
- Declarative XML-based authoring with full MSI feature support
- Supports self-contained .NET deployment bundling
- light.exe with -b publish flag correctly resolves payload paths
- Produces a 60 MB installer with all dependencies included

### Consequences
- WiX v3.14 must be installed at C:\Program Files (x86)\WiX Toolset v3.14in- The light linker requires -b publish to locate the .NET publish output
- Installer upgrades use the standard MSI upgrade mechanism (UpgradeCode GUID)

---

## ADR-007: SetupDI for Device Registry Properties

### Status
Accepted

### Context
WMI provides basic device information but lacks detailed registry-level properties. The SetupDI API family provides comprehensive device metadata.

### Decision
Use SetupDI APIs (SetupDiGetClassDevs, SetupDiEnumDeviceInfo, SetupDiGetDeviceRegistryProperty) to enrich device data from WMI.

### Rationale
- SetupDI provides access to device-specific registry properties not exposed by WMI
- The windows crate v0.58 uses SETUP_DI_REGISTRY_PROPERTY type (not raw u32)
- Direct access to device class, friendly name, hardware ID, and driver key
- Necessary for backup operations to identify driver file locations

### Consequences
- SetupDI functions require careful handle management (SetupDiDestroyDeviceInfoList)
- Property type SETUP_DI_REGISTRY_PROPERTY must be used instead of raw u32 constants
- Some devices may not have all properties populated; missing properties handled gracefully
