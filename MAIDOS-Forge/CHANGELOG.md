# Changelog

## [0.3.0] - 2026-02-07

### Added
- 97 language plugins (15 Tier A/B fully implemented, 82 Tier C with toolchain validation)
- Cross-language glue code generation (C#, Rust, C interop)
- Tree-sitter based incremental parsing (Rust, C, C++)
- FFI layer for C# P/Invoke integration
- Plugin capability system with `ILanguagePlugin` interface
- Multi-target cross-compilation support (--target flag)
- Standardized ForgeError type (file, line, col, severity, message, lang)

### Fixed
- Go plugin encoding corruption (garbled UTF-8 comments merged with code)
- CSharpConfig missing `OutputType` property
- 15 plugin ProjectReference paths pointing to non-existent Forge.Core
- Removed `#![allow(unused_variables)]` from Rust core
- JavaScript plugin Parameters.Length → Parameters.Count

### Changed
- All Chinese comments translated to English (Rust core + C# Core)
- Cleaned repository (~2.85 GB garbage removed)
- Upgraded spec to v3.0

## [0.2.0] - 2026-01-15

### Added
- Rust core with compiler, parser, checker modules
- C# orchestration layer (Forge.Core.New)
- Initial language adapters (C, C++, Rust, Python, Go, JavaScript)
- FFI bridge between Rust and C#
- 28 unit tests passing

## [0.1.0] - 2025-12-01

### Added
- Initial project structure
- Cargo workspace setup
- Basic compiler trait design
