# MAIDOS-Forge Operational Runbook

| Field   | Value                   |
|---------|-------------------------|
| Product | MAIDOS-Forge            |
| Version | 3.0                     |
| Type    | Local CLI tool          |

## 1. Common Operations

### 1.1 Initialize a Project

```bash
forge init my-project --lang rust,cpp
```

Creates `forge.toml` and scaffolds a project directory with language-specific templates.

### 1.2 Build

```bash
# Build all modules defined in forge.toml
forge build

# Build a single module
forge build --module my_lib

# Build with verbose output
forge build --verbose

# Release build
forge build --release
```

### 1.3 Check (Validate Without Compiling)

```bash
forge check
```

Validates toolchains, config syntax, and plugin availability without invoking compilers.

### 1.4 Clean

```bash
forge clean
```

Removes all build artifacts from the output directory.

### 1.5 List Plugins

```bash
forge plugins list
```

Shows all detected plugins, their versions, and toolchain status.

### 1.6 Extract Interface

```bash
forge interface extract src/lib.rs --lang rust
```

Parses source and outputs an `InterfaceDescription` JSON to stdout.

### 1.7 Generate Glue Code

```bash
forge glue generate interface.json --target cpp
```

Generates cross-language binding code from an interface description.

## 2. Troubleshooting

### 2.1 Toolchain Not Found (Exit Code 2)

**Symptom**: `Error: toolchain not found for language 'cpp'`

**Cause**: The compiler required by a plugin is not installed or not on PATH.

**Resolution**:
1. Run `forge check` to see which toolchains are missing.
2. Install the required compiler (see DEPLOY.md prerequisites).
3. Ensure the compiler binary is on your system PATH.
4. On Windows, for MSVC, run Forge from a Developer Command Prompt or ensure `cl.exe` is on PATH.

### 2.2 Compilation Errors (Exit Code 1)

**Symptom**: `ForgeError: compilation failed` with file/line/col details.

**Resolution**:
1. Read the error output -- Forge normalizes errors into a unified format: `{file}:{line}:{col} [{severity}] {message}`.
2. Fix the source code at the indicated location.
3. If the error comes from the native compiler, run the compiler directly to get its full diagnostic output: use `--verbose` to see the raw compiler invocation.

### 2.3 Plugin Loading Failures

**Symptom**: `Warning: failed to load plugin 'MaidOS.Forge.Plugin.Go.dll'`

**Causes and resolutions**:

| Cause                          | Resolution                                           |
|--------------------------------|------------------------------------------------------|
| DLL not in `plugins/` dir      | Copy DLL to the correct directory                    |
| .NET version mismatch          | Rebuild plugin against .NET 8.0                      |
| Missing dependency             | Check plugin's deps with `dotnet --list-runtimes`    |
| Corrupt DLL                    | Re-download or rebuild from source                   |

### 2.4 Config Parse Error (Exit Code 3)

**Symptom**: `Error: failed to parse forge.toml`

**Resolution**:
1. Validate TOML syntax (common issues: missing quotes, bad indentation).
2. Run `forge check` to get the specific parse error with line number.
3. Compare against the config schema in DEPLOY.md.

### 2.5 Out of Memory

**Symptom**: Process killed by OS or `OutOfMemoryException`.

**Resolution**:
1. Reduce `parallel_jobs` in `forge.toml`.
2. Build modules individually with `--module`.
3. Check for extremely large source files (>100K lines).

## 3. Error Codes

| Exit Code | Name              | Meaning                                      |
|-----------|-------------------|----------------------------------------------|
| 0         | Success           | All operations completed without error       |
| 1         | CompilationError  | One or more modules failed to compile        |
| 2         | ToolchainMissing  | A required compiler/toolchain is not found   |
| 3         | ConfigError       | forge.toml is missing, malformed, or invalid |
| 4         | PluginError       | A plugin failed to load or crashed           |
| 5         | IoError           | File system read/write failure               |
| 10        | InternalError     | Unexpected Forge bug -- report to maintainers|

## 4. Log Locations and Diagnostics

### Log Output

Forge writes logs to stderr by default. Redirect to a file for analysis:

```bash
forge build --verbose 2> forge.log
```

### Log Levels

Set via `forge.toml` or command line:

```bash
forge build --log-level trace
```

| Level | Content                                          |
|-------|--------------------------------------------------|
| error | Fatal errors only                                |
| warn  | Warnings and errors                              |
| info  | Build progress, timing, results (default)        |
| debug | Plugin load details, config resolution            |
| trace | Full compiler invocations, raw stdout/stderr      |

### Diagnostic Commands

```bash
# Full environment report
forge doctor

# Output: Rust version, .NET version, loaded plugins, toolchain paths, config validation
```

### Build Timing

```bash
forge build --timings
```

Outputs per-module and per-phase timing breakdown to stderr.

## 5. Recovery Procedures

### 5.1 Corrupt Build Artifacts

```bash
forge clean
forge build
```

All artifacts are reproducible from source. Cleaning and rebuilding is always safe.

### 5.2 Broken Plugin State

```bash
# Remove all plugins
rm -rf plugins/

# Rebuild and redeploy plugins from source
dotnet build src/Plugins/ -c Release
cp src/Plugins/*/bin/Release/net8.0/*.dll plugins/
```

### 5.3 Corrupt Configuration

```bash
# Regenerate default forge.toml
forge init --config-only
```

Or restore from version control:

```bash
git checkout -- forge.toml
```

### 5.4 Full Environment Reset

```bash
forge clean
rm -rf plugins/
git checkout -- forge.toml
cargo build --release
dotnet build src/Forge.Cli/ -c Release
# Redeploy plugins as needed
```
