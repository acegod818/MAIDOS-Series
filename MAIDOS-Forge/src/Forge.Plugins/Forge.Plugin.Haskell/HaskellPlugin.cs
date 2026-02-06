// MAIDOS-Forge Haskell Language Plugin
// UEP v1.7C Compliant - Zero Technical Debt
// Standalone Plugin Module

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.Haskell;

/// <summary>
/// Haskell Language Plugin - Supports ghc native compilation with stack fallback
/// </summary>
/// <impl>
/// APPROACH: Run ghc for direct compilation, fall back to stack build
/// CALLS: ProcessRunner.RunAsync(), ProcessRunner.CommandExistsAsync()
/// EDGES: Neither ghc nor stack found returns error
/// </impl>
public sealed class HaskellPlugin : ILanguagePlugin
{
    private string _compiler = "ghc";
    private bool _useStack;

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "haskell",
        SupportedExtensions = new[] { ".hs", ".lhs" },
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (await ProcessRunner.CommandExistsAsync("ghc"))
        {
            var version = await ProcessRunner.GetVersionAsync("ghc", "--version");
            _compiler = "ghc";
            _useStack = false;
            return (true, $"ghc {version}");
        }

        if (await ProcessRunner.CommandExistsAsync("stack"))
        {
            var version = await ProcessRunner.GetVersionAsync("stack", "--version");
            _compiler = "stack";
            _useStack = true;
            return (true, $"stack {version}");
        }

        return (false, "Neither ghc nor stack found. Install from https://www.haskell.org/ghcup/");
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

        logs.Add($"[Haskell] Using: {toolchainMsg}");

        // Stack project detection
        var stackYaml = Path.Combine(module.ModulePath, "stack.yaml");
        if (_useStack || File.Exists(stackYaml))
        {
            return await CompileWithStackAsync(module, config, logs, stopwatch, ct);
        }

        // Direct ghc compilation
        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var sourceFiles = Directory.GetFiles(srcDir, "*.hs", SearchOption.AllDirectories)
            .Concat(Directory.GetFiles(srcDir, "*.lhs", SearchOption.AllDirectories))
            .ToArray();

        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No .hs or .lhs source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[Haskell] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var mainFile = sourceFiles.FirstOrDefault(f =>
            Path.GetFileName(f).Equals("Main.hs", StringComparison.OrdinalIgnoreCase))
            ?? sourceFiles[0];

        var outputPath = Path.Combine(outputDir,
            OperatingSystem.IsWindows() ? $"{module.Config.Name}.exe" : module.Config.Name);

        var args = BuildGhcArgs(mainFile, outputPath, config);
        logs.Add($"$ ghc {args}");

        var result = await ProcessRunner.RunAsync(
            "ghc", args,
            new ProcessConfig
            {
                WorkingDirectory = module.ModulePath,
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
            return CompileResult.Failure(
                $"GHC compilation failed: {result.Stderr}",
                logs, stopwatch.Elapsed);
        }

        var artifacts = new List<string>();
        if (File.Exists(outputPath))
        {
            artifacts.Add(outputPath);
        }

        stopwatch.Stop();
        logs.Add($"[Haskell] Build succeeded, {artifacts.Count} artifact(s)");
        return CompileResult.Success(artifacts.ToArray(), logs, stopwatch.Elapsed);
    }

    private static string BuildGhcArgs(string mainFile, string outputPath, CompileConfig config)
    {
        var args = new List<string>
        {
            "-o", $"\"{outputPath}\"",
            config.Profile == "debug" ? "-O0" : "-O2",
            "-Wall"
        };

        var srcDir = Path.GetDirectoryName(mainFile);
        if (!string.IsNullOrEmpty(srcDir))
        {
            args.Add($"-i\"{srcDir}\"");
        }

        args.Add($"\"{mainFile}\"");
        return string.Join(" ", args);
    }

    private async Task<CompileResult> CompileWithStackAsync(
        ValidatedModuleConfig module,
        CompileConfig config,
        List<string> logs,
        System.Diagnostics.Stopwatch stopwatch,
        CancellationToken ct)
    {
        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var args = config.Profile == "release" ? "build --ghc-options=\"-O2\"" : "build";
        logs.Add($"$ stack {args}");

        var result = await ProcessRunner.RunAsync(
            "stack", args,
            new ProcessConfig
            {
                WorkingDirectory = module.ModulePath,
                Timeout = TimeSpan.FromMinutes(15)
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
            return CompileResult.Failure($"stack build failed: {result.Stderr}", logs, stopwatch.Elapsed);
        }

        // Copy artifacts from stack local install
        var artifacts = new List<string>();
        var pathResult = await ProcessRunner.RunAsync(
            "stack", "path --local-install-root",
            new ProcessConfig
            {
                WorkingDirectory = module.ModulePath,
                Timeout = TimeSpan.FromSeconds(30)
            }, ct);

        if (pathResult.IsSuccess && !string.IsNullOrEmpty(pathResult.Stdout))
        {
            var binDir = Path.Combine(pathResult.Stdout.Trim(), "bin");
            if (Directory.Exists(binDir))
            {
                foreach (var binFile in Directory.GetFiles(binDir))
                {
                    var destPath = Path.Combine(outputDir, Path.GetFileName(binFile));
                    try
                    {
                        File.Copy(binFile, destPath, overwrite: true);
                        artifacts.Add(destPath);
                    }
                    catch (Exception ex)
                    {
                        logs.Add($"[Haskell] Warning: copy failed: {ex.Message}");
                    }
                }
            }
        }

        stopwatch.Stop();
        logs.Add($"[Haskell] Stack build succeeded, {artifacts.Count} artifact(s)");
        return CompileResult.Success(artifacts.ToArray(), logs, stopwatch.Elapsed);
    }

    public Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        if ((artifactPath.EndsWith(".hs", StringComparison.OrdinalIgnoreCase) ||
             artifactPath.EndsWith(".lhs", StringComparison.OrdinalIgnoreCase)) &&
            File.Exists(artifactPath))
        {
            var lines = File.ReadAllLines(artifactPath);

            foreach (var rawLine in lines)
            {
                var line = rawLine.Trim();
                if (line.StartsWith("--") || string.IsNullOrEmpty(line)) continue;

                // Detect type signatures: functionName :: Type -> Type -> ReturnType
                var sigIdx = line.IndexOf("::", StringComparison.Ordinal);
                if (sigIdx <= 0) continue;

                var funcName = line.Substring(0, sigIdx).Trim();
                if (funcName.Contains(' ') || funcName.StartsWith("(")) continue;
                if (string.IsNullOrEmpty(funcName)) continue;

                var typeSig = line.Substring(sigIdx + 2).Trim();
                var returnType = ExtractReturnType(typeSig);

                exports.Add(new ExportedFunction
                {
                    Name = funcName,
                    ReturnType = MapHaskellTypeToForge(returnType),
                    Parameters = Array.Empty<FunctionParameter>()
                });
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
                Name = "haskell",
                Abi = "native"
            },
            Exports = exports.ToArray()
        });
    }

    private static string ExtractReturnType(string typeSig)
    {
        // Return type is the last component: "Int -> String -> Bool" => "Bool"
        var parts = typeSig.Split("->", StringSplitOptions.RemoveEmptyEntries);
        return parts.Length > 0 ? parts[^1].Trim() : "void";
    }

    private static string MapHaskellTypeToForge(string t) => t switch
    {
        "Int" or "Integer" => "i64",
        "Int32" => "i32",
        "Int64" => "i64",
        "Word" or "Word64" => "u64",
        "Word32" => "u32",
        "Float" => "f32",
        "Double" => "f64",
        "Bool" => "bool",
        "Char" => "u8",
        "String" => "string",
        "()" or "IO ()" => "void",
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Haskell Plugin");
        sb.AppendLine("// Haskell FFI via GHC foreign export");
        sb.AppendLine("using System.Runtime.InteropServices;");
        sb.AppendLine();
        sb.AppendLine($"namespace {pascalName}.Interop;");
        sb.AppendLine();
        sb.AppendLine($"internal static unsafe partial class {pascalName}Native");
        sb.AppendLine("{");
        sb.AppendLine($"    private const string LibraryName = \"{moduleName}\";");
        sb.AppendLine();
        sb.AppendLine("    // GHC RTS must be initialized before calling Haskell functions");
        sb.AppendLine("    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]");
        sb.AppendLine("    public static extern void hs_init(ref int argc, ref IntPtr argv);");
        sb.AppendLine();
        sb.AppendLine("    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]");
        sb.AppendLine("    public static extern void hs_exit();");
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Haskell Plugin");
        sb.AppendLine("// Haskell FFI via GHC foreign export");
        sb.AppendLine("#![allow(non_snake_case)]");
        sb.AppendLine();
        sb.AppendLine($"#[link(name = \"{moduleName}\")]");
        sb.AppendLine("extern \"C\" {");
        sb.AppendLine("    // GHC RTS initialization");
        sb.AppendLine("    pub fn hs_init(argc: *mut i32, argv: *mut *mut *mut i8);");
        sb.AppendLine("    pub fn hs_exit();");
        sb.AppendLine();

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
