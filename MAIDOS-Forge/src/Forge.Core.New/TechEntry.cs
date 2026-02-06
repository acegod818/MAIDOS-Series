// MAIDOS-Forge Technology Entry Definition
// UEP v1.7C - 1,660+ Technologies (Refactored for 95 Languages)

using System;
using System.Collections.Generic;
using System.Linq;

namespace Forge.Core.Plugin.TechIndex;

public sealed record TechEntry(
    string Id,
    string Name,
    string DisplayName,
    TechCategory Category,
    string Description
)
{
    public string PluginId => $"forge.plugin.{Id}";
    public string Version => "1.0.0";
    public string CategoryName => Category.ToString().ToLowerInvariant();
}

public enum TechCategory
{
    Paradigm, Architecture, Pattern, Performance, Security, Testing,
    Distributed, DataProcessing, AIML, Frontend, Observability,
    DevOps, TypeSystem, Compiler, Networking, Database, Embedded,
    GameGraphics, Blockchain, Space, Bio, Neuromorphic, IIoT, BCI,
    Methodology, Mobile, XR, AudioVideo, Robotics, Geospatial,
    FinTech, MusicAudio, Accessibility, LowCode, Quantum, AIRobotics,
    Green, Language
}

public static class CompleteTechIndex
{
    /// <summary>
    /// Get all technology entries
    /// </summary>
    /// <impl>
    /// APPROACH: Currently returns language definitions only; will integrate 1,660+ tech index in the future
    /// </impl>
    public static IReadOnlyList<TechEntry> GetAllTechnologies()
    {
        var all = new List<TechEntry>();

        // Get language entries from LanguageDefinitions
        var languages = LanguageDefinitions.GetAllLanguages();
        foreach (var lang in languages)
        {
            all.Add(new TechEntry(
                lang.Id,
                lang.Name,
                lang.DisplayName,
                TechCategory.Language,
                lang.Description
            ));
        }

        return all;
    }

    public static IReadOnlyList<TechEntry> GetByCategory(TechCategory category)
        => GetAllTechnologies().Where(t => t.Category == category).ToList();

    public static TechEntry? GetById(string id)
        => GetAllTechnologies().FirstOrDefault(t => t.Id == id);
}

public sealed record TechIndexStats
{
    public int TotalCount { get; init; }
    public IReadOnlyDictionary<TechCategory, int> ByCategory { get; init; } = new Dictionary<TechCategory, int>();
}
