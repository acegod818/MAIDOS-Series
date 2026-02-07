# MAIDOS-Forge Acceptance Criteria Matrix

> **Version**: v3.0
> **Date**: 2026-02-07
> **Source**: SPEC-MAIDOS-Forge-v3.0.md, Sections 5 and 8
> **CodeQC**: v3.0 Gate C Compliance

---

## AC-to-Test Evidence Mapping

| AC ID | FR | Description | Test Type | Test File | Status |
|:------|:---|:------------|:----------|:----------|:-------|
| AC-001 | FR-001 | `forge build c hello.c` produces runnable executable when GCC is present | Integration | `maidos-forge-core/src/languages/c.rs` (adapter test) + `src/Forge.Tests/` (E2E) | Pending |
| AC-002 | FR-001 | `forge build rust main.rs` produces binary when rustc is present | Integration | `maidos-forge-core/src/languages/rust.rs` (adapter test) + `src/Forge.Tests/` (E2E) | Pending |
| AC-003 | FR-001 | `forge build swift` reports "toolchain not found: swift" with install hints when Swift compiler is absent | Unit | `src/Forge.Tests/` (mock toolchain) | Pending |
| AC-004 | FR-002 | `forge check go` returns `{found:true, version:"1.21.x", path:"/usr/local/go/bin/go"}` when Go is installed | Unit | `src/Forge.Core.New/GoPlugin.cs` (ValidateToolchainAsync) + `src/Forge.Tests/` | Pending |
| AC-005 | FR-002 | `forge check c` lists both clang and gcc when both are available | Unit | `src/Forge.Core.New/CPlugin.cs` (ValidateToolchainAsync) + `src/Forge.Tests/` | Pending |
| AC-006 | FR-003 | C syntax error produces ForgeError JSON with `file`, `line`, `severity:error`, `message` | Unit | `maidos-forge-core/src/checker.rs` (test_c_checker) + `maidos-forge-core/src/ffi.rs` (test_ffi_check_syntax) | Pending |
| AC-007 | FR-003 | Rust type error produces ForgeError JSON in same format as AC-006 | Unit | `maidos-forge-core/src/checker.rs` (test_rust_checker) + `maidos-forge-core/src/ffi.rs` (test_ffi_parse_rust_source) | Pending |
| AC-008 | FR-004 | `forge build c hello.c --target linux-x64` produces ELF binary on Windows host with cross-compile toolchain | Integration | `src/Forge.Tests/` (cross-compile E2E, requires toolchain) | Pending |
| AC-009 | FR-004 | `forge build go main.go --target linux-arm64` produces ARM64 Linux binary | Integration | `src/Forge.Tests/` (cross-compile E2E, requires toolchain) | Pending |
| AC-010 | FR-005 | `forge interface hello.h` returns JSON with 3 function signatures from header with 3 public functions | Unit | `src/Forge.Core.New/CPlugin.cs` (ExtractInterfaceAsync) + `src/Forge.Core.New/InterfaceExtractor.cs` | Pending |
| AC-011 | FR-006 | Custom plugin DLL placed in `plugins/` enables `forge build <custom-lang>` to compile | Integration | `src/Forge.Tests/` (plugin loading E2E) | Pending |

---

## Detailed AC Specifications

### AC-001: C Single-File Compilation

**Given**: System has GCC or Clang installed and on PATH
**When**: User runs `forge build c hello.c`
**Then**: An executable `./hello` is produced and runs correctly

**Test Evidence**:
- **Rust-side**: `maidos-forge-core/src/languages/c.rs` -- tests the C language adapter can invoke compilation
- **C#-side**: `src/Forge.Core.New/CPlugin.cs` -- `CompileAsync()` invokes clang/gcc with `-c`, `ar rcs` for library
- **E2E**: `src/Forge.Tests/` -- full pipeline from CLI to binary output
- **FFI**: `maidos-forge-core/src/ffi.rs::test_ffi_parse_c_source` -- verifies C source parsing through FFI

### AC-002: Rust Single-File Compilation

**Given**: System has rustc installed
**When**: User runs `forge build rust main.rs`
**Then**: A binary executable is produced

**Test Evidence**:
- **Rust-side**: `maidos-forge-core/src/languages/rust.rs` -- Rust language adapter test
- **FFI**: `maidos-forge-core/src/ffi.rs::test_ffi_parse_rust_source` -- verifies Rust parsing
- **Parser**: `maidos-forge-core/src/parser.rs::test_rust_parser` -- tree-sitter parse correctness

### AC-003: Missing Toolchain Error

**Given**: System does NOT have Swift compiler installed
**When**: User runs `forge build swift main.swift`
**Then**: Error message contains "toolchain not found: swift" and installation suggestions

**Test Evidence**:
- **FFI**: `maidos-forge-core/src/ffi.rs::test_ffi_unsupported_language` -- unsupported language returns null + error
- **C#-side**: `PluginHost.GetPlugin("swift")` returns null, command reports error

### AC-004: Go Toolchain Detection

**Given**: System has Go 1.21+ installed
**When**: User runs `forge check go`
**Then**: Returns `{found: true, version: "1.21.x", path: "/usr/local/go/bin/go"}`

**Test Evidence**:
- **C#-side**: `src/Forge.Core.New/GoPlugin.cs` -- `ValidateToolchainAsync()` calls `go version`
- **Unit**: Validates JSON output format

### AC-005: Multiple C Toolchains

**Given**: System has both Clang and GCC installed
**When**: User runs `forge check c`
**Then**: Both toolchains are listed with version and path

**Test Evidence**:
- **C#-side**: `src/Forge.Core.New/CPlugin.cs` -- `ValidateToolchainAsync()` probes clang then gcc
- **Unit**: Validates multi-toolchain output

### AC-006: C Error Standardization

**Given**: C source file `broken.c` has a syntax error
**When**: User runs `forge build c broken.c`
**Then**: Output contains ForgeError JSON with `file`, `line`, `severity: "error"`, `message`

**Test Evidence**:
- **Rust-side**: `maidos-forge-core/src/checker.rs::test_c_checker` -- CChecker produces Diagnostic objects
- **FFI**: `maidos-forge-core/src/ffi.rs::test_ffi_check_syntax` -- verifies JSON format through FFI
- **Types**: `checker.rs::Diagnostic` struct has `kind`, `code`, `message`, `location` (line, column, file)

### AC-007: Rust Error Standardization

**Given**: Rust source file `bad.rs` has a type error
**When**: User runs `forge build rust bad.rs`
**Then**: Output contains ForgeError JSON in the same format as AC-006

**Test Evidence**:
- **Rust-side**: `maidos-forge-core/src/checker.rs::test_rust_checker` -- RustChecker detects unused variables
- **FFI**: `maidos-forge-core/src/ffi.rs::test_ffi_parse_rust_source` -- Rust source parsing through FFI

### AC-008: C Cross-Compilation to Linux

**Given**: Windows host has cross-compilation toolchain installed
**When**: User runs `forge build c hello.c --target linux-x64`
**Then**: Produces an ELF 64-bit binary

**Test Evidence**:
- **C#-side**: `src/Forge.Core.New/CPlugin.cs` -- `CompileAsync()` with cross-compile target
- **Integration**: Requires actual cross-toolchain; CI environment test

### AC-009: Go Cross-Compilation to ARM64

**Given**: Go source file `main.go`
**When**: User runs `forge build go main.go --target linux-arm64`
**Then**: Produces ARM64 Linux binary (GOARCH=arm64, GOOS=linux)

**Test Evidence**:
- **C#-side**: `src/Forge.Core.New/GoPlugin.cs` -- sets GOOS/GOARCH environment variables
- **Integration**: Requires Go installation; CI environment test

### AC-010: Interface Extraction

**Given**: C header file with 3 public functions
**When**: User runs `forge interface hello.h`
**Then**: JSON output contains 3 function signatures

**Test Evidence**:
- **C#-side**: `src/Forge.Core.New/CPlugin.cs::ExtractInterfaceAsync()` -- invokes `nm`, parses T-type symbols
- **C#-side**: `src/Forge.Core.New/InterfaceExtractor.cs` -- `ExtractAsync()` orchestration

### AC-011: Custom Plugin Loading

**Given**: A custom DLL implementing `ILanguagePlugin` is placed in `plugins/`
**When**: User runs `forge build <custom-lang> source.ext`
**Then**: Forge loads the plugin and compilation succeeds

**Test Evidence**:
- **C#-side**: `src/Forge.Core.New/PluginHost.cs` -- `RegisterPlugin()` accepts any `ILanguagePlugin`
- **Integration**: Plugin load + compile test in `src/Forge.Tests/`

---

## FR-007 CLI Commands Verification

| Command | Expected Exit Code | Verification |
|:--------|:-------------------|:-------------|
| `forge build` | 0 on success, 1 on failure | E2E test |
| `forge check` | 0 always | E2E test |
| `forge clean` | 0 on success | E2E test |
| `forge init myproject` | 0, creates scaffold | E2E test |
| `forge watch` | 0 (runs until interrupted) | Manual test |
| `forge graph` | 0, outputs graph | E2E test |
| `forge toolchain` | 0, lists toolchains | E2E test |
| `forge plugin list` | 0, lists plugins | E2E test |

---

## NFR Verification

| NFR | Target | Evidence Type | Tool |
|:----|:-------|:-------------|:-----|
| NFR-001 | <10ms parser latency | Benchmark | `cargo bench` (criterion) |
| NFR-002 | <500ms overhead | Timed test | Stopwatch comparison |
| NFR-003 | <100MB resident | Profiling | `dotnet-counters` |
| NFR-004 | Rust >=70%, C# >=60% | Coverage | `cargo tarpaulin`, `coverlet` |
| NFR-005 | 0 error, 0 warning | CI build | `cargo build`, `dotnet build` |
| NFR-006 | System compilers only | Audit | Code review of ProcessRunner |

---

*MAIDOS-Forge AC Matrix v3.0 -- CodeQC Gate C Compliant*
