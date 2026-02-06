// MAIDOS-Forge CLI - Run Command
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Core.Config;
using Forge.Core.Linker;
using Forge.Core.Orchestration;
using Forge.Core.Platform;

namespace Forge.Cli.Commands;

/// <summary>
/// forge run - 編譯並執行專案
/// </summary>
/// <impl>
/// APPROACH: 先調用 BuildOrchestrator 編譯，然後執行產物
/// CALLS: BuildOrchestrator, ProcessRunner
/// EDGES: 編譯失敗時不執行, 執行失敗返回錯誤碼
/// </impl>
public sealed class RunCommand : ICommand
{
    private readonly CommandContext _context;

    public string Name => "run";
    public string Description => "Build and run the project";

    public RunCommand(CommandContext context)
    {
        _context = context;
    }

    /// <summary>
    /// 執行 run 命令
    /// </summary>
    /// <impl>
    /// APPROACH: 解析參數，編譯專案，執行產物
    /// CALLS: BuildOrchestrator.BuildAsync(), ProcessRunner.RunAsync()
    /// EDGES: 顯示幫助時返回 OK
    /// </impl>
    public CommandResult Execute(string[] args)
    {
        // 解析參數
        var profile = "release";
        var appArgs = new List<string>();
        var skipBuild = false;
        var parseAppArgs = false;

        for (int i = 0; i < args.Length; i++)
        {
            if (parseAppArgs)
            {
                appArgs.Add(args[i]);
                continue;
            }

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

                case "--skip-build":
                    skipBuild = true;
                    break;

                case "--":
                    parseAppArgs = true;
                    break;

                default:
                    // 未知參數視為應用程式參數
                    appArgs.Add(args[i]);
                    parseAppArgs = true;
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

        // 確定輸出路徑
        var buildDir = Path.Combine(config.ProjectRoot, config.Config.Output.Dir, profile);
        var outputName = config.Config.Name;
        var platform = TargetPlatform.Current;
        var exePath = Path.Combine(buildDir, outputName + platform.GetExecutableExtension());

        // 編譯（除非跳過）
        if (!skipBuild)
        {
            _context.WriteLine($"Building {config.Config.Name}...");
            _context.WriteLine($"Profile: {profile}");
            _context.WriteLine("");

            var buildOptions = new BuildOptions
            {
                Profile = profile,
                OutputType = OutputType.Executable,
                OutputName = outputName,
                Verbose = _context.Verbose,
                ProgressCallback = (phase, msg, cur, total) =>
                {
                    if (_context.Verbose)
                    {
                        _context.WriteLine($"  [{phase}] {msg}");
                    }
                }
            };

            var orchestrator = new BuildOrchestrator();
            var buildResult = orchestrator.BuildAsync(_context.ProjectPath, buildOptions)
                .GetAwaiter().GetResult();

            if (!buildResult.IsSuccess)
            {
                _context.WriteError($"Build failed: {buildResult.Error}");
                return CommandResult.Error(500, buildResult.Error);
            }

            _context.WriteLine($"✓ Build succeeded ({buildResult.TotalDuration.TotalSeconds:F1}s)");
            _context.WriteLine("");

            // 更新執行路徑
            if (!string.IsNullOrEmpty(buildResult.OutputPath) && File.Exists(buildResult.OutputPath))
            {
                exePath = buildResult.OutputPath;
            }
        }

        // 檢查執行檔是否存在
        if (!File.Exists(exePath))
        {
            // 嘗試尋找 .dll (CLR 模式)
            var dllPath = Path.ChangeExtension(exePath, ".dll");
            if (File.Exists(dllPath))
            {
                return RunDotNetApp(dllPath, appArgs);
            }

            _context.WriteError($"Executable not found: {exePath}");
            _context.WriteError("Make sure the project builds to an executable.");
            return CommandResult.Error(302, "Executable not found");
        }

        // 執行
        return RunNativeApp(exePath, appArgs);
    }

    /// <summary>
    /// 執行原生應用程式
    /// </summary>
    /// <impl>
    /// APPROACH: 直接執行可執行檔
    /// CALLS: ProcessRunner.RunAsync()
    /// EDGES: 返回應用程式的退出碼
    /// </impl>
    private CommandResult RunNativeApp(string exePath, List<string> appArgs)
    {
        _context.WriteLine($"Running: {Path.GetFileName(exePath)}");
        if (appArgs.Count > 0)
        {
            _context.WriteLine($"Args: {string.Join(" ", appArgs)}");
        }
        _context.WriteLine("---");
        _context.WriteLine("");

        var argsStr = string.Join(" ", appArgs.Select(a => a.Contains(' ') ? $"\"{a}\"" : a));

        var result = ProcessRunner.RunAsync(
            exePath,
            argsStr,
            new ProcessConfig
            {
                WorkingDirectory = Path.GetDirectoryName(exePath) ?? ".",
                Timeout = TimeSpan.FromHours(24) // 長時間執行
            }).GetAwaiter().GetResult();

        // 輸出 stdout
        if (!string.IsNullOrEmpty(result.Stdout))
        {
            Console.Write(result.Stdout);
        }

        // 輸出 stderr
        if (!string.IsNullOrEmpty(result.Stderr))
        {
            Console.Error.Write(result.Stderr);
        }

        if (result.ExitCode != 0)
        {
            _context.WriteLine("");
            _context.WriteLine($"Process exited with code: {result.ExitCode}");
        }

        return result.ExitCode == 0 
            ? CommandResult.Ok() 
            : CommandResult.Error(result.ExitCode, $"Process exited with code {result.ExitCode}");
    }

    /// <summary>
    /// 執行 .NET 應用程式
    /// </summary>
    /// <impl>
    /// APPROACH: 使用 dotnet 執行 DLL
    /// CALLS: ProcessRunner.RunAsync()
    /// EDGES: 返回應用程式的退出碼
    /// </impl>
    private CommandResult RunDotNetApp(string dllPath, List<string> appArgs)
    {
        _context.WriteLine($"Running: dotnet {Path.GetFileName(dllPath)}");
        if (appArgs.Count > 0)
        {
            _context.WriteLine($"Args: {string.Join(" ", appArgs)}");
        }
        _context.WriteLine("---");
        _context.WriteLine("");

        var allArgs = new List<string> { $"\"{dllPath}\"" };
        allArgs.AddRange(appArgs.Select(a => a.Contains(' ') ? $"\"{a}\"" : a));
        var argsStr = string.Join(" ", allArgs);

        var result = ProcessRunner.RunAsync(
            "dotnet",
            argsStr,
            new ProcessConfig
            {
                WorkingDirectory = Path.GetDirectoryName(dllPath) ?? ".",
                Timeout = TimeSpan.FromHours(24)
            }).GetAwaiter().GetResult();

        if (!string.IsNullOrEmpty(result.Stdout))
        {
            Console.Write(result.Stdout);
        }

        if (!string.IsNullOrEmpty(result.Stderr))
        {
            Console.Error.Write(result.Stderr);
        }

        if (result.ExitCode != 0)
        {
            _context.WriteLine("");
            _context.WriteLine($"Process exited with code: {result.ExitCode}");
        }

        return result.ExitCode == 0 
            ? CommandResult.Ok() 
            : CommandResult.Error(result.ExitCode, $"Process exited with code {result.ExitCode}");
    }

    private void ShowHelp()
    {
        _context.WriteLine("forge run - Build and run the project");
        _context.WriteLine("");
        _context.WriteLine("USAGE:");
        _context.WriteLine("    forge run [options] [-- <app-args>]");
        _context.WriteLine("");
        _context.WriteLine("OPTIONS:");
        _context.WriteLine("    --debug              Build and run debug configuration");
        _context.WriteLine("    --release            Build and run release configuration (default)");
        _context.WriteLine("    --skip-build         Skip build, run existing executable");
        _context.WriteLine("    -v, --verbose        Verbose output");
        _context.WriteLine("    -h, --help           Show this help");
        _context.WriteLine("");
        _context.WriteLine("EXAMPLES:");
        _context.WriteLine("    forge run");
        _context.WriteLine("    forge run --debug");
        _context.WriteLine("    forge run -- arg1 arg2");
        _context.WriteLine("    forge run --skip-build -- --config test.json");
    }
}
