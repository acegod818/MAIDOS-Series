# MAIDOS-Forge State Model

> **Version**: v3.0
> **Date**: 2026-02-07
> **CodeQC**: v3.0 Gate C Compliance

---

## 1. Compilation Pipeline State Model

The `BuildOrchestrator` drives the compilation pipeline through a sequence of well-defined states, tracked by the `BuildPhase` enum.

### 1.1 State Diagram

```
                          forge build
                              |
                              v
                    +-------------------+
                    |       Idle        |
                    | (awaiting command)|
                    +-------------------+
                              |
                              | BuildAsync() called
                              v
                    +-------------------+
                    |    Validating     |
                    | ConfigParser      |
                    | .ParseProject()   |
                    +-------------------+
                          |         |
                   success|         |config error
                          v         v
                    +--------+  +-------+
                    |Analyzing|  | Error |
                    |Deps     |  +-------+
                    +--------+
                          |         |
                   no cycle|         |cycle detected
                          v         v
                    +--------+  +-------+
                    | Parsing |  | Error |
                    |(tree-   |  +-------+
                    | sitter) |
                    +--------+
                          |
                          v
                    +-------------------+
                    |    Checking       |
                    | (lint / analyze)  |
                    +-------------------+
                          |
                          v
                    +-------------------+
                    |    Compiling      |
                    | (per module,      |
                    |  parallel layers) |
                    +-------------------+
                       |      |       |
                success|  skip|   fail|
                       v      v       v
                    +---+  +----+  +-------+
                    |   |  |Skip|  | Error |
                    +---+  +----+  +-------+
                       |      |
                       v      v
                    +-------------------+
                    |   Extracting      |
                    | (interface        |
                    |  extraction)      |
                    +-------------------+
                          |
                          v
                    +-------------------+
                    |  Glue Generation  |
                    | (FFI bindings)    |
                    +-------------------+
                          |
                          v
                    +-------------------+
                    |     Linking       |
                    | (LinkerManager)   |
                    +-------------------+
                       |            |
                success|        fail|
                       v            v
                    +------+    +-------+
                    | Done |    | Error |
                    +------+    +-------+
```

### 1.2 State Definitions

| State | BuildPhase Enum | Description | Entry Action | Exit Condition |
|:------|:----------------|:------------|:-------------|:---------------|
| Idle | -- | No build in progress | None | `BuildAsync()` called |
| Validating | `Init` | Parsing `forge.toml` configuration | `ConfigParser.ParseProject()` | Config parsed successfully, or error |
| Analyzing | `DependencyAnalysis` | Computing dependency graph and build order | `DependencyAnalyzer.Analyze()`, `BuildScheduler.CreateSchedule()` | Acyclic graph computed, or cycle detected |
| Parsing | (within Compilation) | Tree-sitter parsing of source files | `forge_parse_source()` via FFI | Parse tree available |
| Checking | (within Compilation) | Static analysis on parse results | `forge_check_syntax()` via FFI | Diagnostics produced |
| Compiling | `Compilation` | Invoking native compilers per module | `ILanguagePlugin.CompileAsync()` | All modules compiled, or first failure |
| Extracting | `InterfaceExtraction` | Extracting public API from artifacts | `InterfaceExtractor.ExtractAsync()` | Interfaces extracted |
| Glue Gen | `GlueGeneration` | Generating cross-language FFI bindings | `GlueGenerator.Generate()` | Glue source files written |
| Linking | `Linking` | Producing final executable or library | `LinkerManager.LinkAsync()` | Binary produced, or link error |
| Done | `Complete` | Build succeeded | Report duration and output path | Terminal state |
| Error | -- | Build failed at any phase | Report error message and partial results | Terminal state |

### 1.3 Transitions

| From | To | Trigger | Guard |
|:-----|:---|:--------|:------|
| Idle | Validating | `BuildAsync()` invoked | -- |
| Validating | Analyzing | `parseResult.IsSuccess == true` | -- |
| Validating | Error | `parseResult.IsSuccess == false` | Configuration file missing or invalid |
| Analyzing | Compiling | `!analyzeResult.HasCycle && scheduleResult.IsSuccess` | -- |
| Analyzing | Error | `analyzeResult.HasCycle` | Circular dependency detected |
| Compiling | Compiling | Layer completed, next layer available | Parallel modules within layer complete |
| Compiling | Extracting | All modules compiled successfully | `options.CompileOnly == false` |
| Compiling | Done | All modules compiled | `options.CompileOnly == true` |
| Compiling | Error | Any module fails | `!compileResult.IsSuccess` |
| Extracting | Glue Gen | Interfaces extracted | `options.GenerateGlue && interfaces.Count > 0` |
| Extracting | Linking | No interfaces to extract | `interfaces.Count == 0` |
| Glue Gen | Linking | Glue files generated | -- |
| Linking | Done | Link succeeded | `linkResult.IsSuccess` |
| Linking | Error | Link failed | `!linkResult.IsSuccess` |

---

## 2. Incremental Build Decision Model

Within the Compiling phase, each module goes through an incremental build check.

```
                 Module ready to compile
                          |
                          v
                 +-------------------+
                 | Check incremental |
                 | cache enabled?    |
                 +-------------------+
                    |            |
               yes  |            | no (--force-rebuild)
                    v            |
          +------------------+   |
          | Check dependency |   |
          | rebuilt?         |   |
          +------------------+   |
             |           |       |
         no deps|     deps       |
         rebuilt |   rebuilt      |
             v           |       |
     +---------------+   |       |
     | Check source  |   |       |
     | hash changed? |   |       |
     +---------------+   |       |
        |          |     |       |
    unchanged  changed   |       |
        |          |     |       |
        v          v     v       v
    +------+   +-----------+
    | Skip |   |  Rebuild  |
    +------+   +-----------+
```

| Decision | Outcome | Reason |
|:---------|:--------|:-------|
| Cache disabled | Rebuild | `options.Incremental == false` or `options.ForceRebuild == true` |
| Dependency rebuilt | Rebuild | Transitive invalidation -- dependent module changed |
| Source hash unchanged | Skip | `IncrementalBuildManager.CheckModule()` returns `NeedsRebuild == false` |
| Source hash changed | Rebuild | File content differs from cached hash |
| No cache entry | Rebuild | First build, no prior cache |

---

## 3. Plugin Lifecycle Model

Each `ILanguagePlugin` instance goes through a lifecycle managed by `PluginHost`.

```
+----------+    RegisterPlugin()    +----------+
| Unloaded | ---------------------> | Loaded   |
+----------+                        +----------+
                                         |
                                         | GetCapabilities()
                                         v
                                    +----------+
                                    | Validated|
                                    | (caps    |
                                    |  stored) |
                                    +----------+
                                         |
                                         | ValidateToolchainAsync()
                                         v
                                 +-----------------+
                              yes|  Toolchain      |no
                              +--| Available?      |--+
                              |  +-----------------+  |
                              v                       v
                        +---------+            +-----------+
                        | Ready   |            | Unavailable|
                        +---------+            +-----------+
                              |
                              | CompileAsync()
                              v
                        +-----------+
                        | Compiling |
                        +-----------+
                           |      |
                    success |      | failure
                           v      v
                     +----------+ +-------+
                     | Complete | | Error |
                     +----------+ +-------+
```

### 3.1 Plugin State Definitions

| State | Description | Transitions Out |
|:------|:------------|:----------------|
| Unloaded | Plugin DLL exists but is not loaded | `RegisterPlugin()` -> Loaded |
| Loaded | Plugin instance created, not yet queried | `GetCapabilities()` -> Validated |
| Validated | Capabilities retrieved, language name registered | `ValidateToolchainAsync()` -> Ready or Unavailable |
| Ready | Toolchain confirmed available | `CompileAsync()` -> Compiling |
| Unavailable | Required compiler not found on system | Manual install then re-validate |
| Compiling | Native compiler invocation in progress | Success -> Complete, Failure -> Error |
| Complete | Compilation finished, artifacts produced | Ready (for next compilation) |
| Error | Compilation failed | Ready (for retry with fixed source) |

### 3.2 Plugin Registration Flow (PluginHost)

```
PluginHost.RegisterPlugin(plugin)
  |
  |-- plugin is null?
  |     yes -> return Failure("Plugin cannot be null")
  |
  |-- capabilities = plugin.GetCapabilities()
  |-- capabilities.LanguageName is empty?
  |     yes -> return Failure("Plugin language name cannot be empty")
  |
  |-- languageName = capabilities.LanguageName.ToLowerInvariant()
  |-- _plugins[languageName] = plugin
  |-- return Success(capabilities)
```

---

## 4. FFI Error State Model

The Rust FFI layer uses a global error storage pattern.

```
             FFI function called
                    |
                    v
            +---------------+
            | Execute logic |
            +---------------+
               |         |
           success    failure
               |         |
               v         v
        +----------+  +-----------------+
        | Return   |  | set_last_error()|
        | JSON ptr |  | return null     |
        +----------+  +-----------------+
               |              |
               v              v
        [caller must      [caller calls
         forge_free_       forge_last_error()
         string()]         then forge_free_string()]
```

| FFI State | Description | C# Action |
|:----------|:------------|:----------|
| Success | Non-null pointer returned | Marshal string, call `forge_free_string()` |
| Failure | Null pointer returned | Call `forge_last_error()`, read error, call `forge_free_string()` |
| Cleared | `forge_clear_error()` called | `forge_last_error()` returns null |

---

## 5. BuildPhase Progress Callback

The `BuildOrchestrator` reports progress via the `BuildProgressCallback` delegate:

```csharp
public delegate void BuildProgressCallback(
    BuildPhase phase,
    string message,
    int current,
    int total);
```

| Phase | Current | Total | Message |
|:------|:-------:|:-----:|:--------|
| Init | 0 | 6 | "Parsing configuration..." |
| DependencyAnalysis | 1 | 6 | "Analyzing dependencies..." |
| Compilation | 2 | 6 | "Compiling modules..." |
| InterfaceExtraction | 3 | 6 | "Extracting interfaces..." |
| GlueGeneration | 4 | 6 | "Generating glue code..." |
| Linking | 5 | 6 | "Linking..." |
| Complete | 6 | 6 | "Build complete" |

---

*MAIDOS-Forge State Model v3.0 -- CodeQC Gate C Compliant*
