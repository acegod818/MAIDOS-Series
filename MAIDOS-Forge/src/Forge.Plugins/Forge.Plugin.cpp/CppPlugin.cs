// MAIDOS-Forge C++ Language Plugin

using System.Text;
using System.Text.RegularExpressions;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.cpp;

/// <summary>C++ language plugin - supports clang++/g++ compilation with C++17/20/23</summary>
public sealed class CppPlugin : ILanguagePlugin
{
    private string _compiler = "clang++";

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "cpp",
        SupportedExtensions = new[] { ".cpp", ".cxx", ".cc", ".hpp", ".hxx", ".h" },
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = true,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "linux", "windows", "macos", "freebsd" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (await ProcessRunner.CommandExistsAsync("clang++"))
        {
            var version = await ProcessRunner.GetVersionAsync("clang++", "--version");
            _compiler = "clang++";
            return (true, $"clang++ {version}");
        }

        if (await ProcessRunner.CommandExistsAsync("g++"))
        {
            var version = await ProcessRunner.GetVersionAsync("g++", "--version");
            _compiler = "g++";
            return (true, $"g++ {version}");
        }

        return (false, "Neither clang++ nor g++ found");
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

        logs.Add($"[C++] Using: {toolchainMsg}");

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var sourceFiles = new[] { "*.cpp", "*.cxx", "*.cc" }
            .SelectMany(ext => Directory.GetFiles(srcDir, ext, SearchOption.AllDirectories))
            .ToArray();

        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No C++ source files found (.cpp/.cxx/.cc)", logs, stopwatch.Elapsed);
        }

        logs.Add($"[C++] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var cppConfig = module.Config.Cpp ?? new CppConfig();
        var objectFiles = new List<string>();

        // Compile each source file to object file
        foreach (var sourceFile in sourceFiles)
        {
            var objFile = Path.Combine(outputDir,
                Path.GetFileNameWithoutExtension(sourceFile) + ".o");

            var args = BuildCompileArgs(sourceFile, objFile, cppConfig, config);
            logs.Add($"$ {_compiler} {args}");

            var result = await ProcessRunner.RunAsync(
                _compiler, args,
                new ProcessConfig
                {
                    WorkingDirectory = module.ModulePath,
                    Timeout = TimeSpan.FromMinutes(10)
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

        // Link into static library
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
        CppConfig cppConfig,
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

        args.Add($"-std={cppConfig.Standard}");
        args.Add("-Wall");
        args.Add("-Wextra");
        args.Add("-fPIC");

        if (!cppConfig.Exceptions)
        {
            args.Add("-fno-exceptions");
        }

        if (!cppConfig.Rtti)
        {
            args.Add("-fno-rtti");
        }

        foreach (var define in cppConfig.Defines)
        {
            args.Add($"-D{define}");
        }

        foreach (var inc in cppConfig.IncludeDirs)
        {
            args.Add($"-I\"{inc}\"");
        }

        foreach (var lib in cppConfig.Libs)
        {
            args.Add($"-l{lib}");
        }

        return string.Join(" ", args);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        // Use nm to extract symbols, then c++filt to demangle
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
                var mangledName = parts[2];

                if (symbolType != "T") continue;
                if (IsSystemSymbol(mangledName)) continue;

                // Try to demangle C++ symbols
                var displayName = mangledName;
                var demangleResult = await ProcessRunner.RunAsync(
                    "c++filt", $"\"{mangledName}\"",
                    new ProcessConfig { Timeout = TimeSpan.FromSeconds(5) }, ct);

                if (demangleResult.IsSuccess && !string.IsNullOrEmpty(demangleResult.Stdout))
                {
                    displayName = demangleResult.Stdout.Trim();
                }

                // Parse demangled name for function signature
                var funcName = displayName;
                var paramTypes = Array.Empty<FunctionParameter>();

                var parenIdx = displayName.IndexOf('(');
                if (parenIdx > 0)
                {
                    funcName = displayName.Substring(0, parenIdx).Trim();
                    var paramStr = displayName.Substring(parenIdx + 1).TrimEnd(')', ' ');
                    if (!string.IsNullOrWhiteSpace(paramStr))
                    {
                        paramTypes = paramStr.Split(',', StringSplitOptions.RemoveEmptyEntries)
                            .Select((p, i) => new FunctionParameter
                            {
                                Name = $"arg{i}",
                                Type = MapCppTypeToForge(p.Trim())
                            })
                            .ToArray();
                    }
                }

                exports.Add(new ExportedFunction
                {
                    Name = funcName,
                    ReturnType = "i32",
                    Parameters = paramTypes
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
                Name = "cpp",
                Abi = "c++"
            },
            Exports = exports.ToArray()
        };
    }

    private static bool IsSystemSymbol(string name)
    {
        var prefixes = new[] { "__", "_init", "_fini", "_start", "frame_dummy", "_Z" + "TV", "_Z" + "TI" };
        return prefixes.Any(p => name.StartsWith(p, StringComparison.Ordinal));
    }

    private static string MapCppTypeToForge(string cppType)
    {
        var normalized = cppType.Replace("const ", "").Replace("&", "").Replace("*", "").Trim();
        return normalized switch
        {
            "int" => "i32",
            "long" => "i64",
            "short" => "i16",
            "char" => "i8",
            "unsigned int" => "u32",
            "unsigned long" => "u64",
            "unsigned short" => "u16",
            "unsigned char" => "u8",
            "float" => "f32",
            "double" => "f64",
            "bool" => "bool",
            "void" => "void",
            "std::string" => "string",
            _ => "i32"
        };
    }

    public GlueCodeResult GenerateGlue(InterfaceDescription sourceInterface, string targetLanguage)
    {
        return targetLanguage.ToLowerInvariant() switch
        {
            "csharp" or "c#" => GenerateCSharpGlue(sourceInterface),
            "rust" => GenerateRustGlue(sourceInterface),
            "c" => GenerateCHeaderGlue(sourceInterface),
            _ => GlueCodeResult.Failure($"Unsupported target language: {targetLanguage}")
        };
    }

    private static GlueCodeResult GenerateCSharpGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;
        var pascalName = ToPascalCase(moduleName);

        sb.AppendLine("// Auto-generated by MAIDOS-Forge C++ Plugin");
        sb.AppendLine("// Note: C++ name mangling requires extern \"C\" wrappers for P/Invoke");
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
            sb.AppendLine($"    public static extern {returnType} {SanitizeName(export.Name)}({parms});");
            sb.AppendLine();
        }

        sb.AppendLine("}");

        return GlueCodeResult.Success(sb.ToString(), $"{pascalName}.Interop.cs", "csharp");
    }

    private static GlueCodeResult GenerateRustGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;

        sb.AppendLine("// Auto-generated by MAIDOS-Forge C++ Plugin");
        sb.AppendLine("// Note: Requires extern \"C\" wrappers in C++ source");
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
                sb.AppendLine($"    pub fn {SanitizeName(export.Name)}({parms});");
            }
            else
            {
                sb.AppendLine($"    pub fn {SanitizeName(export.Name)}({parms}) -> {returnType};");
            }
        }

        sb.AppendLine("}");

        return GlueCodeResult.Success(sb.ToString(), $"{moduleName}_ffi.rs", "rust");
    }

    private static GlueCodeResult GenerateCHeaderGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;
        var guard = moduleName.ToUpperInvariant().Replace("-", "_").Replace(".", "_") + "_H";

        sb.AppendLine($"/* Auto-generated by MAIDOS-Forge C++ Plugin */");
        sb.AppendLine($"#ifndef {guard}");
        sb.AppendLine($"#define {guard}");
        sb.AppendLine();
        sb.AppendLine("#ifdef __cplusplus");
        sb.AppendLine("extern \"C\" {");
        sb.AppendLine("#endif");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var returnType = MapToCType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{MapToCType(p.Type)} {p.Name}"));

            if (string.IsNullOrEmpty(parms)) parms = "void";
            sb.AppendLine($"{returnType} {SanitizeName(export.Name)}({parms});");
        }

        sb.AppendLine();
        sb.AppendLine("#ifdef __cplusplus");
        sb.AppendLine("}");
        sb.AppendLine("#endif");
        sb.AppendLine();
        sb.AppendLine($"#endif /* {guard} */");

        return GlueCodeResult.Success(sb.ToString(), $"{moduleName}.h", "c");
    }

    private static string SanitizeName(string name)
    {
        // Strip namespace qualifiers for FFI
        var lastColon = name.LastIndexOf("::", StringComparison.Ordinal);
        if (lastColon >= 0) name = name.Substring(lastColon + 2);
        return Regex.Replace(name, @"[^\w]", "_");
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
        "bool" => "bool",
        "string" => "string",
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
        "bool" => "bool",
        "string" => "String",
        _ => "i32"
    };

    private static string MapToCType(string t) => t switch
    {
        "void" => "void",
        "i8" => "int8_t",
        "i16" => "int16_t",
        "i32" or "int" => "int32_t",
        "i64" => "int64_t",
        "u8" => "uint8_t",
        "u16" => "uint16_t",
        "u32" => "uint32_t",
        "u64" => "uint64_t",
        "f32" => "float",
        "f64" => "double",
        "bool" => "int",
        "string" => "const char*",
        _ => "int32_t"
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
