// MAIDOS-Forge CLI - Toolchain Command
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Core.Plugin;

namespace Forge.Cli.Commands;

/// <summary>
/// forge toolchain - 驗證已安裝的工具鏈
/// </summary>
/// <impl>
/// APPROACH: 遍歷所有已註冊插件，驗證其工具鏈可用性
/// CALLS: PluginHost, ILanguagePlugin.ValidateToolchainAsync()
/// EDGES: 無可用插件時顯示警告
/// </impl>
public sealed class ToolchainCommand : ICommand
{
    private readonly CommandContext _context;
    private readonly PluginHost _pluginHost;

    public string Name => "toolchain";
    public string Description => "Check available toolchains";

    public ToolchainCommand(CommandContext context)
    {
        _context = context;
        _pluginHost = new PluginHost();
        _pluginHost.RegisterBuiltinPlugins();
    }

    /// <summary>
    /// 執行 toolchain 命令
    /// </summary>
    /// <impl>
    /// APPROACH: 並行驗證所有插件工具鏈，顯示結果
    /// CALLS: PluginHost.ValidateAllToolchainsAsync()
    /// EDGES: 全部失敗時返回警告碼
    /// </impl>
    public CommandResult Execute(string[] args)
    {
        _context.WriteLine("Checking available toolchains...");
        _context.WriteLine("");

        var results = _pluginHost.ValidateAllToolchainsAsync().GetAwaiter().GetResult();

        var available = 0;
        var unavailable = 0;

        foreach (var (language, result) in results)
        {
            var plugin = _pluginHost.GetPlugin(language);
            var caps = plugin?.GetCapabilities();

            if (result.Available)
            {
                available++;
                _context.WriteLine($"  ✓ {language}");
                _context.WriteLine($"      {result.Message}");
                
                if (caps is not null && _context.Verbose)
                {
                    _context.WriteLine($"      Extensions: {string.Join(", ", caps.SupportedExtensions)}");
                    _context.WriteLine($"      Native: {caps.SupportsNativeCompilation}");
                    _context.WriteLine($"      Cross: {caps.SupportsCrossCompilation}");
                }
            }
            else
            {
                unavailable++;
                _context.WriteLine($"  ✗ {language}");
                _context.WriteLine($"      {result.Message}");
            }

            _context.WriteLine("");
        }

        _context.WriteLine($"Available: {available}, Unavailable: {unavailable}");

        if (available == 0)
        {
            _context.WriteError("No toolchains available. Install at least one compiler.");
            return CommandResult.Error(401, "No toolchains available");
        }

        return CommandResult.Ok();
    }
}
