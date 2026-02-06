// MAIDOS-Forge Generic Language Plugin Template
// UEP v1.7C Compliant - Zero Technical Debt
// Template for implementing new language plugins

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.{{LanguageName}};

/// <summary>
/// {{LanguageDisplayName}} Language Plugin - Generic Template
/// </summary>
/// <impl>
/// APPROACH: Template for implementing {{LanguageName}} language support
/// CALLS: ProcessRunner.RunAsync(), Directory operations
/// EDGES: Returns failure if compiler not available
/// </impl>
public sealed class {{LanguageName}}Plugin : ILanguagePlugin
{
    private string _compiler = "{{DefaultCompiler}}";

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "{{LanguageName}}",
        SupportedExtensions = new[] { {{SupportedExtensions}} },
        SupportsNativeCompilation = {{SupportsNativeCompilation}},
        SupportsCrossCompilation = {{SupportsCrossCompilation}},
        SupportsInterfaceExtraction = {{SupportsInterfaceExtraction}},
        SupportsGlueGeneration = {{SupportsGlueGeneration}},
        SupportedTargets = new[] { "linux", "windows", "macos" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        {{#each CompilerCommands}}
        if (await ProcessRunner.CommandExistsAsync("{{this}}"))
        {
            var version = await ProcessRunner.GetVersionAsync("{{this}}", "{{VersionCommand}}");
            _compiler = "{{this}}";
            return (true, $"{{this}} {version}");
        }
        {{/each}}

        return (false, $"No suitable compiler found for {{LanguageName}}. Checked: {{CompilerCommands}}");
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

        logs.Add($"[{{LanguageDisplayName}}] Using: {toolchainMsg}");

        // Find source files
        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var sourceFiles = Directory.GetFiles(srcDir, "*{{PrimaryExtension}}", SearchOption.AllDirectories);
        if (sourceFiles.Length == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure($"No {{PrimaryExtension}} source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[{{LanguageDisplayName}}] Found {sourceFiles.Length} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        // Language-specific configuration
        var langConfig = module.Config.{{LanguageName}} ?? new {{LanguageName}}Config();
        var objectFiles = new List<string>();

        foreach (var sourceFile in sourceFiles)
        {
            var objFile = Path.Combine(outputDir,
                Path.GetFileNameWithoutExtension(sourceFile) + "{{ObjectExtension}}");

            var args = BuildCompileArgs(sourceFile, objFile, langConfig, config);
            logs.Add($"$ {_compiler} {args}");

            var result = await ProcessRunner.RunAsync(
                _compiler, args,
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
                    $"Compilation failed for {Path.GetFileName(sourceFile)}: {result.Stderr}",
                    logs, stopwatch.Elapsed);
            }

            objectFiles.Add(objFile);
        }

        // Link or create library based on output type
        var artifacts = new List<string>();
        
        {{#if CreatesLibrary}}
        // Create library/archive
        var libName = $"lib{module.Config.Name}.{{{LibraryExtension}}}";
        var libPath = Path.Combine(outputDir, libName);

        var buildArgs = BuildLibraryArgs(objectFiles, libPath);
        logs.Add($"$ {_compiler} {buildArgs}");

        var buildResult = await ProcessRunner.RunAsync(_compiler, buildArgs,
            new ProcessConfig { Timeout = TimeSpan.FromMinutes(2) }, ct);

        if (buildResult.IsSuccess)
        {
            artifacts.Add(libPath);
        }
        else
        {
            // Fallback to object files
            artifacts.AddRange(objectFiles);
            logs.Add($"Library creation failed, using object files");
        }
        {{else}}
        // Use object files directly
        artifacts.AddRange(objectFiles);
        {{/if}}

        stopwatch.Stop();
        return CompileResult.Success(artifacts, logs, stopwatch.Elapsed);
    }

    private string BuildCompileArgs(
        string sourceFile,
        string outputFile,
        {{LanguageName}}Config langConfig,
        CompileConfig config)
    {
        var args = new List<string>
        {
            {{#each CompileBaseArgs}}
            "{{this}}"{{#unless @last}},{{/unless}}
            {{/each}}
            $"\"{sourceFile}\"",
            {{#if HasOutputOption}}
            {{OutputOption}},
            {{/if}}
            $"\"{outputFile}\""
        };

        var optLevel = config.Profile == "debug" ? "{{DebugOptimization}}" : "{{ReleaseOptimization}}";
        args.Add(optLevel);

        if (config.Profile == "debug")
        {
            args.AddRange(new[] { {{#each DebugFlags}}"{{this}}"{{#unless @last}},{{/unless}}{{/each}} });
        }

        // Language-specific flags
        {{#if HasStandardFlag}}
        args.Add($"{{StandardFlag}}={langConfig.Standard}");
        {{/if}}

        {{#if HasWarningFlags}}
        args.AddRange(new[] { {{#each WarningFlags}}"{{this}}"{{#unless @last}},{{/unless}}{{/each}} });
        {{/if}}

        {{#if SupportsPIC}}
        args.Add("-fPIC");
        {{/if}}

        // Defines
        foreach (var define in langConfig.Defines)
        {
            args.Add($"-D{define}");
        }

        // Include directories
        foreach (var inc in langConfig.IncludeDirs)
        {
            args.Add($"-I\"{inc}\"");
        }

        return string.Join(" ", args);
    }

    {{#if CreatesLibrary}}
    private string BuildLibraryArgs(List<string> objectFiles, string outputPath)
    {
        var args = new List<string>
        {
            {{#each LibraryBaseArgs}}
            "{{this}}"{{#unless @last}},{{/unless}}
            {{/each}}
            $"\"{outputPath}\"",
            string.Join(" ", objectFiles.Select(f => $"\"{f}\""))
        };

        return string.Join(" ", args);
    }
    {{/if}}

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        // [TEMPLATE] Language-specific interface extraction should be implemented here.
        // Returning basic interface description as a base.
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
                Name = "{{LanguageName}}",
                Abi = "c"
            },
            Exports = exports.ToArray()
        };
    }

    public GlueCodeResult GenerateGlue(InterfaceDescription sourceInterface, string targetLanguage)
    {
        return targetLanguage.ToLowerInvariant() switch
        {
            "csharp" or "c#" => GenerateCSharpGlue(sourceInterface),
            "rust" => GenerateRustGlue(sourceInterface),
            "c" => GenerateCGlue(sourceInterface),
            _ => GlueCodeResult.Failure($"Unsupported target language: {targetLanguage}")
        };
    }

    private static GlueCodeResult GenerateCSharpGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;
        var pascalName = ToPascalCase(moduleName);

        sb.AppendLine("// Auto-generated by MAIDOS-Forge {{LanguageDisplayName}} Plugin");
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

        sb.AppendLine("// Auto-generated by MAIDOS-Forge {{LanguageDisplayName}} Plugin");
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

    private static GlueCodeResult GenerateCGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name.ToUpperInvariant();

        sb.AppendLine("// Auto-generated by MAIDOS-Forge {{LanguageDisplayName}} Plugin");
        sb.AppendLine($"#ifndef {moduleName}_H");
        sb.AppendLine($"#define {moduleName}_H");
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

            sb.AppendLine($"    {returnType} {export.Name}({parms});");
        }

        sb.AppendLine();
        sb.AppendLine("#ifdef __cplusplus");
        sb.AppendLine("}");
        sb.AppendLine("#endif");
        sb.AppendLine();
        sb.AppendLine($"#endif // {moduleName}_H");

        return GlueCodeResult.Success(sb.ToString(), $"{moduleName.ToLowerInvariant()}.h", "c");
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
        _ => "int"
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

/// <summary>
/// {{LanguageDisplayName}} Configuration
/// </summary>
public class {{LanguageName}}Config
{
    public string Standard { get; set; } = "{{DefaultStandard}}";
    public string[] Defines { get; set; } = Array.Empty<string>();
    public string[] IncludeDirs { get; set; } = Array.Empty<string>();
}