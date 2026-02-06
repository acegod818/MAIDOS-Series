// MAIDOS-Forge CLI Entry Point
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Cli.Commands;

namespace Forge.Cli;

/// <summary>
/// CLI 路由器 - 解析命令並分發執行
/// </summary>
/// <impl>
/// APPROACH: 註冊所有命令，根據 args[0] 選擇對應命令執行
/// CALLS: ICommand.Execute()
/// EDGES: 無參數顯示幫助, 未知命令顯示錯誤, --help 顯示幫助
/// </impl>
public static class Program
{
    private static readonly string Version = "0.7.0-m7";

    public static int Main(string[] args)
    {
        var context = new CommandContext();

        // 處理全域選項
        var argsList = args.ToList();
        
        if (argsList.Contains("-v") || argsList.Contains("--verbose"))
        {
            context.Verbose = true;
            argsList.Remove("-v");
            argsList.RemoveAll(a => a == "--verbose");
        }

        // 處理 --project 選項
        var projectIndex = argsList.IndexOf("--project");
        if (projectIndex == -1) projectIndex = argsList.IndexOf("-p");
        if (projectIndex >= 0 && projectIndex + 1 < argsList.Count)
        {
            context.ProjectPath = Path.GetFullPath(argsList[projectIndex + 1]);
            argsList.RemoveAt(projectIndex + 1);
            argsList.RemoveAt(projectIndex);
        }

        args = argsList.ToArray();

        // 無參數或 --help
        if (args.Length == 0 || args[0] == "--help" || args[0] == "-h")
        {
            ShowHelp(context);
            return 0;
        }

        // --version
        if (args[0] == "--version")
        {
            context.WriteLine($"forge {Version}");
            return 0;
        }

        // 註冊命令
        var commands = new Dictionary<string, ICommand>(StringComparer.OrdinalIgnoreCase)
        {
            ["init"] = new InitCommand(context),
            ["build"] = new BuildCommand(context),
            ["run"] = new RunCommand(context),
            ["watch"] = new WatchCommand(context),
            ["check"] = new CheckCommand(context),
            ["clean"] = new CleanCommand(context),
            ["graph"] = new GraphCommand(context),
            ["toolchain"] = new ToolchainCommand(context),
            ["ffi"] = new FfiCommand(context),
            ["link"] = new LinkCommand(context),
            ["plugin"] = new PluginCommand(context),
        };

        // 查找並執行命令
        var commandName = args[0];
        var commandArgs = args.Skip(1).ToArray();

        if (!commands.TryGetValue(commandName, out var command))
        {
            context.WriteError($"Unknown command: {commandName}");
            context.WriteLine("");
            ShowHelp(context);
            return 1;
        }

        // 命令級 --help
        if (commandArgs.Contains("--help") || commandArgs.Contains("-h"))
        {
            ShowCommandHelp(context, command);
            return 0;
        }

        var result = command.Execute(commandArgs);

        if (!string.IsNullOrEmpty(result.Message) && result.ExitCode != 0)
        {
            context.WriteError(result.Message);
        }

        return result.ExitCode;
    }

    /// <summary>
    /// 顯示主幫助資訊
    /// </summary>
    /// <impl>
    /// APPROACH: 輸出使用說明和命令列表
    /// CALLS: TextWriter.WriteLine()
    /// EDGES: N/A
    /// </impl>
    private static void ShowHelp(CommandContext context)
    {
        context.WriteLine($"MAIDOS-Forge v{Version}");
        context.WriteLine("Cross-language compilation router");
        context.WriteLine("");
        context.WriteLine("USAGE:");
        context.WriteLine("    forge <command> [options]");
        context.WriteLine("");
        context.WriteLine("COMMANDS:");
        context.WriteLine("    init [name]       Initialize a new project");
        context.WriteLine("    build [module]    Build the project (or specific module)");
        context.WriteLine("    run               Build and run the project");
        context.WriteLine("    watch             Watch for changes and rebuild");
        context.WriteLine("    link              Link compiled artifacts");
        context.WriteLine("    check             Validate configuration and dependencies");
        context.WriteLine("    clean             Clean build artifacts");
        context.WriteLine("    graph             Display dependency graph");
        context.WriteLine("    toolchain         Check available toolchains");
        context.WriteLine("    ffi               FFI interface management");
        context.WriteLine("    plugin            Manage plugins (list/install/remove)");
        context.WriteLine("");
        context.WriteLine("OPTIONS:");
        context.WriteLine("    -p, --project <path>    Set project directory");
        context.WriteLine("    -v, --verbose           Enable verbose output");
        context.WriteLine("    -h, --help              Show help");
        context.WriteLine("    --version               Show version");
        context.WriteLine("");
        context.WriteLine("EXAMPLES:");
        context.WriteLine("    forge init my-project");
        context.WriteLine("    forge build");
        context.WriteLine("    forge build --debug");
        context.WriteLine("    forge check --deps");
        context.WriteLine("    forge graph --dot > deps.dot");
        context.WriteLine("");
        context.WriteLine("For more information, run 'forge <command> --help'");
    }

    /// <summary>
    /// 顯示命令級幫助
    /// </summary>
    /// <impl>
    /// APPROACH: 根據命令名輸出對應說明
    /// CALLS: TextWriter.WriteLine()
    /// EDGES: 未知命令顯示通用說明
    /// </impl>
    private static void ShowCommandHelp(CommandContext context, ICommand command)
    {
        context.WriteLine($"forge {command.Name} - {command.Description}");
        context.WriteLine("");

        switch (command.Name)
        {
            case "init":
                context.WriteLine("USAGE:");
                context.WriteLine("    forge init [name]");
                context.WriteLine("");
                context.WriteLine("ARGUMENTS:");
                context.WriteLine("    [name]    Project name (default: current directory name)");
                context.WriteLine("");
                context.WriteLine("EXAMPLES:");
                context.WriteLine("    forge init");
                context.WriteLine("    forge init my-project");
                break;

            case "build":
                context.WriteLine("USAGE:");
                context.WriteLine("    forge build [module] [options]");
                context.WriteLine("");
                context.WriteLine("ARGUMENTS:");
                context.WriteLine("    [module]    Specific module to build (default: all)");
                context.WriteLine("");
                context.WriteLine("OPTIONS:");
                context.WriteLine("    --debug     Build with debug profile");
                context.WriteLine("    --target    Target platform (e.g., linux-arm64)");
                context.WriteLine("");
                context.WriteLine("EXAMPLES:");
                context.WriteLine("    forge build");
                context.WriteLine("    forge build --debug");
                context.WriteLine("    forge build core");
                break;

            case "check":
                context.WriteLine("USAGE:");
                context.WriteLine("    forge check [options]");
                context.WriteLine("");
                context.WriteLine("OPTIONS:");
                context.WriteLine("    --deps      Show dependency graph");
                context.WriteLine("");
                context.WriteLine("EXAMPLES:");
                context.WriteLine("    forge check");
                context.WriteLine("    forge check --deps");
                break;

            case "clean":
                context.WriteLine("USAGE:");
                context.WriteLine("    forge clean [options]");
                context.WriteLine("");
                context.WriteLine("OPTIONS:");
                context.WriteLine("    --all       Also clean .forge/ cache");
                context.WriteLine("");
                context.WriteLine("EXAMPLES:");
                context.WriteLine("    forge clean");
                context.WriteLine("    forge clean --all");
                break;

            case "graph":
                context.WriteLine("USAGE:");
                context.WriteLine("    forge graph [options]");
                context.WriteLine("");
                context.WriteLine("OPTIONS:");
                context.WriteLine("    --dot       Output in Graphviz DOT format");
                context.WriteLine("");
                context.WriteLine("EXAMPLES:");
                context.WriteLine("    forge graph");
                context.WriteLine("    forge graph --dot > deps.dot");
                break;

            default:
                context.WriteLine($"Run 'forge {command.Name} --help' for usage.");
                break;
        }
    }
}
