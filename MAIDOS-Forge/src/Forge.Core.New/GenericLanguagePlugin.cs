// MAIDOS-Forge Generic Language Plugin
// UEP v1.7C Compliant - Zero Technical Debt
// M7 Hot-Pluggable Plugin System - Universal Implementation

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;

namespace Forge.Core.Plugin;

/// <summary>
/// 通用語言插件 - 基於 LanguageDefinition 動態支援所有語言
/// </summary>
/// <impl>
/// APPROACH: 提供通用編譯框架，根據語言定義動態選擇工具鏈
/// CALLS: ProcessRunner, LanguageDefinitions
/// EDGES: 未安裝工具鏈返回失敗
/// </impl>
public sealed partial class GenericLanguagePlugin : ILanguagePlugin
{
    private readonly LanguageDefinition _definition;
    private readonly ToolchainConfig _toolchainConfig;

    public GenericLanguagePlugin(LanguageDefinition definition)
    {
        _definition = definition ?? throw new ArgumentNullException(nameof(definition));
        _toolchainConfig = ToolchainRegistry.GetConfig(_definition.Id);
    }

    /// <summary>
    /// 取得插件能力
    /// </summary>
    public PluginCapabilities GetCapabilities()
    {
        return new PluginCapabilities
        {
            LanguageName = _definition.Id,
            SupportedExtensions = _definition.Extensions,
            SupportsNativeCompilation = _toolchainConfig.SupportsNative,
            SupportsCrossCompilation = _toolchainConfig.SupportsCross,
            SupportsInterfaceExtraction = _toolchainConfig.SupportsFFI,
            SupportsGlueGeneration = _toolchainConfig.SupportsFFI,
            SupportedTargets = _toolchainConfig.SupportedTargets
        };
    }

    /// <summary>
    /// 編譯模組
    /// </summary>
    /// <impl>
    /// APPROACH: 根據語言定義選擇編譯命令並執行
    /// CALLS: ProcessRunner.RunAsync()
    /// EDGES: 無可用工具鏈返回失敗
    /// </impl>
    public async Task<CompileResult> CompileAsync(
        ValidatedModuleConfig module,
        CompileConfig config,
        CancellationToken ct = default)
    {
        var logs = new List<string>();
        var stopwatch = System.Diagnostics.Stopwatch.StartNew();

        logs.Add($"[{_definition.Name}] Compiling module '{module.Config.Name}'");

        // 1. 找到可用的工具鏈
        var toolchain = await FindAvailableToolchainAsync(ct);
        if (toolchain is null)
        {
            stopwatch.Stop();
            return CompileResult.Failure(
                $"No available toolchain for {_definition.Name}. " +
                $"Please install one of: {string.Join(", ", _definition.Toolchains)}",
                logs, stopwatch.Elapsed);
        }

        logs.Add($"[{_definition.Name}] Using toolchain: {toolchain}");

        // 2. 收集源文件
        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var sourceFiles = CollectSourceFiles(srcDir);
        if (sourceFiles.Count == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure(
                $"No source files found with extensions: {string.Join(", ", _definition.Extensions)}",
                logs, stopwatch.Elapsed);
        }

        logs.Add($"[{_definition.Name}] Found {sourceFiles.Count} source file(s)");

        // 3. 準備輸出目錄
        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        // 4. 執行編譯
        var compileResult = await ExecuteCompileAsync(
            toolchain, sourceFiles, outputDir, config, logs, ct);

        stopwatch.Stop();

        if (!compileResult.Success)
        {
            return CompileResult.Failure(compileResult.Error, logs, stopwatch.Elapsed);
        }

        // 5. 收集產物
        var artifacts = CollectArtifacts(outputDir);
        logs.Add($"[{_definition.Name}] Build succeeded, {artifacts.Count} artifact(s)");

        return CompileResult.Success(artifacts, logs, stopwatch.Elapsed);
    }

    /// <summary>
    /// 提取接口
    /// </summary>
    public Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        // 返回基本接口描述
        return Task.FromResult<InterfaceDescription?>(new InterfaceDescription
        {
            Version = "1.0",
            Module = new InterfaceModule
            {
                Name = Path.GetFileNameWithoutExtension(artifactPath),
                Version = "1.0.0"
            },
            Language = new InterfaceLanguage
            {
                Name = _definition.Id,
                Abi = "c",
                Mode = "native"
            },
            Exports = Array.Empty<ExportedFunction>()
        });
    }

    /// <summary>
    /// 生成膠水代碼
    /// </summary>
    public GlueCodeResult GenerateGlue(InterfaceDescription sourceInterface, string targetLanguage)
    {
        if (!_toolchainConfig.SupportsFFI)
        {
            return GlueCodeResult.Failure($"{_definition.Name} does not support FFI glue generation");
        }

        // 通用 C 頭檔生成
        if (targetLanguage.Equals("c", StringComparison.OrdinalIgnoreCase))
        {
            return GenerateCHeader(sourceInterface);
        }

        return GlueCodeResult.Failure($"Unsupported target language: {targetLanguage}");
    }

    /// <summary>
    /// 驗證工具鏈
    /// </summary>
    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        foreach (var toolchain in _definition.Toolchains)
        {
            var versionArg = _toolchainConfig.VersionArgs.GetValueOrDefault(toolchain, "--version");
            var version = await ProcessRunner.GetVersionAsync(toolchain, versionArg, ct);

            if (version is not null)
            {
                return (true, $"{toolchain} {version}");
            }
        }

        return (false, $"No toolchain found. Please install one of: {string.Join(", ", _definition.Toolchains)}");
    }

    /// <summary>
    /// 尋找可用的工具鏈
    /// </summary>
    private async Task<string?> FindAvailableToolchainAsync(CancellationToken ct)
    {
        foreach (var toolchain in _definition.Toolchains)
        {
            var versionArg = _toolchainConfig.VersionArgs.GetValueOrDefault(toolchain, "--version");
            var version = await ProcessRunner.GetVersionAsync(toolchain, versionArg, ct);

            if (version is not null)
            {
                return toolchain;
            }
        }

        return null;
    }

    /// <summary>
    /// 收集源文件
    /// </summary>
    private List<string> CollectSourceFiles(string directory)
    {
        var files = new List<string>();

        if (!Directory.Exists(directory))
        {
            return files;
        }

        foreach (var ext in _definition.Extensions)
        {
            var pattern = ext.StartsWith(".") ? $"*{ext}" : $"*.{ext}";
            files.AddRange(Directory.GetFiles(directory, pattern, SearchOption.AllDirectories));
        }

        return files;
    }

    /// <summary>
    /// 執行編譯
    /// </summary>
    private async Task<(bool Success, string Error)> ExecuteCompileAsync(
        string toolchain,
        List<string> sourceFiles,
        string outputDir,
        CompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var commandBuilder = _toolchainConfig.CommandBuilders.GetValueOrDefault(toolchain);
        if (commandBuilder is null)
        {
            return (false, $"No command builder for toolchain: {toolchain}");
        }

        var args = commandBuilder.BuildCompileArgs(sourceFiles, outputDir, config);
        logs.Add($"[{_definition.Name}] Running: {toolchain} {args}");

        var result = await ProcessRunner.RunAsync(
            toolchain, args,
            new ProcessConfig
            {
                WorkingDirectory = Path.GetDirectoryName(sourceFiles[0]) ?? string.Empty,
                Environment = config.Environment,
                Timeout = TimeSpan.FromMinutes(10)
            }, ct);

        if (!string.IsNullOrEmpty(result.Stdout))
        {
            logs.AddRange(result.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries)
                .Select(l => $"[{toolchain}] {l}"));
        }

        if (!result.IsSuccess)
        {
            if (!string.IsNullOrEmpty(result.Stderr))
            {
                logs.AddRange(result.Stderr.Split('\n', StringSplitOptions.RemoveEmptyEntries)
                    .Select(l => $"[error] {l}"));
            }
            return (false, $"Compilation failed with exit code {result.ExitCode}");
        }

        return (true, string.Empty);
    }

    /// <summary>
    /// 收集編譯產物
    /// </summary>
    private List<string> CollectArtifacts(string outputDir)
    {
        var artifacts = new List<string>();

        if (!Directory.Exists(outputDir))
        {
            return artifacts;
        }

        foreach (var outputType in _definition.OutputTypes)
        {
            var pattern = outputType.StartsWith(".") ? $"*{outputType}" : $"*.{outputType}";
            artifacts.AddRange(Directory.GetFiles(outputDir, pattern, SearchOption.AllDirectories));
        }

        // 如果沒有找到特定類型，嘗試收集所有非源文件
        if (artifacts.Count == 0)
        {
            artifacts.AddRange(Directory.GetFiles(outputDir, "*", SearchOption.AllDirectories)
                .Where(f => !_definition.Extensions.Any(ext =>
                    f.EndsWith(ext, StringComparison.OrdinalIgnoreCase))));
        }

        return artifacts;
    }

    /// <summary>
    /// 生成 C 頭檔
    /// </summary>
    private GlueCodeResult GenerateCHeader(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var guard = $"{source.Module.Name.ToUpperInvariant().Replace(".", "_")}_FFI_H";

        sb.AppendLine($"// Auto-generated by MAIDOS-Forge");
        sb.AppendLine($"// Source: {source.Module.Name} ({_definition.Name})");
        sb.AppendLine();
        sb.AppendLine($"#ifndef {guard}");
        sb.AppendLine($"#define {guard}");
        sb.AppendLine();
        sb.AppendLine("#include <stdint.h>");
        sb.AppendLine();
        sb.AppendLine("#ifdef __cplusplus");
        sb.AppendLine("extern \"C\" {");
        sb.AppendLine("#endif");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var cReturn = MapTypeToC(export.ReturnType);
            var cParams = string.Join(", ",
                export.Parameters.Select(p => $"{MapTypeToC(p.Type)} {p.Name}"));

            if (string.IsNullOrEmpty(cParams)) cParams = "void";
            sb.AppendLine($"{cReturn} {export.Name}({cParams});");
        }

        sb.AppendLine();
        sb.AppendLine("#ifdef __cplusplus");
        sb.AppendLine("}");
        sb.AppendLine("#endif");
        sb.AppendLine();
        sb.AppendLine($"#endif // {guard}");

        return GlueCodeResult.Success(sb.ToString(), $"{source.Module.Name}_ffi.h", "c");
    }

    private static string MapTypeToC(string type) => type.ToLowerInvariant() switch
    {
        "void" => "void",
        "bool" => "_Bool",
        "i8" or "sbyte" or "int8" => "int8_t",
        "i16" or "short" or "int16" => "int16_t",
        "i32" or "int" or "int32" => "int32_t",
        "i64" or "long" or "int64" => "int64_t",
        "u8" or "byte" or "uint8" => "uint8_t",
        "u16" or "ushort" or "uint16" => "uint16_t",
        "u32" or "uint" or "uint32" => "uint32_t",
        "u64" or "ulong" or "uint64" => "uint64_t",
        "f32" or "float" => "float",
        "f64" or "double" => "double",
        "isize" or "nint" => "intptr_t",
        "usize" or "nuint" => "size_t",
        "string" or "str" => "const char*",
        _ => "void*"
    };
}

/// <summary>
/// 工具鏈配置
/// </summary>
public sealed record ToolchainConfig
{
    public bool SupportsNative { get; init; } = true;
    public bool SupportsCross { get; init; }
    public bool SupportsFFI { get; init; }
    public string[] SupportedTargets { get; init; } = new[] { "native" };
    public Dictionary<string, string> VersionArgs { get; init; } = new();
    public Dictionary<string, ICommandBuilder> CommandBuilders { get; init; } = new();
}

/// <summary>
/// 命令構建器介面
/// </summary>
public interface ICommandBuilder
{
    string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config);
}

/// <summary>
/// 通用 C 家族命令構建器
/// </summary>
public sealed class CStyleCommandBuilder : ICommandBuilder
{
    private readonly string _outputFlag;
    private readonly string _optimizeFlag;
    private readonly string _debugFlag;

    public CStyleCommandBuilder(string outputFlag = "-o", string optimizeFlag = "-O2", string debugFlag = "-g")
    {
        _outputFlag = outputFlag;
        _optimizeFlag = optimizeFlag;
        _debugFlag = debugFlag;
    }

    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        var outputFile = Path.Combine(outputDir, "output");
        var files = string.Join(" ", sourceFiles.Select(f => $"\"{f}\""));
        var optFlag = config.Profile == "debug" ? _debugFlag : _optimizeFlag;

        return $"{optFlag} {files} {_outputFlag} \"{outputFile}\"";
    }
}

/// <summary>
/// Cargo 命令構建器 (Rust)
/// </summary>
public sealed class CargoCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        var profile = config.Profile == "debug" ? "" : "--release";
        return $"build {profile} --target-dir \"{outputDir}\"";
    }
}

/// <summary>
/// Go 命令構建器
/// </summary>
public sealed class GoCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        var outputFile = Path.Combine(outputDir, "main");
        return $"build -o \"{outputFile}\" {string.Join(" ", sourceFiles.Select(f => $"\"{f}\""))}";
    }
}

/// <summary>
/// dotnet 命令構建器
/// </summary>
public sealed class DotnetCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        var configuration = config.Profile == "debug" ? "Debug" : "Release";
        // 假設第一個是 .csproj/.fsproj
        var project = sourceFiles.FirstOrDefault(f => f.EndsWith("proj", StringComparison.OrdinalIgnoreCase))
                      ?? sourceFiles[0];
        return $"build \"{project}\" --configuration {configuration} --output \"{outputDir}\"";
    }
}

/// <summary>
/// 工具鏈註冊表
/// </summary>
public static class ToolchainRegistry
{
    private static readonly Dictionary<string, ToolchainConfig> Configs = new(StringComparer.OrdinalIgnoreCase);

    static ToolchainRegistry()
    {
        // ═══════════════════════════════════════════════════════════════
        // 系統編程語言
        // ═══════════════════════════════════════════════════════════════

        Register("c", new ToolchainConfig
        {
            SupportsNative = true,
            SupportsCross = true,
            SupportsFFI = true,
            SupportedTargets = new[] { "native", "linux-x64", "linux-arm64", "win-x64" },
            VersionArgs = new() { ["clang"] = "--version", ["gcc"] = "--version" },
            CommandBuilders = new()
            {
                ["clang"] = new CStyleCommandBuilder(),
                ["gcc"] = new CStyleCommandBuilder(),
                ["tcc"] = new CStyleCommandBuilder("-o", "-O2", "-g")
            }
        });

        Register("cpp", new ToolchainConfig
        {
            SupportsNative = true,
            SupportsCross = true,
            SupportsFFI = true,
            SupportedTargets = new[] { "native", "linux-x64", "linux-arm64", "win-x64" },
            VersionArgs = new() { ["clang++"] = "--version", ["g++"] = "--version" },
            CommandBuilders = new()
            {
                ["clang++"] = new CStyleCommandBuilder(),
                ["g++"] = new CStyleCommandBuilder()
            }
        });

        Register("rust", new ToolchainConfig
        {
            SupportsNative = true,
            SupportsCross = true,
            SupportsFFI = true,
            SupportedTargets = new[] { "native", "wasm32", "linux-x64", "linux-arm64", "win-x64" },
            VersionArgs = new() { ["cargo"] = "--version", ["rustc"] = "--version" },
            CommandBuilders = new()
            {
                ["cargo"] = new CargoCommandBuilder(),
                ["rustc"] = new CStyleCommandBuilder("-o", "-O", "-g")
            }
        });

        Register("zig", new ToolchainConfig
        {
            SupportsNative = true,
            SupportsCross = true,
            SupportsFFI = true,
            VersionArgs = new() { ["zig"] = "version" },
            CommandBuilders = new()
            {
                ["zig"] = new ZigCommandBuilder()
            }
        });

        Register("go", new ToolchainConfig
        {
            SupportsNative = true,
            SupportsCross = true,
            SupportsFFI = true,
            VersionArgs = new() { ["go"] = "version" },
            CommandBuilders = new()
            {
                ["go"] = new GoCommandBuilder()
            }
        });

        // ═══════════════════════════════════════════════════════════════
        // 託管語言
        // ═══════════════════════════════════════════════════════════════

        Register("csharp", new ToolchainConfig
        {
            SupportsNative = true,
            SupportsCross = true,
            SupportsFFI = true,
            VersionArgs = new() { ["dotnet"] = "--version" },
            CommandBuilders = new()
            {
                ["dotnet"] = new DotnetCommandBuilder()
            }
        });

        Register("java", new ToolchainConfig
        {
            SupportsNative = false,
            SupportsFFI = true,
            VersionArgs = new() { ["javac"] = "-version", ["java"] = "-version" },
            CommandBuilders = new()
            {
                ["javac"] = new JavacCommandBuilder()
            }
        });

        Register("kotlin", new ToolchainConfig
        {
            SupportsNative = true,
            SupportsFFI = true,
            VersionArgs = new() { ["kotlinc"] = "-version" },
            CommandBuilders = new()
            {
                ["kotlinc"] = new KotlincCommandBuilder()
            }
        });

        Register("fsharp", new ToolchainConfig
        {
            SupportsNative = true,
            SupportsFFI = true,
            VersionArgs = new() { ["dotnet"] = "--version" },
            CommandBuilders = new()
            {
                ["dotnet"] = new DotnetCommandBuilder()
            }
        });

        // ═══════════════════════════════════════════════════════════════
        // 腳本語言
        // ═══════════════════════════════════════════════════════════════

        Register("python", new ToolchainConfig
        {
            SupportsNative = true, // via Cython/mypyc/Nuitka
            SupportsFFI = true,
            VersionArgs = new() { ["python3"] = "--version", ["python"] = "--version" },
            CommandBuilders = new()
            {
                ["python3"] = new PythonCommandBuilder(),
                ["python"] = new PythonCommandBuilder(),
                ["cython"] = new CythonCommandBuilder()
            }
        });

        // ═══════════════════════════════════════════════════════════════
        // 函數式語言
        // ═══════════════════════════════════════════════════════════════

        Register("haskell", new ToolchainConfig
        {
            SupportsNative = true,
            SupportsFFI = true,
            VersionArgs = new() { ["ghc"] = "--version" },
            CommandBuilders = new()
            {
                ["ghc"] = new GhcCommandBuilder()
            }
        });

        Register("ocaml", new ToolchainConfig
        {
            SupportsNative = true,
            SupportsFFI = true,
            VersionArgs = new() { ["ocamlopt"] = "-version" },
            CommandBuilders = new()
            {
                ["ocamlopt"] = new OcamlCommandBuilder()
            }
        });

        Register("erlang", new ToolchainConfig
        {
            SupportsFFI = true,
            VersionArgs = new() { ["erlc"] = "-version" },
            CommandBuilders = new()
            {
                ["erlc"] = new ErlangCommandBuilder()
            }
        });

        Register("elixir", new ToolchainConfig
        {
            SupportsFFI = true,
            VersionArgs = new() { ["elixir"] = "--version" },
            CommandBuilders = new()
            {
                ["mix"] = new MixCommandBuilder()
            }
        });

        // ═══════════════════════════════════════════════════════════════
        // 科學計算語言
        // ═══════════════════════════════════════════════════════════════

        Register("julia", new ToolchainConfig
        {
            SupportsNative = true,
            SupportsFFI = true,
            VersionArgs = new() { ["julia"] = "--version" },
            CommandBuilders = new()
            {
                ["julia"] = new JuliaCommandBuilder()
            }
        });

        Register("mojo", new ToolchainConfig
        {
            SupportsNative = true,
            SupportsFFI = true,
            VersionArgs = new() { ["mojo"] = "--version" },
            CommandBuilders = new()
            {
                ["mojo"] = new MojoCommandBuilder()
            }
        });

        // ═══════════════════════════════════════════════════════════════
        // 硬體描述語言
        // ═══════════════════════════════════════════════════════════════

        Register("vhdl", new ToolchainConfig
        {
            VersionArgs = new() { ["ghdl"] = "--version" },
            CommandBuilders = new()
            {
                ["ghdl"] = new GhdlCommandBuilder()
            }
        });

        Register("verilog", new ToolchainConfig
        {
            VersionArgs = new() { ["iverilog"] = "-V" },
            CommandBuilders = new()
            {
                ["iverilog"] = new IverilogCommandBuilder()
            }
        });

        // ═══════════════════════════════════════════════════════════════
        // 智能合約語言
        // ═══════════════════════════════════════════════════════════════

        Register("solidity", new ToolchainConfig
        {
            VersionArgs = new() { ["solc"] = "--version" },
            CommandBuilders = new()
            {
                ["solc"] = new SolcCommandBuilder()
            }
        });

        // 為未明確配置的語言提供預設配置
        RegisterDefaults();
    }

    private static void RegisterDefaults()
    {
        var defaultConfig = new ToolchainConfig
        {
            SupportsNative = false,
            SupportsCross = false,
            SupportsFFI = false,
            SupportedTargets = new[] { "native" },
            CommandBuilders = new()
        };

        foreach (var lang in LanguageDefinitions.GetAllLanguages())
        {
            if (!Configs.ContainsKey(lang.Id))
            {
                Configs[lang.Id] = defaultConfig with
                {
                    VersionArgs = lang.Toolchains.ToDictionary(t => t, _ => "--version")
                };
            }
        }
    }

    public static void Register(string languageId, ToolchainConfig config)
    {
        Configs[languageId] = config;
    }

    public static ToolchainConfig GetConfig(string languageId)
    {
        return Configs.TryGetValue(languageId, out var config)
            ? config
            : new ToolchainConfig();
    }
}

// ═══════════════════════════════════════════════════════════════
// 各語言專用命令構建器
// ═══════════════════════════════════════════════════════════════

public sealed class ZigCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        var mode = config.Profile == "debug" ? "Debug" : "ReleaseFast";
        return $"build-exe {string.Join(" ", sourceFiles)} -femit-bin=\"{outputDir}/output\" -O{mode}";
    }
}

public sealed class JavacCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        return $"-d \"{outputDir}\" {string.Join(" ", sourceFiles.Select(f => $"\"{f}\""))}";
    }
}

public sealed class KotlincCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        return $"-d \"{outputDir}\" {string.Join(" ", sourceFiles.Select(f => $"\"{f}\""))}";
    }
}

public sealed class PythonCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        return $"-m py_compile {string.Join(" ", sourceFiles.Select(f => $"\"{f}\""))}";
    }
}

public sealed class CythonCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        return $"-3 --embed {string.Join(" ", sourceFiles)} -o \"{outputDir}/output.c\"";
    }
}

public sealed class GhcCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        var optLevel = config.Profile == "debug" ? "-O0" : "-O2";
        return $"{optLevel} {string.Join(" ", sourceFiles)} -outputdir \"{outputDir}\" -o \"{outputDir}/output\"";
    }
}

public sealed class OcamlCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        return $"-o \"{outputDir}/output\" {string.Join(" ", sourceFiles)}";
    }
}

public sealed class ErlangCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        return $"-o \"{outputDir}\" {string.Join(" ", sourceFiles)}";
    }
}

public sealed class MixCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        var env = config.Profile == "debug" ? "dev" : "prod";
        return $"compile --output \"{outputDir}\"";
    }
}

public sealed class JuliaCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        return $"--compile=all -o \"{outputDir}/output\" {sourceFiles[0]}";
    }
}

public sealed class MojoCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        return $"build {sourceFiles[0]} -o \"{outputDir}/output\"";
    }
}

public sealed class GhdlCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        return $"-a --workdir=\"{outputDir}\" {string.Join(" ", sourceFiles)}";
    }
}

public sealed class IverilogCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        return $"-o \"{outputDir}/output.vvp\" {string.Join(" ", sourceFiles)}";
    }
}

public sealed class SolcCommandBuilder : ICommandBuilder
{
    public string BuildCompileArgs(List<string> sourceFiles, string outputDir, CompileConfig config)
    {
        var optFlag = config.Profile == "debug" ? "" : "--optimize";
        return $"{optFlag} --output-dir \"{outputDir}\" --bin --abi {string.Join(" ", sourceFiles)}";
    }
}
