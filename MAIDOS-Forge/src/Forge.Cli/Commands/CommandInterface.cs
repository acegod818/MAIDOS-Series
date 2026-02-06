// MAIDOS-Forge CLI Command Interface
// UEP v1.7B Compliant - Zero Technical Debt

namespace Forge.Cli.Commands;

/// <summary>
/// CLI 命令結果
/// </summary>
/// <remarks>
/// <para>
/// 封裝 CLI 命令的執行結果，包含退出碼和可選的訊息。
/// </para>
/// <para>
/// APPROACH: 封裝命令執行結果，包含退出碼和訊息
/// CALLS: N/A (純資料)
/// EDGES: ExitCode 0 = 成功, 非 0 = 失敗
/// </para>
/// </remarks>
public readonly struct CommandResult
{
    public int ExitCode { get; }
    public string Message { get; }

    private CommandResult(int exitCode, string message)
    {
        ExitCode = exitCode;
        Message = message;
    }

    public static CommandResult Ok(string message = "") => new(0, message);
    public static CommandResult Error(int code, string message) => new(code, message);

    // 標準錯誤碼 (對應規格附錄 B)
    public static CommandResult ConfigNotFound() => Error(101, "forge.json not found");
    public static CommandResult ConfigSyntaxError(string detail) => Error(102, $"forge.json syntax error: {detail}");
    public static CommandResult ModuleNotFound(string name) => Error(103, $"Module not found: {name}");
    public static CommandResult ModuleSyntaxError(string name, string detail) => Error(104, $"module.json syntax error in {name}: {detail}");
    public static CommandResult CircularDependency(string detail) => Error(201, $"Circular dependency: {detail}");
    public static CommandResult DependencyNotFound(string detail) => Error(202, $"Dependency not found: {detail}");
}

/// <summary>
/// 命令介面
/// </summary>
public interface ICommand
{
    string Name { get; }
    string Description { get; }
    CommandResult Execute(string[] args);
}

/// <summary>
/// 命令上下文 - 共享狀態
/// </summary>
/// <impl>
/// APPROACH: 封裝命令執行時的共享狀態
/// CALLS: N/A (純資料)
/// EDGES: ProjectPath 預設為當前目錄
/// </impl>
public sealed class CommandContext
{
    public string ProjectPath { get; set; }
    public bool Verbose { get; set; }
    public TextWriter Output { get; set; }
    public TextWriter ErrorOutput { get; set; }

    public CommandContext()
    {
        ProjectPath = Directory.GetCurrentDirectory();
        Verbose = false;
        Output = Console.Out;
        ErrorOutput = Console.Error;
    }

    /// <summary>
    /// 輸出訊息
    /// </summary>
    /// <impl>
    /// APPROACH: 寫入 Output
    /// CALLS: TextWriter.WriteLine()
    /// EDGES: N/A
    /// </impl>
    public void WriteLine(string message) => Output.WriteLine(message);

    /// <summary>
    /// 輸出錯誤訊息
    /// </summary>
    /// <impl>
    /// APPROACH: 寫入 ErrorOutput
    /// CALLS: TextWriter.WriteLine()
    /// EDGES: N/A
    /// </impl>
    public void WriteError(string message) => ErrorOutput.WriteLine($"Error: {message}");

    /// <summary>
    /// 輸出詳細訊息（僅在 Verbose 模式）
    /// </summary>
    /// <impl>
    /// APPROACH: Verbose 為 true 時寫入 Output
    /// CALLS: TextWriter.WriteLine()
    /// EDGES: Verbose 為 false 時不輸出
    /// </impl>
    public void WriteVerbose(string message)
    {
        if (Verbose)
        {
            Output.WriteLine($"[DEBUG] {message}");
        }
    }
}