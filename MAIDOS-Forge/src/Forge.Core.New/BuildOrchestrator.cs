// MAIDOS-Forge Build Orchestrator
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Core.Build;
using Forge.Core.Cache;
using Forge.Core.Config;
using Forge.Core.FFI;
using Forge.Core.Linker;
using Forge.Core.Plugin;

namespace Forge.Core.Orchestration;

/// <summary>
/// 編譯流程階段
/// </summary>
public enum BuildPhase
{
    Init,
    DependencyAnalysis,
    Compilation,
    InterfaceExtraction,
    GlueGeneration,
    Linking,
    Complete
}

/// <summary>
/// 編譯進度回調
/// </summary>
public delegate void BuildProgressCallback(BuildPhase phase, string message, int current, int total);

/// <summary>
/// 編譯選項
/// </summary>
/// <impl>
/// APPROACH: 封裝完整編譯流程的所有選項
/// CALLS: N/A (純資料)
/// EDGES: 預設值適用於大多數情況
/// </impl>
public sealed class BuildOptions
{
    /// <summary>編譯配置 (debug/release)</summary>
    public string Profile { get; init; } = "release";

    /// <summary>輸出類型</summary>
    public OutputType OutputType { get; init; } = OutputType.Executable;

    /// <summary>輸出名稱</summary>
    public string? OutputName { get; init; }

    /// <summary>是否只編譯不鏈接</summary>
    public bool CompileOnly { get; init; } = false;

    /// <summary>是否生成膠水代碼</summary>
    public bool GenerateGlue { get; init; } = true;

    /// <summary>是否剝離符號</summary>
    public bool StripSymbols { get; init; } = false;

    /// <summary>詳細輸出</summary>
    public bool Verbose { get; init; } = false;

    /// <summary>只編譯指定模組</summary>
    public string? TargetModule { get; init; }

    /// <summary>Dry run (只顯示計畫)</summary>
    public bool DryRun { get; init; } = false;

    /// <summary>增量編譯 (預設啟用)</summary>
    public bool Incremental { get; init; } = true;

    /// <summary>強制全量重建</summary>
    public bool ForceRebuild { get; init; } = false;

    /// <summary>進度回調</summary>
    public BuildProgressCallback? ProgressCallback { get; init; }
}

/// <summary>
/// 編譯結果
/// </summary>
/// <impl>
/// APPROACH: 封裝完整編譯流程結果
/// CALLS: N/A (純資料)
/// EDGES: IsSuccess 為 false 時 Error 非空
/// </impl>
public sealed class BuildResult
{
    public bool IsSuccess { get; }
    public string Error { get; }
    public string OutputPath { get; }
    public TimeSpan TotalDuration { get; }
    public IReadOnlyList<ModuleBuildResult> ModuleResults { get; }
    public IReadOnlyList<string> GeneratedGlueFiles { get; }

    private BuildResult(
        bool isSuccess,
        string error,
        string outputPath,
        TimeSpan totalDuration,
        IReadOnlyList<ModuleBuildResult>? moduleResults,
        IReadOnlyList<string>? glueFiles)
    {
        IsSuccess = isSuccess;
        Error = error;
        OutputPath = outputPath;
        TotalDuration = totalDuration;
        ModuleResults = moduleResults ?? Array.Empty<ModuleBuildResult>();
        GeneratedGlueFiles = glueFiles ?? Array.Empty<string>();
    }

    public static BuildResult Success(
        string outputPath,
        TimeSpan duration,
        IReadOnlyList<ModuleBuildResult> moduleResults,
        IReadOnlyList<string>? glueFiles = null)
        => new(true, string.Empty, outputPath, duration, moduleResults, glueFiles);

    public static BuildResult Failure(
        string error,
        TimeSpan duration = default,
        IReadOnlyList<ModuleBuildResult>? moduleResults = null)
        => new(false, error, string.Empty, duration, moduleResults, null);
}

/// <summary>
/// 單一模組編譯結果
/// </summary>
public sealed class ModuleBuildResult
{
    public string ModuleName { get; init; } = string.Empty;
    public bool IsSuccess { get; init; }
    public bool Skipped { get; init; } = false;
    public IReadOnlyList<string> Artifacts { get; init; } = Array.Empty<string>();
    public TimeSpan Duration { get; init; }
    public string? Error { get; init; }
}

/// <summary>
/// 編譯流程整合器 - 協調完整的編譯到鏈接流程
/// </summary>
/// <impl>
/// APPROACH: 整合 ConfigParser, DependencyAnalyzer, PluginHost, LinkerManager, IncrementalBuildManager
/// CALLS: 各子系統的方法
/// EDGES: 任一階段失敗則終止並返回錯誤
/// </impl>
public sealed class BuildOrchestrator
{
    private readonly PluginHost _pluginHost;
    private readonly LinkerManager _linkerManager;

    public BuildOrchestrator()
    {
        _pluginHost = new PluginHost();
        _pluginHost.RegisterBuiltinPlugins();

        _linkerManager = new LinkerManager();
    }

    /// <summary>
    /// 執行完整編譯流程
    /// </summary>
    /// <impl>
    /// APPROACH: 依序執行: 配置解析 → 依賴分析 → 編譯 → 接口提取 → 膠水生成 → 鏈接
    /// CALLS: ParseConfig(), AnalyzeDependencies(), CompileModules(), ExtractInterfaces(), GenerateGlue(), Link()
    /// EDGES: 任一階段失敗返回錯誤
    /// </impl>
    public async Task<BuildResult> BuildAsync(
        string projectPath,
        BuildOptions options,
        CancellationToken ct = default)
    {
        var startTime = DateTime.UtcNow;
        var moduleResults = new List<ModuleBuildResult>();
        var glueFiles = new List<string>();

        // === Phase 1: 配置解析 ===
        options.ProgressCallback?.Invoke(BuildPhase.Init, "Parsing configuration...", 0, 6);

        var parseResult = ConfigParser.ParseProject(projectPath);
        if (!parseResult.IsSuccess)
        {
            return BuildResult.Failure(parseResult.Error);
        }

        var config = parseResult.Value!;
        var outputName = options.OutputName ?? config.Config.Name;

        // === Phase 2: 依賴分析 ===
        options.ProgressCallback?.Invoke(BuildPhase.DependencyAnalysis, "Analyzing dependencies...", 1, 6);

        var analyzeResult = DependencyAnalyzer.Analyze(config);
        if (analyzeResult.HasCycle || !string.IsNullOrEmpty(analyzeResult.Error))
        {
            return BuildResult.Failure(analyzeResult.Error);
        }

        var depGraph = analyzeResult.Graph;

        // 計算編譯順序
        var scheduleResult = BuildScheduler.CreateSchedule(depGraph);
        if (!scheduleResult.IsSuccess)
        {
            return BuildResult.Failure(scheduleResult.Error);
        }

        var buildSchedule = scheduleResult.Schedule!;

        // 轉換為模組名稱列表
        var buildLayers = buildSchedule.Layers
            .Select(layer => layer.Modules.Select(m => m.Config.Name).ToList())
            .ToList();

        // 過濾目標模組
        if (!string.IsNullOrEmpty(options.TargetModule))
        {
            var targetModule = config.Modules.FirstOrDefault(m =>
                string.Equals(m.Config.Name, options.TargetModule, StringComparison.OrdinalIgnoreCase));

            if (targetModule is null)
            {
                return BuildResult.Failure($"Module not found: {options.TargetModule}");
            }

            // 找出目標模組及其依賴
            var deps = GetAllDependencies(depGraph, options.TargetModule);
            deps.Add(options.TargetModule);

            buildLayers = buildLayers
                .Select(layer => layer.Where(m => deps.Contains(m)).ToList())
                .Where(layer => layer.Count > 0)
                .ToList();
        }

        // Dry run
        if (options.DryRun)
        {
            var plan = FormatBuildPlan(buildLayers, config);
            return BuildResult.Success(plan, TimeSpan.Zero, moduleResults);
        }

        // === Phase 3: 編譯 ===
        options.ProgressCallback?.Invoke(BuildPhase.Compilation, "Compiling modules...", 2, 6);

        var buildDir = Path.Combine(config.ProjectRoot, config.Config.Output.Dir, options.Profile);
        Directory.CreateDirectory(buildDir);

        var compileConfig = new CompileConfig
        {
            Profile = options.Profile,
            OutputDir = buildDir,
            TargetPlatform = TargetPlatform.Current.ToTriple(),
            Verbose = options.Verbose,
            Environment = new Dictionary<string, string>()
        };

        var compiledModules = new Dictionary<string, (ValidatedModuleConfig Module, IReadOnlyList<string> Artifacts)>();

        // 增量編譯快取
        IncrementalBuildManager? cacheManager = null;
        if (options.Incremental && !options.ForceRebuild)
        {
            cacheManager = new IncrementalBuildManager(config.ProjectRoot);
            cacheManager.LoadCache();
        }

        // 追蹤已重建的模組（用於傳遞式失效）
        var rebuiltModules = new HashSet<string>();

        foreach (var layer in buildLayers)
        {
            // 同層級並行編譯
            var layerTasks = layer.Select(async moduleName =>
            {
                var module = config.Modules.First(m => m.Config.Name == moduleName);
                var plugin = _pluginHost.GetPlugin(module.Config.Language);

                if (plugin is null)
                {
                    return new ModuleBuildResult
                    {
                        ModuleName = moduleName,
                        IsSuccess = false,
                        Error = $"No plugin for language: {module.Config.Language}"
                    };
                }

                // 增量編譯檢查
                if (cacheManager is not null)
                {
                    // 檢查依賴是否重建
                    var depsRebuilt = module.Config.Dependencies.Any(d => rebuiltModules.Contains(d));

                    if (!depsRebuilt)
                    {
                        var checkResult = cacheManager.CheckModule(
                            module.ModulePath,
                            moduleName,
                            module.Config.Language,
                            options.Profile,
                            module.Config.Dependencies);

                        if (!checkResult.NeedsRebuild && checkResult.CacheEntry is not null)
                        {
                            if (options.Verbose)
                            {
                                Console.WriteLine($"  Skipped: {moduleName} ({checkResult.Reason})");
                            }

                            compiledModules[moduleName] = (module, checkResult.CacheEntry.ArtifactPaths);

                            return new ModuleBuildResult
                            {
                                ModuleName = moduleName,
                                IsSuccess = true,
                                Skipped = true,
                                Artifacts = checkResult.CacheEntry.ArtifactPaths,
                                Duration = TimeSpan.Zero
                            };
                        }
                    }
                }

                if (options.Verbose)
                {
                    Console.WriteLine($"  Compiling: {moduleName} ({module.Config.Language})");
                }

                var compileResult = await plugin.CompileAsync(module, compileConfig, ct);

                if (compileResult.IsSuccess)
                {
                    compiledModules[moduleName] = (module, compileResult.Artifacts);
                    rebuiltModules.Add(moduleName);

                    // 更新快取
                    cacheManager?.UpdateModuleCache(
                        module.ModulePath,
                        moduleName,
                        module.Config.Language,
                        options.Profile,
                        module.Config.Dependencies,
                        compileResult.Artifacts);
                }

                return new ModuleBuildResult
                {
                    ModuleName = moduleName,
                    IsSuccess = compileResult.IsSuccess,
                    Artifacts = compileResult.Artifacts,
                    Duration = compileResult.Duration,
                    Error = compileResult.IsSuccess ? null : compileResult.Error
                };
            });

            var layerResults = await Task.WhenAll(layerTasks);
            moduleResults.AddRange(layerResults);

            // 檢查是否有失敗
            var failed = layerResults.FirstOrDefault(r => !r.IsSuccess);
            if (failed is not null)
            {
                cacheManager?.SaveCache();
                var duration = DateTime.UtcNow - startTime;
                return BuildResult.Failure(
                    $"Compilation failed for module {failed.ModuleName}: {failed.Error}",
                    duration, moduleResults);
            }
        }

        // 儲存快取
        cacheManager?.SaveCache();

        // 如果只編譯不鏈接
        if (options.CompileOnly)
        {
            var duration = DateTime.UtcNow - startTime;
            return BuildResult.Success(buildDir, duration, moduleResults);
        }

        // === Phase 4: 接口提取 ===
        options.ProgressCallback?.Invoke(BuildPhase.InterfaceExtraction, "Extracting interfaces...", 3, 6);

        var interfaces = new Dictionary<string, ModuleInterface>();

        foreach (var (moduleName, (module, artifacts)) in compiledModules)
        {
            if (artifacts.Count == 0) continue;

            var artifactPath = artifacts[0];
            var extractResult = await InterfaceExtractor.ExtractAsync(
                artifactPath, moduleName, module.Config.Language, ct);

            if (extractResult.IsSuccess && extractResult.Interface is not null)
            {
                interfaces[moduleName] = extractResult.Interface;

                if (options.Verbose)
                {
                    Console.WriteLine($"  Extracted: {moduleName} ({extractResult.Interface.Exports.Count} exports)");
                }
            }
        }

        // === Phase 5: 膠水代碼生成 ===
        if (options.GenerateGlue && interfaces.Count > 0)
        {
            options.ProgressCallback?.Invoke(BuildPhase.GlueGeneration, "Generating glue code...", 4, 6);

            var glueDir = Path.Combine(buildDir, "glue");
            Directory.CreateDirectory(glueDir);

            foreach (var (moduleName, moduleInterface) in interfaces)
            {
                var (module, _) = compiledModules[moduleName];

                // 找出依賴此模組的其他模組
                var dependents = compiledModules
                    .Where(kv => kv.Key != moduleName)
                    .Select(kv => kv.Value.Module)
                    .Where(m => m.Config.Dependencies.Contains(moduleName))
                    .ToList();

                foreach (var dependent in dependents)
                {
                    var targetLang = dependent.Config.Language;
                    var glueResult = GlueGenerator.Generate(moduleInterface, targetLang);

                    if (glueResult.IsSuccess)
                    {
                        var glueFile = Path.Combine(glueDir,
                            $"{moduleName}_to_{dependent.Config.Name}.{GetExtension(targetLang)}");
                        File.WriteAllText(glueFile, glueResult.SourceCode);
                        glueFiles.Add(glueFile);

                        if (options.Verbose)
                        {
                            Console.WriteLine($"  Generated: {Path.GetFileName(glueFile)}");
                        }
                    }
                }
            }
        }

        // === Phase 6: 鏈接 ===
        options.ProgressCallback?.Invoke(BuildPhase.Linking, "Linking...", 5, 6);

        var linkInputs = LinkerManager.CollectInputs(
            buildDir,
            compiledModules.Select(kv => (kv.Key, kv.Value.Module.Config.Language)).ToList());

        var linkConfig = new LinkConfig
        {
            OutputName = outputName,
            OutputDir = buildDir,
            OutputType = options.OutputType,
            Target = TargetPlatform.Current,
            StripSymbols = options.StripSymbols,
            Profile = options.Profile,
            Verbose = options.Verbose
        };

        var linkResult = await _linkerManager.LinkAsync(linkInputs, linkConfig, ct);

        options.ProgressCallback?.Invoke(BuildPhase.Complete, "Build complete", 6, 6);

        var totalDuration = DateTime.UtcNow - startTime;

        if (!linkResult.IsSuccess)
        {
            return BuildResult.Failure(linkResult.Error, totalDuration, moduleResults);
        }

        return BuildResult.Success(linkResult.OutputPath, totalDuration, moduleResults, glueFiles);
    }

    /// <summary>
    /// 格式化編譯計畫
    /// </summary>
    private static string FormatBuildPlan(
        IReadOnlyList<List<string>> layers,
        ValidatedForgeConfig config)
    {
        var sb = new System.Text.StringBuilder();
        sb.AppendLine("Build Plan:");
        sb.AppendLine();

        for (int i = 0; i < layers.Count; i++)
        {
            sb.AppendLine($"Layer {i} (parallel):");
            foreach (var moduleName in layers[i])
            {
                var module = config.Modules.First(m => m.Config.Name == moduleName);
                sb.AppendLine($"  - {moduleName} ({module.Config.Language})");
            }
        }

        return sb.ToString();
    }

    private static string GetExtension(string language) => language.ToLowerInvariant() switch
    {
        "csharp" or "c#" => "cs",
        "rust" => "rs",
        "c" => "h",
        _ => "txt"
    };

    /// <summary>
    /// 取得模組的所有依賴（遞迴）
    /// </summary>
    private static HashSet<string> GetAllDependencies(DependencyGraph graph, string moduleName)
    {
        var result = new HashSet<string>();
        var node = graph.GetNode(moduleName);
        
        if (node is null)
        {
            return result;
        }

        foreach (var dep in node.Dependencies)
        {
            result.Add(dep);
            foreach (var transitive in GetAllDependencies(graph, dep))
            {
                result.Add(transitive);
            }
        }

        return result;
    }
}
