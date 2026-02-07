# MAIDOS-Forge Non-Functional Requirements

> **Version**: v3.0
> **Date**: 2026-02-07
> **Source**: SPEC-MAIDOS-Forge-v3.0.md, Section 6
> **CodeQC**: v3.0 Gate C Compliance

---

## NFR Summary Table

| NFR ID | Category | Requirement | Target | Measurement |
|:-------|:---------|:------------|:-------|:------------|
| NFR-001 | Latency | Rust core hot path | <10ms | Benchmark test |
| NFR-002 | Overhead | Single-file compile overhead | <500ms above native | Timed test |
| NFR-003 | Memory | Forge resident memory | <100MB | Memory profiling |
| NFR-004 | Coverage | Test coverage | Rust >=70%, C# >=60% | Coverage reports |
| NFR-005 | Build | Zero errors, zero warnings | 0e/0w | CI pipeline |
| NFR-006 | Security | No untrusted code execution | System compilers only | Code audit |

---

## NFR-001: Rust Core Path Latency

**Requirement**: The Rust core parsing and checking path must complete in under 10 milliseconds for a single file.

**Scope**: This covers the `forge_parse_source` and `forge_check_syntax` FFI functions exposed from `maidos-forge-core`. The measurement starts when the FFI function is called and ends when it returns. File I/O is included in the measurement.

**Rationale**: The Rust core is the hot path that every compilation request passes through. Tree-sitter incremental parsing is designed for sub-millisecond performance on typical source files. The 10ms budget includes file read, parse, and tree construction.

**Measurement Method**:
- Benchmark tests using `criterion` in `maidos-forge-core`
- Measured on a standard developer machine (4-core, 16GB RAM)
- Test files: typical source files under 1000 lines
- `ParseResult.duration_ms` field records actual elapsed time

**Verification**: `cargo bench` produces timing reports. All p95 values must be under 10ms.

---

## NFR-002: Single-File Compile Overhead

**Requirement**: For a single-file compilation, Forge must add no more than 500 milliseconds of overhead above the native compiler's own execution time.

**Scope**: The overhead is measured as: `forge build time - native compiler time` for the same source file and equivalent flags.

**Rationale**: Users will not accept a tool that significantly slows down their compilation. The 500ms budget covers: CLI argument parsing, plugin lookup, toolchain validation, process spawning, and error format normalization.

**Measurement Method**:
- Timed comparison: `forge build c hello.c` vs `clang hello.c`
- Both run on the same machine, same file, same optimization level
- Repeated 10 times, median taken
- `BuildResult.TotalDuration` compared to raw compiler invocation

**Breakdown Budget**:

| Component | Budget |
|:----------|:-------|
| CLI argument parsing | <10ms |
| Plugin lookup | <5ms |
| Toolchain validation (cached) | <20ms |
| Process spawn + wait overhead | <200ms |
| Error normalization | <50ms |
| Result serialization | <15ms |
| Margin | 200ms |
| **Total** | **<500ms** |

---

## NFR-003: Forge Resident Memory

**Requirement**: The Forge process must consume no more than 100MB of resident memory during a standard compilation workflow.

**Scope**: Measured as peak RSS (Resident Set Size) during a typical multi-module build with up to 10 modules.

**Rationale**: Forge should be lightweight enough to run alongside other development tools without competing for memory. The 100MB budget covers the .NET runtime, loaded plugins, Rust core (via P/Invoke), and build caches.

**Measurement Method**:
- Memory profiling using `dotnet-counters` or process monitor
- Peak RSS during: `forge build` on a 5-module project with C, Rust, and Go modules
- Measured after GC stabilization (steady state)

**Breakdown Budget**:

| Component | Budget |
|:----------|:-------|
| .NET runtime baseline | ~30MB |
| Loaded plugins (8 builtin) | ~10MB |
| Rust core cdylib | ~5MB |
| Tree-sitter parser state | ~10MB |
| Build cache / incremental state | ~20MB |
| Margin | 25MB |
| **Total** | **<100MB** |

---

## NFR-004: Test Coverage

**Requirement**: Test coverage must meet or exceed the following thresholds:
- Rust (`maidos-forge-core`): **>= 70%** line coverage
- C# (`Forge.Core.New`, `Forge.Cli`): **>= 60%** line coverage

**Scope**: Coverage measured on production code only. Test files, generated code, and FFI glue stubs are excluded.

**Rationale**: The Rust core handles parsing and checking -- critical paths that must be well-tested. The C# layer coordinates compilation, where integration tests provide more value than unit tests, hence the lower threshold.

**Measurement Method**:
- Rust: `cargo tarpaulin --out html` targeting `maidos-forge-core/src/`
- C#: `dotnet test --collect:"XPlat Code Coverage"` with `coverlet`
- Both produce reports stored in `evidence/coverage/`

**Key Test Files** (Rust):

| Module | Test Location | Focus |
|:-------|:-------------|:------|
| parser | `maidos-forge-core/src/parser.rs` | Tree-sitter parse correctness |
| checker | `maidos-forge-core/src/checker.rs` | Diagnostic detection |
| ffi | `maidos-forge-core/src/ffi.rs` | FFI null safety, round-trip |
| compiler | `maidos-forge-core/src/compiler.rs` | CompileConfig validation |
| languages | `maidos-forge-core/src/languages/*.rs` | Per-language adapter tests |

---

## NFR-005: Zero Errors, Zero Warnings

**Requirement**: Both `cargo build` and `dotnet build` must produce **0 errors and 0 warnings** on the default configuration.

**Scope**: Applies to all production code in the workspace. Warnings suppressed via explicit `#[allow(...)]` or `<NoWarn>` must be documented with rationale.

**Rationale**: Warnings are technical debt indicators. A clean build baseline ensures new warnings are immediately visible and actionable.

**Measurement Method**:
- `cargo build 2>&1 | grep -c "warning"` must return 0
- `dotnet build 2>&1 | grep -c "warning"` must return 0
- CI pipeline enforces `--warnaserror` on release builds

**Allowed Suppressions**:

| Suppression | Location | Rationale |
|:------------|:---------|:----------|
| `#![allow(clippy::not_unsafe_ptr_arg_deref)]` | FFI functions | FFI extern "C" functions accept raw pointers by design |
| `#![allow(dead_code)]` | `ffi.rs` | FFI exports are consumed externally, not within the Rust crate |

---

## NFR-006: Security -- System Compilers Only

**Requirement**: Forge must never execute untrusted code. It may only invoke system-installed compilers and well-known tools (clang, gcc, rustc, go, dotnet, javac, node, ar, nm, pnputil).

**Scope**: All code paths that spawn external processes, including plugin-invoked compilation.

**Rationale**: Forge orchestrates compilation by delegating to native compilers. It must not download and execute arbitrary binaries, eval user-provided strings, or run code from untrusted sources.

**Controls**:

| Control | Implementation |
|:--------|:---------------|
| Compiler whitelist | `ProcessRunner` only invokes known compiler executables |
| No `eval` / `exec` | No dynamic code evaluation in Rust or C# layers |
| No network downloads | Forge does not download compilers; `forge toolchain install` is user-initiated |
| Plugin sandboxing | Plugins run in the same process but are loaded from the local `plugins/` directory only |
| Path validation | All file paths are validated and canonicalized before use |

**Verification**: Code audit of `ProcessRunner.RunAsync()`, `CPlugin.CompileAsync()`, and all `ILanguagePlugin.CompileAsync()` implementations. Grep for `Process.Start`, `System.Diagnostics.Process`, and `std::process::Command` to verify all invocations are on the whitelist.

---

*MAIDOS-Forge NFR v3.0 -- CodeQC Gate C Compliant*
