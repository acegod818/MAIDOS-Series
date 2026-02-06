// MAIDOS-Forge Configuration Parser
// UEP v1.7B Compliant - Zero Technical Debt

using System.Text.Json;

namespace Forge.Core.Config;

/// <summary>
/// 配置解析結果
/// </summary>
/// <impl>
/// APPROACH: Result 模式封裝成功/失敗，避免異常作為控制流
/// CALLS: N/A (純資料)
/// EDGES: IsSuccess 與 Error 互斥，Value 僅在成功時有效
/// </impl>
public readonly struct ParseResult<T>
{
    public bool IsSuccess { get; }
    public T? Value { get; }
    public string Error { get; }

    private ParseResult(bool isSuccess, T? value, string error)
    {
        IsSuccess = isSuccess;
        Value = value;
        Error = error;
    }

    public static ParseResult<T> Success(T value) => new(true, value, string.Empty);
    public static ParseResult<T> Failure(string error) => new(false, default, error);
}

/// <summary>
/// 已驗證的專案配置（不可變）
/// </summary>
/// <impl>
/// APPROACH: 將解析後的配置封裝為不可變結構，包含專案根目錄
/// CALLS: N/A (純資料)
/// EDGES: 所有欄位在建構時驗證完成
/// </impl>
public sealed class ValidatedForgeConfig
{
    public string ProjectRoot { get; }
    public ForgeConfig Config { get; }
    public IReadOnlyList<ValidatedModuleConfig> Modules { get; }

    public ValidatedForgeConfig(string projectRoot, ForgeConfig config, IReadOnlyList<ValidatedModuleConfig> modules)
    {
        ProjectRoot = projectRoot;
        Config = config;
        Modules = modules;
    }
}

/// <summary>
/// 已驗證的模組配置（不可變）
/// </summary>
/// <impl>
/// APPROACH: 將解析後的模組配置封裝為不可變結構
/// CALLS: N/A (純資料)
/// EDGES: ModulePath 為絕對路徑
/// </impl>
public sealed class ValidatedModuleConfig
{
    public string ModulePath { get; }
    public ModuleConfig Config { get; }

    public ValidatedModuleConfig(string modulePath, ModuleConfig config)
    {
        ModulePath = modulePath;
        Config = config;
    }
}

/// <summary>
/// 配置解析器 - 負責讀取與驗證 forge.json 和 module.json
/// </summary>
/// <impl>
/// APPROACH: 靜態方法組，從檔案系統讀取 JSON 並反序列化，驗證必填欄位
/// CALLS: File.ReadAllText(), JsonSerializer.Deserialize()
/// EDGES: 檔案不存在返回失敗, JSON 語法錯誤返回失敗, 必填欄位空返回失敗
/// </impl>
public static class ConfigParser
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNameCaseInsensitive = true,
        ReadCommentHandling = JsonCommentHandling.Skip,
        AllowTrailingCommas = true
    };

    /// <summary>
    /// 解析專案配置
    /// </summary>
    /// <impl>
    /// APPROACH: 讀取 forge.json，驗證必填欄位，遞迴解析所有模組
    /// CALLS: File.ReadAllText(), JsonSerializer.Deserialize(), ParseModuleConfig()
    /// EDGES: 專案目錄不存在返回失敗, forge.json 不存在返回失敗, name 空返回失敗
    /// </impl>
    public static ParseResult<ValidatedForgeConfig> ParseProject(string projectPath)
    {
        // 驗證專案目錄存在
        if (!Directory.Exists(projectPath))
        {
            return ParseResult<ValidatedForgeConfig>.Failure($"Project directory not found: {projectPath}");
        }

        var forgeJsonPath = Path.Combine(projectPath, "forge.json");
        
        // 驗證 forge.json 存在
        if (!File.Exists(forgeJsonPath))
        {
            return ParseResult<ValidatedForgeConfig>.Failure($"forge.json not found in: {projectPath}");
        }

        // 讀取並解析 forge.json
        string jsonContent;
        try
        {
            jsonContent = File.ReadAllText(forgeJsonPath);
        }
        catch (IOException ex)
        {
            return ParseResult<ValidatedForgeConfig>.Failure($"Failed to read forge.json: {ex.Message}");
        }

        ForgeConfig? forgeConfig;
        try
        {
            forgeConfig = JsonSerializer.Deserialize<ForgeConfig>(jsonContent, JsonOptions);
        }
        catch (JsonException ex)
        {
            return ParseResult<ValidatedForgeConfig>.Failure($"Invalid JSON in forge.json: {ex.Message}");
        }

        if (forgeConfig is null)
        {
            return ParseResult<ValidatedForgeConfig>.Failure("forge.json deserialized to null");
        }

        // 驗證必填欄位
        if (string.IsNullOrWhiteSpace(forgeConfig.Name))
        {
            return ParseResult<ValidatedForgeConfig>.Failure("forge.json: 'name' is required");
        }

        // 解析所有模組
        var modules = new List<ValidatedModuleConfig>();
        var modulesDir = Path.Combine(projectPath, "modules");

        if (forgeConfig.Modules.Count > 0)
        {
            // 使用顯式列出的模組
            foreach (var moduleName in forgeConfig.Modules)
            {
                var modulePath = Path.Combine(modulesDir, moduleName);
                var moduleResult = ParseModuleConfig(modulePath);
                if (!moduleResult.IsSuccess)
                {
                    return ParseResult<ValidatedForgeConfig>.Failure($"Module '{moduleName}': {moduleResult.Error}");
                }
                modules.Add(moduleResult.Value!);
            }
        }
        else if (Directory.Exists(modulesDir))
        {
            // 自動發現 modules/ 下的所有子目錄
            foreach (var dir in Directory.GetDirectories(modulesDir))
            {
                var moduleJsonPath = Path.Combine(dir, "module.json");
                if (File.Exists(moduleJsonPath))
                {
                    var moduleResult = ParseModuleConfig(dir);
                    if (!moduleResult.IsSuccess)
                    {
                        return ParseResult<ValidatedForgeConfig>.Failure($"Module '{Path.GetFileName(dir)}': {moduleResult.Error}");
                    }
                    modules.Add(moduleResult.Value!);
                }
            }
        }

        return ParseResult<ValidatedForgeConfig>.Success(
            new ValidatedForgeConfig(projectPath, forgeConfig, modules));
    }

    /// <summary>
    /// 解析單一模組配置
    /// </summary>
    /// <impl>
    /// APPROACH: 讀取 module.json，驗證必填欄位
    /// CALLS: File.ReadAllText(), JsonSerializer.Deserialize()
    /// EDGES: 目錄不存在返回失敗, module.json 不存在返回失敗, name/language 空返回失敗
    /// </impl>
    public static ParseResult<ValidatedModuleConfig> ParseModuleConfig(string modulePath)
    {
        // 驗證模組目錄存在
        if (!Directory.Exists(modulePath))
        {
            return ParseResult<ValidatedModuleConfig>.Failure($"Module directory not found: {modulePath}");
        }

        var moduleJsonPath = Path.Combine(modulePath, "module.json");

        // 驗證 module.json 存在
        if (!File.Exists(moduleJsonPath))
        {
            return ParseResult<ValidatedModuleConfig>.Failure($"module.json not found in: {modulePath}");
        }

        // 讀取並解析 module.json
        string jsonContent;
        try
        {
            jsonContent = File.ReadAllText(moduleJsonPath);
        }
        catch (IOException ex)
        {
            return ParseResult<ValidatedModuleConfig>.Failure($"Failed to read module.json: {ex.Message}");
        }

        ModuleConfig? moduleConfig;
        try
        {
            moduleConfig = JsonSerializer.Deserialize<ModuleConfig>(jsonContent, JsonOptions);
        }
        catch (JsonException ex)
        {
            return ParseResult<ValidatedModuleConfig>.Failure($"Invalid JSON in module.json: {ex.Message}");
        }

        if (moduleConfig is null)
        {
            return ParseResult<ValidatedModuleConfig>.Failure("module.json deserialized to null");
        }

        // 驗證必填欄位
        if (string.IsNullOrWhiteSpace(moduleConfig.Name))
        {
            return ParseResult<ValidatedModuleConfig>.Failure("module.json: 'name' is required");
        }

        if (string.IsNullOrWhiteSpace(moduleConfig.Language))
        {
            return ParseResult<ValidatedModuleConfig>.Failure("module.json: 'language' is required");
        }

        // 驗證語言是支援的
        var supportedLanguages = new HashSet<string>(StringComparer.OrdinalIgnoreCase) 
        { 
            "csharp", "rust", "c", "cpp", "go", "python", "typescript", "asm",
            "java", "kotlin", "scala", "fsharp", "ruby", "php", "perl",
            "javascript", "dart", "haskell", "ocaml", "erlang", "elixir",
            "clojure", "swift", "objc", "pony", "julia", "r", "matlab",
            "mojo", "wolfram", "vhdl", "verilog", "systemverilog", "chisel",
            "solidity", "move", "vyper", "cairo", "coq", "lean", "agda",
            "idris", "dafny", "tlaplus", "nim", "crystal", "vlang", "carbon",
            "gleam", "roc", "moonbit", "unison", "cue", "pkl", "prolog",
            "sql", "datalog", "assembly", "forth", "factor", "faust", "smalltalk",
            "bash", "powershell", "awk", "tcl", "groovy", "coffeescript", "graphql",
            "purescript", "flix", "dhall", "jsonnet", "minizinc", "vale"
        };
        
        if (!supportedLanguages.Contains(moduleConfig.Language))
        {
            return ParseResult<ValidatedModuleConfig>.Failure(
                $"module.json: unsupported language '{moduleConfig.Language}'. " +
                $"Supported: csharp, rust, c, cpp, go, python, typescript, asm, and 79 more...");
        }

        return ParseResult<ValidatedModuleConfig>.Success(
            new ValidatedModuleConfig(modulePath, moduleConfig));
    }
}
