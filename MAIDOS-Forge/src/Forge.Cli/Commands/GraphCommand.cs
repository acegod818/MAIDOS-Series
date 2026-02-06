// MAIDOS-Forge CLI - Graph Command
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Core.Build;
using Forge.Core.Config;

namespace Forge.Cli.Commands;

/// <summary>
/// forge graph - 顯示依賴圖
/// </summary>
/// <impl>
/// APPROACH: 解析配置，分析依賴，以 ASCII 或 DOT 格式輸出
/// CALLS: ConfigParser.ParseProject(), DependencyAnalyzer.Analyze()
/// EDGES: --dot 參數輸出 Graphviz DOT 格式, 否則 ASCII
/// </impl>
public sealed class GraphCommand : ICommand
{
    private readonly CommandContext _context;

    public string Name => "graph";
    public string Description => "Display dependency graph";

    public GraphCommand(CommandContext context)
    {
        _context = context;
    }

    /// <summary>
    /// 執行 graph 命令
    /// </summary>
    /// <impl>
    /// APPROACH: 解析配置，分析依賴，根據參數選擇輸出格式
    /// CALLS: ConfigParser.ParseProject(), DependencyAnalyzer.Analyze(), OutputAscii(), OutputDot()
    /// EDGES: --dot 輸出 DOT 格式, 否則 ASCII, 無模組時提示
    /// </impl>
    public CommandResult Execute(string[] args)
    {
        var dotFormat = args.Contains("--dot");

        // 解析配置
        var parseResult = ConfigParser.ParseProject(_context.ProjectPath);
        if (!parseResult.IsSuccess)
        {
            _context.WriteError(parseResult.Error);
            return CommandResult.ConfigSyntaxError(parseResult.Error);
        }

        var config = parseResult.Value!;

        // 分析依賴
        var analysisResult = DependencyAnalyzer.Analyze(config);
        if (!string.IsNullOrEmpty(analysisResult.Error) && !analysisResult.HasCycle)
        {
            _context.WriteError(analysisResult.Error);
            return CommandResult.Error(1, analysisResult.Error);
        }

        // 輸出圖形
        if (dotFormat)
        {
            OutputDot(analysisResult, config);
        }
        else
        {
            OutputAscii(analysisResult, config);
        }

        return CommandResult.Ok();
    }

    /// <summary>
    /// 輸出 ASCII 格式依賴圖
    /// </summary>
    /// <impl>
    /// APPROACH: 遍歷所有模組，顯示名稱和依賴關係
    /// CALLS: TextWriter.WriteLine()
    /// EDGES: 無模組時顯示提示, 有循環時標記警告
    /// </impl>
    private void OutputAscii(DependencyAnalysisResult analysis, ValidatedForgeConfig config)
    {
        _context.WriteLine($"Dependency Graph: {config.Config.Name}");
        _context.WriteLine(new string('=', 50));
        _context.WriteLine("");

        if (config.Modules.Count == 0)
        {
            _context.WriteLine("(no modules)");
            return;
        }

        if (analysis.HasCycle)
        {
            _context.WriteLine("⚠ CIRCULAR DEPENDENCY DETECTED:");
            _context.WriteLine($"  {string.Join(" → ", analysis.CycleChain)}");
            _context.WriteLine("");
        }

        foreach (var module in config.Modules)
        {
            var deps = module.Config.Dependencies;
            _context.WriteLine($"[{module.Config.Language}] {module.Config.Name}");
            
            if (deps.Count > 0)
            {
                foreach (var dep in deps)
                {
                    _context.WriteLine($"  └── {dep}");
                }
            }
            else
            {
                _context.WriteLine("  └── (no dependencies)");
            }
            _context.WriteLine("");
        }
    }

    /// <summary>
    /// 輸出 Graphviz DOT 格式
    /// </summary>
    /// <impl>
    /// APPROACH: 生成 DOT 語法，節點依語言著色
    /// CALLS: TextWriter.WriteLine()
    /// EDGES: 無模組生成空圖, 節點按語言設定顏色
    /// </impl>
    private void OutputDot(DependencyAnalysisResult analysis, ValidatedForgeConfig config)
    {
        _context.WriteLine("digraph dependencies {");
        _context.WriteLine("  rankdir=TB;");
        _context.WriteLine("  node [shape=box, style=filled];");
        _context.WriteLine("");

        // 語言對應顏色
        var colors = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
        {
            ["csharp"] = "#68217A",  // 紫色
            ["rust"] = "#DEA584",    // 橙色
            ["c"] = "#555555",       // 灰色
            ["asm"] = "#0000AA"      // 藍色
        };

        // 輸出節點
        foreach (var module in config.Modules)
        {
            var color = colors.TryGetValue(module.Config.Language, out var c) ? c : "#CCCCCC";
            _context.WriteLine($"  \"{module.Config.Name}\" [fillcolor=\"{color}\", fontcolor=white];");
        }

        _context.WriteLine("");

        // 輸出邊
        foreach (var module in config.Modules)
        {
            foreach (var dep in module.Config.Dependencies)
            {
                _context.WriteLine($"  \"{module.Config.Name}\" -> \"{dep}\";");
            }
        }

        _context.WriteLine("}");
    }
}
