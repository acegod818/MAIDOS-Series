// MAIDOS-Forge Java Language Plugin
// UEP v1.7C Compliant - Zero Technical Debt
// Standalone Plugin Module

using System.Text;
using System.Text.RegularExpressions;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.Java;

/// <summary>
/// Java language plugin - supports javac compilation and javap interface extraction
/// </summary>
/// <impl>
/// APPROACH: Invoke javac for compilation, javap for interface extraction
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: No javac found returns error
/// </impl>
public sealed class JavaPlugin : ILanguagePlugin
{
    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "java",
        SupportedExtensions = new[] { ".java" },
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[] { "jvm" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (!await ProcessRunner.CommandExistsAsync("javac"))
        {
            return (false, "javac not found (install JDK)");
        }

        var javacVersion = await ProcessRunner.GetVersionAsync("javac", "-version");

        if (!await ProcessRunner.CommandExistsAsync("java"))
        {
            return (false, "javac found but java runtime not found");
        }

        var javaVersion = await ProcessRunner.GetVersionAsync("java", "-version");

        return (true, $"javac {javacVersion}, java {javaVersion}");
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

        logs.Add($"[Java] Using: {toolchainMsg}");

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var sourceFiles = Directory.GetFiles(srcDir, "*.java", SearchOption.AllDirectories);
        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No .java source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[Java] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        // Build javac arguments
        var args = BuildCompileArgs(sourceFiles, outputDir, config);
        logs.Add($"$ javac {args}");

        var result = await ProcessRunner.RunAsync(
            "javac", args,
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

        // Collect .class artifacts
        var classFiles = Directory.GetFiles(outputDir, "*.class", SearchOption.AllDirectories);
        logs.Add($"[Java] Produced {classFiles.Length} class file(s)");

        // Package into JAR if multiple class files
        var artifacts = new List<string>(classFiles);

        if (classFiles.Length > 0)
        {
            var jarName = $"{module.Config.Name}.jar";
            var jarPath = Path.Combine(outputDir, jarName);

            var jarArgs = $"cf \"{jarPath}\" -C \"{outputDir}\" .";
            logs.Add($"$ jar {jarArgs}");

            var jarResult = await ProcessRunner.RunAsync("jar", jarArgs,
                new ProcessConfig { Timeout = TimeSpan.FromMinutes(2) }, ct);

            if (jarResult.IsSuccess && File.Exists(jarPath))
            {
                logs.Add($"[Java] Packaged into {jarName}");
                artifacts = new List<string> { jarPath };
            }
            else
            {
                logs.Add($"[Java] jar packaging failed: {jarResult.Stderr}, returning class files");
            }
        }

        stopwatch.Stop();
        return CompileResult.Success(artifacts.ToArray(), logs, stopwatch.Elapsed);
    }

    private static string BuildCompileArgs(
        string[] sourceFiles,
        string outputDir,
        CompileConfig config)
    {
        var args = new List<string>
        {
            "-d",
            $"\"{outputDir}\""
        };

        if (config.Profile == "debug")
        {
            args.Add("-g");
        }
        else
        {
            args.Add("-g:none");
        }

        args.Add("-Xlint:all");
        args.Add("-encoding");
        args.Add("UTF-8");

        foreach (var sourceFile in sourceFiles)
        {
            args.Add($"\"{sourceFile}\"");
        }

        return string.Join(" ", args);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        // Use javap to extract public methods from class files or JAR
        string javapTarget;
        string workDir;

        if (artifactPath.EndsWith(".jar", StringComparison.OrdinalIgnoreCase))
        {
            // List classes in JAR first
            var jarListResult = await ProcessRunner.RunAsync(
                "jar", $"tf \"{artifactPath}\"",
                new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);

            if (!jarListResult.IsSuccess || string.IsNullOrEmpty(jarListResult.Stdout))
            {
                return CreateEmptyInterface(artifactPath);
            }

            var classNames = jarListResult.Stdout
                .Split('\n', StringSplitOptions.RemoveEmptyEntries)
                .Where(l => l.EndsWith(".class") && !l.Contains('$'))
                .Select(l => l.Replace('/', '.').Replace(".class", ""))
                .ToArray();

            foreach (var className in classNames)
            {
                var javapResult = await ProcessRunner.RunAsync(
                    "javap", $"-public -cp \"{artifactPath}\" {className}",
                    new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);

                if (javapResult.IsSuccess && !string.IsNullOrEmpty(javapResult.Stdout))
                {
                    ParseJavapOutput(javapResult.Stdout, exports);
                }
            }
        }
        else if (artifactPath.EndsWith(".class", StringComparison.OrdinalIgnoreCase))
        {
            workDir = Path.GetDirectoryName(artifactPath) ?? ".";
            var className = Path.GetFileNameWithoutExtension(artifactPath);

            var javapResult = await ProcessRunner.RunAsync(
                "javap", $"-public -cp \"{workDir}\" {className}",
                new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);

            if (javapResult.IsSuccess && !string.IsNullOrEmpty(javapResult.Stdout))
            {
                ParseJavapOutput(javapResult.Stdout, exports);
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
                Name = "java",
                Abi = "jni",
                Mode = "jvm"
            },
            Exports = exports.ToArray()
        };
    }

    private static void ParseJavapOutput(string javapOutput, List<ExportedFunction> exports)
    {
        // javap output format:
        //   public static int add(int, int);
        //   public void doSomething();
        var methodRegex = new Regex(
            @"public\s+(?:static\s+)?(\w[\w\[\]<>,\s]*?)\s+(\w+)\(([^)]*)\)\s*;",
            RegexOptions.Compiled);

        foreach (var line in javapOutput.Split('\n', StringSplitOptions.RemoveEmptyEntries))
        {
            var match = methodRegex.Match(line.Trim());
            if (!match.Success) continue;

            var javaReturnType = match.Groups[1].Value.Trim();
            var methodName = match.Groups[2].Value;
            var paramStr = match.Groups[3].Value;

            // Skip constructors and standard Object methods
            if (IsStandardMethod(methodName)) continue;

            var parameters = ParseJavaParameters(paramStr);
            var returnType = MapJavaTypeToForge(javaReturnType);

            if (exports.Any(e => e.Name == methodName)) continue;

            exports.Add(new ExportedFunction
            {
                Name = methodName,
                ReturnType = returnType,
                Parameters = parameters
            });
        }
    }

    private static bool IsStandardMethod(string name)
    {
        var standard = new[] { "main", "toString", "hashCode", "equals", "getClass",
            "notify", "notifyAll", "wait", "clone", "finalize" };
        return standard.Contains(name);
    }

    private static FunctionParameter[] ParseJavaParameters(string paramStr)
    {
        if (string.IsNullOrWhiteSpace(paramStr)) return Array.Empty<FunctionParameter>();

        return paramStr.Split(',', StringSplitOptions.RemoveEmptyEntries)
            .Select((p, i) =>
            {
                var parts = p.Trim().Split(' ', StringSplitOptions.RemoveEmptyEntries);
                var type = parts.Length > 0 ? MapJavaTypeToForge(parts[0]) : "i32";
                var name = parts.Length > 1 ? parts[1] : $"arg{i}";
                return new FunctionParameter { Name = name, Type = type };
            })
            .ToArray();
    }

    private static string MapJavaTypeToForge(string javaType) => javaType switch
    {
        "void" => "void",
        "byte" => "i8",
        "short" => "i16",
        "int" => "i32",
        "long" => "i64",
        "float" => "f32",
        "double" => "f64",
        "boolean" => "u8",
        "char" => "u16",
        _ => "i32"
    };

    private static InterfaceDescription CreateEmptyInterface(string artifactPath) => new()
    {
        Version = "1.0",
        Module = new InterfaceModule
        {
            Name = Path.GetFileNameWithoutExtension(artifactPath),
            Version = "1.0.0"
        },
        Language = new InterfaceLanguage { Name = "java", Abi = "jni", Mode = "jvm" },
        Exports = Array.Empty<ExportedFunction>()
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Java Plugin");
        sb.AppendLine("// Java interop via JNI (Java Native Interface)");
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Java Plugin");
        sb.AppendLine("// Java interop via JNI extern declarations");
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
