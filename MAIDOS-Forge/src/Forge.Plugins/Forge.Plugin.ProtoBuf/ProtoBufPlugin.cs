using Forge.Core;
// MAIDOS-Forge ProtoBuf Language Plugin

using System.Text;
using Forge.Core.Config;
using Forge.Core.Platform;
using Forge.Core.Plugin;

namespace Forge.Plugin.ProtoBuf;

/// <summary>ProtoBuf language plugin</summary>
public sealed class ProtoBufPlugin : ILanguagePlugin
{
    private static readonly string[] SourceExtensions = { ".proto" };

    public PluginCapabilities GetCapabilities() => new()
    {
        LanguageName = "protobuf",
        SupportedExtensions = SourceExtensions,
        SupportsNativeCompilation = true,
        SupportsCrossCompilation = false,
        SupportsInterfaceExtraction = false,
        SupportsGlueGeneration = false,
        SupportedTargets = new[] { "native" }
    };

    public async Task<(bool Available, string Message)> ValidateToolchainAsync(CancellationToken ct = default)
    {
        if (!await ProcessRunner.CommandExistsAsync("protoc"))
            return (false, "protoc not found. Install from https://grpc.io/docs/protoc-installation/");
        var version = await ProcessRunner.GetVersionAsync("protoc", "--version");
        return (true, $"protoc {version}");
    }

    public async Task<CompileResult> CompileAsync(ValidatedModuleConfig module, CompileConfig config, CancellationToken ct = default)
    {
        var logs = new List<string>();
        var sw = System.Diagnostics.Stopwatch.StartNew();

        var (ok, msg) = await ValidateToolchainAsync(ct);
        if (!ok) { sw.Stop(); return CompileResult.Failure(msg, logs, sw.Elapsed); }
        logs.Add($"[ProtoBuf] Using: {msg}");

        var srcDir = Path.Combine(module.ModulePath, "src");
        if (!Directory.Exists(srcDir)) srcDir = module.ModulePath;

        var files = SourceExtensions.SelectMany(e => Directory.GetFiles(srcDir, $"*{e}", SearchOption.AllDirectories)).ToArray();
        if (files.Length == 0) { sw.Stop(); return CompileResult.Failure("No source files found", logs, sw.Elapsed); }

        var outDir = Path.Combine(config.OutputDir, module.Config.Name);
        Directory.CreateDirectory(outDir);

        foreach (var f in files)
        {
            var fn = Path.GetFileName(f);
            logs.Add($"[ProtoBuf] Processing: {fn}");

            var r = await ProcessRunner.RunAsync("protoc", $"--proto_path=\"{srcDir}\" --csharp_out=\"{outDir}\" \"{f}\"",
                new ProcessConfig { WorkingDirectory = Path.GetDirectoryName(f) ?? module.ModulePath, Timeout = TimeSpan.FromMinutes(10) }, ct);
            if (!string.IsNullOrEmpty(r.Stdout)) logs.Add(r.Stdout);
            if (!string.IsNullOrEmpty(r.Stderr)) logs.Add(r.Stderr);
            if (!r.IsSuccess) { sw.Stop(); return CompileResult.Failure($"Failed: {fn}: {r.Stderr}", logs, sw.Elapsed); }
        }

        var artifacts = Directory.GetFiles(outDir, "*", SearchOption.AllDirectories)
            .Where(x => !x.EndsWith(".tmp"))
            .ToList();

        sw.Stop();
        return artifacts.Count > 0 ? CompileResult.Success(artifacts.ToArray(), logs, sw.Elapsed)
            : CompileResult.Failure("No artifacts produced", logs, sw.Elapsed);
    }

    public async Task<InterfaceDescription?> ExtractInterfaceAsync(string artifactPath, CancellationToken ct = default)
        => new InterfaceDescription
        {
            Version = "1.0",
            Module = new InterfaceModule { Name = Path.GetFileNameWithoutExtension(artifactPath), Version = "1.0.0" },
            Language = new InterfaceLanguage { Name = "protobuf", Abi = "native" },
            Exports = (await NativeSymbolExtractor.ExtractFromBinaryAsync(artifactPath, "protobuf", ct)).ToArray()
        });

    public GlueCodeResult GenerateGlue(InterfaceDescription src, string target)
        => NativeSymbolExtractor.GenerateCHeader(src, target);
}
