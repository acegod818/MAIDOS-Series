// MAIDOS-Forge CLI - Check Command
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Core.Build;
using Forge.Core.Config;

namespace Forge.Cli.Commands;

/// <summary>
/// forge check - 驗證配置與依賴
/// </summary>
/// <remarks>
/// <para>
/// 此命令用於驗證專案的配置文件和模組依賴關係。
/// 它會檢查 forge.json 和 module.json 文件的語法正確性，
/// 分析模組間的依賴關係，並生成構建排程以確保一切正常。
/// </para>
/// <para>
/// APPROACH: 解析配置，分析依賴，報告問題
/// CALLS: ConfigParser.ParseProject(), DependencyAnalyzer.Analyze()
/// EDGES: 配置錯誤時顯示具體問題, 循環依賴時顯示循環路徑
/// </para>
/// </remarks>
/// <example>
/// <code>
/// forge check
/// forge check --deps
/// </code>
/// </example>
public sealed class CheckCommand : ICommand
{
    private readonly CommandContext _context;

    public string Name => "check";
    public string Description => "Validate configuration and dependencies";

    public CheckCommand(CommandContext context)
    {
        _context = context;
    }

    /// <summary>
    /// 執行 check 命令
    /// </summary>
    /// <impl>
    /// APPROACH: 依序執行配置解析、依賴分析、排程生成，報告結果
    /// CALLS: ConfigParser.ParseProject(), DependencyAnalyzer.Analyze(), BuildScheduler.CreateSchedule()
    /// EDGES: 任一階段失敗時提前返回錯誤, --deps 參數顯示依賴圖
    /// </impl>
    public CommandResult Execute(string[] args)
    {
        var showDeps = args.Contains("--deps");

        _context.WriteLine($"Checking project at: {_context.ProjectPath}");
        _context.WriteLine("");

        // 階段 1: 解析配置
        _context.WriteLine("[1/3] Parsing configuration...");
        var parseResult = ConfigParser.ParseProject(_context.ProjectPath);
        if (!parseResult.IsSuccess)
        {
            _context.WriteError(parseResult.Error);
            return CommandResult.ConfigSyntaxError(parseResult.Error);
        }

        var config = parseResult.Value!;
        _context.WriteLine($"      Project: {config.Config.Name}");
        _context.WriteLine($"      Modules: {config.Modules.Count}");
        _context.WriteLine("      ✓ Configuration valid");
        _context.WriteLine("");

        // 階段 2: 分析依賴
        _context.WriteLine("[2/3] Analyzing dependencies...");
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

        _context.WriteLine("      ✓ No circular dependencies");
        _context.WriteLine("");

        // 階段 3: 生成排程
        _context.WriteLine("[3/3] Generating build schedule...");
        var scheduleResult = BuildScheduler.CreateSchedule(analysisResult);
        if (!scheduleResult.IsSuccess)
        {
            _context.WriteError(scheduleResult.Error);
            return CommandResult.Error(1, scheduleResult.Error);
        }

        var schedule = scheduleResult.Schedule!;
        _context.WriteLine($"      Layers: {schedule.Layers.Count}");
        _context.WriteLine($"      Max parallelism: {schedule.MaxParallelism}");
        _context.WriteLine("      ✓ Schedule generated");
        _context.WriteLine("");

        // 顯示依賴圖（如果請求）
        if (showDeps && schedule.Layers.Count > 0)
        {
            _context.WriteLine("Dependency Graph:");
            _context.WriteLine("");
            
            foreach (var layer in schedule.Layers)
            {
                _context.WriteLine($"  Layer {layer.Level}:");
                foreach (var module in layer.Modules)
                {
                    var deps = module.Config.Dependencies;
                    var depStr = deps.Count > 0 
                        ? $" → depends on: {string.Join(", ", deps)}" 
                        : " (no dependencies)";
                    _context.WriteLine($"    [{module.Config.Language}] {module.Config.Name}{depStr}");
                }
            }
            _context.WriteLine("");
        }

        _context.WriteLine("✓ All checks passed");
        return CommandResult.Ok();
    }
}