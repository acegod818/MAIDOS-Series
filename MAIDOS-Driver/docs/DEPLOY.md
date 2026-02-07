# Deployment Guide - MAIDOS-Driver

## 1. Overview

MAIDOS-Driver is distributed as a self-contained MSI installer built with WiX Toolset v3.14. The installer bundles the .NET 10 runtime, the C# WPF application, the Rust cdylib engine, and the driver database into a single package.

## 2. Build Prerequisites

| Tool | Version | Path |
|------|---------|------|
| Rust (stable) | latest | System PATH |
| .NET SDK | 10.0 | System PATH |
| WiX Toolset | v3.14 | C:\Program Files (x86)\WiX Toolset v3.14\bin\ |
| Visual Studio Build Tools | 2022+ | For C# compilation |

## 3. Build Pipeline

### 3.1 Build the Rust Engine

```batch
cargo build --release
cargo clippy -- -D warnings
cargo test
```

Output: target/release/maidOS_driver.dll

### 3.2 Publish the C# Application

```batch
dotnet publish -c Release -r win-x64 --self-contained -o publish
```

### 3.3 Copy Artifacts

```batch
copy targetelease\maidOS_driver.dll publishxcopy data\*.tsv publish\data\ /I
```

### 3.4 Build the MSI Installer

```batch
cd installer
candle.exe Product.wxs -o obj\Product.wixobj
light.exe obj\Product.wixobj -o out\MAIDOS-Driver.msi -b ..\publish
```

Output: installer/out/MAIDOS-Driver.msi (approximately 60 MB)

## 4. Installer Contents

| Component | Location in Install Dir | Source |
|-----------|------------------------|--------|
| MAIDOS-Driver.exe | Root | dotnet publish output |
| maidOS_driver.dll | Root | Rust cargo build |
| drivers.tsv | data/ | Local database |
| .NET runtime DLLs | Root | Self-contained publish |

## 5. Installation Methods

### 5.1 Interactive Installation

```batch
msiexec /i MAIDOS-Driver.msi
```

### 5.2 Silent Installation

```batch
msiexec /i MAIDOS-Driver.msi /quiet /norestart
```

### 5.3 Silent Installation with Log

```batch
msiexec /i MAIDOS-Driver.msi /quiet /norestart /l*v install.log
```

## 6. Installed Directory Structure

```
C:\Program Files\MAIDOS\DriverManager  MAIDOS-Driver.exe
  maidOS_driver.dll
  data    drivers.tsv

%ProgramData%\MAIDOS\DriverManager  logs  backups  drivers.tsv  (user-updated copy)
```

## 7. Uninstallation

Interactive: Control Panel > Programs and Features > MAIDOS-Driver > Uninstall

Silent: msiexec /x MAIDOS-Driver.msi /quiet /norestart

## 8. Upgrade Strategy

MAIDOS-Driver uses the standard MSI major upgrade mechanism:
- The UpgradeCode GUID remains constant across versions
- Installing a new version automatically removes the previous version
- User data in %ProgramData% is preserved during upgrades

## 9. Post-Installation Verification

1. Check maidOS_driver.dll is present alongside the .exe
2. Check data/drivers.tsv is present
3. Run the application and verify hardware detection works
4. Test with administrator privileges for full functionality

## 10. Signing

- **Code signing identity**: ZGWC_acegod818 <wocao@maidos.dev>
- Applied to: MAIDOS-Driver.exe, maidOS_driver.dll, MAIDOS-Driver.msi
- The same signing identity is shared across Driver, IME, and Forge products

## 11. System Requirements

| Requirement | Minimum |
|-------------|---------|
| OS | Windows 10 v1809 or Windows 11 |
| Architecture | x86-64 |
| Disk Space | 200 MB free |
| RAM | 512 MB available |
| Display | 1024x768 |
| Network | Optional (required for driver downloads) |
