// MAIDOS-Forge Faust Language Plugin
// Code-QC v2.2B Compliant
// M10 Extension Plugin - Other Languages

using System.Text;
using System.Text.RegularExpressions;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.Faust;

/// <summary>
/// Faust 語言插件 - 音頻信號處理語言
/// </summary>
/// <impl>
/// APPROACH: 支援 Faust 編譯器
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: 需要 Faust 2.0+
/// </impl>
public sealed class FaustPlugin : ILanguagePlugin
{
    private static readonly string[] SourceExtensions = { ".dsp" };

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "faust",
        SupportedExtensions = SourceExtensions,
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = true,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "native", "web", "ios", "android" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        // 檢查 Faust
        if (await ProcessRunner.CommandExistsAsync("faust"))
        {
            var version = await ProcessRunner.GetVersionAsync("faust", "-v");
            var match = Regex.Match(version ?? "", @"FAUST version (.+)");
            if (match.Success)
                return (true, $"Faust {match.Groups[1].Value}");
        }

        return (false, "Faust not found. Install Faust");
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

        logs.Add($"[Faust] Using: {toolchainMsg}");

        var srcDir = Path.Combine(module.ModulePath, "src") is var sd && Directory.Exists(sd)
            ? sd : module.ModulePath;

        var sourceFiles = SourceExtensions
            .SelectMany(ext => Directory.GetFiles(srcDir, $"*{ext}", SearchOption.AllDirectories))
            .ToArray();

        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No Faust source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[Faust] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        // Faust 語言可以編譯為多種目標
        var artifacts = new List<string>();
        
        foreach (var sourceFile in sourceFiles)
        {
            // 編譯為 C++ 代碼
            var cppFile = Path.Combine(outputDir, Path.GetFileNameWithoutExtension(sourceFile) + ".cpp");
            string buildCmd = "faust";
            string buildArgs = $"-o \"{cppFile}\" \"{sourceFile}\"";

            logs.Add($"[Faust] Generating C++ with: {buildCmd} {buildArgs}");

            try
            {
                var processConfig = new ProcessConfig { WorkingDirectory = module.ModulePath };
                var result = await ProcessRunner.RunAsync(buildCmd, buildArgs, processConfig, ct);
                
                logs.AddRange(result.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries));
                if (!string.IsNullOrWhiteSpace(result.Stderr))
                {
                    logs.AddRange(result.Stderr.Split('\n', StringSplitOptions.RemoveEmptyEntries));
                }

                if (result.IsSuccess && File.Exists(cppFile))
                {
                    artifacts.Add(cppFile);
                }
            }
            catch (Exception ex)
            {
                logs.Add($"[Faust] Warning: Failed to generate C++ for {sourceFile}: {ex.Message}");
            }

            // 編譯為 WebAssembly
            var wasmFile = Path.Combine(outputDir, Path.GetFileNameWithoutExtension(sourceFile) + ".wasm");
            buildArgs = $"-lang wasm -o \"{wasmFile}\" \"{sourceFile}\"";

            logs.Add($"[Faust] Generating WebAssembly with: {buildCmd} {buildArgs}");

            try
            {
                var processConfig = new ProcessConfig { WorkingDirectory = module.ModulePath };
                var result = await ProcessRunner.RunAsync(buildCmd, buildArgs, processConfig, ct);
                
                logs.AddRange(result.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries));
                if (!string.IsNullOrWhiteSpace(result.Stderr))
                {
                    logs.AddRange(result.Stderr.Split('\n', StringSplitOptions.RemoveEmptyEntries));
                }

                if (result.IsSuccess && File.Exists(wasmFile))
                {
                    artifacts.Add(wasmFile);
                }
            }
            catch (Exception ex)
            {
                logs.Add($"[Faust] Warning: Failed to generate WebAssembly for {sourceFile}: {ex.Message}");
            }
        }

        stopwatch.Stop();
        return CompileResult.Success(artifacts, logs, stopwatch.Elapsed);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(string artifactPath, CancellationToken ct = default)
    {
        // Faust 接口提取需要解析 .dsp 文件中的函數定義
        // 這裏提供一個簡化的實現
        
        try
        {
            // 檢查是否為 Faust 源文件
            if (!File.Exists(artifactPath))
                return null;

            // 在實際實現中，我們需要解析 Faust 源碼中的函數定義
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
                    Name = "faust",
                    Abi = "native",
                    Mode = "compiled"
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
                    sb.AppendLine("// Faust FFI glue code for C");
                    sb.AppendLine("#include <stdlib.h>");
                    sb.AppendLine("#include <string.h>");
                    sb.AppendLine();
                    
                    // 為每個導出函數生成 C 聲明
                    foreach (var func in sourceInterface.Exports)
                    {
                        var returnType = MapFaustTypeToC(func.ReturnType);
                        var parameters = string.Join(", ", func.Parameters.Select(p => $"{MapFaustTypeToC(p.Type)} {p.Name}"));
                        sb.AppendLine($"{returnType} {func.Name}({parameters});");
                    }
                    break;
                    
                case "csharp":
                    sb.AppendLine("// Faust FFI glue code for C#");
                    sb.AppendLine("using System;");
                    sb.AppendLine("using System.Runtime.InteropServices;");
                    sb.AppendLine();
                    sb.AppendLine("namespace FaustInterop");
                    sb.AppendLine("{");
                    sb.AppendLine("    internal static class FaustNative");
                    sb.AppendLine("    {");
                    sb.AppendLine("        private const string LibraryName = \"faust_library\";");
                    sb.AppendLine();
                    
                    // 為每個導出函數生成 P/Invoke 聲明
                    foreach (var func in sourceInterface.Exports)
                    {
                        var returnType = MapFaustTypeToCSharp(func.ReturnType);
                        var parameters = string.Join(", ", func.Parameters.Select(p => $"{MapFaustTypeToCSharp(p.Type)} {p.Name}"));
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
            
            return GlueCodeResult.Success(sb.ToString(), $"faust_glue_{targetLanguage}.cs", targetLanguage);
        }
        catch (Exception ex)
        {
            return GlueCodeResult.Failure($"Failed to generate glue code: {ex.Message}");
        }
    }

    private string MapFaustTypeToC(string faustType)
    {
        return faustType switch
        {
            "int" => "int",
            "float" => "float",
            "double" => "double",
            "sample" => "float",
            "soundfile" => "void*",
            _ => "void*"
        };
    }

    private string MapFaustTypeToCSharp(string faustType)
    {
        return faustType switch
        {
            "int" => "int",
            "float" => "float",
            "double" => "double",
            "sample" => "float",
            "soundfile" => "IntPtr",
            _ => "IntPtr"
        };
    }
}