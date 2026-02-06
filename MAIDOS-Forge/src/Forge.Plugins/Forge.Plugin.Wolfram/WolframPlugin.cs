// MAIDOS-Forge Wolfram Language Plugin
// Code-QC v2.2B Compliant
// M10 Extension Plugin - Scientific Languages

using System.Text;
using System.Text.RegularExpressions;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.Wolfram;

/// <summary>
/// Wolfram 語言插件 - 科學計算語言
/// </summary>
/// <impl>
/// APPROACH: 支援 wolframscript 工具
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: 需要 Wolfram Engine 或 Mathematica
/// </impl>
public sealed class WolframPlugin : ILanguagePlugin
{
    private static readonly string[] SourceExtensions = { ".wl", ".wls" };

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "wolfram",
        SupportedExtensions = SourceExtensions,
        SupportsNativeCompilation = false,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        // 檢查 wolframscript
        if (await ProcessRunner.CommandExistsAsync("wolframscript"))
        {
            var version = await ProcessRunner.GetVersionAsync("wolframscript", "--version");
            return (true, $"wolframscript {version ?? "unknown version"}");
        }

        return (false, "Wolfram not found. Install Wolfram Engine or Mathematica");
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

        logs.Add($"[Wolfram] Using: {toolchainMsg}");

        var srcDir = Path.Combine(module.ModulePath, "src") is var sd && Directory.Exists(sd)
            ? sd : module.ModulePath;

        var sourceFiles = SourceExtensions
            .SelectMany(ext => Directory.GetFiles(srcDir, $"*{ext}", SearchOption.AllDirectories))
            .ToArray();

        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No Wolfram source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[Wolfram] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        // Wolfram 語言通常不需要傳統的編譯，而是解釋執行或打包
        // 這裏我們創建一個簡單的打包過程
        var artifacts = new List<string>();
        
        foreach (var sourceFile in sourceFiles)
        {
            var destFile = Path.Combine(outputDir, Path.GetFileName(sourceFile));
            File.Copy(sourceFile, destFile, true);
            artifacts.Add(destFile);
        }

        stopwatch.Stop();
        return CompileResult.Success(artifacts, logs, stopwatch.Elapsed);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(string artifactPath, CancellationToken ct = default)
    {
        // Wolfram 接口提取需要解析 .wl 文件中的函數定義
        // 這裏提供一個簡化的實現
        
        try
        {
            // 檢查是否為 Wolfram 源文件
            if (!File.Exists(artifactPath))
                return null;

            // 在實際實現中，我們需要解析 Wolfram 源碼中的函數定義
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
                    Name = "wolfram",
                    Abi = "native",
                    Mode = "interpreted"
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
                    sb.AppendLine("// Wolfram FFI glue code for C");
                    sb.AppendLine("#include <wolframlibrary.h>");
                    sb.AppendLine();
                    
                    // 為每個導出函數生成 C 聲明
                    foreach (var func in sourceInterface.Exports)
                    {
                        var returnType = MapWolframTypeToC(func.ReturnType);
                        var parameters = string.Join(", ", func.Parameters.Select(p => $"{MapWolframTypeToC(p.Type)} {p.Name}"));
                        sb.AppendLine($"DLLEXPORT {returnType} {func.Name}({parameters});");
                    }
                    break;
                    
                case "csharp":
                    sb.AppendLine("// Wolfram FFI glue code for C#");
                    sb.AppendLine("using System;");
                    sb.AppendLine("using System.Runtime.InteropServices;");
                    sb.AppendLine();
                    sb.AppendLine("namespace WolframInterop");
                    sb.AppendLine("{");
                    sb.AppendLine("    internal static class WolframNative");
                    sb.AppendLine("    {");
                    sb.AppendLine("        private const string LibraryName = \"wolfram_library\";");
                    sb.AppendLine();
                    
                    // 為每個導出函數生成 P/Invoke 聲明
                    foreach (var func in sourceInterface.Exports)
                    {
                        var returnType = MapWolframTypeToCSharp(func.ReturnType);
                        var parameters = string.Join(", ", func.Parameters.Select(p => $"{MapWolframTypeToCSharp(p.Type)} {p.Name}"));
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
            
            return GlueCodeResult.Success(sb.ToString(), $"wolfram_glue_{targetLanguage}.cs", targetLanguage);
        }
        catch (Exception ex)
        {
            return GlueCodeResult.Failure($"Failed to generate glue code: {ex.Message}");
        }
    }

    private string MapWolframTypeToC(string wolframType)
    {
        return wolframType switch
        {
            "Integer" => "mint",
            "Real" => "mreal",
            "Complex" => "mcomplex",
            "String" => "char*",
            _ => "MTensor"
        };
    }

    private string MapWolframTypeToCSharp(string wolframType)
    {
        return wolframType switch
        {
            "Integer" => "long",
            "Real" => "double",
            "Complex" => "System.Numerics.Complex",
            "String" => "string",
            _ => "IntPtr"
        };
    }
}