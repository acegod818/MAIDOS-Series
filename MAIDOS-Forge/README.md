# MAIDOS-Forge

Cross-language compilation framework with 97 language plugins and multi-platform support.

## Architecture

```
C# WPF Studio (UI)
    |
C# Forge.Cli (CLI)
    |
C# Forge.Core.New (Orchestrator)
    |                       \
Rust maidos-forge-core       C# Forge.Plugins (97 languages)
(Tree-sitter parser,         (ILanguagePlugin interface)
 FFI, Compiler core)
```

**Rust Core** provides parsing (Tree-sitter), syntax checking, and FFI exports.
**C# Core** provides plugin orchestration, module config, and compilation pipeline.
**C# Plugins** implement `ILanguagePlugin` for each supported language.

## Supported Languages

### Tier A (Standard - fully implemented)
C, C++, C#, Rust, Go

### Tier B (Standard - fully implemented)
Python, JavaScript, TypeScript, Java, Kotlin, Swift, Ruby, Dart, Haskell, Elixir

### Tier C (Extension modules - skeleton)
Ada, Agda, Assembly, Awk, BashScript, Cairo, Carbon, Chisel, Clojure, COBOL, Coq,
Crystal, Cue, D, Dafny, Datalog, Delphi, Dhall, Erlang, F#, Factor, Faust, Forth,
Fortran, Gleam, GraphQL, Groovy, Idris, Julia, LaTeX, Lean, Lisp, Lua, Markdown,
MATLAB, MiniZinc, Mojo, MoonBit, Move, Nim, Nushell, Objective-C, OCaml, Odin,
Pascal, Perl, PHP, Pkl, Pony, PowerShell, Prolog, R, Roc, Scala, Sed, Smalltalk,
Solidity, SQL, SystemVerilog, Tcl, Typst, Unison, V, Vale, Verilog, VHDL, Vyper,
WebAssembly, WGSL, Wolfram, Zig

### Data/Schema formats
AsciiDoc, AsyncAPI, CapnProto, FlatBuffers, GLSL, HLSL, Jsonnet, OpenAPI, ProtoBuf,
Thrift, TLA+

## Build

### Prerequisites
- Rust toolchain (1.70+)
- .NET 8.0 SDK
- Optional: GCC/G++, Go, Python, Node.js (for language-specific plugins)

### Rust Core
```bash
cargo build --release
cargo test
cargo clippy
```

### C# Core + CLI
```bash
dotnet build src/Forge.Core.New/
dotnet build src/Forge.Cli/
```

### Plugins (Tier A example)
```bash
dotnet build src/Forge.Plugins/Forge.Plugin.Rust/
```

## Project Structure

```
MAIDOS-Forge/
  Cargo.toml              # Rust workspace
  maidos-forge-core/      # Rust core library (parser, FFI, compiler)
  maidos-forge-cli/       # Rust CLI binary
  src/
    Forge.Core.New/       # C# core orchestrator
    Forge.Cli/            # C# CLI
    Forge.Plugins/        # 97 language plugins
    Forge.Studio/         # WPF Studio (experimental)
    Forge.Tests/          # Unit tests
    Forge.VSCode/         # VS Code extension
  templates/              # Project templates
  scripts/                # Build/deploy scripts
  extensions/             # Extension system
  samples/                # Sample projects
  demo-project/           # Demo project
  tests/                  # Integration tests
  SPEC-MAIDOS-Forge-v3.0.md  # Internal specification (Chinese)
  LICENSE
```

## License

See [LICENSE](LICENSE) for details.

Part of the [MAIDOS](https://github.com/nicemid/MAIDOS-Series) product family.
