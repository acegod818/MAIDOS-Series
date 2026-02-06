// MAIDOS-Forge Elixir Language Plugin
// UEP v1.7C Compliant - Zero Technical Debt
// Standalone Plugin Module

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.Elixir;

/// <summary>
/// Elixir Language Plugin - Compiles to BEAM bytecode via elixirc or mix
/// </summary>
/// <impl>
/// APPROACH: Run elixirc for direct compilation or mix build for projects
/// CALLS: ProcessRunner.RunAsync(), ProcessRunner.CommandExistsAsync()
/// EDGES: Neither elixirc nor mix found returns error
/// </impl>
public sealed class ElixirPlugin : ILanguagePlugin
{
    private string _compiler = "elixirc";
    private bool _useMix;

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "elixir",
        SupportedExtensions = new[] { ".ex", ".exs" },
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (await ProcessRunner.CommandExistsAsync("elixirc"))
        {
            var version = await ProcessRunner.GetVersionAsync("elixir", "--version");
            _compiler = "elixirc";
            _useMix = false;

            var hasMix = await ProcessRunner.CommandExistsAsync("mix");
            var extra = hasMix ? " (mix available)" : "";

            return (true, $"elixir {version}{extra}");
        }

        if (await ProcessRunner.CommandExistsAsync("mix"))
        {
            var version = await ProcessRunner.GetVersionAsync("elixir", "--version");
            _compiler = "mix";
            _useMix = true;
            return (true, $"elixir {version} (mix only)");
        }

        return (false, "elixirc not found. Install Elixir from https://elixir-lang.org/install.html");
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

        logs.Add($"[Elixir] Using: {toolchainMsg}");

        // Mix project detection
        var mixExs = Path.Combine(module.ModulePath, "mix.exs");
        if (_useMix || File.Exists(mixExs))
        {
            return await CompileWithMixAsync(module, config, logs, stopwatch, ct);
        }

        // Direct elixirc compilation
        var srcDir = Path.Combine(module.ModulePath, "lib");
        if (!Directory.Exists(srcDir))
        {
            srcDir = Path.Combine(module.ModulePath, "src");
        }
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var sourceFiles = Directory.GetFiles(srcDir, "*.ex", SearchOption.AllDirectories)
            .Concat(Directory.GetFiles(srcDir, "*.exs", SearchOption.AllDirectories))
            .ToArray();

        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No .ex or .exs source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[Elixir] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var beamDir = Path.Combine(outputDir, "ebin");
        Directory.CreateDirectory(beamDir);

        foreach (var sourceFile in sourceFiles)
        {
            var fileName = Path.GetFileName(sourceFile);
            var args = $"\"{sourceFile}\" --dest \"{beamDir}\"";

            if (config.Profile == "debug")
            {
                args += " --debug-info";
            }

            logs.Add($"$ elixirc {args}");

            var result = await ProcessRunner.RunAsync(
                "elixirc", args,
                new ProcessConfig
                {
                    WorkingDirectory = module.ModulePath,
                    Timeout = TimeSpan.FromMinutes(5)
                }, ct);

            if (!string.IsNullOrEmpty(result.Stdout))
            {
                logs.Add(result.Stdout);
            }

            if (!string.IsNullOrEmpty(result.Stderr))
            {
                logs.Add(result.Stderr);
            }

            if (!result.IsSuccess)
            {
                stopwatch.Stop();
                return CompileResult.Failure(
                    $"Compilation failed for {fileName}: {result.Stderr}",
                    logs, stopwatch.Elapsed);
            }
        }

        var artifacts = Directory.Exists(beamDir)
            ? Directory.GetFiles(beamDir, "*.beam", SearchOption.AllDirectories).ToList()
            : new List<string>();

        stopwatch.Stop();
        logs.Add($"[Elixir] Build succeeded, {artifacts.Count} BEAM artifact(s)");
        return CompileResult.Success(artifacts.ToArray(), logs, stopwatch.Elapsed);
    }

    private async Task<CompileResult> CompileWithMixAsync(
        ValidatedModuleConfig module,
        CompileConfig config,
        List<string> logs,
        System.Diagnostics.Stopwatch stopwatch,
        CancellationToken ct)
    {
        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        // Fetch dependencies
        logs.Add("[Elixir] Fetching dependencies...");
        var depsResult = await ProcessRunner.RunAsync(
            "mix", "deps.get",
            new ProcessConfig
            {
                WorkingDirectory = module.ModulePath,
                Timeout = TimeSpan.FromMinutes(5)
            }, ct);

        if (!depsResult.IsSuccess)
        {
            logs.Add($"[Elixir] Warning: mix deps.get failed: {depsResult.Stderr}");
        }

        var mixEnv = config.Profile == "debug" ? "dev" : "prod";
        var envVars = new Dictionary<string, string>(config.Environment ?? new Dictionary<string, string>())
        {
            ["MIX_ENV"] = mixEnv
        };

        logs.Add($"$ MIX_ENV={mixEnv} mix compile --force");

        var result = await ProcessRunner.RunAsync(
            "mix", "compile --force",
            new ProcessConfig
            {
                WorkingDirectory = module.ModulePath,
                Environment = envVars,
                Timeout = TimeSpan.FromMinutes(10)
            }, ct);

        if (!string.IsNullOrEmpty(result.Stdout))
        {
            logs.Add(result.Stdout);
        }

        if (!string.IsNullOrEmpty(result.Stderr))
        {
            logs.Add(result.Stderr);
        }

        if (!result.IsSuccess)
        {
            stopwatch.Stop();
            return CompileResult.Failure($"mix compile failed: {result.Stderr}", logs, stopwatch.Elapsed);
        }

        // Collect BEAM artifacts from _build
        var artifacts = new List<string>();
        var buildDir = Path.Combine(module.ModulePath, "_build", mixEnv, "lib");
        if (Directory.Exists(buildDir))
        {
            foreach (var beamFile in Directory.GetFiles(buildDir, "*.beam", SearchOption.AllDirectories))
            {
                var relativePath = Path.GetRelativePath(buildDir, beamFile);
                var destPath = Path.Combine(outputDir, relativePath);
                var destDir = Path.GetDirectoryName(destPath);
                if (!string.IsNullOrEmpty(destDir)) Directory.CreateDirectory(destDir);

                try
                {
                    File.Copy(beamFile, destPath, overwrite: true);
                    artifacts.Add(destPath);
                }
                catch (Exception ex)
                {
                    logs.Add($"[Elixir] Warning: copy failed: {ex.Message}");
                }
            }
        }

        stopwatch.Stop();
        logs.Add($"[Elixir] Mix build succeeded, {artifacts.Count} BEAM artifact(s)");
        return CompileResult.Success(artifacts.ToArray(), logs, stopwatch.Elapsed);
    }

    public Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        if ((artifactPath.EndsWith(".ex", StringComparison.OrdinalIgnoreCase) ||
             artifactPath.EndsWith(".exs", StringComparison.OrdinalIgnoreCase)) &&
            File.Exists(artifactPath))
        {
            var lines = File.ReadAllLines(artifactPath);
            string? currentModule = null;

            foreach (var rawLine in lines)
            {
                var line = rawLine.Trim();
                if (line.StartsWith("#")) continue;

                // Detect module: defmodule ModuleName do
                if (line.StartsWith("defmodule ", StringComparison.Ordinal))
                {
                    var rest = line.Substring(10).Trim();
                    var doIdx = rest.IndexOf(" do", StringComparison.Ordinal);
                    currentModule = doIdx >= 0 ? rest.Substring(0, doIdx).Trim() : rest;
                    continue;
                }

                // Detect public functions: def function_name(...)
                if (line.StartsWith("def ", StringComparison.Ordinal) &&
                    !line.StartsWith("defp ", StringComparison.Ordinal) &&
                    !line.StartsWith("defmodule ", StringComparison.Ordinal) &&
                    !line.StartsWith("defmacro ", StringComparison.Ordinal))
                {
                    var funcPart = line.Substring(4).Trim();
                    var parenIdx = funcPart.IndexOf('(');
                    var doIdx = funcPart.IndexOf(" do", StringComparison.Ordinal);

                    string funcName;
                    if (parenIdx >= 0) funcName = funcPart.Substring(0, parenIdx).Trim();
                    else if (doIdx >= 0) funcName = funcPart.Substring(0, doIdx).Trim();
                    else funcName = funcPart.Split(' ', StringSplitOptions.RemoveEmptyEntries)[0];

                    if (!string.IsNullOrEmpty(funcName))
                    {
                        var qualifiedName = currentModule != null
                            ? $"{currentModule}.{funcName}" : funcName;

                        exports.Add(new ExportedFunction
                        {
                            Name = qualifiedName,
                            ReturnType = "void",
                            Parameters = Array.Empty<FunctionParameter>()
                        });
                    }
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
                Name = "elixir",
                Abi = "beam"
            },
            Exports = exports.ToArray()
        });
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Elixir Plugin");
        sb.AppendLine("// Elixir/BEAM interop via Port or NIF bridge");
        sb.AppendLine("using System.Diagnostics;");
        sb.AppendLine("using System.Runtime.InteropServices;");
        sb.AppendLine();
        sb.AppendLine($"namespace {pascalName}.Interop;");
        sb.AppendLine();
        sb.AppendLine($"internal static unsafe partial class {pascalName}Native");
        sb.AppendLine("{");
        sb.AppendLine($"    private const string ModuleName = \"{moduleName}\";");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var returnType = MapToCSharpType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{MapToCSharpType(p.Type)} {p.Name}"));

            sb.AppendLine($"    [DllImport(ModuleName, CallingConvention = CallingConvention.Cdecl)]");
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
        var snakeName = ToSnakeCase(moduleName);

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Elixir Plugin");
        sb.AppendLine("// Elixir/BEAM interop via Rustler NIF");
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

        return GlueCodeResult.Success(sb.ToString(), $"{snakeName}_ffi.rs", "rust");
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
        _ => "object"
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
        "string" => "String",
        "bool" => "bool",
        _ => "i32"
    };

    private static string ToPascalCase(string s)
    {
        if (string.IsNullOrEmpty(s)) return s;
        var parts = s.Split('-', '_', '.');
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
