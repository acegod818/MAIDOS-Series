// MAIDOS-Forge Assembly Language Plugin (Builtin)
// UEP v1.7C Compliant - Zero Technical Debt
// Code-QC v2.2B Compliant

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;

namespace Forge.Core.Plugin;

/// <summary>
/// Assembly 語言插件 - 支援 NASM/GAS/MASM/YASM
/// </summary>
/// <impl>
/// APPROACH: 調用彙編器編譯 .asm/.s 為目標檔
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: 無彙編器時返回錯誤，不同平台使用不同彙編器
/// </impl>
public sealed class AsmPlugin : ILanguagePlugin
{
    private string _assembler = "nasm";

    private enum AsmSyntax { Intel, Att }

    private static readonly string[] SourceExtensions = { ".asm", ".s", ".S", ".nasm" };

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "asm",
        SupportedExtensions = SourceExtensions,
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = true,
        SupportsInterfaceExtraction = true,
        SupportsGlueGeneration = true,
        SupportedTargets = new[]
        {
            "x86", "x86_64", "i386", "amd64",
            "arm", "aarch64", "arm64",
            "riscv32", "riscv64",
            "mips", "mips64"
        }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        // 優先檢查 NASM（跨平台，Intel 語法）
        if (await ProcessRunner.CommandExistsAsync("nasm"))
        {
            var version = await ProcessRunner.GetVersionAsync("nasm", "-v");
            _assembler = "nasm";
            return (true, $"NASM {version}");
        }

        // 檢查 YASM（NASM 相容）
        if (await ProcessRunner.CommandExistsAsync("yasm"))
        {
            var version = await ProcessRunner.GetVersionAsync("yasm", "--version");
            _assembler = "yasm";
            return (true, $"YASM {version}");
        }

        // 檢查 GAS（GNU Assembler，通常透過 as 或 gcc）
        if (await ProcessRunner.CommandExistsAsync("as"))
        {
            var version = await ProcessRunner.GetVersionAsync("as", "--version");
            _assembler = "as";
            return (true, $"GNU Assembler {version}");
        }

        // Windows: 檢查 MASM (ml64)
        if (OperatingSystem.IsWindows())
        {
            if (await ProcessRunner.CommandExistsAsync("ml64"))
            {
                _assembler = "ml64";
                return (true, "MASM (ml64.exe)");
            }
            if (await ProcessRunner.CommandExistsAsync("ml"))
            {
                _assembler = "ml";
                return (true, "MASM (ml.exe)");
            }
        }

        return (false, "No assembler found (tried nasm, yasm, as, ml64, ml)");
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

        logs.Add($"[ASM] Using: {toolchainMsg}");

        // 查找源碼
        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = module.ModulePath;
        }

        var sourceFiles = new List<string>();
        foreach (var ext in SourceExtensions)
        {
            sourceFiles.AddRange(Directory.GetFiles(srcDir, $"*{ext}", SearchOption.AllDirectories));
        }

        if (sourceFiles.Count == 0)
        {
            stopwatch.Stop();
            return CompileResult.Failure("No assembly source files found", logs, stopwatch.Elapsed);
        }

        logs.Add($"[ASM] Found {sourceFiles.Count} source file(s)");

        var outputDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outputDir);

        var asmConfig = module.Config.Asm ?? new AsmConfig();
        var objectFiles = new List<string>();

        // 決定目標格式
        var format = DetermineFormat(asmConfig, config);
        logs.Add($"[ASM] Target format: {format}");

        foreach (var sourceFile in sourceFiles)
        {
            var objFile = Path.Combine(outputDir,
                Path.GetFileNameWithoutExtension(sourceFile) + ".o");

            var args = BuildAssembleArgs(sourceFile, objFile, asmConfig, format, config);
            logs.Add($"$ {_assembler} {args}");

            var result = await ProcessRunner.RunAsync(
                _assembler, args,
                new ProcessConfig
                {
                    WorkingDirectory = module.ModulePath,
                    Timeout = TimeSpan.FromMinutes(5)
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
                return CompileResult.Failure(
                    $"Assembly failed for {Path.GetFileName(sourceFile)}: {result.Stderr}",
                    logs, stopwatch.Elapsed);
            }

            if (File.Exists(objFile))
            {
                objectFiles.Add(objFile);
            }
        }

        // 打包為靜態庫
        var libName = OperatingSystem.IsWindows() 
            ? $"{module.Config.Name}.lib" 
            : $"lib{module.Config.Name}.a";
        var libPath = Path.Combine(outputDir, libName);

        if (objectFiles.Count > 0)
        {
            if (OperatingSystem.IsWindows() && await ProcessRunner.CommandExistsAsync("lib"))
            {
                var libArgs = $"/OUT:\"{libPath}\" /NOLOGO {string.Join(" ", objectFiles.Select(f => $"\"{f}\""))}";
                logs.Add($"$ lib {libArgs}");

                var libResult = await ProcessRunner.RunAsync("lib", libArgs,
                    new ProcessConfig { Timeout = TimeSpan.FromMinutes(2) }, ct);

                if (!libResult.IsSuccess)
                {
                    logs.Add($"lib.exe failed: {libResult.Stderr}");
                }
            }
            else if (await ProcessRunner.CommandExistsAsync("ar"))
            {
                var arArgs = $"rcs \"{libPath}\" {string.Join(" ", objectFiles.Select(f => $"\"{f}\""))}";
                logs.Add($"$ ar {arArgs}");

                var arResult = await ProcessRunner.RunAsync("ar", arArgs,
                    new ProcessConfig { Timeout = TimeSpan.FromMinutes(2) }, ct);

                if (!arResult.IsSuccess)
                {
                    logs.Add($"ar failed: {arResult.Stderr}");
                }
            }
        }

        stopwatch.Stop();
        var artifacts = File.Exists(libPath)
            ? new[] { libPath }
            : objectFiles.ToArray();

        return CompileResult.Success(artifacts, logs, stopwatch.Elapsed);
    }

    private string DetermineFormat(AsmConfig asmConfig, CompileConfig config)
    {
        // 如果有明確指定格式
        if (!string.IsNullOrEmpty(asmConfig.Format))
        {
            return asmConfig.Format;
        }

        // 根據平台自動決定
        var arch = asmConfig.Arch ?? "x86_64";
        var is64Bit = arch.Contains("64") || arch == "amd64" || arch == "aarch64";

        if (OperatingSystem.IsWindows())
        {
            return is64Bit ? "win64" : "win32";
        }
        else if (OperatingSystem.IsMacOS())
        {
            return "macho64";
        }
        else
        {
            return is64Bit ? "elf64" : "elf32";
        }
    }

    private string BuildAssembleArgs(
        string sourceFile,
        string outputFile,
        AsmConfig asmConfig,
        string format,
        CompileConfig config)
    {
        var args = new List<string>();

        switch (_assembler)
        {
            case "nasm":
            case "yasm":
                args.Add($"-f {format}");
                args.Add($"-o \"{outputFile}\"");
                
                if (config.Profile == "debug")
                {
                    args.Add("-g");  // Debug 符號
                    args.Add("-F dwarf");  // DWARF 格式
                }

                // Include 目錄
                foreach (var inc in asmConfig.Includes)
                {
                    args.Add($"-I\"{inc}\"");
                }

                // 預處理宏
                foreach (var define in asmConfig.Defines)
                {
                    args.Add($"-D{define}");
                }

                args.Add($"\"{sourceFile}\"");
                break;

            case "as":
                // GNU Assembler
                if (config.Profile == "debug")
                {
                    args.Add("-g");
                }

                // Include 目錄
                foreach (var inc in asmConfig.Includes)
                {
                    args.Add($"-I\"{inc}\"");
                }

                // 預處理宏
                foreach (var define in asmConfig.Defines)
                {
                    args.Add($"--defsym {define}=1");
                }

                args.Add("-o");
                args.Add($"\"{outputFile}\"");
                args.Add($"\"{sourceFile}\"");
                break;

            case "ml64":
            case "ml":
                // MASM
                args.Add("/c");  // 只編譯不鏈接
                args.Add($"/Fo\"{outputFile}\"");

                if (config.Profile == "debug")
                {
                    args.Add("/Zi");  // Debug 符號
                }

                // Include 目錄
                foreach (var inc in asmConfig.Includes)
                {
                    args.Add($"/I\"{inc}\"");
                }

                // 預處理宏
                foreach (var define in asmConfig.Defines)
                {
                    args.Add($"/D{define}");
                }

                args.Add($"\"{sourceFile}\"");
                break;
        }

        return string.Join(" ", args);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(
        string artifactPath, CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        // 使用 nm 提取全域符號
        string nmCmd = "nm";
        string nmArgs = OperatingSystem.IsMacOS()
            ? $"-g -U \"{artifactPath}\""  // macOS: -U 只顯示已定義
            : $"-g --defined-only \"{artifactPath}\"";

        var nmResult = await ProcessRunner.RunAsync(
            nmCmd, nmArgs,
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);

        if (nmResult.IsSuccess && !string.IsNullOrEmpty(nmResult.Stdout))
        {
            foreach (var line in nmResult.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries))
            {
                var parts = line.Split(' ', StringSplitOptions.RemoveEmptyEntries);
                if (parts.Length < 3) continue;

                var symbolType = parts[1];
                var symbolName = parts[2];

                // T = text (code), D = data, B = bss
                if (symbolType != "T" && symbolType != "t") continue;

                // 過濾內部符號
                if (IsInternalSymbol(symbolName)) continue;

                // 移除前導下劃線（macOS/Windows convention）
                if (symbolName.StartsWith("_") && !symbolName.StartsWith("__"))
                {
                    symbolName = symbolName.TrimStart('_');
                }

                exports.Add(new ExportedFunction
                {
                    Name = symbolName,
                    ReturnType = "void",  // ASM 無法推斷類型
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
            Language = new InterfaceLanguage { Name = "asm", Abi = "c" },
            Exports = exports.ToArray()
        };
    }

    private static bool IsInternalSymbol(string name)
    {
        var prefixes = new[]
        {
            "__", ".L", ".l", "L_", "l_",
            "_start", "_init", "_fini",
            ".text", ".data", ".bss", ".rodata"
        };
        return prefixes.Any(p => name.StartsWith(p, StringComparison.Ordinal));
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

        sb.AppendLine("// ═══════════════════════════════════════════════════════════════");
        sb.AppendLine("// Auto-generated by MAIDOS-Forge ASM Plugin");
        sb.AppendLine($"// Source: {moduleName} (Assembly)");
        sb.AppendLine($"// Generated: {DateTime.UtcNow:yyyy-MM-ddTHH:mm:ssZ}");
        sb.AppendLine("// DO NOT EDIT - Changes will be overwritten");
        sb.AppendLine("//");
        sb.AppendLine("// NOTE: Assembly functions have unknown signatures.");
        sb.AppendLine("//       Please verify parameters and return types manually.");
        sb.AppendLine("// ═══════════════════════════════════════════════════════════════");
        sb.AppendLine();
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
            sb.AppendLine($"    /// <summary>");
            sb.AppendLine($"    /// Assembly function: {export.Name}");
            sb.AppendLine($"    /// WARNING: Signature unknown - verify manually");
            sb.AppendLine($"    /// </summary>");
            sb.AppendLine($"    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]");
            sb.AppendLine($"    public static extern void {export.Name}();");
            sb.AppendLine();
        }

        sb.AppendLine("}");
        return GlueCodeResult.Success(sb.ToString(), $"{pascalName}.Interop.cs", "csharp");
    }

    private static GlueCodeResult GenerateRustGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;

        sb.AppendLine("// ═══════════════════════════════════════════════════════════════");
        sb.AppendLine("// Auto-generated by MAIDOS-Forge ASM Plugin");
        sb.AppendLine($"// Source: {moduleName} (Assembly)");
        sb.AppendLine($"// Generated: {DateTime.UtcNow:yyyy-MM-ddTHH:mm:ssZ}");
        sb.AppendLine("// DO NOT EDIT - Changes will be overwritten");
        sb.AppendLine("//");
        sb.AppendLine("// NOTE: Assembly functions have unknown signatures.");
        sb.AppendLine("//       Please verify parameters and return types manually.");
        sb.AppendLine("// ═══════════════════════════════════════════════════════════════");
        sb.AppendLine();
        sb.AppendLine("#![allow(non_snake_case)]");
        sb.AppendLine("#![allow(dead_code)]");
        sb.AppendLine();
        sb.AppendLine($"#[link(name = \"{moduleName}\")]");
        sb.AppendLine("extern \"C\" {");

        foreach (var export in source.Exports)
        {
            sb.AppendLine($"    /// Assembly function: {export.Name}");
            sb.AppendLine($"    /// WARNING: Signature unknown - verify manually");
            sb.AppendLine($"    pub fn {export.Name}();");
        }

        sb.AppendLine("}");
        return GlueCodeResult.Success(sb.ToString(), $"{moduleName}_ffi.rs", "rust");
    }

    private static GlueCodeResult GenerateCHeaderGlue(InterfaceDescription source)
    {
        var sb = new StringBuilder();
        var moduleName = source.Module.Name;
        var guardName = $"{moduleName.ToUpperInvariant().Replace("-", "_")}_ASM_H";

        sb.AppendLine("/* ═══════════════════════════════════════════════════════════════ */");
        sb.AppendLine("/* Auto-generated by MAIDOS-Forge ASM Plugin                       */");
        sb.AppendLine($"/* Source: {moduleName} (Assembly)                                 */");
        sb.AppendLine($"/* Generated: {DateTime.UtcNow:yyyy-MM-ddTHH:mm:ssZ}               */");
        sb.AppendLine("/* DO NOT EDIT - Changes will be overwritten                       */");
        sb.AppendLine("/*                                                                  */");
        sb.AppendLine("/* NOTE: Assembly functions have unknown signatures.               */");
        sb.AppendLine("/*       Please verify parameters and return types manually.       */");
        sb.AppendLine("/* ═══════════════════════════════════════════════════════════════ */");
        sb.AppendLine();
        sb.AppendLine($"#ifndef {guardName}");
        sb.AppendLine($"#define {guardName}");
        sb.AppendLine();
        sb.AppendLine("#include <stdint.h>");
        sb.AppendLine();
        sb.AppendLine("#ifdef __cplusplus");
        sb.AppendLine("extern \"C\" {");
        sb.AppendLine("#endif");
        sb.AppendLine();

        foreach (var export in source.Exports)
        {
            sb.AppendLine($"/* Assembly function: {export.Name} */");
            sb.AppendLine($"/* WARNING: Signature unknown - verify manually */");
            sb.AppendLine($"void {export.Name}(void);");
            sb.AppendLine();
        }

        sb.AppendLine("#ifdef __cplusplus");
        sb.AppendLine("}");
        sb.AppendLine("#endif");
        sb.AppendLine();
        sb.AppendLine($"#endif /* {guardName} */");

        return GlueCodeResult.Success(sb.ToString(), $"{moduleName}.h", "c");
    }

    private static string ToPascalCase(string s)
    {
        if (string.IsNullOrEmpty(s)) return s;
        var parts = s.Split('-', '_', '.');
        return string.Concat(parts.Select(p =>
            p.Length > 0 ? char.ToUpperInvariant(p[0]) + p.Substring(1).ToLowerInvariant() : ""));
    }
}