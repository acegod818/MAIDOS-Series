// MAIDOS-Forge GLSL Language Plugin
// Code-QC v2.2B Compliant | M11 Specialist Plugin - Shader Languages

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;

namespace Forge.Plugin.Glsl;

/// <summary>GLSL 語言插件 - OpenGL Shading Language</summary>
public sealed class GlslPlugin : ILanguagePlugin
{
    private static readonly string[] SourceExtensions = { ".glsl", ".vert", ".frag" };

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "glsl",
        SupportedExtensions = SourceExtensions,
        SupportsNativeCompilation = false,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = false,
        SupportsGlueGeneration = false,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (!await ProcessRunner.CommandExistsAsync("glslangValidator"))
            return (false, "glslangValidator not found. Install from https://www.khronos.org/opengl/wiki/OpenGL_Shading_Language");
        var version = await ProcessRunner.GetVersionAsync("glslangValidator", "--version");
        return (true, $"glslangValidator {version}");
    }

    public async Task<CompileResult> CompileAsync(ValidatedModuleConfig module, CompileConfig config, CancellationToken ct = default)
    {
        var logs = new List<string>();
        var sw = System.Diagnostics.Stopwatch.StartNew();

        var (ok, msg) = await ValidateToolchainAsync(ct);
        if (!ok) { sw.Stop(); return CompileResult.Failure(msg, logs, sw.Elapsed); }
        logs.Add($"[GLSL] Using: {msg}");

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
            var bn = Path.GetFileNameWithoutExtension(f);
            var outFile = Path.Combine(outDir, bn + ".out");
            logs.Add($"[GLSL] Processing: {fn}");

            var spvOut = Path.Combine(outDir, bn + ".spv");
            var r = await ProcessRunner.RunAsync("glslangValidator", $"-V \"{f}\" -o \"{spvOut}\"",
                new ProcessConfig { WorkingDirectory = module.ModulePath, Timeout = TimeSpan.FromMinutes(2) }, ct);
            if (!string.IsNullOrEmpty(r.Stdout)) logs.Add(r.Stdout);
            if (!string.IsNullOrEmpty(r.Stderr)) logs.Add(r.Stderr);
            if (r.IsSuccess && File.Exists(spvOut)) artifacts.Add(spvOut);
        }

        if (artifacts.Count == 0)
            artifacts.AddRange(Directory.GetFiles(outDir).Where(x => !x.EndsWith(".tmp")));

        sw.Stop();
        return artifacts.Count > 0 ? CompileResult.Success(artifacts.ToArray(), logs, sw.Elapsed)
            : CompileResult.Failure("No artifacts", logs, sw.Elapsed);
    }

    public Task<InterfaceDescription?> ExtractInterfaceAsync(string artifactPath, CancellationToken ct = default)
        => Task.FromResult<InterfaceDescription?>(new InterfaceDescription
        {
            Version = "1.0",
            Module = new InterfaceModule { Name = Path.GetFileNameWithoutExtension(artifactPath), Version = "1.0.0" },
            Language = new InterfaceLanguage { Name = "glsl", Abi = "native" },
            Exports = Array.Empty<ExportedFunction>()
        });

    public GlueCodeResult GenerateGlue(InterfaceDescription src, string target)
        => GlueCodeResult.Failure($"GLSL glue generation not supported for {target}");
}
