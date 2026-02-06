// MAIDOS-Forge Multi-Target Build Orchestrator
// Code-QC v2.2B Compliant | M14 Cross-Compilation Module

using System.Collections.Concurrent;
using System.Diagnostics;

namespace Forge.Core.CrossCompile;

/// <summary>
/// 多目標編譯結果
/// </summary>
public sealed class MultiTargetBuildResult
{
    public bool AllSucceeded { get; init; }
    public int SuccessCount { get; init; }
    public int FailureCount { get; init; }
    public TimeSpan TotalDuration { get; init; }
    public IReadOnlyList<(CrossTarget Target, CrossCompileResult Result)> Results { get; init; } 
        = Array.Empty<(CrossTarget, CrossCompileResult)>();

    public static MultiTargetBuildResult Create(
        IReadOnlyList<(CrossTarget, CrossCompileResult)> results,
        TimeSpan duration)
    {
        var successes = results.Count(r => r.Item2.Success);
        return new MultiTargetBuildResult
        {
            AllSucceeded = successes == results.Count,
            SuccessCount = successes,
            FailureCount = results.Count - successes,
            TotalDuration = duration,
            Results = results
        };
    }
}

/// <summary>
/// 多目標編譯配置
/// </summary>
public sealed class MultiTargetConfig
{
    /// <summary>目標平台列表</summary>
    public IReadOnlyList<CrossTarget> Targets { get; init; } = Array.Empty<CrossTarget>();
    
    /// <summary>輸出根目錄</summary>
    public string OutputDir { get; init; } = "build";
    
    /// <summary>輸出名稱</summary>
    public string OutputName { get; init; } = "output";
    
    /// <summary>輸出類型</summary>
    public CrossOutputType OutputType { get; init; } = CrossOutputType.Executable;
    
    /// <summary>優化等級</summary>
    public string OptLevel { get; init; } = "2";
    
    /// <summary>是否並行編譯</summary>
    public bool Parallel { get; init; } = true;
    
    /// <summary>最大並行數</summary>
    public int MaxParallelism { get; init; } = Environment.ProcessorCount;
    
    /// <summary>遇到錯誤時是否繼續</summary>
    public bool ContinueOnError { get; init; } = true;
    
    /// <summary>是否剝離符號</summary>
    public bool Strip { get; init; } = true;
    
    /// <summary>是否生成 Debug 資訊</summary>
    public bool Debug { get; init; }
    
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
    
    /// <summary>是否顯示詳細輸出</summary>
    public bool Verbose { get; init; }

    /// <summary>預設多目標配置 (Windows + Linux + macOS + WASM)</summary>
    public static MultiTargetConfig DefaultTargets => new()
    {
        Targets = new[]
        {
            CrossTarget.LinuxX64,
            CrossTarget.WindowsX64,
            CrossTarget.MacOSX64,
            CrossTarget.Wasm32Wasi
        }
    };

    /// <summary>所有桌面平台</summary>
    public static MultiTargetConfig AllDesktop => new()
    {
        Targets = new[]
        {
            CrossTarget.LinuxX64,
            CrossTarget.LinuxArm64,
            CrossTarget.WindowsX64,
            CrossTarget.WindowsArm64,
            CrossTarget.MacOSX64,
            CrossTarget.MacOSArm64
        }
    };
}

/// <summary>
/// 多目標編譯協調器
/// </summary>
public sealed class MultiTargetOrchestrator
{
    private readonly CrossCompiler _compiler;
    private readonly WasmCompiler _wasmCompiler;

    public MultiTargetOrchestrator(CrossCompiler? compiler = null, WasmCompiler? wasmCompiler = null)
    {
        _compiler = compiler ?? new CrossCompiler();
        _wasmCompiler = wasmCompiler ?? new WasmCompiler();
    }

    /// <summary>編譯到多個目標平台</summary>
    public async Task<MultiTargetBuildResult> BuildAsync(
        IReadOnlyList<string> sourceFiles,
        MultiTargetConfig config,
        IProgress<(CrossTarget target, string status)>? progress = null,
        CancellationToken ct = default)
    {
        var sw = Stopwatch.StartNew();
        var results = new ConcurrentBag<(CrossTarget, CrossCompileResult)>();

        if (config.Targets.Count == 0)
        {
            sw.Stop();
            return MultiTargetBuildResult.Create(Array.Empty<(CrossTarget, CrossCompileResult)>(), sw.Elapsed);
        }

        Console.WriteLine($"[MultiTarget] Building for {config.Targets.Count} target(s)...");

        if (config.Parallel && config.Targets.Count > 1)
        {
            // 並行編譯
            var semaphore = new SemaphoreSlim(config.MaxParallelism);
            var tasks = config.Targets.Select(async target =>
            {
                await semaphore.WaitAsync(ct);
                try
                {
                    progress?.Report((target, "Building..."));
                    var result = await BuildSingleTargetAsync(sourceFiles, target, config, ct);
                    results.Add((target, result));
                    
                    var status = result.Success ? "✓" : "✗";
                    progress?.Report((target, status));
                    Console.WriteLine($"  [{status}] {target.Triple}");

                    if (!result.Success && !config.ContinueOnError)
                    {
                        throw new OperationCanceledException($"Build failed for {target}");
                    }
                }
                finally
                {
                    semaphore.Release();
                }
            });

            await Task.WhenAll(tasks);
        }
        else
        {
            // 串行編譯
            foreach (var target in config.Targets)
            {
                progress?.Report((target, "Building..."));
                var result = await BuildSingleTargetAsync(sourceFiles, target, config, ct);
                results.Add((target, result));

                var status = result.Success ? "✓" : "✗";
                progress?.Report((target, status));
                Console.WriteLine($"  [{status}] {target.Triple}");

                if (!result.Success && !config.ContinueOnError)
                {
                    break;
                }
            }
        }

        sw.Stop();
        var orderedResults = results.OrderBy(r => r.Item1.Triple).ToList();
        return MultiTargetBuildResult.Create(orderedResults, sw.Elapsed);
    }

    private async Task<CrossCompileResult> BuildSingleTargetAsync(
        IReadOnlyList<string> sourceFiles,
        CrossTarget target,
        MultiTargetConfig config,
        CancellationToken ct)
    {
        // WASM 目標使用專用編譯器
        if (target.Arch is TargetArch.Wasm32 or TargetArch.Wasm64)
        {
            var wasmConfig = new WasmCompileConfig
            {
                Target = target.OS == TargetOS.Wasi ? WasmTarget.Wasi : WasmTarget.Freestanding,
                OutputDir = Path.Combine(config.OutputDir, target.Triple),
                OutputName = config.OutputName,
                OptLevel = config.OptLevel,
                Debug = config.Debug,
                GenerateJs = true,
                GenerateDts = true,
                ExtraFlags = config.ExtraCFlags.ToList()
            };

            var wasmResult = await _wasmCompiler.CompileAsync(sourceFiles, wasmConfig, ct);
            
            return new CrossCompileResult
            {
                Success = wasmResult.Success,
                OutputPath = wasmResult.WasmPath,
                Target = target,
                Duration = wasmResult.Duration,
                Logs = wasmResult.Logs,
                ErrorMessage = wasmResult.ErrorMessage
            };
        }

        // 一般目標使用交叉編譯器
        var crossConfig = new CrossCompileConfig
        {
            Target = target,
            OutputType = config.OutputType,
            OutputDir = config.OutputDir,
            OutputName = config.OutputName,
            OptLevel = config.OptLevel,
            Debug = config.Debug,
            Strip = config.Strip && !config.Debug,
            ExtraCFlags = config.ExtraCFlags,
            ExtraLdFlags = config.ExtraLdFlags,
            IncludeDirs = config.IncludeDirs,
            LibDirs = config.LibDirs,
            Libraries = config.Libraries,
            Verbose = config.Verbose
        };

        return await _compiler.CompileAsync(sourceFiles, crossConfig, ct);
    }

    /// <summary>檢查所有目標的工具鏈可用性</summary>
    public async Task<IReadOnlyList<(CrossTarget target, bool available, string message)>> CheckToolchainsAsync(
        IReadOnlyList<CrossTarget> targets,
        CancellationToken ct = default)
    {
        var results = new List<(CrossTarget, bool, string)>();
        var toolchainManager = new CrossToolchainManager();

        foreach (var target in targets)
        {
            if (target.Arch is TargetArch.Wasm32 or TargetArch.Wasm64)
            {
                var wasmConfig = new WasmCompileConfig
                {
                    Target = target.OS == TargetOS.Wasi ? WasmTarget.Wasi : WasmTarget.Freestanding
                };
                var (available, message) = await _wasmCompiler.InitializeAsync(wasmConfig, ct);
                results.Add((target, available, message));
            }
            else
            {
                var toolchain = await toolchainManager.GetToolchainAsync(target, ct);
                results.Add((target, toolchain.IsAvailable, toolchain.ValidationMessage ?? "Unknown"));
            }
        }

        return results;
    }
}

/// <summary>
/// 交叉編譯矩陣 - 用於 CI/CD
/// </summary>
public sealed class CrossCompileMatrix
{
    /// <summary>平台 x 架構矩陣</summary>
    public static IReadOnlyList<CrossTarget> GenerateMatrix(
        IReadOnlyList<TargetOS> operatingSystems,
        IReadOnlyList<TargetArch> architectures)
    {
        var targets = new List<CrossTarget>();

        foreach (var os in operatingSystems)
        {
            foreach (var arch in architectures)
            {
                // 過濾不支援的組合
                if (!IsValidCombination(os, arch))
                    continue;

                targets.Add(new CrossTarget
                {
                    OS = os,
                    Arch = arch,
                    Vendor = GetDefaultVendor(os),
                    Abi = GetDefaultAbi(os, arch)
                });
            }
        }

        return targets;
    }

    /// <summary>標準 CI 矩陣</summary>
    public static IReadOnlyList<CrossTarget> StandardCIMatrix => GenerateMatrix(
        new[] { TargetOS.Linux, TargetOS.Windows, TargetOS.MacOS },
        new[] { TargetArch.X86_64, TargetArch.Arm64 }
    );

    /// <summary>完整矩陣（含 WASM）</summary>
    public static IReadOnlyList<CrossTarget> FullMatrix => StandardCIMatrix
        .Concat(new[] { CrossTarget.Wasm32Wasi })
        .ToList();

    private static bool IsValidCombination(TargetOS os, TargetArch arch)
    {
        return (os, arch) switch
        {
            // WASM 只支援 Wasm32/Wasm64
            (TargetOS.Wasi or TargetOS.Freestanding, TargetArch.Wasm32 or TargetArch.Wasm64) => true,
            (TargetOS.Wasi or TargetOS.Freestanding, _) => false,
            (_, TargetArch.Wasm32 or TargetArch.Wasm64) => false,
            
            // iOS 只支援 ARM
            (TargetOS.iOS, TargetArch.X86_64 or TargetArch.X86) => false,
            
            // 一般組合
            _ => true
        };
    }

    private static string GetDefaultVendor(TargetOS os) => os switch
    {
        TargetOS.Windows => "pc",
        TargetOS.MacOS or TargetOS.iOS => "apple",
        _ => "unknown"
    };

    private static string? GetDefaultAbi(TargetOS os, TargetArch arch) => os switch
    {
        TargetOS.Linux => "gnu",
        TargetOS.Windows => "msvc",
        _ => null
    };
}
