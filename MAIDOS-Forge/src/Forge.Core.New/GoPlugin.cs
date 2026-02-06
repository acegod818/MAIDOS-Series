// MAIDOS-Forge Go Language Plugin (Builtin)
// UEP v1.7C Compliant - Zero Technical Debt
// Code-QC v2.2B Compliant

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;

namespace Forge.Core.Plugin;

/// <summary>
/// Go 語言插件 - 支援 cgo 編譯為 C 共享庫
/// </summary>
/// <impl>
/// APPROACH: 調用 go build 生成 c-shared 庫，支援 cgo FFI
/// CALLS: ProcessRunner.RunAsync(), go CLI
/// EDGES: 無 go 時返回錯誤，需啟用 CGO_ENABLED
/// </impl>
public sealed class GoPlugin : ILanguagePlugin
{
    private const string GoCommand = "go";

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "go",
        SupportedExtensions = new[] { ".go", "go.mod", "go.sum" },
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = true,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[]
        {
            "windows/amd64", "windows/arm64",
            "linux/amd64", "linux/arm64",
            "darwin/amd64", "darwin/arm64"
        }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (!await ProcessRunner.CommandExistsAsync(GoCommand))
        {
            return (false, "go command not found");
        }

        var versionResult = await ProcessRunner.RunAsync(
            GoCommand, "version",
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(10) }, ct);

        if (!versionResult.IsSuccess)
        {
            return (false, $"Failed to get go version: {versionResult.Stderr}");
        }

        var version = versionResult.Stdout.Trim();
        return (true, version);
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

        logs.Add($"[Go] Using: {toolchainMsg}");

        // 查找 go.mod 或源碼
        var srcDir = module.ModulePath;
        var goModPath = Path.Combine(srcDir, "go.mod");
        
        if (!File.Exists(goModPath))
        {
            // 檢查 src 子目錄
            var srcSubDir = Path.Combine(srcDir, "src");
            if (Directory.Exists(srcSubDir))
            {
                goModPath = Path.Combine(srcSubDir, "go.mod");
                if (File.Exists(goModPath))
                {
                    srcDir = srcSubDir;
                }
            }
        }

        var goFiles = Directory.GetFiles(srcDir, "*.go", SearchOption.AllDirectories);
        if (goFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No .go source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[Go] Found {goFiles.Length} source file(s)");

        // 準備輸出
        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var goConfig = module.Config.Go ?? new GoConfig();
        var buildMode = goConfig.BuildMode;

        // 輸出檔案名稱
        string outputExt;
        if (buildMode == "c-shared")
        {
            outputExt = OperatingSystem.IsWindows() ? ".dll" 
                : OperatingSystem.IsMacOS() ? ".dylib" 
                : ".so";
        }
        else if (buildMode == "c-archive")
        {
            outputExt = ".a";
        }
        else
        {
            outputExt = OperatingSystem.IsWindows() ? ".exe" : "";
        }

        var outputName = module.Config.Name + outputExt;
        var outputPath = Path.Combine(outputDir, outputName);

        // 構建參數
        var args = new List<string> { "build" };

        if (!string.IsNullOrEmpty(buildMode) && buildMode != "default")
        {
            args.Add($"-buildmode={buildMode}");
        }

        // 輸出路徑
        args.Add($"-o");
        args.Add($"\"{outputPath}\"");

        // Tags
        if (goConfig.Tags.Count > 0)
        {
            args.Add($"-tags={string.Join(",", goConfig.Tags)}");
        }

        // LdFlags
        if (goConfig.LdFlags.Count > 0)
        {
            args.Add($"-ldflags=\"{string.Join(" ", goConfig.LdFlags)}\"");
        }

        // 編譯優化
        if (config.Profile == "release")
        {
            args.Add("-trimpath");
        }

        // 加入主包路徑
        args.Add(".");

        var argsStr = string.Join(" ", args);
        logs.Add($"$ go {argsStr}");

        // 設置環境變數
        var env = new Dictionary<string, string>(config.Environment ?? new Dictionary<string, string>());
        
        // CGO_ENABLED
        if (goConfig.CgoEnabled || buildMode == "c-shared" || buildMode == "c-archive")
        {
            env["CGO_ENABLED"] = "1";
        }

        // 交叉編譯
        if (config.TargetPlatform != "native")
        {
            var (goos, goarch) = ParseGoTarget(config.TargetPlatform);
            if (!string.IsNullOrEmpty(goos))
            {
                env["GOOS"] = goos;
                logs.Add($"[Go] GOOS={goos}");
            }
            if (!string.IsNullOrEmpty(goarch))
            {
                env["GOARCH"] = goarch;
                logs.Add($"[Go] GOARCH={goarch}");
            }
        }

        var result = await ProcessRunner.RunAsync(
            GoCommand, argsStr,
            new ProcessConfig
            {
                WorkingDirectory = srcDir,
                Environment = env,
                Timeout = TimeSpan.FromMinutes(10)
            }, ct);

        if (!string.IsNullOrEmpty(result.Stdout))
        {
            logs.AddRange(result.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries)
                .Select(l => $"[go] {l}"));
        }

        if (!result.IsSuccess)
        {
            logs.Add($"[Go] Build failed with exit code {result.ExitCode}");
            if (!string.IsNullOrEmpty(result.Stderr))
            {
                logs.AddRange(result.Stderr.Split('\n', StringSplitOptions.RemoveEmptyEntries)
                    .Select(l => $"[error] {l}"));
            }
            stopwatch.Stop();
            return CompileResult.Failure($"go build failed: {result.Stderr}", logs, stopwatch.Elapsed);
        }

        stopwatch.Stop();

        // 收集產物
        var artifacts = new List<string>();
        if (File.Exists(outputPath))
        {
            artifacts.Add(outputPath);
        }

        // c-shared 模式會生成 .h 頭文件
        var headerPath = Path.ChangeExtension(outputPath, ".h");
        if (File.Exists(headerPath))
        {
            artifacts.Add(headerPath);
        }

        return CompileResult.Success(artifacts.ToArray(), logs, stopwatch.Elapsed);
    }

    private static (string Goos, string Goarch) ParseGoTarget(string target)
    {
        // 支援格式: "linux/amd64", "windows-x64", "darwin-arm64"
        var parts = target.Replace("-", "/").Split('/');
        if (parts.Length < 2) return (string.Empty, string.Empty);

        var os = parts[0].ToLowerInvariant() switch
        {
            "win" or "windows" => "windows",
            "linux" => "linux",
            "darwin" or "macos" or "osx" => "darwin",
            _ => parts[0]
        };

        var arch = parts[1].ToLowerInvariant() switch
        {
            "x64" or "amd64" or "x86_64" => "amd64",
            "arm64" or "aarch64" => "arm64",
            "x86" or "386" or "i386" => "386",
            _ => parts[1]
        };

        return (os, arch);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath, CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        // Go c-shared 會生成 .h 頭文件，從中提取接口
        var headerPath = Path.ChangeExtension(artifactPath, ".h");
        if (File.Exists(headerPath))
        {
            exports = await ExtractFromHeader(headerPath, ct);
        }
        else
        {
            // 嘗試用 nm 提取
            exports = await ExtractWithNm(artifactPath, ct);
        }

        return new InterfaceDescription
        {
            Version = "1.0",
            Module = new InterfaceModule
            {
                Name = Path.GetFileNameWithoutExtension(artifactPath),
                Version = "1.0.0"
            },
            Language = new InterfaceLanguage { Name = "go", Abi = "c" },
            Exports = exports.ToArray()
        };
    }

    private static async Task<List<ExportedFunction>> ExtractFromHeader(string headerPath, CancellationToken ct)
    {
        var exports = new List<ExportedFunction>();
        var content = await File.ReadAllTextAsync(headerPath, ct);

        // 解析 cgo 生成的 .h 文件
        // 格式: extern <type> <name>(<params>);
        var lines = content.Split('\n');
        foreach (var line in lines)
        {
            var trimmed = line.Trim();
            if (!trimmed.StartsWith("extern ") || !trimmed.Contains("(")) continue;

            // 跳過 Go 運行時內部符號
            if (trimmed.Contains("_cgo_") || trimmed.Contains("crosscall")) continue;

            // 簡單解析
            var afterExtern = trimmed.Substring(7).Trim();
            var parenIdx = afterExtern.IndexOf('(');
            if (parenIdx <= 0) continue;

            var beforeParen = afterExtern.Substring(0, parenIdx).Trim();
            var lastSpace = beforeParen.LastIndexOf(' ');
            if (lastSpace <= 0) continue;

            var returnType = beforeParen.Substring(0, lastSpace).Trim();
            var funcName = beforeParen.Substring(lastSpace + 1).Trim();

            // 過濾 Go 內部符號
            if (funcName.StartsWith("_") || funcName.Contains("cgo")) continue;

            exports.Add(new ExportedFunction
            {
                Name = funcName,
                ReturnType = MapCTypeToInternal(returnType),
                Parameters = Array.Empty<FunctionParameter>()
            });
        }

        return exports;
    }

    private static async Task<List<ExportedFunction>> ExtractWithNm(string artifactPath, CancellationToken ct)
    {
        var exports = new List<ExportedFunction>();

        var nmResult = await ProcessRunner.RunAsync(
            "nm", $"-g --defined-only \"{artifactPath}\"",
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);

        if (!nmResult.IsSuccess || string.IsNullOrEmpty(nmResult.Stdout)) return exports;

        foreach (var line in nmResult.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries))
        {
            var parts = line.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length < 3) continue;

            var symbolType = parts[1];
            var symbolName = parts[2];

            if (symbolType != "T") continue;

            // 過濾 Go 運行時和 cgo 內部符號
            if (symbolName.StartsWith("_cgo") || 
                symbolName.StartsWith("runtime.") ||
                symbolName.StartsWith("go.") ||
                symbolName.Contains("·") ||  // Go 內部符號
                symbolName.StartsWith("_")) continue;

            exports.Add(new ExportedFunction
            {
                Name = symbolName,
                ReturnType = "i32",
                Parameters = Array.Empty<FunctionParameter>()
            });
        }

        return exports;
    }

    private static string MapCTypeToInternal(string cType)
    {
        var normalized = cType.Replace("const ", "").Replace("unsigned ", "u").Trim();
        return normalized switch
        {
            "void" => "void",
            "char" or "GoInt8" => "i8",
            "short" or "GoInt16" => "i16",
            "int" or "GoInt32" or "GoInt" => "i32",
            "long" or "long long" or "GoInt64" => "i64",
            "uchar" or "GoUint8" => "u8",
            "ushort" or "GoUint16" => "u16",
            "uint" or "GoUint32" or "GoUint" => "u32",
            "ulong" or "GoUint64" => "u64",
            "float" or "GoFloat32" => "f32",
            "double" or "GoFloat64" => "f64",
            _ when normalized.Contains("*") => "ptr",
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Go Plugin");
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Go Plugin");
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

        sb.AppendLine("/* Auto-generated by MAIDOS-Forge Go Plugin */");
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
        "i8" => "sbyte", "i16" => "short", "i32" => "int", "i64" => "long",
        "u8" => "byte", "u16" => "ushort", "u32" => "uint", "u64" => "ulong",
        "f32" => "float", "f64" => "double",
        "ptr" => "IntPtr",
        _ => "int"
    };

    private static string MapToRustType(string t) => t switch
    {
        "void" => "()", "bool" => "bool",
        "i8" => "i8", "i16" => "i16", "i32" => "i32", "i64" => "i64",
        "u8" => "u8", "u16" => "u16", "u32" => "u32", "u64" => "u64",
        "f32" => "f32", "f64" => "f64",
        "ptr" => "*mut std::ffi::c_void",
        _ => "i32"
    };

    private static string MapToCType(string t) => t switch
    {
        "void" => "void", "bool" => "_Bool",
        "i8" => "int8_t", "i16" => "int16_t", "i32" => "int32_t", "i64" => "int64_t",
        "u8" => "uint8_t", "u16" => "uint16_t", "u32" => "uint32_t", "u64" => "uint64_t",
        "f32" => "float", "f64" => "double",
        "ptr" => "void*",
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
