// <impl>
// WHAT: C# wrapper for maidos-auth native library
// WHY: Provide idiomatic C# API for capability-based authentication
// HOW: Static methods for token operations, Capability enum for type safety
// TEST: Unit tests for token creation, verification, capability checks
// </impl>

using System.Runtime.InteropServices;

namespace MaidosShared;

/// <summary>
/// Exception thrown when an authentication operation fails.
/// </summary>
public class AuthException : Exception
{
    public AuthException(string message) : base(message) { }
}

/// <summary>
/// Capability flags for MAIDOS authentication.
/// </summary>
[Flags]
public enum Capability : ulong
{
    None = 0,
    ConfigRead = 1UL << 0,
    ConfigWrite = 1UL << 1,
    BusPublish = 1UL << 2,
    BusSubscribe = 1UL << 3,
    LlmChat = 1UL << 4,
    LlmEmbed = 1UL << 5,
    StorageRead = 1UL << 6,
    StorageWrite = 1UL << 7,
    SystemInfo = 1UL << 8,
    SystemControl = 1UL << 9,
    PluginLoad = 1UL << 10,
    PluginUnload = 1UL << 11,
    NetworkAccess = 1UL << 12,
    FileRead = 1UL << 13,
    FileWrite = 1UL << 14,
    ProcessSpawn = 1UL << 15,
    Admin = 1UL << 16,
    Super = 1UL << 17,

    /// <summary>All capabilities granted.</summary>
    All = ulong.MaxValue
}

/// <summary>
/// Token verification result.
/// </summary>
public sealed class TokenVerificationResult
{
    /// <summary>Whether the token is valid.</summary>
    public bool IsValid { get; }

    /// <summary>Capabilities granted by the token (if valid).</summary>
    public Capability Capabilities { get; }

    internal TokenVerificationResult(bool isValid, ulong capabilities)
    {
        IsValid = isValid;
        Capabilities = (Capability)capabilities;
    }

    /// <summary>Check if a specific capability is granted.</summary>
    public bool HasCapability(Capability capability)
    {
        return IsValid && Capabilities.HasFlag(capability);
    }
}

/// <summary>
/// MAIDOS authentication manager.
/// Provides capability-based token creation and verification.
/// </summary>
/// <remarks>
/// Example usage:
/// <code>
/// var secret = "my-secret-key";
/// var token = MaidosAuth.CreateToken(secret, Capability.ConfigRead | Capability.LlmChat, 3600);
/// var result = MaidosAuth.VerifyToken(secret, token);
/// if (result.IsValid && result.HasCapability(Capability.LlmChat))
/// {
///     // Access granted
/// }
/// </code>
/// </remarks>
public static class MaidosAuth
{
    /// <summary>
    /// Create a new capability token.
    /// </summary>
    /// <param name="secret">Secret key for signing (should be at least 32 bytes).</param>
    /// <param name="capabilities">Capabilities to grant.</param>
    /// <param name="ttlSeconds">Time-to-live in seconds.</param>
    /// <returns>The signed token string.</returns>
    /// <exception cref="AuthException">Thrown when token creation fails.</exception>
    public static string CreateToken(string secret, Capability capabilities, uint ttlSeconds = 3600)
    {
        ArgumentNullException.ThrowIfNull(secret);

        var ptr = Native.AuthCreateToken(secret, (ulong)capabilities, ttlSeconds);
        if (ptr == IntPtr.Zero)
        {
            var errorPtr = Native.AuthLastError();
            var error = errorPtr != IntPtr.Zero
                ? Marshal.PtrToStringUTF8(errorPtr) ?? "Unknown error"
                : "Failed to create token";
            throw new AuthException(error);
        }

        try
        {
            return Marshal.PtrToStringUTF8(ptr) ?? throw new AuthException("Failed to marshal token");
        }
        finally
        {
            Native.AuthFreeString(ptr);
        }
    }

    /// <summary>
    /// Verify a capability token.
    /// </summary>
    /// <param name="secret">Secret key used to sign the token.</param>
    /// <param name="token">Token to verify.</param>
    /// <returns>Verification result with validity and capabilities.</returns>
    public static TokenVerificationResult VerifyToken(string secret, string token)
    {
        ArgumentNullException.ThrowIfNull(secret);
        ArgumentNullException.ThrowIfNull(token);

        var isValid = Native.AuthVerifyToken(secret, token, out var capabilities);
        return new TokenVerificationResult(isValid, capabilities);
    }

    /// <summary>
    /// Check if a token has a specific capability.
    /// </summary>
    /// <param name="secret">Secret key used to sign the token.</param>
    /// <param name="token">Token to check.</param>
    /// <param name="capability">Capability to check for.</param>
    /// <returns>True if the token is valid and has the capability.</returns>
    public static bool TokenHasCapability(string secret, string token, Capability capability)
    {
        ArgumentNullException.ThrowIfNull(secret);
        ArgumentNullException.ThrowIfNull(token);

        return Native.AuthTokenHasCapability(secret, token, (uint)GetCapabilityIndex(capability));
    }

    /// <summary>
    /// Get a capability from its name.
    /// </summary>
    /// <param name="name">Capability name (e.g., "config_read", "llm_chat").</param>
    /// <returns>The capability, or None if not found.</returns>
    public static Capability CapabilityFromName(string name)
    {
        ArgumentNullException.ThrowIfNull(name);

        var index = Native.AuthCapabilityFromName(name);
        if (index == 0 && name.ToLowerInvariant() != "config_read")
        {
            return Capability.None;
        }
        return (Capability)(1UL << (int)index);
    }

    /// <summary>
    /// Parse multiple capabilities from a comma-separated string.
    /// </summary>
    /// <param name="names">Comma-separated capability names.</param>
    /// <returns>Combined capabilities.</returns>
    public static Capability ParseCapabilities(string names)
    {
        ArgumentNullException.ThrowIfNull(names);

        var result = Capability.None;
        foreach (var name in names.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries))
        {
            result |= CapabilityFromName(name);
        }
        return result;
    }

    private static int GetCapabilityIndex(Capability capability)
    {
        var value = (ulong)capability;
        var index = 0;
        while (value > 1)
        {
            value >>= 1;
            index++;
        }
        return index;
    }
}
