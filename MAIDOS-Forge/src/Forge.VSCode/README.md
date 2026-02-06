# MAIDOS-Forge VS Code Extension

Cross-language build system integration for Visual Studio Code.

## Features

- **Project Management**: Initialize, build, and run Forge projects
- **Module Explorer**: View and manage project modules
- **Toolchain Detection**: Check available language toolchains
- **Dependency Graph**: Visualize module dependencies
- **FFI Inspector**: Analyze and generate cross-language bindings
- **Task Integration**: Run Forge commands as VS Code tasks
- **Diagnostics**: Inline error display from all language compilers

## Commands

| Command | Description |
|---------|-------------|
| `Forge: Initialize Project` | Create new forge.json |
| `Forge: Build Project` | Build all modules |
| `Forge: Rebuild Project` | Clean and rebuild |
| `Forge: Clean Project` | Remove build artifacts |
| `Forge: Run Project` | Build and execute |
| `Forge: Watch Mode` | Auto-rebuild on changes |
| `Forge: Add Module` | Create new module |
| `Forge: Show Dependency Graph` | Visualize dependencies |
| `Forge: Show FFI Interfaces` | Inspect exported functions |
| `Forge: Check Toolchains` | Verify installed compilers |

## Configuration

| Setting | Description | Default |
|---------|-------------|---------|
| `forge.executablePath` | Path to forge CLI | `forge` |
| `forge.autoWatch` | Start watch mode on open | `false` |
| `forge.buildOnSave` | Build on file save | `false` |
| `forge.defaultConfig` | Default build config | `Debug` |
| `forge.showInlineErrors` | Show errors inline | `true` |
| `forge.parallelJobs` | Parallel build jobs | `0` (auto) |

## Requirements

- Forge CLI installed and in PATH
- Language-specific toolchains for your modules

## Links

- [Forge Documentation](https://forge.maidos.dev)
- [GitHub Repository](https://github.com/maidos/forge-vscode)
