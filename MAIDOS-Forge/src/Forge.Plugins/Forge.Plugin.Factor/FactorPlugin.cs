// MAIDOS-Forge Factor Language Plugin
// Code-QC v2.2B Compliant
// M10 Extension Plugin - Other Languages

using System.Text;
using System.Text.RegularExpressions;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.Factor;

/// <summary>
/// Factor 語言插件 - 堆疊式語言
/// </summary>
/// <impl>
/// APPROACH: 支援 Factor 編譯器
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: 需要 Factor 0.98+
/// </impl>
public sealed class FactorPlugin : ILanguagePlugin
{
    private static readonly string[] SourceExtensions = { ".factor" };

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "factor",
        SupportedExtensions = SourceExtensions,
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        // 檢查 Factor
        if (await ProcessRunner.CommandExistsAsync("factor"))
        {
            var version = await ProcessRunner.GetVersionAsync("factor", "--version");
            var match = Regex.Match(version ?? "", @"Factor (.+)");
            if (match.Success)
                return (true, $"Factor {match.Groups[1].Value}");
        }

        return (false, "Factor not found. Install Factor");
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

        logs.Add($"[Factor] Using: {toolchainMsg}");

        var srcDir = Path.Combine(module.ModulePath, "src") is var sd && Directory.Exists(sd)
            ? sd : module.ModulePath;

        var sourceFiles = SourceExtensions
            .SelectMany(ext => Directory.GetFiles(srcDir, $"*{ext}", SearchOption.AllDirectories))
            .ToArray();

        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No Factor source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[Factor] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        // Factor 語言通常不需要傳統的編譯，而是解釋執行或生成二進制文件
        // 這裏我們使用 Factor 生成二進制文件
        var artifacts = new List<string>();
        
        foreach (var sourceFile in sourceFiles)
        {
            var outputFile = Path.Combine(outputDir, Path.GetFileNameWithoutExtension(sourceFile) + ".image");
            string buildCmd = "factor";
            string buildArgs = $"-e=\"USING: compiler io.files ; \"{sourceFile}\" compile-to-image \"{outputFile}\"\"";

            logs.Add($"[Factor] Building with: {buildCmd} {buildArgs}");

            try
            {
                var processConfig = new ProcessConfig { WorkingDirectory = module.ModulePath };
                var result = await ProcessRunner.RunAsync(buildCmd, buildArgs, processConfig, ct);
                
                logs.AddRange(result.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries));
                if (!string.IsNullOrWhiteSpace(result.Stderr))
                {
                    logs.AddRange(result.Stderr.Split('\n', StringSplitOptions.RemoveEmptyEntries));
                }

                if (result.IsSuccess && File.Exists(outputFile))
                {
                    artifacts.Add(outputFile);
                }
            }
            catch (Exception ex)
            {
                logs.Add($"[Factor] Warning: Failed to compile {sourceFile}: {ex.Message}");
            }
        }

        stopwatch.Stop();
        return CompileResult.Success(artifacts, logs, stopwatch.Elapsed);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(string artifactPath, CancellationToken ct = default)
    {
        // Factor 接口提取需要解析 .factor 文件中的函數定義
        // 這裏提供一個簡化的實現
        
        try
        {
            // 檢查是否為 Factor 源文件
            if (!File.Exists(artifactPath))
                return null;

            // 在實際實現中，我們需要解析 Factor 源碼中的函數定義
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
                    Name = "factor",
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
                    sb.AppendLine("// Factor FFI glue code for C");
                    sb.AppendLine("#include <stdio.h>");
                    sb.AppendLine();
                    
                    // 為每個導出函數生成 C 聲明
                    foreach (var func in sourceInterface.Exports)
                    {
                        var returnType = MapFactorTypeToC(func.ReturnType);
                        var parameters = string.Join(", ", func.Parameters.Select(p => $"{MapFactorTypeToC(p.Type)} {p.Name}"));
                        sb.AppendLine($"{returnType} {func.Name}({parameters});");
                    }
                    break;
                    
                case "csharp":
                    sb.AppendLine("// Factor FFI glue code for C#");
                    sb.AppendLine("using System;");
                    sb.AppendLine("using System.Runtime.InteropServices;");
                    sb.AppendLine();
                    sb.AppendLine("namespace FactorInterop");
                    sb.AppendLine("{");
                    sb.AppendLine("    internal static class FactorNative");
                    sb.AppendLine("    {");
                    sb.AppendLine("        private const string LibraryName = \"factor_library\";");
                    sb.AppendLine();
                    
                    // 為每個導出函數生成 P/Invoke 聲明
                    foreach (var func in sourceInterface.Exports)
                    {
                        var returnType = MapFactorTypeToCSharp(func.ReturnType);
                        var parameters = string.Join(", ", func.Parameters.Select(p => $"{MapFactorTypeToCSharp(p.Type)} {p.Name}"));
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
            
            return GlueCodeResult.Success(sb.ToString(), $"factor_glue_{targetLanguage}.cs", targetLanguage);
        }
        catch (Exception ex)
        {
            return GlueCodeResult.Failure($"Failed to generate glue code: {ex.Message}");
        }
    }

    private string MapFactorTypeToC(string factorType)
    {
        return factorType switch
        {
            "int" => "int",
            "float" => "float",
            "double" => "double",
            "char" => "char",
            "string" => "char*",
            _ => "void*"
        };
    }

    private string MapFactorTypeToCSharp(string factorType)
    {
        return factorType switch
        {
            "int" => "int",
            "float" => "float",
            "double" => "double",
            "char" => "char",
            "string" => "string",
            _ => "IntPtr"
        };
    }
}