# Changelog

## [0.3.0] - 2026-02-06

### Added
- 99 language plugins (Tier A/B fully implemented, Tier C skeleton)
- Cross-language glue code generation (C#, Rust, C interop)
- Tree-sitter based incremental parsing (Rust, C, C++)
- FFI layer for C# P/Invoke integration
- Plugin capability system with `ILanguagePlugin` interface
- Multi-target cross-compilation support

### Fixed
- Go plugin encoding corruption (garbled UTF-8 comments merged with code)
- CSharpConfig missing `OutputType` property
- 15 plugin ProjectReference paths pointing to non-existent Forge.Core
- Removed `#![allow(unused_variables)]` from Rust core

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
