# MAIDOS-Driver -- Deployment Guide

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved

---

## 1. Installer Technology

MAIDOS-Driver is distributed as a Windows Installer package (MSI) built with WiX Toolset v3.14.
The MSI bundles a self-contained .NET 10.0 WPF application and the Rust cdylib DLL.

| Component | Details |
|:----------|:--------|
| Installer format | MSI (Windows Installer) |
| Build tool | WiX Toolset v3.14, `C:\Program Files (x86)\WiX Toolset v3.14\bin\` |
| Package size | ~60 MB |
| Installed size | ~135 MB |
| Runtime | Self-contained .NET 10.0 (no separate runtime install required) |

## 2. Build Process

### 2.1 Rust Engine

```
cd MAIDOS-Driver
cargo build --release
# Output: target/release/maidOS_driver.dll
```

### 2.2 C# Application

```
cd src/MAIDOS.Driver.App
dotnet publish -c Release -r win-x64 --self-contained true -o ../../publish
# Copy maidOS_driver.dll into publish/
copy ..\..\target\release\maidOS_driver.dll ..\..\publish\
```

### 2.3 MSI Package

```
cd installer
candle MAIDOS-Driver.wxs -o MAIDOS-Driver.wixobj
light MAIDOS-Driver.wixobj -o MAIDOS-Driver.msi -b ../publish
```

The `light` linker requires `-b publish` to resolve file references from the WiX source.

## 3. Application Manifest

The WPF executable includes an embedded manifest requesting administrator elevation:

```xml
<requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
```

This ensures UAC prompts for install, update, and rollback operations (NFR-003).

## 4. Installation Layout

```
C:\Program Files\MAIDOS\DriverManager\
  MAIDOS.Driver.App.exe        (WPF entry point)
  maidOS_driver.dll            (Rust engine)
  data\drivers.tsv             (driver database)
  *.dll                        (.NET runtime + dependencies)

C:\ProgramData\MAIDOS\DriverManager\
  audit.log                    (audit log file)
  drivers.tsv                  (fallback database location)
```

## 5. System Requirements

| Requirement | Minimum |
|:------------|:--------|
| OS | Windows 10 21H2 (build 19044) or Windows 11 |
| Architecture | x64 |
| Disk space | 200 MB free |
| RAM | 4 GB (application uses < 50 MB) |
| Privileges | Administrator for install/update/rollback; standard user for scan/diagnose |

## 6. Upgrade Path

- MSI major upgrade: new version replaces previous installation automatically.
- Driver database (`drivers.tsv`) is overwritten with the latest version on upgrade.
- Audit log in `%ProgramData%` is preserved across upgrades.

## 7. Uninstallation

Standard Windows uninstall via Settings > Apps or `msiexec /x {ProductCode}`.
The `%ProgramData%\MAIDOS\DriverManager\` directory is preserved for audit log retention.
