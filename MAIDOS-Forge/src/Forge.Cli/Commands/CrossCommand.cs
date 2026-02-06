// MAIDOS-Forge Cross-Compilation CLI Command
// Code-QC v2.2B Compliant | M14 Cross-Compilation Module

using Forge.Core.CrossCompile;

namespace Forge.Cli.Commands;

/// <summary>
/// 交叉編譯命令
/// </summary>
public sealed class CrossCommand : ICommand
{
    public string Name => "cross";
    public string Description => "Cross-compile to multiple target platforms";
    public string Usage => @"
Usage: forge cross [options] <sources...>

Options:
  -t, --target <target>    Target platform (can be specified multiple times)
                           Examples: linux-x64, windows-x64, macos-arm64, wasm32-wasi
  -o, --output <dir>       Output directory (default: build)
  -n, --name <name>        Output name (default: from first source file)
  --type <type>            Output type: exe, shared, static (default: exe)
  --opt <level>            Optimization level: 0, 1, 2, 3, s, z (default: 2)
  -g, --debug              Include debug information
  -s, --strip              Strip symbols from output
  -I <dir>                 Add include directory
  -L <dir>                 Add library directory
  -l <lib>                 Link library
  -D <name>[=value]        Define preprocessor macro
  --parallel               Enable parallel compilation (default)
  --sequential             Disable parallel compilation
  --continue-on-error      Continue building other targets on error
  --list-targets           List all available targets
  --check-toolchains       Check toolchain availability for targets
  -v, --verbose            Show verbose output
  -h, --help               Show this help

Predefined Target Sets:
  --all-desktop            Linux/Windows/macOS x64+arm64
  --default                Linux/Windows/macOS x64 + WASM
  --wasm-only              WASM targets only

Examples:
  forge cross -t linux-x64 -t windows-x64 src/*.c
  forge cross --default -o dist -n myapp src/main.c
  forge cross --all-desktop --type shared -n libfoo src/*.c
  forge cross -t wasm32-wasi --opt s src/module.c
";

    public CommandResult Execute(string[] args)
    {
        // 在同步方法中執行異步操作
        return ExecuteAsync(args).GetAwaiter().GetResult();
    }

    private async Task<CommandResult> ExecuteAsync(string[] args)
    {
        if (args.Length == 0 || args.Contains("-h") || args.Contains("--help"))
        {
            Console.WriteLine(Usage);
            return CommandResult.Ok();
        }

        // 特殊命令
        if (args.Contains("--list-targets"))
        {
            ListTargets();
            return CommandResult.Ok();
        }

        if (args.Contains("--check-toolchains"))
        {
            var exitCode = await CheckToolchainsAsync(args, default);
            return exitCode == 0 ? CommandResult.Ok() : CommandResult.Error(exitCode, "Toolchain check failed");
        }

        // 解析參數
        var config = ParseArgs(args, out var sourceFiles);

        if (sourceFiles.Count == 0)
        {
            Console.Error.WriteLine("Error: No source files specified");
            return CommandResult.Error(1, "No source files specified");
        }

        if (config.Targets.Count == 0)
        {
            Console.Error.WriteLine("Error: No targets specified. Use -t or --target, or use --default");
            return CommandResult.Error(1, "No targets specified");
        }

        // 驗證源檔案存在
        foreach (var src in sourceFiles)
        {
            if (!File.Exists(src))
            {
                Console.Error.WriteLine($"Error: Source file not found: {src}");
                return CommandResult.Error(1, $"Source file not found: {src}");
            }
        }

        // 執行編譯
        Console.WriteLine($"Cross-compiling to {config.Targets.Count} target(s)...");
        Console.WriteLine();

        var orchestrator = new MultiTargetOrchestrator();
        var result = await orchestrator.BuildAsync(sourceFiles, config, null, default);

        // 顯示結果
        Console.WriteLine();
        Console.WriteLine("═══════════════════════════════════════════════════════════════");
        Console.WriteLine($"Build Results: {result.SuccessCount}/{result.Results.Count} succeeded");
        Console.WriteLine($"Total Time: {result.TotalDuration.TotalSeconds:F2}s");
        Console.WriteLine("═══════════════════════════════════════════════════════════════");

        foreach (var (target, r) in result.Results)
        {
            var status = r.Success ? "✓" : "✗";
            var output = r.Success ? r.OutputPath : r.ErrorMessage;
            Console.WriteLine($"  [{status}] {target.Triple,-30} {output}");
        }

        if (config.Verbose && result.FailureCount > 0)
        {
            Console.WriteLine();
            Console.WriteLine("Failed build logs:");
            foreach (var (target, r) in result.Results.Where(x => !x.Result.Success))
            {
                Console.WriteLine($"\n--- {target.Triple} ---");
                foreach (var log in r.Logs)
                {
                    Console.WriteLine(log);
                }
            }
        }

        return result.AllSucceeded ? CommandResult.Ok() : CommandResult.Error(1, "Cross-compilation failed");
    }

    private static int ListTargets()
    {
        Console.WriteLine("Available Cross-Compilation Targets:");
        Console.WriteLine();
        Console.WriteLine("  Desktop:");
        Console.WriteLine("    linux-x64          x86_64-unknown-linux-gnu");
        Console.WriteLine("    linux-arm64        aarch64-unknown-linux-gnu");
        Console.WriteLine("    linux-musl         x86_64-unknown-linux-musl");
        Console.WriteLine("    windows-x64        x86_64-pc-windows-msvc");
        Console.WriteLine("    windows-arm64      aarch64-pc-windows-msvc");
        Console.WriteLine("    windows-gnu        x86_64-pc-windows-gnu (MinGW)");
        Console.WriteLine("    macos-x64          x86_64-apple-darwin");
        Console.WriteLine("    macos-arm64        aarch64-apple-darwin");
        Console.WriteLine();
        Console.WriteLine("  WebAssembly:");
        Console.WriteLine("    wasm32-wasi        wasm32-wasi");
        Console.WriteLine("    wasm32             wasm32-unknown-unknown (freestanding)");
        Console.WriteLine();
        Console.WriteLine("  Mobile:");
        Console.WriteLine("    android-arm64      aarch64-linux-android");
        Console.WriteLine("    ios-arm64          aarch64-apple-ios");
        Console.WriteLine();
        Console.WriteLine("  Predefined Sets:");
        Console.WriteLine("    --default          linux-x64, windows-x64, macos-x64, wasm32-wasi");
        Console.WriteLine("    --all-desktop      All 6 desktop targets");
        Console.WriteLine("    --wasm-only        wasm32-wasi, wasm32");
        return 0;
    }

    private static async Task<int> CheckToolchainsAsync(string[] args, CancellationToken ct)
    {
        var targets = new List<CrossTarget>();

        // 解析目標
        for (int i = 0; i < args.Length; i++)
        {
            if ((args[i] == "-t" || args[i] == "--target") && i + 1 < args.Length)
            {
                targets.Add(CrossTarget.Parse(args[++i]));
            }
            else if (args[i] == "--default")
            {
                targets.AddRange(MultiTargetConfig.DefaultTargets.Targets);
            }
            else if (args[i] == "--all-desktop")
            {
                targets.AddRange(MultiTargetConfig.AllDesktop.Targets);
            }
        }

        if (targets.Count == 0)
        {
            targets.AddRange(CrossTarget.AllTargets);
        }

        Console.WriteLine("Checking toolchain availability...");
        Console.WriteLine();

        var orchestrator = new MultiTargetOrchestrator();
        var results = await orchestrator.CheckToolchainsAsync(targets, ct);

        var availableCount = 0;
        foreach (var (target, available, message) in results)
        {
            var status = available ? "✓" : "✗";
            Console.WriteLine($"  [{status}] {target.Triple,-30} {message}");
            if (available) availableCount++;
        }

        Console.WriteLine();
        Console.WriteLine($"Available: {availableCount}/{results.Count}");

        return availableCount > 0 ? 0 : 1;
    }

    private static MultiTargetConfig ParseArgs(string[] args, out List<string> sourceFiles)
    {
        var targets = new List<CrossTarget>();
        sourceFiles = new List<string>();
        var outputDir = "build";
        var outputName = "";
        var outputType = CrossOutputType.Executable;
        var optLevel = "2";
        var debugMode = false;
        var strip = false;
        var parallel = true;
        var continueOnError = true;
        var verbose = false;
        var includeDirs = new List<string>();
        var libDirs = new List<string>();
        var libraries = new List<string>();
        var extraCFlags = new List<string>();
        var extraLdFlags = new List<string>();

        for (int i = 0; i < args.Length; i++)
        {
            var arg = args[i];

            switch (arg)
            {
                case "-t" or "--target" when i + 1 < args.Length:
                    targets.Add(CrossTarget.Parse(args[++i]));
                    break;

                case "--default":
                    targets.AddRange(MultiTargetConfig.DefaultTargets.Targets);
                    break;

                case "--all-desktop":
                    targets.AddRange(MultiTargetConfig.AllDesktop.Targets);
                    break;

                case "--wasm-only":
                    targets.Add(CrossTarget.Wasm32Wasi);
                    targets.Add(CrossTarget.Wasm32Freestanding);
                    break;

                case "-o" or "--output" when i + 1 < args.Length:
                    outputDir = args[++i];
                    break;

                case "-n" or "--name" when i + 1 < args.Length:
                    outputName = args[++i];
                    break;

                case "--type" when i + 1 < args.Length:
                    outputType = args[++i].ToLowerInvariant() switch
                    {
                        "shared" or "dll" or "so" or "dylib" => CrossOutputType.SharedLibrary,
                        "static" or "lib" or "a" => CrossOutputType.StaticLibrary,
                        "obj" or "object" => CrossOutputType.Object,
                        _ => CrossOutputType.Executable
                    };
                    break;

                case "--opt" when i + 1 < args.Length:
                    optLevel = args[++i];
                    break;

                case "-g" or "--debug":
                    debugMode = true;
                    break;

                case "-s" or "--strip":
                    strip = true;
                    break;

                case "--parallel":
                    parallel = true;
                    break;

                case "--sequential":
                    parallel = false;
                    break;

                case "--continue-on-error":
                    continueOnError = true;
                    break;

                case "-v" or "--verbose":
                    verbose = true;
                    break;

                case "-I" when i + 1 < args.Length:
                    includeDirs.Add(args[++i]);
                    break;

                case "-L" when i + 1 < args.Length:
                    libDirs.Add(args[++i]);
                    break;

                case "-l" when i + 1 < args.Length:
                    libraries.Add(args[++i]);
                    break;

                case "-D" when i + 1 < args.Length:
                    extraCFlags.Add($"-D{args[++i]}");
                    break;

                case var _ when arg.StartsWith("-I"):
                    includeDirs.Add(arg.Substring(2));
                    break;

                case var _ when arg.StartsWith("-L"):
                    libDirs.Add(arg.Substring(2));
                    break;

                case var _ when arg.StartsWith("-l"):
                    libraries.Add(arg.Substring(2));
                    break;

                case var _ when arg.StartsWith("-D"):
                    extraCFlags.Add(arg);
                    break;

                case var _ when !arg.StartsWith("-"):
                    // 可能是 glob 模式
                    if (arg.Contains("*"))
                    {
                        var dir = Path.GetDirectoryName(arg) ?? ".";
                        var pattern = Path.GetFileName(arg);
                        sourceFiles.AddRange(Directory.GetFiles(dir, pattern));
                    }
                    else
                    {
                        sourceFiles.Add(arg);
                    }
                    break;
            }
        }

        // 預設輸出名稱
        if (string.IsNullOrEmpty(outputName) && sourceFiles.Count > 0)
        {
            outputName = Path.GetFileNameWithoutExtension(sourceFiles[0]);
        }

        return new MultiTargetConfig
        {
            Targets = targets.Distinct().ToList(),
            OutputDir = outputDir,
            OutputName = outputName,
            OutputType = outputType,
            OptLevel = optLevel,
            Debug = debugMode,
            Strip = strip,
            Parallel = parallel,
            ContinueOnError = continueOnError,
            Verbose = verbose,
            IncludeDirs = includeDirs,
            LibDirs = libDirs,
            Libraries = libraries,
            ExtraCFlags = extraCFlags,
            ExtraLdFlags = extraLdFlags
        };
    }
}
