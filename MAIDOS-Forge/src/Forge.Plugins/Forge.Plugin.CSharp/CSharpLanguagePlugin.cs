// MAIDOS-Forge C# Language Plugin
// UEP v1.7C Compliant - Zero Technical Debt
// Standalone Plugin Module

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.CSharp;

/// <summary>
/// C# 語言插件 - 支援 CLR 和 NativeAOT 模式
/// </summary>
/// <impl>
/// APPROACH: 封裝 dotnet build 命令，支援兩種編譯模式
/// CALLS: ProcessRunner.RunAsync(), dotnet CLI
/// EDGES: 無 dotnet CLI 時 ValidateToolchain 返回失敗
/// </impl>
public sealed class CSharpLanguagePlugin : ILanguagePlugin
{
    private const string DotnetCommand = "dotnet";

    /// <summary>
    /// 取得插件能力
    /// </summary>
    public PluginCapabilities GetCapabilities()
    {
        return new PluginCapabilities
        {
            LanguageName = "csharp",
            SupportedExtensions = new[] { ".cs", ".csproj" },
            SupportsNativeCompilation = true,
            SupportsCrossCompilation = true,
            SupportsInterfaceExtraction = true,
            SupportsGlueGeneration = true,
            SupportedTargets = new[]
            {
                "win-x64", "win-x86", "win-arm64",
                "linux-x64", "linux-arm64",
                "osx-x64", "osx-arm64"
            }
        };
    }

    /// <summary>
    /// 編譯 C# 模組
    /// </summary>
    public async Task<CompileResult> CompileAsync(
        ValidatedModuleConfig module, 
        CompileConfig config,
        CancellationToken ct = default)
    {
        var logs = new List<string>();
        var stopwatch = System.Diagnostics.Stopwatch.StartNew();

        var mode = module.Config.CSharp?.Mode?.ToLowerInvariant() ?? "clr";
        logs.Add($"[CSharp] Compiling module '{module.Config.Name}' in {mode} mode");

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var csprojFiles = Directory.GetFiles(srcDir, "*.csproj", SearchOption.TopDirectoryOnly);
        string projectFile;

        if (csprojFiles.Length == 0)
        {
            logs.Add("[CSharp] No .csproj found, creating temporary project");
            var createResult = await CreateTemporaryProjectAsync(module, srcDir, config, ct);
            if (!createResult.Success)
            {
                stopwatch.Stop();
                return CompileResult.Failure(createResult.Error, logs, stopwatch.Elapsed);
            }
            projectFile = createResult.ProjectFile;
            logs.Add($"[CSharp] Created temporary project: {projectFile}");
        }
        else
        {
            projectFile = csprojFiles[0];
            logs.Add($"[CSharp] Using project: {projectFile}");
        }

        CompileResult result;
        if (mode == "nativeaot")
        {
            result = await CompileNativeAotAsync(module, projectFile, config, logs, ct);
        }
        else
        {
            result = await CompileClrAsync(module, projectFile, config, logs, ct);
        }

        stopwatch.Stop();
        return result;
    }

    private async Task<CompileResult> CompileClrAsync(
        ValidatedModuleConfig module,
        string projectFile,
        CompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var stopwatch = System.Diagnostics.Stopwatch.StartNew();
        var configuration = config.Profile == "debug" ? "Debug" : "Release";
        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var args = $"build \"{projectFile}\" " +
                   $"--configuration {configuration} " +
                   $"--output \"{outputDir}\"";

        if (config.Verbose)
        {
            args += " --verbosity detailed";
        }

        logs.Add($"[CSharp] Running: dotnet {args}");

        var result = await ProcessRunner.RunAsync(
            DotnetCommand, args,
            new ProcessConfig
            {
                WorkingDirectory = Path.GetDirectoryName(projectFile) ?? string.Empty,
                Environment = config.Environment,
                Timeout = TimeSpan.FromMinutes(5)
            }, ct);

        if (!string.IsNullOrEmpty(result.Stdout))
        {
            logs.AddRange(result.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries)
                .Select(l => $"[dotnet] {l}"));
        }

        if (!result.IsSuccess)
        {
            logs.Add($"[CSharp] Build failed with exit code {result.ExitCode}");
            if (!string.IsNullOrEmpty(result.Stderr))
            {
                logs.AddRange(result.Stderr.Split('\n', StringSplitOptions.RemoveEmptyEntries)
                    .Select(l => $"[error] {l}"));
            }
            stopwatch.Stop();
            return CompileResult.Failure($"dotnet build failed: {result.Stderr}", logs, stopwatch.Elapsed);
        }

        var artifacts = Directory.GetFiles(outputDir, "*.dll")
            .Concat(Directory.GetFiles(outputDir, "*.exe"))
            .ToList();

        logs.Add($"[CSharp] Build succeeded, {artifacts.Count} artifact(s)");
        stopwatch.Stop();
        return CompileResult.Success(artifacts, logs, stopwatch.Elapsed);
    }

    private async Task<CompileResult> CompileNativeAotAsync(
        ValidatedModuleConfig module,
        string projectFile,
        CompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var stopwatch = System.Diagnostics.Stopwatch.StartNew();
        var configuration = config.Profile == "debug" ? "Debug" : "Release";
        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var rid = config.TargetPlatform;
        if (rid == "native")
        {
            rid = GetCurrentRid();
        }

        var args = $"publish \"{projectFile}\" " +
                   $"--configuration {configuration} " +
                   $"--output \"{outputDir}\" " +
                   $"-r {rid} " +
                   "-p:PublishAot=true " +
                   "-p:StripSymbols=true";

        if (config.Verbose)
        {
            args += " --verbosity detailed";
        }

        logs.Add($"[CSharp] Running NativeAOT: dotnet {args}");

        var result = await ProcessRunner.RunAsync(
            DotnetCommand, args,
            new ProcessConfig
            {
                WorkingDirectory = Path.GetDirectoryName(projectFile) ?? string.Empty,
                Environment = config.Environment,
                Timeout = TimeSpan.FromMinutes(15)
            }, ct);

        if (!string.IsNullOrEmpty(result.Stdout))
        {
            logs.AddRange(result.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries)
                .Select(l => $"[dotnet] {l}"));
        }

        if (!result.IsSuccess)
        {
            logs.Add($"[CSharp] NativeAOT publish failed with exit code {result.ExitCode}");
            if (!string.IsNullOrEmpty(result.Stderr))
            {
                logs.AddRange(result.Stderr.Split('\n', StringSplitOptions.RemoveEmptyEntries)
                    .Select(l => $"[error] {l}"));
            }
            stopwatch.Stop();
            return CompileResult.Failure($"NativeAOT publish failed: {result.Stderr}", logs, stopwatch.Elapsed);
        }

        var artifacts = new List<string>();
        var exePattern = OperatingSystem.IsWindows() ? "*.exe" : "*";
        foreach (var file in Directory.GetFiles(outputDir))
        {
            var name = Path.GetFileName(file);
            if (!name.Contains('.') || name.EndsWith(".exe"))
            {
                artifacts.Add(file);
            }
        }

        logs.Add($"[CSharp] NativeAOT publish succeeded, {artifacts.Count} artifact(s)");
        stopwatch.Stop();
        return CompileResult.Success(artifacts, logs, stopwatch.Elapsed);
    }

    private async Task<(bool Success, string ProjectFile, string Error)> CreateTemporaryProjectAsync(
        ValidatedModuleConfig module,
        string srcDir,
        CompileConfig config,
        CancellationToken ct)
    {
        var csFiles = Directory.GetFiles(srcDir, "*.cs", SearchOption.AllDirectories);
        if (csFiles.Length == 0)
        {
            return (false, string.Empty, "No .cs files found in module");
        }

        var projectFile = Path.Combine(srcDir, $"{module.Config.Name}.csproj");
        var csharpConfig = module.Config.CSharp ?? new CSharpConfig();
        
        var outputType = csharpConfig.OutputType ?? "Library";
        var framework = csharpConfig.Framework ?? "net8.0";
        var nullable = csharpConfig.Nullable;
        var implicitUsings = csharpConfig.ImplicitUsings ? "enable" : "disable";

        var csproj = $"""
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup>
                <OutputType>{outputType}</OutputType>
                <TargetFramework>{framework}</TargetFramework>
                <Nullable>{nullable}</Nullable>
                <ImplicitUsings>{implicitUsings}</ImplicitUsings>
                <TreatWarningsAsErrors>true</TreatWarningsAsErrors>
              </PropertyGroup>
            </Project>
            """;

        try
        {
            await File.WriteAllTextAsync(projectFile, csproj, ct);
            return (true, projectFile, string.Empty);
        }
        catch (Exception ex)
        {
            return (false, string.Empty, $"Failed to create project file: {ex.Message}");
        }
    }

    /// <summary>
    /// 從編譯產物提取接口
    /// </summary>
    public Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath, 
        CancellationToken ct = default)
    {
        if (!artifactPath.EndsWith(".dll", StringComparison.OrdinalIgnoreCase))
        {
            return Task.FromResult<InterfaceDescription?>(null);
        }

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
                Name = "csharp",
                Abi = "clr",
                Mode = "clr"
            },
            Exports = Array.Empty<ExportedFunction>()
        });
    }

    /// <summary>
    /// 生成跨語言膠水代碼
    /// </summary>
    public GlueCodeResult GenerateGlue(InterfaceDescription sourceInterface, string targetLanguage)
    {
        return targetLanguage.ToLowerInvariant() switch
        {
            "rust" => GenerateRustGlue(sourceInterface),
            "c" => GenerateCGlue(sourceInterface),
            _ => GlueCodeResult.Failure($"Unsupported target language: {targetLanguage}")
        };
    }

    private GlueCodeResult GenerateRustGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        sb.AppendLine("// Auto-generated by MAIDOS-Forge C# Plugin");
        sb.AppendLine($"// Source: {source.Module.Name}");
        sb.AppendLine();
        sb.AppendLine("#[link(name = \"" + source.Module.Name + "\")]");
        sb.AppendLine("extern \"C\" {");

        foreach (var export in source.Exports)
        {
            var rustReturn = MapTypeToRust(export.ReturnType);
            var rustParams = string.Join(", ", 
                export.Parameters.Select(p => $"{p.Name}: {MapTypeToRust(p.Type)}"));
            
            sb.AppendLine($"    pub fn {export.Name}({rustParams}) -> {rustReturn};");
        }

        sb.AppendLine("}");

        var fileName = $"{source.Module.Name}_ffi.rs";
        return GlueCodeResult.Success(sb.ToString(), fileName, "rust");
    }

    private GlueCodeResult GenerateCGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var guard = $"{source.Module.Name.ToUpperInvariant()}_FFI_H";
        
        sb.AppendLine("// Auto-generated by MAIDOS-Forge C# Plugin");
        sb.AppendLine($"// Source: {source.Module.Name}");
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

        var fileName = $"{source.Module.Name}_ffi.h";
        return GlueCodeResult.Success(sb.ToString(), fileName, "c");
    }

    /// <summary>
    /// 驗證 dotnet CLI 是否可用
    /// </summary>
    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        var version = await ProcessRunner.GetVersionAsync(DotnetCommand, "--version", ct);
        
        if (version is null)
        {
            return (false, "dotnet CLI not found. Please install .NET SDK from https://dot.net");
        }

        return (true, $"dotnet {version}");
    }

    private static string GetCurrentRid()
    {
        var os = OperatingSystem.IsWindows() ? "win" :
                 OperatingSystem.IsMacOS() ? "osx" : "linux";
        
        var arch = System.Runtime.InteropServices.RuntimeInformation.OSArchitecture switch
        {
            System.Runtime.InteropServices.Architecture.X64 => "x64",
            System.Runtime.InteropServices.Architecture.X86 => "x86",
            System.Runtime.InteropServices.Architecture.Arm64 => "arm64",
            System.Runtime.InteropServices.Architecture.Arm => "arm",
            _ => "x64"
        };

        return $"{os}-{arch}";
    }

    private static string MapTypeToRust(string type) => type.ToLowerInvariant() switch
    {
        "void" => "()",
        "bool" => "bool",
        "i8" or "sbyte" => "i8",
        "i16" or "short" => "i16",
        "i32" or "int" => "i32",
        "i64" or "long" => "i64",
        "u8" or "byte" => "u8",
        "u16" or "ushort" => "u16",
        "u32" or "uint" => "u32",
        "u64" or "ulong" => "u64",
        "f32" or "float" => "f32",
        "f64" or "double" => "f64",
        "isize" or "nint" => "isize",
        "usize" or "nuint" => "usize",
        _ => "*mut std::ffi::c_void"
    };

    private static string MapTypeToC(string type) => type.ToLowerInvariant() switch
    {
        "void" => "void",
        "bool" => "_Bool",
        "i8" or "sbyte" => "int8_t",
        "i16" or "short" => "int16_t",
        "i32" or "int" => "int32_t",
        "i64" or "long" => "int64_t",
        "u8" or "byte" => "uint8_t",
        "u16" or "ushort" => "uint16_t",
        "u32" or "uint" => "uint32_t",
        "u64" or "ulong" => "uint64_t",
        "f32" or "float" => "float",
        "f64" or "double" => "double",
        "isize" or "nint" => "intptr_t",
        "usize" or "nuint" => "size_t",
        _ => "void*"
    };
}
