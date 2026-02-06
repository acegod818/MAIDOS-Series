// MAIDOS-Forge Incremental Build Cache
// UEP v1.7B Compliant - Zero Technical Debt

using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Forge.Core.Cache;

/// <summary>
/// 模組快取條目
/// </summary>
/// <impl>
/// APPROACH: 記錄模組編譯狀態和檔案 hash
/// CALLS: N/A (純資料)
/// EDGES: N/A
/// </impl>
public sealed class ModuleCacheEntry
{
    [JsonPropertyName("module_name")]
    public string ModuleName { get; set; } = string.Empty;

    [JsonPropertyName("language")]
    public string Language { get; set; } = string.Empty;

    [JsonPropertyName("source_hash")]
    public string SourceHash { get; set; } = string.Empty;

    [JsonPropertyName("config_hash")]
    public string ConfigHash { get; set; } = string.Empty;

    [JsonPropertyName("dependencies_hash")]
    public string DependenciesHash { get; set; } = string.Empty;

    [JsonPropertyName("artifact_paths")]
    public List<string> ArtifactPaths { get; set; } = new();

    [JsonPropertyName("compiled_at")]
    public DateTime CompiledAt { get; set; }

    [JsonPropertyName("profile")]
    public string Profile { get; set; } = "release";
}

/// <summary>
/// 專案編譯快取
/// </summary>
/// <impl>
/// APPROACH: 儲存所有模組的快取條目
/// CALLS: N/A (純資料)
/// EDGES: N/A
/// </impl>
public sealed class BuildCache
{
    [JsonPropertyName("version")]
    public string Version { get; set; } = "1.0";

    [JsonPropertyName("project_name")]
    public string ProjectName { get; set; } = string.Empty;

    [JsonPropertyName("last_build")]
    public DateTime LastBuild { get; set; }

    [JsonPropertyName("modules")]
    public Dictionary<string, ModuleCacheEntry> Modules { get; set; } = new();
}

/// <summary>
/// 增量編譯檢查結果
/// </summary>
public sealed class IncrementalCheckResult
{
    public string ModuleName { get; init; } = string.Empty;
    public bool NeedsRebuild { get; init; }
    public string Reason { get; init; } = string.Empty;
    public ModuleCacheEntry? CacheEntry { get; init; }
}

/// <summary>
/// 增量編譯管理器
/// </summary>
/// <impl>
/// APPROACH: 比對檔案 hash 判斷是否需要重新編譯
/// CALLS: SHA256, File I/O
/// EDGES: 快取檔案不存在時視為全部需要重編
/// </impl>
public sealed class IncrementalBuildManager
{
    private const string CacheFileName = ".forge-cache.json";
    private readonly string _cacheDir;
    private BuildCache? _cache;

    public IncrementalBuildManager(string projectRoot)
    {
        _cacheDir = Path.Combine(projectRoot, ".forge");
    }

    /// <summary>
    /// 載入快取
    /// </summary>
    /// <impl>
    /// APPROACH: 從 .forge-cache.json 讀取快取
    /// CALLS: JsonSerializer.Deserialize()
    /// EDGES: 檔案不存在返回空快取
    /// </impl>
    public BuildCache LoadCache()
    {
        var cachePath = Path.Combine(_cacheDir, CacheFileName);

        if (!File.Exists(cachePath))
        {
            _cache = new BuildCache();
            return _cache;
        }

        try
        {
            var json = File.ReadAllText(cachePath);
            _cache = JsonSerializer.Deserialize<BuildCache>(json) ?? new BuildCache();
        }
        catch
        {
            _cache = new BuildCache();
        }

        return _cache;
    }

    /// <summary>
    /// 儲存快取
    /// </summary>
    /// <impl>
    /// APPROACH: 序列化快取到 .forge-cache.json
    /// CALLS: JsonSerializer.Serialize()
    /// EDGES: 目錄不存在時建立
    /// </impl>
    public void SaveCache()
    {
        if (_cache is null) return;

        Directory.CreateDirectory(_cacheDir);
        var cachePath = Path.Combine(_cacheDir, CacheFileName);

        var options = new JsonSerializerOptions { WriteIndented = true };
        var json = JsonSerializer.Serialize(_cache, options);
        File.WriteAllText(cachePath, json);
    }

    /// <summary>
    /// 檢查模組是否需要重新編譯
    /// </summary>
    /// <impl>
    /// APPROACH: 比對源碼 hash、配置 hash、依賴 hash
    /// CALLS: ComputeSourceHash(), ComputeConfigHash()
    /// EDGES: 任一 hash 不匹配或產物不存在則需要重編
    /// </impl>
    public IncrementalCheckResult CheckModule(
        string modulePath,
        string moduleName,
        string language,
        string profile,
        IReadOnlyList<string> dependencies)
    {
        if (_cache is null)
        {
            LoadCache();
        }

        // 計算當前 hash
        var sourceHash = ComputeSourceHash(modulePath, language);
        var configHash = ComputeConfigHash(modulePath);
        var depsHash = ComputeDependenciesHash(dependencies);

        // 檢查快取
        var cacheKey = $"{moduleName}:{profile}";
        if (!_cache!.Modules.TryGetValue(cacheKey, out var entry))
        {
            return new IncrementalCheckResult
            {
                ModuleName = moduleName,
                NeedsRebuild = true,
                Reason = "No cache entry"
            };
        }

        // 比對 hash
        if (entry.SourceHash != sourceHash)
        {
            return new IncrementalCheckResult
            {
                ModuleName = moduleName,
                NeedsRebuild = true,
                Reason = "Source changed",
                CacheEntry = entry
            };
        }

        if (entry.ConfigHash != configHash)
        {
            return new IncrementalCheckResult
            {
                ModuleName = moduleName,
                NeedsRebuild = true,
                Reason = "Config changed",
                CacheEntry = entry
            };
        }

        if (entry.DependenciesHash != depsHash)
        {
            return new IncrementalCheckResult
            {
                ModuleName = moduleName,
                NeedsRebuild = true,
                Reason = "Dependencies changed",
                CacheEntry = entry
            };
        }

        // 檢查產物是否存在
        foreach (var artifact in entry.ArtifactPaths)
        {
            if (!File.Exists(artifact))
            {
                return new IncrementalCheckResult
                {
                    ModuleName = moduleName,
                    NeedsRebuild = true,
                    Reason = $"Artifact missing: {Path.GetFileName(artifact)}",
                    CacheEntry = entry
                };
            }
        }

        return new IncrementalCheckResult
        {
            ModuleName = moduleName,
            NeedsRebuild = false,
            Reason = "Up to date",
            CacheEntry = entry
        };
    }

    /// <summary>
    /// 更新模組快取
    /// </summary>
    /// <impl>
    /// APPROACH: 記錄編譯後的 hash 和產物路徑
    /// CALLS: ComputeSourceHash(), ComputeConfigHash()
    /// EDGES: N/A
    /// </impl>
    public void UpdateModuleCache(
        string modulePath,
        string moduleName,
        string language,
        string profile,
        IReadOnlyList<string> dependencies,
        IReadOnlyList<string> artifacts)
    {
        if (_cache is null)
        {
            LoadCache();
        }

        var cacheKey = $"{moduleName}:{profile}";
        var entry = new ModuleCacheEntry
        {
            ModuleName = moduleName,
            Language = language,
            SourceHash = ComputeSourceHash(modulePath, language),
            ConfigHash = ComputeConfigHash(modulePath),
            DependenciesHash = ComputeDependenciesHash(dependencies),
            ArtifactPaths = artifacts.ToList(),
            CompiledAt = DateTime.UtcNow,
            Profile = profile
        };

        _cache!.Modules[cacheKey] = entry;
        _cache.LastBuild = DateTime.UtcNow;
    }

    /// <summary>
    /// 清除快取
    /// </summary>
    public void ClearCache()
    {
        _cache = new BuildCache();
        var cachePath = Path.Combine(_cacheDir, CacheFileName);
        if (File.Exists(cachePath))
        {
            File.Delete(cachePath);
        }
    }

    /// <summary>
    /// 計算源碼 hash
    /// </summary>
    /// <impl>
    /// APPROACH: 遍歷所有源檔案計算 SHA256
    /// CALLS: SHA256.HashData()
    /// EDGES: 空目錄返回空 hash
    /// </impl>
    private static string ComputeSourceHash(string modulePath, string language)
    {
        var srcDir = Path.Combine(modulePath, "src");
        if (!Directory.Exists(srcDir))
        {
            srcDir = modulePath;
        }

        var extensions = language.ToLowerInvariant() switch
        {
            "csharp" or "c#" => new[] { "*.cs" },
            "rust" => new[] { "*.rs" },
            "c" => new[] { "*.c", "*.h" },
            _ => new[] { "*.*" }
        };

        var hasher = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
        var fileCount = 0;

        foreach (var ext in extensions)
        {
            foreach (var file in Directory.GetFiles(srcDir, ext, SearchOption.AllDirectories))
            {
                var content = File.ReadAllBytes(file);
                hasher.AppendData(content);
                hasher.AppendData(Encoding.UTF8.GetBytes(file));
                fileCount++;
            }
        }

        if (fileCount == 0)
        {
            return string.Empty;
        }

        var hash = hasher.GetHashAndReset();
        return Convert.ToHexString(hash).ToLowerInvariant();
    }

    /// <summary>
    /// 計算配置 hash
    /// </summary>
    /// <impl>
    /// APPROACH: 讀取 module.json 並計算 SHA256
    /// CALLS: SHA256.HashData()
    /// EDGES: 檔案不存在返回空 hash
    /// </impl>
    private static string ComputeConfigHash(string modulePath)
    {
        var configPath = Path.Combine(modulePath, "module.json");
        if (!File.Exists(configPath))
        {
            return string.Empty;
        }

        var content = File.ReadAllBytes(configPath);
        var hash = SHA256.HashData(content);
        return Convert.ToHexString(hash).ToLowerInvariant();
    }

    /// <summary>
    /// 計算依賴 hash
    /// </summary>
    /// <impl>
    /// APPROACH: 將依賴列表排序後計算 SHA256
    /// CALLS: SHA256.HashData()
    /// EDGES: 空依賴返回固定 hash
    /// </impl>
    private static string ComputeDependenciesHash(IReadOnlyList<string> dependencies)
    {
        if (dependencies.Count == 0)
        {
            return "empty";
        }

        var sorted = dependencies.OrderBy(d => d).ToList();
        var combined = string.Join("|", sorted);
        var hash = SHA256.HashData(Encoding.UTF8.GetBytes(combined));
        return Convert.ToHexString(hash).ToLowerInvariant();
    }
}
