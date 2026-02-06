// MAIDOS-Forge C Language Plugin
// UEP v1.7C Compliant - Zero Technical Debt
// Standalone Plugin Module

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.C;

/// <summary>
/// C 語言插件 - 支援 clang/gcc 編譯
/// </summary>
/// <impl>
/// APPROACH: 調用 clang 或 gcc 編譯 C 源碼
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: 無編譯器時返回錯誤
/// </impl>
public sealed class CLanguagePlugin : ILanguagePlugin
{
    private string _compiler = "clang";

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "c",
        SupportedExtensions = new[] { ".c", ".h" },
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = true,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "linux", "windows", "macos", "freebsd" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (await ProcessRunner.CommandExistsAsync("clang"))
        {
            var version = await ProcessRunner.GetVersionAsync("clang", "--version");
            _compiler = "clang";
            return (true, $"clang {version}");
        }

        if (await ProcessRunner.CommandExistsAsync("gcc"))
        {
            var version = await ProcessRunner.GetVersionAsync("gcc", "--version");
            _compiler = "gcc";
            return (true, $"gcc {version}");
        }

        return (false, "Neither clang nor gcc found");
    }

    public async Task<CompileResult> CompileAsync(
        ValidatedModuleConfig module,
        CompileConfig config,
        CancellationToken ct = default)
    {
        var logs = new List<string>();
        var stopwatch = System.Diagnostics.Stopwatch.StartNew();

        var (available, toolchainMsg) = await ValidateToolchainAsync(ct);
        if (!available)
        {
            stopwatch.Stop();
            return CompileResult.Failure(toolchainMsg, logs, stopwatch.Elapsed);
        }

        logs.Add($"[C] Using: {toolchainMsg}");

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var sourceFiles = Directory.GetFiles(srcDir, "*.c", SearchOption.AllDirectories);
        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No .c source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[C] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var cConfig = module.Config.C ?? new CConfig();
        var objectFiles = new List<string>();

        foreach (var sourceFile in sourceFiles)
        {
            var objFile = Path.Combine(outputDir,
                Path.GetFileNameWithoutExtension(sourceFile) + ".o");

            var args = BuildCompileArgs(sourceFile, objFile, cConfig, config);
            logs.Add($"$ {_compiler} {args}");

            var result = await ProcessRunner.RunAsync(
                _compiler, args,
                new ProcessConfig
                {
                    WorkingDirectory = module.ModulePath,
                    Timeout = TimeSpan.FromMinutes(5)
                }, ct);

            if (!string.IsNullOrEmpty(result.Stderr))
            {
                logs.Add(result.Stderr);
            }

            if (!result.IsSuccess)
            {
                stopwatch.Stop();
                return CompileResult.Failure(
                    $"Compilation failed for {Path.GetFileName(sourceFile)}: {result.Stderr}",
                    logs, stopwatch.Elapsed);
            }

            objectFiles.Add(objFile);
        }

        var libName = $"lib{module.Config.Name}.a";
        var libPath = Path.Combine(outputDir, libName);

        var arArgs = $"rcs \"{libPath}\" {string.Join(" ", objectFiles.Select(f => $"\"{f}\""))}";
        logs.Add($"$ ar {arArgs}");

        var arResult = await ProcessRunner.RunAsync("ar", arArgs,
            new ProcessConfig { Timeout = TimeSpan.FromMinutes(2) }, ct);

        if (!arResult.IsSuccess)
        {
            logs.Add($"ar failed: {arResult.Stderr}, returning object files");
        }

        stopwatch.Stop();
        var artifacts = File.Exists(libPath) 
            ? new[] { libPath } 
            : objectFiles.ToArray();

        return CompileResult.Success(artifacts, logs, stopwatch.Elapsed);
    }

    private static string BuildCompileArgs(
        string sourceFile,
        string outputFile,
        CConfig cConfig,
        CompileConfig config)
    {
        var args = new List<string>
        {
            "-c",
            $"\"{sourceFile}\"",
            "-o",
            $"\"{outputFile}\""
        };

        var optLevel = config.Profile == "debug" ? "-O0" : "-O2";
        args.Add(optLevel);

        if (config.Profile == "debug")
        {
            args.Add("-g");
        }

        args.Add($"-std={cConfig.Standard}");
        args.Add("-Wall");
        args.Add("-Wextra");
        args.Add("-fPIC");

        foreach (var define in cConfig.Defines)
        {
            args.Add($"-D{define}");
        }

        foreach (var inc in cConfig.IncludeDirs)
        {
            args.Add($"-I\"{inc}\"");
        }

        return string.Join(" ", args);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        var nmResult = await ProcessRunner.RunAsync(
            "nm", $"-g --defined-only \"{artifactPath}\"",
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);

        if (nmResult.IsSuccess && !string.IsNullOrEmpty(nmResult.Stdout))
        {
            foreach (var line in nmResult.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries))
            {
                var parts = line.Split(' ', StringSplitOptions.RemoveEmptyEntries);
                if (parts.Length < 3) continue;

                var symbolType = parts[1];
                var symbolName = parts[2];

                if (symbolType != "T") continue;

                if (symbolName.StartsWith("_") && !symbolName.StartsWith("__"))
                {
                    symbolName = symbolName.TrimStart('_');
                }

                if (IsSystemSymbol(symbolName)) continue;

                exports.Add(new ExportedFunction
                {
                    Name = symbolName,
                    ReturnType = "i32",
                    Parameters = Array.Empty<FunctionParameter>()
                });
            }
        }

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
                Name = "c",
                Abi = "c"
            },
            Exports = exports.ToArray()
        };
    }

    private static bool IsSystemSymbol(string name)
    {
        var prefixes = new[] { "__", "_init", "_fini", "_start", "frame_dummy" };
        return prefixes.Any(p => name.StartsWith(p, StringComparison.Ordinal));
    }

    public GlueCodeResult GenerateGlue(InterfaceDescription sourceInterface, string targetLanguage)
    {
        return targetLanguage.ToLowerInvariant() switch
        {
            "csharp" or "c#" => GenerateCSharpGlue(sourceInterface),
            "rust" => GenerateRustGlue(sourceInterface),
            _ => GlueCodeResult.Failure($"Unsupported target language: {targetLanguage}")
        };
    }

    private static GlueCodeResult GenerateCSharpGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;
        var pascalName = ToPascalCase(moduleName);

        sb.AppendLine("// Auto-generated by MAIDOS-Forge C Plugin");
        sb.AppendLine("using System.Runtime.InteropServices;");
        sb.AppendLine();
        sb.AppendLine($"namespace {pascalName}.Interop;");
        sb.AppendLine();
        sb.AppendLine($"internal static unsafe partial class {pascalName}Native");
        sb.AppendLine("{");
        sb.AppendLine($"    private const string LibraryName = \"{moduleName}\";");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var returnType = MapToCSharpType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{MapToCSharpType(p.Type)} {p.Name}"));

            sb.AppendLine($"    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]");
            sb.AppendLine($"    public static extern {returnType} {export.Name}({parms});");
            sb.AppendLine();
        }

        sb.AppendLine("}");

        return GlueCodeResult.Success(sb.ToString(), $"{pascalName}.Interop.cs", "csharp");
    }

    private static GlueCodeResult GenerateRustGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;

        sb.AppendLine("// Auto-generated by MAIDOS-Forge C Plugin");
        sb.AppendLine("#![allow(non_snake_case)]");
        sb.AppendLine();
        sb.AppendLine($"#[link(name = \"{moduleName}\")]");
        sb.AppendLine("extern \"C\" {");

        foreach (var export in source.Exports)
        {
            var returnType = MapToRustType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{ToSnakeCase(p.Name)}: {MapToRustType(p.Type)}"));

            if (returnType == "()")
            {
                sb.AppendLine($"    pub fn {export.Name}({parms});");
            }
            else
            {
                sb.AppendLine($"    pub fn {export.Name}({parms}) -> {returnType};");
            }
        }

        sb.AppendLine("}");

        return GlueCodeResult.Success(sb.ToString(), $"{moduleName}_ffi.rs", "rust");
    }

    private static string MapToCSharpType(string t) => t switch
    {
        "void" => "void",
        "i8" => "sbyte",
        "i16" => "short",
        "i32" or "int" => "int",
        "i64" => "long",
        "u8" => "byte",
        "u16" => "ushort",
        "u32" => "uint",
        "u64" => "ulong",
        "f32" => "float",
        "f64" => "double",
        _ => "int"
    };

    private static string MapToRustType(string t) => t switch
    {
        "void" => "()",
        "i8" => "i8",
        "i16" => "i16",
        "i32" or "int" => "i32",
        "i64" => "i64",
        "u8" => "u8",
        "u16" => "u16",
        "u32" => "u32",
        "u64" => "u64",
        "f32" => "f32",
        "f64" => "f64",
        _ => "i32"
    };

    private static string ToPascalCase(string s)
    {
        if (string.IsNullOrEmpty(s)) return s;
        var parts = s.Split('-', '_');
        return string.Concat(parts.Select(p =>
            char.ToUpperInvariant(p[0]) + p.Substring(1).ToLowerInvariant()));
    }

    private static string ToSnakeCase(string s)
    {
        if (string.IsNullOrEmpty(s)) return s;
        var sb = new StringBuilder();
        foreach (var c in s)
        {
            if (char.IsUpper(c) && sb.Length > 0) sb.Append('_');
            sb.Append(char.ToLowerInvariant(c));
        }
        return sb.ToString();
    }
}
