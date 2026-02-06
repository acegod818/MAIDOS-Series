// MAIDOS-Forge Platform Linker
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Core.Platform;

namespace Forge.Core.Linker;

/// <summary>
/// 平台鏈接器介面
/// </summary>
/// <impl>
/// APPROACH: 抽象鏈接器操作，支援不同平台
/// CALLS: ProcessRunner
/// EDGES: 平台不支援時返回錯誤
/// </impl>
public interface IPlatformLinker
{
    /// <summary>取得鏈接器名稱</summary>
    string Name { get; }

    /// <summary>支援的平台</summary>
    IReadOnlyList<string> SupportedPlatforms { get; }

    /// <summary>檢查鏈接器是否可用</summary>
    Task<(bool Available, string Message)> CheckAvailabilityAsync(CancellationToken ct = default);

    /// <summary>執行鏈接</summary>
    Task<LinkResult> LinkAsync(IReadOnlyList<LinkInput> inputs, LinkConfig config, CancellationToken ct = default);
}

/// <summary>
/// GNU ld 鏈接器
/// </summary>
/// <impl>
/// APPROACH: 調用 ld 或 gcc 進行鏈接
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: ld 不可用時嘗試 gcc
/// </impl>
public sealed class GnuLinker : IPlatformLinker
{
    public string Name => "GNU ld";
    public IReadOnlyList<string> SupportedPlatforms => new[] { "linux", "freebsd" };

    private string _linkerCommand = "ld";
    private bool _useGcc = false;

    /// <summary>
    /// 檢查 ld 或 gcc 是否可用
    /// </summary>
    /// <impl>
    /// APPROACH: 嘗試 ld --version，失敗則嘗試 gcc
    /// CALLS: ProcessRunner.CommandExistsAsync()
    /// EDGES: 兩者都不可用返回失敗
    /// </impl>
    public async Task<(bool Available, string Message)> CheckAvailabilityAsync(CancellationToken ct = default)
    {
        // 優先嘗試 ld
        if (await ProcessRunner.CommandExistsAsync("ld"))
        {
            var version = await ProcessRunner.GetVersionAsync("ld", "--version");
            _linkerCommand = "ld";
            _useGcc = false;
            return (true, $"ld {version}");
        }

        // 回退到 gcc -o (作為前端調用 ld)
        if (await ProcessRunner.CommandExistsAsync("gcc"))
        {
            var version = await ProcessRunner.GetVersionAsync("gcc", "--version");
            _linkerCommand = "gcc";
            _useGcc = true;
            return (true, $"gcc {version} (as linker)");
        }

        return (false, "Neither ld nor gcc found");
    }

    /// <summary>
    /// 執行鏈接
    /// </summary>
    /// <impl>
    /// APPROACH: 構建 ld/gcc 命令行參數並執行
    /// CALLS: ProcessRunner.RunAsync()
    /// EDGES: 鏈接失敗返回錯誤訊息
    /// </impl>
    public async Task<LinkResult> LinkAsync(
        IReadOnlyList<LinkInput> inputs,
        LinkConfig config,
        CancellationToken ct = default)
    {
        var startTime = DateTime.UtcNow;
        var logs = new List<string>();

        // 過濾掉 CLR DLL (不能直接鏈接)
        var linkableInputs = inputs
            .Where(i => i.Type != LinkInputType.DotNetAssembly)
            .ToList();

        if (linkableInputs.Count == 0)
        {
            return LinkResult.Failure("No linkable inputs (CLR assemblies cannot be linked with native linker)");
        }

        // 構建輸出路徑
        var outputExt = config.OutputType switch
        {
            OutputType.SharedLibrary => config.Target.GetSharedLibExtension(),
            OutputType.StaticLibrary => config.Target.GetStaticLibExtension(),
            _ => config.Target.GetExecutableExtension()
        };

        var outputPath = Path.Combine(config.OutputDir, config.OutputName + outputExt);
        Directory.CreateDirectory(Path.GetDirectoryName(outputPath) ?? ".");

        // 構建命令
        var args = new List<string>();

        if (_useGcc)
        {
            // GCC 模式
            args.Add("-o");
            args.Add($"\"{outputPath}\"");

            if (config.OutputType == OutputType.SharedLibrary)
            {
                args.Add("-shared");
            }

            // 輸入檔案
            foreach (var input in linkableInputs)
            {
                args.Add($"\"{input.Path}\"");
            }

            // 庫路徑
            foreach (var libPath in config.LibPaths)
            {
                args.Add($"-L\"{libPath}\"");
            }

            // 系統庫
            foreach (var lib in config.SystemLibs)
            {
                if (lib.StartsWith("-l") || lib.StartsWith("-L"))
                {
                    args.Add(lib);
                }
                else
                {
                    args.Add($"-l{lib}");
                }
            }

            // 常用庫
            args.Add("-lpthread");
            args.Add("-lm");
            args.Add("-ldl");

            if (config.StripSymbols)
            {
                args.Add("-s");
            }
        }
        else
        {
            // 純 ld 模式
            args.Add("-o");
            args.Add($"\"{outputPath}\"");

            if (config.OutputType == OutputType.SharedLibrary)
            {
                args.Add("-shared");
            }

            // 動態鏈接器
            if (config.OutputType == OutputType.Executable)
            {
                args.Add("-dynamic-linker");
                args.Add("/lib64/ld-linux-x86-64.so.2");
            }

            // 輸入檔案
            foreach (var input in linkableInputs)
            {
                args.Add($"\"{input.Path}\"");
            }

            // 庫路徑
            foreach (var libPath in config.LibPaths)
            {
                args.Add($"-L\"{libPath}\"");
            }

            // 系統庫
            foreach (var lib in config.SystemLibs)
            {
                args.Add($"-l{lib.TrimStart('-').TrimStart('l')}");
            }

            if (config.StripSymbols)
            {
                args.Add("-s");
            }
        }

        // 額外標誌
        foreach (var flag in config.ExtraFlags)
        {
            args.Add(flag);
        }

        var cmdLine = string.Join(" ", args);
        logs.Add($"$ {_linkerCommand} {cmdLine}");

        if (config.Verbose)
        {
            Console.WriteLine($"  [{_linkerCommand}] {cmdLine}");
        }

        // 執行鏈接
        var result = await ProcessRunner.RunAsync(
            _linkerCommand,
            cmdLine,
            new ProcessConfig { Timeout = TimeSpan.FromMinutes(10) },
            ct);

        var duration = DateTime.UtcNow - startTime;

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
            return LinkResult.Failure(
                $"Link failed (exit code {result.ExitCode}): {result.Stderr}",
                duration,
                logs);
        }

        return LinkResult.Success(outputPath, duration, logs);
    }
}

/// <summary>
/// LLVM lld 鏈接器
/// </summary>
/// <impl>
/// APPROACH: 調用 lld 或 clang 進行鏈接
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: lld 不可用時嘗試 clang
/// </impl>
public sealed class LldLinker : IPlatformLinker
{
    public string Name => "LLVM lld";
    public IReadOnlyList<string> SupportedPlatforms => new[] { "linux", "macos", "windows", "freebsd" };

    private string _linkerCommand = "lld";
    private bool _useClang = false;

    public async Task<(bool Available, string Message)> CheckAvailabilityAsync(CancellationToken ct = default)
    {
        // 嘗試 lld
        foreach (var lldName in new[] { "ld.lld", "lld", "lld-link" })
        {
            if (await ProcessRunner.CommandExistsAsync(lldName))
            {
                var version = await ProcessRunner.GetVersionAsync(lldName, "--version");
                _linkerCommand = lldName;
                _useClang = false;
                return (true, $"{lldName} {version}");
            }
        }

        // 回退到 clang
        if (await ProcessRunner.CommandExistsAsync("clang"))
        {
            var version = await ProcessRunner.GetVersionAsync("clang", "--version");
            _linkerCommand = "clang";
            _useClang = true;
            return (true, $"clang {version} (as linker)");
        }

        return (false, "Neither lld nor clang found");
    }

    public async Task<LinkResult> LinkAsync(
        IReadOnlyList<LinkInput> inputs,
        LinkConfig config,
        CancellationToken ct = default)
    {
        var startTime = DateTime.UtcNow;
        var logs = new List<string>();

        var linkableInputs = inputs
            .Where(i => i.Type != LinkInputType.DotNetAssembly)
            .ToList();

        if (linkableInputs.Count == 0)
        {
            return LinkResult.Failure("No linkable inputs");
        }

        var outputExt = config.OutputType switch
        {
            OutputType.SharedLibrary => config.Target.GetSharedLibExtension(),
            OutputType.StaticLibrary => config.Target.GetStaticLibExtension(),
            _ => config.Target.GetExecutableExtension()
        };

        var outputPath = Path.Combine(config.OutputDir, config.OutputName + outputExt);
        Directory.CreateDirectory(Path.GetDirectoryName(outputPath) ?? ".");

        var args = new List<string>();

        if (_useClang)
        {
            args.Add($"-fuse-ld=lld");
        }

        args.Add("-o");
        args.Add($"\"{outputPath}\"");

        if (config.OutputType == OutputType.SharedLibrary)
        {
            args.Add("-shared");
        }

        foreach (var input in linkableInputs)
        {
            args.Add($"\"{input.Path}\"");
        }

        foreach (var libPath in config.LibPaths)
        {
            args.Add($"-L\"{libPath}\"");
        }

        foreach (var lib in config.SystemLibs)
        {
            args.Add($"-l{lib.TrimStart('-').TrimStart('l')}");
        }

        if (config.StripSymbols)
        {
            args.Add("-s");
        }

        foreach (var flag in config.ExtraFlags)
        {
            args.Add(flag);
        }

        var cmdLine = string.Join(" ", args);
        logs.Add($"$ {_linkerCommand} {cmdLine}");

        var result = await ProcessRunner.RunAsync(
            _linkerCommand, cmdLine,
            new ProcessConfig { Timeout = TimeSpan.FromMinutes(10) }, ct);

        var duration = DateTime.UtcNow - startTime;

        if (!string.IsNullOrEmpty(result.Stdout)) logs.Add(result.Stdout);
        if (!string.IsNullOrEmpty(result.Stderr)) logs.Add(result.Stderr);

        if (!result.IsSuccess)
        {
            return LinkResult.Failure($"Link failed: {result.Stderr}", duration, logs);
        }

        return LinkResult.Success(outputPath, duration, logs);
    }
}

/// <summary>
/// MSVC link.exe 鏈接器
/// </summary>
/// <impl>
/// APPROACH: 調用 MSVC link.exe
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: 僅支援 Windows
/// </impl>
public sealed class MsvcLinker : IPlatformLinker
{
    public string Name => "MSVC link.exe";
    public IReadOnlyList<string> SupportedPlatforms => new[] { "windows" };

    public async Task<(bool Available, string Message)> CheckAvailabilityAsync(CancellationToken ct = default)
    {
        if (!OperatingSystem.IsWindows())
        {
            return (false, "MSVC only available on Windows");
        }

        if (await ProcessRunner.CommandExistsAsync("link"))
        {
            return (true, "MSVC link.exe");
        }

        return (false, "MSVC link.exe not found. Install Visual Studio Build Tools.");
    }

    public async Task<LinkResult> LinkAsync(
        IReadOnlyList<LinkInput> inputs,
        LinkConfig config,
        CancellationToken ct = default)
    {
        var startTime = DateTime.UtcNow;
        var logs = new List<string>();

        var linkableInputs = inputs
            .Where(i => i.Type != LinkInputType.DotNetAssembly)
            .ToList();

        if (linkableInputs.Count == 0)
        {
            return LinkResult.Failure("No linkable inputs");
        }

        var outputExt = config.OutputType switch
        {
            OutputType.SharedLibrary => ".dll",
            OutputType.StaticLibrary => ".lib",
            _ => ".exe"
        };

        var outputPath = Path.Combine(config.OutputDir, config.OutputName + outputExt);
        Directory.CreateDirectory(Path.GetDirectoryName(outputPath) ?? ".");

        var args = new List<string>
        {
            $"/OUT:\"{outputPath}\"",
            "/NOLOGO"
        };

        if (config.OutputType == OutputType.SharedLibrary)
        {
            args.Add("/DLL");
        }

        foreach (var input in linkableInputs)
        {
            args.Add($"\"{input.Path}\"");
        }

        foreach (var libPath in config.LibPaths)
        {
            args.Add($"/LIBPATH:\"{libPath}\"");
        }

        foreach (var lib in config.SystemLibs)
        {
            var libName = lib.EndsWith(".lib") ? lib : $"{lib}.lib";
            args.Add(libName);
        }

        // 標準庫
        args.Add("kernel32.lib");
        args.Add("user32.lib");
        args.Add("ucrt.lib");
        args.Add("vcruntime.lib");

        foreach (var flag in config.ExtraFlags)
        {
            args.Add(flag);
        }

        var cmdLine = string.Join(" ", args);
        logs.Add($"$ link {cmdLine}");

        var result = await ProcessRunner.RunAsync(
            "link", cmdLine,
            new ProcessConfig { Timeout = TimeSpan.FromMinutes(10) }, ct);

        var duration = DateTime.UtcNow - startTime;

        if (!string.IsNullOrEmpty(result.Stdout)) logs.Add(result.Stdout);
        if (!string.IsNullOrEmpty(result.Stderr)) logs.Add(result.Stderr);

        if (!result.IsSuccess)
        {
            return LinkResult.Failure($"Link failed: {result.Stderr}", duration, logs);
        }

        return LinkResult.Success(outputPath, duration, logs);
    }
}

/// <summary>
/// Apple ld64 鏈接器
/// </summary>
/// <impl>
/// APPROACH: 調用 macOS 的 ld 或 clang
/// CALLS: ProcessRunner.RunAsync()
/// EDGES: 僅支援 macOS
/// </impl>
public sealed class AppleLinker : IPlatformLinker
{
    public string Name => "Apple ld64";
    public IReadOnlyList<string> SupportedPlatforms => new[] { "macos", "ios" };

    public async Task<(bool Available, string Message)> CheckAvailabilityAsync(CancellationToken ct = default)
    {
        if (!OperatingSystem.IsMacOS())
        {
            return (false, "Apple linker only available on macOS");
        }

        // macOS 使用 clang 作為前端
        if (await ProcessRunner.CommandExistsAsync("clang"))
        {
            var version = await ProcessRunner.GetVersionAsync("clang", "--version");
            return (true, $"Apple clang {version}");
        }

        return (false, "clang not found. Install Xcode Command Line Tools.");
    }

    public async Task<LinkResult> LinkAsync(
        IReadOnlyList<LinkInput> inputs,
        LinkConfig config,
        CancellationToken ct = default)
    {
        var startTime = DateTime.UtcNow;
        var logs = new List<string>();

        var linkableInputs = inputs
            .Where(i => i.Type != LinkInputType.DotNetAssembly)
            .ToList();

        if (linkableInputs.Count == 0)
        {
            return LinkResult.Failure("No linkable inputs");
        }

        var outputExt = config.OutputType switch
        {
            OutputType.SharedLibrary => ".dylib",
            OutputType.StaticLibrary => ".a",
            _ => ""
        };

        var outputPath = Path.Combine(config.OutputDir, config.OutputName + outputExt);
        Directory.CreateDirectory(Path.GetDirectoryName(outputPath) ?? ".");

        var args = new List<string>
        {
            "-o",
            $"\"{outputPath}\""
        };

        if (config.OutputType == OutputType.SharedLibrary)
        {
            args.Add("-dynamiclib");
        }

        foreach (var input in linkableInputs)
        {
            args.Add($"\"{input.Path}\"");
        }

        foreach (var libPath in config.LibPaths)
        {
            args.Add($"-L\"{libPath}\"");
        }

        foreach (var lib in config.SystemLibs)
        {
            if (lib.StartsWith("-framework"))
            {
                args.Add(lib);
            }
            else
            {
                args.Add($"-l{lib.TrimStart('-').TrimStart('l')}");
            }
        }

        // 系統庫
        args.Add("-lSystem");

        if (config.StripSymbols)
        {
            args.Add("-Wl,-dead_strip");
        }

        foreach (var flag in config.ExtraFlags)
        {
            args.Add(flag);
        }

        var cmdLine = string.Join(" ", args);
        logs.Add($"$ clang {cmdLine}");

        var result = await ProcessRunner.RunAsync(
            "clang", cmdLine,
            new ProcessConfig { Timeout = TimeSpan.FromMinutes(10) }, ct);

        var duration = DateTime.UtcNow - startTime;

        if (!string.IsNullOrEmpty(result.Stdout)) logs.Add(result.Stdout);
        if (!string.IsNullOrEmpty(result.Stderr)) logs.Add(result.Stderr);

        if (!result.IsSuccess)
        {
            return LinkResult.Failure($"Link failed: {result.Stderr}", duration, logs);
        }

        return LinkResult.Success(outputPath, duration, logs);
    }
}
