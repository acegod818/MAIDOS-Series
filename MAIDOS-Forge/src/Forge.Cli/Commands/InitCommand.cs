// MAIDOS-Forge CLI - Init Command
// UEP v1.7B Compliant - Zero Technical Debt

using System.Text.Json;

namespace Forge.Cli.Commands;

/// <summary>
/// forge init - 建立新專案
/// </summary>
/// <remarks>
/// <para>
/// 此命令用於初始化一個新的 MAIDOS-Forge 專案，建立基本的目錄結構和配置文件。
/// </para>
/// <para>
/// APPROACH: 在指定目錄建立專案骨架（forge.json + modules/ 目錄）
/// CALLS: Directory.CreateDirectory(), File.WriteAllText()
/// EDGES: 目錄已存在且有 forge.json 時報錯, 寫入失敗時報錯
/// </para>
/// </remarks>
/// <example>
/// <code>
/// forge init my-project
/// </code>
/// </example>
public sealed class InitCommand : ICommand
{
    private readonly CommandContext _context;

    public string Name => "init";
    public string Description => "Initialize a new Forge project";

    public InitCommand(CommandContext context)
    {
        _context = context;
    }

    /// <summary>
    /// 執行 init 命令
    /// </summary>
    /// <impl>
    /// APPROACH: 解析參數，建立目錄結構，寫入預設 forge.json
    /// CALLS: Directory.CreateDirectory(), File.WriteAllText(), JsonSerializer.Serialize()
    /// EDGES: 第一個參數為專案名，無參數使用當前目錄名, forge.json 已存在報錯
    /// </impl>
    public CommandResult Execute(string[] args)
    {
        // 解析專案名稱
        string projectName;
        string projectPath;

        if (args.Length > 0 && !args[0].StartsWith('-'))
        {
            projectName = args[0];
            projectPath = Path.Combine(_context.ProjectPath, projectName);
        }
        else
        {
            projectName = Path.GetFileName(_context.ProjectPath);
            projectPath = _context.ProjectPath;
        }

        // 驗證專案名
        if (string.IsNullOrWhiteSpace(projectName))
        {
            return CommandResult.Error(1, "Project name cannot be empty");
        }

        // 檢查是否已存在 forge.json
        var forgeJsonPath = Path.Combine(projectPath, "forge.json");
        if (File.Exists(forgeJsonPath))
        {
            return CommandResult.Error(1, $"Project already initialized: {forgeJsonPath}");
        }

        try
        {
            // 建立目錄結構
            Directory.CreateDirectory(projectPath);
            Directory.CreateDirectory(Path.Combine(projectPath, "modules"));
            Directory.CreateDirectory(Path.Combine(projectPath, "build"));
            Directory.CreateDirectory(Path.Combine(projectPath, "generated"));

            // 建立預設 forge.json
            var config = new
            {
                name = projectName,
                version = "0.1.0",
                output = new
                {
                    dir = "build",
                    artifact_name = projectName
                },
                target = new
                {
                    @default = "native"
                },
                modules = Array.Empty<string>()
            };

            var jsonOptions = new JsonSerializerOptions 
            { 
                WriteIndented = true,
                PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower
            };
            var json = JsonSerializer.Serialize(config, jsonOptions);
            File.WriteAllText(forgeJsonPath, json);

            // 建立 .gitignore
            var gitignore = """
                build/
                .forge/
                generated/
                *.user
                *.suo
                .vs/
                .vscode/
                .idea/
                """;
            File.WriteAllText(Path.Combine(projectPath, ".gitignore"), gitignore);

            _context.WriteLine($"Initialized project '{projectName}' in {projectPath}");
            _context.WriteLine("");
            _context.WriteLine("Next steps:");
            _context.WriteLine($"  cd {projectName}");
            _context.WriteLine("  forge build");

            return CommandResult.Ok();
        }
        catch (IOException ex)
        {
            return CommandResult.Error(1, $"Failed to create project: {ex.Message}");
        }
    }
}