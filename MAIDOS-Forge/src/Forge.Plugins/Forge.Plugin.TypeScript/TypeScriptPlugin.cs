// MAIDOS-Forge TypeScript Language Plugin

using System.Text;
using System.Text.RegularExpressions;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.typescript;

/// <summary>TypeScript language plugin - supports tsc compilation and type checking</summary>
public sealed class TypeScriptPlugin : ILanguagePlugin
{
    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "typescript",
        SupportedExtensions = new[] { ".ts", ".tsx", ".mts", ".cts" },
        SupportsNativeCompilation = false,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "es2020", "es2021", "es2022", "esnext" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (await ProcessRunner.CommandExistsAsync("tsc"))
        {
            var version = await ProcessRunner.GetVersionAsync("tsc", "--version");
            return (true, $"tsc {version}");
        }

        // Check npx tsc as fallback
        if (await ProcessRunner.CommandExistsAsync("npx"))
        {
            var result = await ProcessRunner.RunAsync("npx", "tsc --version",
                new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);
            if (result.IsSuccess)
            {
                return (true, $"npx tsc {result.Stdout.Trim()}");
            }
        }

        return (false, "TypeScript compiler (tsc) not found. Install via: npm install -g typescript");
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

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var sourceFiles = new[] { "*.ts", "*.tsx", "*.mts", "*.cts" }
            .SelectMany(ext => Directory.GetFiles(srcDir, ext, SearchOption.AllDirectories))
            .Where(f => !f.EndsWith(".d.ts", StringComparison.OrdinalIgnoreCase))
            .ToArray();

        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No TypeScript source files found (.ts/.tsx/.mts/.cts)", logs, stopwatch.Elapsed);
        }

        logs.Add($"[TypeScript] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var tsConfig = module.Config.TypeScript ?? new TypeScriptConfig();
        var tscCmd = await ProcessRunner.CommandExistsAsync("tsc") ? "tsc" : "npx";
        var tscPrefix = tscCmd == "npx" ? "tsc " : "";

        // Check if tsconfig.json exists in module root
        var tsconfigPath = Path.Combine(module.ModulePath, "tsconfig.json");
        string args;

        if (File.Exists(tsconfigPath))
        {
            // Use existing tsconfig.json
            args = $"{tscPrefix}--project \"{tsconfigPath}\" --outDir \"{outputDir}\"";
            logs.Add("[TypeScript] Using existing tsconfig.json");
        }
        else
        {
            // Build args from config
            var fileList = string.Join(" ", sourceFiles.Select(f => $"\"{f}\""));
            var strict = tsConfig.Strict ? "--strict " : "";
            var declaration = tsConfig.Declaration ? "--declaration " : "";
            args = $"{tscPrefix}--target {tsConfig.Target} --module {tsConfig.Module} " +
                   $"{strict}{declaration}--outDir \"{outputDir}\" {fileList}";
        }

        logs.Add($"$ {tscCmd} {args}");

        var result = await ProcessRunner.RunAsync(
            tscCmd, args,
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
            var errorMsg = !string.IsNullOrEmpty(result.Stderr) ? result.Stderr : result.Stdout;
            return CompileResult.Failure($"TypeScript compilation failed: {errorMsg}", logs, stopwatch.Elapsed);
        }

        logs.Add("[TypeScript] Compilation succeeded");

        // Collect output artifacts
        var artifacts = new List<string>();
        if (Directory.Exists(outputDir))
        {
            artifacts.AddRange(Directory.GetFiles(outputDir, "*.js", SearchOption.AllDirectories));
            artifacts.AddRange(Directory.GetFiles(outputDir, "*.d.ts", SearchOption.AllDirectories));
            artifacts.AddRange(Directory.GetFiles(outputDir, "*.js.map", SearchOption.AllDirectories));
        }

        // Copy package.json if present
        var packageJson = Path.Combine(module.ModulePath, "package.json");
        if (File.Exists(packageJson))
        {
            var destPackageJson = Path.Combine(outputDir, "package.json");
            File.Copy(packageJson, destPackageJson, overwrite: true);
            artifacts.Add(destPackageJson);
            logs.Add("[TypeScript] Copied package.json");
        }

        logs.Add($"[TypeScript] Produced {artifacts.Count} artifact(s)");

        stopwatch.Stop();
        return artifacts.Count > 0
            ? CompileResult.Success(artifacts.ToArray(), logs, stopwatch.Elapsed)
            : CompileResult.Failure("No output artifacts produced", logs, stopwatch.Elapsed);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        // Try .d.ts file first, fall back to .ts source
        var dtsPath = artifactPath.EndsWith(".js", StringComparison.OrdinalIgnoreCase)
            ? artifactPath.Substring(0, artifactPath.Length - 3) + ".d.ts"
            : artifactPath;

        var filePath = File.Exists(dtsPath) ? dtsPath : artifactPath;
        if (!File.Exists(filePath))
        {
            return null;
        }

        var content = await File.ReadAllTextAsync(filePath, ct);

        // Match: export function funcName(params): returnType
        var exportFuncRegex = new Regex(
            @"export\s+(?:declare\s+)?function\s+(\w+)\s*\(([^)]*)\)\s*(?::\s*(\w[\w<>\[\],\s|]*))?\s*[;{]",
            RegexOptions.Compiled);

        foreach (Match match in exportFuncRegex.Matches(content))
        {
            var funcName = match.Groups[1].Value;
            var paramList = match.Groups[2].Value;
            var returnType = match.Groups[3].Success ? MapTsTypeToForge(match.Groups[3].Value.Trim()) : "void";
            var parameters = ParseTsParameters(paramList);

            exports.Add(new ExportedFunction
            {
                Name = funcName,
                ReturnType = returnType,
                Parameters = parameters
            });
        }

        // Match: export class ClassName
        var exportClassRegex = new Regex(
            @"export\s+(?:declare\s+)?class\s+(\w+)",
            RegexOptions.Compiled);

        foreach (Match match in exportClassRegex.Matches(content))
        {
            var className = match.Groups[1].Value;
            if (exports.Any(e => e.Name == className)) continue;

            exports.Add(new ExportedFunction
            {
                Name = className,
                ReturnType = "void",
                Parameters = Array.Empty<FunctionParameter>()
            });
        }

        // Match: export interface InterfaceName
        var exportInterfaceRegex = new Regex(
            @"export\s+(?:declare\s+)?interface\s+(\w+)",
            RegexOptions.Compiled);

        foreach (Match match in exportInterfaceRegex.Matches(content))
        {
            var ifaceName = match.Groups[1].Value;
            if (exports.Any(e => e.Name == ifaceName)) continue;

            exports.Add(new ExportedFunction
            {
                Name = ifaceName,
                ReturnType = "void",
                Parameters = Array.Empty<FunctionParameter>()
            });
        }

        // Match: export const/let funcName = (params) =>
        var exportArrowRegex = new Regex(
            @"export\s+(?:const|let)\s+(\w+)\s*=\s*\(([^)]*)\)\s*(?::\s*(\w[\w<>\[\],\s|]*))?\s*=>",
            RegexOptions.Compiled);

        foreach (Match match in exportArrowRegex.Matches(content))
        {
            var funcName = match.Groups[1].Value;
            if (exports.Any(e => e.Name == funcName)) continue;

            var paramList = match.Groups[2].Value;
            var returnType = match.Groups[3].Success ? MapTsTypeToForge(match.Groups[3].Value.Trim()) : "void";
            var parameters = ParseTsParameters(paramList);

            exports.Add(new ExportedFunction
            {
                Name = funcName,
                ReturnType = returnType,
                Parameters = parameters
            });
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
                Name = "typescript",
                Abi = "node",
                Mode = "compiled"
            },
            Exports = exports.ToArray()
        };
    }

    private static FunctionParameter[] ParseTsParameters(string paramList)
    {
        if (string.IsNullOrWhiteSpace(paramList)) return Array.Empty<FunctionParameter>();

        return paramList.Split(',', StringSplitOptions.RemoveEmptyEntries)
            .Select(p => p.Trim())
            .Where(p => !string.IsNullOrEmpty(p))
            .Select(p =>
            {
                var optional = p.Contains('?');
                var parts = p.Replace("?", "").Split(':', 2);
                var name = parts[0].Trim();
                var type = parts.Length > 1 ? MapTsTypeToForge(parts[1].Split('=')[0].Trim()) : "any";
                return new FunctionParameter { Name = name, Type = type };
            })
            .ToArray();
    }

    private static string MapTsTypeToForge(string tsType)
    {
        return tsType.ToLowerInvariant() switch
        {
            "number" => "f64",
            "string" => "string",
            "boolean" => "bool",
            "void" => "void",
            "undefined" => "void",
            "null" => "void",
            "any" => "any",
            "unknown" => "any",
            "object" => "any",
            "bigint" => "i64",
            _ => "any"
        };
    }

    public GlueCodeResult GenerateGlue(InterfaceDescription sourceInterface, string targetLanguage)
    {
        return targetLanguage.ToLowerInvariant() switch
        {
            "csharp" or "c#" => GenerateCSharpGlue(sourceInterface),
            "rust" => GenerateRustGlue(sourceInterface),
            "javascript" or "js" => GenerateJavaScriptGlue(sourceInterface),
            _ => GlueCodeResult.Failure($"Unsupported target language: {targetLanguage}")
        };
    }

    private static GlueCodeResult GenerateCSharpGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;
        var pascalName = ToPascalCase(moduleName);

        sb.AppendLine("// Auto-generated by MAIDOS-Forge TypeScript Plugin");
        sb.AppendLine("// TypeScript interop via Node.js child process");
        sb.AppendLine("using System.Diagnostics;");
        sb.AppendLine("using System.Text.Json;");
        sb.AppendLine();
        sb.AppendLine($"namespace {pascalName}.Interop;");
        sb.AppendLine();
        sb.AppendLine($"/// <summary>TypeScript interop wrapper for {moduleName}</summary>");
        sb.AppendLine($"internal static class {pascalName}TsInterop");
        sb.AppendLine("{");
        sb.AppendLine($"    private const string ModulePath = \"./{moduleName}\";");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var returnType = MapToCSharpType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{MapToCSharpType(p.Type)} {p.Name}"));

            var argsList = string.Join(", ", export.Parameters.Select(p => p.Name));
            var argsJson = export.Parameters.Count > 0
                ? $"JsonSerializer.Serialize(new object[] {{ {argsList} }})"
                : "\"[]\"";

            sb.AppendLine($"    /// <summary>Invoke TS function: {export.Name}</summary>");
            sb.AppendLine($"    public static {returnType} {ToPascalCase(export.Name)}({parms})");
            sb.AppendLine("    {");
            sb.AppendLine($"        var args = {argsJson};");
            sb.AppendLine($"        var script = $\"const m = require('{moduleName}'); \" +");
            sb.AppendLine($"            $\"console.log(JSON.stringify(m.{export.Name}(...JSON.parse(args))))\";");
            sb.AppendLine("        var psi = new ProcessStartInfo(\"node\", $\"-e \\\"{script}\\\"\")");
            sb.AppendLine("        {");
            sb.AppendLine("            RedirectStandardOutput = true,");
            sb.AppendLine("            UseShellExecute = false");
            sb.AppendLine("        };");
            sb.AppendLine("        using var proc = Process.Start(psi);");
            sb.AppendLine("        var output = proc?.StandardOutput.ReadToEnd();");
            sb.AppendLine("        proc?.WaitForExit();");

            if (returnType != "void")
            {
                sb.AppendLine($"        return JsonSerializer.Deserialize<{returnType}>(output ?? \"null\");");
            }

            sb.AppendLine("    }");
            sb.AppendLine();
        }

        sb.AppendLine("}");

        return GlueCodeResult.Success(sb.ToString(), $"{pascalName}.TsInterop.cs", "csharp");
    }

    private static GlueCodeResult GenerateRustGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;
        var snakeName = ToSnakeCase(moduleName);

        sb.AppendLine("// Auto-generated by MAIDOS-Forge TypeScript Plugin");
        sb.AppendLine("// TypeScript interop via Node.js subprocess");
        sb.AppendLine("use std::process::Command;");
        sb.AppendLine("use serde_json::Value;");
        sb.AppendLine();
        sb.AppendLine($"/// TypeScript module: {moduleName}");
        sb.AppendLine($"pub mod {snakeName} {{");
        sb.AppendLine("    use super::*;");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var returnType = MapToRustType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{ToSnakeCase(p.Name)}: {MapToRustType(p.Type)}"));

            sb.AppendLine($"    /// Invoke TS function: {export.Name}");

            if (returnType == "()")
            {
                sb.AppendLine($"    pub fn {ToSnakeCase(export.Name)}({parms}) {{");
            }
            else
            {
                sb.AppendLine($"    pub fn {ToSnakeCase(export.Name)}({parms}) -> {returnType} {{");
            }

            sb.AppendLine($"        let script = format!(");
            sb.AppendLine($"            \"const m = require('./{moduleName}'); console.log(JSON.stringify(m.{export.Name}()))\"");
            sb.AppendLine($"        );");
            sb.AppendLine("        let output = Command::new(\"node\")");
            sb.AppendLine("            .args(&[\"-e\", &script])");
            sb.AppendLine("            .output()");
            sb.AppendLine("            .expect(\"Failed to invoke node\");");

            if (returnType != "()")
            {
                sb.AppendLine("        let stdout = String::from_utf8_lossy(&output.stdout);");
                sb.AppendLine($"        serde_json::from_str(&stdout).unwrap_or_default()");
            }

            sb.AppendLine("    }");
            sb.AppendLine();
        }

        sb.AppendLine("}");

        return GlueCodeResult.Success(sb.ToString(), $"{snakeName}_ts_ffi.rs", "rust");
    }

    private static GlueCodeResult GenerateJavaScriptGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;

        sb.AppendLine($"// Auto-generated by MAIDOS-Forge TypeScript Plugin");
        sb.AppendLine($"// JavaScript type stubs for {moduleName}");
        sb.AppendLine($"// These are the compiled JS exports from the TypeScript source");
        sb.AppendLine();
        sb.AppendLine($"const {moduleName} = require('./{moduleName}');");
        sb.AppendLine();
        sb.AppendLine("module.exports = {");

        for (int i = 0; i < source.Exports.Count; i++)
        {
            var export = source.Exports[i];
            var comma = i < source.Exports.Count - 1 ? "," : "";
            sb.AppendLine($"    {export.Name}: {moduleName}.{export.Name}{comma}");
        }

        sb.AppendLine("};");

        return GlueCodeResult.Success(sb.ToString(), $"{moduleName}_wrapper.js", "javascript");
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
        "any" => "object",
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
        "any" => "Value",
        _ => "Value"
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
