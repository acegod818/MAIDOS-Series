// MAIDOS-Forge Swift Language Plugin
// UEP v1.7C Compliant - Zero Technical Debt
// Standalone Plugin Module

using System.Text;
using System.Text.RegularExpressions;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.Swift;

/// <summary>
/// Swift language plugin - supports swiftc compilation and nm-based interface extraction
/// </summary>
/// <impl>
/// APPROACH: Invoke swiftc for compilation, nm/swift-demangle for interface extraction
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: No swiftc found returns error
/// </impl>
public sealed class SwiftPlugin : ILanguagePlugin
{
    private string _compiler = "swiftc";

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "swift",
        SupportedExtensions = new[] { ".swift" },
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = true,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "linux", "windows", "macos", "ios", "watchos", "tvos" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (await ProcessRunner.CommandExistsAsync("swiftc"))
        {
            var version = await ProcessRunner.GetVersionAsync("swiftc", "--version");
            _compiler = "swiftc";
            return (true, $"swiftc {version}");
        }

        if (await ProcessRunner.CommandExistsAsync("swift"))
        {
            var version = await ProcessRunner.GetVersionAsync("swift", "--version");
            _compiler = "swift";
            return (true, $"swift {version}");
        }

        return (false, "Neither swiftc nor swift found");
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

        logs.Add($"[Swift] Using: {toolchainMsg}");

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var sourceFiles = Directory.GetFiles(srcDir, "*.swift", SearchOption.AllDirectories);
        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No .swift source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[Swift] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        // Determine output type based on module config
        var moduleType = module.Config.Type.ToLowerInvariant();
        string outputPath;
        string typeFlag;

        if (moduleType == "library")
        {
            var libExt = GetLibraryExtension();
            outputPath = Path.Combine(outputDir, $"lib{module.Config.Name}{libExt}");
            typeFlag = "-emit-library";
        }
        else
        {
            var exeExt = OperatingSystem.IsWindows() ? ".exe" : "";
            outputPath = Path.Combine(outputDir, $"{module.Config.Name}{exeExt}");
            typeFlag = "";
        }

        // Build swiftc arguments
        var args = BuildCompileArgs(sourceFiles, outputPath, typeFlag, config);
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
                $"Compilation failed: {result.Stderr}",
                logs, stopwatch.Elapsed);
        }

        // Collect artifacts
        var artifacts = new List<string>();

        if (File.Exists(outputPath))
        {
            artifacts.Add(outputPath);
            logs.Add($"[Swift] Produced {Path.GetFileName(outputPath)}");
        }

        // Also collect .swiftmodule if generated
        var swiftModuleFiles = Directory.GetFiles(outputDir, "*.swiftmodule", SearchOption.AllDirectories);
        foreach (var swiftModule in swiftModuleFiles)
        {
            artifacts.Add(swiftModule);
        }

        // Collect .swiftinterface if generated
        var swiftInterfaceFiles = Directory.GetFiles(outputDir, "*.swiftinterface", SearchOption.AllDirectories);
        foreach (var swiftInterface in swiftInterfaceFiles)
        {
            artifacts.Add(swiftInterface);
        }

        if (artifacts.Count == 0)
        {
            // Fallback: collect any generated files
            var allFiles = Directory.GetFiles(outputDir, "*", SearchOption.AllDirectories);
            artifacts.AddRange(allFiles);
        }

        stopwatch.Stop();
        return CompileResult.Success(artifacts.ToArray(), logs, stopwatch.Elapsed);
    }

    private static string BuildCompileArgs(
        string[] sourceFiles,
        string outputPath,
        string typeFlag,
        CompileConfig config)
    {
        var args = new List<string>();

        // Source files
        foreach (var sourceFile in sourceFiles)
        {
            args.Add($"\"{sourceFile}\"");
        }

        // Output
        args.Add("-o");
        args.Add($"\"{outputPath}\"");

        // Library emit flag
        if (!string.IsNullOrEmpty(typeFlag))
        {
            args.Add(typeFlag);
        }

        // Optimization
        if (config.Profile == "debug")
        {
            args.Add("-Onone");
            args.Add("-g");
        }
        else
        {
            args.Add("-O");
        }

        // Warnings
        args.Add("-warnings-as-errors");

        // Module name derived from output
        var moduleName = Path.GetFileNameWithoutExtension(outputPath)
            .TrimStart("lib".ToCharArray());
        args.Add("-module-name");
        args.Add(moduleName);

        return string.Join(" ", args);
    }

    private static string GetLibraryExtension()
    {
        if (OperatingSystem.IsWindows()) return ".dll";
        if (OperatingSystem.IsMacOS()) return ".dylib";
        return ".so";
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        // Use nm to extract exported symbols from compiled binary/library
        var nmResult = await ProcessRunner.RunAsync(
            "nm", $"-g --defined-only \"{artifactPath}\"",
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);

        if (nmResult.IsSuccess && !string.IsNullOrEmpty(nmResult.Stdout))
        {
            // Try to demangle Swift symbols using swift-demangle
            var demangleResult = await ProcessRunner.RunAsync(
                "swift-demangle", "",
                new ProcessConfig { Timeout = TimeSpan.FromSeconds(5) }, ct);

            var canDemangle = demangleResult.IsSuccess ||
                await ProcessRunner.CommandExistsAsync("swift-demangle");

            foreach (var line in nmResult.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries))
            {
                var parts = line.Split(' ', StringSplitOptions.RemoveEmptyEntries);
                if (parts.Length < 3) continue;

                var symbolType = parts[1];
                var symbolName = parts[2];

                // Only exported text symbols
                if (symbolType != "T" && symbolType != "t") continue;

                // Skip internal Swift runtime symbols
                if (IsSwiftRuntimeSymbol(symbolName)) continue;

                var displayName = symbolName;

                // Attempt to demangle Swift symbol
                if (canDemangle && symbolName.StartsWith("$s"))
                {
                    var demangled = await ProcessRunner.RunAsync(
                        "swift-demangle", $"\"{symbolName}\"",
                        new ProcessConfig { Timeout = TimeSpan.FromSeconds(5) }, ct);

                    if (demangled.IsSuccess && !string.IsNullOrEmpty(demangled.Stdout))
                    {
                        displayName = demangled.Stdout.Trim();
                    }
                }

                // Extract function name from demangled or raw symbol
                var funcName = ExtractFunctionName(displayName);
                if (string.IsNullOrEmpty(funcName)) continue;
                if (exports.Any(e => e.Name == funcName)) continue;

                exports.Add(new ExportedFunction
                {
                    Name = funcName,
                    ReturnType = "i32",
                    Parameters = Array.Empty<FunctionParameter>()
                });
            }
        }

        // Fallback: parse .swiftinterface file if present
        var interfacePath = Path.ChangeExtension(artifactPath, ".swiftinterface");
        if (File.Exists(interfacePath))
        {
            var content = await File.ReadAllTextAsync(interfacePath, ct);
            ParseSwiftInterface(content, exports);
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
                Name = "swift",
                Abi = "c"
            },
            Exports = exports.ToArray()
        };
    }

    private static bool IsSwiftRuntimeSymbol(string name)
    {
        var prefixes = new[] { "__swift", "_swift", "$sS", "$sSS", "$ss",
            "__", "_$s0", "___swift", "_main", "_objc" };
        return prefixes.Any(p => name.StartsWith(p, StringComparison.Ordinal));
    }

    private static string ExtractFunctionName(string symbol)
    {
        // For C-exported functions (@_cdecl), the name is typically clean
        if (!symbol.Contains("$") && !symbol.Contains(" "))
        {
            return symbol.TrimStart('_');
        }

        // For demangled Swift symbols: "module.funcName(param:) -> RetType"
        var funcMatch = Regex.Match(symbol, @"\.(\w+)\(");
        if (funcMatch.Success)
        {
            return funcMatch.Groups[1].Value;
        }

        return string.Empty;
    }

    private static void ParseSwiftInterface(string content, List<ExportedFunction> exports)
    {
        // Parse: public func funcName(paramName: ParamType) -> RetType
        var funcRegex = new Regex(
            @"public\s+func\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*(\w+))?",
            RegexOptions.Compiled);

        foreach (Match match in funcRegex.Matches(content))
        {
            var funcName = match.Groups[1].Value;
            var paramStr = match.Groups[2].Value;
            var returnTypeStr = match.Groups[3].Success ? match.Groups[3].Value : "void";

            if (exports.Any(e => e.Name == funcName)) continue;

            var parameters = ParseSwiftParameters(paramStr);
            var returnType = MapSwiftTypeToForge(returnTypeStr);

            exports.Add(new ExportedFunction
            {
                Name = funcName,
                ReturnType = returnType,
                Parameters = parameters
            });
        }
    }

    private static FunctionParameter[] ParseSwiftParameters(string paramStr)
    {
        if (string.IsNullOrWhiteSpace(paramStr)) return Array.Empty<FunctionParameter>();

        return paramStr.Split(',', StringSplitOptions.RemoveEmptyEntries)
            .Select(p =>
            {
                // Swift param format: "label name: Type" or "name: Type"
                var colonIdx = p.IndexOf(':');
                if (colonIdx < 0) return null;

                var namePart = p.Substring(0, colonIdx).Trim();
                var typePart = p.Substring(colonIdx + 1).Trim();

                // Use the last word before colon as parameter name
                var nameWords = namePart.Split(' ', StringSplitOptions.RemoveEmptyEntries);
                var name = nameWords.Length > 0 ? nameWords[nameWords.Length - 1] : "arg";

                // Skip underscore-only labels
                if (name == "_" && nameWords.Length > 1)
                {
                    name = nameWords[nameWords.Length - 2];
                }
                else if (name == "_")
                {
                    name = "arg";
                }

                return new FunctionParameter
                {
                    Name = name,
                    Type = MapSwiftTypeToForge(typePart)
                };
            })
            .Where(p => p is not null)
            .ToArray()!;
    }

    private static string MapSwiftTypeToForge(string swiftType) => swiftType.Trim() switch
    {
        "Void" or "void" or "()" => "void",
        "Int8" => "i8",
        "Int16" => "i16",
        "Int32" or "Int" or "CInt" => "i32",
        "Int64" => "i64",
        "UInt8" => "u8",
        "UInt16" => "u16",
        "UInt32" or "UInt" => "u32",
        "UInt64" => "u64",
        "Float" or "CFloat" => "f32",
        "Double" or "CDouble" => "f64",
        "Bool" or "CBool" => "u8",
        _ => "i32"
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Swift Plugin");
        sb.AppendLine("// Swift interop via @_cdecl exported C-ABI functions");
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Swift Plugin");
        sb.AppendLine("// Swift interop via @_cdecl exported C-ABI functions");
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
