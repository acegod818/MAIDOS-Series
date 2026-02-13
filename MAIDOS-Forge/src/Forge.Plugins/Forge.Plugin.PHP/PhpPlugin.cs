using Forge.Core;
// MAIDOS-Forge PHP Language Plugin

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.PHP;

/// <summary>PHP language plugin</summary>
public sealed class PhpPlugin : ILanguagePlugin
{
    private static readonly string[] SourceExtensions = { ".php" };

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "php",
        SupportedExtensions = SourceExtensions,
        SupportsNativeCompilation = false,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = false,
        SupportsGlueGeneration = false,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (!await ProcessRunner.CommandExistsAsync("php"))
            return (false, "php not found. Install from https://www.php.net/downloads");
        var version = await ProcessRunner.GetVersionAsync("php", "--version");
        return (true, $"php {version}");
    }

    public async Task<CompileResult> CompileAsync(ValidatedModuleConfig module, CompileConfig config, CancellationToken ct = default)
    {
        var logs = new List<string>();
        var sw = System.Diagnostics.Stopwatch.StartNew();

        var (ok, msg) = await ValidateToolchainAsync(ct);
        if (!ok) { sw.Stop(); return CompileResult.Failure(msg, logs, sw.Elapsed); }
        logs.Add($"[PHP] Using: {msg}");

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir)) srcDir = module.ModulePath;

        var files = SourceExtensions.SelectMany(e => Directory.GetFiles(srcDir, $"*{e}", SearchOption.AllDirectories)).ToArray();
        if (files.Length == 0) { sw.Stop(); return CompileResult.Failure("No source files found", logs, sw.Elapsed); }

        var outDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outDir);
        var artifacts = new List<string>();

        foreach (var f in files)
        {
            var fn = Path.GetFileName(f);
            logs.Add($"[PHP] Syntax check: {fn}");

            var r = await ProcessRunner.RunAsync("php", $"-l \"{f}\"",
                new ProcessConfig { WorkingDirectory = Path.GetDirectoryName(f) ?? module.ModulePath, Timeout = TimeSpan.FromMinutes(2) }, ct);
            if (!string.IsNullOrEmpty(r.Stdout)) logs.Add(r.Stdout);
            if (!string.IsNullOrEmpty(r.Stderr)) logs.Add(r.Stderr);
            if (!r.IsSuccess) { sw.Stop(); return CompileResult.Failure($"Syntax error: {fn}: {r.Stderr}", logs, sw.Elapsed); }

            // Copy validated source as deliverable
            var dest = Path.Combine(outDir, fn);
            File.Copy(f, dest, overwrite: true);
            artifacts.Add(dest);
        }

        sw.Stop();
        return artifacts.Count > 0 ? CompileResult.Success(artifacts.ToArray(), logs, sw.Elapsed)
            : CompileResult.Failure("No artifacts", logs, sw.Elapsed);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(string artifactPath, CancellationToken ct = default)
        => new InterfaceDescription
        {
            Version = "1.0",
            Module = new InterfaceModule { Name = Path.GetFileNameWithoutExtension(artifactPath), Version = "1.0.0" },
            Language = new InterfaceLanguage { Name = "php", Abi = "native" },
            Exports = (await NativeSymbolExtractor.ExtractFromBinaryAsync(artifactPath, "php", ct)).ToArray()
        });

    public GlueCodeResult GenerateGlue(InterfaceDescription src, string target)
        => NativeSymbolExtractor.GenerateCHeader(src, target);
}
