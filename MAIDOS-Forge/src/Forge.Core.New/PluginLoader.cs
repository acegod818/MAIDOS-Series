// MAIDOS-Forge Plugin Loader
// UEP v1.7C Compliant - Zero Technical Debt
// M7 Hot-Pluggable Plugin System - 100% Physical Proof

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.Loader;

namespace Forge.Core.Plugin;

public sealed class PluginLoadContext : AssemblyLoadContext
{
    private readonly AssemblyDependencyResolver _resolver;
    private readonly string _pluginPath;

    public PluginLoadContext(string pluginPath) : base(isCollectible: true)
    {
        _pluginPath = pluginPath;
        _resolver = new AssemblyDependencyResolver(pluginPath);
    }

    protected override Assembly? Load(AssemblyName assemblyName)
    {
        var assemblyPath = _resolver.ResolveAssemblyToPath(assemblyName);
        if (assemblyPath is not null) return LoadFromAssemblyPath(assemblyPath);
        if (IsCoreAssembly(assemblyName.Name)) return null;
        var directPath = Path.Combine(Path.GetDirectoryName(_pluginPath) ?? string.Empty, $"{assemblyName.Name}.dll");
        if (File.Exists(directPath)) return LoadFromAssemblyPath(directPath);
        return null;
    }

    protected override IntPtr LoadUnmanagedDll(string unmanagedDllName)
    {
        var libraryPath = _resolver.ResolveUnmanagedDllToPath(unmanagedDllName);
        return libraryPath is not null ? LoadUnmanagedDllFromPath(libraryPath) : IntPtr.Zero;
    }

    private static bool IsCoreAssembly(string? name)
    {
        if (name is null) return false;
        return name.StartsWith("Forge.Core", StringComparison.OrdinalIgnoreCase) ||
               name.StartsWith("System.", StringComparison.OrdinalIgnoreCase) ||
               name.StartsWith("Microsoft.", StringComparison.OrdinalIgnoreCase) ||
               name.Equals("netstandard", StringComparison.OrdinalIgnoreCase);
    }
}

public sealed class PluginLoader
{
    private readonly Dictionary<string, LoadedPlugin> _loadedPlugins = new(StringComparer.OrdinalIgnoreCase);
    private readonly object _lock = new();

    public IReadOnlyList<LoadedPlugin> LoadedPlugins { get { lock (_lock) { return _loadedPlugins.Values.ToList(); } } }

    public LoadedPlugin LoadPlugin(string pluginDir)
    {
        var metadataPath = Path.Combine(pluginDir, "plugin.json");
        var metadata = PluginMetadata.LoadFromFile(metadataPath);
        if (metadata is null) return LoadedPlugin.Failure(new PluginMetadata { Name = pluginDir }, "Missing plugin.json");

        var (isValid, error) = metadata.Validate();
        if (!isValid) return LoadedPlugin.Failure(metadata, error);

        var entryPath = Path.Combine(pluginDir, metadata.Entry);
        if (!File.Exists(entryPath)) return LoadedPlugin.Failure(metadata, $"Entry not found: {entryPath}");

        PluginLoadContext? loadContext = null;
        try {
            loadContext = new PluginLoadContext(entryPath);
            var assembly = loadContext.LoadFromAssemblyPath(entryPath);
            var pluginType = assembly.GetType(metadata.PluginClass);
            if (pluginType is null) { loadContext.Unload(); return LoadedPlugin.Failure(metadata, "Class not found"); }

            var instance = Activator.CreateInstance(pluginType) as ILanguagePlugin;
            if (instance is null) { loadContext.Unload(); return LoadedPlugin.Failure(metadata, "Failed to create instance"); }

            var loadedPlugin = LoadedPlugin.Success(metadata, instance, loadContext);
            lock (_lock) { _loadedPlugins[metadata.Language] = loadedPlugin; }
            return loadedPlugin;
        } catch (Exception ex) {
            loadContext?.Unload();
            return LoadedPlugin.Failure(metadata, ex.Message);
        }
    }

    public LoadedPlugin LoadBuiltinPlugin(PluginMetadata metadata, ILanguagePlugin instance)
    {
        lock (_lock) {
            var loaded = LoadedPlugin.Success(metadata, instance, null);
            _loadedPlugins[metadata.Language] = loaded;
            return loaded;
        }
    }

    public (bool Success, string Message) UnloadPlugin(string language)
    {
        lock (_lock) {
            if (!_loadedPlugins.TryGetValue(language, out var plugin)) return (false, "Not loaded");
            if (plugin.Metadata.IsBuiltin) return (false, "Cannot unload builtin");
            _loadedPlugins.Remove(language);
            
            // 關鍵修復：使用顯式轉型穿透可能存在的 object 型別障礙
            if (plugin.LoadContext is AssemblyLoadContext alc) { alc.Unload(); }
            
            return (true, "Unloaded");
        }
    }

    public ILanguagePlugin? GetPlugin(string language) { lock (_lock) { return _loadedPlugins.TryGetValue(language, out var p) ? p.Instance : null; } }
    public bool IsLoaded(string language) { lock (_lock) { return _loadedPlugins.ContainsKey(language); } }
    public PluginMetadata? GetMetadata(string language) { lock (_lock) { return _loadedPlugins.TryGetValue(language, out var p) ? p.Metadata : null; } }
}