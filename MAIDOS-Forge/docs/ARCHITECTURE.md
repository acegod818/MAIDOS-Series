# MAIDOS-Forge Architecture Document

> **Version**: v3.0
> **Date**: 2026-02-07
> **CodeQC**: v3.0 Gate C Compliance

---

## 1. Layer Diagram

```
+------------------------------------------------------------------+
|                         CLI Layer (C#)                             |
|  forge build | check | clean | init | watch | graph | toolchain  |
|  forge run   | link  | ffi   | plugin                            |
|  src/Forge.Cli/Program.cs -> Commands/*.cs                       |
+------------------------------------------------------------------+
         |  ICommand.Execute(args) -> CommandResult
         v
+------------------------------------------------------------------+
|                   C# Orchestration Layer (.NET 8.0)               |
|  +---------------+ +------------------+ +-------------------+    |
|  | PluginHost    | | BuildOrchestrator| | CrossCompiler     |    |
|  | (load/register| | (schedule/cache/ | | (target platform  |    |
|  |  plugins)     | |  compile/link)   | |  resolution)      |    |
|  +---------------+ +------------------+ +-------------------+    |
|  +------------------+ +---------------------+                    |
|  | LinkerManager    | | IncrementalBuildMgr  |                    |
|  | (collect inputs, | | (hash-based skip,    |                    |
|  |  invoke linker)  | |  transitive rebuild) |                    |
|  +------------------+ +---------------------+                    |
|  src/Forge.Core.New/*.cs                                         |
+------------------------------------------------------------------+
         |  P/Invoke (cdylib FFI)        |  ILanguagePlugin
         v                               v
+-----------------------------+  +-----------------------------+
|    Rust Core (cdylib)       |  |    Language Plugins          |
|  +--------+ +---------+    |  |  CPlugin     RustPlugin     |
|  | Parser | | Checker |    |  |  CppPlugin   GoPlugin       |
|  |(tree-  | |(lint/   |    |  |  CSharpPlugin PythonPlugin  |
|  |sitter) | |analyze) |    |  |  TypeScriptPlugin AsmPlugin |
|  +--------+ +---------+    |  |  + 7 more Tier B plugins    |
|  +--------+ +------+       |  |  + community plugins/       |
|  |Builder | | FFI  |       |  +-----------------------------+
|  |(invoke | |(C#<->|       |           |
|  |compile)| |Rust) |       |           v
|  +--------+ +------+       |  +-----------------------------+
|  maidos-forge-core/src/    |  |    Native Compilers          |
+-----------------------------+  |  GCC/Clang | rustc/cargo    |
                                 |  go build  | dotnet build   |
                                 |  javac     | node/tsc       |
                                 |  python3   | swift          |
                                 +-----------------------------+
```

---

## 2. Component Descriptions

### 2.1 CLI Layer (`src/Forge.Cli/`)

**Entry point**: `Program.cs`

The CLI layer is a pure routing layer. It parses command-line arguments, resolves global options (`--verbose`, `--project`), and dispatches to the appropriate `ICommand` implementation. It does not contain business logic.

**Registered Commands** (from `Program.cs`):

| Command | Class | File |
|:--------|:------|:-----|
| `init` | `InitCommand` | `Commands/InitCommand.cs` |
| `build` | `BuildCommand` | `Commands/BuildCommand.cs` |
| `run` | `RunCommand` | `Commands/RunCommand.cs` |
| `watch` | `WatchCommand` | `Commands/WatchCommand.cs` |
| `check` | `CheckCommand` | `Commands/CheckCommand.cs` |
| `clean` | `CleanCommand` | `Commands/CleanCommand.cs` |
| `graph` | `GraphCommand` | `Commands/GraphCommand.cs` |
| `toolchain` | `ToolchainCommand` | `Commands/ToolchainCommand.cs` |
| `ffi` | `FfiCommand` | `Commands/FfiCommand.cs` |
| `link` | `LinkCommand` | `Commands/LinkCommand.cs` |
| `plugin` | `PluginCommand` | `Commands/PluginCommand.cs` |

**Data flow**: `string[] args -> ICommand.Execute(args) -> CommandResult { ExitCode, Message }`

### 2.2 C# Orchestration Layer (`src/Forge.Core.New/`)

This layer contains the core business logic for compilation orchestration.

**PluginHost** (`PluginHost.cs`)

Manages the lifecycle of language plugins. Maintains a `Dictionary<string, ILanguagePlugin>` keyed by language name (case-insensitive). Provides:
- `RegisterBuiltinPlugins()` -- registers 8 builtin plugins (C, C++, C#, Rust, Go, Python, TypeScript, ASM)
- `GetPlugin(language)` -- lookup by language name
- `GetPluginByExtension(ext)` -- lookup by file extension
- `ValidateAllToolchainsAsync()` -- parallel toolchain validation
- `GetAllCapabilities()` -- enumerate all registered plugin capabilities

**BuildOrchestrator** (`BuildOrchestrator.cs`)

Coordinates the full compilation pipeline across 6 phases:

```
Phase 1: Init             -- Parse forge.toml configuration
Phase 2: DependencyAnalysis -- Detect cycles, compute build order
Phase 3: Compilation       -- Invoke plugins per module (parallel within layers)
Phase 4: InterfaceExtraction -- Extract public API from artifacts
Phase 5: GlueGeneration    -- Generate cross-language FFI bindings
Phase 6: Linking           -- Produce final executable/library
```

Key types:
- `BuildOptions` -- compile config (profile, output type, incremental, dry-run)
- `BuildResult` -- success/failure with module results, artifacts, duration
- `ModuleBuildResult` -- per-module outcome
- `BuildPhase` enum -- tracks pipeline progress

**IncrementalBuildManager** (`IncrementalBuildManager.cs`)

Hash-based incremental compilation. Checks source file hashes and dependency graphs to skip unchanged modules. Supports transitive invalidation: if module A depends on module B and B is rebuilt, A is also rebuilt.

**LinkerManager** (`LinkerManager.cs`)

Platform-aware linker invocation. Collects object files from all compiled modules and invokes the appropriate system linker. Supports `PlatformLinker` abstraction for Windows (MSVC link.exe), Linux (GNU ld), and macOS (Apple ld).

**InterfaceExtractor** (`InterfaceExtractor.cs`)

Extracts public API from compiled artifacts. Uses `nm` for C/C++ static libraries, reflection for .NET assemblies, and `go doc` for Go packages. Produces `ModuleInterface` objects consumed by `GlueGenerator`.

**GlueGenerator** (`GlueGenerator.cs`)

Generates cross-language FFI binding code. Given a `ModuleInterface` from language A, produces binding source code for language B. Currently supports C-to-C# (P/Invoke), C-to-Rust (extern "C"), and Rust-to-C# (P/Invoke).

**TypeSystem** (`TypeSystem.cs`)

Maps types across languages. Converts between C types (`int`, `char*`), Rust types (`i32`, `*const c_char`), and C# types (`int`, `IntPtr`). Used by `GlueGenerator` for type-correct binding generation.

### 2.3 Rust Core (`maidos-forge-core/`)

High-performance compilation core implemented in Rust. Compiled as a `cdylib` for P/Invoke from C#.

**Modules**:

| Module | File | Purpose |
|:-------|:-----|:--------|
| `parser` | `parser.rs` | Tree-sitter incremental parsing (Rust, C, C++) |
| `checker` | `checker.rs` | Code quality checks (unused vars, unsafe code, buffer overflow) |
| `compiler` | `compiler.rs` | `CompilerCore` trait, `CompileConfig`, `CompileResult` |
| `ffi` | `ffi.rs` | 10 FFI exports (6 feature + 4 memory management) |
| `languages` | `languages/*.rs` | Per-language adapters (C, C++, Rust, Go, Python, JS) |
| `config` | `config.rs` | Configuration parsing |
| `dependency` | `dependency.rs` | Dependency graph analysis |
| `scheduler` | `scheduler.rs` | Build order computation |
| `plugin` | `plugin.rs` | Plugin system types |
| `error` | `error.rs` | Error types (`ForgeError`, `ConfigError`, `PluginError`) |
| `fs` | `fs.rs` | Filesystem operations |

**FFI Export Surface** (`ffi.rs`):

| Function | Purpose |
|:---------|:--------|
| `forge_parse_source(lang, path)` | Parse file, return JSON AST |
| `forge_check_syntax(lang, path)` | Check file, return JSON diagnostics |
| `forge_supported_languages()` | Return JSON array of supported languages |
| `forge_version()` | Return version/capability JSON |
| `forge_parse_batch(lang, paths_json)` | Batch-parse multiple files |
| `forge_parse_incremental(lang, path, prev_hash)` | Incremental parse with hash comparison |
| `forge_last_error()` | Retrieve last error message |
| `forge_free_string(ptr)` | Free Rust-allocated string |
| `forge_init()` | Initialize core (logging) |
| `forge_clear_error()` | Clear last error |

**Error Pattern**: All FFI functions return `*mut c_char` (JSON on success, null on failure). On failure, `forge_last_error()` retrieves the error. All returned strings must be freed with `forge_free_string()`.

### 2.4 Plugin System

**ILanguagePlugin Interface** (5 methods):

```csharp
public interface ILanguagePlugin
{
    PluginCapabilities GetCapabilities();

    Task<(bool Available, string Message)> ValidateToolchainAsync(
        CancellationToken ct = default);

    Task<CompileResult> CompileAsync(
        ValidatedModuleConfig module,
        CompileConfig config,
        CancellationToken ct = default);

    Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default);

    GlueCodeResult GenerateGlue(
        InterfaceDescription sourceInterface,
        string targetLanguage);
}
```

**PluginCapabilities**:

```csharp
public class PluginCapabilities
{
    public string LanguageName { get; init; }
    public string[] SupportedExtensions { get; init; }
    public bool SupportsNativeCompilation { get; init; }
    public bool SupportsCrossCompilation { get; init; }
    public bool SupportsInterfaceExtraction { get; init; }
    public bool SupportsGlueGeneration { get; init; }
    public string[] SupportedTargets { get; init; }
}
```

**Builtin Plugins** (8):

| Plugin | Class | Compiler | Cross-compile |
|:-------|:------|:---------|:------------:|
| C | `CPlugin` | clang / gcc | Yes |
| C++ | `CppPlugin` | clang++ / g++ | Yes |
| C# | `CSharpPlugin` | dotnet | Yes |
| Rust | `RustPlugin` | rustc / cargo | Yes |
| Go | `GoPlugin` | go | Yes |
| Python | `PythonPlugin` | python3 / py_compile | No |
| TypeScript | `TypeScriptPlugin` | tsc / node | No |
| Assembly | `AsmPlugin` | nasm | No |

---

## 3. Data Flow: Full Build Pipeline

```
CLI args
  |
  v
[CommandContext]
  |  parse --verbose, --project
  v
[BuildCommand.Execute(args)]
  |  construct BuildOptions
  v
[BuildOrchestrator.BuildAsync(projectPath, options)]
  |
  |-- Phase 1: ConfigParser.ParseProject(path) -> ValidatedForgeConfig
  |
  |-- Phase 2: DependencyAnalyzer.Analyze(config) -> DependencyGraph
  |             BuildScheduler.CreateSchedule(graph) -> BuildSchedule
  |
  |-- Phase 3: For each layer (parallel within layer):
  |             PluginHost.GetPlugin(module.Language) -> ILanguagePlugin
  |             plugin.CompileAsync(module, compileConfig) -> CompileResult
  |             IncrementalBuildManager.CheckModule() -> skip or rebuild
  |
  |-- Phase 4: InterfaceExtractor.ExtractAsync(artifact) -> ModuleInterface
  |
  |-- Phase 5: GlueGenerator.Generate(interface, targetLang) -> GlueCodeResult
  |
  |-- Phase 6: LinkerManager.LinkAsync(inputs, linkConfig) -> LinkResult
  |
  v
BuildResult { IsSuccess, OutputPath, TotalDuration, ModuleResults, GeneratedGlueFiles }
```

---

## 4. Technology Choices and Rationale

| Decision | Choice | Rationale |
|:---------|:-------|:----------|
| Core language | Rust | Tree-sitter is Rust-native; zero-cost abstractions for hot path; memory safety without GC |
| Orchestration language | C# .NET 8.0 | Rich plugin ecosystem; async/await for I/O-bound compilation; cross-platform via .NET |
| FFI mechanism | P/Invoke (cdylib) | Stable, well-supported, no runtime overhead; Rust `extern "C"` + C# `DllImport` |
| Parser | Tree-sitter | Incremental parsing, sub-millisecond re-parse, wide language support |
| Build format | TOML (`forge.toml`) | Human-readable, widely adopted in Rust/Go ecosystems |
| Error serialization | JSON | Universal, parseable by any tool, structured error reporting |
| Plugin loading | .NET Assembly Load | Dynamic loading without recompilation; familiar to .NET ecosystem |

---

## 5. Cross-Cutting Concerns

### 5.1 Error Handling

- Rust core: `ForgeError` enum with `thiserror` derive. FFI layer captures errors in `LAST_ERROR` mutex.
- C# layer: `CompileResult` / `BuildResult` with `IsSuccess` + `Error` pattern. No exceptions across FFI boundary.
- CLI layer: Non-zero exit code + stderr output for errors.

### 5.2 Logging

- Rust: `tracing` crate with `tracing-subscriber` for structured logging.
- C#: `CommandContext.WriteError()` / `Console.WriteLine()` for verbose output.
- Audit trail: `[MAIDOS-AUDIT]` prefix on significant operations.

### 5.3 Incremental Compilation

- File hash comparison (DefaultHasher in Rust, SHA256 in C#)
- Transitive invalidation through dependency graph
- Cache stored in `.forge/cache/` directory
- `--force-rebuild` flag to bypass cache

---

*MAIDOS-Forge Architecture v3.0 -- CodeQC Gate C Compliant*
