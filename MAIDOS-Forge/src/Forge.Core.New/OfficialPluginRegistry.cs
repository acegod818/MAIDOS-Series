// MAIDOS-Forge Official Plugin Registry
// UEP v1.7C - 95 Languages (Tier 1-3)
// 100% SPEC v2.2 Compliant - Exactly 95 Languages

using System.Collections.Generic;
using System.Linq;

namespace Forge.Core.Plugin;

/// <summary>
/// Official plugin registry - Manages metadata for 95 language plugins
/// </summary>
public static class OfficialPluginRegistry
{
    public static readonly string[] Categories = {
        "system", "managed", "scripting", "web", "functional",
        "mobile", "concurrent", "scientific", "hardware",
        "blockchain", "verification", "modern", "configuration",
        "logic", "markup", "other"
    };

    /// <summary>
    /// Generate the complete registry
    /// </summary>
    public static PluginRegistry GenerateRegistry()
    {
        var plugins = new List<PluginRegistryEntry>();

        // Automatically generate registry entries for 95 languages from LanguageDefinitions
        var allLanguages = LanguageDefinitions.GetAllLanguages();

        // Filter out builtin languages (8), the rest become extension plugins (87)
        foreach (var lang in allLanguages.Where(l => !l.IsBuiltin))
        {
            plugins.Add(new PluginRegistryEntry
            {
                Name = $"forge.plugin.{lang.Id}",
                Language = lang.Id,
                Version = "1.0.0",
                Category = lang.Category.ToString().ToLowerInvariant(),
                Description = lang.Description,
                Author = "MAIDOS",
                FfiMethod = "c-abi"
            });
        }

        return new PluginRegistry
        {
            Version = "2.2",
            Updated = System.DateTime.UtcNow,
            Plugins = plugins.ToArray()
        };
    }

    public static IEnumerable<PluginRegistryEntry> Search(string query) =>
        GenerateRegistry().Plugins.Where(p =>
            p.Name.Contains(query, System.StringComparison.OrdinalIgnoreCase) ||
            p.Language.Contains(query, System.StringComparison.OrdinalIgnoreCase));

    public static IEnumerable<PluginRegistryEntry> GetByCategory(string category) =>
        GenerateRegistry().Plugins.Where(p => p.Category == category.ToLowerInvariant());
}
