// MAIDOS-IME Native FFI Bindings
// 對齊 Rust maidos-core/src/ffi.rs 的 11 個 FFI 導出函數
// 參照 SharedCore P/Invoke 模式
//
// [MAIDOS-AUDIT] 11 個 DllImport 聲明對齊 Rust FFI 導出

using System;
using System.Runtime.InteropServices;

namespace MAIDOS.IME.AI
{
    /// <summary>
    /// Rust maidos_core.dll 的 P/Invoke 綁定
    /// [MAIDOS-AUDIT] 對齊 Rust ffi.rs 的 11 個 #[no_mangle] 導出
    /// </summary>
    internal static class ImeNativeMethods
    {
        private const string NativeDll = "maidos_core";

        // 核心功能 (8 個)

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "ime_process_input")]
        internal static extern IntPtr ime_process_input(IntPtr schemeName, IntPtr input);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "ime_get_candidates")]
        internal static extern IntPtr ime_get_candidates(IntPtr schemeName, IntPtr input);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "ime_convert_charset")]
        internal static extern IntPtr ime_convert_charset(IntPtr text, IntPtr fromCharset, IntPtr toCharset);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "ime_supported_schemes")]
        internal static extern IntPtr ime_supported_schemes();

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "ime_detect_language")]
        internal static extern IntPtr ime_detect_language(IntPtr text);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "ime_version")]
        internal static extern IntPtr ime_version();

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "ime_pinyin_lookup")]
        internal static extern IntPtr ime_pinyin_lookup(IntPtr pinyin);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "ime_init")]
        internal static extern int ime_init();

        // 記憶體管理 (3 個)

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "ime_last_error")]
        internal static extern IntPtr ime_last_error();

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "ime_free_string")]
        internal static extern void ime_free_string(IntPtr s);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "ime_clear_error")]
        internal static extern void ime_clear_error();
    }

    /// <summary>
    /// 高階 Managed 包裝 — 封裝 ImeNativeMethods 的指標操作
    /// 提供 C# 友善的 API
    /// </summary>
    public static class ImeNative
    {
        private static bool _initialized;

        /// <summary>
        /// 初始化 IME Core 引擎
        /// </summary>
        public static void Initialize()
        {
            if (_initialized) return;
            ImeNativeMethods.ime_init();
            _initialized = true;
        }

        /// <summary>
        /// 處理輸入並返回候選字 JSON
        /// </summary>
        public static string ProcessInput(string schemeName, string input)
        {
            IntPtr schemePtr = Marshal.StringToHGlobalAnsi(schemeName);
            IntPtr inputPtr = Marshal.StringToHGlobalAnsi(input);
            try
            {
                IntPtr resultPtr = ImeNativeMethods.ime_process_input(schemePtr, inputPtr);
                return ConsumeStringResult(resultPtr, "ProcessInput");
            }
            finally
            {
                Marshal.FreeHGlobal(schemePtr);
                Marshal.FreeHGlobal(inputPtr);
            }
        }

        /// <summary>
        /// 獲取候選字 JSON
        /// </summary>
        public static string GetCandidates(string schemeName, string input)
        {
            IntPtr schemePtr = Marshal.StringToHGlobalAnsi(schemeName);
            IntPtr inputPtr = Marshal.StringToHGlobalAnsi(input);
            try
            {
                IntPtr resultPtr = ImeNativeMethods.ime_get_candidates(schemePtr, inputPtr);
                return ConsumeStringResult(resultPtr, "GetCandidates");
            }
            finally
            {
                Marshal.FreeHGlobal(schemePtr);
                Marshal.FreeHGlobal(inputPtr);
            }
        }

        /// <summary>
        /// 字符集轉換 (繁↔簡)
        /// </summary>
        public static string ConvertCharset(string text, string fromCharset, string toCharset)
        {
            IntPtr textPtr = Marshal.StringToHGlobalAnsi(text);
            IntPtr fromPtr = Marshal.StringToHGlobalAnsi(fromCharset);
            IntPtr toPtr = Marshal.StringToHGlobalAnsi(toCharset);
            try
            {
                IntPtr resultPtr = ImeNativeMethods.ime_convert_charset(textPtr, fromPtr, toPtr);
                return ConsumeStringResult(resultPtr, "ConvertCharset");
            }
            finally
            {
                Marshal.FreeHGlobal(textPtr);
                Marshal.FreeHGlobal(fromPtr);
                Marshal.FreeHGlobal(toPtr);
            }
        }

        /// <summary>
        /// 偵測文字語言
        /// </summary>
        public static string DetectLanguage(string text)
        {
            IntPtr textPtr = Marshal.StringToHGlobalAnsi(text);
            try
            {
                IntPtr resultPtr = ImeNativeMethods.ime_detect_language(textPtr);
                return ConsumeStringResult(resultPtr, "DetectLanguage");
            }
            finally
            {
                Marshal.FreeHGlobal(textPtr);
            }
        }

        /// <summary>
        /// 獲取支援的輸入方案列表 (JSON)
        /// </summary>
        public static string GetSupportedSchemes()
        {
            IntPtr resultPtr = ImeNativeMethods.ime_supported_schemes();
            return ConsumeStringResult(resultPtr, "GetSupportedSchemes");
        }

        /// <summary>
        /// 獲取版本資訊 (JSON)
        /// </summary>
        public static string GetVersion()
        {
            IntPtr resultPtr = ImeNativeMethods.ime_version();
            return ConsumeStringResult(resultPtr, "GetVersion");
        }

        /// <summary>
        /// 拼音快速查詢
        /// </summary>
        public static string PinyinLookup(string pinyin)
        {
            IntPtr pinyinPtr = Marshal.StringToHGlobalAnsi(pinyin);
            try
            {
                IntPtr resultPtr = ImeNativeMethods.ime_pinyin_lookup(pinyinPtr);
                return ConsumeStringResult(resultPtr, "PinyinLookup");
            }
            finally
            {
                Marshal.FreeHGlobal(pinyinPtr);
            }
        }

        // Helper

        private static string ConsumeStringResult(IntPtr ptr, string operation)
        {
            if (ptr == IntPtr.Zero)
            {
                string error = GetLastError();
                throw new InvalidOperationException(
                    $"[MAIDOS-AUDIT] IME FFI {operation} failed: {error}");
            }

            try
            {
                return Marshal.PtrToStringAnsi(ptr) ?? string.Empty;
            }
            finally
            {
                ImeNativeMethods.ime_free_string(ptr);
            }
        }

        private static string GetLastError()
        {
            IntPtr errPtr = ImeNativeMethods.ime_last_error();
            if (errPtr == IntPtr.Zero) return "Unknown error";
            try
            {
                return Marshal.PtrToStringAnsi(errPtr) ?? "Unknown error";
            }
            finally
            {
                ImeNativeMethods.ime_free_string(errPtr);
            }
        }
    }
}
