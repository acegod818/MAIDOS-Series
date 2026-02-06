// MAIDOS-Forge Linker Configuration
// UEP v1.7B Compliant - Zero Technical Debt

namespace Forge.Core.Linker;

/// <summary>
/// 輸出類型
/// </summary>
public enum OutputType
{
    Executable,
    SharedLibrary,
    StaticLibrary
}

/// <summary>
/// 目標平台
/// </summary>
/// <impl>
/// APPROACH: 封裝平台和架構資訊
/// CALLS: N/A (純資料)
/// EDGES: N/A
/// </impl>
public sealed class TargetPlatform
{
    public string Os { get; init; } = "native";
    public string Arch { get; init; } = "native";

    /// <summary>
    /// 取得平台三元組
    /// </summary>
    /// <impl>
    /// APPROACH: 根據 OS/Arch 生成標準三元組
    /// CALLS: GetCurrentPlatform()
    /// EDGES: native 時使用當前平台
    /// </impl>
    public string ToTriple()
    {
        var os = Os == "native" ? GetCurrentOs() : Os;
        var arch = Arch == "native" ? GetCurrentArch() : Arch;

        return $"{arch}-{os}";
    }

    /// <summary>
    /// 取得當前平台
    /// </summary>
    /// <impl>
    /// APPROACH: 使用 OperatingSystem API
    /// CALLS: OperatingSystem.IsXxx()
    /// EDGES: 未知系統返回 unknown
    /// </impl>
    public static TargetPlatform Current => new()
    {
        Os = GetCurrentOs(),
        Arch = GetCurrentArch()
    };

    private static string GetCurrentOs()
    {
        if (OperatingSystem.IsWindows()) return "windows";
        if (OperatingSystem.IsLinux()) return "linux";
        if (OperatingSystem.IsMacOS()) return "macos";
        if (OperatingSystem.IsFreeBSD()) return "freebsd";
        return "unknown";
    }

    private static string GetCurrentArch()
    {
        return System.Runtime.InteropServices.RuntimeInformation.OSArchitecture switch
        {
            System.Runtime.InteropServices.Architecture.X64 => "x86_64",
            System.Runtime.InteropServices.Architecture.X86 => "x86",
            System.Runtime.InteropServices.Architecture.Arm64 => "aarch64",
            System.Runtime.InteropServices.Architecture.Arm => "arm",
            _ => "unknown"
        };
    }

    /// <summary>
    /// 取得共享庫副檔名
    /// </summary>
    /// <impl>
    /// APPROACH: 根據 OS 返回對應副檔名
    /// CALLS: N/A
    /// EDGES: 未知系統返回 .so
    /// </impl>
    public string GetSharedLibExtension()
    {
        var os = Os == "native" ? GetCurrentOs() : Os;
        return os switch
        {
            "windows" => ".dll",
            "macos" => ".dylib",
            _ => ".so"
        };
    }

    /// <summary>
    /// 取得可執行檔副檔名
    /// </summary>
    public string GetExecutableExtension()
    {
        var os = Os == "native" ? GetCurrentOs() : Os;
        return os == "windows" ? ".exe" : "";
    }

    /// <summary>
    /// 取得靜態庫副檔名
    /// </summary>
    public string GetStaticLibExtension()
    {
        var os = Os == "native" ? GetCurrentOs() : Os;
        return os == "windows" ? ".lib" : ".a";
    }
}

/// <summary>
/// 鏈接配置
/// </summary>
/// <impl>
/// APPROACH: 封裝鏈接器所需的所有配置
/// CALLS: N/A (純資料)
/// EDGES: 預設值適用於大多數情況
/// </impl>
public sealed class LinkConfig
{
    /// <summary>輸出檔案名 (不含副檔名)</summary>
    public string OutputName { get; init; } = "output";

    /// <summary>輸出目錄</summary>
    public string OutputDir { get; init; } = "build";

    /// <summary>輸出類型</summary>
    public OutputType OutputType { get; init; } = OutputType.Executable;

    /// <summary>目標平台</summary>
    public TargetPlatform Target { get; init; } = TargetPlatform.Current;

    /// <summary>是否啟用 LTO</summary>
    public bool EnableLto { get; init; } = false;

    /// <summary>是否剝離符號</summary>
    public bool StripSymbols { get; init; } = false;

    /// <summary>額外的鏈接器標誌</summary>
    public IReadOnlyList<string> ExtraFlags { get; init; } = Array.Empty<string>();

    /// <summary>系統庫</summary>
    public IReadOnlyList<string> SystemLibs { get; init; } = Array.Empty<string>();

    /// <summary>庫搜索路徑</summary>
    public IReadOnlyList<string> LibPaths { get; init; } = Array.Empty<string>();

    /// <summary>編譯配置 (debug/release)</summary>
    public string Profile { get; init; } = "release";

    /// <summary>詳細輸出</summary>
    public bool Verbose { get; init; } = false;
}

/// <summary>
/// 鏈接輸入 - 單一目標檔案或庫
/// </summary>
/// <impl>
/// APPROACH: 封裝輸入檔案資訊
/// CALLS: N/A (純資料)
/// EDGES: N/A
/// </impl>
public sealed class LinkInput
{
    /// <summary>檔案路徑</summary>
    public string Path { get; init; } = string.Empty;

    /// <summary>來源模組名稱</summary>
    public string ModuleName { get; init; } = string.Empty;

    /// <summary>來源語言</summary>
    public string Language { get; init; } = string.Empty;

    /// <summary>輸入類型</summary>
    public LinkInputType Type { get; init; } = LinkInputType.Object;

    /// <summary>是否為膠水代碼</summary>
    public bool IsGlue { get; init; } = false;
}

/// <summary>
/// 鏈接輸入類型
/// </summary>
public enum LinkInputType
{
    Object,         // .o
    StaticLib,      // .a / .lib
    SharedLib,      // .so / .dll / .dylib
    RustLib,        // .rlib
    DotNetAssembly  // .dll (CLR)
}

/// <summary>
/// 鏈接結果
/// </summary>
/// <impl>
/// APPROACH: 封裝鏈接結果
/// CALLS: N/A (純資料)
/// EDGES: IsSuccess 為 false 時 Error 非空
/// </impl>
public sealed class LinkResult
{
    public bool IsSuccess { get; }
    public string Error { get; }
    public string OutputPath { get; }
    public TimeSpan Duration { get; }
    public IReadOnlyList<string> Logs { get; }

    private LinkResult(bool isSuccess, string error, string outputPath, TimeSpan duration, IReadOnlyList<string>? logs)
    {
        IsSuccess = isSuccess;
        Error = error;
        OutputPath = outputPath;
        Duration = duration;
        Logs = logs ?? Array.Empty<string>();
    }

    public static LinkResult Success(string outputPath, TimeSpan duration, IReadOnlyList<string>? logs = null)
        => new(true, string.Empty, outputPath, duration, logs);

    public static LinkResult Failure(string error, TimeSpan duration = default, IReadOnlyList<string>? logs = null)
        => new(false, error, string.Empty, duration, logs);
}
