# MAIDOS-Forge User Journeys

> **Version**: v3.0
> **Date**: 2026-02-07
> **CodeQC**: v3.0 Gate C Compliance

---

## J-001: Developer Compiles a C Project

**Actor**: Cross-language developer
**Precondition**: GCC or Clang is installed and on PATH
**Trigger**: Developer has a C source file and wants to compile it

**Steps**:

1. Developer runs: `forge build c hello.c`
2. Forge CLI routes to `BuildCommand`, which calls `BuildOrchestrator.BuildAsync()`
3. `PluginHost.GetPlugin("c")` returns the `CPlugin` instance
4. `CPlugin.ValidateToolchainAsync()` checks for clang first, falls back to gcc
5. `CPlugin.CompileAsync()` invokes `clang -c hello.c -o hello.o -O2 -std=c17 -Wall -Wextra -fPIC`
6. On success, `ar rcs libhello.a hello.o` creates a static library
7. `LinkerManager.LinkAsync()` produces the final executable
8. CLI outputs: `Build succeeded: ./build/release/hello (142ms)`

**Expected Output**:
```
$ forge build c hello.c
[C] Using: clang 17.0.6
[C] Found 1 source file(s)
Build succeeded: ./build/release/hello (142ms)
```

**Error Path**: If neither clang nor gcc is found, Forge outputs:
```
Error: Neither clang nor gcc found
Hint: Install clang via 'apt install clang' or 'brew install llvm'
```

**Maps to**: AC-001 (FR-001)

---

## J-002: Developer Checks Toolchain Availability

**Actor**: Developer setting up a new machine
**Precondition**: None (checking what is available)
**Trigger**: Developer wants to know which compilers are installed

**Steps**:

1. Developer runs: `forge check go`
2. Forge CLI routes to `CheckCommand`
3. `PluginHost.GetPlugin("go")` returns the `GoPlugin`
4. `GoPlugin.ValidateToolchainAsync()` runs `go version` and captures output
5. CLI outputs the result as JSON

**Expected Output**:
```json
{
  "found": true,
  "version": "1.21.5",
  "path": "/usr/local/go/bin/go"
}
```

**Alternative -- Multiple toolchains**:
```
$ forge check c
Found 2 toolchains for C:
  1. clang 17.0.6  /usr/bin/clang
  2. gcc 13.2.0    /usr/bin/gcc
Active: clang
```

**Maps to**: AC-004, AC-005 (FR-002)

---

## J-003: Developer Gets Unified Error Output

**Actor**: Developer debugging a compilation error
**Precondition**: rustc is installed
**Trigger**: Developer compiles a Rust file with a type error

**Steps**:

1. Developer runs: `forge build rust bad.rs`
2. Forge CLI routes to `BuildCommand`
3. `RustPlugin.CompileAsync()` invokes `rustc bad.rs`
4. rustc exits with non-zero status and stderr output
5. `ErrorNormalizer` parses rustc's error output
6. Forge outputs a standardized `ForgeError` JSON object

**Expected Output**:
```json
{
  "file": "bad.rs",
  "line": 7,
  "column": 13,
  "severity": "error",
  "message": "mismatched types: expected `i32`, found `&str`",
  "lang": "rust",
  "code": "E0308"
}
```

**Key Property**: The JSON format is identical regardless of whether the underlying compiler is clang, rustc, go, or javac. The fields `file`, `line`, `column`, `severity`, `message`, and `lang` are always present.

**Maps to**: AC-006, AC-007 (FR-003)

---

## J-004: Developer Cross-Compiles for Linux

**Actor**: Windows developer targeting Linux deployment
**Precondition**: Cross-compilation toolchain is installed (e.g., `x86_64-unknown-linux-gnu` for GCC or Zig CC)
**Trigger**: Developer needs to produce a Linux binary from a Windows host

**Steps**:

1. Developer runs: `forge build c hello.c --target linux-x64`
2. Forge CLI passes `--target linux-x64` to `BuildOrchestrator`
3. `CrossCompiler` resolves the target triple to `x86_64-unknown-linux-gnu`
4. `CPlugin.CompileAsync()` invokes the cross-compiler with appropriate sysroot flags
5. Linker produces an ELF binary

**Expected Output**:
```
$ forge build c hello.c --target linux-x64
[C] Cross-compiling for linux-x64
[C] Using: x86_64-unknown-linux-gnu-gcc 13.2.0
Build succeeded: ./build/release/hello (ELF 64-bit, 287ms)
```

**Supported Targets** (Tier A only):

| Target | Triple | Artifact |
|:-------|:-------|:---------|
| `linux-x64` | `x86_64-unknown-linux-gnu` | ELF 64-bit |
| `linux-arm64` | `aarch64-unknown-linux-gnu` | ELF aarch64 |
| `windows-x64` | `x86_64-pc-windows-msvc` | PE32+ |
| `macos-x64` | `x86_64-apple-darwin` | Mach-O |
| `macos-arm64` | `aarch64-apple-darwin` | Mach-O |

**Maps to**: AC-008, AC-009 (FR-004)

---

## J-005: Developer Extracts Interface from Compiled Artifact

**Actor**: Developer building cross-language bindings
**Precondition**: A compiled static library exists
**Trigger**: Developer needs to know the public API of a C library for FFI binding

**Steps**:

1. Developer runs: `forge interface libmath.a`
2. Forge CLI routes to `FfiCommand`
3. `CPlugin.ExtractInterfaceAsync()` invokes `nm -g --defined-only libmath.a`
4. Symbol output is parsed: T-type symbols (text section) are extracted
5. System symbols (prefixed `__`, `_init`, `_fini`, etc.) are filtered out
6. Forge outputs a JSON interface description

**Expected Output**:
```json
{
  "version": "1.0",
  "module": { "name": "math", "version": "1.0.0" },
  "language": { "name": "c", "abi": "c" },
  "exports": [
    { "name": "add", "returnType": "i32", "parameters": [
      { "name": "a", "type": "i32" },
      { "name": "b", "type": "i32" }
    ]},
    { "name": "multiply", "returnType": "i32", "parameters": [
      { "name": "a", "type": "i32" },
      { "name": "b", "type": "i32" }
    ]},
    { "name": "factorial", "returnType": "i64", "parameters": [
      { "name": "n", "type": "i32" }
    ]}
  ]
}
```

**Maps to**: AC-010 (FR-005)

---

## J-006: Developer Uses a Custom Plugin

**Actor**: Plugin author / team with a niche language
**Precondition**: Developer has built a DLL implementing `ILanguagePlugin`
**Trigger**: Developer wants Forge to support a custom language

**Steps**:

1. Developer creates a .NET class library implementing `ILanguagePlugin`
2. The DLL implements all 5 interface methods:
   - `GetCapabilities()` returns language name, extensions, features
   - `ValidateToolchainAsync()` checks for the language's compiler
   - `CompileAsync()` invokes the compiler
   - `ExtractInterfaceAsync()` extracts public API
   - `GenerateGlue()` generates FFI bindings
3. Developer places the DLL in `plugins/` directory
4. On startup, `PluginHost.RegisterBuiltinPlugins()` loads builtin plugins, then scans `plugins/`
5. Developer runs: `forge build custom-lang source.cl`
6. Forge routes to the custom plugin and compiles

**Expected Output**:
```
$ forge plugin list
Installed plugins:
  c        (builtin)   clang/gcc
  cpp      (builtin)   clang++/g++
  csharp   (builtin)   dotnet
  rust     (builtin)   rustc
  go       (builtin)   go
  python   (builtin)   python3
  typescript (builtin) tsc/node
  asm      (builtin)   nasm
  custom-lang (plugin) plugins/CustomLang.dll

$ forge build custom-lang source.cl
[custom-lang] Using: custom-compiler 2.0.1
Build succeeded: ./build/release/output (95ms)
```

**Maps to**: AC-011 (FR-006)

---

## J-007: Developer Initializes a New Project

**Actor**: Developer starting a new cross-language project
**Precondition**: Forge is installed
**Trigger**: Developer wants to scaffold a new Forge project

**Steps**:

1. Developer runs: `forge init myproject`
2. `InitCommand` creates the project directory and scaffold:
   - `myproject/forge.toml` -- project configuration
   - `myproject/src/` -- source directory
   - `myproject/build/` -- output directory (gitignored)
3. `forge.toml` contains default configuration:

```toml
[project]
name = "myproject"
version = "0.1.0"

[output]
dir = "build"

[[modules]]
name = "main"
language = "c"
path = "src"
```

**Expected Output**:
```
$ forge init myproject
Created project 'myproject' with default configuration
  myproject/
    forge.toml
    src/
    build/

Next steps:
  cd myproject
  forge build
```

**Maps to**: FR-007 (`forge init` command)

---

## Journey Coverage Matrix

| Journey | FR | AC | Tier | Priority |
|:--------|:---|:---|:-----|:--------:|
| J-001 | FR-001 | AC-001 | A | P0 |
| J-002 | FR-002 | AC-004, AC-005 | A/B | P0 |
| J-003 | FR-003 | AC-006, AC-007 | A/B | P0 |
| J-004 | FR-004 | AC-008, AC-009 | A | P1 |
| J-005 | FR-005 | AC-010 | A | P1 |
| J-006 | FR-006 | AC-011 | C | P1 |
| J-007 | FR-007 | -- | -- | P1 |

---

*MAIDOS-Forge User Journeys v3.0 -- CodeQC Gate C Compliant*
