# MAIDOS-CodeQC -- Architecture Document

> **Version**: 1.0
> **Date**: 2026-02-07
> **Product**: MAIDOS-CodeQC v2.6.1

---

## 1. High-Level Overview

```
 CLI Interface
      |
      v
 +------------------+
 |   Core Engine     |   maidos-codeqc (Rust binary)
 |  +-------------+  |
 |  | File Walker  |  |   Discovers and reads source files
 |  +-------------+  |
 |  | Rule Engine  |  |   Applies rules, evaluates gates
 |  +-------------+  |
 |  | Reporter     |  |   Formats output (console/JSON/HTML)
 |  +-------------+  |
 |  | Plugin Mgr   |  |   Loads, manages, routes to plugins
 |  +------+------+  |
 +---------|----------+
           |  (FFI: cdylib shared libraries)
   +-------+-------+-------+-------+
   |       |       |       |       |
 config  data   dotnet  web    ... (10 plugins)
```

## 2. Core Engine Components

| Component | Responsibility |
|:----------|:---------------|
| **File Walker** | Recursively discovers files, applies ignore rules (.codeqcignore) |
| **Rule Engine** | Loads rule definitions, executes rule functions, collects violations |
| **Gate Evaluator** | Aggregates violations per gate (G1-G4), determines pass/fail |
| **Reporter** | Serializes results into console, JSON, or HTML format |
| **Plugin Manager** | Discovers, loads, and dispatches work to plugins via FFI |
| **Config Loader** | Reads `.codeqc.toml`, validates against schema, merges defaults |

## 3. Plugin Architecture

Each plugin is a Rust cdylib (`.dll` / `.so` / `.dylib`) that implements the
`CodeQcPlugin` trait. The Plugin Manager uses `libloading` to dynamically load
plugins at startup or on demand (hot-reload).

**Plugin Lifecycle**:
1. **Discovery** -- Plugin Manager scans the plugin directory for shared libraries.
2. **Loading** -- Each library is loaded; the `codeqc_plugin_entry` symbol is resolved.
3. **Registration** -- Plugin reports its name, version, and supported file extensions.
4. **Dispatch** -- During a scan, files are routed to matching plugins.
5. **Execution** -- Plugin analyzes the file and returns a list of violations.
6. **Unload** -- On shutdown or hot-reload, the plugin is safely unloaded.

## 4. Rule Processing Pipeline

```
Source File
    |
    v
[Parse] --> AST / Token Stream
    |
    v
[Match] --> Apply rule patterns to AST nodes
    |
    v
[Evaluate] --> Determine severity (error / warning / info)
    |
    v
[Aggregate] --> Group violations by gate level
    |
    v
[Report] --> Output formatted results
```

## 5. Gate System

| Gate | Name | Purpose |
|:-----|:-----|:--------|
| G1 | Hygiene | No fake implementations, no TODO residue, no empty handlers |
| G2 | Correctness | Type safety, null checks, error handling patterns |
| G3 | Maintainability | Complexity limits, naming conventions, documentation coverage |
| G4 | Release | Full compliance: G1+G2+G3 plus deployment readiness checks |

## 6. Data Flow

1. CLI parses arguments and loads configuration.
2. File Walker produces a file manifest.
3. Plugin Manager routes each file to its matching plugin(s).
4. Each plugin returns `Vec<Violation>` for the file.
5. Rule Engine merges all violations and evaluates gate thresholds.
6. Reporter outputs the final result.

---

*This architecture supports the NFRs defined in NFR.md, particularly plugin isolation,
performance targets, and cross-platform portability.*
