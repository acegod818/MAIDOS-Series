using Forge.Core;
// MAIDOS-Forge Cap'n Proto Language Plugin
// Code-QC v2.2B Compliant | Tier C Plugin

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;

namespace Forge.Plugin.CapnProto;

/// <summary>Cap'n Proto language plugin</summary>
public sealed class CapnProtoPlugin : ILanguagePlugin
{
    private static readonly string[] SourceExtensions = { ".capnp" };

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "capnproto",
        SupportedExtensions = SourceExtensions,
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = false,
        SupportsGlueGeneration = false,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (!await ProcessRunner.CommandExistsAsync("capnp"))
            return (false, "capnp not found. Install from https://capnproto.org/install.html");
        var version = await ProcessRunner.GetVersionAsync("capnp", "--version");
        return (true, $"capnp {version}");
    }

    public async Task<CompileResult> CompileAsync(ValidatedModuleConfig module, CompileConfig config, CancellationToken ct = default)
    {
        var logs = new List<string>();
        var sw = System.Diagnostics.Stopwatch.StartNew();

        var (ok, msg) = await ValidateToolchainAsync(ct);
        if (!ok) { sw.Stop(); return CompileResult.Failure(msg, logs, sw.Elapsed); }
        logs.Add($"[CapnProto] Using: {msg}");

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir)) srcDir = module.ModulePath;

        var files = SourceExtensions.SelectMany(e => Directory.GetFiles(srcDir, $"*{e}", SearchOption.AllDirectories)).ToArray();
        if (files.Length == 0) { sw.Stop(); return CompileResult.Failure("No source files found", logs, sw.Elapsed); }

        var outDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outDir);
        var beforeFiles = new HashSet<string>(Directory.GetFiles(srcDir, "*", SearchOption.AllDirectories));

        foreach (var f in files)
        {
            var fn = Path.GetFileName(f);
            logs.Add($"[CapnProto] Processing: {fn}");

            var r = await ProcessRunner.RunAsync("capnp", $"compile -oc++ \"{f}\"",
                new ProcessConfig { WorkingDirectory = Path.GetDirectoryName(f) ?? module.ModulePath, Timeout = TimeSpan.FromMinutes(10) }, ct);
            if (!string.IsNullOrEmpty(r.Stdout)) logs.Add(r.Stdout);
            if (!string.IsNullOrEmpty(r.Stderr)) logs.Add(r.Stderr);
            if (!r.IsSuccess) { sw.Stop(); return CompileResult.Failure($"Failed: {fn}: {r.Stderr}", logs, sw.Elapsed); }
        }

        // Collect newly generated files from source directory
        var artifacts = new List<string>();
        foreach (var f in Directory.GetFiles(srcDir, "*", SearchOption.AllDirectories))
        {
            if (!beforeFiles.Contains(f))
            {
                var dest = Path.Combine(outDir, Path.GetFileName(f));
                File.Copy(f, dest, overwrite: true);
                artifacts.Add(dest);
            }
        }

        sw.Stop();
        return artifacts.Count > 0 ? CompileResult.Success(artifacts.ToArray(), logs, sw.Elapsed)
            : CompileResult.Failure("No artifacts produced", logs, sw.Elapsed);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(string artifactPath, CancellationToken ct = default)
        => new InterfaceDescription
        {
            Version = "1.0",
            Module = new InterfaceModule { Name = Path.GetFileNameWithoutExtension(artifactPath), Version = "1.0.0" },
            Language = new InterfaceLanguage { Name = "capnproto", Abi = "native" },
            Exports = (await NativeSymbolExtractor.ExtractFromBinaryAsync(artifactPath, "capnproto", ct)).ToArray()
        });

    public GlueCodeResult GenerateGlue(InterfaceDescription src, string target)
        => NativeSymbolExtractor.GenerateCHeader(src, target);
}
