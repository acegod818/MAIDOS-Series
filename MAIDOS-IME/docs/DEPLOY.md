# MAIDOS-IME -- Deployment Guide

| Field   | Value        |
|---------|--------------|
| Product | MAIDOS-IME   |
| Version | 0.2.0        |

## Prerequisites

- Rust 1.78+ with `x86_64-pc-windows-msvc` target
- CMake 3.24+
- Visual Studio 2022 Build Tools (MSVC v143)
- Windows 10 SDK (10.0.22621+)

## Build Steps

```bat
:: 1. Build Rust core
cd maidos-core
cargo build --release

:: 2. Build C++ TSF DLL
cd ..	sf
cmake -B build -G "Visual Studio 17 2022"
cmake --build build --config Release

:: 3. Copy artefacts
copy maidos-core	argetelease\maidos_core.dll   installcopy tsfuild\Release\maidos_ime.dll              install```

## Registration

```bat
regsvr32 /s install\maidos_ime.dll
```

## Unregistration

```bat
regsvr32 /u /s install\maidos_ime.dll
```

## Verification

- Open Notepad, switch IME, type test input
- Confirm candidate window appears

*MAIDOS-IME DEPLOY v0.2.0 -- CodeQC Gate C Compliant*
