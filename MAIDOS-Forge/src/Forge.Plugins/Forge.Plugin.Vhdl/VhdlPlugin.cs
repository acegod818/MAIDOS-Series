// MAIDOS-Forge VHDL Language Plugin
// Code-QC v2.2B Compliant | M11 Specialist Plugin - Hardware Languages

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;

namespace Forge.Plugin.Vhdl;

/// <summary>
/// VHDL 語言插件 - VHSIC Hardware Description Language
/// </summary>
public sealed class VhdlPlugin : ILanguagePlugin
{
    private static readonly string[] SourceExtensions = { ".vhd", ".vhdl" };

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "vhdl",
        SupportedExtensions = SourceExtensions,
        SupportsNativeCompilation = false,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = false,
        SupportsGlueGeneration = false,
        SupportedTargets = new[] { "simulation", "synthesis" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (!await ProcessRunner.CommandExistsAsync("ghdl"))
            return (false, "GHDL not found. Install from https://github.com/ghdl/ghdl");
        var version = await ProcessRunner.GetVersionAsync("ghdl", "--version");
        return (true, $"GHDL {version}");
    }

    public async Task<CompileResult> CompileAsync(ValidatedModuleConfig module, CompileConfig config, CancellationToken ct = default)
    {
        var logs = new List<string>();
        var sw = System.Diagnostics.Stopwatch.StartNew();

        var (ok, msg) = await ValidateToolchainAsync(ct);
        if (!ok) { sw.Stop(); return CompileResult.Failure(msg, logs, sw.Elapsed); }
        logs.Add($"[VHDL] Using: {msg}");

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir)) srcDir = module.ModulePath;

        var files = SourceExtensions.SelectMany(e => Directory.GetFiles(srcDir, $"*{e}", SearchOption.AllDirectories)).ToArray();
        if (files.Length == 0) { sw.Stop(); return CompileResult.Failure("No VHDL files found", logs, sw.Elapsed); }

        var outDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outDir);
        var artifacts = new List<string>();

        // 分析階段
        foreach (var f in files)
        {
            logs.Add($"[VHDL] Analyzing: {Path.GetFileName(f)}");
            var r = await ProcessRunner.RunAsync("ghdl", $"-a --workdir=\"{outDir}\" \"{f}\"",
                new ProcessConfig { WorkingDirectory = module.ModulePath, Timeout = TimeSpan.FromMinutes(5) }, ct);
            if (!string.IsNullOrEmpty(r.Stdout)) logs.Add(r.Stdout);
            if (!string.IsNullOrEmpty(r.Stderr)) logs.Add(r.Stderr);
            if (!r.IsSuccess) { sw.Stop(); return CompileResult.Failure($"Analysis failed: {f}", logs, sw.Elapsed); }
        }

        // 建立可執行模擬器
        var topEntity = module.Config.Options?.GetValueOrDefault("top_entity") ?? Path.GetFileNameWithoutExtension(files[0]);
        var simExe = Path.Combine(outDir, topEntity);
        logs.Add($"[VHDL] Elaborating: {topEntity}");
        var er = await ProcessRunner.RunAsync("ghdl", $"-e --workdir=\"{outDir}\" -o \"{simExe}\" {topEntity}",
            new ProcessConfig { WorkingDirectory = outDir, Timeout = TimeSpan.FromMinutes(5) }, ct);
        if (!string.IsNullOrEmpty(er.Stdout)) logs.Add(er.Stdout);
        if (!string.IsNullOrEmpty(er.Stderr)) logs.Add(er.Stderr);

        if (er.IsSuccess && File.Exists(simExe)) artifacts.Add(simExe);
        artifacts.AddRange(Directory.GetFiles(outDir, "*.o"));
        artifacts.AddRange(Directory.GetFiles(outDir, "*.cf"));

        sw.Stop();
        return artifacts.Count > 0 ? CompileResult.Success(artifacts.ToArray(), logs, sw.Elapsed)
            : CompileResult.Failure("No artifacts", logs, sw.Elapsed);
    }

    public Task<InterfaceDescription?> ExtractInterfaceAsync(string artifactPath, CancellationToken ct = default)
        => Task.FromResult<InterfaceDescription?>(new InterfaceDescription
        {
            Version = "1.0",
            Module = new InterfaceModule { Name = Path.GetFileNameWithoutExtension(artifactPath), Version = "1.0.0" },
            Language = new InterfaceLanguage { Name = "vhdl", Abi = "simulation" },
            Exports = Array.Empty<ExportedFunction>()
        });

    public GlueCodeResult GenerateGlue(InterfaceDescription src, string target)
        => GlueCodeResult.Failure($"VHDL glue generation not supported for {target}");
}
