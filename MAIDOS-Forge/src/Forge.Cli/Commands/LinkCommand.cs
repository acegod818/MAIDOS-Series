// MAIDOS-Forge CLI - Link Command
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Core.Config;
using Forge.Core.Linker;

namespace Forge.Cli.Commands;

/// <summary>
/// forge link - 鏈接編譯產物
/// </summary>
/// <impl>
/// APPROACH: 收集編譯產物並執行鏈接
/// CALLS: LinkerManager, ConfigParser
/// EDGES: 編譯產物不存在時返回錯誤
/// </impl>
public sealed class LinkCommand : ICommand
{
    private readonly CommandContext _context;

    public string Name => "link";
    public string Description => "Link compiled artifacts";

    public LinkCommand(CommandContext context)
    {
        _context = context;
    }

    /// <summary>
    /// 執行 link 命令
    /// </summary>
    /// <impl>
    /// APPROACH: 解析參數，收集輸入，執行鏈接
    /// CALLS: LinkerManager.LinkAsync()
    /// EDGES: 顯示幫助時返回 OK
    /// </impl>
    public CommandResult Execute(string[] args)
    {
        // 解析參數
        var profile = "release";
        var outputName = string.Empty;
        var outputType = OutputType.Executable;
        var strip = false;
        var showLinkers = false;

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

                case "-o" or "--output":
                    if (i + 1 < args.Length)
                    {
                        outputName = args[++i];
                    }
                    break;

                case "--shared" or "--dll":
                    outputType = OutputType.SharedLibrary;
                    break;

                case "--static" or "--lib":
                    outputType = OutputType.StaticLibrary;
                    break;

                case "--strip" or "-s":
                    strip = true;
                    break;

                case "--linkers":
                    showLinkers = true;
                    break;
            }
        }

        // 顯示可用鏈接器
        if (showLinkers)
        {
            return ShowLinkers();
        }

        // 解析專案配置
        var parseResult = ConfigParser.ParseProject(_context.ProjectPath);
        if (!parseResult.IsSuccess)
        {
            _context.WriteError(parseResult.Error);
            return CommandResult.ConfigSyntaxError(parseResult.Error);
        }

        var config = parseResult.Value!;

        // 設定輸出名稱
        if (string.IsNullOrEmpty(outputName))
        {
            outputName = config.Config.Name;
        }

        // 收集編譯產物
        var buildDir = Path.Combine(config.ProjectRoot, config.Config.Output.Dir, profile);

        if (!Directory.Exists(buildDir))
        {
            _context.WriteError($"Build directory not found: {buildDir}");
            _context.WriteError("Run 'forge build' first.");
            return CommandResult.Error(302, "Build directory not found");
        }

        var modules = config.Modules
            .Select(m => (m.Config.Name, m.Config.Language))
            .ToList();

        var inputs = LinkerManager.CollectInputs(buildDir, modules);

        if (inputs.Count == 0)
        {
            _context.WriteError("No linkable artifacts found.");
            _context.WriteError("Run 'forge build' first.");
            return CommandResult.Error(302, "No linkable artifacts");
        }

        _context.WriteLine($"Linking {inputs.Count} artifact(s)...");
        _context.WriteLine($"Profile: {profile}");
        _context.WriteLine("");

        if (_context.Verbose)
        {
            _context.WriteLine("Inputs:");
            foreach (var input in inputs)
            {
                _context.WriteLine($"  [{input.Type}] {input.ModuleName}: {input.Path}");
            }
            _context.WriteLine("");
        }

        // 執行鏈接
        var outputDir = Path.Combine(config.ProjectRoot, config.Config.Output.Dir, profile);

        var linkConfig = new LinkConfig
        {
            OutputName = outputName,
            OutputDir = outputDir,
            OutputType = outputType,
            Target = TargetPlatform.Current,
            StripSymbols = strip,
            Profile = profile,
            Verbose = _context.Verbose
        };

        var linkerManager = new LinkerManager();
        var linkResult = linkerManager.LinkAsync(inputs, linkConfig).GetAwaiter().GetResult();

        if (!linkResult.IsSuccess)
        {
            _context.WriteError($"Link failed: {linkResult.Error}");

            if (_context.Verbose && linkResult.Logs.Count > 0)
            {
                _context.WriteLine("");
                _context.WriteLine("Logs:");
                foreach (var log in linkResult.Logs)
                {
                    _context.WriteLine($"  {log}");
                }
            }

            return CommandResult.Error(500, linkResult.Error);
        }

        _context.WriteLine($"✓ Link succeeded ({linkResult.Duration.TotalSeconds:F1}s)");
        _context.WriteLine($"  Output: {linkResult.OutputPath}");

        return CommandResult.Ok();
    }

    /// <summary>
    /// 顯示可用鏈接器
    /// </summary>
    /// <impl>
    /// APPROACH: 遍歷並驗證所有鏈接器
    /// CALLS: LinkerManager.ValidateAllAsync()
    /// EDGES: N/A
    /// </impl>
    private CommandResult ShowLinkers()
    {
        _context.WriteLine("Checking available linkers...");
        _context.WriteLine("");

        var linkerManager = new LinkerManager();
        var results = linkerManager.ValidateAllAsync().GetAwaiter().GetResult();

        var available = 0;
        var unavailable = 0;

        foreach (var (name, (isAvailable, message)) in results)
        {
            if (isAvailable)
            {
                available++;
                _context.WriteLine($"  ✓ {name}");
                _context.WriteLine($"      {message}");
            }
            else
            {
                unavailable++;
                _context.WriteLine($"  ✗ {name}");
                _context.WriteLine($"      {message}");
            }
            _context.WriteLine("");
        }

        _context.WriteLine($"Available: {available}, Unavailable: {unavailable}");

        return CommandResult.Ok();
    }

    private void ShowHelp()
    {
        _context.WriteLine("forge link - Link compiled artifacts");
        _context.WriteLine("");
        _context.WriteLine("USAGE:");
        _context.WriteLine("    forge link [options]");
        _context.WriteLine("");
        _context.WriteLine("OPTIONS:");
        _context.WriteLine("    -o, --output <name>  Set output file name");
        _context.WriteLine("    --debug              Link debug build");
        _context.WriteLine("    --release            Link release build (default)");
        _context.WriteLine("    --shared, --dll      Create shared library (.so/.dll/.dylib)");
        _context.WriteLine("    --static, --lib      Create static library (.a/.lib)");
        _context.WriteLine("    --strip, -s          Strip debug symbols");
        _context.WriteLine("    --linkers            Show available linkers");
        _context.WriteLine("    -v, --verbose        Verbose output");
        _context.WriteLine("    -h, --help           Show this help");
        _context.WriteLine("");
        _context.WriteLine("EXAMPLES:");
        _context.WriteLine("    forge link");
        _context.WriteLine("    forge link -o my-app");
        _context.WriteLine("    forge link --shared -o libcrypto");
        _context.WriteLine("    forge link --strip --release");
        _context.WriteLine("    forge link --linkers");
    }
}
