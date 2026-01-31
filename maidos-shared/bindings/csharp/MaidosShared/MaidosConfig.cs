// <impl>
// WHAT: C# wrapper for maidos-config native library
// WHY: Provide idiomatic C# API for configuration management
// HOW: IDisposable pattern with SafeHandle, automatic string marshaling
// TEST: Unit tests for load, get operations, error handling
// </impl>

using System.Runtime.InteropServices;

namespace MaidosShared;

/// <summary>
/// Exception thrown when a configuration operation fails.
/// </summary>
public class ConfigException : Exception
{
    public ConfigException(string message) : base(message) { }
}

/// <summary>
/// MAIDOS configuration manager.
/// Loads and manages TOML configuration files with hot-reload support.
/// </summary>
/// <remarks>
/// Example usage:
/// <code>
/// using var config = MaidosConfig.Load("config.toml");
/// var serverHost = config.GetString("server.host");
/// var serverPort = config.GetUInt64("server.port");
/// </code>
/// </remarks>
public sealed class MaidosConfig : IDisposable
{
    private IntPtr _handle;
    private bool _disposed;

    private MaidosConfig(IntPtr handle)
    {
        _handle = handle;
    }

    /// <summary>
    /// Load configuration from a TOML file.
    /// </summary>
    /// <param name="path">Path to the TOML configuration file.</param>
    /// <returns>A new MaidosConfig instance.</returns>
    /// <exception cref="ConfigException">Thrown when loading fails.</exception>
    public static MaidosConfig Load(string path)
    {
        ArgumentNullException.ThrowIfNull(path);

        var handle = Native.ConfigLoad(path);
        if (handle == IntPtr.Zero)
        {
            var errorPtr = Native.ConfigLastError();
            var error = errorPtr != IntPtr.Zero
                ? Marshal.PtrToStringUTF8(errorPtr) ?? "Unknown error"
                : "Failed to load configuration";
            throw new ConfigException(error);
        }

        return new MaidosConfig(handle);
    }

    /// <summary>
    /// Get the entire configuration as a JSON string.
    /// </summary>
    /// <returns>JSON representation of the configuration.</returns>
    /// <exception cref="ConfigException">Thrown when serialization fails.</exception>
    public string ToJson()
    {
        ThrowIfDisposed();

        var ptr = Native.ConfigToJson(_handle);
        if (ptr == IntPtr.Zero)
        {
            throw new ConfigException("Failed to serialize configuration to JSON");
        }

        try
        {
            return Marshal.PtrToStringUTF8(ptr) ?? "{}";
        }
        finally
        {
            Native.ConfigFreeString(ptr);
        }
    }

    /// <summary>
    /// Get a string value from the configuration.
    /// </summary>
    /// <param name="key">Dot-separated key path (e.g., "server.host").</param>
    /// <returns>The string value, or null if not found.</returns>
    public string? GetString(string key)
    {
        ThrowIfDisposed();
        ArgumentNullException.ThrowIfNull(key);

        var ptr = Native.ConfigGetString(_handle, key);
        if (ptr == IntPtr.Zero)
        {
            return null;
        }

        try
        {
            return Marshal.PtrToStringUTF8(ptr);
        }
        finally
        {
            Native.ConfigFreeString(ptr);
        }
    }

    /// <summary>
    /// Get a string value from the configuration, throwing if not found.
    /// </summary>
    /// <param name="key">Dot-separated key path.</param>
    /// <returns>The string value.</returns>
    /// <exception cref="ConfigException">Thrown when key is not found.</exception>
    public string GetRequiredString(string key)
    {
        return GetString(key) ?? throw new ConfigException($"Configuration key not found: {key}");
    }

    /// <summary>
    /// Get a double value from the configuration.
    /// </summary>
    /// <param name="key">Dot-separated key path.</param>
    /// <returns>The double value, or null if not found.</returns>
    public double? GetDouble(string key)
    {
        ThrowIfDisposed();
        ArgumentNullException.ThrowIfNull(key);

        if (Native.ConfigGetF64(_handle, key, out var value))
        {
            return value;
        }
        return null;
    }

    /// <summary>
    /// Get a double value from the configuration, throwing if not found.
    /// </summary>
    /// <param name="key">Dot-separated key path.</param>
    /// <returns>The double value.</returns>
    /// <exception cref="ConfigException">Thrown when key is not found.</exception>
    public double GetRequiredDouble(string key)
    {
        return GetDouble(key) ?? throw new ConfigException($"Configuration key not found: {key}");
    }

    /// <summary>
    /// Get an unsigned 64-bit integer value from the configuration.
    /// </summary>
    /// <param name="key">Dot-separated key path.</param>
    /// <returns>The integer value, or null if not found.</returns>
    public ulong? GetUInt64(string key)
    {
        ThrowIfDisposed();
        ArgumentNullException.ThrowIfNull(key);

        if (Native.ConfigGetU64(_handle, key, out var value))
        {
            return value;
        }
        return null;
    }

    /// <summary>
    /// Get an unsigned 64-bit integer value, throwing if not found.
    /// </summary>
    /// <param name="key">Dot-separated key path.</param>
    /// <returns>The integer value.</returns>
    /// <exception cref="ConfigException">Thrown when key is not found.</exception>
    public ulong GetRequiredUInt64(string key)
    {
        return GetUInt64(key) ?? throw new ConfigException($"Configuration key not found: {key}");
    }

    /// <summary>
    /// Get an integer value from the configuration.
    /// </summary>
    /// <param name="key">Dot-separated key path.</param>
    /// <returns>The integer value, or null if not found.</returns>
    public int? GetInt(string key)
    {
        var value = GetUInt64(key);
        return value.HasValue ? (int)value.Value : null;
    }

    /// <summary>
    /// Get an integer value, throwing if not found.
    /// </summary>
    /// <param name="key">Dot-separated key path.</param>
    /// <returns>The integer value.</returns>
    /// <exception cref="ConfigException">Thrown when key is not found.</exception>
    public int GetRequiredInt(string key)
    {
        return GetInt(key) ?? throw new ConfigException($"Configuration key not found: {key}");
    }

    /// <summary>
    /// Check if a key exists in the configuration.
    /// </summary>
    /// <param name="key">Dot-separated key path.</param>
    /// <returns>True if the key exists.</returns>
    public bool HasKey(string key)
    {
        // Try to get as string first (most general)
        return GetString(key) != null || GetDouble(key) != null;
    }

    private void ThrowIfDisposed()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
    }

    /// <summary>
    /// Releases the configuration handle.
    /// </summary>
    public void Dispose()
    {
        if (!_disposed)
        {
            if (_handle != IntPtr.Zero)
            {
                Native.ConfigFree(_handle);
                _handle = IntPtr.Zero;
            }
            _disposed = true;
        }
    }
}
