# MAIDOS-Forge Backup and Disaster Recovery

| Field   | Value                                              |
|---------|----------------------------------------------------|
| Product | MAIDOS-Forge                                       |
| Version | 3.0                                                |
| Type    | Local CLI tool -- no server state to back up       |

## 1. Overview

MAIDOS-Forge is distributed as source code and produces deterministic build artifacts. There is no database, no server state, and no cloud deployment. "Disaster recovery" means restoring the ability to build from a clean machine.

## 2. What to Back Up

| Component          | Location                          | Backup Method         | Priority |
|--------------------|-----------------------------------|-----------------------|----------|
| Source code        | Git repository (GitHub)           | Git push              | Critical |
| forge.toml         | Project root                      | Committed to Git      | Critical |
| Plugin manifests   | plugins/plugin.json per plugin    | Committed to Git      | High     |
| Plugin source      | src/Plugins/                      | Committed to Git      | High     |
| Build artifacts    | build/ or target/                 | Not backed up         | None     |
| User templates     | templates/                        | Committed to Git      | Medium   |

### Items That Do NOT Need Backup

- **Build artifacts** (`target/`, `build/`): Fully reproducible from source.
- **Downloaded toolchains**: Reinstallable via standard package managers.
- **.NET runtime**: Reinstallable via `dotnet` installer.

## 3. Source Code

### Primary Backup: GitHub

```bash
git remote -v
# origin  https://github.com/AceGod818/MAIDOS-Forge.git
```

All source, configuration, and plugin code is tracked in Git.

### Backup Verification

```bash
# Verify local repo matches remote
git fetch origin
git status
git diff origin/main
```

### Secondary Backup (Optional)

For additional safety, mirror to a second remote:

```bash
git remote add backup https://backup-host/MAIDOS-Forge.git
git push backup --all
git push backup --tags
```

## 4. Configuration

### forge.toml

The project configuration file lives in the repository root and is version-controlled.

```bash
# Verify it's tracked
git ls-files forge.toml
```

If forge.toml is accidentally deleted or corrupted:

```bash
# Restore from Git
git checkout -- forge.toml

# Or regenerate defaults
forge init --config-only
```

### Plugin Manifests

Each plugin directory contains a `plugin.json` manifest describing the plugin's capabilities, version, and dependencies. These are committed to Git alongside plugin source code.

## 5. Build Artifact Reproducibility

All build outputs are reproducible from source:

```bash
cargo build --release
dotnet build src/Forge.Cli/ -c Release
dotnet build src/Plugins/ -c Release
```

No build artifacts require backup. If artifacts are lost, rebuild from source.

## 6. Recovery Procedures

### 6.1 Recover on the Same Machine

**Scenario**: Build directory corrupted or accidentally deleted.

```bash
forge clean
cargo build --release
dotnet build src/Forge.Cli/ -c Release
```

Time to recover: ~2-5 minutes (compile time).

### 6.2 Recover on a New Machine

**Scenario**: Machine failure, need to set up from scratch.

**Step 1: Install prerequisites**

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# .NET 8.0
# Windows: download from https://dotnet.microsoft.com/download/dotnet/8.0
# Linux:
sudo apt install dotnet-sdk-8.0
```

**Step 2: Clone and build**

```bash
git clone https://github.com/AceGod818/MAIDOS-Forge.git
cd MAIDOS-Forge
cargo build --release
dotnet build src/Forge.Cli/ -c Release
dotnet build src/Plugins/ -c Release
cp src/Plugins/*/bin/Release/net8.0/*.dll plugins/
```

**Step 3: Verify**

```bash
forge doctor
forge check
```

Time to recover: ~15-30 minutes (including toolchain installation).

### 6.3 Recover a Specific Version

```bash
git checkout v3.0.0
cargo build --release
dotnet build src/Forge.Cli/ -c Release
```

All releases are tagged in Git. Any past version can be rebuilt.

## 7. Recovery Time Objectives

| Scenario              | RTO           | Notes                              |
|-----------------------|---------------|------------------------------------|
| Corrupt build output  | < 5 minutes   | `forge clean` + rebuild            |
| Lost configuration    | < 1 minute    | `git checkout -- forge.toml`       |
| New machine setup     | < 30 minutes  | Toolchain install + clone + build  |
| Full repo loss        | < 5 minutes   | Clone from GitHub mirror           |

## 8. Testing Recovery

Periodically verify that recovery works:

```bash
# Simulate clean-room build
git clone https://github.com/AceGod818/MAIDOS-Forge.git /tmp/forge-test
cd /tmp/forge-test
cargo build --release
dotnet build src/Forge.Cli/ -c Release
forge doctor
```

Run this test on CI to catch dependency drift or undocumented build requirements.
