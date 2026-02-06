// MAIDOS-Forge Plugin Interface & Models
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Core.Config;

namespace Forge.Core.Plugin;

/// <summary>
/// Compilation result
/// </summary>
/// <impl>
/// APPROACH: Encapsulates compiler output including artifact paths, logs, errors
/// CALLS: N/A (pure data)
/// EDGES: Error is non-empty when IsSuccess is false, Artifacts can be empty
/// </impl>
public sealed class CompileResult
{
    public bool IsSuccess { get; }
    public string Error { get; }
    public IReadOnlyList<string> Artifacts { get; }
    public IReadOnlyList<string> Logs { get; }
    public TimeSpan Duration { get; }
    public List<ForgeError> Errors { get; set; } = new();

    private CompileResult(bool isSuccess, string error,
        IReadOnlyList<string> artifacts, IReadOnlyList<string> logs, TimeSpan duration)
    {
        IsSuccess = isSuccess;
        Error = error;
        Artifacts = artifacts;
        Logs = logs;
        Duration = duration;
    }

    public static CompileResult Success(IReadOnlyList<string> artifacts,
        IReadOnlyList<string> logs, TimeSpan duration)
        => new(true, string.Empty, artifacts, logs, duration);

    public static CompileResult Failure(string error, IReadOnlyList<string> logs, TimeSpan duration)
        => new(false, error, Array.Empty<string>(), logs, duration);
}

/// <summary>
/// Compilation configuration
/// </summary>
/// <impl>
/// APPROACH: Encapsulates all configuration needed for a single compilation
/// CALLS: N/A (pure data)
/// EDGES: Profile defaults to release, OutputDir must be valid
/// </impl>
public sealed class CompileConfig
{
    public string Profile { get; init; } = "release";
    public string OutputDir { get; init; } = string.Empty;
    public string TargetPlatform { get; init; } = "native";
    public bool Verbose { get; init; }
    public Dictionary<string, string> Environment { get; init; } = new();
}

/// <summary>
/// Plugin capabilities description
/// </summary>
/// <impl>
/// APPROACH: Describes the feature set supported by a plugin
/// CALLS: N/A (pure data)
/// EDGES: All fields have default values
/// </impl>
public sealed class PluginCapabilities
{
    public string LanguageName { get; init; } = string.Empty;
    public IReadOnlyList<string> SupportedExtensions { get; init; } = Array.Empty<string>();
    public bool SupportsNativeCompilation { get; init; }
    public bool SupportsCrossCompilation { get; init; }
    public bool SupportsInterfaceExtraction { get; init; }
    public bool SupportsGlueGeneration { get; init; }
    public IReadOnlyList<string> SupportedTargets { get; init; } = Array.Empty<string>();
}

/// <summary>
/// Interface description JSON structure
/// </summary>
/// <impl>
/// APPROACH: Corresponds to spec section 4 Interface JSON Schema
/// CALLS: N/A (pure data)
/// EDGES: Exports can be empty but not null
/// </impl>
public sealed class InterfaceDescription
{
    public string Version { get; init; } = "1.0";
    public InterfaceModule Module { get; init; } = new();
    public InterfaceLanguage Language { get; init; } = new();
    public IReadOnlyList<ExportedFunction> Exports { get; init; } = Array.Empty<ExportedFunction>();
    public IReadOnlyList<ImportedFunction> Imports { get; init; } = Array.Empty<ImportedFunction>();
}

public sealed class InterfaceModule
{
    public string Name { get; init; } = string.Empty;
    public string Version { get; init; } = "1.0.0";
}

public sealed class InterfaceLanguage
{
    public string Name { get; init; } = string.Empty;
    public string Abi { get; init; } = "c";
    public string Mode { get; init; } = "native";
}

/// <summary>
/// Exported function description
/// </summary>
/// <impl>
/// APPROACH: Describes a cross-language callable function signature
/// CALLS: N/A (pure data)
/// EDGES: Name is required, Parameters can be empty
/// </impl>
public sealed class ExportedFunction
{
    public string Name { get; init; } = string.Empty;
    public string ReturnType { get; init; } = "void";
    public string CallingConvention { get; init; } = "cdecl";
    public IReadOnlyList<FunctionParameter> Parameters { get; init; } = Array.Empty<FunctionParameter>();
}

public sealed class ImportedFunction
{
    public string Name { get; init; } = string.Empty;
    public string FromModule { get; init; } = string.Empty;
    public string ReturnType { get; init; } = "void";
    public string CallingConvention { get; init; } = "cdecl";
    public IReadOnlyList<FunctionParameter> Parameters { get; init; } = Array.Empty<FunctionParameter>();
}

public sealed class FunctionParameter
{
    public string Name { get; init; } = string.Empty;
    public string Type { get; init; } = string.Empty;
}

/// <summary>
/// Glue code generation result
/// </summary>
/// <impl>
/// APPROACH: Encapsulates generated FFI binding code
/// CALLS: N/A (pure data)
/// EDGES: Error is non-empty when IsSuccess is false
/// </impl>
public sealed class GlueCodeResult
{
    public bool IsSuccess { get; }
    public string Error { get; }
    public string SourceCode { get; }
    public string FileName { get; }
    public string TargetLanguage { get; }

    private GlueCodeResult(bool isSuccess, string error,
        string sourceCode, string fileName, string targetLanguage)
    {
        IsSuccess = isSuccess;
        Error = error;
        SourceCode = sourceCode;
        FileName = fileName;
        TargetLanguage = targetLanguage;
    }

    public static GlueCodeResult Success(string sourceCode, string fileName, string targetLanguage)
        => new(true, string.Empty, sourceCode, fileName, targetLanguage);

    public static GlueCodeResult Failure(string error)
        => new(false, error, string.Empty, string.Empty, string.Empty);
}

/// <summary>
/// Language plugin interface
/// </summary>
/// <impl>
/// APPROACH: Defines the methods all language plugins must implement
/// CALLS: Implemented by concrete plugins
/// EDGES: All methods should handle errors and return Result types
/// </impl>
public interface ILanguagePlugin
{
    /// <summary>
    /// Get plugin capabilities description
    /// </summary>
    PluginCapabilities GetCapabilities();

    /// <summary>
    /// Compile a module
    /// </summary>
    /// <impl>
    /// APPROACH: Invokes the corresponding language compiler/build tool
    /// CALLS: External compiler process
    /// EDGES: Compilation failure returns CompileResult.Failure
    /// </impl>
    Task<CompileResult> CompileAsync(ValidatedModuleConfig module, CompileConfig config, CancellationToken ct = default);

    /// <summary>
    /// Extract interface from compiled artifacts
    /// </summary>
    /// <impl>
    /// APPROACH: Parses binary or symbol table to extract exported functions
    /// CALLS: Depends on language (nm, dumpbin, ildasm, etc.)
    /// EDGES: No exports returns empty list, parse failure returns null
    /// </impl>
    Task<InterfaceDescription?> ExtractInterfaceAsync(string artifactPath, CancellationToken ct = default);

    /// <summary>
    /// Generate cross-language glue code
    /// </summary>
    /// <impl>
    /// APPROACH: Generates FFI binding based on interface description
    /// CALLS: String template generation
    /// EDGES: Unsupported target language returns Failure
    /// </impl>
    GlueCodeResult GenerateGlue(InterfaceDescription sourceInterface, string targetLanguage);

    /// <summary>
    /// Validate whether the compiler/toolchain is available
    /// </summary>
    /// <impl>
    /// APPROACH: Checks if the compiler exists in PATH
    /// CALLS: ProcessRunner
    /// EDGES: Unavailable returns error message
    /// </impl>
    Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default);
}
