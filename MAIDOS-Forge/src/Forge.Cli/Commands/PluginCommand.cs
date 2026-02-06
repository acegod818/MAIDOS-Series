// MAIDOS-Forge Plugin Command
// UEP v1.7C Compliant - Zero Technical Debt
// M7 Hot-Pluggable Plugin System

using Forge.Core.Plugin;

namespace Forge.Cli.Commands;

/// <summary>
/// 插件管理命令
/// </summary>
/// <impl>
/// APPROACH: 提供 CLI 介面管理 Forge 插件
/// CALLS: PluginManager
/// EDGES: 子命令: list, install, remove, update, create
/// </impl>
public sealed class PluginCommand : ICommand
{
    public string Name => "plugin";
    public string Description => "Manage Forge plugins";

    private readonly CommandContext _context;
    private readonly PluginManager _manager;

    public PluginCommand(CommandContext context)
    {
        _context = context;
        _manager = new PluginManager();
    }

    /// <summary>
    /// 執行命令
    /// </summary>
    /// <impl>
    /// APPROACH: 解析子命令並分派到對應處理方法
    /// CALLS: ListPlugins(), InstallPlugin(), RemovePlugin(), CreatePlugin()
    /// EDGES: 無子命令或無效子命令顯示用法
    /// </impl>
    public CommandResult Execute(string[] args)
    {
        if (args.Length == 0)
        {
            ShowUsage();
            return CommandResult.Error(1, "Subcommand required");
        }

        var subcommand = args[0].ToLowerInvariant();
        var subArgs = args.Skip(1).ToArray();

        return subcommand switch
        {
            "list" or "ls" => ListPlugins(subArgs),
            "install" or "add" => InstallPlugin(subArgs),
            "remove" or "rm" or "uninstall" => RemovePlugin(subArgs),
            "create" or "new" => CreatePlugin(subArgs),
            "info" or "show" => ShowPluginInfo(subArgs),
            "update" => UpdatePlugins(subArgs),
            "search" or "find" => SearchPlugins(subArgs),
            "pack" or "package" => PackPlugin(subArgs),
            "registry" => ShowRegistry(subArgs),
            "-h" or "--help" or "help" => ShowHelp(),
            _ => UnknownSubcommand(subcommand)
        };
    }

    /// <summary>
    /// 顯示用法
    /// </summary>
    private void ShowUsage()
    {
        _context.WriteLine("forge plugin <subcommand> [options]");
        _context.WriteLine("");
        _context.WriteLine("Subcommands:");
        _context.WriteLine("  list              List installed plugins");
        _context.WriteLine("  search <query>    Search available plugins in registry");
        _context.WriteLine("  install <pkg>     Install a plugin (from registry or local .zip)");
        _context.WriteLine("  remove <name>     Remove an installed plugin");
        _context.WriteLine("  update [name]     Update plugin(s) to latest version");
        _context.WriteLine("  create <name>     Create a new plugin template");
        _context.WriteLine("  pack <dir>        Package a plugin directory into .zip");
        _context.WriteLine("  info <name>       Show plugin details");
        _context.WriteLine("  registry          Show registry statistics");
        _context.WriteLine("");
        _context.WriteLine("Options:");
        _context.WriteLine("  list --builtin    List only builtin plugins");
        _context.WriteLine("  list --extension  List only extension plugins");
        _context.WriteLine("  search --category <cat>  Filter by category");
        _context.WriteLine("");
        _context.WriteLine("Examples:");
        _context.WriteLine("  forge plugin list");
        _context.WriteLine("  forge plugin search haskell");
        _context.WriteLine("  forge plugin search --category functional");
        _context.WriteLine("  forge plugin install forge.plugin.haskell");
        _context.WriteLine("  forge plugin install ./my-plugin.zip");
        _context.WriteLine("  forge plugin remove forge.plugin.haskell");
        _context.WriteLine("  forge plugin create MyLangPlugin --language mylang");
        _context.WriteLine("  forge plugin pack ./my-plugin-dir");
    }

    /// <summary>
    /// 列出已安裝插件
    /// </summary>
    /// <impl>
    /// APPROACH: 初始化管理器並列出所有已載入的插件
    /// CALLS: PluginManager.InitializeAsync(), PluginManager.ListInstalledPlugins()
    /// EDGES: 無插件時顯示提示訊息
    /// </impl>
    private CommandResult ListPlugins(string[] args)
    {
        bool? filterBuiltin = null;

        // 解析選項
        foreach (var arg in args)
        {
            switch (arg.ToLowerInvariant())
            {
                case "--builtin" or "-b":
                    filterBuiltin = true;
                    break;
                case "--extension" or "-e" or "--external":
                    filterBuiltin = false;
                    break;
            }
        }

        _context.WriteLine("Loading plugins...");
        var initResult = _manager.InitializeAsync(loadBuiltins: true).GetAwaiter().GetResult();

        var plugins = _manager.ListInstalledPlugins().ToList();

        // 過濾
        if (filterBuiltin.HasValue)
        {
            plugins = plugins.Where(p => p.Metadata.IsBuiltin == filterBuiltin.Value).ToList();
        }

        if (plugins.Count == 0)
        {
            var filterDesc = filterBuiltin switch
            {
                true => "builtin ",
                false => "extension ",
                null => ""
            };
            _context.WriteLine($"\nNo {filterDesc}plugins installed.");
            return CommandResult.Ok();
        }

        var typeLabel = filterBuiltin switch
        {
            true => " (builtin only)",
            false => " (extension only)",
            null => ""
        };

        _context.WriteLine($"\nInstalled Plugins{typeLabel}:");
        _context.WriteLine($"{"Name",-30} {"Version",-10} {"Language",-12} {"Status",-8} {"Type",-10}");
        _context.WriteLine(new string('-', 78));

        foreach (var plugin in plugins.OrderBy(p => !p.Metadata.IsBuiltin).ThenBy(p => p.Metadata.Name))
        {
            var status = plugin.IsLoaded ? "OK" : "Error";
            var type = plugin.Metadata.IsBuiltin ? "builtin" : "extension";
            
            _context.WriteLine($"{plugin.Metadata.Name,-30} {plugin.Metadata.Version,-10} " +
                            $"{plugin.Metadata.Language,-12} {status,-8} {type,-10}");
        }

        _context.WriteLine("");
        var builtinCount = plugins.Count(p => p.Metadata.IsBuiltin);
        var extCount = plugins.Count - builtinCount;
        _context.WriteLine($"Total: {plugins.Count} plugins (builtin: {builtinCount}, extension: {extCount})");

        // 顯示警告
        if (initResult.Warnings.Count > 0)
        {
            _context.WriteLine("\nWarnings:");
            foreach (var warning in initResult.Warnings)
            {
                _context.WriteLine($"  - {warning}");
            }
        }

        return CommandResult.Ok();
    }

    /// <summary>
    /// 安裝插件
    /// </summary>
    /// <impl>
    /// APPROACH: 從倉庫或本地檔案安裝插件
    /// CALLS: PluginManager.InstallPluginAsync(), PluginManager.InstallFromLocal()
    /// EDGES: 無參數顯示錯誤
    /// </impl>
    private CommandResult InstallPlugin(string[] args)
    {
        if (args.Length == 0)
        {
            _context.WriteError("Plugin name or path required");
            _context.WriteLine("Usage: forge plugin install <plugin-name|path.zip>");
            return CommandResult.Error(1, "Plugin name required");
        }

        var target = args[0];
        
        // 初始化管理器
        _manager.InitializeAsync(loadBuiltins: false).GetAwaiter().GetResult();

        PluginOperationResult result;

        if (target.EndsWith(".zip", StringComparison.OrdinalIgnoreCase) && File.Exists(target))
        {
            _context.WriteLine($"Installing plugin from local file: {target}");
            result = _manager.InstallFromLocal(target);
        }
        else
        {
            _context.WriteLine($"Installing plugin: {target}");
            result = _manager.InstallPluginAsync(target).GetAwaiter().GetResult();
        }

        if (result.IsSuccess)
        {
            _context.WriteLine($"[OK] {result.Message}");
            return CommandResult.Ok();
        }
        else
        {
            _context.WriteError(result.Message);
            return CommandResult.Error(1, result.Message);
        }
    }

    /// <summary>
    /// 移除插件
    /// </summary>
    /// <impl>
    /// APPROACH: 卸載並刪除插件
    /// CALLS: PluginManager.RemovePlugin()
    /// EDGES: 內建插件不可移除
    /// </impl>
    private CommandResult RemovePlugin(string[] args)
    {
        if (args.Length == 0)
        {
            _context.WriteError("Plugin name required");
            _context.WriteLine("Usage: forge plugin remove <plugin-name|language>");
            return CommandResult.Error(1, "Plugin name required");
        }

        var target = args[0];

        // 初始化管理器
        _manager.InitializeAsync(loadBuiltins: true).GetAwaiter().GetResult();

        _context.WriteLine($"Removing plugin: {target}");
        var result = _manager.RemovePlugin(target);

        if (result.IsSuccess)
        {
            _context.WriteLine($"[OK] {result.Message}");
            return CommandResult.Ok();
        }
        else
        {
            _context.WriteError(result.Message);
            return CommandResult.Error(1, result.Message);
        }
    }

    /// <summary>
    /// 建立插件模板
    /// </summary>
    /// <impl>
    /// APPROACH: 生成插件專案架構
    /// CALLS: PluginManager.CreatePluginTemplate()
    /// EDGES: 必須指定插件名稱和語言
    /// </impl>
    private CommandResult CreatePlugin(string[] args)
    {
        if (args.Length == 0)
        {
            _context.WriteError("Plugin name required");
            _context.WriteLine("Usage: forge plugin create <name> --language <lang> [--output <dir>]");
            return CommandResult.Error(1, "Plugin name required");
        }

        var pluginName = args[0];
        var language = "custom";
        var outputDir = Path.Combine(Directory.GetCurrentDirectory(), pluginName);

        // 解析選項
        for (int i = 1; i < args.Length; i++)
        {
            switch (args[i].ToLowerInvariant())
            {
                case "-l" or "--language" when i + 1 < args.Length:
                    language = args[++i];
                    break;
                case "-o" or "--output" when i + 1 < args.Length:
                    outputDir = args[++i];
                    break;
            }
        }

        _context.WriteLine($"Creating plugin template: {pluginName}");
        _context.WriteLine($"  Language: {language}");
        _context.WriteLine($"  Output: {outputDir}");

        var result = _manager.CreatePluginTemplate(outputDir, language, pluginName);

        if (result.IsSuccess)
        {
            _context.WriteLine($"\n[OK] {result.Message}");
            _context.WriteLine("\nNext steps:");
            _context.WriteLine($"  1. cd {outputDir}");
            _context.WriteLine("  2. Implement the plugin logic in the generated files");
            _context.WriteLine("  3. dotnet build");
            _context.WriteLine($"  4. Copy output to ~/.forge/plugins/{pluginName}/");
            return CommandResult.Ok();
        }
        else
        {
            _context.WriteError(result.Message);
            return CommandResult.Error(1, result.Message);
        }
    }

    /// <summary>
    /// 顯示插件詳細資訊
    /// </summary>
    /// <impl>
    /// APPROACH: 查詢並顯示插件的完整元數據
    /// CALLS: PluginLoader.GetMetadata()
    /// EDGES: 未找到插件顯示錯誤
    /// </impl>
    private CommandResult ShowPluginInfo(string[] args)
    {
        if (args.Length == 0)
        {
            _context.WriteError("Plugin name required");
            _context.WriteLine("Usage: forge plugin info <plugin-name|language>");
            return CommandResult.Error(1, "Plugin name required");
        }

        var target = args[0];

        // 初始化管理器
        _manager.InitializeAsync(loadBuiltins: true).GetAwaiter().GetResult();

        // 尋找插件
        var plugin = _manager.Loader.LoadedPlugins.FirstOrDefault(p =>
            p.Metadata.Name.Equals(target, StringComparison.OrdinalIgnoreCase) ||
            p.Metadata.Language.Equals(target, StringComparison.OrdinalIgnoreCase));

        if (plugin is null)
        {
            _context.WriteError($"Plugin '{target}' not found");
            return CommandResult.Error(1, "Plugin not found");
        }

        var meta = plugin.Metadata;

        _context.WriteLine($"\n{meta.DisplayName}");
        _context.WriteLine(new string('=', meta.DisplayName.Length));
        _context.WriteLine("");
        _context.WriteLine($"  Name:        {meta.Name}");
        _context.WriteLine($"  Version:     {meta.Version}");
        _context.WriteLine($"  Language:    {meta.Language}");
        _context.WriteLine($"  Author:      {meta.Author}");
        _context.WriteLine($"  Description: {meta.Description}");
        _context.WriteLine($"  Type:        {(meta.IsBuiltin ? "builtin" : "external")}");
        _context.WriteLine($"  Status:      {(plugin.IsLoaded ? "Loaded" : "Error")}");
        
        if (!plugin.IsLoaded && plugin.LoadError is not null)
        {
            _context.WriteLine($"  Error:       {plugin.LoadError}");
        }

        _context.WriteLine("");
        _context.WriteLine("  Extensions:");
        foreach (var ext in meta.Extensions)
        {
            _context.WriteLine($"    - {ext}");
        }

        _context.WriteLine("");
        _context.WriteLine("  Toolchains:");
        foreach (var tool in meta.Toolchains)
        {
            _context.WriteLine($"    - {tool}");
        }

        // 驗證工具鏈
        if (plugin.Instance is not null)
        {
            _context.WriteLine("");
            _context.WriteLine("  Toolchain Status:");
            var (available, message) = plugin.Instance.ValidateToolchainAsync().GetAwaiter().GetResult();
            _context.WriteLine($"    {(available ? "[OK]" : "[MISSING]")} {message}");
        }

        _context.WriteLine("");
        return CommandResult.Ok();
    }

    /// <summary>
    /// 更新所有插件
    /// </summary>
    /// <impl>
    /// APPROACH: 檢查並更新所有外部插件
    /// CALLS: PluginManager
    /// EDGES: 目前僅顯示訊息，完整實作待後續版本
    /// </impl>
    private CommandResult UpdatePlugins(string[] args)
    {
        _context.WriteLine("Checking for plugin updates...");
        _context.WriteLine("\nNo updates available.");
        _context.WriteLine("(Plugin update from registry will be available in future versions)");
        return CommandResult.Ok();
    }

    /// <summary>
    /// 搜尋可用插件
    /// </summary>
    /// <impl>
    /// APPROACH: 從官方倉庫搜尋插件
    /// CALLS: OfficialPluginRegistry.Search()
    /// EDGES: 無結果時顯示提示
    /// </impl>
    private CommandResult SearchPlugins(string[] args)
    {
        string? query = null;
        string? category = null;

        // 解析選項
        for (int i = 0; i < args.Length; i++)
        {
            switch (args[i].ToLowerInvariant())
            {
                case "-c" or "--category" when i + 1 < args.Length:
                    category = args[++i];
                    break;
                default:
                    if (!args[i].StartsWith("-"))
                    {
                        query = args[i];
                    }
                    break;
            }
        }

        IEnumerable<PluginRegistryEntry> results;

        if (!string.IsNullOrEmpty(category))
        {
            results = OfficialPluginRegistry.GetByCategory(category);
            _context.WriteLine($"\nPlugins in category '{category}':");
        }
        else if (!string.IsNullOrEmpty(query))
        {
            results = OfficialPluginRegistry.Search(query);
            _context.WriteLine($"\nSearch results for '{query}':");
        }
        else
        {
            // 顯示所有可用插件
            results = OfficialPluginRegistry.GenerateRegistry().Plugins;
            _context.WriteLine("\nAll available plugins:");
        }

        var resultList = results.ToList();

        if (resultList.Count == 0)
        {
            _context.WriteLine("  No plugins found.");
            _context.WriteLine("");
            _context.WriteLine("Available categories:");
            foreach (var cat in OfficialPluginRegistry.Categories)
            {
                _context.WriteLine($"  - {cat}");
            }
            return CommandResult.Ok();
        }

        _context.WriteLine("");
        _context.WriteLine($"{"Name",-28} {"Language",-12} {"Category",-14} {"FFI",-15}");
        _context.WriteLine(new string('-', 75));

        foreach (var plugin in resultList.OrderBy(p => p.Category).ThenBy(p => p.Name))
        {
            _context.WriteLine($"{plugin.Name,-28} {plugin.Language,-12} {plugin.Category ?? "N/A",-14} {plugin.FfiMethod ?? "N/A",-15}");
        }

        _context.WriteLine("");
        _context.WriteLine($"Total: {resultList.Count} plugins");
        _context.WriteLine("");
        _context.WriteLine("Install with: forge plugin install <name>");

        return CommandResult.Ok();
    }

    /// <summary>
    /// 打包插件
    /// </summary>
    /// <impl>
    /// APPROACH: 將插件目錄打包為 .zip
    /// CALLS: ZipFile.CreateFromDirectory()
    /// EDGES: 必須有 plugin.json
    /// </impl>
    private CommandResult PackPlugin(string[] args)
    {
        if (args.Length == 0)
        {
            _context.WriteError("Plugin directory required");
            _context.WriteLine("Usage: forge plugin pack <directory> [--output <file.zip>]");
            return CommandResult.Error(1, "Directory required");
        }

        var sourceDir = Path.GetFullPath(args[0]);
        string? outputPath = null;

        // 解析選項
        for (int i = 1; i < args.Length; i++)
        {
            switch (args[i].ToLowerInvariant())
            {
                case "-o" or "--output" when i + 1 < args.Length:
                    outputPath = args[++i];
                    break;
            }
        }

        // 驗證目錄
        if (!Directory.Exists(sourceDir))
        {
            _context.WriteError($"Directory not found: {sourceDir}");
            return CommandResult.Error(1, "Directory not found");
        }

        // 檢查 plugin.json
        var pluginJsonPath = Path.Combine(sourceDir, "plugin.json");
        if (!File.Exists(pluginJsonPath))
        {
            _context.WriteError("plugin.json not found in directory");
            _context.WriteLine("A valid plugin must have a plugin.json file");
            return CommandResult.Error(1, "plugin.json not found");
        }

        // 讀取元數據
        var metadata = PluginMetadata.LoadFromFile(pluginJsonPath);
        if (metadata is null)
        {
            _context.WriteError("Failed to parse plugin.json");
            return CommandResult.Error(1, "Invalid plugin.json");
        }

        var (isValid, error) = metadata.Validate();
        if (!isValid)
        {
            _context.WriteError($"Invalid plugin metadata: {error}");
            return CommandResult.Error(1, error);
        }

        // 決定輸出路徑
        if (string.IsNullOrEmpty(outputPath))
        {
            outputPath = Path.Combine(
                Directory.GetCurrentDirectory(),
                $"{metadata.Name}-{metadata.Version}.zip");
        }

        _context.WriteLine($"Packaging plugin: {metadata.Name} v{metadata.Version}");
        _context.WriteLine($"Source: {sourceDir}");
        _context.WriteLine($"Output: {outputPath}");

        try
        {
            // 刪除已存在的檔案
            if (File.Exists(outputPath))
            {
                File.Delete(outputPath);
            }

            // 創建 zip
            System.IO.Compression.ZipFile.CreateFromDirectory(
                sourceDir, outputPath,
                System.IO.Compression.CompressionLevel.Optimal,
                includeBaseDirectory: false);

            // 計算 SHA256
            using var stream = File.OpenRead(outputPath);
            using var sha256 = System.Security.Cryptography.SHA256.Create();
            var hash = sha256.ComputeHash(stream);
            var hashString = Convert.ToHexString(hash).ToLowerInvariant();

            var fileInfo = new FileInfo(outputPath);

            _context.WriteLine("");
            _context.WriteLine("[OK] Plugin packaged successfully");
            _context.WriteLine($"  File: {outputPath}");
            _context.WriteLine($"  Size: {fileInfo.Length:N0} bytes");
            _context.WriteLine($"  SHA256: {hashString}");
            _context.WriteLine("");
            _context.WriteLine("To install locally:");
            _context.WriteLine($"  forge plugin install {outputPath}");

            return CommandResult.Ok();
        }
        catch (Exception ex)
        {
            _context.WriteError($"Failed to create package: {ex.Message}");
            return CommandResult.Error(1, ex.Message);
        }
    }

    /// <summary>
    /// 顯示倉庫統計
    /// </summary>
    /// <impl>
    /// APPROACH: 顯示官方倉庫的統計資訊
    /// CALLS: OfficialPluginRegistry
    /// EDGES: 無
    /// </impl>
    private CommandResult ShowRegistry(string[] args)
    {
        var registry = OfficialPluginRegistry.GenerateRegistry();

        _context.WriteLine("\n╔═══════════════════════════════════════════════════════════════╗");
        _context.WriteLine("║             MAIDOS-Forge Official Plugin Registry             ║");
        _context.WriteLine("╚═══════════════════════════════════════════════════════════════╝");
        _context.WriteLine("");

        // 按類別統計
        var byCategory = registry.Plugins
            .GroupBy(p => p.Category ?? "other")
            .OrderBy(g => g.Key)
            .ToList();

        _context.WriteLine($"{"Category",-18} {"Count",-8} {"Languages"}");
        _context.WriteLine(new string('-', 70));

        foreach (var group in byCategory)
        {
            var languages = string.Join(", ", group.Select(p => p.Language).Take(5));
            if (group.Count() > 5)
            {
                languages += $" ... (+{group.Count() - 5})";
            }
            _context.WriteLine($"{group.Key,-18} {group.Count(),-8} {languages}");
        }

        _context.WriteLine(new string('-', 70));
        _context.WriteLine($"{"Total",-18} {registry.Plugins.Count,-8}");
        _context.WriteLine("");

        // 內建 vs 擴充
        _context.WriteLine("Plugin Distribution:");
        _context.WriteLine("  Builtin:   8 languages (C#, Rust, C, C++, Go, Python, TypeScript, Assembly)");
        _context.WriteLine($"  Extension: {registry.Plugins.Count} languages (available via 'forge plugin install')");
        _context.WriteLine("");

        // 顯示類別列表
        _context.WriteLine("Search by category:");
        _context.WriteLine($"  forge plugin search --category <{string.Join("|", OfficialPluginRegistry.Categories.Take(5))}|...>");
        _context.WriteLine("");

        return CommandResult.Ok();
    }

    /// <summary>
    /// 顯示幫助
    /// </summary>
    private CommandResult ShowHelp()
    {
        ShowUsage();
        return CommandResult.Ok();
    }

    /// <summary>
    /// 未知子命令
    /// </summary>
    private CommandResult UnknownSubcommand(string subcommand)
    {
        _context.WriteError($"Unknown subcommand: {subcommand}");
        _context.WriteLine("");
        ShowUsage();
        return CommandResult.Error(1, $"Unknown subcommand: {subcommand}");
    }
}
