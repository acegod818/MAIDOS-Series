# MAIDOS-CodeQC -- Plugin API Contract

> **Version**: 1.0
> **Date**: 2026-02-07
> **Product**: MAIDOS-CodeQC v2.6.1

---

## 1. Plugin Trait Interface

Every plugin must implement the `CodeQcPlugin` trait and export a C-ABI entry point.

```rust
/// The core trait that every plugin must implement.
pub trait CodeQcPlugin: Send + Sync {
    /// Returns the plugin metadata.
    fn info(&self) -> PluginInfo;

    /// Returns the list of file extensions this plugin handles.
    fn supported_extensions(&self) -> Vec<String>;

    /// Analyzes a single file and returns violations found.
    fn analyze(&self, ctx: &AnalysisContext) -> Vec<Violation>;

    /// Called once when the plugin is loaded. Optional initialization.
    fn on_load(&mut self) -> Result<(), PluginError> { Ok(()) }

    /// Called before the plugin is unloaded. Optional cleanup.
    fn on_unload(&mut self) -> Result<(), PluginError> { Ok(()) }
}
```

## 2. Entry Point Symbol

Each plugin shared library must export:

```rust
#[no_mangle]
pub extern "C" fn codeqc_plugin_entry() -> Box<dyn CodeQcPlugin>
```

The core engine resolves this symbol via `libloading` to instantiate the plugin.

## 3. Data Structures

```rust
pub struct PluginInfo {
    pub name: String,          // e.g., "codeqc-plugin-web"
    pub version: String,       // semver, e.g., "2.6.1"
    pub api_version: u32,      // must match core's expected API version
    pub description: String,
}

pub struct AnalysisContext {
    pub file_path: PathBuf,
    pub file_content: String,
    pub config: RuleConfig,    // merged default + project config
}

pub struct Violation {
    pub rule_id: String,       // e.g., "WEB-001"
    pub gate: Gate,            // G1, G2, G3, or G4
    pub severity: Severity,    // Error, Warning, Info
    pub file_path: PathBuf,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub suggestion: Option<String>,
}

pub enum Gate { G1, G2, G3, G4 }
pub enum Severity { Error, Warning, Info }
```

## 4. Rule Definition Format

Rules are defined in TOML files shipped with each plugin:

```toml
[[rule]]
id = "WEB-001"
gate = "G1"
severity = "error"
description = "Detects empty catch blocks in JavaScript/TypeScript"
pattern = "catch\\s*\\([^)]*\\)\\s*\\{\\s*\\}"
languages = ["javascript", "typescript"]
```

## 5. Message Protocol

Communication between core and plugin is via direct function calls (in-process FFI).
There is no IPC or serialization overhead. The `analyze` method receives an owned
`AnalysisContext` and returns an owned `Vec<Violation>`.

## 6. Versioning Contract

| API Version | Core Version | Compatibility |
|:------------|:-------------|:--------------|
| 1 | v2.0 - v2.6.1 | Current stable |
| 2 | v3.0+ | Planned; will add async analyze |

Plugins with a mismatched `api_version` are rejected at load time with a clear error.

---

*Plugin authors: see `examples/plugin-template/` for a minimal working plugin.*
