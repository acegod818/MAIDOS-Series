// MAIDOS-Forge WebAssembly Compiler
// Code-QC v2.2B Compliant | M14 Cross-Compilation Module

using System.Diagnostics;
using System.Text;
using Forge.Core.Platform;

namespace Forge.Core.CrossCompile;

/// <summary>
/// WebAssembly 編譯配置
/// </summary>
public sealed class WasmCompileConfig
{
    /// <summary>WASI 或 Freestanding</summary>
    public WasmTarget Target { get; init; } = WasmTarget.Wasi;
    
    /// <summary>輸出目錄</summary>
    public string OutputDir { get; init; } = "build/wasm";
    
    /// <summary>輸出名稱</summary>
    public string OutputName { get; init; } = "module";
    
    /// <summary>優化等級 (0-3, s, z)</summary>
    public string OptLevel { get; init; } = "s";
    
    /// <summary>是否生成 Debug 資訊</summary>
    public bool Debug { get; init; }
    
    /// <summary>初始記憶體大小 (頁數，每頁 64KB)</summary>
    public int InitialMemory { get; init; } = 16;
    
    /// <summary>最大記憶體大小 (頁數，0 = 無限制)</summary>
    public int MaxMemory { get; init; } = 256;
    
    /// <summary>堆疊大小 (bytes)</summary>
    public int StackSize { get; init; } = 65536;
    
    /// <summary>是否導出所有符號</summary>
    public bool ExportAll { get; init; } = true;
    
    /// <summary>指定導出的函數</summary>
    public IReadOnlyList<string> Exports { get; init; } = Array.Empty<string>();
    
    /// <summary>是否生成 JavaScript 膠水代碼</summary>
    public bool GenerateJs { get; init; }
    
    /// <summary>是否生成 TypeScript 定義</summary>
    public bool GenerateDts { get; init; }
    
    /// <summary>額外的編譯標誌</summary>
    public IReadOnlyList<string> ExtraFlags { get; init; } = Array.Empty<string>();
    
    /// <summary>WASI SDK 路徑</summary>
    public string? WasiSdkPath { get; init; }
    
    /// <summary>是否啟用 SIMD</summary>
    public bool EnableSimd { get; init; }
    
    /// <summary>是否啟用線程</summary>
    public bool EnableThreads { get; init; }
    
    /// <summary>是否啟用例外處理</summary>
    public bool EnableExceptions { get; init; }
}

/// <summary>WASM 目標類型</summary>
public enum WasmTarget
{
    /// <summary>WASI - WebAssembly System Interface</summary>
    Wasi,
    /// <summary>Freestanding - 無系統依賴</summary>
    Freestanding,
    /// <summary>Emscripten - 瀏覽器環境</summary>
    Emscripten
}

/// <summary>
/// WebAssembly 編譯結果
/// </summary>
public sealed class WasmCompileResult
{
    public bool Success { get; init; }
    public string? WasmPath { get; init; }
    public string? JsPath { get; init; }
    public string? DtsPath { get; init; }
    public long WasmSize { get; init; }
    public TimeSpan Duration { get; init; }
    public IReadOnlyList<string> Logs { get; init; } = Array.Empty<string>();
    public string? ErrorMessage { get; init; }

    public static WasmCompileResult Ok(string wasmPath, string? jsPath, string? dtsPath, 
        long size, TimeSpan duration, IReadOnlyList<string> logs)
        => new() { Success = true, WasmPath = wasmPath, JsPath = jsPath, DtsPath = dtsPath, 
            WasmSize = size, Duration = duration, Logs = logs };

    public static WasmCompileResult Fail(string error, TimeSpan duration, IReadOnlyList<string> logs)
        => new() { Success = false, ErrorMessage = error, Duration = duration, Logs = logs };
}

/// <summary>
/// WebAssembly 編譯器
/// </summary>
public sealed class WasmCompiler
{
    private string? _clangPath;
    private string? _wasmLdPath;
    private string? _wasmOptPath;
    private string? _sysroot;

    /// <summary>初始化 WASM 編譯環境</summary>
    public async Task<(bool available, string message)> InitializeAsync(WasmCompileConfig config, CancellationToken ct = default)
    {
        // 檢查 WASI SDK
        var wasiSdk = config.WasiSdkPath ?? Environment.GetEnvironmentVariable("WASI_SDK_PATH");
        
        if (!string.IsNullOrEmpty(wasiSdk) && Directory.Exists(wasiSdk))
        {
            _clangPath = Path.Combine(wasiSdk, "bin", OperatingSystem.IsWindows() ? "clang.exe" : "clang");
            _wasmLdPath = Path.Combine(wasiSdk, "bin", OperatingSystem.IsWindows() ? "wasm-ld.exe" : "wasm-ld");
            _sysroot = Path.Combine(wasiSdk, "share", "wasi-sysroot");

            if (File.Exists(_clangPath))
            {
                return (true, $"WASI SDK: {wasiSdk}");
            }
        }

        // 檢查系統 Clang
        if (await ProcessRunner.CommandExistsAsync("clang"))
        {
            _clangPath = "clang";
            
            // 檢查 wasm-ld
            if (await ProcessRunner.CommandExistsAsync("wasm-ld"))
            {
                _wasmLdPath = "wasm-ld";
            }
            else if (await ProcessRunner.CommandExistsAsync("ld.lld"))
            {
                _wasmLdPath = "ld.lld";
            }

            // 檢查 wasm-opt
            if (await ProcessRunner.CommandExistsAsync("wasm-opt"))
            {
                _wasmOptPath = "wasm-opt";
            }

            var version = await ProcessRunner.GetVersionAsync("clang", "--version");
            return (true, $"System Clang: {version}");
        }

        // 檢查 Emscripten
        if (config.Target == WasmTarget.Emscripten)
        {
            if (await ProcessRunner.CommandExistsAsync("emcc"))
            {
                _clangPath = "emcc";
                var version = await ProcessRunner.GetVersionAsync("emcc", "--version");
                return (true, $"Emscripten: {version}");
            }
        }

        return (false, "No WASM toolchain found. Install WASI SDK or Emscripten.");
    }

    /// <summary>編譯 C/C++ 到 WebAssembly</summary>
    public async Task<WasmCompileResult> CompileAsync(
        IReadOnlyList<string> sourceFiles,
        WasmCompileConfig config,
        CancellationToken ct = default)
    {
        var sw = Stopwatch.StartNew();
        var logs = new List<string>();

        // 初始化工具鏈
        var (available, toolchainMsg) = await InitializeAsync(config, ct);
        if (!available)
        {
            sw.Stop();
            return WasmCompileResult.Fail(toolchainMsg, sw.Elapsed, logs);
        }

        logs.Add($"[WASM] Toolchain: {toolchainMsg}");
        logs.Add($"[WASM] Target: {config.Target}");

        Directory.CreateDirectory(config.OutputDir);

        try
        {
            string wasmPath;

            if (config.Target == WasmTarget.Emscripten)
            {
                wasmPath = await CompileWithEmscriptenAsync(sourceFiles, config, logs, ct);
            }
            else
            {
                wasmPath = await CompileWithClangAsync(sourceFiles, config, logs, ct);
            }

            // 優化 WASM
            if (_wasmOptPath != null && !config.Debug)
            {
                await OptimizeWasmAsync(wasmPath, config, logs, ct);
            }

            // 生成 JS 膠水代碼
            string? jsPath = null;
            string? dtsPath = null;

            if (config.GenerateJs)
            {
                jsPath = await GenerateJsGlueAsync(wasmPath, config, logs, ct);
            }

            if (config.GenerateDts)
            {
                dtsPath = await GenerateDtsAsync(wasmPath, config, logs, ct);
            }

            var wasmSize = new FileInfo(wasmPath).Length;
            sw.Stop();

            logs.Add($"[WASM] Output: {wasmPath} ({wasmSize:N0} bytes)");
            return WasmCompileResult.Ok(wasmPath, jsPath, dtsPath, wasmSize, sw.Elapsed, logs);
        }
        catch (Exception ex)
        {
            sw.Stop();
            return WasmCompileResult.Fail($"WASM compilation failed: {ex.Message}", sw.Elapsed, logs);
        }
    }

    private async Task<string> CompileWithClangAsync(
        IReadOnlyList<string> sourceFiles,
        WasmCompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var wasmPath = Path.Combine(config.OutputDir, config.OutputName + ".wasm");
        var args = new List<string>();

        // 目標三元組
        var target = config.Target == WasmTarget.Wasi ? "wasm32-wasi" : "wasm32-unknown-unknown";
        args.Add($"--target={target}");

        // Sysroot
        if (_sysroot != null && config.Target == WasmTarget.Wasi)
        {
            args.Add($"--sysroot=\"{_sysroot}\"");
        }

        // 優化
        args.Add($"-O{config.OptLevel}");

        // Debug
        if (config.Debug)
        {
            args.Add("-g");
        }

        // 特性啟用
        if (config.EnableSimd)
        {
            args.Add("-msimd128");
        }

        if (config.EnableThreads)
        {
            args.Add("-pthread");
            args.Add("-matomics");
            args.Add("-mbulk-memory");
        }

        if (!config.EnableExceptions)
        {
            args.Add("-fno-exceptions");
        }

        // 記憶體設定
        args.Add($"-Wl,--initial-memory={config.InitialMemory * 65536}");
        if (config.MaxMemory > 0)
        {
            args.Add($"-Wl,--max-memory={config.MaxMemory * 65536}");
        }
        args.Add($"-Wl,-z,stack-size={config.StackSize}");

        // 導出
        if (config.ExportAll)
        {
            args.Add("-Wl,--export-all");
        }
        else
        {
            foreach (var export in config.Exports)
            {
                args.Add($"-Wl,--export={export}");
            }
        }

        // Freestanding 模式
        if (config.Target == WasmTarget.Freestanding)
        {
            args.Add("-nostdlib");
            args.Add("-Wl,--no-entry");
        }

        // 額外標誌
        foreach (var flag in config.ExtraFlags)
        {
            args.Add(flag);
        }

        // 輸出
        args.Add("-o");
        args.Add($"\"{wasmPath}\"");

        // 源檔案
        foreach (var src in sourceFiles)
        {
            args.Add($"\"{src}\"");
        }

        var cmdLine = string.Join(" ", args);
        logs.Add($"$ {_clangPath} {cmdLine}");

        var result = await ProcessRunner.RunAsync(_clangPath!, cmdLine,
            new ProcessConfig { Timeout = TimeSpan.FromMinutes(10) }, ct);

        if (!string.IsNullOrEmpty(result.Stdout)) logs.Add(result.Stdout);
        if (!string.IsNullOrEmpty(result.Stderr)) logs.Add(result.Stderr);

        if (!result.IsSuccess)
        {
            throw new Exception($"Clang compilation failed: {result.Stderr}");
        }

        return wasmPath;
    }

    private async Task<string> CompileWithEmscriptenAsync(
        IReadOnlyList<string> sourceFiles,
        WasmCompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var wasmPath = Path.Combine(config.OutputDir, config.OutputName + ".wasm");
        var jsPath = Path.Combine(config.OutputDir, config.OutputName + ".js");

        var args = new List<string>
        {
            $"-O{config.OptLevel}",
            "-s", "WASM=1",
            "-s", $"INITIAL_MEMORY={config.InitialMemory * 65536}",
            "-s", $"STACK_SIZE={config.StackSize}"
        };

        if (config.Debug)
        {
            args.Add("-g");
        }

        if (config.ExportAll)
        {
            args.Add("-s");
            args.Add("EXPORTED_FUNCTIONS=['_main']");
            args.Add("-s");
            args.Add("EXPORTED_RUNTIME_METHODS=['ccall','cwrap']");
        }

        if (config.EnableSimd)
        {
            args.Add("-msimd128");
        }

        if (config.EnableThreads)
        {
            args.Add("-pthread");
            args.Add("-s");
            args.Add("USE_PTHREADS=1");
        }

        foreach (var flag in config.ExtraFlags)
        {
            args.Add(flag);
        }

        args.Add("-o");
        args.Add($"\"{jsPath}\"");

        foreach (var src in sourceFiles)
        {
            args.Add($"\"{src}\"");
        }

        var cmdLine = string.Join(" ", args);
        logs.Add($"$ emcc {cmdLine}");

        var result = await ProcessRunner.RunAsync("emcc", cmdLine,
            new ProcessConfig { Timeout = TimeSpan.FromMinutes(10) }, ct);

        if (!string.IsNullOrEmpty(result.Stdout)) logs.Add(result.Stdout);
        if (!string.IsNullOrEmpty(result.Stderr)) logs.Add(result.Stderr);

        if (!result.IsSuccess)
        {
            throw new Exception($"Emscripten compilation failed: {result.Stderr}");
        }

        return wasmPath;
    }

    private async Task OptimizeWasmAsync(
        string wasmPath,
        WasmCompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var optLevel = config.OptLevel switch
        {
            "s" => "-Os",
            "z" => "-Oz",
            "3" => "-O3",
            "2" => "-O2",
            _ => "-O1"
        };

        var args = $"{optLevel} \"{wasmPath}\" -o \"{wasmPath}\"";
        logs.Add($"$ wasm-opt {args}");

        var result = await ProcessRunner.RunAsync(_wasmOptPath!, args,
            new ProcessConfig { Timeout = TimeSpan.FromMinutes(5) }, ct);

        if (!string.IsNullOrEmpty(result.Stderr)) logs.Add(result.Stderr);
    }

    private async Task<string> GenerateJsGlueAsync(
        string wasmPath,
        WasmCompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var jsPath = Path.ChangeExtension(wasmPath, ".js");
        var moduleName = Path.GetFileNameWithoutExtension(wasmPath);

        var sb = new StringBuilder();
        sb.AppendLine("// Auto-generated by MAIDOS-Forge WASM Compiler");
        sb.AppendLine($"// Module: {moduleName}");
        sb.AppendLine();
        sb.AppendLine("export async function loadWasm(importObject = {}) {");
        sb.AppendLine($"    const response = await fetch('./{moduleName}.wasm');");
        sb.AppendLine("    const bytes = await response.arrayBuffer();");
        sb.AppendLine("    const { instance } = await WebAssembly.instantiate(bytes, importObject);");
        sb.AppendLine("    return instance.exports;");
        sb.AppendLine("}");
        sb.AppendLine();
        sb.AppendLine("export async function loadWasmSync(wasmBytes, importObject = {}) {");
        sb.AppendLine("    const { instance } = await WebAssembly.instantiate(wasmBytes, importObject);");
        sb.AppendLine("    return instance.exports;");
        sb.AppendLine("}");

        await File.WriteAllTextAsync(jsPath, sb.ToString(), ct);
        logs.Add($"[WASM] Generated JS glue: {jsPath}");

        return jsPath;
    }

    private async Task<string> GenerateDtsAsync(
        string wasmPath,
        WasmCompileConfig config,
        List<string> logs,
        CancellationToken ct)
    {
        var dtsPath = Path.ChangeExtension(wasmPath, ".d.ts");
        var moduleName = Path.GetFileNameWithoutExtension(wasmPath);

        var sb = new StringBuilder();
        sb.AppendLine("// Auto-generated by MAIDOS-Forge WASM Compiler");
        sb.AppendLine($"// Module: {moduleName}");
        sb.AppendLine();
        sb.AppendLine("export interface WasmExports {");
        sb.AppendLine("    memory: WebAssembly.Memory;");
        sb.AppendLine("    [key: string]: WebAssembly.ExportValue;");
        sb.AppendLine("}");
        sb.AppendLine();
        sb.AppendLine("export function loadWasm(importObject?: WebAssembly.Imports): Promise<WasmExports>;");
        sb.AppendLine("export function loadWasmSync(wasmBytes: ArrayBuffer, importObject?: WebAssembly.Imports): Promise<WasmExports>;");

        await File.WriteAllTextAsync(dtsPath, sb.ToString(), ct);
        logs.Add($"[WASM] Generated TypeScript definitions: {dtsPath}");

        return dtsPath;
    }
}
