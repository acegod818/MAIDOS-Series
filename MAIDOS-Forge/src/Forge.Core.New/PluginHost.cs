// MAIDOS-Forge Plugin Host
// UEP v1.7B Compliant - Zero Technical Debt

namespace Forge.Core.Plugin;

/// <summary>
/// 插件註冊結果
/// </summary>
/// <impl>
/// APPROACH: 封裝插件載入/註冊結果
/// CALLS: N/A (純資料)
/// EDGES: IsSuccess 為 false 時 Error 非空
/// </impl>
public readonly struct PluginRegistrationResult
{
    public bool IsSuccess { get; }
    public string Error { get; }
    public PluginCapabilities? Capabilities { get; }

    private PluginRegistrationResult(bool isSuccess, string error, PluginCapabilities? capabilities)
    {
        IsSuccess = isSuccess;
        Error = error;
        Capabilities = capabilities;
    }

    public static PluginRegistrationResult Success(PluginCapabilities capabilities)
        => new(true, string.Empty, capabilities);

    public static PluginRegistrationResult Failure(string error)
        => new(false, error, null);
}

/// <summary>
/// 插件主機 - 負責載入與管理語言插件
/// </summary>
/// <impl>
/// APPROACH: 維護語言名稱到插件實例的映射，提供查詢與驗證功能
/// CALLS: ILanguagePlugin 實例
/// EDGES: 未註冊的語言返回 null, 重複註冊覆蓋舊插件
/// </impl>
public sealed class PluginHost
{
    private readonly Dictionary<string, ILanguagePlugin> _plugins = new(StringComparer.OrdinalIgnoreCase);
    private readonly List<string> _registrationOrder = new();

    /// <summary>
    /// 已註冊的語言列表
    /// </summary>
    public IReadOnlyList<string> RegisteredLanguages => _registrationOrder;

    /// <summary>
    /// 註冊內建插件
    /// </summary>
    /// <impl>
    /// APPROACH: 建立並註冊所有內建語言插件
    /// CALLS: RegisterPlugin(), CSharpPlugin, RustPlugin, CPlugin
    /// EDGES: 任一插件註冊失敗記錄錯誤但繼續
    /// </impl>
    public void RegisterBuiltinPlugins()
    {
        // 系統語言
        RegisterPlugin(new CSharpPlugin());
        RegisterPlugin(new RustPlugin());
        RegisterPlugin(new CPlugin());
        RegisterPlugin(new CppPlugin());
        
        // 應用語言
        RegisterPlugin(new GoPlugin());
        RegisterPlugin(new PythonPlugin());
        RegisterPlugin(new TypeScriptPlugin());
        
        // 底層語言
        RegisterPlugin(new AsmPlugin());
    }

    /// <summary>
    /// 手動註冊插件 (供外部使用)
    /// </summary>
    public void Register(ILanguagePlugin plugin)
    {
        RegisterPlugin(plugin);
    }

    /// <summary>
    /// 註冊單一插件
    /// </summary>
    /// <impl>
    /// APPROACH: 取得插件能力，以語言名稱為 key 存入字典
    /// CALLS: ILanguagePlugin.GetCapabilities()
    /// EDGES: 重複註冊覆蓋舊插件，null 插件返回失敗
    /// </impl>
    public PluginRegistrationResult RegisterPlugin(ILanguagePlugin plugin)
    {
        if (plugin is null)
        {
            return PluginRegistrationResult.Failure("Plugin cannot be null");
        }

        var capabilities = plugin.GetCapabilities();
        if (string.IsNullOrEmpty(capabilities.LanguageName))
        {
            return PluginRegistrationResult.Failure("Plugin language name cannot be empty");
        }

        var languageName = capabilities.LanguageName.ToLowerInvariant();

        if (!_plugins.ContainsKey(languageName))
        {
            _registrationOrder.Add(languageName);
        }

        _plugins[languageName] = plugin;
        return PluginRegistrationResult.Success(capabilities);
    }

    /// <summary>
    /// 取得指定語言的插件
    /// </summary>
    /// <impl>
    /// APPROACH: 字典查找
    /// CALLS: Dictionary.TryGetValue()
    /// EDGES: 未註冊返回 null
    /// </impl>
    public ILanguagePlugin? GetPlugin(string language)
    {
        return _plugins.TryGetValue(language, out var plugin) ? plugin : null;
    }

    /// <summary>
    /// 檢查語言是否已註冊
    /// </summary>
    /// <impl>
    /// APPROACH: 字典 ContainsKey
    /// CALLS: Dictionary.ContainsKey()
    /// EDGES: N/A
    /// </impl>
    public bool HasPlugin(string language)
    {
        return _plugins.ContainsKey(language);
    }

    /// <summary>
    /// 取得所有已註冊插件的能力
    /// </summary>
    /// <impl>
    /// APPROACH: 遍歷所有插件取得能力
    /// CALLS: ILanguagePlugin.GetCapabilities()
    /// EDGES: 空字典返回空列表
    /// </impl>
    public IReadOnlyList<PluginCapabilities> GetAllCapabilities()
    {
        return _plugins.Values
            .Select(p => p.GetCapabilities())
            .ToList();
    }

    /// <summary>
    /// 驗證所有插件的工具鏈
    /// </summary>
    /// <impl>
    /// APPROACH: 並行驗證所有插件工具鏈
    /// CALLS: ILanguagePlugin.ValidateToolchainAsync()
    /// EDGES: 返回所有驗證結果，包含失敗的
    /// </impl>
    public async Task<IReadOnlyDictionary<string, (bool Available, string Message)>> 
        ValidateAllToolchainsAsync(CancellationToken ct = default)
    {
        var results = new Dictionary<string, (bool, string)>(StringComparer.OrdinalIgnoreCase);

        foreach (var (language, plugin) in _plugins)
        {
            var result = await plugin.ValidateToolchainAsync(ct);
            results[language] = result;
        }

        return results;
    }

    /// <summary>
    /// 根據檔案副檔名取得對應插件
    /// </summary>
    /// <impl>
    /// APPROACH: 遍歷所有插件，檢查支援的副檔名
    /// CALLS: PluginCapabilities.SupportedExtensions
    /// EDGES: 無匹配返回 null
    /// </impl>
    public ILanguagePlugin? GetPluginByExtension(string extension)
    {
        extension = extension.TrimStart('.').ToLowerInvariant();

        foreach (var plugin in _plugins.Values)
        {
            var capabilities = plugin.GetCapabilities();
            if (capabilities.SupportedExtensions.Any(ext => 
                ext.TrimStart('.').Equals(extension, StringComparison.OrdinalIgnoreCase)))
            {
                return plugin;
            }
        }

        return null;
    }
}
