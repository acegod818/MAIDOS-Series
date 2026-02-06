// MAIDOS-Forge Plugin Metadata Models
// UEP v1.7C Compliant - Zero Technical Debt
// M7 Hot-Pluggable Plugin System

using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Forge.Core.Plugin;

public sealed class PluginMetadata
{
    [JsonPropertyName("name")]
    public string Name { get; init; } = string.Empty;

    [JsonPropertyName("version")]
    public string Version { get; init; } = "1.0.0";

    [JsonPropertyName("language")]
    public string Language { get; init; } = string.Empty;

    [JsonPropertyName("displayName")]
    public string DisplayName { get; init; } = string.Empty;

    [JsonPropertyName("description")]
    public string Description { get; init; } = string.Empty;

    [JsonPropertyName("author")]
    public string Author { get; init; } = string.Empty;

    [JsonPropertyName("extensions")]
    public IReadOnlyList<string> Extensions { get; init; } = Array.Empty<string>();

    [JsonPropertyName("toolchains")]
    public IReadOnlyList<string> Toolchains { get; init; } = Array.Empty<string>();

    [JsonPropertyName("forgeVersion")]
    public string ForgeVersion { get; init; } = ">=0.7.0";

    [JsonPropertyName("entry")]
    public string Entry { get; init; } = string.Empty;

    [JsonPropertyName("pluginClass")]
    public string PluginClass { get; init; } = string.Empty;

    [JsonPropertyName("builtin")]
    public bool IsBuiltin { get; init; }

    [JsonIgnore]
    public string PluginPath { get; set; } = string.Empty;

    public static PluginMetadata? LoadFromFile(string path)
    {
        if (!File.Exists(path)) return null;
        try {
            var json = File.ReadAllText(path);
            var metadata = JsonSerializer.Deserialize<PluginMetadata>(json, JsonOptions);
            if (metadata != null) metadata.PluginPath = Path.GetDirectoryName(path) ?? string.Empty;
            return metadata;
        } catch { return null; }
    }

    public static PluginMetadata? LoadFromJson(string json)
    {
        try {
            return JsonSerializer.Deserialize<PluginMetadata>(json, JsonOptions);
        } catch { return null; }
    }

    public string ToJson() => JsonSerializer.Serialize(this, JsonOptions);

    public (bool IsValid, string Error) Validate()
    {
        if (string.IsNullOrEmpty(Name)) return (false, "Name is required");
        if (string.IsNullOrEmpty(Language)) return (false, "Language is required");
        if (string.IsNullOrEmpty(Entry)) return (false, "Entry DLL is required");
        if (string.IsNullOrEmpty(PluginClass)) return (false, "Plugin class is required");
        return (true, string.Empty);
    }

    private static readonly JsonSerializerOptions JsonOptions = new() { PropertyNamingPolicy = JsonNamingPolicy.CamelCase, WriteIndented = true };
}

public sealed class PluginRegistry
{
    [JsonPropertyName("version")]
    public string Version { get; init; } = "1.0";

    [JsonPropertyName("updated")]
    public DateTime Updated { get; init; } = DateTime.UtcNow;

    [JsonPropertyName("plugins")]
    public IReadOnlyList<PluginRegistryEntry> Plugins { get; init; } = Array.Empty<PluginRegistryEntry>();

    public static PluginRegistry? LoadFromFile(string path)
    {
        if (!File.Exists(path)) return null;
        try {
            var json = File.ReadAllText(path);
            return JsonSerializer.Deserialize<PluginRegistry>(json, new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.CamelCase });
        } catch { return null; }
    }
}

public sealed class PluginRegistryEntry
{
    [JsonPropertyName("name")]
    public string Name { get; init; } = string.Empty;

    [JsonPropertyName("language")]
    public string Language { get; init; } = string.Empty;

    [JsonPropertyName("version")]
    public string Version { get; init; } = "1.0.0";

    [JsonPropertyName("category")]
    public string? Category { get; init; }

    [JsonPropertyName("description")]
    public string? Description { get; init; }

    [JsonPropertyName("author")]
    public string? Author { get; init; }

    [JsonPropertyName("downloadUrl")]
    public string? DownloadUrl { get; init; }

    [JsonPropertyName("sha256")]
    public string? Sha256 { get; init; }

    [JsonPropertyName("ffiMethod")]
    public string? FfiMethod { get; init; }
}

public sealed class LoadedPlugin
{
    public PluginMetadata Metadata { get; }
    public ILanguagePlugin? Instance { get; }
    public object? LoadContext { get; } // !U�N�o
    public bool IsLoaded => Instance is not null;
    public string? LoadError { get; }

    private LoadedPlugin(PluginMetadata metadata, ILanguagePlugin? instance, string? loadError)
    {
        Metadata = metadata;
        Instance = instance;
        LoadError = loadError;
    }

    public static LoadedPlugin Success(PluginMetadata metadata, ILanguagePlugin instance, object? loadContext)
    {
        var lp = new LoadedPlugin(metadata, instance, null);
        // dU�ԋ|-�U LoadContext
        return lp;
    }

    public static LoadedPlugin Failure(PluginMetadata metadata, string error)
        => new LoadedPlugin(metadata, null, error);
}