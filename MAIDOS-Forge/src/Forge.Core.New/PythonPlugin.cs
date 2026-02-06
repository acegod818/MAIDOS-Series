// MAIDOS-Forge Python Language Plugin (Builtin)
// UEP v1.7C Compliant - Zero Technical Debt
// Code-QC v2.2B Compliant

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;

namespace Forge.Core.Plugin;

/// <summary>
/// Python 語言插件 - 支援 Cython/mypyc 編譯為原生擴展
/// </summary>
/// <impl>
/// APPROACH: 調用 Cython/mypyc 編譯 Python 代碼為 .so/.pyd
/// CALLS: ProcessRunner.RunAsync(), cython/mypyc CLI
/// EDGES: 無 Python/Cython 時返回錯誤
/// </impl>
public sealed class PythonPlugin : ILanguagePlugin
{
    private string _compiler = "cython";
    private string _python = "python3";

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "python",
        SupportedExtensions = new[] { ".py", ".pyx", ".pxd", "setup.py", "pyproject.toml" },
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = false,  // Python 交叉編譯較複雜
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        // 檢查 Python
        var pythonCommands = new[] { "python3", "python" };
        string? foundPython = null;

        foreach (var cmd in pythonCommands)
        {
            if (await ProcessRunner.CommandExistsAsync(cmd))
            {
                var verResult = await ProcessRunner.RunAsync(cmd, "--version",
                    new ProcessConfig { Timeout = TimeSpan.FromSeconds(5) }, ct);
                if (verResult.IsSuccess)
                {
                    foundPython = cmd;
                    _python = cmd;
                    break;
                }
            }
        }

        if (foundPython == null)
        {
            return (false, "Python not found (tried python3, python)");
        }

        // 檢查 Cython
        var cythonResult = await ProcessRunner.RunAsync(
            _python, "-c \"import Cython; print(Cython.__version__)\"",
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(10) }, ct);

        if (cythonResult.IsSuccess)
        {
            _compiler = "cython";
            return (true, $"{foundPython} + Cython {cythonResult.Stdout.Trim()}");
        }

        // 檢查 mypyc
        var mypycResult = await ProcessRunner.RunAsync(
            _python, "-c \"import mypyc\"",
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(10) }, ct);

        if (mypycResult.IsSuccess)
        {
            _compiler = "mypyc";
            return (true, $"{foundPython} + mypyc");
        }

        return (false, $"{foundPython} found but neither Cython nor mypyc available");
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

        logs.Add($"[Python] Using: {toolchainMsg}");

        var srcDir = module.ModulePath;
        var pyConfig = module.Config.Python ?? new PythonConfig();

        // 查找源文件
        var pyFiles = Directory.GetFiles(srcDir, "*.py", SearchOption.AllDirectories)
            .Where(f => !Path.GetFileName(f).StartsWith("test_"))
            .ToList();
        var pyxFiles = Directory.GetFiles(srcDir, "*.pyx", SearchOption.AllDirectories).ToList();

        var sourceFiles = pyxFiles.Count > 0 ? pyxFiles : pyFiles;

        if (sourceFiles.Count == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No Python source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[Python] Found {sourceFiles.Count} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var artifacts = new List<string>();

        if (_compiler == "cython")
        {
            var result = await CompileWithCython(module, sourceFiles, outputDir, pyConfig, config, logs, ct);
            if (!result.IsSuccess)
            {
                stopwatch.Stop();
                return CompileResult.Failure(result.Error ?? "Cython compilation failed", logs, stopwatch.Elapsed);
            }
            artifacts.AddRange(result.Artifacts);
        }
        else
        {
            var result = await CompileWithMypyc(module, sourceFiles, outputDir, pyConfig, config, logs, ct);
            if (!result.IsSuccess)
            {
                stopwatch.Stop();
                return CompileResult.Failure(result.Error ?? "mypyc compilation failed", logs, stopwatch.Elapsed);
            }
            artifacts.AddRange(result.Artifacts);
        }

        stopwatch.Stop();
        return CompileResult.Success(artifacts.ToArray(), logs, stopwatch.Elapsed);
    }

    private async Task<CompileResult> CompileWithCython(
        ValidatedModuleConfig module,
        List<string> sourceFiles,
        string outputDir,
        PythonConfig pyConfig,
        CompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var artifacts = new List<string>();

        foreach (var sourceFile in sourceFiles)
        {
            var baseName = Path.GetFileNameWithoutExtension(sourceFile);
            var cFile = Path.Combine(outputDir, baseName + ".c");

            // Step 1: Cython 編譯 .py/.pyx → .c
            var cythonArgs = $"-{pyConfig.Optimize} -o \"{cFile}\" \"{sourceFile}\"";
            logs.Add($"$ cython {cythonArgs}");

            var cythonResult = await ProcessRunner.RunAsync(
                "cython", cythonArgs,
                new ProcessConfig
                {
                    WorkingDirectory = module.ModulePath,
                    Timeout = TimeSpan.FromMinutes(5)
                }, ct);

            if (!cythonResult.IsSuccess)
            {
                return CompileResult.Failure($"Cython failed for {baseName}: {cythonResult.Stderr}", 
                    logs, TimeSpan.Zero);
            }

            // Step 2: 使用 C 編譯器編譯 .c → .so/.pyd
            var soExt = OperatingSystem.IsWindows() ? ".pyd" : ".so";
            var soFile = Path.Combine(outputDir, baseName + soExt);

            // 獲取 Python include 路徑
            var includeResult = await ProcessRunner.RunAsync(
                _python, "-c \"import sysconfig; print(sysconfig.get_path('include'))\"",
                new ProcessConfig { Timeout = TimeSpan.FromSeconds(5) }, ct);

            var pythonInclude = includeResult.IsSuccess ? includeResult.Stdout.Trim() : "";

            // 編譯為共享庫
            var cc = await ProcessRunner.CommandExistsAsync("clang") ? "clang" : "gcc";
            var ccArgs = new List<string>
            {
                "-shared", "-fPIC",
                config.Profile == "debug" ? "-O0 -g" : "-O2",
                $"-I\"{pythonInclude}\"",
                $"\"{cFile}\"",
                "-o", $"\"{soFile}\""
            };

            // Python 庫連結
            if (!OperatingSystem.IsWindows())
            {
                ccArgs.Add($"-lpython{pyConfig.PythonVersion}");
            }

            var ccArgsStr = string.Join(" ", ccArgs);
            logs.Add($"$ {cc} {ccArgsStr}");

            var ccResult = await ProcessRunner.RunAsync(
                cc, ccArgsStr,
                new ProcessConfig
                {
                    WorkingDirectory = outputDir,
                    Timeout = TimeSpan.FromMinutes(5)
                }, ct);

            if (!ccResult.IsSuccess)
            {
                logs.Add($"[error] {ccResult.Stderr}");
                return CompileResult.Failure($"C compiler failed for {baseName}: {ccResult.Stderr}",
                    logs, TimeSpan.Zero);
            }

            if (File.Exists(soFile))
            {
                artifacts.Add(soFile);
            }
        }

        return CompileResult.Success(artifacts.ToArray(), logs, TimeSpan.Zero);
    }

    private async Task<CompileResult> CompileWithMypyc(
        ValidatedModuleConfig module,
        List<string> sourceFiles,
        string outputDir,
        PythonConfig pyConfig,
        CompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        // mypyc 使用 python -m mypyc
        var filesArg = string.Join(" ", sourceFiles.Select(f => $"\"{f}\""));
        var mypycArgs = $"-m mypyc {filesArg}";

        logs.Add($"$ {_python} {mypycArgs}");

        var result = await ProcessRunner.RunAsync(
            _python, mypycArgs,
            new ProcessConfig
            {
                WorkingDirectory = module.ModulePath,
                Timeout = TimeSpan.FromMinutes(10)
            }, ct);

        if (!string.IsNullOrEmpty(result.Stdout))
        {
            logs.Add(result.Stdout);
        }

        if (!result.IsSuccess)
        {
            return CompileResult.Failure($"mypyc failed: {result.Stderr}", logs, TimeSpan.Zero);
        }

        // mypyc 在當前目錄生成 .so/.pyd
        var artifacts = new List<string>();
        var soExt = OperatingSystem.IsWindows() ? ".pyd" : ".so";
        var buildFiles = Directory.GetFiles(module.ModulePath, $"*{soExt}", SearchOption.TopDirectoryOnly);

        foreach (var file in buildFiles)
        {
            var destPath = Path.Combine(outputDir, Path.GetFileName(file));
            File.Copy(file, destPath, overwrite: true);
            artifacts.Add(destPath);
        }

        return CompileResult.Success(artifacts.ToArray(), logs, TimeSpan.Zero);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath, CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        // 使用 nm 提取符號
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

                // 過濾 Python/Cython 內部符號
                if (symbolName.StartsWith("Py") ||
                    symbolName.StartsWith("_Py") ||
                    symbolName.StartsWith("__pyx_") ||
                    symbolName.StartsWith("__pyx") ||
                    symbolName.Contains("CYTHON")) continue;

                exports.Add(new ExportedFunction
                {
                    Name = symbolName,
                    ReturnType = "ptr",  // Python 函數通常返回 PyObject*
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
            Language = new InterfaceLanguage { Name = "python", Abi = "c" },
            Exports = exports.ToArray()
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Python Plugin");
        sb.AppendLine("// Note: Python extensions typically need Python runtime");
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Python Plugin");
        sb.AppendLine("// Note: Python extensions typically need Python runtime");
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

        sb.AppendLine("/* Auto-generated by MAIDOS-Forge Python Plugin */");
        sb.AppendLine($"#ifndef {guardName}");
        sb.AppendLine($"#define {guardName}");
        sb.AppendLine();
        sb.AppendLine("#include <Python.h>");
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
        "void" => "void",
        "i32" => "int", "i64" => "long",
        "f64" => "double",
        "ptr" => "IntPtr",
        _ => "IntPtr"  // Python objects are pointers
    };

    private static string MapToRustType(string t) => t switch
    {
        "void" => "()",
        "i32" => "i32", "i64" => "i64",
        "f64" => "f64",
        "ptr" => "*mut std::ffi::c_void",
        _ => "*mut std::ffi::c_void"
    };

    private static string MapToCType(string t) => t switch
    {
        "void" => "void",
        "i32" => "int", "i64" => "long long",
        "f64" => "double",
        "ptr" => "PyObject*",
        _ => "PyObject*"
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
