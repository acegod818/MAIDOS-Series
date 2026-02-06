// MAIDOS-Forge Rust Language Plugin
// UEP v1.7C Compliant - Zero Technical Debt
// Standalone Plugin Module

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.Rust;

/// <summary>
/// Rust 語言插件 - 封裝 cargo build 和 rustc
/// </summary>
/// <impl>
/// APPROACH: 封裝 cargo 命令，支援原生編譯和交叉編譯
/// CALLS: ProcessRunner.RunAsync(), cargo/rustc CLI
/// EDGES: 無 cargo/rustc 時 ValidateToolchain 返回失敗
/// </impl>
public sealed class RustLanguagePlugin : ILanguagePlugin
{
    private const string CargoCommand = "cargo";
    private const string RustcCommand = "rustc";

    public PluginCapabilities GetCapabilities()
    {
        return new PluginCapabilities
        {
            LanguageName = "rust",
            SupportedExtensions = new[] { ".rs", "Cargo.toml" },
            SupportsNativeCompilation = true,
            SupportsCrossCompilation = true,
            SupportsInterfaceExtraction = true,
            SupportsGlueGeneration = true,
            SupportedTargets = new[]
            {
                "x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu",
                "i686-pc-windows-msvc", "i686-pc-windows-gnu",
                "aarch64-pc-windows-msvc",
                "x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl",
                "aarch64-unknown-linux-gnu", "aarch64-unknown-linux-musl",
                "x86_64-apple-darwin", "aarch64-apple-darwin"
            }
        };
    }

    public async Task<CompileResult> CompileAsync(
        ValidatedModuleConfig module,
        CompileConfig config,
        CancellationToken ct = default)
    {
        var logs = new List<string>();
        var stopwatch = System.Diagnostics.Stopwatch.StartNew();

        logs.Add($"[Rust] Compiling module '{module.Config.Name}'");

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var cargoToml = Path.Combine(module.ModulePath, "Cargo.toml");
        if (!File.Exists(cargoToml))
        {
            cargoToml = Path.Combine(srcDir, "Cargo.toml");
        }

        CompileResult result;
        if (File.Exists(cargoToml))
        {
            logs.Add($"[Rust] Found Cargo.toml: {cargoToml}");
            result = await CompileWithCargoAsync(module, cargoToml, config, logs, ct);
        }
        else
        {
            var rsFiles = Directory.GetFiles(srcDir, "*.rs", SearchOption.AllDirectories);
            if (rsFiles.Length == 0)
            {
                stopwatch.Stop();
                return CompileResult.Failure($"No Cargo.toml or .rs files found in {module.ModulePath}", 
                    logs, stopwatch.Elapsed);
            }

            logs.Add($"[Rust] No Cargo.toml, using rustc directly");
            result = await CompileWithRustcAsync(module, rsFiles, config, logs, ct);
        }

        stopwatch.Stop();
        return result;
    }

    private async Task<CompileResult> CompileWithCargoAsync(
        ValidatedModuleConfig module,
        string cargoToml,
        CompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var stopwatch = System.Diagnostics.Stopwatch.StartNew();
        var cargoDir = Path.GetDirectoryName(cargoToml) ?? string.Empty;
        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var releaseFlag = config.Profile == "debug" ? "" : "--release";
        var targetDir = Path.Combine(cargoDir, "target");

        var args = $"build {releaseFlag}";

        if (config.TargetPlatform != "native")
        {
            var rustTarget = MapToRustTarget(config.TargetPlatform);
            if (!string.IsNullOrEmpty(rustTarget))
            {
                args += $" --target {rustTarget}";
            }
        }

        if (config.Verbose)
        {
            args += " -v";
        }

        logs.Add($"[Rust] Running: cargo {args}");

        var result = await ProcessRunner.RunAsync(
            CargoCommand, args,
            new ProcessConfig
            {
                WorkingDirectory = cargoDir,
                Environment = config.Environment,
                Timeout = TimeSpan.FromMinutes(10)
            }, ct);

        if (!string.IsNullOrEmpty(result.Stdout))
        {
            logs.AddRange(result.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries)
                .Select(l => $"[cargo] {l}"));
        }

        if (!result.IsSuccess)
        {
            logs.Add($"[Rust] Cargo build failed with exit code {result.ExitCode}");
            if (!string.IsNullOrEmpty(result.Stderr))
            {
                logs.AddRange(result.Stderr.Split('\n', StringSplitOptions.RemoveEmptyEntries)
                    .Select(l => $"[error] {l}"));
            }
            stopwatch.Stop();
            return CompileResult.Failure($"cargo build failed: {result.Stderr}", logs, stopwatch.Elapsed);
        }

        var artifacts = new List<string>();
        var profileDir = config.Profile == "debug" ? "debug" : "release";
        var searchDir = Path.Combine(targetDir, profileDir);

        if (config.TargetPlatform != "native")
        {
            var rustTarget = MapToRustTarget(config.TargetPlatform);
            if (!string.IsNullOrEmpty(rustTarget))
            {
                searchDir = Path.Combine(targetDir, rustTarget, profileDir);
            }
        }

        if (Directory.Exists(searchDir))
        {
            var patterns = new[] { "*.rlib", "*.a", "*.so", "*.dylib", "*.dll" };
            foreach (var pattern in patterns)
            {
                artifacts.AddRange(Directory.GetFiles(searchDir, pattern));
            }

            foreach (var artifact in artifacts.ToList())
            {
                var destPath = Path.Combine(outputDir, Path.GetFileName(artifact));
                try
                {
                    File.Copy(artifact, destPath, overwrite: true);
                    artifacts.Remove(artifact);
                    artifacts.Add(destPath);
                }
                catch (Exception ex)
                {
                    logs.Add($"[Rust] Warning: Failed to copy {artifact}: {ex.Message}");
                }
            }
        }

        logs.Add($"[Rust] Build succeeded, {artifacts.Count} artifact(s)");
        stopwatch.Stop();
        return CompileResult.Success(artifacts, logs, stopwatch.Elapsed);
    }

    private async Task<CompileResult> CompileWithRustcAsync(
        ValidatedModuleConfig module,
        string[] rsFiles,
        CompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var stopwatch = System.Diagnostics.Stopwatch.StartNew();
        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var artifacts = new List<string>();
        var mainFile = rsFiles.FirstOrDefault(f => 
            Path.GetFileName(f).Equals("lib.rs", StringComparison.OrdinalIgnoreCase)) 
            ?? rsFiles[0];

        var outputFile = Path.Combine(outputDir, $"lib{module.Config.Name}.rlib");
        var crateType = module.Config.Type?.ToLowerInvariant() == "executable" ? "bin" : "rlib";
        
        if (crateType == "bin")
        {
            outputFile = Path.Combine(outputDir, 
                OperatingSystem.IsWindows() ? $"{module.Config.Name}.exe" : module.Config.Name);
        }

        var releaseFlag = config.Profile == "debug" ? "" : "-O";
        var args = $"{mainFile} --crate-type {crateType} -o \"{outputFile}\" {releaseFlag}";

        if (config.TargetPlatform != "native")
        {
            var rustTarget = MapToRustTarget(config.TargetPlatform);
            if (!string.IsNullOrEmpty(rustTarget))
            {
                args += $" --target {rustTarget}";
            }
        }

        logs.Add($"[Rust] Running: rustc {args}");

        var result = await ProcessRunner.RunAsync(
            RustcCommand, args,
            new ProcessConfig
            {
                WorkingDirectory = Path.GetDirectoryName(mainFile) ?? string.Empty,
                Environment = config.Environment,
                Timeout = TimeSpan.FromMinutes(5)
            }, ct);

        if (!string.IsNullOrEmpty(result.Stdout))
        {
            logs.AddRange(result.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries)
                .Select(l => $"[rustc] {l}"));
        }

        if (!result.IsSuccess)
        {
            logs.Add($"[Rust] rustc failed with exit code {result.ExitCode}");
            if (!string.IsNullOrEmpty(result.Stderr))
            {
                logs.AddRange(result.Stderr.Split('\n', StringSplitOptions.RemoveEmptyEntries)
                    .Select(l => $"[error] {l}"));
            }
            stopwatch.Stop();
            return CompileResult.Failure($"rustc failed: {result.Stderr}", logs, stopwatch.Elapsed);
        }

        if (File.Exists(outputFile))
        {
            artifacts.Add(outputFile);
        }

        logs.Add($"[Rust] Build succeeded, {artifacts.Count} artifact(s)");
        stopwatch.Stop();
        return CompileResult.Success(artifacts, logs, stopwatch.Elapsed);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath,
        CancellationToken ct = default)
    {
        var ext = Path.GetExtension(artifactPath).ToLowerInvariant();
        if (ext != ".rlib" && ext != ".a" && ext != ".so" && ext != ".dylib")
        {
            return null;
        }

        var result = await ProcessRunner.RunAsync(
            "nm", $"-g \"{artifactPath}\"",
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);

        var exports = new List<ExportedFunction>();

        if (result.IsSuccess && !string.IsNullOrEmpty(result.Stdout))
        {
            foreach (var line in result.Stdout.Split('\n'))
            {
                var parts = line.Split(' ', StringSplitOptions.RemoveEmptyEntries);
                if (parts.Length >= 3 && parts[1] == "T")
                {
                    var symbol = parts[2];
                    if (!symbol.StartsWith("_ZN") && !symbol.Contains("$"))
                    {
                        exports.Add(new ExportedFunction
                        {
                            Name = symbol.TrimStart('_'),
                            CallingConvention = "cdecl",
                            ReturnType = "void",
                            Parameters = Array.Empty<FunctionParameter>()
                        });
                    }
                }
            }
        }

        return new InterfaceDescription
        {
            Version = "1.0",
            Module = new InterfaceModule
            {
                Name = Path.GetFileNameWithoutExtension(artifactPath).TrimStart("lib".ToCharArray()),
                Version = "1.0.0"
            },
            Language = new InterfaceLanguage
            {
                Name = "rust",
                Abi = "c",
                Mode = "native"
            },
            Exports = exports
        };
    }

    public GlueCodeResult GenerateGlue(InterfaceDescription sourceInterface, string targetLanguage)
    {
        return targetLanguage.ToLowerInvariant() switch
        {
            "csharp" => GenerateCSharpGlue(sourceInterface),
            "c" => GenerateCGlue(sourceInterface),
            _ => GlueCodeResult.Failure($"Unsupported target language: {targetLanguage}")
        };
    }

    private GlueCodeResult GenerateCSharpGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        sb.AppendLine("// Auto-generated by MAIDOS-Forge Rust Plugin");
        sb.AppendLine($"// Source: {source.Module.Name}");
        sb.AppendLine();
        sb.AppendLine("using System.Runtime.InteropServices;");
        sb.AppendLine();
        sb.AppendLine($"namespace {source.Module.Name}.Interop;");
        sb.AppendLine();
        sb.AppendLine($"public static partial class {ToPascalCase(source.Module.Name)}Native");
        sb.AppendLine("{");
        sb.AppendLine($"    private const string LibraryName = \"{source.Module.Name}\";");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var csharpReturn = MapTypeToCSharp(export.ReturnType);
            var csharpParams = string.Join(", ",
                export.Parameters.Select(p => $"{MapTypeToCSharp(p.Type)} {p.Name}"));

            sb.AppendLine($"    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]");
            sb.AppendLine($"    public static extern {csharpReturn} {export.Name}({csharpParams});");
            sb.AppendLine();
        }

        sb.AppendLine("}");

        var fileName = $"{source.Module.Name}.Interop.cs";
        return GlueCodeResult.Success(sb.ToString(), fileName, "csharp");
    }

    private GlueCodeResult GenerateCGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var guard = $"{source.Module.Name.ToUpperInvariant()}_FFI_H";

        sb.AppendLine("// Auto-generated by MAIDOS-Forge Rust Plugin");
        sb.AppendLine($"// Source: {source.Module.Name}");
        sb.AppendLine();
        sb.AppendLine($"#ifndef {guard}");
        sb.AppendLine($"#define {guard}");
        sb.AppendLine();
        sb.AppendLine("#include <stdint.h>");
        sb.AppendLine();
        sb.AppendLine("#ifdef __cplusplus");
        sb.AppendLine("extern \"C\" {");
        sb.AppendLine("#endif");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            var cReturn = MapTypeToC(export.ReturnType);
            var cParams = string.Join(", ",
                export.Parameters.Select(p => $"{MapTypeToC(p.Type)} {p.Name}"));

            if (string.IsNullOrEmpty(cParams)) cParams = "void";
            sb.AppendLine($"{cReturn} {export.Name}({cParams});");
        }

        sb.AppendLine();
        sb.AppendLine("#ifdef __cplusplus");
        sb.AppendLine("}");
        sb.AppendLine("#endif");
        sb.AppendLine();
        sb.AppendLine($"#endif // {guard}");

        var fileName = $"{source.Module.Name}_ffi.h";
        return GlueCodeResult.Success(sb.ToString(), fileName, "c");
    }

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        var rustcVersion = await ProcessRunner.GetVersionAsync(RustcCommand, "--version", ct);
        if (rustcVersion is null)
        {
            return (false, "rustc not found. Please install Rust from https://rustup.rs");
        }

        var cargoVersion = await ProcessRunner.GetVersionAsync(CargoCommand, "--version", ct);
        if (cargoVersion is null)
        {
            return (false, "cargo not found. Please install Rust from https://rustup.rs");
        }

        return (true, $"{rustcVersion}, {cargoVersion}");
    }

    private static string? MapToRustTarget(string forgeTarget) => forgeTarget.ToLowerInvariant() switch
    {
        "win-x64" or "windows-x86_64" => "x86_64-pc-windows-msvc",
        "win-x86" or "windows-x86" => "i686-pc-windows-msvc",
        "win-arm64" or "windows-arm64" => "aarch64-pc-windows-msvc",
        "linux-x64" or "linux-x86_64" => "x86_64-unknown-linux-gnu",
        "linux-arm64" or "linux-aarch64" => "aarch64-unknown-linux-gnu",
        "osx-x64" or "macos-x86_64" => "x86_64-apple-darwin",
        "osx-arm64" or "macos-arm64" => "aarch64-apple-darwin",
        _ => null
    };

    private static string ToPascalCase(string s)
    {
        if (string.IsNullOrEmpty(s)) return s;
        var parts = s.Split('-', '_');
        return string.Concat(parts.Select(p => 
            char.ToUpperInvariant(p[0]) + p.Substring(1).ToLowerInvariant()));
    }

    private static string MapTypeToCSharp(string type) => type.ToLowerInvariant() switch
    {
        "void" or "()" => "void",
        "bool" => "bool",
        "i8" => "sbyte",
        "i16" => "short",
        "i32" => "int",
        "i64" => "long",
        "u8" => "byte",
        "u16" => "ushort",
        "u32" => "uint",
        "u64" => "ulong",
        "f32" => "float",
        "f64" => "double",
        "isize" => "nint",
        "usize" => "nuint",
        _ => "IntPtr"
    };

    private static string MapTypeToC(string type) => type.ToLowerInvariant() switch
    {
        "void" or "()" => "void",
        "bool" => "_Bool",
        "i8" => "int8_t",
        "i16" => "int16_t",
        "i32" => "int32_t",
        "i64" => "int64_t",
        "u8" => "uint8_t",
        "u16" => "uint16_t",
        "u32" => "uint32_t",
        "u64" => "uint64_t",
        "f32" => "float",
        "f64" => "double",
        "isize" => "intptr_t",
        "usize" => "size_t",
        _ => "void*"
    };
}
