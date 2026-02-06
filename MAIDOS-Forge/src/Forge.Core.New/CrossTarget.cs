// MAIDOS-Forge Cross-Compilation Target Definitions
// Code-QC v2.2B Compliant | M14 Cross-Compilation Module

namespace Forge.Core.CrossCompile;

/// <summary>
/// 交叉編譯目標平台
/// </summary>
public sealed class CrossTarget
{
    /// <summary>目標作業系統</summary>
    public TargetOS OS { get; init; }
    
    /// <summary>目標架構</summary>
    public TargetArch Arch { get; init; }
    
    /// <summary>ABI 變體</summary>
    public string? Abi { get; init; }
    
    /// <summary>供應商</summary>
    public string Vendor { get; init; } = "unknown";

    /// <summary>目標三元組 (e.g., x86_64-unknown-linux-gnu)</summary>
    public string Triple => $"{ArchString}-{Vendor}-{OsString}{(Abi != null ? $"-{Abi}" : "")}";

    /// <summary>是否為本機目標</summary>
    public bool IsNative => OS == CurrentOS && Arch == CurrentArch;

    private string ArchString => Arch switch
    {
        TargetArch.X86_64 => "x86_64",
        TargetArch.X86 => "i686",
        TargetArch.Arm64 => "aarch64",
        TargetArch.Arm => "arm",
        TargetArch.Riscv64 => "riscv64",
        TargetArch.Wasm32 => "wasm32",
        TargetArch.Wasm64 => "wasm64",
        _ => "unknown"
    };

    private string OsString => OS switch
    {
        TargetOS.Linux => "linux",
        TargetOS.Windows => "windows",
        TargetOS.MacOS => "darwin",
        TargetOS.FreeBSD => "freebsd",
        TargetOS.Wasi => "wasi",
        TargetOS.Freestanding => "freestanding",
        TargetOS.Android => "android",
        TargetOS.iOS => "ios",
        _ => "unknown"
    };

    /// <summary>取得共享庫擴展名</summary>
    public string SharedLibExtension => OS switch
    {
        TargetOS.Windows => ".dll",
        TargetOS.MacOS or TargetOS.iOS => ".dylib",
        TargetOS.Wasi or TargetOS.Freestanding => ".wasm",
        _ => ".so"
    };

    /// <summary>取得靜態庫擴展名</summary>
    public string StaticLibExtension => OS switch
    {
        TargetOS.Windows => ".lib",
        _ => ".a"
    };

    /// <summary>取得執行檔擴展名</summary>
    public string ExecutableExtension => OS switch
    {
        TargetOS.Windows => ".exe",
        TargetOS.Wasi or TargetOS.Freestanding => ".wasm",
        _ => ""
    };

    /// <summary>取得目標檔案擴展名</summary>
    public string ObjectExtension => OS switch
    {
        TargetOS.Windows => ".obj",
        _ => ".o"
    };

    // ════════════════════════════════════════════════════════════════════
    // 預定義目標
    // ════════════════════════════════════════════════════════════════════

    public static readonly CrossTarget LinuxX64 = new() { OS = TargetOS.Linux, Arch = TargetArch.X86_64, Abi = "gnu" };
    public static readonly CrossTarget LinuxArm64 = new() { OS = TargetOS.Linux, Arch = TargetArch.Arm64, Abi = "gnu" };
    public static readonly CrossTarget LinuxMusl = new() { OS = TargetOS.Linux, Arch = TargetArch.X86_64, Abi = "musl" };
    
    public static readonly CrossTarget WindowsX64 = new() { OS = TargetOS.Windows, Arch = TargetArch.X86_64, Vendor = "pc", Abi = "msvc" };
    public static readonly CrossTarget WindowsArm64 = new() { OS = TargetOS.Windows, Arch = TargetArch.Arm64, Vendor = "pc", Abi = "msvc" };
    public static readonly CrossTarget WindowsGnu = new() { OS = TargetOS.Windows, Arch = TargetArch.X86_64, Vendor = "pc", Abi = "gnu" };
    
    public static readonly CrossTarget MacOSX64 = new() { OS = TargetOS.MacOS, Arch = TargetArch.X86_64, Vendor = "apple" };
    public static readonly CrossTarget MacOSArm64 = new() { OS = TargetOS.MacOS, Arch = TargetArch.Arm64, Vendor = "apple" };
    
    public static readonly CrossTarget Wasm32Wasi = new() { OS = TargetOS.Wasi, Arch = TargetArch.Wasm32 };
    public static readonly CrossTarget Wasm32Freestanding = new() { OS = TargetOS.Freestanding, Arch = TargetArch.Wasm32 };

    public static readonly CrossTarget AndroidArm64 = new() { OS = TargetOS.Android, Arch = TargetArch.Arm64 };
    public static readonly CrossTarget iOSArm64 = new() { OS = TargetOS.iOS, Arch = TargetArch.Arm64, Vendor = "apple" };

    // ════════════════════════════════════════════════════════════════════
    // 當前平台檢測
    // ════════════════════════════════════════════════════════════════════

    public static TargetOS CurrentOS =>
        OperatingSystem.IsWindows() ? TargetOS.Windows :
        OperatingSystem.IsMacOS() ? TargetOS.MacOS :
        OperatingSystem.IsLinux() ? TargetOS.Linux :
        OperatingSystem.IsFreeBSD() ? TargetOS.FreeBSD :
        TargetOS.Unknown;

    public static TargetArch CurrentArch =>
        System.Runtime.InteropServices.RuntimeInformation.ProcessArchitecture switch
        {
            System.Runtime.InteropServices.Architecture.X64 => TargetArch.X86_64,
            System.Runtime.InteropServices.Architecture.X86 => TargetArch.X86,
            System.Runtime.InteropServices.Architecture.Arm64 => TargetArch.Arm64,
            System.Runtime.InteropServices.Architecture.Arm => TargetArch.Arm,
            _ => TargetArch.Unknown
        };

    public static CrossTarget Native => new() { OS = CurrentOS, Arch = CurrentArch };

    // ════════════════════════════════════════════════════════════════════
    // 解析目標字串
    // ════════════════════════════════════════════════════════════════════

    /// <summary>從字串解析目標 (e.g., "linux-x64", "windows-arm64", "wasm32-wasi")</summary>
    public static CrossTarget Parse(string target)
    {
        var lower = target.ToLowerInvariant();
        
        return lower switch
        {
            "native" or "host" => Native,
            "linux-x64" or "x86_64-linux-gnu" => LinuxX64,
            "linux-arm64" or "aarch64-linux-gnu" => LinuxArm64,
            "linux-musl" or "x86_64-linux-musl" => LinuxMusl,
            "windows-x64" or "x86_64-pc-windows-msvc" => WindowsX64,
            "windows-arm64" or "aarch64-pc-windows-msvc" => WindowsArm64,
            "windows-gnu" or "x86_64-pc-windows-gnu" => WindowsGnu,
            "macos-x64" or "x86_64-apple-darwin" => MacOSX64,
            "macos-arm64" or "aarch64-apple-darwin" => MacOSArm64,
            "wasm32-wasi" or "wasm-wasi" or "wasi" => Wasm32Wasi,
            "wasm32" or "wasm" => Wasm32Freestanding,
            "android-arm64" or "aarch64-linux-android" => AndroidArm64,
            "ios-arm64" or "aarch64-apple-ios" => iOSArm64,
            _ => ParseTriple(target)
        };
    }

    private static CrossTarget ParseTriple(string triple)
    {
        var parts = triple.Split('-');
        if (parts.Length < 2) return Native;

        var arch = parts[0] switch
        {
            "x86_64" or "amd64" => TargetArch.X86_64,
            "i686" or "i386" or "x86" => TargetArch.X86,
            "aarch64" or "arm64" => TargetArch.Arm64,
            "arm" or "armv7" => TargetArch.Arm,
            "riscv64" => TargetArch.Riscv64,
            "wasm32" => TargetArch.Wasm32,
            "wasm64" => TargetArch.Wasm64,
            _ => TargetArch.Unknown
        };

        var os = parts.Length > 2 ? parts[2] : parts[1];
        var targetOs = os switch
        {
            "linux" => TargetOS.Linux,
            "windows" => TargetOS.Windows,
            "darwin" or "macos" => TargetOS.MacOS,
            "freebsd" => TargetOS.FreeBSD,
            "wasi" => TargetOS.Wasi,
            "freestanding" or "unknown" when arch is TargetArch.Wasm32 or TargetArch.Wasm64 => TargetOS.Freestanding,
            "android" => TargetOS.Android,
            "ios" => TargetOS.iOS,
            _ => TargetOS.Unknown
        };

        var vendor = parts.Length > 2 ? parts[1] : "unknown";
        var abi = parts.Length > 3 ? parts[3] : null;

        return new CrossTarget { OS = targetOs, Arch = arch, Vendor = vendor, Abi = abi };
    }

    /// <summary>取得所有預定義目標</summary>
    public static IReadOnlyList<CrossTarget> AllTargets => new[]
    {
        LinuxX64, LinuxArm64, LinuxMusl,
        WindowsX64, WindowsArm64, WindowsGnu,
        MacOSX64, MacOSArm64,
        Wasm32Wasi, Wasm32Freestanding,
        AndroidArm64, iOSArm64
    };

    public override string ToString() => Triple;
}

/// <summary>目標作業系統</summary>
public enum TargetOS
{
    Unknown,
    Linux,
    Windows,
    MacOS,
    FreeBSD,
    Wasi,
    Freestanding,
    Android,
    iOS
}

/// <summary>目標架構</summary>
public enum TargetArch
{
    Unknown,
    X86_64,
    X86,
    Arm64,
    Arm,
    Riscv64,
    Wasm32,
    Wasm64
}
