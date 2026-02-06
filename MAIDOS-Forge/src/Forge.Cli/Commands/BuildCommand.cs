// MAIDOS-Forge CLI - Build Command
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Core.Build;
using Forge.Core.Config;
using Forge.Core.Plugin;

namespace Forge.Cli.Commands;

/// <summary>
/// forge build - 編譯專案
/// </summary>
/// <remarks>
/// <para>
/// 此命令用於編譯整個 MAIDOS-Forge 專案或指定的模組。
/// 它會解析配置、分析依賴關係、生成構建排程，然後使用插件系統執行實際的編譯工作。
/// </para>
/// <para>
/// APPROACH: 解析配置、分析依賴、排程、使用插件系統執行編譯
/// CALLS: ConfigParser, DependencyAnalyzer, BuildScheduler, PluginHost, ILanguagePlugin
/// EDGES: 插件不存在時報錯, 編譯失敗顯示日誌
/// </para>
/// </remarks>
/// <example>
/// <code>
/// forge build
/// forge build --debug
/// forge build core-module
/// </code>
/// </example>
public sealed class BuildCommand : ICommand
{
    private readonly CommandContext _context;
    private readonly PluginHost _pluginHost;
    private readonly PluginManager _pluginManager;

    public string Name => "build";
    public string Description => "Build the project";

    public BuildCommand(CommandContext context)
    {
        _context = context;
        _pluginHost = new PluginHost();
        _pluginHost.RegisterBuiltinPlugins();
        
        // 初始化插件管理器並加載外部插件
        _pluginManager = new PluginManager();
        var initResult = _pluginManager.InitializeAsync(loadBuiltins: false).GetAwaiter().GetResult();
        
        _context.WriteVerbose($"Plugin initialization result: {initResult.SuccessCount} succeeded, {initResult.FailureCount} failed");
        
        // 顯示初始化結果詳情
        foreach (var result in initResult.Results)
        {
            _context.WriteVerbose($"Plugin {result.Name}: {(result.Success ? "SUCCESS" : "FAILED")} - {result.Message}");
        }
        
        // 顯示警告
        foreach (var warning in initResult.Warnings)
        {
            _context.WriteVerbose($"Warning: {warning}");
        }
        
        // 將外部插件註冊到 PluginHost
        foreach (var plugin in _pluginManager.ListInstalledPlugins())
        {
            _context.WriteVerbose($"Registering plugin: {plugin.Metadata.Name} ({plugin.Metadata.Language})");
            // 注意：這裡需要根據插件類型創建對應的插件實例
            // 由於這是外部插件，我們需要從 PluginLoader 獲取實例
            var loaderPlugin = _pluginManager.Loader.GetPlugin(plugin.Metadata.Language);
            if (loaderPlugin != null)
            {
                _pluginHost.Register(loaderPlugin);
                _context.WriteVerbose($"Successfully registered plugin: {plugin.Metadata.Name}");
            }
            else
            {
                _context.WriteVerbose($"Failed to get plugin instance for: {plugin.Metadata.Name}");
            }
        }
        
        // 手動加載 Java 插件作為備用方案
        var javaPluginDir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), 
            ".forge", "plugins", "Forge.Plugin.Java");
        if (Directory.Exists(javaPluginDir))
        {
            var javaPlugin = _pluginManager.Loader.LoadPlugin(javaPluginDir);
            if (javaPlugin.IsLoaded)
            {
                _pluginHost.Register(javaPlugin.Instance!);
                _context.WriteVerbose($"Manually registered Java plugin");
            }
            else
            {
                _context.WriteVerbose($"Failed to manually load Java plugin: {javaPlugin.LoadError}");
                
                // 嘗試直接創建 JavaPlugin 實例
                try
                {
                    // 使用反射創建 JavaPlugin 實例
                    var assembly = System.Reflection.Assembly.LoadFrom(
                        Path.Combine(javaPluginDir, "bin", "Debug", "net8.0", "Forge.Plugin.Java.dll"));
                    var pluginType = assembly.GetType("Forge.Plugin.Java.JavaPlugin");
                    if (pluginType != null)
                    {
                        var javaPluginInstance = System.Activator.CreateInstance(pluginType) as ILanguagePlugin;
                        if (javaPluginInstance != null)
                        {
                            var capabilities = javaPluginInstance.GetCapabilities();
                            _pluginHost.Register(javaPluginInstance);
                            _context.WriteVerbose($"Directly registered Java plugin with capabilities: {capabilities.LanguageName}");
                        }
                    }
                }
                catch (Exception ex)
                {
                    _context.WriteVerbose($"Failed to directly register Java plugin: {ex.Message}");
                }
            }
        }
        
        _context.WriteVerbose($"Registered languages: {string.Join(", ", _pluginHost.RegisteredLanguages)}");
    }

    /// <summary>
    /// 執行 build 命令
    /// </summary>
    /// <impl>
    /// APPROACH: 解析配置、排程、按層級並行編譯
    /// CALLS: ConfigParser, DependencyAnalyzer, BuildScheduler, ExecuteBuildAsync()
    /// EDGES: --debug 使用 debug profile, --dry-run 只顯示計畫, 編譯失敗提前返回
    /// </impl>
    public CommandResult Execute(string[] args)
    {
        var isDebug = args.Contains("--debug");
        var isDryRun = args.Contains("--dry-run");
        var targetModule = args.FirstOrDefault(a => !a.StartsWith('-'));
        var targetPlatform = args.FirstOrDefault(a => a.StartsWith("--target="))?.Split('=', 2)[1] ?? "native";
        var profile = isDebug ? "debug" : "release";

        _context.WriteLine($"Building project at: {_context.ProjectPath}");
        _context.WriteLine($"Profile: {profile}");
        if (targetPlatform != "native")
            _context.WriteLine($"Target: {targetPlatform}");
        _context.WriteLine("");

        // 階段 1: 解析配置
        _context.WriteVerbose("Parsing configuration...");
        var parseResult = ConfigParser.ParseProject(_context.ProjectPath);
        if (!parseResult.IsSuccess)
        {
            _context.WriteError(parseResult.Error);
            return CommandResult.ConfigSyntaxError(parseResult.Error);
        }

        var config = parseResult.Value!;

        if (config.Modules.Count == 0)
        {
            _context.WriteLine("No modules to build.");
            _context.WriteLine("Add modules to the 'modules/' directory.");
            return CommandResult.Ok();
        }

        // 階段 2: 分析依賴
        _context.WriteVerbose("Analyzing dependencies...");
        var analysisResult = DependencyAnalyzer.Analyze(config);

        if (!string.IsNullOrEmpty(analysisResult.Error) && !analysisResult.HasCycle)
        {
            _context.WriteError(analysisResult.Error);
            return CommandResult.DependencyNotFound(analysisResult.Error);
        }

        if (analysisResult.HasCycle)
        {
            _context.WriteError(analysisResult.Error);
            return CommandResult.CircularDependency(string.Join(" → ", analysisResult.CycleChain));
        }

        // 階段 3: 生成排程
        _context.WriteVerbose("Generating build schedule...");
        var scheduleResult = BuildScheduler.CreateSchedule(analysisResult);
        if (!scheduleResult.IsSuccess)
        {
            _context.WriteError(scheduleResult.Error);
            return CommandResult.Error(1, scheduleResult.Error);
        }

        var schedule = scheduleResult.Schedule!;

        // 過濾要編譯的模組
        var modulesToBuild = GetModulesToBuild(schedule, targetModule, config);
        if (modulesToBuild.Count == 0)
        {
            if (!string.IsNullOrEmpty(targetModule))
            {
                _context.WriteError($"Module not found: {targetModule}");
                return CommandResult.ModuleNotFound(targetModule);
            }
            _context.WriteLine("No modules to build.");
            return CommandResult.Ok();
        }

        // 顯示編譯計畫
        _context.WriteLine("Build Plan:");
        _context.WriteLine("");
        foreach (var layer in schedule.Layers)
        {
            var layerModules = modulesToBuild.Where(m => 
                layer.Modules.Any(lm => lm.Config.Name == m.Config.Name)).ToList();
            if (layerModules.Count == 0) continue;

            _context.WriteLine($"  Layer {layer.Level} (parallel):");
            foreach (var module in layerModules)
            {
                _context.WriteLine($"    [{module.Config.Language}] {module.Config.Name}");
            }
        }
        _context.WriteLine("");
        _context.WriteLine($"Total: {modulesToBuild.Count} module(s)");
        _context.WriteLine("");

        // Dry run 模式
        if (isDryRun)
        {
            _context.WriteLine("Dry run - no actual compilation");
            return CommandResult.Ok();
        }

        // 階段 4: 執行編譯
        var buildResult = ExecuteBuildAsync(schedule, modulesToBuild, config, profile, targetPlatform).GetAwaiter().GetResult();
        return buildResult;
    }

    /// <summary>
    /// 執行實際編譯
    /// </summary>
    /// <impl>
    /// APPROACH: 按層級順序編譯，同層級並行
    /// CALLS: ILanguagePlugin.CompileAsync()
    /// EDGES: 任一模組編譯失敗停止並返回錯誤
    /// </impl>
    private async Task<CommandResult> ExecuteBuildAsync(
        BuildSchedule schedule,
        List<ValidatedModuleConfig> modulesToBuild,
        ValidatedForgeConfig projectConfig,
        string profile,
        string targetPlatform)
    {
        var outputDir = Path.Combine(projectConfig.ProjectRoot, 
            projectConfig.Config.Output.Dir, profile);
        Directory.CreateDirectory(outputDir);

        var compileConfig = new CompileConfig
        {
            Profile = profile,
            OutputDir = outputDir,
            TargetPlatform = targetPlatform,
            Verbose = _context.Verbose
        };

        var compiled = 0;
        var failed = 0;

        _context.WriteLine("Compiling...");
        _context.WriteLine("");

        foreach (var layer in schedule.Layers)
        {
            var layerModules = modulesToBuild
                .Where(m => layer.Modules.Any(lm => lm.Config.Name == m.Config.Name))
                .ToList();

            if (layerModules.Count == 0) continue;

            // 並行編譯同層級模組
            var tasks = layerModules.Select(async module =>
            {
                var plugin = _pluginHost.GetPlugin(module.Config.Language);
                if (plugin is null)
                {
                    return (module, CompileResult.Failure(
                        $"No plugin for language: {module.Config.Language}",
                        Array.Empty<string>(), TimeSpan.Zero));
                }

                _context.WriteLine($"  [{module.Config.Language}] {module.Config.Name}...");
                var result = await plugin.CompileAsync(module, compileConfig);
                return (module, result);
            });

            var results = await Task.WhenAll(tasks);

            foreach (var (module, result) in results)
            {
                if (result.IsSuccess)
                {
                    compiled++;
                    _context.WriteLine($"    ✓ {module.Config.Name} ({result.Duration.TotalSeconds:F1}s)");
                    
                    if (_context.Verbose)
                    {
                        foreach (var artifact in result.Artifacts)
                        {
                            _context.WriteLine($"      → {Path.GetFileName(artifact)}");
                        }
                    }
                }
                else
                {
                    failed++;
                    _context.WriteLine($"    ✗ {module.Config.Name} FAILED");
                    _context.WriteError(result.Error);
                    
                    if (_context.Verbose)
                    {
                        foreach (var log in result.Logs)
                        {
                            _context.WriteLine($"      {log}");
                        }
                    }

                    // 停止編譯
                    _context.WriteLine("");
                    _context.WriteLine($"Build failed: {compiled} succeeded, {failed} failed");
                    return CommandResult.Error(302, $"Compilation failed: {module.Config.Name}");
                }
            }
        }

        _context.WriteLine("");
        _context.WriteLine($"Build succeeded: {compiled} module(s) compiled");
        _context.WriteLine($"Output: {outputDir}");
        return CommandResult.Ok();
    }

    /// <summary>
    /// 取得需要編譯的模組列表
    /// </summary>
    /// <impl>
    /// APPROACH: 如果指定目標模組，包含其所有傳遞依賴
    /// CALLS: IsTransitiveDependency()
    /// EDGES: 無目標模組返回全部, 目標不存在返回空列表
    /// </impl>
    private List<ValidatedModuleConfig> GetModulesToBuild(
        BuildSchedule schedule,
        string? targetModule,
        ValidatedForgeConfig config)
    {
        var allModules = schedule.GetFlattenedOrder().ToList();

        if (string.IsNullOrEmpty(targetModule))
        {
            return allModules;
        }

        // 檢查目標模組是否存在
        var target = allModules.FirstOrDefault(m =>
            string.Equals(m.Config.Name, targetModule, StringComparison.OrdinalIgnoreCase));

        if (target is null)
        {
            return new List<ValidatedModuleConfig>();
        }

        // 包含目標及其所有依賴
        return allModules.Where(m =>
            string.Equals(m.Config.Name, targetModule, StringComparison.OrdinalIgnoreCase) ||
            IsTransitiveDependency(targetModule, m.Config.Name, config)).ToList();
    }

    /// <summary>
    /// 檢查 target 是否傳遞依賴於 module
    /// </summary>
    /// <impl>
    /// APPROACH: 遞迴檢查依賴鏈
    /// CALLS: ValidatedForgeConfig.Modules
    /// EDGES: target 不存在返回 false, 無依賴返回 false
    /// </impl>
    private static bool IsTransitiveDependency(string target, string module, ValidatedForgeConfig config)
    {
        var targetModule = config.Modules.FirstOrDefault(m => 
            string.Equals(m.Config.Name, target, StringComparison.OrdinalIgnoreCase));

        if (targetModule is null) return false;

        var visited = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        return CheckDependency(targetModule.Config.Name, module, config, visited);
    }

    /// <summary>
    /// 遞迴檢查依賴
    /// </summary>
    /// <impl>
    /// APPROACH: DFS 遍歷依賴鏈
    /// CALLS: 自身遞迴
    /// EDGES: 找到 module 返回 true, 已訪問過返回 false 避免循環
    /// </impl>
    private static bool CheckDependency(string current, string target, 
        ValidatedForgeConfig config, HashSet<string> visited)
    {
        if (visited.Contains(current)) return false;
        visited.Add(current);

        var currentModule = config.Modules.FirstOrDefault(m =>
            string.Equals(m.Config.Name, current, StringComparison.OrdinalIgnoreCase));

        if (currentModule is null) return false;

        foreach (var dep in currentModule.Config.Dependencies)
        {
            if (string.Equals(dep, target, StringComparison.OrdinalIgnoreCase))
                return true;

            if (CheckDependency(dep, target, config, visited))
                return true;
        }

        return false;
    }
}