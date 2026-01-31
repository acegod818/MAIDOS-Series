// <impl>
// WHAT: P/Invoke declarations for all MAIDOS native functions
// WHY: Bridge between C# managed code and Rust native library
// HOW: DllImport with proper marshaling and calling conventions
// TEST: Indirect testing through wrapper classes
// </impl>

using System.Runtime.InteropServices;

namespace MaidosShared;

/// <summary>
/// Native P/Invoke declarations for MAIDOS shared library.
/// Internal use only - use the wrapper classes instead.
/// </summary>
internal static partial class Native
{
    private const string LibraryName = "maidos_shared";

    // ========================================================================
    // maidos-config
    // ========================================================================

    [LibraryImport(LibraryName, EntryPoint = "maidos_config_load", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial IntPtr ConfigLoad(string path);

    [LibraryImport(LibraryName, EntryPoint = "maidos_config_free")]
    internal static partial void ConfigFree(IntPtr handle);

    [LibraryImport(LibraryName, EntryPoint = "maidos_config_last_error")]
    internal static partial IntPtr ConfigLastError();

    [LibraryImport(LibraryName, EntryPoint = "maidos_config_to_json")]
    internal static partial IntPtr ConfigToJson(IntPtr handle);

    [LibraryImport(LibraryName, EntryPoint = "maidos_config_get_string", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial IntPtr ConfigGetString(IntPtr handle, string key);

    [LibraryImport(LibraryName, EntryPoint = "maidos_config_get_f64", StringMarshalling = StringMarshalling.Utf8)]
    [return: MarshalAs(UnmanagedType.I1)]
    internal static partial bool ConfigGetF64(IntPtr handle, string key, out double value);

    [LibraryImport(LibraryName, EntryPoint = "maidos_config_get_u64", StringMarshalling = StringMarshalling.Utf8)]
    [return: MarshalAs(UnmanagedType.I1)]
    internal static partial bool ConfigGetU64(IntPtr handle, string key, out ulong value);

    [LibraryImport(LibraryName, EntryPoint = "maidos_config_free_string")]
    internal static partial void ConfigFreeString(IntPtr s);

    // ========================================================================
    // maidos-auth
    // ========================================================================

    [LibraryImport(LibraryName, EntryPoint = "maidos_auth_last_error")]
    internal static partial IntPtr AuthLastError();

    [LibraryImport(LibraryName, EntryPoint = "maidos_auth_create_token", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial IntPtr AuthCreateToken(string secret, ulong capabilities, uint ttlSecs);

    [LibraryImport(LibraryName, EntryPoint = "maidos_auth_verify_token", StringMarshalling = StringMarshalling.Utf8)]
    [return: MarshalAs(UnmanagedType.I1)]
    internal static partial bool AuthVerifyToken(string secret, string token, out ulong capabilities);

    [LibraryImport(LibraryName, EntryPoint = "maidos_auth_token_has_capability", StringMarshalling = StringMarshalling.Utf8)]
    [return: MarshalAs(UnmanagedType.I1)]
    internal static partial bool AuthTokenHasCapability(string secret, string token, uint capability);

    [LibraryImport(LibraryName, EntryPoint = "maidos_auth_capability_from_name", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial uint AuthCapabilityFromName(string name);

    [LibraryImport(LibraryName, EntryPoint = "maidos_auth_free_string")]
    internal static partial void AuthFreeString(IntPtr s);

    // ========================================================================
    // maidos-bus
    // ========================================================================

    [LibraryImport(LibraryName, EntryPoint = "maidos_bus_publisher_create", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial IntPtr BusPublisherCreate(string bindAddr);

    [LibraryImport(LibraryName, EntryPoint = "maidos_bus_publisher_port")]
    internal static partial ushort BusPublisherPort(IntPtr handle);

    [LibraryImport(LibraryName, EntryPoint = "maidos_bus_publish", StringMarshalling = StringMarshalling.Utf8)]
    [return: MarshalAs(UnmanagedType.I1)]
    internal static partial bool BusPublish(IntPtr handle, string topic, string source, IntPtr payload, nuint payloadLen);

    [LibraryImport(LibraryName, EntryPoint = "maidos_bus_publisher_destroy")]
    internal static partial void BusPublisherDestroy(IntPtr handle);

    [LibraryImport(LibraryName, EntryPoint = "maidos_bus_subscriber_create", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial IntPtr BusSubscriberCreate(string connectAddr, string? topicFilter);

    [LibraryImport(LibraryName, EntryPoint = "maidos_bus_receive")]
    internal static partial IntPtr BusReceive(IntPtr handle, uint timeoutMs);

    [LibraryImport(LibraryName, EntryPoint = "maidos_bus_event_free")]
    internal static partial void BusEventFree(IntPtr eventPtr);

    [LibraryImport(LibraryName, EntryPoint = "maidos_bus_subscriber_destroy")]
    internal static partial void BusSubscriberDestroy(IntPtr handle);

    // ========================================================================
    // maidos-llm
    // ========================================================================

    [LibraryImport(LibraryName, EntryPoint = "maidos_llm_create", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial IntPtr LlmCreate(string providerName, string? apiKey, string? baseUrl);

    [LibraryImport(LibraryName, EntryPoint = "maidos_llm_provider_name")]
    internal static partial IntPtr LlmProviderName(IntPtr handle);

    [LibraryImport(LibraryName, EntryPoint = "maidos_llm_health_check")]
    [return: MarshalAs(UnmanagedType.I1)]
    internal static partial bool LlmHealthCheck(IntPtr handle);

    [LibraryImport(LibraryName, EntryPoint = "maidos_llm_complete", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial IntPtr LlmComplete(
        IntPtr handle,
        string model,
        string? system,
        string userMessage,
        uint maxTokens,
        float temperature);

    [LibraryImport(LibraryName, EntryPoint = "maidos_llm_result_free")]
    internal static partial void LlmResultFree(IntPtr result);

    [LibraryImport(LibraryName, EntryPoint = "maidos_llm_destroy")]
    internal static partial void LlmDestroy(IntPtr handle);

    [LibraryImport(LibraryName, EntryPoint = "maidos_llm_string_free")]
    internal static partial void LlmStringFree(IntPtr s);
}

/// <summary>
/// FFI Event structure from maidos-bus
/// </summary>
[StructLayout(LayoutKind.Sequential)]
internal struct FfiEvent
{
    public IntPtr Topic;
    public IntPtr Id;
    public ulong Timestamp;
    public IntPtr Source;
    public IntPtr Payload;
    public nuint PayloadLen;
}

/// <summary>
/// FFI Completion result structure from maidos-llm
/// </summary>
[StructLayout(LayoutKind.Sequential)]
internal struct FfiCompletionResult
{
    [MarshalAs(UnmanagedType.I1)]
    public bool Success;
    public IntPtr Text;
    public IntPtr Error;
    public uint PromptTokens;
    public uint CompletionTokens;
    public IntPtr Model;
}
