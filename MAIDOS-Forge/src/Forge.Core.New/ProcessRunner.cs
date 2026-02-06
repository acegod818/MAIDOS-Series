// MAIDOS-Forge Process Runner
// UEP v1.7B Compliant - Zero Technical Debt

using System.Diagnostics;
using System.Text;

namespace Forge.Core.Platform;

/// <summary>
/// 進程執行結果
/// </summary>
public sealed class ProcessResult
{
    public int ExitCode { get; }
    public string Stdout { get; }
    public string Stderr { get; }
    public TimeSpan Duration { get; }
    public bool IsSuccess => ExitCode == 0;

    public ProcessResult(int exitCode, string stdout, string stderr, TimeSpan duration)
    {
        ExitCode = exitCode;
        Stdout = stdout;
        Stderr = stderr;
        Duration = duration;
    }
}

/// <summary>
/// 進程執行配置
/// </summary>
public sealed class ProcessConfig
{
    public string WorkingDirectory { get; init; } = string.Empty;
    public Dictionary<string, string> Environment { get; init; } = new();
    public TimeSpan Timeout { get; init; } = TimeSpan.FromMinutes(10);
    public bool RedirectOutput { get; init; } = true;
}

/// <summary>
/// 外部進程執行器
/// </summary>
public static class ProcessRunner
{
    public static async Task<ProcessResult> RunAsync(
        string command, 
        string arguments, 
        ProcessConfig? config = null,
        CancellationToken ct = default)
    {
        config ??= new ProcessConfig();
        var stopwatch = Stopwatch.StartNew();

        var startInfo = new ProcessStartInfo
        {
            FileName = command,
            Arguments = arguments,
            UseShellExecute = false,
            RedirectStandardOutput = config.RedirectOutput,
            RedirectStandardError = config.RedirectOutput,
            CreateNoWindow = true
        };

        if (!string.IsNullOrEmpty(config.WorkingDirectory))
        {
            startInfo.WorkingDirectory = config.WorkingDirectory;
        }

        foreach (var (key, value) in config.Environment)
        {
            startInfo.EnvironmentVariables[key] = value;
        }

        using var process = new Process { StartInfo = startInfo };
        
        var stdoutBuilder = new StringBuilder();
        var stderrBuilder = new StringBuilder();

        if (config.RedirectOutput)
        {
            process.OutputDataReceived += (_, e) =>
            {
                if (e.Data is not null) stdoutBuilder.AppendLine(e.Data);
            };

            process.ErrorDataReceived += (_, e) =>
            {
                if (e.Data is not null) stderrBuilder.AppendLine(e.Data);
            };
        }

        try
        {
            process.Start();

            if (config.RedirectOutput)
            {
                process.BeginOutputReadLine();
                process.BeginErrorReadLine();
            }

            using var timeoutCts = CancellationTokenSource.CreateLinkedTokenSource(ct);
            timeoutCts.CancelAfter(config.Timeout);

            try
            {
                await process.WaitForExitAsync(timeoutCts.Token);
            }
            catch (OperationCanceledException) when (!ct.IsCancellationRequested)
            {
                // 超時處理
                try 
                { 
                    if (!process.HasExited) process.Kill(entireProcessTree: true); 
                } 
                catch (Exception ex) 
                { 
                    Debug.WriteLine($"[MAIDOS-AUDIT] Failed to kill timed-out process: {ex.Message}"); 
                }

                stopwatch.Stop();
                return new ProcessResult(-1, stdoutBuilder.ToString(), 
                    $"Process timed out after {config.Timeout.TotalSeconds}s", stopwatch.Elapsed);
            }
            catch (OperationCanceledException)
            {
                // 用戶取消處理
                try 
                { 
                    if (!process.HasExited) process.Kill(entireProcessTree: true); 
                } 
                catch (Exception ex) 
                { 
                    Debug.WriteLine($"[MAIDOS-AUDIT] Failed to kill cancelled process: {ex.Message}"); 
                }

                stopwatch.Stop();
                return new ProcessResult(-2, stdoutBuilder.ToString(), 
                    "Process cancelled", stopwatch.Elapsed);
            }

            stopwatch.Stop();
            return new ProcessResult(
                process.ExitCode,
                stdoutBuilder.ToString(),
                stderrBuilder.ToString(),
                stopwatch.Elapsed);
        }
        catch (Exception ex)
        {
            stopwatch.Stop();
            return new ProcessResult(-1, string.Empty, ex.Message, stopwatch.Elapsed);
        }
    }

    public static async Task<bool> CommandExistsAsync(string command, CancellationToken ct = default)
    {
        var whichCommand = OperatingSystem.IsWindows() ? "where" : "which";
        var result = await RunAsync(whichCommand, command, 
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(5) }, ct);
        return result.IsSuccess;
    }

    public static async Task<string?> GetVersionAsync(string command, 
        string versionArg = "--version", CancellationToken ct = default)
    {
        var result = await RunAsync(command, versionArg,
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(10) }, ct);
        
        if (!result.IsSuccess) return null;

        var output = string.IsNullOrEmpty(result.Stdout) ? result.Stderr : result.Stdout;
        return output.Split('\n', StringSplitOptions.RemoveEmptyEntries).FirstOrDefault()?.Trim();
    }
}