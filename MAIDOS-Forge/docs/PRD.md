# MAIDOS-Forge Product Requirements Document

> **Version**: v3.0
> **Date**: 2026-02-07
> **Status**: Governor Approved
> **CodeQC**: v3.0 Gate C Compliance

---

## 1. Product Vision

MAIDOS-Forge is a cross-language compilation framework that provides **one CLI to compile any supported language**. It acts as a unified frontend that routes compilation requests to native compilers, standardizes error output, and supports cross-compilation -- all through a single `forge` command.

Forge does not replace native compilers. It orchestrates them.

---

## 2. Target Users

| Persona | Description | Primary Need |
|:--------|:------------|:-------------|
| Cross-language Developer | Works with C, Rust, Go in a single project | Single build command across languages |
| Polyglot Team | Team members use different languages for different modules | Unified error format, consistent tooling |
| Systems Programmer | Needs cross-compilation for multiple targets | `forge build --target linux-arm64` workflow |
| Plugin Author | Wants to add support for niche languages | Well-defined ILanguagePlugin interface |

---

## 3. Core Features

### 3.1 Single-Command Compilation (FR-001)

```
forge build c hello.c        # Invokes clang/gcc, produces binary
forge build rust main.rs      # Invokes rustc, produces binary
forge build go main.go        # Invokes go build, produces binary
```

One command shape for all 15 supported languages. Forge discovers the appropriate native compiler, invokes it with correct flags, and produces the artifact.

### 3.2 Unified Error Format (FR-003)

All compiler errors are normalized into a consistent JSON structure:

```json
{
  "file": "hello.c",
  "line": 10,
  "column": 5,
  "severity": "error",
  "message": "undeclared identifier 'x'",
  "lang": "c"
}
```

This enables tooling (editors, CI pipelines) to consume errors from any language without custom parsers.

### 3.3 Cross-Compilation (FR-004)

```
forge build c hello.c --target linux-x64      # Produces ELF binary
forge build go main.go --target linux-arm64   # Produces ARM64 Linux binary
```

Supported for Tier A languages (C, C++, Rust, Go, C#). Forge configures the correct cross-compiler toolchain automatically.

### 3.4 Plugin System (FR-006)

Third-party language support via the `ILanguagePlugin` interface:

- `GetCapabilities()` -- declares language name, extensions, supported features
- `ValidateToolchainAsync()` -- checks if the native compiler is installed
- `CompileAsync()` -- performs compilation
- `ExtractInterfaceAsync()` -- extracts public API from artifacts
- `GenerateGlue()` -- generates FFI binding code

Plugins are loaded from the `plugins/` directory. No recompilation of Forge required.

### 3.5 Toolchain Detection (FR-002)

```
forge check go
# {"found": true, "version": "1.21.5", "path": "/usr/local/go/bin/go"}
```

Detects all installed compilers/runtimes with version and path information.

### 3.6 CLI Command Set (FR-007)

| Command | Function | Priority |
|:--------|:---------|:--------:|
| `forge build` | Compile source files | P0 |
| `forge check` | Detect toolchains | P0 |
| `forge clean` | Remove build artifacts | P0 |
| `forge init` | Initialize a new project | P1 |
| `forge watch` | File-watch auto-rebuild | P1 |
| `forge graph` | Dependency graph visualization | P2 |
| `forge toolchain` | Toolchain management | P1 |
| `forge plugin` | Plugin management (list/install/remove) | P1 |

---

## 4. Language Support Tiers

| Tier | Definition | Languages | Count |
|:-----|:-----------|:----------|:-----:|
| **A (Full)** | Compile + Check + Cross-compile + Interface extraction | C, C++, C#, Rust, Go | 5 |
| **B (Basic)** | Compile + Check, cross-compile optional | Python, JavaScript, TypeScript, Java, Kotlin, Swift, Ruby, Dart, Haskell, Elixir | 10 |
| **C (Community)** | Plugin skeleton + documentation | Lua, Perl, R, PHP, etc. | Unlimited |

**v3.0 deliverable: 5 full + 10 basic = 15 languages compiling.**

---

## 5. Success Metrics

| Metric | Target | Measurement |
|:-------|:-------|:------------|
| Languages compiling | 15 | Integration tests per language |
| Parser overhead (Rust core) | <10ms | Benchmark tests on tree-sitter path |
| Single-file overhead vs native | <500ms | Timed comparison tests |
| Resident memory | <100MB | Memory profiling |
| Test coverage (Rust) | >=70% | `cargo tarpaulin` |
| Test coverage (C#) | >=60% | `dotnet test --collect` |
| Build warnings | 0 | `cargo build` + `dotnet build` |

---

## 6. Non-Goals

| What Forge Is Not | Rationale |
|:-------------------|:----------|
| **Not an IDE** | No editor, no LSP server, no UI (except optional Forge.Studio) |
| **Not a package manager** | Does not replace npm, cargo, pip, or any language-specific package manager |
| **Not a build system** | Does not replace CMake, Make, Gradle, or MSBuild -- Forge invokes them |
| **Not a language runtime** | Does not interpret or JIT-compile code |
| **Not a CI system** | Single-machine tool, not a distributed build service |

---

## 7. Platform Support

| Platform | Architecture | Tier A | Tier B |
|:---------|:-------------|:------:|:------:|
| Windows | x64 | Yes | Yes |
| Linux | x64 | Yes | Yes |
| macOS | x64, ARM64 | Yes | -- |
| WebAssembly | WASM | -- | Yes (JS/TS) |

---

## 8. Technology Stack

| Layer | Technology | Version | Purpose |
|:------|:-----------|:--------|:--------|
| Core (hot path) | Rust | 1.70+ | Parser (tree-sitter), Checker, Builder, FFI exports |
| Orchestration | C# .NET | 8.0 | PluginManager, BuildOrchestrator, CrossCompiler |
| Parsing | Tree-sitter | latest | Incremental parsing for C/C++/Rust |
| FFI bridge | P/Invoke | -- | C# calls into Rust cdylib |
| Compilers | Native | -- | GCC/Clang, rustc, go, dotnet, javac, node, etc. |

---

## 9. Deliverable Structure

```
MAIDOS-Forge/
  maidos-forge-core/        # Rust core (parser/checker/builder/ffi)
  maidos-forge-cli/         # Rust CLI entry point
  src/
    Forge.Core.New/         # C# orchestration layer
    Forge.Cli/              # C# CLI command routing
    Forge.Plugins/          # Language plugins (15 Tier A/B)
    Forge.Tests/            # C# test suite
    Forge.Studio/           # Optional Avalonia GUI
    Forge.VSCode/           # VS Code extension
  docs/                     # CodeQC v3.0 compliance documents
  evidence/                 # CodeQC gate evidence
```

---

*MAIDOS-Forge PRD v3.0 -- CodeQC Gate C Compliant*
