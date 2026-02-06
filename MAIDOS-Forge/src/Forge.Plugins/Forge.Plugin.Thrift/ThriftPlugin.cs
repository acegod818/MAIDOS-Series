// MAIDOS-Forge Thrift Language Plugin

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;

namespace Forge.Plugin.Thrift;

/// <summary>Apache Thrift IDL plugin</summary>
public sealed class ThriftPlugin : ILanguagePlugin
{
    private static readonly string[] SourceExtensions = { ".thrift" };

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "thrift",
        SupportedExtensions = SourceExtensions,
        SupportsNativeCompilation = false,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = false,
        SupportsGlueGeneration = false,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (!await ProcessRunner.CommandExistsAsync("thrift"))
            return (false, "thrift not found. Install from https://thrift.apache.org/");
        var version = await ProcessRunner.GetVersionAsync("thrift", "--version");
        return (true, $"thrift {version}");
    }

    public async Task<CompileResult> CompileAsync(ValidatedModuleConfig module, CompileConfig config, CancellationToken ct = default)
    {
        var logs = new List<string>();
        var sw = System.Diagnostics.Stopwatch.StartNew();
        var (ok, msg) = await ValidateToolchainAsync(ct);
        if (!ok) { sw.Stop(); return CompileResult.Failure(msg, logs, sw.Elapsed); }
        logs.Add($"[Thrift] Using: {msg}");
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
            logs.Add($"[Thrift] Processing: {fn}");
            var r = await ProcessRunner.RunAsync("thrift", $"--gen csharp -o \"{outDir}\" \"{f}\"",
                new ProcessConfig { WorkingDirectory = Path.GetDirectoryName(f) ?? module.ModulePath, Timeout = TimeSpan.FromMinutes(10) }, ct);
            if (!string.IsNullOrEmpty(r.Stdout)) logs.Add(r.Stdout);
            if (!string.IsNullOrEmpty(r.Stderr)) logs.Add(r.Stderr);
            if (!r.IsSuccess) { sw.Stop(); return CompileResult.Failure($"Failed: {fn}: {r.Stderr}", logs, sw.Elapsed); }
        }
        artifacts.AddRange(Directory.GetFiles(outDir, "*", SearchOption.AllDirectories));
        sw.Stop();
        return artifacts.Count > 0 ? CompileResult.Success(artifacts.ToArray(), logs, sw.Elapsed)
            : CompileResult.Failure("No artifacts", logs, sw.Elapsed);
    }

    public Task<InterfaceDescription?> ExtractInterfaceAsync(string artifactPath, CancellationToken ct = default)
        => Task.FromResult<InterfaceDescription?>(new InterfaceDescription
        {
            Version = "1.0",
            Module = new InterfaceModule { Name = Path.GetFileNameWithoutExtension(artifactPath), Version = "1.0.0" },
            Language = new InterfaceLanguage { Name = "thrift", Abi = "native" },
            Exports = Array.Empty<ExportedFunction>()
        });

    public GlueCodeResult GenerateGlue(InterfaceDescription src, string target)
        => GlueCodeResult.Failure($"Thrift glue generation not supported for {target}");
}
