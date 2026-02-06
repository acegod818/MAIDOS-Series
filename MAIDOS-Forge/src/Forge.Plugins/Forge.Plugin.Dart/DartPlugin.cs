// MAIDOS-Forge Dart Language Plugin
// UEP v1.7C Compliant - Zero Technical Debt
// Standalone Plugin Module

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.Dart;

/// <summary>
/// Dart Language Plugin - Supports dart compile exe for native compilation
/// </summary>
/// <impl>
/// APPROACH: Run dart compile exe to produce native executables
/// CALLS: ProcessRunner.RunAsync(), ProcessRunner.CommandExistsAsync()
/// EDGES: No dart SDK found returns error
/// </impl>
public sealed class DartPlugin : ILanguagePlugin
{
    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "dart",
        SupportedExtensions = new[] { ".dart" },
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (await ProcessRunner.CommandExistsAsync("dart"))
        {
            var version = await ProcessRunner.GetVersionAsync("dart", "--version");
            return (true, $"dart {version}");
        }

        return (false, "dart not found. Install Dart SDK from https://dart.dev/get-dart");
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

        logs.Add($"[Dart] Using: {toolchainMsg}");

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        // Check for pubspec.yaml (Dart project marker)
        var pubspecPath = Path.Combine(module.ModulePath, "pubspec.yaml");
        var hasPubspec = File.Exists(pubspecPath);
        if (hasPubspec)
        {
            logs.Add("[Dart] Found pubspec.yaml - running pub get first");
            var pubGetResult = await ProcessRunner.RunAsync(
                "dart", "pub get",
                new ProcessConfig
                {
                    WorkingDirectory = module.ModulePath,
                    Timeout = TimeSpan.FromMinutes(3)
                }, ct);

            if (!string.IsNullOrEmpty(pubGetResult.Stdout))
            {
                logs.Add(pubGetResult.Stdout.Trim());
            }

            if (!pubGetResult.IsSuccess)
            {
                logs.Add($"[Dart] Warning: pub get failed: {pubGetResult.Stderr}");
            }
        }

        var sourceFiles = Directory.GetFiles(srcDir, "*.dart", SearchOption.AllDirectories);
        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No .dart source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[Dart] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var artifacts = new List<string>();

        // Find main entry point: prefer bin/main.dart or lib/main.dart or first .dart file
        var mainFile = FindMainDartFile(sourceFiles, module.ModulePath);
        logs.Add($"[Dart] Entry point: {Path.GetFileName(mainFile)}");

        // First run dart analyze for diagnostics (advisory, non-blocking)
        var analyzeResult = await ProcessRunner.RunAsync(
            "dart", $"analyze \"{srcDir}\"",
            new ProcessConfig
            {
                WorkingDirectory = module.ModulePath,
                Timeout = TimeSpan.FromMinutes(2)
            }, ct);

        if (!string.IsNullOrEmpty(analyzeResult.Stdout))
        {
            foreach (var line in analyzeResult.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries))
            {
                logs.Add($"[analyze] {line}");
            }
        }

        // Compile to native executable
        var exeName = OperatingSystem.IsWindows()
            ? $"{module.Config.Name}.exe"
            : module.Config.Name;
        var outputPath = Path.Combine(outputDir, exeName);

        var compileArgs = $"compile exe \"{mainFile}\" -o \"{outputPath}\"";

        if (config.Profile == "release")
        {
            // Dart compile exe is already AOT; no separate release flag needed
            logs.Add("[Dart] Compiling in release mode (AOT)");
        }
        else
        {
            logs.Add("[Dart] Compiling in debug mode (AOT)");
        }

        logs.Add($"$ dart {compileArgs}");

        var compileResult = await ProcessRunner.RunAsync(
            "dart", compileArgs,
            new ProcessConfig
            {
                WorkingDirectory = module.ModulePath,
                Timeout = TimeSpan.FromMinutes(10)
            }, ct);

        if (!string.IsNullOrEmpty(compileResult.Stdout))
        {
            foreach (var line in compileResult.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries))
            {
                logs.Add($"[dart] {line}");
            }
        }

        if (!string.IsNullOrEmpty(compileResult.Stderr))
        {
            foreach (var line in compileResult.Stderr.Split('\n', StringSplitOptions.RemoveEmptyEntries))
            {
                logs.Add($"[dart] {line}");
            }
        }

        if (!compileResult.IsSuccess)
        {
            stopwatch.Stop();
            return CompileResult.Failure(
                $"Dart compilation failed: {compileResult.Stderr}",
                logs, stopwatch.Elapsed);
        }

        if (File.Exists(outputPath))
        {
            artifacts.Add(outputPath);
        }

        stopwatch.Stop();
        logs.Add($"[Dart] Build succeeded, {artifacts.Count} artifact(s)");

        return CompileResult.Success(artifacts.ToArray(), logs, stopwatch.Elapsed);
    }

    private static string FindMainDartFile(string[] sourceFiles, string modulePath)
    {
        // Priority: bin/main.dart > lib/main.dart > any main.dart > first .dart
        var binMain = sourceFiles.FirstOrDefault(f =>
            f.Replace('\\', '/').Contains("/bin/main.dart", StringComparison.OrdinalIgnoreCase));
        if (binMain != null) return binMain;

        var libMain = sourceFiles.FirstOrDefault(f =>
            f.Replace('\\', '/').Contains("/lib/main.dart", StringComparison.OrdinalIgnoreCase));
        if (libMain != null) return libMain;

        var anyMain = sourceFiles.FirstOrDefault(f =>
            Path.GetFileName(f).Equals("main.dart", StringComparison.OrdinalIgnoreCase));
        if (anyMain != null) return anyMain;

        return sourceFiles[0];
    }

    public Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        // For Dart source files, parse top-level function declarations
        if (artifactPath.EndsWith(".dart", StringComparison.OrdinalIgnoreCase) && File.Exists(artifactPath))
        {
            var content = File.ReadAllText(artifactPath);
            var lines = content.Split('\n');

            foreach (var rawLine in lines)
            {
                var line = rawLine.Trim();

                // Match top-level function: returnType functionName(...)
                // Skip class methods (indented), skip imports/comments
                if (line.StartsWith("//") || line.StartsWith("import ") ||
                    line.StartsWith("class ") || line.StartsWith("abstract "))
                    continue;

                // Pattern: type name(params) { or type name(params) =>
                var funcMatch = ExtractDartFunction(line);
                if (funcMatch != null)
                {
                    exports.Add(funcMatch);
                }
            }
        }

        var moduleName = Path.GetFileNameWithoutExtension(artifactPath);

        return Task.FromResult<InterfaceDescription?>(new InterfaceDescription
        {
            Version = "1.0",
            Module = new InterfaceModule
            {
                Name = moduleName,
                Version = "1.0.0"
            },
            Language = new InterfaceLanguage
            {
                Name = "dart",
                Abi = "native"
            },
            Exports = exports.ToArray()
        });
    }

    private static ExportedFunction? ExtractDartFunction(string line)
    {
        // Detect patterns like: void main(...) {, int calculate(...) {, String getName(...) =>
        var dartTypes = new[] { "void", "int", "double", "String", "bool", "List", "Map", "Future", "dynamic" };

        foreach (var dt in dartTypes)
        {
            if (!line.StartsWith(dt + " ", StringComparison.Ordinal)) continue;

            var rest = line.Substring(dt.Length + 1).Trim();
            var parenIdx = rest.IndexOf('(');
            if (parenIdx <= 0) continue;

            var funcName = rest.Substring(0, parenIdx).Trim();
            if (string.IsNullOrEmpty(funcName) || funcName.Contains(' ')) continue;

            return new ExportedFunction
            {
                Name = funcName,
                ReturnType = MapDartTypeToForge(dt),
                Parameters = Array.Empty<FunctionParameter>()
            };
        }

        return null;
    }

    private static string MapDartTypeToForge(string dartType) => dartType switch
    {
        "void" => "void",
        "int" => "i64",
        "double" => "f64",
        "bool" => "bool",
        "String" => "string",
        _ => "void"
    };

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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Dart Plugin");
        sb.AppendLine("// Dart AOT native interop via P/Invoke");
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Dart Plugin");
        sb.AppendLine("// Dart AOT native interop via FFI");
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
        "string" => "string",
        "bool" => "bool",
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
        "string" => "*const std::os::raw::c_char",
        "bool" => "bool",
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
