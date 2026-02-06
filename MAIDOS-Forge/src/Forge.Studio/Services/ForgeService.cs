// MAIDOS-Forge Studio - Forge Service
// Code-QC v2.2B Compliant
// Bridge to Forge.Core

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Runtime.CompilerServices;
using System.Threading;
using System.Threading.Tasks;

namespace Forge.Studio.Services;

/// <summary>
/// Service for interacting with Forge.Core build system
/// </summary>
public class ForgeService
{
    private readonly string _forgePath;

    public ForgeService()
    {
        // Try to find forge CLI
        _forgePath = FindForgeCli();
    }

    private static string FindForgeCli()
    {
        // Check common locations
        var candidates = new[]
        {
            Path.Combine(AppContext.BaseDirectory, "forge"),
            Path.Combine(AppContext.BaseDirectory, "forge.exe"),
            "/usr/local/bin/forge",
            "~/.forge/bin/forge"
        };

        foreach (var path in candidates)
        {
            var expanded = Environment.ExpandEnvironmentVariables(path.Replace("~", Environment.GetFolderPath(Environment.SpecialFolder.UserProfile)));
            if (File.Exists(expanded))
                return expanded;
        }

        // Fallback to dotnet run
        return "dotnet";
    }

    public List<string> GetAvailableLanguages()
    {
        return new List<string>
        {
            // Native/System
            "C", "C++", "Rust", "Zig", "Nim", "Odin", "D", "Ada", "Fortran",
            "Go", "Swift", "V", "Crystal", "Objective-C", "Assembly",
            // Managed/JVM
            "C#", "F#", "Java", "Kotlin", "Scala", "Groovy", "Clojure",
            // Scripting
            "Python", "Ruby", "Lua", "Perl", "PHP", "JavaScript", "R",
            // Functional
            "Haskell", "OCaml", "Erlang", "Elixir",
            // Web/Mobile
            "TypeScript", "Dart", "WebAssembly",
            // Legacy
            "COBOL", "Julia"
        };
    }

    public async IAsyncEnumerable<string> BuildAsync(
        string projectPath,
        string configuration,
        [EnumeratorCancellation] CancellationToken ct = default)
    {
        yield return $"Building project at: {projectPath}";
        yield return $"Configuration: {configuration}";
        yield return "";

        var startInfo = new ProcessStartInfo
        {
            FileName = _forgePath,
            Arguments = _forgePath == "dotnet" 
                ? $"run --project Forge.Cli -- build --config {configuration}"
                : $"build --config {configuration}",
            WorkingDirectory = projectPath,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true
        };

        using var process = new Process { StartInfo = startInfo };

        // NOTE: C# does not allow `yield return` inside a try-block that has a catch/finally.
        // Keep streaming, but keep exception handling outside of any `yield` usage.
        string? startError = null;
        try { process.Start(); }
        catch (Exception ex) { startError = ex.Message; }
        if (startError != null)
        {
            yield return $"Error: {startError}";
            yield break;
        }

        // Read output line by line
        while (!process.StandardOutput.EndOfStream && !ct.IsCancellationRequested)
        {
            string? line = null;
            string? readError = null;
            try { line = await process.StandardOutput.ReadLineAsync(ct); }
            catch (Exception ex) { readError = ex.Message; }

            if (readError != null)
            {
                yield return $"Error: {readError}";
                yield break;
            }
            if (line == null) break;
            yield return line;
        }

        // Read any errors
        string errors = "";
        string? stderrError = null;
        try { errors = await process.StandardError.ReadToEndAsync(ct); }
        catch (Exception ex) { stderrError = ex.Message; }
        if (stderrError != null)
        {
            yield return $"Error: {stderrError}";
            yield break;
        }
        if (!string.IsNullOrWhiteSpace(errors))
        {
            yield return "";
            yield return "=== Errors ===";
            yield return errors;
        }

        string? waitError = null;
        try { await process.WaitForExitAsync(ct); }
        catch (Exception ex) { waitError = ex.Message; }
        if (waitError != null)
        {
            yield return $"Error: {waitError}";
            yield break;
        }

        yield return "";
        yield return process.ExitCode == 0
            ? "[OK] Build completed successfully"
            : $"[FAIL] Build failed with exit code {process.ExitCode}";
    }

    public async Task CleanAsync(string projectPath, CancellationToken ct = default)
    {
        if (string.IsNullOrEmpty(projectPath))
            return;

        var buildDir = Path.Combine(projectPath, "build");
        if (Directory.Exists(buildDir))
        {
            await Task.Run(() => Directory.Delete(buildDir, true), ct);
        }

        var outputDir = Path.Combine(projectPath, "output");
        if (Directory.Exists(outputDir))
        {
            await Task.Run(() => Directory.Delete(outputDir, true), ct);
        }
    }

    public async Task<Dictionary<string, bool>> CheckToolchainsAsync(CancellationToken ct = default)
    {
        var result = new Dictionary<string, bool>();
        
        var toolchains = new Dictionary<string, string>
        {
            ["dotnet"] = "dotnet --version",
            ["rustc"] = "rustc --version",
            ["gcc"] = "gcc --version",
            ["go"] = "go version",
            ["python"] = "python --version",
            ["node"] = "node --version",
            ["zig"] = "zig version",
            ["ghc"] = "ghc --version",
            ["julia"] = "julia --version"
        };

        foreach (var (name, command) in toolchains)
        {
            try
            {
                var parts = command.Split(' ', 2);
                var psi = new ProcessStartInfo
                {
                    FileName = parts[0],
                    Arguments = parts.Length > 1 ? parts[1] : "",
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    UseShellExecute = false,
                    CreateNoWindow = true
                };

                using var process = Process.Start(psi);
                if (process != null)
                {
                    await process.WaitForExitAsync(ct);
                    result[name] = process.ExitCode == 0;
                }
                else
                {
                    result[name] = false;
                }
            }
            catch
            {
                result[name] = false;
            }
        }

        return result;
    }
}
