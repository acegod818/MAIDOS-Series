// MAIDOS-Forge C++ Language Plugin (Builtin)
// UEP v1.7C Compliant - Zero Technical Debt
// Code-QC v2.2B Compliant

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;

namespace Forge.Core.Plugin;

/// <summary>
/// C++ 語言插件 - 支援 clang++/g++/MSVC 編譯
/// </summary>
/// <impl>
/// APPROACH: 調用 C++ 編譯器編譯源碼，生成靜態庫
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: 無編譯器時返回錯誤，MSVC 使用不同參數格式
/// </impl>
public sealed class CppPlugin : ILanguagePlugin
{
    private string _compiler = "clang++";
    private bool _isMsvc = false;

    private static readonly string[] SourceExtensions = { ".cpp", ".cc", ".cxx", ".c++", ".C" };
    private static readonly string[] HeaderExtensions = { ".hpp", ".hxx", ".h++", ".hh", ".h" };

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "cpp",
        SupportedExtensions = SourceExtensions.Concat(HeaderExtensions).ToArray(),
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = true,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[]
        {
            "x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu",
            "x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl",
            "aarch64-unknown-linux-gnu",
            "x86_64-apple-darwin", "aarch64-apple-darwin"
        }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (await ProcessRunner.CommandExistsAsync("clang++"))
        {
            var version = await ProcessRunner.GetVersionAsync("clang++", "--version");
            _compiler = "clang++";
            _isMsvc = false;
            return (true, $"clang++ {version}");
        }

        if (await ProcessRunner.CommandExistsAsync("g++"))
        {
            var version = await ProcessRunner.GetVersionAsync("g++", "--version");
            _compiler = "g++";
            _isMsvc = false;
            return (true, $"g++ {version}");
        }

        if (OperatingSystem.IsWindows() && await ProcessRunner.CommandExistsAsync("cl"))
        {
            _compiler = "cl";
            _isMsvc = true;
            return (true, "MSVC cl.exe");
        }

        return (false, "No C++ compiler found (tried clang++, g++, cl.exe)");
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

        var sourceFiles = new List<string>();
        foreach (var ext in SourceExtensions)
        {
            sourceFiles.AddRange(Directory.GetFiles(srcDir, $"*{ext}", SearchOption.AllDirectories));
        }

        if (sourceFiles.Count == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No C++ source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[C++] Found {sourceFiles.Count} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var cppConfig = module.Config.Cpp ?? new CppConfig();
        var objectFiles = new List<string>();

        foreach (var sourceFile in sourceFiles)
        {
            var objExt = _isMsvc ? ".obj" : ".o";
            var objFile = Path.Combine(outputDir,
                Path.GetFileNameWithoutExtension(sourceFile) + objExt);

            var args = _isMsvc
                ? BuildMsvcCompileArgs(sourceFile, objFile, cppConfig, config)
                : BuildGnuCompileArgs(sourceFile, objFile, cppConfig, config);

            logs.Add($"$ {_compiler} {args}");

            var result = await ProcessRunner.RunAsync(
                _compiler, args,
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
                    $"Compilation failed for {Path.GetFileName(sourceFile)}: {result.Stderr}",
                    logs, stopwatch.Elapsed);
            }

            objectFiles.Add(objFile);
        }

        var libName = _isMsvc ? $"{module.Config.Name}.lib" : $"lib{module.Config.Name}.a";
        var libPath = Path.Combine(outputDir, libName);

        if (_isMsvc)
        {
            var libArgs = $"/OUT:\"{libPath}\" /NOLOGO {string.Join(" ", objectFiles.Select(f => $"\"{f}\""))}";
            logs.Add($"$ lib {libArgs}");

            var libResult = await ProcessRunner.RunAsync("lib", libArgs,
                new ProcessConfig { Timeout = TimeSpan.FromMinutes(5) }, ct);

            if (!libResult.IsSuccess)
            {
                logs.Add($"lib.exe failed: {libResult.Stderr}");
            }
        }
        else
        {
            var arArgs = $"rcs \"{libPath}\" {string.Join(" ", objectFiles.Select(f => $"\"{f}\""))}";
            logs.Add($"$ ar {arArgs}");

            var arResult = await ProcessRunner.RunAsync("ar", arArgs,
                new ProcessConfig { Timeout = TimeSpan.FromMinutes(5) }, ct);

            if (!arResult.IsSuccess)
            {
                logs.Add($"ar failed: {arResult.Stderr}");
            }
        }

        stopwatch.Stop();
        var artifacts = File.Exists(libPath)
            ? new[] { libPath }
            : objectFiles.ToArray();

        return CompileResult.Success(artifacts, logs, stopwatch.Elapsed);
    }

    private static string BuildGnuCompileArgs(
        string sourceFile, string outputFile, CppConfig cppConfig, CompileConfig config)
    {
        var args = new List<string>
        {
            "-c", $"\"{sourceFile}\"", "-o", $"\"{outputFile}\""
        };

        args.Add(config.Profile == "debug" ? "-O0" : "-O2");
        if (config.Profile == "debug") args.Add("-g");

        args.Add($"-std={cppConfig.Standard}");
        args.AddRange(new[] { "-Wall", "-Wextra", "-Wpedantic", "-fPIC" });

        if (!cppConfig.Exceptions) args.Add("-fno-exceptions");
        if (!cppConfig.Rtti) args.Add("-fno-rtti");

        foreach (var define in cppConfig.Defines) args.Add($"-D{define}");
        foreach (var inc in cppConfig.IncludeDirs) args.Add($"-I\"{inc}\"");

        return string.Join(" ", args);
    }

    private static string BuildMsvcCompileArgs(
        string sourceFile, string outputFile, CppConfig cppConfig, CompileConfig config)
    {
        var args = new List<string>
        {
            "/c", $"\"{sourceFile}\"", $"/Fo\"{outputFile}\"", "/nologo", "/EHsc"
        };

        if (config.Profile == "debug")
        {
            args.AddRange(new[] { "/Od", "/Zi", "/MDd" });
        }
        else
        {
            args.AddRange(new[] { "/O2", "/MD" });
        }

        var msvcStd = cppConfig.Standard switch
        {
            "c++11" or "c++14" => "/std:c++14",
            "c++17" => "/std:c++17",
            "c++20" => "/std:c++20",
            "c++23" => "/std:c++latest",
            _ => "/std:c++17"
        };
        args.Add(msvcStd);
        args.Add("/W4");

        if (!cppConfig.Exceptions) args.Remove("/EHsc");
        if (!cppConfig.Rtti) args.Add("/GR-");

        foreach (var define in cppConfig.Defines) args.Add($"/D{define}");
        foreach (var inc in cppConfig.IncludeDirs) args.Add($"/I\"{inc}\"");

        return string.Join(" ", args);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath, CancellationToken ct = default)
    {
        var exports = _isMsvc || artifactPath.EndsWith(".lib", StringComparison.OrdinalIgnoreCase)
            ? await ExtractWithDumpbin(artifactPath, ct)
            : await ExtractWithNm(artifactPath, ct);

        return new InterfaceDescription
        {
            Version = "1.0",
            Module = new InterfaceModule
            {
                Name = Path.GetFileNameWithoutExtension(artifactPath),
                Version = "1.0.0"
            },
            Language = new InterfaceLanguage { Name = "cpp", Abi = "c" },
            Exports = exports.ToArray()
        };
    }

    private async Task<List<ExportedFunction>> ExtractWithNm(string artifactPath, CancellationToken ct)
    {
        var exports = new List<ExportedFunction>();

        var nmResult = await ProcessRunner.RunAsync(
            "nm", $"-g --defined-only --demangle \"{artifactPath}\"",
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);

        if (!nmResult.IsSuccess || string.IsNullOrEmpty(nmResult.Stdout)) return exports;

        foreach (var line in nmResult.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries))
        {
            var parts = line.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length < 3) continue;

            var symbolType = parts[1];
            var symbolName = parts[2];

            if (symbolType != "T") continue;
            if (IsSystemSymbol(symbolName) || IsMangledCppSymbol(symbolName)) continue;

            if (symbolName.StartsWith("_") && !symbolName.StartsWith("__"))
                symbolName = symbolName.TrimStart('_');

            exports.Add(new ExportedFunction
            {
                Name = symbolName,
                ReturnType = "i32",
                Parameters = Array.Empty<FunctionParameter>()
            });
        }

        return exports;
    }

    private async Task<List<ExportedFunction>> ExtractWithDumpbin(string artifactPath, CancellationToken ct)
    {
        var exports = new List<ExportedFunction>();

        var dumpResult = await ProcessRunner.RunAsync(
            "dumpbin", $"/SYMBOLS \"{artifactPath}\"",
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);

        if (!dumpResult.IsSuccess || string.IsNullOrEmpty(dumpResult.Stdout)) return exports;

        foreach (var line in dumpResult.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries))
        {
            if (!line.Contains("External") || !line.Contains("SECT")) continue;

            var parts = line.Split('|', StringSplitOptions.TrimEntries);
            if (parts.Length < 2) continue;

            var symbolName = parts[^1].Trim();
            if (IsSystemSymbol(symbolName) || IsMangledCppSymbol(symbolName)) continue;

            exports.Add(new ExportedFunction
            {
                Name = symbolName,
                ReturnType = "i32",
                Parameters = Array.Empty<FunctionParameter>()
            });
        }

        return exports;
    }

    private static bool IsSystemSymbol(string name) =>
        new[] { "__", "_init", "_fini", "_start", "frame_dummy", "_GLOBAL_", "__cxa_", "__gxx_", ".L", "_Z" }
            .Any(p => name.StartsWith(p, StringComparison.Ordinal));

    private static bool IsMangledCppSymbol(string name) =>
        name.StartsWith("_Z") || name.StartsWith("?");

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
                $"{MapToCSharpType(p.Type)} {EscapeKeyword(p.Name)}"));

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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge C++ Plugin");
        sb.AppendLine("#![allow(non_snake_case)]");
        sb.AppendLine();
        sb.AppendLine($"#[link(name = \"{moduleName}\")]");
        sb.AppendLine("extern \"C\" {");

        foreach (var export in source.Exports)
        {
            var returnType = MapToRustType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{ToSnakeCase(p.Name)}: {MapToRustType(p.Type)}"));

            sb.AppendLine(returnType == "()"
                ? $"    pub fn {export.Name}({parms});"
                : $"    pub fn {export.Name}({parms}) -> {returnType};");
        }

        sb.AppendLine("}");
        return GlueCodeResult.Success(sb.ToString(), $"{moduleName}_ffi.rs", "rust");
    }

    private static GlueCodeResult GenerateCHeaderGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;
        var guardName = $"{moduleName.ToUpperInvariant()}_FFI_H";

        sb.AppendLine("/* Auto-generated by MAIDOS-Forge C++ Plugin */");
        sb.AppendLine($"#ifndef {guardName}");
        sb.AppendLine($"#define {guardName}");
        sb.AppendLine();
        sb.AppendLine("#ifdef __cplusplus");
        sb.AppendLine("extern \"C\" {");
        sb.AppendLine("#endif");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var returnType = MapToCType(export.ReturnType);
            var parms = export.Parameters.Count == 0 ? "void"
                : string.Join(", ", export.Parameters.Select(p => $"{MapToCType(p.Type)} {p.Name}"));
            sb.AppendLine($"{returnType} {export.Name}({parms});");
        }

        sb.AppendLine();
        sb.AppendLine("#ifdef __cplusplus");
        sb.AppendLine("}");
        sb.AppendLine("#endif");
        sb.AppendLine($"#endif /* {guardName} */");

        return GlueCodeResult.Success(sb.ToString(), $"{moduleName}_ffi.h", "c");
    }

    private static string MapToCSharpType(string t) => t switch
    {
        "void" => "void", "bool" => "bool",
        "i8" or "char" => "sbyte", "i16" => "short", "i32" or "int" => "int", "i64" => "long",
        "u8" => "byte", "u16" => "ushort", "u32" => "uint", "u64" => "ulong",
        "f32" => "float", "f64" => "double",
        "isize" => "nint", "usize" => "nuint",
        _ when t.EndsWith("*") => "IntPtr",
        _ => "int"
    };

    private static string MapToRustType(string t) => t switch
    {
        "void" => "()", "bool" => "bool",
        "i8" or "char" => "i8", "i16" => "i16", "i32" or "int" => "i32", "i64" => "i64",
        "u8" => "u8", "u16" => "u16", "u32" => "u32", "u64" => "u64",
        "f32" => "f32", "f64" => "f64",
        "isize" => "isize", "usize" => "usize",
        _ when t.EndsWith("*") => "*mut std::ffi::c_void",
        _ => "i32"
    };

    private static string MapToCType(string t) => t switch
    {
        "void" => "void", "bool" => "_Bool",
        "i8" => "int8_t", "i16" => "int16_t", "i32" => "int32_t", "i64" => "int64_t",
        "u8" => "uint8_t", "u16" => "uint16_t", "u32" => "uint32_t", "u64" => "uint64_t",
        "f32" => "float", "f64" => "double",
        "isize" => "intptr_t", "usize" => "size_t",
        _ => "int"
    };

    private static string ToPascalCase(string s)
    {
        if (string.IsNullOrEmpty(s)) return s;
        var parts = s.Split('-', '_', '.');
        return string.Concat(parts.Select(p =>
            p.Length > 0 ? char.ToUpperInvariant(p[0]) + p.Substring(1).ToLowerInvariant() : ""));
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

    private static readonly HashSet<string> CSharpKeywords = new()
    {
        "abstract", "as", "base", "bool", "break", "byte", "case", "catch",
        "char", "checked", "class", "const", "continue", "decimal", "default",
        "delegate", "do", "double", "else", "enum", "event", "explicit", "extern",
        "false", "finally", "fixed", "float", "for", "foreach", "goto", "if",
        "implicit", "in", "int", "interface", "internal", "is", "lock", "long",
        "namespace", "new", "null", "object", "operator", "out", "override",
        "params", "private", "protected", "public", "readonly", "ref", "return",
        "sbyte", "sealed", "short", "sizeof", "stackalloc", "static", "string",
        "struct", "switch", "this", "throw", "true", "try", "typeof", "uint",
        "ulong", "unchecked", "unsafe", "ushort", "using", "virtual", "void",
        "volatile", "while"
    };

    private static string EscapeKeyword(string name) =>
        CSharpKeywords.Contains(name) ? $"@{name}" : name;
}
