# MAIDOS-Forge Deployment Guide

| Field       | Value                                      |
|-------------|--------------------------------------------|
| Product     | MAIDOS-Forge                               |
| Version     | 3.0                                        |
| Type        | Local CLI tool (no cloud / no server)      |
| Distribution| Source code (cargo build + dotnet build)   |

## 1. Prerequisites

### Required

| Component      | Minimum Version | Verify Command              |
|----------------|-----------------|-----------------------------|
| Rust toolchain | 1.70+           | `rustc --version`           |
| Cargo          | 1.70+           | `cargo --version`           |
| .NET SDK       | 8.0+            | `dotnet --version`          |

### Optional (per-language plugins)

Install only the compilers for the languages you intend to use.

| Plugin Language | Toolchain Required         | Verify Command          |
|-----------------|----------------------------|-------------------------|
| C / C++         | MSVC / GCC / Clang         | `cl`, `gcc --version`   |
| Go              | Go 1.21+                   | `go version`            |
| Python          | Python 3.10+               | `python --version`      |
| Java            | JDK 17+                    | `javac -version`        |
| TypeScript      | Node 18+ / tsc 5+          | `tsc --version`         |

Additional compilers follow the same pattern. Each plugin validates its own toolchain at load time.

## 2. Build from Source

### 2.1 Clone

```bash
git clone https://github.com/AceGod818/MAIDOS-Forge.git
cd MAIDOS-Forge
```

### 2.2 Build Rust Core

```bash
cargo build --release
```

Output: `target/release/libmaidos_forge_core.{dll,so,dylib}`

### 2.3 Build .NET CLI

```bash
dotnet build src/Forge.Cli/ -c Release
```

### 2.4 Publish Self-Contained Binary

```bash
# Windows x64
dotnet publish src/Forge.Cli/ -c Release -r win-x64 --self-contained -o publish/win-x64

# Linux x64
dotnet publish src/Forge.Cli/ -c Release -r linux-x64 --self-contained -o publish/linux-x64

# macOS x64
dotnet publish src/Forge.Cli/ -c Release -r osx-x64 --self-contained -o publish/osx-x64

# macOS ARM64
dotnet publish src/Forge.Cli/ -c Release -r osx-arm64 --self-contained -o publish/osx-arm64
```

## 3. Install

### Option A: Copy to PATH

Copy the published binary to a directory on your system PATH.

```bash
# Windows (PowerShell, run as Administrator)
Copy-Item publish\win-x64\forge.exe C:\Tools\forge.exe

# Linux / macOS
cp publish/linux-x64/forge /usr/local/bin/forge
chmod +x /usr/local/bin/forge
```

### Option B: dotnet tool install

```bash
dotnet tool install --global MaidOS.Forge.Cli
```

After installation, `forge` is available globally.

### Verify Installation

```bash
forge --version
# Expected: MAIDOS-Forge 3.0.x
```

## 4. Platform Support

| Platform       | Architecture | Status    | Notes                      |
|----------------|-------------|-----------|----------------------------|
| Windows 10/11  | x64         | Supported | Primary development target |
| Ubuntu 22.04+  | x64         | Supported |                            |
| macOS 13+      | x64         | Supported |                            |
| macOS 13+      | ARM64       | Supported | Apple Silicon native       |

Windows ARM64 and Linux ARM64 are not officially tested.

## 5. Plugin Deployment

Plugins are .NET assemblies (.dll) placed in the `plugins/` directory relative to the Forge executable.

### Directory Layout

```
forge.exe (or forge)
plugins/
  MaidOS.Forge.Plugin.Cpp.dll
  MaidOS.Forge.Plugin.Go.dll
  MaidOS.Forge.Plugin.Python.dll
  ...
  plugin.json              # plugin manifest (auto-generated)
```

### Adding a Plugin

1. Build the plugin project: `dotnet build src/Plugins/Forge.Plugin.Cpp/ -c Release`
2. Copy the output DLL to `plugins/`.
3. Run `forge plugins list` to verify detection.

### Removing a Plugin

Delete the DLL from `plugins/`. No restart required -- plugins are loaded per invocation.

## 6. Configuration

Forge reads `forge.toml` from the current working directory or the path specified by `--config`.

```toml
[forge]
default_lang = "rust"
parallel_jobs = 4

[paths]
plugins = "./plugins"
output  = "./build"

[logging]
level = "info"   # trace | debug | info | warn | error
```

## 7. Uninstall

### If installed via copy

Delete the binary and the `plugins/` directory.

### If installed via dotnet tool

```bash
dotnet tool uninstall --global MaidOS.Forge.Cli
```
