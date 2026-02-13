using System.Diagnostics;
using System.Text.RegularExpressions;

namespace Forge.Core;

/// <summary>
/// Extracts exported symbols from compiled native binaries using platform tools.
/// Uses 'nm -D' on Linux/macOS and 'dumpbin /EXPORTS' on Windows.
/// Fallback: regex scan of source files for public function declarations.
/// </summary>
public static class NativeSymbolExtractor
{
    /// <summary>
    /// Extract exported functions from a compiled artifact (shared lib / object file).
    /// </summary>
    public static async Task<List<ExportedFunction>> ExtractFromBinaryAsync(
        string artifactPath,
        string language,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();

        if (!File.Exists(artifactPath)) return exports;

        // Try nm first (Linux/macOS), then dumpbin (Windows)
        var symbols = await TryNmAsync(artifactPath, ct)
                   ?? await TryDumpbinAsync(artifactPath, ct);

        if (symbols != null)
        {
            foreach (var sym in symbols)
            {
                exports.Add(new ExportedFunction
                {
                    Name = sym,
                    ReturnType = "i32", // Default; refined by source-level analysis
                    Parameters = Array.Empty<FunctionParameter>()
                });
            }
        }

        return exports;
    }

    /// <summary>
    /// Extract exported functions from source code using language-agnostic regex patterns.
    /// Covers C-style, Rust-style, and many other function declaration syntaxes.
    /// </summary>
    public static async Task<List<ExportedFunction>> ExtractFromSourceAsync(
        string sourcePath,
        string language,
        CancellationToken ct = default)
    {
        var exports = new List<ExportedFunction>();
        if (!File.Exists(sourcePath)) return exports;

        var content = await File.ReadAllTextAsync(sourcePath, ct);
        var patterns = GetPatternsForLanguage(language);

        foreach (var pattern in patterns)
        {
            var regex = new Regex(pattern, RegexOptions.Compiled | RegexOptions.Multiline);
            foreach (Match match in regex.Matches(content))
            {
                var funcName = match.Groups["name"].Success ? match.Groups["name"].Value : match.Groups[1].Value;
                if (string.IsNullOrEmpty(funcName) || funcName.StartsWith("_")) continue;

                exports.Add(new ExportedFunction
                {
                    Name = funcName,
                    ReturnType = match.Groups["ret"].Success ? match.Groups["ret"].Value : "void",
                    Parameters = Array.Empty<FunctionParameter>()
                });
            }
        }

        return exports;
    }

    /// <summary>
    /// Generate a C-compatible header from extracted exports (universal FFI bridge).
    /// </summary>
    public static GlueCodeResult GenerateCHeader(
        InterfaceDescription src,
        string targetLanguage)
    {
        if (src.Exports == null || src.Exports.Length == 0)
            return GlueCodeResult.Failure($"No exports available to generate {targetLanguage} glue");

        var sb = new System.Text.StringBuilder();
        sb.AppendLine($"/* Auto-generated FFI glue: {src.Language.Name} -> {targetLanguage} */");
        sb.AppendLine($"/* Module: {src.Module.Name} v{src.Module.Version} */");
        sb.AppendLine();
        sb.AppendLine("#pragma once");
        sb.AppendLine("#ifdef __cplusplus");
        sb.AppendLine("extern \"C\" {");
        sb.AppendLine("#endif");
        sb.AppendLine();

        foreach (var fn in src.Exports)
        {
            var ret = MapToCType(fn.ReturnType);
            var parms = fn.Parameters?.Length > 0
                ? string.Join(", ", fn.Parameters.Select(p => $"{MapToCType(p.Type)} {p.Name}"))
                : "void";
            sb.AppendLine($"{ret} {fn.Name}({parms});");
        }

        sb.AppendLine();
        sb.AppendLine("#ifdef __cplusplus");
        sb.AppendLine("}");
        sb.AppendLine("#endif");

        var filename = $"{src.Module.Name}_ffi.h";
        return GlueCodeResult.Success(sb.ToString(), filename, targetLanguage);
    }

    // ───────────────────── Private helpers ─────────────────────

    private static async Task<List<string>?> TryNmAsync(string path, CancellationToken ct)
    {
        try
        {
            var psi = new ProcessStartInfo("nm", $"-D --defined-only \"{path}\"")
            {
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false
            };
            using var proc = Process.Start(psi);
            if (proc == null) return null;
            var output = await proc.StandardOutput.ReadToEndAsync(ct);
            await proc.WaitForExitAsync(ct);
            if (proc.ExitCode != 0) return null;

            var symbols = new List<string>();
            var nmRegex = new Regex(@"^\w+\s+T\s+(\w+)", RegexOptions.Multiline);
            foreach (Match m in nmRegex.Matches(output))
                symbols.Add(m.Groups[1].Value);
            return symbols.Count > 0 ? symbols : null;
        }
        catch { return null; }
    }

    private static async Task<List<string>?> TryDumpbinAsync(string path, CancellationToken ct)
    {
        try
        {
            var psi = new ProcessStartInfo("dumpbin", $"/EXPORTS \"{path}\"")
            {
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false
            };
            using var proc = Process.Start(psi);
            if (proc == null) return null;
            var output = await proc.StandardOutput.ReadToEndAsync(ct);
            await proc.WaitForExitAsync(ct);
            if (proc.ExitCode != 0) return null;

            var symbols = new List<string>();
            // dumpbin output: "ordinal hint RVA name"
            var re = new Regex(@"^\s+\d+\s+\w+\s+\w+\s+(\w+)", RegexOptions.Multiline);
            foreach (Match m in re.Matches(output))
                symbols.Add(m.Groups[1].Value);
            return symbols.Count > 0 ? symbols : null;
        }
        catch { return null; }
    }

    private static string[] GetPatternsForLanguage(string lang)
    {
        return lang.ToLowerInvariant() switch
        {
            // C-family
            "c" or "cpp" or "objc" or "d" or "zig" or "odin" or "v" =>
                new[] { @"^\s*(?:pub\s+)?(?:export\s+)?(?:extern\s+)?(?<ret>\w[\w\*\s]*?)\s+(?<name>\w+)\s*\(" },
            // Rust
            "rust" =>
                new[] { @"^\s*pub\s+(?:unsafe\s+)?(?:extern\s+""C""\s+)?fn\s+(?<name>\w+)" },
            // Go
            "go" =>
                new[] { @"^func\s+(?<name>[A-Z]\w*)\s*\(" },
            // Fortran
            "fortran" =>
                new[] { @"(?i)^\s*(?:pure\s+|elemental\s+)?(?:recursive\s+)?(?:subroutine|function)\s+(?<name>\w+)" },
            // Haskell
            "haskell" =>
                new[] { @"^foreign\s+export\s+ccall\s+(?<name>\w+)" },
            // Julia
            "julia" =>
                new[] { @"^(?:export\s+)?function\s+(?<name>\w+)" },
            // Generic fallback (catches most function-like declarations)
            _ =>
                new[] {
                    @"^\s*(?:pub(?:lic)?\s+)?(?:static\s+)?(?:fn|func|function|def|sub|proc|method)\s+(?<name>\w+)",
                    @"^\s*(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s+(?<name>\w+)"
                }
        };
    }

    private static string MapToCType(string forgeType)
    {
        return forgeType?.ToLowerInvariant() switch
        {
            "i8" or "int8" => "int8_t",
            "i16" or "int16" => "int16_t",
            "i32" or "int" or "int32" => "int32_t",
            "i64" or "int64" or "long" => "int64_t",
            "u8" or "uint8" or "byte" => "uint8_t",
            "u16" or "uint16" => "uint16_t",
            "u32" or "uint" or "uint32" => "uint32_t",
            "u64" or "uint64" => "uint64_t",
            "f32" or "float" => "float",
            "f64" or "double" => "double",
            "bool" => "_Bool",
            "string" or "str" => "const char*",
            "void" or null or "" => "void",
            _ => "void*"
        };
    }
}
