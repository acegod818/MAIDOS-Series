// MAIDOS-Forge Native FFI Bindings
// 對齊 Rust maidos-forge-core/src/ffi.rs 的 10 個 FFI 導出函數
// 參照 SharedCore P/Invoke 模式
//
// [MAIDOS-AUDIT] 10 個 DllImport 聲明對齊 Rust FFI 導出

using System;
using System.Runtime.InteropServices;
using System.Text.Json;

namespace Forge.Core.Native;

using CallingConvention = System.Runtime.InteropServices.CallingConvention;

/// <summary>
/// Rust maidos_forge_core.dll 的 P/Invoke 綁定
/// </summary>
internal static class ForgeNative
{
    private const string NativeDll = "maidos_forge_core";

    // =================================================================
    // 核心功能 FFI (6 個)
    // =================================================================

    /// <summary>
    /// 解析源碼文件，返回 JSON 格式的 AST
    /// </summary>
    /// <param name="language">語言名稱 ("rust", "c", "cpp")</param>
    /// <param name="sourcePath">源碼文件路徑</param>
    /// <returns>JSON 字串指標，需用 forge_free_string 釋放；失敗返回 IntPtr.Zero</returns>
    [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "forge_parse_source")]
    internal static extern IntPtr forge_parse_source(IntPtr language, IntPtr sourcePath);

    /// <summary>
    /// 檢查源碼語法，返回 JSON 格式的診斷結果
    /// </summary>
    [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "forge_check_syntax")]
    internal static extern IntPtr forge_check_syntax(IntPtr language, IntPtr sourcePath);

    /// <summary>
    /// 返回支援的解析語言列表 (JSON 陣列)
    /// </summary>
    [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "forge_supported_languages")]
    internal static extern IntPtr forge_supported_languages();

    /// <summary>
    /// 返回 Forge Core 版本資訊 (JSON)
    /// </summary>
    [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "forge_version")]
    internal static extern IntPtr forge_version();

    /// <summary>
    /// 批次解析多個源碼文件
    /// </summary>
    /// <param name="language">語言名稱</param>
    /// <param name="pathsJson">JSON 陣列格式的路徑列表</param>
    [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "forge_parse_batch")]
    internal static extern IntPtr forge_parse_batch(IntPtr language, IntPtr pathsJson);

    /// <summary>
    /// 增量解析源碼文件
    /// </summary>
    /// <param name="language">語言名稱</param>
    /// <param name="sourcePath">源碼文件路徑</param>
    /// <param name="prevHash">前次的 file_hash (可為 IntPtr.Zero)</param>
    [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "forge_parse_incremental")]
    internal static extern IntPtr forge_parse_incremental(IntPtr language, IntPtr sourcePath, IntPtr prevHash);

    // =================================================================
    // 記憶體管理 FFI (4 個)
    // =================================================================

    /// <summary>
    /// 獲取最後一次錯誤信息
    /// </summary>
    [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "forge_last_error")]
    internal static extern IntPtr forge_last_error();

    /// <summary>
    /// 釋放 FFI 返回的字串
    /// </summary>
    [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "forge_free_string")]
    internal static extern void forge_free_string(IntPtr s);

    /// <summary>
    /// 初始化 Forge Core
    /// </summary>
    [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "forge_init")]
    internal static extern int forge_init();

    /// <summary>
    /// 清除最後一次錯誤
    /// </summary>
    [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "forge_clear_error")]
    internal static extern void forge_clear_error();
}

/// <summary>
/// 高階 Managed 包裝 — 封裝 ForgeNative 的指標操作
/// 提供 C# 友善的 API
/// </summary>
public static class ForgeCoreBridge
{
    private static bool _initialized;

    /// <summary>
    /// 初始化 Forge Core 引擎
    /// </summary>
    public static void Initialize()
    {
        if (_initialized) return;
        ForgeNative.forge_init();
        _initialized = true;
    }

    /// <summary>
    /// 解析源碼文件並返回 JSON AST
    /// </summary>
    /// <param name="language">語言名稱 ("rust", "c", "cpp")</param>
    /// <param name="sourcePath">源碼文件的完整路徑</param>
    /// <returns>JSON 字串，失敗時拋出異常</returns>
    public static string ParseSource(string language, string sourcePath)
    {
        IntPtr langPtr = Marshal.StringToHGlobalAnsi(language);
        IntPtr pathPtr = Marshal.StringToHGlobalAnsi(sourcePath);
        try
        {
            IntPtr resultPtr = ForgeNative.forge_parse_source(langPtr, pathPtr);
            return ConsumeStringResult(resultPtr, "ParseSource");
        }
        finally
        {
            Marshal.FreeHGlobal(langPtr);
            Marshal.FreeHGlobal(pathPtr);
        }
    }

    /// <summary>
    /// 檢查源碼語法並返回 JSON 診斷結果
    /// </summary>
    public static string CheckSyntax(string language, string sourcePath)
    {
        IntPtr langPtr = Marshal.StringToHGlobalAnsi(language);
        IntPtr pathPtr = Marshal.StringToHGlobalAnsi(sourcePath);
        try
        {
            IntPtr resultPtr = ForgeNative.forge_check_syntax(langPtr, pathPtr);
            return ConsumeStringResult(resultPtr, "CheckSyntax");
        }
        finally
        {
            Marshal.FreeHGlobal(langPtr);
            Marshal.FreeHGlobal(pathPtr);
        }
    }

    /// <summary>
    /// 獲取支援的語言列表
    /// </summary>
    public static string[] GetSupportedLanguages()
    {
        IntPtr resultPtr = ForgeNative.forge_supported_languages();
        string json = ConsumeStringResult(resultPtr, "GetSupportedLanguages");
        return JsonSerializer.Deserialize<string[]>(json) ?? Array.Empty<string>();
    }

    /// <summary>
    /// 獲取 Forge Core 版本資訊
    /// </summary>
    public static string GetVersion()
    {
        IntPtr resultPtr = ForgeNative.forge_version();
        return ConsumeStringResult(resultPtr, "GetVersion");
    }

    /// <summary>
    /// 批次解析多個源碼文件
    /// </summary>
    public static string ParseBatch(string language, string[] paths)
    {
        IntPtr langPtr = Marshal.StringToHGlobalAnsi(language);
        string pathsJson = JsonSerializer.Serialize(paths);
        IntPtr pathsPtr = Marshal.StringToHGlobalAnsi(pathsJson);
        try
        {
            IntPtr resultPtr = ForgeNative.forge_parse_batch(langPtr, pathsPtr);
            return ConsumeStringResult(resultPtr, "ParseBatch");
        }
        finally
        {
            Marshal.FreeHGlobal(langPtr);
            Marshal.FreeHGlobal(pathsPtr);
        }
    }

    /// <summary>
    /// 增量解析源碼文件
    /// </summary>
    public static string ParseIncremental(string language, string sourcePath, string? prevHash = null)
    {
        IntPtr langPtr = Marshal.StringToHGlobalAnsi(language);
        IntPtr pathPtr = Marshal.StringToHGlobalAnsi(sourcePath);
        IntPtr hashPtr = string.IsNullOrEmpty(prevHash)
            ? IntPtr.Zero
            : Marshal.StringToHGlobalAnsi(prevHash);
        try
        {
            IntPtr resultPtr = ForgeNative.forge_parse_incremental(langPtr, pathPtr, hashPtr);
            return ConsumeStringResult(resultPtr, "ParseIncremental");
        }
        finally
        {
            Marshal.FreeHGlobal(langPtr);
            Marshal.FreeHGlobal(pathPtr);
            if (hashPtr != IntPtr.Zero) Marshal.FreeHGlobal(hashPtr);
        }
    }

    // =================================================================
    // Helper
    // =================================================================

    /// <summary>
    /// 從 Rust FFI 返回的指標讀取字串並釋放記憶體
    /// </summary>
    private static string ConsumeStringResult(IntPtr ptr, string operation)
    {
        if (ptr == IntPtr.Zero)
        {
            string error = GetLastError();
            throw new InvalidOperationException(
                $"[MAIDOS-AUDIT] Forge FFI {operation} failed: {error}");
        }

        try
        {
            return Marshal.PtrToStringAnsi(ptr) ?? string.Empty;
        }
        finally
        {
            ForgeNative.forge_free_string(ptr);
        }
    }

    /// <summary>
    /// 獲取最後一次 Rust 端錯誤信息
    /// </summary>
    private static string GetLastError()
    {
        IntPtr errPtr = ForgeNative.forge_last_error();
        if (errPtr == IntPtr.Zero)
        {
            return "Unknown error";
        }

        try
        {
            return Marshal.PtrToStringAnsi(errPtr) ?? "Unknown error";
        }
        finally
        {
            ForgeNative.forge_free_string(errPtr);
        }
    }
}
