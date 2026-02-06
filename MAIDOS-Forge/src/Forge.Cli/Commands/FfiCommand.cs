// MAIDOS-Forge CLI - FFI Command
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Core.Config;
using Forge.Core.FFI;

namespace Forge.Cli.Commands;

/// <summary>
/// forge ffi - FFI 接口管理
/// </summary>
/// <impl>
/// APPROACH: 子命令處理接口提取、膠水生成、接口檢視
/// CALLS: InterfaceExtractor, GlueGenerator
/// EDGES: 無子命令顯示幫助
/// </impl>
public sealed class FfiCommand : ICommand
{
    private readonly CommandContext _context;

    public string Name => "ffi";
    public string Description => "FFI interface management";

    public FfiCommand(CommandContext context)
    {
        _context = context;
    }

    /// <summary>
    /// 執行 ffi 命令
    /// </summary>
    /// <impl>
    /// APPROACH: 根據子命令分發到對應處理函數
    /// CALLS: HandleExtract(), HandleGlue(), HandleShow()
    /// EDGES: 無子命令顯示幫助
    /// </impl>
    public CommandResult Execute(string[] args)
    {
        if (args.Length == 0 || args[0] == "--help" || args[0] == "-h")
        {
            ShowHelp();
            return CommandResult.Ok();
        }

        var subCommand = args[0];
        var subArgs = args.Skip(1).ToArray();

        return subCommand.ToLowerInvariant() switch
        {
            "extract" => HandleExtract(subArgs),
            "glue" => HandleGlue(subArgs),
            "show" => HandleShow(subArgs),
            _ => HandleUnknown(subCommand)
        };
    }

    /// <summary>
    /// 處理 extract 子命令
    /// </summary>
    /// <impl>
    /// APPROACH: 從編譯產物提取接口並輸出 JSON
    /// CALLS: InterfaceExtractor.ExtractAsync()
    /// EDGES: 模組不存在返回錯誤, 無產物返回錯誤
    /// </impl>
    private CommandResult HandleExtract(string[] args)
    {
        if (args.Length == 0)
        {
            _context.WriteError("Usage: forge ffi extract <module>");
            return CommandResult.Error(1, "Missing module name");
        }

        var moduleName = args[0];
        var outputFile = args.Length > 1 ? args[1] : null;

        // 解析專案配置
        var parseResult = ConfigParser.ParseProject(_context.ProjectPath);
        if (!parseResult.IsSuccess)
        {
            _context.WriteError(parseResult.Error);
            return CommandResult.ConfigSyntaxError(parseResult.Error);
        }

        var config = parseResult.Value!;

        // 查找模組
        var module = config.Modules.FirstOrDefault(m =>
            string.Equals(m.Config.Name, moduleName, StringComparison.OrdinalIgnoreCase));

        if (module is null)
        {
            _context.WriteError($"Module not found: {moduleName}");
            return CommandResult.ModuleNotFound(moduleName);
        }

        // 查找編譯產物
        var buildDir = Path.Combine(config.ProjectRoot, config.Config.Output.Dir, "release", moduleName);
        if (!Directory.Exists(buildDir))
        {
            _context.WriteError($"Build output not found. Run 'forge build' first.");
            return CommandResult.Error(302, "Build output not found");
        }

        // 根據語言選擇產物
        var artifactPath = module.Config.Language.ToLowerInvariant() switch
        {
            "csharp" => Directory.GetFiles(buildDir, "*.dll").FirstOrDefault(),
            "rust" => Directory.GetFiles(buildDir, "*.rlib")
                .Concat(Directory.GetFiles(buildDir, "*.a"))
                .Concat(Directory.GetFiles(buildDir, "*.so"))
                .FirstOrDefault(),
            _ => null
        };

        if (artifactPath is null)
        {
            _context.WriteError($"No artifact found for module: {moduleName}");
            return CommandResult.Error(302, "Artifact not found");
        }

        _context.WriteLine($"Extracting interface from: {artifactPath}");

        // 提取接口
        var extractResult = InterfaceExtractor.ExtractAsync(
            artifactPath, moduleName, module.Config.Language).GetAwaiter().GetResult();

        if (!extractResult.IsSuccess)
        {
            _context.WriteError(extractResult.Error);
            return CommandResult.Error(500, extractResult.Error);
        }

        var iface = extractResult.Interface!;
        var json = iface.ToJson();

        // 輸出
        if (outputFile is not null)
        {
            var outputPath = Path.GetFullPath(outputFile);
            File.WriteAllText(outputPath, json);
            _context.WriteLine($"Interface saved to: {outputPath}");
        }
        else
        {
            _context.WriteLine("");
            _context.WriteLine(json);
        }

        _context.WriteLine("");
        _context.WriteLine($"Exports: {iface.Exports.Count} function(s)");

        return CommandResult.Ok();
    }

    /// <summary>
    /// 處理 glue 子命令
    /// </summary>
    /// <impl>
    /// APPROACH: 從接口 JSON 生成膠水代碼
    /// CALLS: GlueGenerator.Generate()
    /// EDGES: 接口檔案不存在返回錯誤
    /// </impl>
    private CommandResult HandleGlue(string[] args)
    {
        if (args.Length < 2)
        {
            _context.WriteError("Usage: forge ffi glue <interface.json> <target-language>");
            _context.WriteError("       forge ffi glue <source-module> <target-module>");
            return CommandResult.Error(1, "Missing arguments");
        }

        var source = args[0];
        var target = args[1];
        var outputDir = args.Length > 2 ? args[2] : ".";

        ModuleInterface? iface;

        // 判斷 source 是檔案還是模組名稱
        if (source.EndsWith(".json", StringComparison.OrdinalIgnoreCase) && File.Exists(source))
        {
            // 從檔案載入
            var json = File.ReadAllText(source);
            iface = ModuleInterface.FromJson(json);

            if (iface is null)
            {
                _context.WriteError($"Failed to parse interface file: {source}");
                return CommandResult.Error(500, "Invalid interface file");
            }
        }
        else
        {
            // 從模組提取
            var parseResult = ConfigParser.ParseProject(_context.ProjectPath);
            if (!parseResult.IsSuccess)
            {
                _context.WriteError(parseResult.Error);
                return CommandResult.ConfigSyntaxError(parseResult.Error);
            }

            var config = parseResult.Value!;
            var module = config.Modules.FirstOrDefault(m =>
                string.Equals(m.Config.Name, source, StringComparison.OrdinalIgnoreCase));

            if (module is null)
            {
                _context.WriteError($"Module not found: {source}");
                return CommandResult.ModuleNotFound(source);
            }

            // 查找產物
            var buildDir = Path.Combine(config.ProjectRoot, config.Config.Output.Dir, "release", source);
            var artifactPath = Directory.Exists(buildDir)
                ? Directory.GetFiles(buildDir, "*.*").FirstOrDefault(f =>
                    f.EndsWith(".dll") || f.EndsWith(".rlib") || f.EndsWith(".a") || f.EndsWith(".so"))
                : null;

            if (artifactPath is null)
            {
                _context.WriteError($"No artifact found. Run 'forge build' first.");
                return CommandResult.Error(302, "Artifact not found");
            }

            var extractResult = InterfaceExtractor.ExtractAsync(
                artifactPath, source, module.Config.Language).GetAwaiter().GetResult();

            if (!extractResult.IsSuccess)
            {
                _context.WriteError(extractResult.Error);
                return CommandResult.Error(500, extractResult.Error);
            }

            iface = extractResult.Interface!;
        }

        _context.WriteLine($"Generating {target} glue for: {iface.ModuleName}");

        // 生成膠水代碼
        var glueResult = GlueGenerator.Generate(iface, target);

        if (!glueResult.IsSuccess)
        {
            _context.WriteError(glueResult.Error);
            return CommandResult.Error(500, glueResult.Error);
        }

        // 輸出檔案
        Directory.CreateDirectory(outputDir);
        var outputPath = Path.Combine(outputDir, glueResult.FileName);
        File.WriteAllText(outputPath, glueResult.SourceCode);

        _context.WriteLine($"Generated: {outputPath}");
        _context.WriteLine($"Language: {glueResult.TargetLanguage}");
        _context.WriteLine($"Size: {glueResult.SourceCode.Length} bytes");

        return CommandResult.Ok();
    }

    /// <summary>
    /// 處理 show 子命令
    /// </summary>
    /// <impl>
    /// APPROACH: 顯示接口 JSON 的摘要
    /// CALLS: ModuleInterface.FromJson()
    /// EDGES: 檔案不存在返回錯誤
    /// </impl>
    private CommandResult HandleShow(string[] args)
    {
        if (args.Length == 0)
        {
            _context.WriteError("Usage: forge ffi show <interface.json>");
            return CommandResult.Error(1, "Missing interface file");
        }

        var filePath = args[0];

        if (!File.Exists(filePath))
        {
            _context.WriteError($"File not found: {filePath}");
            return CommandResult.Error(404, "File not found");
        }

        var json = File.ReadAllText(filePath);
        var iface = ModuleInterface.FromJson(json);

        if (iface is null)
        {
            _context.WriteError("Failed to parse interface file");
            return CommandResult.Error(500, "Invalid interface file");
        }

        _context.WriteLine($"Module: {iface.ModuleName} v{iface.ModuleVersion}");
        _context.WriteLine($"Language: {iface.Language} ({iface.Abi})");
        _context.WriteLine($"Mode: {iface.Mode}");
        _context.WriteLine("");
        _context.WriteLine($"Exports ({iface.Exports.Count}):");

        foreach (var export in iface.Exports)
        {
            var sig = export.Signature;
            var parms = string.Join(", ", sig.Parameters.Select(p => $"{p.Name}: {p.Type.ToRustType()}"));
            var ret = sig.ReturnType.ToRustType();

            _context.WriteLine($"  fn {export.Name}({parms}) -> {ret}");

            if (export.Attributes.Count > 0)
            {
                _context.WriteLine($"     [{string.Join(", ", export.Attributes)}]");
            }
        }

        return CommandResult.Ok();
    }

    private CommandResult HandleUnknown(string subCommand)
    {
        _context.WriteError($"Unknown subcommand: {subCommand}");
        ShowHelp();
        return CommandResult.Error(1, "Unknown subcommand");
    }

    private void ShowHelp()
    {
        _context.WriteLine("forge ffi - FFI interface management");
        _context.WriteLine("");
        _context.WriteLine("SUBCOMMANDS:");
        _context.WriteLine("    extract <module>              Extract interface from compiled module");
        _context.WriteLine("    glue <source> <target-lang>   Generate glue code for target language");
        _context.WriteLine("    show <interface.json>         Display interface summary");
        _context.WriteLine("");
        _context.WriteLine("EXAMPLES:");
        _context.WriteLine("    forge ffi extract crypto");
        _context.WriteLine("    forge ffi extract crypto -o crypto.interface.json");
        _context.WriteLine("    forge ffi glue crypto.interface.json csharp");
        _context.WriteLine("    forge ffi glue crypto rust -o src/generated/");
        _context.WriteLine("    forge ffi show crypto.interface.json");
    }
}
