# MAIDOS-CodeQC -- State Model

> **Version**: 1.0
> **Date**: 2026-02-07
> **Product**: MAIDOS-CodeQC v2.6.1

---

## 1. Scan Lifecycle States

```
  +--------+
  |  Idle  | <----------------------------------+
  +---+----+                                    |
      |  (user invokes scan)                    |
      v                                         |
  +------------------+                          |
  | Loading Config   |                          |
  +--------+---------+                          |
      |  (config valid)     (config invalid)    |
      v                          |              |
  +------------------+     +-----v------+       |
  | Loading Plugins  |     | Error      |-------+
  +--------+---------+     +------------+       |
      |  (plugins ready)   (plugin fail: warn)  |
      v                          |              |
  +------------------+           |              |
  | Discovering Files| <---------+              |
  +--------+---------+                          |
      |  (file manifest ready)                  |
      v                                         |
  +------------------+                          |
  | Scanning         |                          |
  +--------+---------+                          |
      |  (all files processed)                  |
      v                                         |
  +------------------+                          |
  | Evaluating Gates |                          |
  +--------+---------+                          |
      |  (gates evaluated)                      |
      v                                         |
  +------------------+                          |
  | Reporting        |                          |
  +--------+---------+                          |
      |  (report written)                       |
      v                                         |
  +------------------+                          |
  | Complete         |--------------------------+
  +------------------+
```

## 2. State Descriptions

| State | Description | Transitions To |
|:------|:------------|:---------------|
| **Idle** | Engine is initialized but no scan is active | Loading Config |
| **Loading Config** | Reads and validates `.codeqc.toml` | Loading Plugins, Error |
| **Loading Plugins** | Discovers and loads plugin shared libraries | Discovering Files |
| **Discovering Files** | Walks the target directory, builds file manifest | Scanning |
| **Scanning** | Dispatches files to plugins, collects violations | Evaluating Gates |
| **Evaluating Gates** | Aggregates violations, checks gate thresholds | Reporting |
| **Reporting** | Formats and writes output (console/JSON/HTML) | Complete |
| **Complete** | Scan finished; exit code determined | Idle |
| **Error** | Unrecoverable error (invalid config, I/O failure) | Idle |

## 3. Error Handling by State

| State | Error Type | Behavior |
|:------|:-----------|:---------|
| Loading Config | Invalid TOML syntax | Transition to Error with diagnostic |
| Loading Config | Missing config file | Use defaults, continue to Loading Plugins |
| Loading Plugins | Plugin load failure | Log warning, continue without that plugin |
| Discovering Files | Permission denied on directory | Skip directory, log warning |
| Scanning | Plugin panic during analysis | Catch panic, record as tool error, continue |
| Reporting | Write failure (disk full) | Transition to Error |

## 4. Concurrency Model

During the **Scanning** state, files are processed in parallel using a thread pool.
Each file is dispatched to its matching plugin independently. The Rule Engine
collects results through a channel (`crossbeam::channel`) to avoid lock contention.

## 5. Watch Mode (Planned)

In watch mode, the engine cycles between **Idle** and **Scanning** upon file system
change events. The **Loading Config** and **Loading Plugins** states are skipped
after the initial load unless the config or plugin directory changes.

---

*State transitions are logged at DEBUG level for troubleshooting.*
