// MAIDOS-Forge TypeScript Language Plugin (Builtin)
// UEP v1.7C Compliant - Zero Technical Debt
// Code-QC v2.2B Compliant

using System.Text;
using System.Text.Json;
using Forge.Core.Config;
using Forge.Core.Platform;

namespace Forge.Core.Plugin;

/// <summary>
/// TypeScript 語言插件 - 支援 tsc/esbuild/bun 編譯
/// </summary>
/// <impl>
/// APPROACH: 調用 TypeScript 工具鏈編譯為 JavaScript
/// CALLS: ProcessRunner.RunAsync(), tsc/esbuild/bun CLI
/// EDGES: 無工具鏈時返回錯誤
/// </impl>
public sealed class TypeScriptPlugin : ILanguagePlugin
{
    private string _bundler = "esbuild";
    private string _runtime = "node";

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "typescript",
        SupportedExtensions = new[] { ".ts", ".tsx", ".mts", ".cts", "package.json", "tsconfig.json" },
        SupportsNativeCompilation = false,  // TypeScript 編譯為 JS，非原生
        SupportsCrossCompilation = true,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "es2020", "es2021", "es2022", "esnext", "node18", "node20" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        var tools = new List<string>();

        // 檢查 Node.js
        if (await ProcessRunner.CommandExistsAsync("node"))
        {
            var nodeVer = await ProcessRunner.GetVersionAsync("node", "--version");
            tools.Add($"node {nodeVer}");
            _runtime = "node";
        }
        else if (await ProcessRunner.CommandExistsAsync("bun"))
        {
            var bunVer = await ProcessRunner.GetVersionAsync("bun", "--version");
            tools.Add($"bun {bunVer}");
            _runtime = "bun";
        }
        else
        {
            return (false, "Neither node nor bun found");
        }

        // 檢查打包工具
        if (await ProcessRunner.CommandExistsAsync("esbuild"))
        {
            var ver = await ProcessRunner.GetVersionAsync("esbuild", "--version");
            tools.Add($"esbuild {ver}");
            _bundler = "esbuild";
        }
        else if (await ProcessRunner.CommandExistsAsync("tsc"))
        {
            var ver = await ProcessRunner.GetVersionAsync("tsc", "--version");
            tools.Add($"tsc {ver}");
            _bundler = "tsc";
        }
        else if (_runtime == "bun")
        {
            tools.Add("bun (bundler)");
            _bundler = "bun";
        }
        else
        {
            return (false, $"{_runtime} found but no TypeScript compiler (tried esbuild, tsc)");
        }

        return (true, string.Join(" + ", tools));
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

        logs.Add($"[TypeScript] Using: {toolchainMsg}");

        var srcDir = module.ModulePath;
        var tsConfig = module.Config.TypeScript ?? new TypeScriptConfig();

        // 查找入口文件
        var entryFile = FindEntryFile(srcDir);
        if (entryFile == null)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No TypeScript entry file found (tried index.ts, main.ts, src/index.ts)", 
                logs, stopwatch.Elapsed);
        }

        logs.Add($"[TypeScript] Entry: {entryFile}");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        CompileResult result;
        if (_bundler == "esbuild")
        {
            result = await CompileWithEsbuild(module, entryFile, outputDir, tsConfig, config, logs, ct);
        }
        else if (_bundler == "bun")
        {
            result = await CompileWithBun(module, entryFile, outputDir, tsConfig, config, logs, ct);
        }
        else
        {
            result = await CompileWithTsc(module, srcDir, outputDir, tsConfig, config, logs, ct);
        }

        stopwatch.Stop();
        return result.IsSuccess
            ? CompileResult.Success(result.Artifacts, logs, stopwatch.Elapsed)
            : CompileResult.Failure(result.Error ?? "Compilation failed", logs, stopwatch.Elapsed);
    }

    private static string? FindEntryFile(string srcDir)
    {
        var candidates = new[]
        {
            Path.Combine(srcDir, "index.ts"),
            Path.Combine(srcDir, "main.ts"),
            Path.Combine(srcDir, "src", "index.ts"),
            Path.Combine(srcDir, "src", "main.ts"),
            Path.Combine(srcDir, "lib", "index.ts")
        };

        foreach (var candidate in candidates)
        {
            if (File.Exists(candidate)) return candidate;
        }

        // 查找任何 .ts 文件
        var tsFiles = Directory.GetFiles(srcDir, "*.ts", SearchOption.AllDirectories)
            .Where(f => !f.Contains("node_modules") && !f.EndsWith(".d.ts"))
            .ToList();

        return tsFiles.FirstOrDefault();
    }

    private async Task<CompileResult> CompileWithEsbuild(
        ValidatedModuleConfig module,
        string entryFile,
        string outputDir,
        TypeScriptConfig tsConfig,
        CompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var outFile = Path.Combine(outputDir, module.Config.Name + ".js");

        var args = new List<string>
        {
            $"\"{entryFile}\"",
            "--bundle",
            $"--outfile=\"{outFile}\"",
            $"--target={tsConfig.Target}",
            $"--format={MapFormat(tsConfig.Module)}",
            "--platform=node"
        };

        if (config.Profile == "release")
        {
            args.Add("--minify");
        }
        else
        {
            args.Add("--sourcemap");
        }

        var argsStr = string.Join(" ", args);
        logs.Add($"$ esbuild {argsStr}");

        var result = await ProcessRunner.RunAsync(
            "esbuild", argsStr,
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
            return CompileResult.Failure($"esbuild failed: {result.Stderr}", logs, TimeSpan.Zero);
        }

        var artifacts = new List<string>();
        if (File.Exists(outFile)) artifacts.Add(outFile);

        // 生成類型宣告
        if (tsConfig.Declaration)
        {
            var dtsResult = await GenerateDeclarations(module, entryFile, outputDir, logs, ct);
            if (dtsResult != null) artifacts.Add(dtsResult);
        }

        return CompileResult.Success(artifacts.ToArray(), logs, TimeSpan.Zero);
    }

    private async Task<CompileResult> CompileWithBun(
        ValidatedModuleConfig module,
        string entryFile,
        string outputDir,
        TypeScriptConfig tsConfig,
        CompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var outFile = Path.Combine(outputDir, module.Config.Name + ".js");

        var args = new List<string>
        {
            "build",
            $"\"{entryFile}\"",
            $"--outfile=\"{outFile}\"",
            $"--target={MapBunTarget(tsConfig.Target)}"
        };

        if (config.Profile == "release")
        {
            args.Add("--minify");
        }

        var argsStr = string.Join(" ", args);
        logs.Add($"$ bun {argsStr}");

        var result = await ProcessRunner.RunAsync(
            "bun", argsStr,
            new ProcessConfig
            {
                WorkingDirectory = module.ModulePath,
                Timeout = TimeSpan.FromMinutes(5)
            }, ct);

        if (!result.IsSuccess)
        {
            return CompileResult.Failure($"bun build failed: {result.Stderr}", logs, TimeSpan.Zero);
        }

        var artifacts = new List<string>();
        if (File.Exists(outFile)) artifacts.Add(outFile);

        return CompileResult.Success(artifacts.ToArray(), logs, TimeSpan.Zero);
    }

    private async Task<CompileResult> CompileWithTsc(
        ValidatedModuleConfig module,
        string srcDir,
        string outputDir,
        TypeScriptConfig tsConfig,
        CompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var args = new List<string>
        {
            $"--outDir \"{outputDir}\"",
            $"--target {tsConfig.Target}",
            $"--module {tsConfig.Module}"
        };

        if (tsConfig.Declaration)
        {
            args.Add("--declaration");
        }

        if (tsConfig.Strict)
        {
            args.Add("--strict");
        }

        // 檢查是否有 tsconfig.json
        var tsconfigPath = Path.Combine(srcDir, "tsconfig.json");
        if (File.Exists(tsconfigPath))
        {
            args.Clear();
            args.Add($"--project \"{tsconfigPath}\"");
            args.Add($"--outDir \"{outputDir}\"");
        }

        var argsStr = string.Join(" ", args);
        logs.Add($"$ tsc {argsStr}");

        var result = await ProcessRunner.RunAsync(
            "tsc", argsStr,
            new ProcessConfig
            {
                WorkingDirectory = srcDir,
                Timeout = TimeSpan.FromMinutes(10)
            }, ct);

        if (!result.IsSuccess)
        {
            logs.Add(result.Stderr);
            return CompileResult.Failure($"tsc failed: {result.Stderr}", logs, TimeSpan.Zero);
        }

        // 收集產物
        var artifacts = Directory.GetFiles(outputDir, "*.js", SearchOption.AllDirectories)
            .Concat(Directory.GetFiles(outputDir, "*.d.ts", SearchOption.AllDirectories))
            .ToArray();

        return CompileResult.Success(artifacts, logs, TimeSpan.Zero);
    }

    private async Task<string?> GenerateDeclarations(
        ValidatedModuleConfig module,
        string entryFile,
        string outputDir,
        List<string> logs,
        CancellationToken ct)
    {
        if (!await ProcessRunner.CommandExistsAsync("tsc"))
        {
            logs.Add("[TypeScript] tsc not found, skipping declaration generation");
            return null;
        }

        var dtsFile = Path.Combine(outputDir, module.Config.Name + ".d.ts");
        var args = $"\"{entryFile}\" --declaration --emitDeclarationOnly --outDir \"{outputDir}\"";

        logs.Add($"$ tsc {args}");

        var result = await ProcessRunner.RunAsync(
            "tsc", args,
            new ProcessConfig
            {
                WorkingDirectory = module.ModulePath,
                Timeout = TimeSpan.FromMinutes(5)
            }, ct);

        if (!result.IsSuccess)
        {
            logs.Add($"[TypeScript] Declaration generation failed: {result.Stderr}");
            return null;
        }

        // 找到生成的 .d.ts 文件
        var dtsFiles = Directory.GetFiles(outputDir, "*.d.ts", SearchOption.AllDirectories);
        return dtsFiles.FirstOrDefault();
    }

    private static string MapFormat(string module)
    {
        return module.ToLowerInvariant() switch
        {
            "commonjs" or "cjs" => "cjs",
            "esm" or "es6" or "es2015" or "es2020" => "esm",
            _ => "cjs"
        };
    }

    private static string MapBunTarget(string target)
    {
        return target.ToLowerInvariant() switch
        {
            "node18" or "node20" => "node",
            "browser" => "browser",
            _ => "bun"
        };
    }

    public Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath, CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        // 解析 .d.ts 文件提取類型定義
        var dtsPath = Path.ChangeExtension(artifactPath, ".d.ts");
        if (File.Exists(dtsPath))
        {
            exports = ExtractFromDts(dtsPath);
        }
        else if (artifactPath.EndsWith(".js"))
        {
            // 嘗試從 JS 分析導出
            exports = ExtractFromJs(artifactPath);
        }

        return Task.FromResult<InterfaceDescription?>(new InterfaceDescription
        {
            Version = "1.0",
            Module = new InterfaceModule
            {
                Name = Path.GetFileNameWithoutExtension(artifactPath),
                Version = "1.0.0"
            },
            Language = new InterfaceLanguage { Name = "typescript", Abi = "js" },
            Exports = exports.ToArray()
        });
    }

    private static List<ExportedFunction> ExtractFromDts(string dtsPath)
    {
        var exports = new List<ExportedFunction>();
        var content = File.ReadAllText(dtsPath);

        // 簡單解析 export function / export const
        var lines = content.Split('\n');
        foreach (var line in lines)
        {
            var trimmed = line.Trim();

            // export function foo(...)
            if (trimmed.StartsWith("export function "))
            {
                var afterFunc = trimmed.Substring(16);
                var parenIdx = afterFunc.IndexOf('(');
                if (parenIdx > 0)
                {
                    var funcName = afterFunc.Substring(0, parenIdx).Trim();
                    exports.Add(new ExportedFunction
                    {
                        Name = funcName,
                        ReturnType = "any",
                        Parameters = Array.Empty<FunctionParameter>()
                    });
                }
            }
            // export const foo = ...
            else if (trimmed.StartsWith("export const ") || trimmed.StartsWith("export let "))
            {
                var afterExport = trimmed.Substring(trimmed.IndexOf(' ') + 1);
                afterExport = afterExport.Substring(afterExport.IndexOf(' ') + 1);
                var colonIdx = afterExport.IndexOf(':');
                var eqIdx = afterExport.IndexOf('=');
                var endIdx = colonIdx > 0 ? colonIdx : (eqIdx > 0 ? eqIdx : afterExport.Length);

                var varName = afterExport.Substring(0, endIdx).Trim();
                if (!string.IsNullOrEmpty(varName))
                {
                    exports.Add(new ExportedFunction
                    {
                        Name = varName,
                        ReturnType = "any",
                        Parameters = Array.Empty<FunctionParameter>()
                    });
                }
            }
        }

        return exports;
    }

    private static List<ExportedFunction> ExtractFromJs(string jsPath)
    {
        var exports = new List<ExportedFunction>();
        var content = File.ReadAllText(jsPath);

        // 簡單解析 module.exports / exports.xxx
        var lines = content.Split('\n');
        foreach (var line in lines)
        {
            var trimmed = line.Trim();

            // exports.foo = ...
            if (trimmed.StartsWith("exports."))
            {
                var afterExports = trimmed.Substring(8);
                var eqIdx = afterExports.IndexOf('=');
                if (eqIdx > 0)
                {
                    var name = afterExports.Substring(0, eqIdx).Trim();
                    exports.Add(new ExportedFunction
                    {
                        Name = name,
                        ReturnType = "any",
                        Parameters = Array.Empty<FunctionParameter>()
                    });
                }
            }
        }

        return exports;
    }

    public GlueCodeResult GenerateGlue(InterfaceDescription sourceInterface, string targetLanguage)
    {
        return targetLanguage.ToLowerInvariant() switch
        {
            "csharp" or "c#" => GenerateCSharpGlue(sourceInterface),
            "rust" => GenerateRustGlue(sourceInterface),
            "typescript" or "ts" => GenerateTypeScriptTypes(sourceInterface),
            _ => GlueCodeResult.Failure($"Unsupported target language: {targetLanguage}")
        };
    }

    private static GlueCodeResult GenerateCSharpGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;
        var pascalName = ToPascalCase(moduleName);

        sb.AppendLine("// Auto-generated by MAIDOS-Forge TypeScript Plugin");
        sb.AppendLine("// Note: Requires a JavaScript engine (e.g., Jint, ClearScript)");
        sb.AppendLine();
        sb.AppendLine($"namespace {pascalName}.Interop;");
        sb.AppendLine();
        sb.AppendLine("/// <summary>");
        sb.AppendLine($"/// TypeScript module interface for {moduleName}");
        sb.AppendLine("/// </summary>");
        sb.AppendLine($"public interface I{pascalName}Module");
        sb.AppendLine("{");

        foreach (var export in source.Exports)
        {
            var returnType = MapToCSharpType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{MapToCSharpType(p.Type)} {EscapeKeyword(p.Name)}"));

            sb.AppendLine($"    {returnType} {ToPascalCase(export.Name)}({parms});");
        }

        sb.AppendLine("}");

        return GlueCodeResult.Success(sb.ToString(), $"I{pascalName}Module.cs", "csharp");
    }

    private static GlueCodeResult GenerateRustGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;

        sb.AppendLine("// Auto-generated by MAIDOS-Forge TypeScript Plugin");
        sb.AppendLine("// Note: Requires a JavaScript engine (e.g., deno_core, boa)");
        sb.AppendLine();
        sb.AppendLine($"/// TypeScript module interface for {moduleName}");
        sb.AppendLine($"pub trait {ToPascalCase(moduleName)}Module {{");

        foreach (var export in source.Exports)
        {
            var returnType = MapToRustType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{ToSnakeCase(p.Name)}: {MapToRustType(p.Type)}"));

            var fnName = ToSnakeCase(export.Name);
            sb.AppendLine($"    fn {fnName}(&self{(parms.Length > 0 ? ", " + parms : "")}) -> {returnType};");
        }

        sb.AppendLine("}");

        return GlueCodeResult.Success(sb.ToString(), $"{moduleName}_types.rs", "rust");
    }

    private static GlueCodeResult GenerateTypeScriptTypes(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;

        sb.AppendLine("// Auto-generated by MAIDOS-Forge TypeScript Plugin");
        sb.AppendLine();
        sb.AppendLine($"export interface {ToPascalCase(moduleName)}Exports {{");

        foreach (var export in source.Exports)
        {
            var tsType = MapToTsType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{p.Name}: {MapToTsType(p.Type)}"));

            sb.AppendLine($"  {export.Name}: ({parms}) => {tsType};");
        }

        sb.AppendLine("}");

        return GlueCodeResult.Success(sb.ToString(), $"{moduleName}.types.ts", "typescript");
    }

    private static string MapToCSharpType(string t) => t switch
    {
        "void" => "void",
        "number" => "double",
        "string" => "string",
        "boolean" => "bool",
        "any" => "object",
        _ => "object"
    };

    private static string MapToRustType(string t) => t switch
    {
        "void" => "()",
        "number" => "f64",
        "string" => "String",
        "boolean" => "bool",
        "any" => "serde_json::Value",
        _ => "serde_json::Value"
    };

    private static string MapToTsType(string t) => t switch
    {
        "void" => "void",
        "i32" or "i64" or "f32" or "f64" => "number",
        "bool" => "boolean",
        "str" => "string",
        _ => "any"
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
