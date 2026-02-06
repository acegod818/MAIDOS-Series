// MAIDOS-Forge Language Plugin Factory
// UEP v1.7C - 70+ Languages Full Coverage
// M7 Hot-Pluggable Plugin System

namespace Forge.Core.Plugin;

/// <summary>
/// 語言插件工廠 - 為所有 UEP v1.7C 收錄語言創建插件
/// </summary>
/// <impl>
/// APPROACH: 從 LanguageDefinitions 創建 GenericLanguagePlugin 實例
/// CALLS: LanguageDefinitions.GetAllLanguages(), GenericLanguagePlugin ctor
/// EDGES: 無語言定義時返回空列表
/// </impl>
public static class LanguagePluginFactory
{
    private static readonly Dictionary<string, GenericLanguagePlugin> _cache = new();
    private static readonly object _lock = new();

    /// <summary>
    /// 取得所有語言插件 (70+ 種)
    /// </summary>
    /// <returns>所有語言的插件實例</returns>
    public static IReadOnlyList<ILanguagePlugin> GetAllPlugins()
    {
        EnsureInitialized();
        lock (_lock)
        {
            return _cache.Values.Cast<ILanguagePlugin>().ToList();
        }
    }

    /// <summary>
    /// 依語言 ID 取得插件
    /// </summary>
    /// <param name="languageId">語言 ID (c, rust, python...)</param>
    /// <returns>語言插件，若不存在則返回 null</returns>
    public static ILanguagePlugin? GetPlugin(string languageId)
    {
        EnsureInitialized();
        lock (_lock)
        {
            return _cache.TryGetValue(languageId, out var plugin) ? plugin : null;
        }
    }

    /// <summary>
    /// 依類別取得插件
    /// </summary>
    /// <param name="category">語言類別</param>
    /// <returns>該類別的所有語言插件</returns>
    public static IReadOnlyList<ILanguagePlugin> GetByCategory(LanguageCategory category)
    {
        EnsureInitialized();
        lock (_lock)
        {
            return _cache.Values
                .Where(p => p.Definition.Category == category)
                .Cast<ILanguagePlugin>()
                .ToList();
        }
    }

    /// <summary>
    /// 取得內建語言插件 (C, C#, Rust, Python)
    /// </summary>
    public static IReadOnlyList<ILanguagePlugin> GetBuiltinPlugins()
    {
        EnsureInitialized();
        lock (_lock)
        {
            return _cache.Values
                .Where(p => p.Definition.IsBuiltin)
                .Cast<ILanguagePlugin>()
                .ToList();
        }
    }

    /// <summary>
    /// 依檔案擴展名取得插件
    /// </summary>
    /// <param name="extension">檔案擴展名 (含 dot, 如 .rs)</param>
    /// <returns>處理該擴展名的語言插件</returns>
    public static ILanguagePlugin? GetByExtension(string extension)
    {
        EnsureInitialized();
        lock (_lock)
        {
            return _cache.Values
                .FirstOrDefault(p => p.Definition.Extensions
                    .Any(e => e.Equals(extension, StringComparison.OrdinalIgnoreCase)));
        }
    }

    /// <summary>
    /// 取得插件統計
    /// </summary>
    public static PluginStatistics GetStatistics()
    {
        EnsureInitialized();
        lock (_lock)
        {
            var byCategory = _cache.Values
                .GroupBy(p => p.Definition.Category)
                .ToDictionary(g => g.Key, g => g.Count());

            return new PluginStatistics
            {
                TotalCount = _cache.Count,
                BuiltinCount = _cache.Values.Count(p => p.Definition.IsBuiltin),
                ByCategory = byCategory,
                Languages = _cache.Keys.ToList()
            };
        }
    }

    /// <summary>
    /// 確保初始化
    /// </summary>
    private static void EnsureInitialized()
    {
        if (_cache.Count > 0) return;

        lock (_lock)
        {
            if (_cache.Count > 0) return;

            var languages = LanguageDefinitions.GetAllLanguages();
            foreach (var lang in languages)
            {
                var plugin = new GenericLanguagePlugin(lang);
                _cache[lang.Id] = plugin;
            }
        }
    }

    /// <summary>
    /// 重新載入所有插件 (用於熱更新)
    /// </summary>
    public static void Reload()
    {
        lock (_lock)
        {
            _cache.Clear();
        }
        EnsureInitialized();
    }
}

/// <summary>
/// 擴展 GenericLanguagePlugin 以暴露 Definition
/// </summary>
public sealed partial class GenericLanguagePlugin
{
    /// <summary>
    /// 取得語言定義
    /// </summary>
    public LanguageDefinition Definition => _definition;
}

/// <summary>
/// 插件統計
/// </summary>
public sealed record PluginStatistics
{
    /// <summary>總插件數</summary>
    public int TotalCount { get; init; }

    /// <summary>內建插件數</summary>
    public int BuiltinCount { get; init; }

    /// <summary>按類別統計</summary>
    public IReadOnlyDictionary<LanguageCategory, int> ByCategory { get; init; }
        = new Dictionary<LanguageCategory, int>();

    /// <summary>所有語言 ID</summary>
    public IReadOnlyList<string> Languages { get; init; } = Array.Empty<string>();

    /// <summary>
    /// 格式化輸出
    /// </summary>
    public override string ToString()
    {
        var sb = new System.Text.StringBuilder();
        sb.AppendLine($"MAIDOS-Forge Language Plugins: {TotalCount} languages");
        sb.AppendLine($"  Builtin: {BuiltinCount}");
        sb.AppendLine("  By Category:");

        foreach (var (category, count) in ByCategory.OrderByDescending(kv => kv.Value))
        {
            sb.AppendLine($"    {category}: {count}");
        }

        return sb.ToString();
    }
}

/// <summary>
/// 語言插件擴展方法
/// </summary>
public static class LanguagePluginExtensions
{
    /// <summary>
    /// 列印所有語言插件資訊
    /// </summary>
    public static string ToDetailedString(this IReadOnlyList<ILanguagePlugin> plugins)
    {
        var sb = new System.Text.StringBuilder();
        sb.AppendLine($"Total: {plugins.Count} language plugins");
        sb.AppendLine();

        var grouped = plugins
            .Cast<GenericLanguagePlugin>()
            .GroupBy(p => p.Definition.Category)
            .OrderBy(g => g.Key);

        foreach (var group in grouped)
        {
            sb.AppendLine($"═══ {group.Key} ({group.Count()}) ═══");
            foreach (var plugin in group.OrderBy(p => p.Definition.Name))
            {
                var def = plugin.Definition;
                sb.AppendLine($"  [{def.Id}] {def.DisplayName}");
                sb.AppendLine($"      Extensions: {string.Join(", ", def.Extensions)}");
                sb.AppendLine($"      Toolchains: {string.Join(", ", def.Toolchains)}");
                if (def.IsBuiltin) sb.AppendLine("      [BUILTIN]");
            }
            sb.AppendLine();
        }

        return sb.ToString();
    }
}
