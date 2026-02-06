// MAIDOS-Forge Python Language Plugin

using System.Text;
using System.Text.RegularExpressions;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.python;

/// <summary>Python language plugin - supports syntax validation, bytecode compilation, and Cython</summary>
public sealed class PythonPlugin : ILanguagePlugin
{
    private string _runtime = "python3";

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "python",
        SupportedExtensions = new[] { ".py", ".pyx", ".pxd" },
        SupportsNativeCompilation = false,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "cpython", "cython", "pypy" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (await ProcessRunner.CommandExistsAsync("python3"))
        {
            var version = await ProcessRunner.GetVersionAsync("python3", "--version");
            _runtime = "python3";
            return (true, $"python3 {version}");
        }

        if (await ProcessRunner.CommandExistsAsync("python"))
        {
            var version = await ProcessRunner.GetVersionAsync("python", "--version");
            _runtime = "python";
            return (true, $"python {version}");
        }

        return (false, "Neither python3 nor python found");
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

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var sourceFiles = Directory.GetFiles(srcDir, "*.py", SearchOption.AllDirectories)
            .Concat(Directory.GetFiles(srcDir, "*.pyx", SearchOption.AllDirectories))
            .ToArray();

        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No .py/.pyx source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[Python] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var pyConfig = module.Config.Python ?? new PythonConfig();

        // Syntax validation pass: py_compile on each file
        foreach (var sourceFile in sourceFiles)
        {
            if (!sourceFile.EndsWith(".py", StringComparison.OrdinalIgnoreCase)) continue;

            var checkScript = $"-m py_compile \"{sourceFile}\"";
            logs.Add($"$ {_runtime} {checkScript}");

            var result = await ProcessRunner.RunAsync(
                _runtime, checkScript,
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

        logs.Add("[Python] Syntax validation passed");

        // Compile to bytecode (.pyc)
        var compileScript = $"-m compileall -b -f -q \"{srcDir}\"";
        if (pyConfig.Optimize > 0)
        {
            compileScript = $"-O{(pyConfig.Optimize > 1 ? "O" : "")} " + compileScript;
        }
        logs.Add($"$ {_runtime} {compileScript}");

        var compileResult = await ProcessRunner.RunAsync(
            _runtime, compileScript,
            new ProcessConfig
            {
                WorkingDirectory = module.ModulePath,
                Timeout = TimeSpan.FromMinutes(5)
            }, ct);

        if (!string.IsNullOrEmpty(compileResult.Stderr))
        {
            logs.Add(compileResult.Stderr);
        }

        // Copy source files + bytecode to output
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

            // Copy .pyc if exists
            var pycFile = sourceFile + "c";
            if (File.Exists(pycFile))
            {
                var pycDest = destPath + "c";
                File.Copy(pycFile, pycDest, overwrite: true);
                artifacts.Add(pycDest);
            }

            // Check __pycache__ for compiled files
            var cacheDir = Path.Combine(Path.GetDirectoryName(sourceFile) ?? "", "__pycache__");
            if (Directory.Exists(cacheDir))
            {
                var baseName = Path.GetFileNameWithoutExtension(sourceFile);
                var cachedFiles = Directory.GetFiles(cacheDir, $"{baseName}*.pyc");
                foreach (var cached in cachedFiles)
                {
                    var destCacheDir = Path.Combine(outputDir, "__pycache__");
                    Directory.CreateDirectory(destCacheDir);
                    var destCached = Path.Combine(destCacheDir, Path.GetFileName(cached));
                    File.Copy(cached, destCached, overwrite: true);
                    artifacts.Add(destCached);
                }
            }
        }

        // Copy requirements.txt if present
        var reqFile = Path.Combine(module.ModulePath, "requirements.txt");
        if (File.Exists(reqFile))
        {
            var destReq = Path.Combine(outputDir, "requirements.txt");
            File.Copy(reqFile, destReq, overwrite: true);
            artifacts.Add(destReq);
            logs.Add("[Python] Copied requirements.txt");
        }

        // Copy setup.py / pyproject.toml if present
        foreach (var meta in new[] { "setup.py", "pyproject.toml", "setup.cfg" })
        {
            var metaFile = Path.Combine(module.ModulePath, meta);
            if (File.Exists(metaFile))
            {
                var destMeta = Path.Combine(outputDir, meta);
                File.Copy(metaFile, destMeta, overwrite: true);
                artifacts.Add(destMeta);
                logs.Add($"[Python] Copied {meta}");
            }
        }

        logs.Add($"[Python] Compiled and copied {sourceFiles.Length} file(s) to output");

        stopwatch.Stop();
        return artifacts.Count > 0
            ? CompileResult.Success(artifacts.ToArray(), logs, stopwatch.Elapsed)
            : CompileResult.Failure("No artifacts produced", logs, stopwatch.Elapsed);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        if (!File.Exists(artifactPath))
        {
            return null;
        }

        var content = await File.ReadAllTextAsync(artifactPath, ct);

        // Match: def func_name(params):
        var defRegex = new Regex(
            @"^def\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*(\w+))?\s*:",
            RegexOptions.Compiled | RegexOptions.Multiline);

        foreach (Match match in defRegex.Matches(content))
        {
            var funcName = match.Groups[1].Value;
            if (funcName.StartsWith("_")) continue; // Skip private functions

            var paramList = match.Groups[2].Value;
            var returnType = match.Groups[3].Success ? MapPythonTypeToForge(match.Groups[3].Value) : "void";
            var parameters = ParsePythonParameters(paramList);

            exports.Add(new ExportedFunction
            {
                Name = funcName,
                ReturnType = returnType,
                Parameters = parameters
            });
        }

        // Match: class ClassName:
        var classRegex = new Regex(
            @"^class\s+(\w+)\s*(?:\([^)]*\))?\s*:",
            RegexOptions.Compiled | RegexOptions.Multiline);

        foreach (Match match in classRegex.Matches(content))
        {
            var className = match.Groups[1].Value;
            if (className.StartsWith("_")) continue;

            exports.Add(new ExportedFunction
            {
                Name = className,
                ReturnType = "void",
                Parameters = Array.Empty<FunctionParameter>()
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
                Name = "python",
                Abi = "cpython",
                Mode = "interpreted"
            },
            Exports = exports.ToArray()
        };
    }

    private static FunctionParameter[] ParsePythonParameters(string paramList)
    {
        if (string.IsNullOrWhiteSpace(paramList)) return Array.Empty<FunctionParameter>();

        return paramList.Split(',', StringSplitOptions.RemoveEmptyEntries)
            .Select(p => p.Trim())
            .Where(p => !string.IsNullOrEmpty(p) && p != "self" && p != "cls")
            .Select(p =>
            {
                var parts = p.Split(':', 2);
                var name = parts[0].Split('=')[0].Trim();
                var type = parts.Length > 1 ? MapPythonTypeToForge(parts[1].Split('=')[0].Trim()) : "any";
                return new FunctionParameter { Name = name, Type = type };
            })
            .ToArray();
    }

    private static string MapPythonTypeToForge(string pyType)
    {
        return pyType.ToLowerInvariant() switch
        {
            "int" => "i64",
            "float" => "f64",
            "str" => "string",
            "bool" => "bool",
            "bytes" => "u8",
            "none" => "void",
            "list" => "any",
            "dict" => "any",
            "tuple" => "any",
            _ => "any"
        };
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Python Plugin");
        sb.AppendLine("// Python interop via Python.NET (pythonnet) or subprocess");
        sb.AppendLine("using System.Diagnostics;");
        sb.AppendLine("using System.Text.Json;");
        sb.AppendLine();
        sb.AppendLine($"namespace {pascalName}.Interop;");
        sb.AppendLine();
        sb.AppendLine($"/// <summary>Python interop wrapper for {moduleName}</summary>");
        sb.AppendLine($"internal static class {pascalName}PyInterop");
        sb.AppendLine("{");
        sb.AppendLine($"    private const string ModuleName = \"{moduleName}\";");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var returnType = MapToCSharpType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{MapToCSharpType(p.Type)} {p.Name}"));

            var argsJson = export.Parameters.Count > 0
                ? string.Join(", ", export.Parameters.Select(p => p.Name))
                : "";
            var argsBuild = export.Parameters.Count > 0
                ? $"JsonSerializer.Serialize(new object[] {{ {argsJson} }})"
                : "\"[]\"";

            sb.AppendLine($"    /// <summary>Invoke Python function: {export.Name}</summary>");
            sb.AppendLine($"    public static {returnType} {ToPascalCase(export.Name)}({parms})");
            sb.AppendLine("    {");
            sb.AppendLine($"        var args = {argsBuild};");
            sb.AppendLine($"        var script = $\"import {moduleName}; import json; \" +");
            sb.AppendLine($"            $\"print(json.dumps({moduleName}.{export.Name}(*json.loads(args))))\";");
            sb.AppendLine("        var psi = new ProcessStartInfo(\"python3\", $\"-c \\\"{script}\\\"\")");
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Python Plugin");
        sb.AppendLine("// Python interop via pyo3 or subprocess");
        sb.AppendLine("use std::process::Command;");
        sb.AppendLine("use serde_json::Value;");
        sb.AppendLine();
        sb.AppendLine($"/// Python module: {moduleName}");
        sb.AppendLine($"pub mod {snakeName} {{");
        sb.AppendLine("    use super::*;");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var returnType = MapToRustType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{ToSnakeCase(p.Name)}: {MapToRustType(p.Type)}"));

            sb.AppendLine($"    /// Invoke Python function: {export.Name}");

            if (returnType == "()")
            {
                sb.AppendLine($"    pub fn {ToSnakeCase(export.Name)}({parms}) {{");
            }
            else
            {
                sb.AppendLine($"    pub fn {ToSnakeCase(export.Name)}({parms}) -> {returnType} {{");
            }

            sb.AppendLine($"        let script = format!(");
            sb.AppendLine($"            \"import {moduleName}; import json; print(json.dumps({moduleName}.{export.Name}()))\"");
            sb.AppendLine($"        );");
            sb.AppendLine("        let output = Command::new(\"python3\")");
            sb.AppendLine("            .args(&[\"-c\", &script])");
            sb.AppendLine("            .output()");
            sb.AppendLine("            .expect(\"Failed to invoke python3\");");

            if (returnType != "()")
            {
                sb.AppendLine("        let stdout = String::from_utf8_lossy(&output.stdout);");
                sb.AppendLine($"        serde_json::from_str(&stdout).unwrap_or_default()");
            }

            sb.AppendLine("    }");
            sb.AppendLine();
        }

        sb.AppendLine("}");

        return GlueCodeResult.Success(sb.ToString(), $"{snakeName}_py_ffi.rs", "rust");
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
