// MAIDOS-Forge Odin Language Plugin

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.Odin;

/// <summary>Odin language plugin</summary>
public sealed class OdinPlugin : ILanguagePlugin
{
    private static readonly string[] SourceExtensions = { ".odin" };

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "odin",
        SupportedExtensions = SourceExtensions,
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = false,
        SupportsGlueGeneration = false,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (!await ProcessRunner.CommandExistsAsync("odin"))
            return (false, "odin not found. Install from https://odin-lang.org/docs/install/");
        var version = await ProcessRunner.GetVersionAsync("odin", "version");
        return (true, $"odin {version}");
    }

    public async Task<CompileResult> CompileAsync(ValidatedModuleConfig module, CompileConfig config, CancellationToken ct = default)
    {
        var logs = new List<string>();
        var sw = System.Diagnostics.Stopwatch.StartNew();
        var (ok, msg) = await ValidateToolchainAsync(ct);
        if (!ok) { sw.Stop(); return CompileResult.Failure(msg, logs, sw.Elapsed); }
        logs.Add($"[Odin] Using: {msg}");

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
            var outFile = Path.Combine(outDir, Path.GetFileNameWithoutExtension(f) + ".out");
            logs.Add($"[Odin] Processing: {fn}");
            var r = await ProcessRunner.RunAsync("odin", $"build \"{f}\" -out:\"{outFile}\"",
                new ProcessConfig { WorkingDirectory = Path.GetDirectoryName(f) ?? module.ModulePath, Timeout = TimeSpan.FromMinutes(10) }, ct);
            if (!string.IsNullOrEmpty(r.Stdout)) logs.Add(r.Stdout);
            if (!string.IsNullOrEmpty(r.Stderr)) logs.Add(r.Stderr);
            if (!r.IsSuccess) { sw.Stop(); return CompileResult.Failure($"Failed: {fn}: {r.Stderr}", logs, sw.Elapsed); }
            artifacts.Add(outFile);
        }

        if (artifacts.Count == 0) artifacts.AddRange(Directory.GetFiles(outDir));
        sw.Stop();
        return artifacts.Count > 0 ? CompileResult.Success(artifacts.ToArray(), logs, sw.Elapsed)
            : CompileResult.Failure("No artifacts", logs, sw.Elapsed);
    }

    public Task<InterfaceDescription?> ExtractInterfaceAsync(string artifactPath, CancellationToken ct = default)
        => Task.FromResult<InterfaceDescription?>(new InterfaceDescription
        {
            Version = "1.0",
            Module = new InterfaceModule { Name = Path.GetFileNameWithoutExtension(artifactPath), Version = "1.0.0" },
            Language = new InterfaceLanguage { Name = "odin", Abi = "native" },
            Exports = Array.Empty<ExportedFunction>()
        });

    public GlueCodeResult GenerateGlue(InterfaceDescription src, string target)
        => GlueCodeResult.Failure($"Odin glue generation not supported for {target}");
}
