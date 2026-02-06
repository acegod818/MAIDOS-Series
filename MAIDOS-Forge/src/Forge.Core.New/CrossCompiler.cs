// MAIDOS-Forge Cross-Compiler Core
// Code-QC v2.2B Compliant | M14 Cross-Compilation Module

using System.Diagnostics;
using Forge.Core.Config;
using Forge.Core.Platform;

namespace Forge.Core.CrossCompile;

/// <summary>
/// 交叉編譯結果
/// </summary>
public sealed class CrossCompileResult
{
    public bool Success { get; init; }
    public string? OutputPath { get; init; }
    public CrossTarget Target { get; init; } = CrossTarget.Native;
    public TimeSpan Duration { get; init; }
    public IReadOnlyList<string> Logs { get; init; } = Array.Empty<string>();
    public string? ErrorMessage { get; init; }

    public static CrossCompileResult Ok(string outputPath, CrossTarget target, TimeSpan duration, IReadOnlyList<string> logs)
        => new() { Success = true, OutputPath = outputPath, Target = target, Duration = duration, Logs = logs };

    public static CrossCompileResult Fail(string error, CrossTarget target, TimeSpan duration, IReadOnlyList<string> logs)
        => new() { Success = false, ErrorMessage = error, Target = target, Duration = duration, Logs = logs };
}

/// <summary>
/// 交叉編譯配置
/// </summary>
public sealed class CrossCompileConfig
{
    /// <summary>目標平台</summary>
    public CrossTarget Target { get; init; } = CrossTarget.Native;
    
    /// <summary>輸出類型</summary>
    public CrossOutputType OutputType { get; init; } = CrossOutputType.Executable;
    
    /// <summary>輸出目錄</summary>
    public string OutputDir { get; init; } = "build";
    
    /// <summary>輸出名稱（不含副檔名）</summary>
    public string OutputName { get; init; } = "output";
    
    /// <summary>優化等級 (0-3, s, z)</summary>
    public string OptLevel { get; init; } = "2";
    
    /// <summary>是否為 Debug 模式</summary>
    public bool Debug { get; init; }
    
    /// <summary>是否剝離符號</summary>
    public bool Strip { get; init; }
    
    /// <summary>額外的編譯標誌</summary>
    public IReadOnlyList<string> ExtraCFlags { get; init; } = Array.Empty<string>();
    
    /// <summary>額外的連結標誌</summary>
    public IReadOnlyList<string> ExtraLdFlags { get; init; } = Array.Empty<string>();
    
    /// <summary>包含目錄</summary>
    public IReadOnlyList<string> IncludeDirs { get; init; } = Array.Empty<string>();
    
    /// <summary>庫目錄</summary>
    public IReadOnlyList<string> LibDirs { get; init; } = Array.Empty<string>();
    
    /// <summary>連結的庫</summary>
    public IReadOnlyList<string> Libraries { get; init; } = Array.Empty<string>();
    
    /// <summary>定義的巨集</summary>
    public IReadOnlyDictionary<string, string?> Defines { get; init; } = new Dictionary<string, string?>();
    
    /// <summary>是否顯示詳細輸出</summary>
    public bool Verbose { get; init; }
}

/// <summary>輸出類型</summary>
public enum CrossOutputType
{
    Executable,
    SharedLibrary,
    StaticLibrary,
    Object
}

/// <summary>
/// 交叉編譯器
/// </summary>
public sealed class CrossCompiler
{
    private readonly CrossToolchainManager _toolchainManager;

    public CrossCompiler(CrossToolchainManager? toolchainManager = null)
    {
        _toolchainManager = toolchainManager ?? new CrossToolchainManager();
    }

    /// <summary>編譯 C/C++ 源碼到目標平台</summary>
    public async Task<CrossCompileResult> CompileAsync(
        IReadOnlyList<string> sourceFiles,
        CrossCompileConfig config,
        CancellationToken ct = default)
    {
        var sw = Stopwatch.StartNew();
        var logs = new List<string>();

        // 取得工具鏈
        var toolchain = await _toolchainManager.GetToolchainAsync(config.Target, ct);
        
        if (!toolchain.IsAvailable)
        {
            sw.Stop();
            return CrossCompileResult.Fail(
                $"Toolchain not available for {config.Target}: {toolchain.ValidationMessage}",
                config.Target, sw.Elapsed, logs);
        }

        logs.Add($"[CrossCompiler] Target: {config.Target}");
        logs.Add($"[CrossCompiler] Toolchain: {toolchain.GetCCCommand()}");

        // 建立輸出目錄
        var targetOutputDir = Path.Combine(config.OutputDir, config.Target.Triple);
        Directory.CreateDirectory(targetOutputDir);

        try
        {
            // 編譯所有源檔案
            var objectFiles = new List<string>();
            
            foreach (var srcFile in sourceFiles)
            {
                var objFile = Path.Combine(targetOutputDir, 
                    Path.GetFileNameWithoutExtension(srcFile) + config.Target.ObjectExtension);

                var compileResult = await CompileSourceAsync(srcFile, objFile, toolchain, config, logs, ct);
                
                if (!compileResult.success)
                {
                    sw.Stop();
                    return CrossCompileResult.Fail(compileResult.error!, config.Target, sw.Elapsed, logs);
                }

                objectFiles.Add(objFile);
            }

            // 如果只需要目標檔案，直接返回
            if (config.OutputType == CrossOutputType.Object)
            {
                sw.Stop();
                return CrossCompileResult.Ok(targetOutputDir, config.Target, sw.Elapsed, logs);
            }

            // 連結
            var outputPath = GetOutputPath(targetOutputDir, config);
            var linkResult = await LinkAsync(objectFiles, outputPath, toolchain, config, logs, ct);

            if (!linkResult.success)
            {
                sw.Stop();
                return CrossCompileResult.Fail(linkResult.error!, config.Target, sw.Elapsed, logs);
            }

            // 剝離符號
            if (config.Strip && !config.Debug)
            {
                await StripAsync(outputPath, toolchain, logs, ct);
            }

            sw.Stop();
            logs.Add($"[CrossCompiler] Success: {outputPath}");
            return CrossCompileResult.Ok(outputPath, config.Target, sw.Elapsed, logs);
        }
        catch (Exception ex)
        {
            sw.Stop();
            return CrossCompileResult.Fail($"Compilation failed: {ex.Message}", config.Target, sw.Elapsed, logs);
        }
    }

    private async Task<(bool success, string? error)> CompileSourceAsync(
        string srcFile, string objFile, CrossToolchain toolchain,
        CrossCompileConfig config, List<string> logs, CancellationToken ct)
    {
        var isCpp = srcFile.EndsWith(".cpp") || srcFile.EndsWith(".cxx") || srcFile.EndsWith(".cc");
        var cc = isCpp ? toolchain.GetCXXCommand() : toolchain.GetCCCommand();

        var args = new List<string> { "-c", $"\"{srcFile}\"", "-o", $"\"{objFile}\"" };

        // 優化等級
        args.Add($"-O{config.OptLevel}");

        // Debug 符號
        if (config.Debug)
        {
            args.Add("-g");
        }

        // 目標特定標誌
        foreach (var flag in toolchain.GetTargetCFlags())
        {
            args.Add(flag);
        }

        // 包含目錄
        foreach (var inc in config.IncludeDirs)
        {
            args.Add($"-I\"{inc}\"");
        }

        // 定義
        foreach (var (name, value) in config.Defines)
        {
            args.Add(value != null ? $"-D{name}={value}" : $"-D{name}");
        }

        // 平台特定定義
        AddPlatformDefines(args, config.Target);

        // 額外標誌
        foreach (var flag in config.ExtraCFlags)
        {
            args.Add(flag);
        }

        var cmdLine = string.Join(" ", args);
        logs.Add($"$ {cc} {cmdLine}");

        if (config.Verbose)
        {
            Console.WriteLine($"  [CC] {Path.GetFileName(srcFile)} -> {Path.GetFileName(objFile)}");
        }

        var result = await ProcessRunner.RunAsync(cc, cmdLine,
            new ProcessConfig { Timeout = TimeSpan.FromMinutes(5) }, ct);

        if (!string.IsNullOrEmpty(result.Stdout)) logs.Add(result.Stdout);
        if (!string.IsNullOrEmpty(result.Stderr)) logs.Add(result.Stderr);

        if (!result.IsSuccess)
        {
            return (false, $"Compilation failed for {srcFile}: {result.Stderr}");
        }

        return (true, null);
    }

    private async Task<(bool success, string? error)> LinkAsync(
        IReadOnlyList<string> objectFiles, string outputPath, CrossToolchain toolchain,
        CrossCompileConfig config, List<string> logs, CancellationToken ct)
    {
        var ld = toolchain.GetLDCommand();
        var args = new List<string>();

        // 目標特定連結標誌
        foreach (var flag in toolchain.GetTargetLdFlags())
        {
            args.Add(flag);
        }

        // 輸出類型
        switch (config.OutputType)
        {
            case CrossOutputType.SharedLibrary:
                if (config.Target.OS == TargetOS.MacOS)
                    args.Add("-dynamiclib");
                else
                    args.Add("-shared");
                args.Add("-fPIC");
                break;
            case CrossOutputType.StaticLibrary:
                // 使用 ar 建立靜態庫
                return await CreateStaticLibraryAsync(objectFiles, outputPath, toolchain, logs, ct);
        }

        // 輸出路徑
        args.Add("-o");
        args.Add($"\"{outputPath}\"");

        // 輸入目標檔案
        foreach (var obj in objectFiles)
        {
            args.Add($"\"{obj}\"");
        }

        // 庫目錄
        foreach (var libDir in config.LibDirs)
        {
            args.Add($"-L\"{libDir}\"");
        }

        // 庫
        foreach (var lib in config.Libraries)
        {
            args.Add($"-l{lib}");
        }

        // 平台特定庫
        AddPlatformLibs(args, config.Target);

        // 額外連結標誌
        foreach (var flag in config.ExtraLdFlags)
        {
            args.Add(flag);
        }

        var cmdLine = string.Join(" ", args);
        logs.Add($"$ {ld} {cmdLine}");

        if (config.Verbose)
        {
            Console.WriteLine($"  [LD] {Path.GetFileName(outputPath)}");
        }

        var result = await ProcessRunner.RunAsync(ld, cmdLine,
            new ProcessConfig { Timeout = TimeSpan.FromMinutes(10) }, ct);

        if (!string.IsNullOrEmpty(result.Stdout)) logs.Add(result.Stdout);
        if (!string.IsNullOrEmpty(result.Stderr)) logs.Add(result.Stderr);

        if (!result.IsSuccess)
        {
            return (false, $"Linking failed: {result.Stderr}");
        }

        return (true, null);
    }

    private async Task<(bool success, string? error)> CreateStaticLibraryAsync(
        IReadOnlyList<string> objectFiles, string outputPath, CrossToolchain toolchain,
        List<string> logs, CancellationToken ct)
    {
        var ar = toolchain.AR ?? "ar";
        var args = $"rcs \"{outputPath}\" {string.Join(" ", objectFiles.Select(f => $"\"{f}\""))}";

        logs.Add($"$ {ar} {args}");

        var result = await ProcessRunner.RunAsync(ar, args,
            new ProcessConfig { Timeout = TimeSpan.FromMinutes(5) }, ct);

        if (!string.IsNullOrEmpty(result.Stdout)) logs.Add(result.Stdout);
        if (!string.IsNullOrEmpty(result.Stderr)) logs.Add(result.Stderr);

        if (!result.IsSuccess)
        {
            return (false, $"Static library creation failed: {result.Stderr}");
        }

        return (true, null);
    }

    private async Task StripAsync(string path, CrossToolchain toolchain, List<string> logs, CancellationToken ct)
    {
        var strip = toolchain.Strip ?? "strip";
        
        logs.Add($"$ {strip} \"{path}\"");

        var result = await ProcessRunner.RunAsync(strip, $"\"{path}\"",
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);

        if (!string.IsNullOrEmpty(result.Stderr)) logs.Add(result.Stderr);
    }

    private static string GetOutputPath(string outputDir, CrossCompileConfig config)
    {
        var ext = config.OutputType switch
        {
            CrossOutputType.SharedLibrary => config.Target.SharedLibExtension,
            CrossOutputType.StaticLibrary => config.Target.StaticLibExtension,
            CrossOutputType.Executable => config.Target.ExecutableExtension,
            _ => config.Target.ObjectExtension
        };

        var prefix = config.OutputType switch
        {
            CrossOutputType.SharedLibrary when config.Target.OS != TargetOS.Windows => "lib",
            CrossOutputType.StaticLibrary when config.Target.OS != TargetOS.Windows => "lib",
            _ => ""
        };

        return Path.Combine(outputDir, prefix + config.OutputName + ext);
    }

    private static void AddPlatformDefines(List<string> args, CrossTarget target)
    {
        // 作業系統定義
        switch (target.OS)
        {
            case TargetOS.Linux:
                args.Add("-D__linux__");
                args.Add("-DLINUX");
                break;
            case TargetOS.Windows:
                args.Add("-D_WIN32");
                args.Add("-DWIN32");
                if (target.Arch == TargetArch.X86_64)
                    args.Add("-D_WIN64");
                break;
            case TargetOS.MacOS:
                args.Add("-D__APPLE__");
                args.Add("-D__MACH__");
                break;
            case TargetOS.Wasi:
                args.Add("-D__wasi__");
                break;
            case TargetOS.Freestanding:
                args.Add("-DWASM");
                args.Add("-D__wasm__");
                break;
        }

        // 架構定義
        switch (target.Arch)
        {
            case TargetArch.X86_64:
                args.Add("-D__x86_64__");
                break;
            case TargetArch.Arm64:
                args.Add("-D__aarch64__");
                break;
            case TargetArch.Wasm32:
                args.Add("-D__wasm32__");
                break;
        }
    }

    private static void AddPlatformLibs(List<string> args, CrossTarget target)
    {
        switch (target.OS)
        {
            case TargetOS.Linux:
                args.Add("-lpthread");
                args.Add("-lm");
                args.Add("-ldl");
                break;
            case TargetOS.Windows when target.Abi == "gnu":
                args.Add("-lkernel32");
                args.Add("-luser32");
                args.Add("-lws2_32");
                break;
            case TargetOS.MacOS:
                args.Add("-lSystem");
                break;
        }
    }
}
