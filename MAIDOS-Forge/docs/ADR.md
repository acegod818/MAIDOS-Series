# MAIDOS-Forge Architecture Decision Records

> **Version**: v3.0
> **Date**: 2026-02-07
> **CodeQC**: v3.0 Gate C Compliance

---

## ADR-001: Rust for Core Parser and Checker

**Status**: Accepted
**Date**: 2025-11-15
**Deciders**: MAIDOS Technical Committee

### Context

MAIDOS-Forge needs a high-performance parsing engine that can process source files in under 10ms (NFR-001). The parser is on the critical hot path -- every compilation request passes through it. Tree-sitter, the most mature incremental parsing library, is written in C with first-class Rust bindings.

### Decision

Use Rust for the core parsing and checking layer (`maidos-forge-core`). Compile as a `cdylib` for consumption by the C# orchestration layer via P/Invoke.

### Rationale

| Factor | Rust | C# | C++ |
|:-------|:-----|:---|:----|
| Tree-sitter integration | First-class (`tree-sitter` crate) | Requires C interop | Native but manual memory management |
| Parse latency | Sub-millisecond, zero-cost abstractions | GC pauses risk, JIT warmup | Comparable but harder to maintain |
| Memory safety | Guaranteed at compile time | GC-managed | Manual, error-prone |
| FFI export | `extern "C"` + `cdylib` is straightforward | N/A (is the consumer) | Manual header management |
| Error handling | `Result<T, E>` with `thiserror` | Exceptions (cannot cross FFI) | Error codes or exceptions |

### Consequences

- **Positive**: Tree-sitter grammars for C, C++, and Rust are directly usable (`tree_sitter_c`, `tree_sitter_cpp`, `tree_sitter_rust` crates).
- **Positive**: Deterministic performance with no GC pauses.
- **Positive**: `serde_json` provides zero-copy JSON serialization for FFI return values.
- **Negative**: Build requires both Rust and .NET toolchains.
- **Negative**: Debugging across FFI boundary requires dual tooling.

### Implementation

- Crate: `maidos-forge-core` (workspace member)
- Output: `target/release/maidos_forge_core.dll` (cdylib)
- Key modules: `parser.rs`, `checker.rs`, `ffi.rs`
- FFI surface: 10 exported functions (6 feature + 4 memory management)

---

## ADR-002: C# for Orchestration Layer

**Status**: Accepted
**Date**: 2025-11-15
**Deciders**: MAIDOS Technical Committee

### Context

The orchestration layer manages plugin loading, build scheduling, cross-compilation configuration, incremental caching, and linker invocation. These are I/O-bound tasks that benefit from async/await patterns and dynamic assembly loading.

### Decision

Use C# with .NET 8.0 for the orchestration layer (`Forge.Core.New`, `Forge.Cli`).

### Rationale

| Factor | C# .NET 8.0 | Rust | Go |
|:-------|:-------------|:-----|:---|
| Plugin ecosystem | `Assembly.LoadFrom()`, mature | `libloading`, less ergonomic | `plugin` package, limited |
| Async I/O | `async/await`, first-class | `tokio`, heavier setup | Goroutines, different model |
| Cross-platform | .NET 8.0 runs on Windows/Linux/macOS | Same | Same |
| GUI potential | Avalonia (Forge.Studio) | egui, less mature for desktop | Limited |
| Process management | `System.Diagnostics.Process` | `std::process::Command` | `os/exec` |
| Developer familiarity | MAIDOS team proficiency | MAIDOS team proficiency | Lower |

### Consequences

- **Positive**: Rich `async Task<T>` model for parallel module compilation.
- **Positive**: `ILanguagePlugin` interface with `Task<CompileResult> CompileAsync()` is natural.
- **Positive**: .NET 8.0 self-contained deployment (no framework dependency on target).
- **Positive**: Avalonia enables optional GUI (Forge.Studio) from same codebase.
- **Negative**: .NET runtime adds ~30MB baseline memory.
- **Negative**: P/Invoke requires careful string marshaling (UTF-8 via `PtrToStringUTF8`).

### Implementation

- Projects: `Forge.Cli/`, `Forge.Core.New/`, `Forge.Plugins/`, `Forge.Tests/`
- Target: .NET 8.0 (self-contained for deployment)
- Key classes: `BuildOrchestrator`, `PluginHost`, `LinkerManager`, `IncrementalBuildManager`

---

## ADR-003: P/Invoke FFI Bridge

**Status**: Accepted
**Date**: 2025-11-20
**Deciders**: MAIDOS Technical Committee

### Context

The C# orchestration layer needs to call Rust core functions for parsing and checking. The bridge must be stable, low-overhead, and safe across the language boundary.

### Alternatives Considered

| Option | Pros | Cons |
|:-------|:-----|:-----|
| **P/Invoke (chosen)** | Mature, well-documented, no extra runtime | Manual memory management, C-only ABI |
| gRPC / IPC | Language-agnostic, strong typing | Process overhead, serialization cost |
| Shared memory | Zero-copy | Complex synchronization, platform-specific |
| C++/CLI | Direct interop | Windows-only, complex build |
| COM | Rich interface | Windows-only, heavy boilerplate |

### Decision

Use P/Invoke with Rust `extern "C"` functions exported from a `cdylib`. All data crosses the boundary as C strings (UTF-8 JSON) or integer error codes.

### Rationale

- P/Invoke is the standard .NET mechanism for calling native code.
- Rust `extern "C"` produces stable ABI functions.
- JSON serialization at the boundary provides clear contracts and easy debugging.
- Memory ownership is explicit: Rust allocates (`CString::into_raw`), C# reads (`PtrToStringUTF8`), Rust frees (`forge_free_string`).

### Consequences

- **Positive**: No external dependencies (no gRPC, no protobuf).
- **Positive**: In-process calls, no IPC latency.
- **Positive**: JSON at boundary is self-documenting.
- **Negative**: Must manually free strings returned by Rust (`forge_free_string`).
- **Negative**: Must use `PtrToStringUTF8` (not `PtrToStringAnsi`) because Rust CString is UTF-8.
- **Negative**: Errors cannot propagate as exceptions; must check null returns + `forge_last_error()`.

### Error Pattern

```
C# calls forge_parse_source(lang, path)
  -> Rust returns *mut c_char (JSON) or null
  -> If null: C# calls forge_last_error() -> reads error string -> forge_free_string()
  -> If non-null: C# reads JSON -> forge_free_string()
```

### FFI Memory Contract

| Allocator | Responsibility |
|:----------|:---------------|
| Rust `string_to_c()` | Allocates via `CString::into_raw()` |
| C# `Marshal.PtrToStringUTF8()` | Reads (copies to managed heap) |
| Rust `forge_free_string()` | Frees via `CString::from_raw()` |

---

## ADR-004: ILanguagePlugin Interface Design

**Status**: Accepted
**Date**: 2025-12-01
**Deciders**: MAIDOS Technical Committee

### Context

Forge must support 15+ languages with a uniform interface. Each language has different compilers, flags, and capabilities. The interface must be implementable by both the Forge team (builtin plugins) and third parties (community plugins).

### Decision

Define a C# `ILanguagePlugin` interface with 5 async methods that every language plugin must implement.

### Interface Definition

```csharp
public interface ILanguagePlugin
{
    // 1. Declare capabilities (language name, extensions, features)
    PluginCapabilities GetCapabilities();

    // 2. Check if the native compiler is installed
    Task<(bool Available, string Message)> ValidateToolchainAsync(
        CancellationToken ct = default);

    // 3. Compile source code
    Task<CompileResult> CompileAsync(
        ValidatedModuleConfig module,
        CompileConfig config,
        CancellationToken ct = default);

    // 4. Extract public API from compiled artifacts
    Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default);

    // 5. Generate cross-language FFI binding code
    GlueCodeResult GenerateGlue(
        InterfaceDescription sourceInterface,
        string targetLanguage);
}
```

### Rationale

| Method | Purpose | Why Async |
|:-------|:--------|:----------|
| `GetCapabilities()` | Discovery: what can this plugin do? | Sync -- pure data, no I/O |
| `ValidateToolchainAsync()` | Probe system for compiler | Runs `Process.Start()`, may wait for output |
| `CompileAsync()` | Invoke compiler | Process execution is I/O-bound |
| `ExtractInterfaceAsync()` | Run `nm`, reflection, etc. | Process or I/O-bound |
| `GenerateGlue()` | Generate source code strings | Sync -- pure computation, string building |

### Design Decisions

1. **5 methods, not fewer**: Separating validation from compilation enables `forge check` without compiling. Separating interface extraction from compilation enables `forge interface` standalone.

2. **CancellationToken on all async methods**: Enables `forge watch` to cancel in-progress builds on file change.

3. **`ValidatedModuleConfig` parameter**: Plugin receives pre-validated configuration, not raw TOML. Validation happens once in the orchestrator.

4. **Return types use success/failure pattern, not exceptions**: `CompileResult.IsSuccess` and `CompileResult.Error` instead of throwing. This prevents exceptions from crossing plugin boundaries and provides a uniform error model.

5. **`GenerateGlue()` is sync**: Glue code generation is pure string manipulation with no I/O. Making it async would add unnecessary complexity.

### Consequences

- **Positive**: Any .NET developer can create a language plugin by implementing 5 methods.
- **Positive**: `PluginHost` can validate, compile, and extract without knowing language specifics.
- **Positive**: Community plugins have the same interface as builtin plugins.
- **Negative**: Some languages may not support all 5 features. `ExtractInterfaceAsync` and `GenerateGlue` return null/failure for unsupported operations.
- **Negative**: Interface changes require updating all plugins (mitigated by having few methods).

---

## ADR-005: Tiered Language Support

**Status**: Accepted
**Date**: 2026-02-06
**Deciders**: MAIDOS Governor

### Context

The v2.2 specification claimed 80+ language support, which resulted in empty shell plugins that did not actually compile anything. The v3.0 specification requires honest, verifiable language support with a realistic scope.

### Decision

Adopt a three-tier language support model with clearly defined capabilities per tier.

### Tier Definitions

| Tier | Name | Definition | Languages | Count | Verification |
|:-----|:-----|:-----------|:----------|:-----:|:-------------|
| A | Full Support | Compile + Check + Cross-compile + Interface extraction -- all ACs pass | C, C++, C#, Rust, Go | 5 | All AC-001 through AC-011 |
| B | Basic Support | Compile + Check pass, cross-compile optional | Python, JavaScript, TypeScript, Java, Kotlin, Swift, Ruby, Dart, Haskell, Elixir | 10 | FR-001 through FR-003 |
| C | Community | Plugin skeleton + documentation, marked "Community Welcome" | Lua, Perl, R, PHP, etc. | Unlimited | Has `plugin.json` + README |

### Rationale

1. **Honest scope**: 15 actually working languages is better than 80 empty shells.
2. **Verifiable**: Each tier has specific acceptance criteria that can be tested.
3. **Extensible**: Tier C allows community contribution without burdening the core team.
4. **Tier A is the real product**: These 5 languages cover the primary cross-language use case (systems programming with C/C++/Rust, modern backend with Go/C#).

### Migration from v2.2

| v2.2 | v3.0 | Action |
|:-----|:-----|:-------|
| 80+ empty shell plugins | Deleted | Remove all plugins that only contain `NotImplementedException` |
| "Tier 2" (30 languages) | Tier B (10) or Tier C | Keep 10 that have real compiler invocation, move rest to community |
| "Tier 3" (50 languages) | Deleted or Tier C | Community plugin skeletons only |

### Consequences

- **Positive**: Every claimed language actually compiles in testing.
- **Positive**: Clear expectations for users: Tier A = full feature set, Tier B = basic compile.
- **Positive**: Community can contribute Tier C plugins without release pressure.
- **Negative**: v3.0 claims fewer languages than v2.2 (15 vs 80+). This is an honest tradeoff.
- **Negative**: Some users may be disappointed that their niche language is Tier C.

### Verification

Each tier has a gate:

| Tier | Gate | Evidence |
|:-----|:-----|:---------|
| A | All ACs pass + NFRs met | Integration test suite |
| B | FR-001 through FR-003 pass | Per-language hello-world compiles |
| C | Plugin loads, `GetCapabilities()` returns valid data | Plugin load test |

---

## Decision Log

| ADR | Decision | Date | Status |
|:----|:---------|:-----|:-------|
| ADR-001 | Rust for core parser | 2025-11-15 | Accepted |
| ADR-002 | C# for orchestration | 2025-11-15 | Accepted |
| ADR-003 | P/Invoke FFI bridge | 2025-11-20 | Accepted |
| ADR-004 | ILanguagePlugin 5-method interface | 2025-12-01 | Accepted |
| ADR-005 | Tiered language support (A/B/C) | 2026-02-06 | Accepted |

---

*MAIDOS-Forge ADR v3.0 -- CodeQC Gate C Compliant*
