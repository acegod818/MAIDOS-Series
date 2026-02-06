// MAIDOS-Forge Ruby Language Plugin
// UEP v1.7C Compliant - Zero Technical Debt
// Standalone Plugin Module

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.Ruby;

/// <summary>
/// Ruby Language Plugin - Interpreted language with syntax validation
/// </summary>
/// <impl>
/// APPROACH: Run ruby -c for syntax check, copy source to output (interpreted)
/// CALLS: ProcessRunner.RunAsync(), ProcessRunner.CommandExistsAsync()
/// EDGES: No ruby found returns error; rubocop optional linting
/// </impl>
public sealed class RubyPlugin : ILanguagePlugin
{
    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "ruby",
        SupportedExtensions = new[] { ".rb" },
        SupportsNativeCompilation = false,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (await ProcessRunner.CommandExistsAsync("ruby"))
        {
            var version = await ProcessRunner.GetVersionAsync("ruby", "--version");

            var hasRubocop = await ProcessRunner.CommandExistsAsync("rubocop");
            var extra = hasRubocop ? " (rubocop available)" : "";

            return (true, $"ruby {version}{extra}");
        }

        return (false, "ruby not found. Install from https://www.ruby-lang.org/");
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

        logs.Add($"[Ruby] Using: {toolchainMsg}");

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var sourceFiles = Directory.GetFiles(srcDir, "*.rb", SearchOption.AllDirectories);
        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No .rb source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[Ruby] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var artifacts = new List<string>();

        // Ruby is interpreted - validate syntax then copy to output
        foreach (var sourceFile in sourceFiles)
        {
            var fileName = Path.GetFileName(sourceFile);
            logs.Add($"[Ruby] Checking syntax: {fileName}");

            // Run ruby -c for syntax validation
            var syntaxResult = await ProcessRunner.RunAsync(
                "ruby", $"-c \"{sourceFile}\"",
                new ProcessConfig
                {
                    WorkingDirectory = module.ModulePath,
                    Timeout = TimeSpan.FromMinutes(1)
                }, ct);

            if (!string.IsNullOrEmpty(syntaxResult.Stdout))
            {
                logs.Add(syntaxResult.Stdout.Trim());
            }

            if (!string.IsNullOrEmpty(syntaxResult.Stderr))
            {
                logs.Add(syntaxResult.Stderr.Trim());
            }

            if (!syntaxResult.IsSuccess)
            {
                stopwatch.Stop();
                return CompileResult.Failure(
                    $"Syntax check failed for {fileName}: {syntaxResult.Stderr}",
                    logs, stopwatch.Elapsed);
            }

            // Copy validated source to output directory
            var relativePath = Path.GetRelativePath(srcDir, sourceFile);
            var destPath = Path.Combine(outputDir, relativePath);
            var destDir = Path.GetDirectoryName(destPath);
            if (!string.IsNullOrEmpty(destDir))
            {
                Directory.CreateDirectory(destDir);
            }

            File.Copy(sourceFile, destPath, overwrite: true);
            artifacts.Add(destPath);
            logs.Add($"[Ruby] Copied: {relativePath}");
        }

        // Optional rubocop linting (non-blocking)
        if (await ProcessRunner.CommandExistsAsync("rubocop"))
        {
            logs.Add("[Ruby] Running rubocop lint (advisory)...");
            var lintResult = await ProcessRunner.RunAsync(
                "rubocop", $"--format simple \"{srcDir}\"",
                new ProcessConfig
                {
                    WorkingDirectory = module.ModulePath,
                    Timeout = TimeSpan.FromMinutes(2)
                }, ct);

            if (!string.IsNullOrEmpty(lintResult.Stdout))
            {
                foreach (var line in lintResult.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries))
                {
                    logs.Add($"[rubocop] {line}");
                }
            }
        }

        stopwatch.Stop();
        logs.Add($"[Ruby] Syntax check passed for {artifacts.Count} file(s)");

        return CompileResult.Success(artifacts.ToArray(), logs, stopwatch.Elapsed);
    }

    public Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        if (File.Exists(artifactPath))
        {
            var content = File.ReadAllText(artifactPath);
            var lines = content.Split('\n');

            // Extract module and method definitions
            string? currentModule = null;
            foreach (var rawLine in lines)
            {
                var line = rawLine.Trim();

                // Detect module declarations
                if (line.StartsWith("module ", StringComparison.Ordinal))
                {
                    currentModule = line.Replace("module ", "").Trim();
                    continue;
                }

                // Detect method definitions: def method_name(...)
                if (line.StartsWith("def ", StringComparison.Ordinal))
                {
                    var methodPart = line.Substring(4).Trim();
                    var parenIdx = methodPart.IndexOf('(');
                    var methodName = parenIdx >= 0
                        ? methodPart.Substring(0, parenIdx).Trim()
                        : methodPart.Split(' ', StringSplitOptions.RemoveEmptyEntries)[0];

                    if (!string.IsNullOrEmpty(methodName))
                    {
                        var qualifiedName = currentModule != null
                            ? $"{currentModule}.{methodName}"
                            : methodName;

                        exports.Add(new ExportedFunction
                        {
                            Name = qualifiedName,
                            ReturnType = "void",
                            Parameters = Array.Empty<FunctionParameter>()
                        });
                    }
                }
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
                Name = "ruby",
                Abi = "interpreted"
            },
            Exports = exports.ToArray()
        });
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Ruby Plugin");
        sb.AppendLine("// Ruby FFI bridge via IronRuby or process invocation");
        sb.AppendLine("using System.Diagnostics;");
        sb.AppendLine("using System.Runtime.InteropServices;");
        sb.AppendLine();
        sb.AppendLine($"namespace {pascalName}.Interop;");
        sb.AppendLine();
        sb.AppendLine($"/// <summary>Ruby interop wrapper for {moduleName}</summary>");
        sb.AppendLine($"public static class {pascalName}Native");
        sb.AppendLine("{");
        sb.AppendLine($"    private const string ScriptName = \"{moduleName}.rb\";");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var returnType = MapToCSharpType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{MapToCSharpType(p.Type)} {p.Name}"));

            sb.AppendLine($"    /// <summary>Calls Ruby method: {export.Name}</summary>");
            sb.AppendLine($"    public static {returnType} {ToPascalCase(export.Name)}({parms})");
            sb.AppendLine("    {");
            sb.AppendLine($"        var psi = new ProcessStartInfo(\"ruby\", $\"-e \\\"require './{moduleName}'; {export.Name}\\\"\")");
            sb.AppendLine("        {");
            sb.AppendLine("            RedirectStandardOutput = true,");
            sb.AppendLine("            UseShellExecute = false");
            sb.AppendLine("        };");
            sb.AppendLine("        using var proc = Process.Start(psi);");
            sb.AppendLine("        proc?.WaitForExit();");
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Ruby Plugin");
        sb.AppendLine("// Ruby FFI bridge via std::process::Command");
        sb.AppendLine("use std::process::Command;");
        sb.AppendLine();
        sb.AppendLine($"/// Ruby interop wrapper for {moduleName}");
        sb.AppendLine($"pub mod {snakeName} {{");
        sb.AppendLine($"    const SCRIPT: &str = \"{moduleName}.rb\";");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var returnType = MapToRustType(export.ReturnType);
            var parms = string.Join(", ", export.Parameters.Select(p =>
                $"{ToSnakeCase(p.Name)}: {MapToRustType(p.Type)}"));

            var retDecl = returnType == "()" ? "" : $" -> {returnType}";

            sb.AppendLine($"    /// Calls Ruby method: {export.Name}");
            sb.AppendLine($"    pub fn {ToSnakeCase(export.Name)}({parms}){retDecl} {{");
            sb.AppendLine($"        let output = Command::new(\"ruby\")");
            sb.AppendLine($"            .arg(\"-e\")");
            sb.AppendLine($"            .arg(format!(\"require './{{}}'; {export.Name}\", SCRIPT))");
            sb.AppendLine("            .output()");
            sb.AppendLine("            .expect(\"Failed to execute ruby\");");

            if (returnType != "()")
            {
                sb.AppendLine("        let stdout = String::from_utf8_lossy(&output.stdout);");
                sb.AppendLine($"        stdout.trim().parse::<{returnType}>().unwrap_or_default()");
            }

            sb.AppendLine("    }");
            sb.AppendLine();
        }

        sb.AppendLine("}");

        return GlueCodeResult.Success(sb.ToString(), $"{snakeName}_ffi.rs", "rust");
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
        _ => "i32"
    };

    private static string ToPascalCase(string s)
    {
        if (string.IsNullOrEmpty(s)) return s;
        var parts = s.Split('-', '_', '.');
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
