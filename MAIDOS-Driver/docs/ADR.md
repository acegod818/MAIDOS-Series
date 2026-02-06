# MAIDOS-Driver -- Architecture Decision Records

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved

---

## ADR-001: Rust for the FFI Native Layer

**Status**: Accepted
**Context**: The native layer must call Windows APIs (SetupDI, CM, WMI, BITS, WinVerifyTrust)
and expose functions to C# via P/Invoke. Options considered: C++, C, Rust.
**Decision**: Use Rust compiled as a cdylib (`maidOS_driver.dll`).
**Rationale**:
- Memory safety without garbage collection eliminates use-after-free and buffer overflows in FFI.
- The `windows` crate (0.58.0) provides type-safe bindings to all required Windows APIs.
- `#[no_mangle] extern "C"` functions are directly callable from C# `DllImport`.
- Rust's ownership model ensures allocated memory has clear lifecycle semantics.
- `cargo test` integrates unit and integration tests without external frameworks.
**Consequences**: Developers must know Rust. Build requires MSVC linker. Crate name `maidOS-driver`
maps to `maidOS_driver` in test imports.

---

## ADR-002: WPF over WinUI 3 for the UI Layer

**Status**: Accepted
**Context**: The desktop UI must run on Windows 10 21H2+ and Windows 11 with admin elevation.
Options considered: WPF (.NET 10.0), WinUI 3, Avalonia, MAUI.
**Decision**: Use WPF on .NET 10.0.
**Rationale**:
- WPF is mature, stable, and fully supported on .NET 10.0.
- WinUI 3 has known packaging issues with admin-required applications (no elevation manifest in MSIX).
- WPF supports `requireAdministrator` in the application manifest natively.
- Rich DataGrid and binding infrastructure suits the device list and audit log views.
- Self-contained deployment via `dotnet publish` produces a standalone package for WiX.
**Consequences**: No modern Fluent UI by default (can be themed). Windows-only (acceptable per scope).

---

## ADR-003: pnputil for Driver Backup and Install

**Status**: Accepted
**Context**: Driver backup and installation require interacting with the Windows driver store.
Options considered: SetupAPI `DiInstallDriver`, direct driver store manipulation, `pnputil.exe`.
**Decision**: Use `pnputil.exe` via `std::process::Command`.
**Rationale**:
- `pnputil /export-driver *` reliably exports all OEM drivers with correct metadata.
- `pnputil /add-driver /install` handles driver store staging and device association.
- pnputil is a Microsoft-maintained tool present on all supported Windows versions.
- SetupAPI `DiInstallDriver` has undocumented failure modes on newer Windows builds.
- Process-based invocation provides clear success/failure exit codes for audit logging.
**Consequences**: Requires admin privileges. Output parsing needed for structured results.
Slight overhead from process spawning (acceptable for infrequent operations).

---

## ADR-004: BITS for Driver Download

**Status**: Accepted
**Context**: Driver update packages must be downloaded from trusted URLs. Options considered:
WinHTTP, WinINet, BITS (Background Intelligent Transfer Service), reqwest (Rust HTTP client).
**Decision**: Use BITS via COM API.
**Rationale**:
- BITS provides automatic resume on network interruption -- critical for large driver packages.
- BITS respects system proxy settings and bandwidth throttling policies.
- Downloads continue even if the application is temporarily suspended.
- BITS is a Windows system service, requiring no additional dependencies.
- A trusted source whitelist ensures only approved URLs are downloaded.
**Consequences**: COM initialization required (CoInitializeEx). BITS job management adds
complexity. SHA-256 verification of downloaded files is performed post-download.
