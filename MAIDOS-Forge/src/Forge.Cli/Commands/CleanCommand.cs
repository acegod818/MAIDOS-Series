// MAIDOS-Forge CLI - Clean Command
// UEP v1.7B Compliant - Zero Technical Debt

namespace Forge.Cli.Commands;

/// <summary>
/// forge clean - 清理編譯產物
/// </summary>
/// <impl>
/// APPROACH: 刪除 build/ 目錄，--all 時同時刪除 .forge/ 快取
/// CALLS: Directory.Delete()
/// EDGES: 目錄不存在時靜默跳過, 刪除失敗報錯
/// </impl>
public sealed class CleanCommand : ICommand
{
    private readonly CommandContext _context;

    public string Name => "clean";
    public string Description => "Clean build artifacts";

    public CleanCommand(CommandContext context)
    {
        _context = context;
    }

    /// <summary>
    /// 執行 clean 命令
    /// </summary>
    /// <impl>
    /// APPROACH: 根據參數刪除 build/ 和可選的 .forge/ 目錄
    /// CALLS: Directory.Delete(), Directory.Exists()
    /// EDGES: --all 包含快取, 目錄不存在跳過, 刪除失敗報錯
    /// </impl>
    public CommandResult Execute(string[] args)
    {
        var cleanAll = args.Contains("--all");

        var buildDir = Path.Combine(_context.ProjectPath, "build");
        var cacheDir = Path.Combine(_context.ProjectPath, ".forge");
        var generatedDir = Path.Combine(_context.ProjectPath, "generated");

        var cleaned = false;

        // 清理 build/
        if (Directory.Exists(buildDir))
        {
            try
            {
                Directory.Delete(buildDir, recursive: true);
                _context.WriteLine($"Removed: {buildDir}");
                cleaned = true;
            }
            catch (IOException ex)
            {
                _context.WriteError($"Failed to remove {buildDir}: {ex.Message}");
                return CommandResult.Error(1, $"Failed to clean build directory: {ex.Message}");
            }
        }

        // 清理 generated/
        if (Directory.Exists(generatedDir))
        {
            try
            {
                Directory.Delete(generatedDir, recursive: true);
                _context.WriteLine($"Removed: {generatedDir}");
                cleaned = true;
            }
            catch (IOException ex)
            {
                _context.WriteError($"Failed to remove {generatedDir}: {ex.Message}");
                return CommandResult.Error(1, $"Failed to clean generated directory: {ex.Message}");
            }
        }

        // 清理 .forge/ (僅 --all)
        if (cleanAll && Directory.Exists(cacheDir))
        {
            try
            {
                Directory.Delete(cacheDir, recursive: true);
                _context.WriteLine($"Removed: {cacheDir}");
                cleaned = true;
            }
            catch (IOException ex)
            {
                _context.WriteError($"Failed to remove {cacheDir}: {ex.Message}");
                return CommandResult.Error(1, $"Failed to clean cache directory: {ex.Message}");
            }
        }

        if (cleaned)
        {
            _context.WriteLine("");
            _context.WriteLine("Clean complete.");
        }
        else
        {
            _context.WriteLine("Nothing to clean.");
        }

        return CommandResult.Ok();
    }
}
