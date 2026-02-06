// MAIDOS-Forge CLI - Watch Command
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Core.Config;
using Forge.Core.Linker;
using Forge.Core.Orchestration;

namespace Forge.Cli.Commands;

/// <summary>
/// forge watch - 監視模式，檔案變更時自動重編
/// </summary>
/// <impl>
/// APPROACH: 使用 FileSystemWatcher 監視源碼變更，觸發重編
/// CALLS: FileSystemWatcher, BuildOrchestrator
/// EDGES: Ctrl+C 終止監視
/// </impl>
public sealed class WatchCommand : ICommand
{
    private readonly CommandContext _context;
    private volatile bool _isBuilding = false;
    private volatile bool _needsRebuild = false;
    private DateTime _lastBuildTime = DateTime.MinValue;
    private const int DebounceMs = 500;

    public string Name => "watch";
    public string Description => "Watch for changes and rebuild automatically";

    public WatchCommand(CommandContext context)
    {
        _context = context;
    }

    /// <summary>
    /// 執行 watch 命令
    /// </summary>
    /// <impl>
    /// APPROACH: 初始編譯，然後監視檔案變更
    /// CALLS: BuildOrchestrator.BuildAsync(), FileSystemWatcher
    /// EDGES: Ctrl+C 終止
    /// </impl>
    public CommandResult Execute(string[] args)
    {
        // 解析參數
        var profile = "debug";
        var runAfterBuild = false;
        var clearScreen = true;

        for (int i = 0; i < args.Length; i++)
        {
            switch (args[i])
            {
                case "--help" or "-h":
                    ShowHelp();
                    return CommandResult.Ok();

                case "--debug":
                    profile = "debug";
                    break;

                case "--release":
                    profile = "release";
                    break;

                case "--run" or "-r":
                    runAfterBuild = true;
                    break;

                case "--no-clear":
                    clearScreen = false;
                    break;
            }
        }

        // 解析專案配置
        var parseResult = ConfigParser.ParseProject(_context.ProjectPath);
        if (!parseResult.IsSuccess)
        {
            _context.WriteError(parseResult.Error);
            return CommandResult.ConfigSyntaxError(parseResult.Error);
        }

        var config = parseResult.Value!;

        _context.WriteLine($"Watching project: {config.Config.Name}");
        _context.WriteLine($"Profile: {profile}");
        _context.WriteLine($"Run after build: {runAfterBuild}");
        _context.WriteLine("Press Ctrl+C to stop");
        _context.WriteLine("");

        // 設定取消處理
        var cts = new CancellationTokenSource();
        Console.CancelKeyPress += (s, e) =>
        {
            e.Cancel = true;
            cts.Cancel();
            _context.WriteLine("");
            _context.WriteLine("Stopping watch...");
        };

        // 初始編譯
        DoBuild(config, profile, runAfterBuild, clearScreen);

        // 設定監視器
        var watchers = new List<FileSystemWatcher>();

        foreach (var module in config.Modules)
        {
            var srcDir = Path.Combine(module.ModulePath, "src");
            if (!Directory.Exists(srcDir))
            {
                srcDir = module.ModulePath;
            }

            var watcher = new FileSystemWatcher(srcDir)
            {
                IncludeSubdirectories = true,
                NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.FileName | NotifyFilters.Size,
                EnableRaisingEvents = true
            };

            var extensions = GetExtensions(module.Config.Language);
            foreach (var ext in extensions)
            {
                watcher.Filters.Add($"*{ext}");
            }

            watcher.Changed += (s, e) => OnFileChanged(e, config, profile, runAfterBuild, clearScreen);
            watcher.Created += (s, e) => OnFileChanged(e, config, profile, runAfterBuild, clearScreen);
            watcher.Deleted += (s, e) => OnFileChanged(e, config, profile, runAfterBuild, clearScreen);
            watcher.Renamed += (s, e) => OnFileChanged(e, config, profile, runAfterBuild, clearScreen);

            watchers.Add(watcher);
        }

        // 監視配置檔案
        var configWatcher = new FileSystemWatcher(config.ProjectRoot, "*.json")
        {
            NotifyFilter = NotifyFilters.LastWrite,
            EnableRaisingEvents = true
        };
        configWatcher.Changed += (s, e) => OnFileChanged(e, config, profile, runAfterBuild, clearScreen);
        watchers.Add(configWatcher);

        // 等待取消
        try
        {
            cts.Token.WaitHandle.WaitOne();
        }
        catch (OperationCanceledException)
        {
            // 正常取消
        }

        // 清理
        foreach (var watcher in watchers)
        {
            watcher.EnableRaisingEvents = false;
            watcher.Dispose();
        }

        _context.WriteLine("Watch stopped.");
        return CommandResult.Ok();
    }

    /// <summary>
    /// 檔案變更處理
    /// </summary>
    /// <impl>
    /// APPROACH: 防抖動處理，觸發重編
    /// CALLS: DoBuild()
    /// EDGES: 編譯中則標記需要重編
    /// </impl>
    private void OnFileChanged(
        FileSystemEventArgs e,
        ValidatedForgeConfig config,
        string profile,
        bool runAfterBuild,
        bool clearScreen)
    {
        // 忽略臨時檔案
        if (e.Name?.StartsWith(".") == true ||
            e.Name?.Contains("~") == true ||
            e.Name?.EndsWith(".tmp") == true)
        {
            return;
        }

        // 防抖動
        var now = DateTime.UtcNow;
        if ((now - _lastBuildTime).TotalMilliseconds < DebounceMs)
        {
            _needsRebuild = true;
            return;
        }

        // 如果正在編譯，標記需要重編
        if (_isBuilding)
        {
            _needsRebuild = true;
            return;
        }

        _context.WriteLine("");
        _context.WriteLine($"[{DateTime.Now:HH:mm:ss}] Change detected: {e.Name}");

        // 延遲一下等待所有變更完成
        Thread.Sleep(100);

        DoBuild(config, profile, runAfterBuild, clearScreen);

        // 檢查是否需要再次編譯
        if (_needsRebuild)
        {
            _needsRebuild = false;
            DoBuild(config, profile, runAfterBuild, clearScreen);
        }
    }

    /// <summary>
    /// 執行編譯
    /// </summary>
    /// <impl>
    /// APPROACH: 調用 BuildOrchestrator，可選執行
    /// CALLS: BuildOrchestrator.BuildAsync()
    /// EDGES: N/A
    /// </impl>
    private void DoBuild(
        ValidatedForgeConfig config,
        string profile,
        bool runAfterBuild,
        bool clearScreen)
    {
        _isBuilding = true;
        _lastBuildTime = DateTime.UtcNow;

        try
        {
            if (clearScreen)
            {
                Console.Clear();
            }

            _context.WriteLine($"[{DateTime.Now:HH:mm:ss}] Building...");
            _context.WriteLine("");

            var buildOptions = new BuildOptions
            {
                Profile = profile,
                OutputType = OutputType.Executable,
                Verbose = _context.Verbose
            };

            var orchestrator = new BuildOrchestrator();
            var buildResult = orchestrator.BuildAsync(config.ProjectRoot, buildOptions)
                .GetAwaiter().GetResult();

            if (!buildResult.IsSuccess)
            {
                _context.WriteError($"Build failed: {buildResult.Error}");
                _context.WriteLine("");
                _context.WriteLine("Waiting for changes...");
                return;
            }

            _context.WriteLine($"✓ Build succeeded ({buildResult.TotalDuration.TotalSeconds:F1}s)");

            // 執行
            if (runAfterBuild && !string.IsNullOrEmpty(buildResult.OutputPath))
            {
                _context.WriteLine("");
                _context.WriteLine("Running...");
                _context.WriteLine("---");

                RunApp(buildResult.OutputPath, config.ProjectRoot);
            }

            _context.WriteLine("");
            _context.WriteLine("Waiting for changes...");
        }
        finally
        {
            _isBuilding = false;
        }
    }

    /// <summary>
    /// 執行應用程式
    /// </summary>
    /// <impl>
    /// APPROACH: 根據檔案類型執行
    /// CALLS: Process.Start()
    /// EDGES: DLL 用 dotnet 執行
    /// </impl>
    private void RunApp(string outputPath, string workingDir)
    {
        try
        {
            string command;
            string arguments;

            if (outputPath.EndsWith(".dll", StringComparison.OrdinalIgnoreCase))
            {
                command = "dotnet";
                arguments = $"\"{outputPath}\"";
            }
            else
            {
                command = outputPath;
                arguments = string.Empty;
            }

            var startInfo = new System.Diagnostics.ProcessStartInfo
            {
                FileName = command,
                Arguments = arguments,
                WorkingDirectory = workingDir,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true
            };

            using var process = System.Diagnostics.Process.Start(startInfo);
            if (process is null)
            {
                _context.WriteError("Failed to start process");
                return;
            }

            // 非阻塞讀取輸出
            process.OutputDataReceived += (s, e) =>
            {
                if (e.Data is not null) Console.WriteLine(e.Data);
            };
            process.ErrorDataReceived += (s, e) =>
            {
                if (e.Data is not null) Console.Error.WriteLine(e.Data);
            };

            process.BeginOutputReadLine();
            process.BeginErrorReadLine();

            // 等待最多 5 秒
            if (!process.WaitForExit(5000))
            {
                _context.WriteLine("(Process still running, continuing watch...)");
            }
            else
            {
                _context.WriteLine($"Exit code: {process.ExitCode}");
            }
        }
        catch (Exception ex)
        {
            _context.WriteError($"Run failed: {ex.Message}");
        }
    }

    private static string[] GetExtensions(string language) => language.ToLowerInvariant() switch
    {
        "csharp" or "c#" => new[] { ".cs" },
        "rust" => new[] { ".rs" },
        "c" => new[] { ".c", ".h" },
        _ => new[] { ".*" }
    };

    private void ShowHelp()
    {
        _context.WriteLine("forge watch - Watch for changes and rebuild automatically");
        _context.WriteLine("");
        _context.WriteLine("USAGE:");
        _context.WriteLine("    forge watch [options]");
        _context.WriteLine("");
        _context.WriteLine("OPTIONS:");
        _context.WriteLine("    --debug              Use debug configuration (default)");
        _context.WriteLine("    --release            Use release configuration");
        _context.WriteLine("    --run, -r            Run after successful build");
        _context.WriteLine("    --no-clear           Don't clear screen before build");
        _context.WriteLine("    -v, --verbose        Verbose output");
        _context.WriteLine("    -h, --help           Show this help");
        _context.WriteLine("");
        _context.WriteLine("EXAMPLES:");
        _context.WriteLine("    forge watch");
        _context.WriteLine("    forge watch --run");
        _context.WriteLine("    forge watch --release --no-clear");
    }
}
