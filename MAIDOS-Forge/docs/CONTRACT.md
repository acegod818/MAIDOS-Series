# MAIDOS-Forge API Contract / Interface Specification

| Field   | Value                         |
|---------|-------------------------------|
| Product | MAIDOS-Forge                  |
| Version | 3.0                           |
| Type    | Plugin interface + CLI contract|

## 1. ILanguagePlugin Interface

Every language plugin must implement the `ILanguagePlugin` interface. Forge discovers and invokes plugins through this contract.

### C# Interface Definition

```csharp
namespace MaidOS.Forge.Contracts;

public interface ILanguagePlugin
{
    /// Returns the plugin's capabilities and metadata.
    PluginCapabilities GetCapabilities();

    /// Validates that the required toolchain is installed and functional.
    /// Returns (true, "") on success, (false, reason) on failure.
    Task<(bool IsValid, string ErrorMessage)> ValidateToolchainAsync();

    /// Compiles a module using the validated configuration.
    Task<CompileResult> CompileAsync(ValidatedModuleConfig module, CompileConfig config);

    /// Extracts a cross-language interface description from a source file.
    /// Returns null if the file contains no exportable interface.
    Task<InterfaceDescription?> ExtractInterfaceAsync(string sourceFilePath);

    /// Generates glue code to bridge the given interface to a target language.
    GlueCodeResult GenerateGlue(InterfaceDescription iface, string targetLang);
}
```

### Method Details

#### GetCapabilities()

Synchronous. Called once during plugin discovery.

**Returns**: `PluginCapabilities`

```csharp
public sealed class PluginCapabilities
{
    public string LanguageId { get; init; }         // e.g., "rust", "cpp", "go"
    public string DisplayName { get; init; }        // e.g., "C++ (MSVC/GCC/Clang)"
    public string PluginVersion { get; init; }      // SemVer, e.g., "3.0.0"
    public string MinForgeVersion { get; init; }    // Minimum compatible Forge version
    public string[] FileExtensions { get; init; }   // e.g., [".rs"], [".cpp", ".cxx", ".cc"]
    public string[] SupportedTargets { get; init; } // e.g., ["win-x64", "linux-x64"]
    public bool SupportsInterfaceExtraction { get; init; }
    public bool SupportsGlueGeneration { get; init; }
}
```

#### ValidateToolchainAsync()

Called before compilation. Checks that the compiler binary exists, is on PATH, and meets the minimum version requirement.

**Returns**: `(bool IsValid, string ErrorMessage)`

- `(true, "")` -- toolchain is ready.
- `(false, "g++ not found on PATH")` -- toolchain is missing or incompatible.

#### CompileAsync(ValidatedModuleConfig, CompileConfig)

The core compilation method. Forge calls this after toolchain validation passes.

**Parameters**:

```csharp
public sealed class ValidatedModuleConfig
{
    public string ModuleName { get; init; }
    public string[] SourceFiles { get; init; }      // Absolute paths
    public string OutputDir { get; init; }           // Absolute path
    public string OutputType { get; init; }          // "exe", "lib", "dylib"
    public Dictionary<string, string> Options { get; init; } // Language-specific
}

public sealed class CompileConfig
{
    public bool Release { get; init; }
    public int Parallelism { get; init; }
    public string? TargetTriple { get; init; }       // e.g., "x86_64-pc-windows-msvc"
    public Dictionary<string, string> Environment { get; init; }
}
```

**Returns**: `CompileResult` (see Section 2).

#### ExtractInterfaceAsync(string sourceFilePath)

Parses a source file and extracts all publicly exported symbols into an `InterfaceDescription`.

**Parameters**: Absolute path to a source file.

**Returns**: `InterfaceDescription?` -- null if no exportable interface is found (see Section 4).

#### GenerateGlue(InterfaceDescription, string targetLang)

Generates binding code that allows `targetLang` to call into the interface described by `iface`.

**Parameters**:
- `iface`: The interface to generate bindings for.
- `targetLang`: Language identifier of the consumer (e.g., `"csharp"`, `"python"`).

**Returns**: `GlueCodeResult`

```csharp
public sealed class GlueCodeResult
{
    public bool Success { get; init; }
    public string? GeneratedCode { get; init; }      // The glue source code
    public string? OutputFileName { get; init; }     // Suggested file name
    public string? ErrorMessage { get; init; }       // null on success
}
```

## 2. CompileResult Schema

Returned by `CompileAsync`. Also serialized to JSON when using `--output-format json`.

### C# Type

```csharp
public sealed class CompileResult
{
    public bool Success { get; init; }
    public string ModuleName { get; init; }
    public string? OutputPath { get; init; }         // Absolute path to artifact, null on failure
    public TimeSpan CompileTime { get; init; }
    public ForgeError[] Errors { get; init; }
    public ForgeError[] Warnings { get; init; }
}
```

### JSON Representation

```json
{
  "success": true,
  "moduleName": "my_lib",
  "outputPath": "/home/user/project/build/libmy_lib.so",
  "compileTimeMs": 1234,
  "errors": [],
  "warnings": [
    {
      "file": "src/utils.rs",
      "line": 17,
      "col": 9,
      "severity": "warning",
      "message": "unused variable `x`",
      "lang": "rust"
    }
  ]
}
```

## 3. ForgeError Schema

A normalized error representation that unifies diagnostics from all compilers.

### C# Type

```csharp
public sealed class ForgeError
{
    public string File { get; init; }        // Relative path from project root
    public int Line { get; init; }           // 1-based line number, 0 if unknown
    public int Col { get; init; }            // 1-based column number, 0 if unknown
    public string Severity { get; init; }    // "error", "warning", "info"
    public string Message { get; init; }     // Human-readable message
    public string Lang { get; init; }        // Language that produced the diagnostic
}
```

### JSON Representation

```json
{
  "file": "src/main.cpp",
  "line": 42,
  "col": 5,
  "severity": "error",
  "message": "use of undeclared identifier 'foo'",
  "lang": "cpp"
}
```

### Severity Values

| Value     | Meaning                                               |
|-----------|-------------------------------------------------------|
| `error`   | Prevents compilation; build fails                     |
| `warning` | Non-fatal diagnostic; build succeeds                  |
| `info`    | Informational note from the compiler                  |

## 4. InterfaceDescription Schema

Describes the public API surface of a compiled module for cross-language binding generation.

### C# Type

```csharp
public sealed class InterfaceDescription
{
    public string ModuleName { get; init; }
    public string SourceLang { get; init; }
    public FunctionSignature[] Functions { get; init; }
    public TypeDefinition[] Types { get; init; }
}

public sealed class FunctionSignature
{
    public string Name { get; init; }
    public ParameterInfo[] Parameters { get; init; }
    public string ReturnType { get; init; }          // Forge canonical type name
    public string CallingConvention { get; init; }   // "cdecl", "stdcall", "default"
    public bool IsUnsafe { get; init; }
}

public sealed class ParameterInfo
{
    public string Name { get; init; }
    public string Type { get; init; }                // Forge canonical type name
    public bool IsPointer { get; init; }
    public bool IsNullable { get; init; }
}

public sealed class TypeDefinition
{
    public string Name { get; init; }
    public string Kind { get; init; }                // "struct", "enum", "alias"
    public FieldInfo[] Fields { get; init; }         // For structs
    public string[] Variants { get; init; }          // For enums
}

public sealed class FieldInfo
{
    public string Name { get; init; }
    public string Type { get; init; }
    public int Offset { get; init; }                 // Byte offset, -1 if not applicable
}
```

### JSON Representation

```json
{
  "moduleName": "math_core",
  "sourceLang": "rust",
  "functions": [
    {
      "name": "add_vectors",
      "parameters": [
        { "name": "a", "type": "Vec3", "isPointer": true, "isNullable": false },
        { "name": "b", "type": "Vec3", "isPointer": true, "isNullable": false }
      ],
      "returnType": "Vec3",
      "callingConvention": "cdecl",
      "isUnsafe": false
    }
  ],
  "types": [
    {
      "name": "Vec3",
      "kind": "struct",
      "fields": [
        { "name": "x", "type": "f64", "offset": 0 },
        { "name": "y", "type": "f64", "offset": 8 },
        { "name": "z", "type": "f64", "offset": 16 }
      ],
      "variants": []
    }
  ]
}
```

### Forge Canonical Types

| Canonical Type | Description         | C Equivalent     | Rust Equivalent |
|---------------|---------------------|------------------|-----------------|
| `i8`          | Signed 8-bit int    | `int8_t`         | `i8`            |
| `i16`         | Signed 16-bit int   | `int16_t`        | `i16`           |
| `i32`         | Signed 32-bit int   | `int32_t`        | `i32`           |
| `i64`         | Signed 64-bit int   | `int64_t`        | `i64`           |
| `u8`          | Unsigned 8-bit int  | `uint8_t`        | `u8`            |
| `u16`         | Unsigned 16-bit int | `uint16_t`       | `u16`           |
| `u32`         | Unsigned 32-bit int | `uint32_t`       | `u32`           |
| `u64`         | Unsigned 64-bit int | `uint64_t`       | `u64`           |
| `f32`         | 32-bit float        | `float`          | `f32`           |
| `f64`         | 64-bit float        | `double`         | `f64`           |
| `bool`        | Boolean             | `_Bool`          | `bool`          |
| `string`      | UTF-8 string        | `const char*`    | `&str`          |
| `ptr`         | Opaque pointer      | `void*`          | `*mut c_void`   |
| `void`        | No return value     | `void`           | `()`            |

## 5. CLI Exit Codes

| Code | Name               | Meaning                                               |
|------|--------------------|-------------------------------------------------------|
| 0    | Success            | All operations completed without error                |
| 1    | CompilationError   | One or more modules failed to compile                 |
| 2    | ToolchainMissing   | A required compiler/toolchain was not found on PATH   |
| 3    | ConfigError        | forge.toml is missing, malformed, or semantically invalid |
| 4    | PluginError        | A required plugin failed to load or threw an exception|
| 5    | IoError            | File system operation failed (read/write/permission)  |
| 10   | InternalError      | Unexpected Forge bug -- please file an issue          |

### Exit Code Contract

- Exit code 0 is returned if and only if all requested operations succeed.
- Exit codes 1-5 always accompany a human-readable error message on stderr.
- Exit code 10 indicates a bug in Forge itself and includes a stack trace when `--log-level debug` or higher is set.
- Stderr contains diagnostics; stdout contains structured output (if `--output-format json` is used).

## 6. Versioning and Compatibility

- The `ILanguagePlugin` interface follows semantic versioning tied to Forge major versions.
- Forge 3.x will not introduce breaking changes to `ILanguagePlugin`.
- Plugins declare `MinForgeVersion` in their capabilities. Forge skips plugins that require a newer version.
- JSON schemas are additive: new fields may be added in minor versions but existing fields will not be removed or renamed.
