// MAIDOS-Forge JavaScript Language Plugin
// UEP v1.7C Compliant - Zero Technical Debt
// Standalone Plugin Module

using System.Text;
using System.Text.RegularExpressions;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.JavaScript;

/// <summary>
/// JavaScript language plugin - supports Node.js/Bun runtime validation and syntax checking
/// </summary>
/// <impl>
/// APPROACH: Use node --check for syntax validation, copy sources to output (interpreted language)
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: No runtime found returns error
/// </impl>
public sealed class JavaScriptPlugin : ILanguagePlugin
{
    private string _runtime = "node";

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "javascript",
        SupportedExtensions = new[] { ".js", ".mjs", ".cjs" },
        SupportsNativeCompilation = false,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "node", "bun" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (await ProcessRunner.CommandExistsAsync("node"))
        {
            var version = await ProcessRunner.GetVersionAsync("node", "--version");
            _runtime = "node";
            return (true, $"node {version}");
        }

        if (await ProcessRunner.CommandExistsAsync("bun"))
        {
            var version = await ProcessRunner.GetVersionAsync("bun", "--version");
            _runtime = "bun";
            return (true, $"bun {version}");
        }

        return (false, "Neither node nor bun found");
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

        logs.Add($"[JavaScript] Using: {toolchainMsg}");

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var sourceFiles = Directory.GetFiles(srcDir, "*.js", SearchOption.AllDirectories)
            .Concat(Directory.GetFiles(srcDir, "*.mjs", SearchOption.AllDirectories))
            .Concat(Directory.GetFiles(srcDir, "*.cjs", SearchOption.AllDirectories))
            .ToArray();

        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No .js/.mjs/.cjs source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[JavaScript] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        // Syntax validation pass: use node --check or bun --check on each file
        foreach (var sourceFile in sourceFiles)
        {
            var checkArgs = $"--check \"{sourceFile}\"";
            logs.Add($"$ {_runtime} {checkArgs}");

            var result = await ProcessRunner.RunAsync(
                _runtime, checkArgs,
                new ProcessConfig
                {
                    WorkingDirectory = module.ModulePath,
                    Timeout = TimeSpan.FromMinutes(2)
                }, ct);

            if (!string.IsNullOrEmpty(result.Stderr))
            {
                logs.Add(result.Stderr);
            }

            if (!result.IsSuccess)
            {
                stopwatch.Stop();
                return CompileResult.Failure(
                    $"Syntax error in {Path.GetFileName(sourceFile)}: {result.Stderr}",
                    logs, stopwatch.Elapsed);
            }
        }

        logs.Add("[JavaScript] Syntax validation passed");

        // Copy validated source files to output directory (JS is interpreted)
        var artifacts = new List<string>();

        foreach (var sourceFile in sourceFiles)
        {
            var relativePath = Path.GetRelativePath(srcDir, sourceFile);
            var destPath = Path.Combine(outputDir, relativePath);

            var destDir = Path.GetDirectoryName(destPath);
            if (destDir is not null)
            {
                Directory.CreateDirectory(destDir);
            }

            File.Copy(sourceFile, destPath, overwrite: true);
            artifacts.Add(destPath);
        }

        // Copy package.json if present
        var packageJson = Path.Combine(module.ModulePath, "package.json");
        if (File.Exists(packageJson))
        {
            var destPackageJson = Path.Combine(outputDir, "package.json");
            File.Copy(packageJson, destPackageJson, overwrite: true);
            artifacts.Add(destPackageJson);
            logs.Add("[JavaScript] Copied package.json");
        }

        logs.Add($"[JavaScript] Copied {sourceFiles.Length} file(s) to output");

        stopwatch.Stop();
        return CompileResult.Success(artifacts.ToArray(), logs, stopwatch.Elapsed);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        // Parse module.exports / export statements from JS source files
        if (!File.Exists(artifactPath))
        {
            return null;
        }

        var content = await File.ReadAllTextAsync(artifactPath, ct);

        // Match: module.exports.funcName = function(...)
        var moduleExportsRegex = new Regex(
            @"module\.exports\.(\w+)\s*=\s*function\s*\(([^)]*)\)",
            RegexOptions.Compiled);

        foreach (Match match in moduleExportsRegex.Matches(content))
        {
            var funcName = match.Groups[1].Value;
            var paramList = match.Groups[2].Value;
            var parameters = ParseJsParameters(paramList);

            exports.Add(new ExportedFunction
            {
                Name = funcName,
                ReturnType = "void",
                Parameters = parameters
            });
        }

        // Match: exports.funcName = function(...)
        var exportsRegex = new Regex(
            @"exports\.(\w+)\s*=\s*function\s*\(([^)]*)\)",
            RegexOptions.Compiled);

        foreach (Match match in exportsRegex.Matches(content))
        {
            var funcName = match.Groups[1].Value;
            if (exports.Any(e => e.Name == funcName)) continue;

            var paramList = match.Groups[2].Value;
            var parameters = ParseJsParameters(paramList);

            exports.Add(new ExportedFunction
            {
                Name = funcName,
                ReturnType = "void",
                Parameters = parameters
            });
        }

        // Match: export function funcName(...)
        var exportFuncRegex = new Regex(
            @"export\s+function\s+(\w+)\s*\(([^)]*)\)",
            RegexOptions.Compiled);

        foreach (Match match in exportFuncRegex.Matches(content))
        {
            var funcName = match.Groups[1].Value;
            if (exports.Any(e => e.Name == funcName)) continue;

            var paramList = match.Groups[2].Value;
            var parameters = ParseJsParameters(paramList);

            exports.Add(new ExportedFunction
            {
                Name = funcName,
                ReturnType = "void",
                Parameters = parameters
            });
        }

        // Match: module.exports = { funcName, ... } (shorthand exports)
        var moduleExportsObjRegex = new Regex(
            @"module\.exports\s*=\s*\{([^}]+)\}",
            RegexOptions.Compiled);

        var objMatch = moduleExportsObjRegex.Match(content);
        if (objMatch.Success)
        {
            var names = objMatch.Groups[1].Value
                .Split(',', StringSplitOptions.RemoveEmptyEntries)
                .Select(n => n.Trim().Split(':')[0].Trim())
                .Where(n => !string.IsNullOrEmpty(n) && Regex.IsMatch(n, @"^\w+$"));

            foreach (var name in names)
            {
                if (exports.Any(e => e.Name == name)) continue;

                exports.Add(new ExportedFunction
                {
                    Name = name,
                    ReturnType = "void",
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
            Language = new InterfaceLanguage
            {
                Name = "javascript",
                Abi = "node",
                Mode = "interpreted"
            },
            Exports = exports.ToArray()
        };
    }

    private static FunctionParameter[] ParseJsParameters(string paramList)
    {
        if (string.IsNullOrWhiteSpace(paramList)) return Array.Empty<FunctionParameter>();

        return paramList.Split(',', StringSplitOptions.RemoveEmptyEntries)
            .Select(p => p.Trim())
            .Where(p => !string.IsNullOrEmpty(p))
            .Select(p => new FunctionParameter
            {
                Name = p.Split('=')[0].Trim(),
                Type = "any"
            })
            .ToArray();
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge JavaScript Plugin");
        sb.AppendLine("// JavaScript interop via Node.js child process or Jint engine");
        sb.AppendLine("using System.Diagnostics;");
        sb.AppendLine("using System.Text.Json;");
        sb.AppendLine();
        sb.AppendLine($"namespace {pascalName}.Interop;");
        sb.AppendLine();
        sb.AppendLine($"/// <summary>");
        sb.AppendLine($"/// JavaScript interop wrapper for {moduleName}");
        sb.AppendLine($"/// </summary>");
        sb.AppendLine($"internal static class {pascalName}JsInterop");
        sb.AppendLine("{");
        sb.AppendLine($"    private const string ModulePath = \"{moduleName}\";");
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

            sb.AppendLine($"    /// <summary>Invoke JS function: {export.Name}</summary>");
            sb.AppendLine($"    public static {returnType} {ToPascalCase(export.Name)}({parms})");
            sb.AppendLine("    {");
            sb.AppendLine($"        var args = {argsJson};");
            sb.AppendLine($"        var script = $\"const m = require('./{moduleName}'); \" +");
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

        return GlueCodeResult.Success(sb.ToString(), $"{pascalName}.Interop.cs", "csharp");
    }

    private static GlueCodeResult GenerateRustGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;
        var snakeName = ToSnakeCase(moduleName);

        sb.AppendLine("// Auto-generated by MAIDOS-Forge JavaScript Plugin");
        sb.AppendLine("// JavaScript interop via std::process::Command (Node.js)");
        sb.AppendLine("#![allow(non_snake_case)]");
        sb.AppendLine();
        sb.AppendLine("use std::process::Command;");
        sb.AppendLine("use serde_json::Value;");
        sb.AppendLine();
        sb.AppendLine($"/// JavaScript module: {moduleName}");
        sb.AppendLine($"pub mod {snakeName} {{");
        sb.AppendLine("    use super::*;");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var returnType = MapToRustType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{ToSnakeCase(p.Name)}: {MapToRustType(p.Type)}"));

            var argsFormat = export.Parameters.Count > 0
                ? string.Join(", ", export.Parameters.Select(p => $"{{{ToSnakeCase(p.Name)}}}"))
                : "";

            sb.AppendLine($"    /// Invoke JS function: {export.Name}");

            if (returnType == "()")
            {
                sb.AppendLine($"    pub fn {ToSnakeCase(export.Name)}({parms}) {{");
            }
            else
            {
                sb.AppendLine($"    pub fn {ToSnakeCase(export.Name)}({parms}) -> {returnType} {{");
            }

            sb.AppendLine($"        let script = format!(");
            sb.AppendLine($"            \"const m = require('./{moduleName}'); console.log(JSON.stringify(m.{export.Name}({argsFormat})))\"");
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

        return GlueCodeResult.Success(sb.ToString(), $"{snakeName}_js_ffi.rs", "rust");
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
