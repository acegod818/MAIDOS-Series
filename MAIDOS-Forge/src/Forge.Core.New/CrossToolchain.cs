// MAIDOS-Forge Cross-Compilation Toolchain Manager
// Code-QC v2.2B Compliant | M14 Cross-Compilation Module

using Forge.Core.Platform;

namespace Forge.Core.CrossCompile;

public sealed class CrossToolchain
{
    public CrossTarget Target { get; init; } = CrossTarget.Native;
    public string? CC { get; init; }
    public string? CXX { get; init; }
    public string? LD { get; init; }
    public string? AR { get; init; }
    public string? Strip { get; init; }
    public string? Sysroot { get; init; }
    public IReadOnlyList<string> CFlags { get; init; } = Array.Empty<string>();
    public IReadOnlyList<string> LdFlags { get; init; } = Array.Empty<string>();
    public bool IsAvailable { get; private set; }
    public string? ValidationMessage { get; private set; }

    public async Task<bool> ValidateAsync(CancellationToken ct = default)
    {
        var cc = CC ?? GetDefaultCC();
        if (!await ProcessRunner.CommandExistsAsync(cc))
        {
            ValidationMessage = $"C compiler not found: {cc}";
            IsAvailable = false;
            return false;
        }

        var testResult = await TestCompilerAsync(cc, ct);
        if (!testResult.success)
        {
            ValidationMessage = $"Compiler test failed: {testResult.message}";
            IsAvailable = false;
            return false;
        }

        ValidationMessage = $"Toolchain OK: {cc}";
        IsAvailable = true;
        return true;
    }

    private string GetDefaultCC() => Target.OS switch
    {
        TargetOS.Windows when Target.Abi == "msvc" => "cl",
        TargetOS.Windows when Target.Abi == "gnu" => $"{Target.Arch.ToGnuPrefix()}-w64-mingw32-gcc",
        TargetOS.Wasi or TargetOS.Freestanding => "clang",
        _ when !Target.IsNative => $"{Target.Triple}-gcc",
        _ => "gcc"
    };

    private async Task<(bool success, string message)> TestCompilerAsync(string cc, CancellationToken ct)
    {
        var testDir = Path.Combine(Path.GetTempPath(), $"forge-toolchain-test-{Guid.NewGuid():N}");
        Directory.CreateDirectory(testDir);

        try
        {
            var testSrc = Path.Combine(testDir, "test.c");
            var testOut = Path.Combine(testDir, "test" + Target.ObjectExtension);
            await File.WriteAllTextAsync(testSrc, "int main() { return 0; }", ct);
            var args = new List<string> { "-c", $"\"{testSrc}\"", "-o", $"\"{testOut}\"" };
            if (Target.OS is TargetOS.Wasi or TargetOS.Freestanding) args.Insert(0, $"--target={Target.Triple}");
            if (!string.IsNullOrEmpty(Sysroot)) args.Add($"--sysroot=\"{Sysroot}\"");

            var result = await ProcessRunner.RunAsync(cc, string.Join(" ", args),
                new ProcessConfig { WorkingDirectory = testDir, Timeout = TimeSpan.FromSeconds(30) }, ct);
            if (!result.IsSuccess) return (false, result.Stderr);
            return File.Exists(testOut) ? (true, "OK") : (false, "No output produced");
        }
        finally
        {
            try 
            { 
                if (Directory.Exists(testDir)) Directory.Delete(testDir, true); 
            } 
            catch (Exception ex) 
            { 
                System.Diagnostics.Debug.WriteLine($"[MAIDOS-AUDIT] Cleanup failed: {ex.Message}"); 
            }
        }
    }

    public string GetCCCommand() => CC ?? GetDefaultCC();
    public string GetCXXCommand() => CXX ?? (GetDefaultCC().Replace("gcc", "g++").Replace("clang", "clang++"));
    public string GetLDCommand() => LD ?? GetDefaultCC();

    public IEnumerable<string> GetTargetCFlags()
    {
        foreach (var flag in CFlags) yield return flag;
        if (Target.OS is TargetOS.Wasi or TargetOS.Freestanding || !Target.IsNative) yield return $"--target={Target.Triple}";
        if (!string.IsNullOrEmpty(Sysroot)) yield return $"--sysroot={Sysroot}";
        if (Target.Arch is TargetArch.Wasm32 or TargetArch.Wasm64) { yield return "-fno-exceptions"; yield return "-fno-rtti"; }
        if (Target.OS == TargetOS.Windows && Target.Abi == "gnu") { yield return "-static-libgcc"; yield return "-static-libstdc++"; }
    }

    public IEnumerable<string> GetTargetLdFlags()
    {
        foreach (var flag in LdFlags) yield return flag;
        if (Target.OS is TargetOS.Wasi or TargetOS.Freestanding || !Target.IsNative) yield return $"--target={Target.Triple}";
        if (!string.IsNullOrEmpty(Sysroot)) yield return $"--sysroot={Sysroot}";
        if (Target.Arch is TargetArch.Wasm32 or TargetArch.Wasm64)
        {
            yield return "-nostdlib";
            if (Target.OS == TargetOS.Wasi) { yield return "-Wl,--export-all"; yield return "-Wl,--no-entry"; }
        }
    }
}

public sealed class CrossToolchainManager
{
    private readonly Dictionary<string, CrossToolchain> _toolchains = new();

    public async Task<CrossToolchain> GetToolchainAsync(CrossTarget target, CancellationToken ct = default)
    {
        var key = target.Triple;
        if (_toolchains.TryGetValue(key, out var cached)) return cached;
        var toolchain = await DetectToolchainAsync(target, ct);
        _toolchains[key] = toolchain;
        return toolchain;
    }

    private async Task<CrossToolchain> DetectToolchainAsync(CrossTarget target, CancellationToken ct)
    {
        if (target.IsNative) return await DetectNativeToolchainAsync(ct);
        return new CrossToolchain { Target = target }; // Simplified for now
    }

    private async Task<CrossToolchain> DetectNativeToolchainAsync(CancellationToken ct)
    {
        var toolchain = new CrossToolchain { Target = CrossTarget.Native };
        if (await ProcessRunner.CommandExistsAsync("clang"))
        {
            toolchain = new CrossToolchain { Target = toolchain.Target, CC = "clang", CXX = "clang++", LD = "clang" };
        }
        else if (await ProcessRunner.CommandExistsAsync("gcc"))
        {
            toolchain = new CrossToolchain { Target = toolchain.Target, CC = "gcc", CXX = "g++", LD = "gcc" };
        }
        await toolchain.ValidateAsync(ct);
        return toolchain;
    }
}

public static class TargetArchExtensions
{
    public static string ToGnuPrefix(this TargetArch arch) => arch switch
    {
        TargetArch.X86_64 => "x86_64",
        TargetArch.X86 => "i686",
        TargetArch.Arm64 => "aarch64",
        TargetArch.Arm => "arm",
        TargetArch.Riscv64 => "riscv64",
        _ => "unknown"
    };
}