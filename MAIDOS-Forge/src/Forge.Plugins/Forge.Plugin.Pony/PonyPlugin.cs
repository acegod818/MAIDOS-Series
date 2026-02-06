// MAIDOS-Forge Pony Language Plugin
// Code-QC v2.2B Compliant
// M10 Extension Plugin - Concurrent Languages

using System.Text;
using System.Text.RegularExpressions;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.Pony;

/// <summary>
/// Pony 語言插件 - 高性能並發語言
/// </summary>
/// <impl>
/// APPROACH: 支援 ponyc 編譯器
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: 需要 ponyc 0.50+
/// </impl>
public sealed class PonyPlugin : ILanguagePlugin
{
    private static readonly string[] SourceExtensions = { ".pony" };

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "pony",
        SupportedExtensions = SourceExtensions,
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        // 檢查 ponyc
        if (await ProcessRunner.CommandExistsAsync("ponyc"))
        {
            var version = await ProcessRunner.GetVersionAsync("ponyc", "--version");
            var match = Regex.Match(version ?? "", @"ponyc ([\d.]+)");
            if (match.Success)
                return (true, $"ponyc {match.Groups[1].Value}");
        }

        return (false, "Pony not found. Install from https://github.com/ponylang/ponyc");
    }

    public async Task<CompileResult> CompileAsync(ValidatedModuleConfig module, CompileConfig config, CancellationToken ct = default)
    {
        var logs = new List<string>();
        var stopwatch = System.Diagnostics.Stopwatch.StartNew();

        var (available, toolchainMsg) = await ValidateToolchainAsync(ct);
        if (!available)
        {
            stopwatch.Stop();
            return CompileResult.Failure(toolchainMsg, logs, stopwatch.Elapsed);
        }

        logs.Add($"[Pony] Using: {toolchainMsg}");

        var srcDir = Path.Combine(module.ModulePath, "src") is var sd && Directory.Exists(sd)
            ? sd : module.ModulePath;

        var sourceFiles = SourceExtensions
            .SelectMany(ext => Directory.GetFiles(srcDir, $"*{ext}", SearchOption.AllDirectories))
            .ToArray();

        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No Pony source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[Pony] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        // 構建命令
        string buildCmd = "ponyc";
        string buildArgs = $"--output=\"{outputDir}\" \"{srcDir}\"";

        logs.Add($"[Pony] Building with: {buildCmd} {buildArgs}");

        try
        {
            var processConfig = new ProcessConfig { WorkingDirectory = module.ModulePath };
            var result = await ProcessRunner.RunAsync(buildCmd, buildArgs, processConfig, ct);
            
            logs.AddRange(result.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries));
            if (!string.IsNullOrWhiteSpace(result.Stderr))
            {
                logs.AddRange(result.Stderr.Split('\n', StringSplitOptions.RemoveEmptyEntries));
            }

            stopwatch.Stop();

            if (result.IsSuccess)
            {
                // 尋找生成的可執行文件
                var artifacts = new List<string>();
                var exePath = Path.Combine(outputDir, module.Config.Name + (Environment.OSVersion.Platform == PlatformID.Win32NT ? ".exe" : ""));
                if (File.Exists(exePath))
                {
                    artifacts.Add(exePath);
                }

                return CompileResult.Success(artifacts, logs, stopwatch.Elapsed);
            }
            else
            {
                return CompileResult.Failure($"Build failed with exit code {result.ExitCode}", logs, stopwatch.Elapsed);
            }
        }
        catch (OperationCanceledException)
        {
            stopwatch.Stop();
            return CompileResult.Failure("Build cancelled", logs, stopwatch.Elapsed);
        }
        catch (Exception ex)
        {
            stopwatch.Stop();
            return CompileResult.Failure($"Build error: {ex.Message}", logs, stopwatch.Elapsed);
        }
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(string artifactPath, CancellationToken ct = default)
    {
        // Pony 接口提取是一個複雜的過程，需要解析 Pony 源碼中的 actor 和 interface 聲明
        // 這裏提供一個簡化的實現
        
        try
        {
            // 檢查是否為 Pony 可執行文件
            if (!File.Exists(artifactPath))
                return null;

            // 在實際實現中，我們需要解析 Pony 源碼中的公共接口
            // 這裏返回一個簡化的接口描述
            
            return new InterfaceDescription
            {
                Version = "1.0",
                Module = new InterfaceModule
                {
                    Name = Path.GetFileNameWithoutExtension(artifactPath),
                    Version = "1.0.0"
                },
                Language = new InterfaceLanguage
                {
                    Name = "pony",
                    Abi = "native",
                    Mode = "native"
                },
                Exports = Array.Empty<ExportedFunction>(), // 實際實現中需要解析源碼
                Imports = Array.Empty<ImportedFunction>()
            };
        }
        catch
        {
            return null;
        }
    }

    public GlueCodeResult GenerateGlue(InterfaceDescription sourceInterface, string targetLanguage)
    {
        try
        {
            // 生成 FFI 膠水代碼
            var sb = new StringBuilder();
            
            switch (targetLanguage.ToLower())
            {
                case "c":
                    sb.AppendLine("// Pony FFI glue code for C");
                    sb.AppendLine("#include <pony.h>");
                    sb.AppendLine();
                    
                    // 為每個導出函數生成 C 聲明
                    foreach (var func in sourceInterface.Exports)
                    {
                        var returnType = MapPonyTypeToC(func.ReturnType);
                        var parameters = string.Join(", ", func.Parameters.Select(p => $"{MapPonyTypeToC(p.Type)} {p.Name}"));
                        sb.AppendLine($"PONY_EXTERN {returnType} {func.Name}({parameters});");
                    }
                    break;
                    
                case "csharp":
                    sb.AppendLine("// Pony FFI glue code for C#");
                    sb.AppendLine("using System;");
                    sb.AppendLine("using System.Runtime.InteropServices;");
                    sb.AppendLine();
                    sb.AppendLine("namespace PonyInterop");
                    sb.AppendLine("{");
                    sb.AppendLine("    internal static class PonyNative");
                    sb.AppendLine("    {");
                    sb.AppendLine("        private const string LibraryName = \"pony_library\";");
                    sb.AppendLine();
                    
                    // 為每個導出函數生成 P/Invoke 聲明
                    foreach (var func in sourceInterface.Exports)
                    {
                        var returnType = MapPonyTypeToCSharp(func.ReturnType);
                        var parameters = string.Join(", ", func.Parameters.Select(p => $"{MapPonyTypeToCSharp(p.Type)} {p.Name}"));
                        sb.AppendLine($"        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]");
                        sb.AppendLine($"        public static extern {returnType} {func.Name}({parameters});");
                        sb.AppendLine();
                    }
                    
                    sb.AppendLine("    }");
                    sb.AppendLine("}");
                    break;
                    
                default:
                    return GlueCodeResult.Failure($"Unsupported target language: {targetLanguage}");
            }
            
            return GlueCodeResult.Success(sb.ToString(), $"pony_glue_{targetLanguage}.cs", targetLanguage);
        }
        catch (Exception ex)
        {
            return GlueCodeResult.Failure($"Failed to generate glue code: {ex.Message}");
        }
    }

    private string MapPonyTypeToC(string ponyType)
    {
        return ponyType switch
        {
            "I32" => "int32_t",
            "I64" => "int64_t",
            "U32" => "uint32_t",
            "U64" => "uint64_t",
            "F32" => "float",
            "F64" => "double",
            "Bool" => "bool",
            "String" => "char*",
            _ => "void*"
        };
    }

    private string MapPonyTypeToCSharp(string ponyType)
    {
        return ponyType switch
        {
            "I32" => "int",
            "I64" => "long",
            "U32" => "uint",
            "U64" => "ulong",
            "F32" => "float",
            "F64" => "double",
            "Bool" => "bool",
            "String" => "string",
            _ => "IntPtr"
        };
    }
}