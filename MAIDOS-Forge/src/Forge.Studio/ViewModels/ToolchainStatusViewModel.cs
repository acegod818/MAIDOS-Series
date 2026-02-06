// MAIDOS-Forge Studio - Toolchain Status ViewModel
// Code-QC v2.2B Compliant

using System;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Media;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;

namespace Forge.Studio.ViewModels;

public partial class ToolchainStatusViewModel : ViewModelBase
{
    private readonly Window _window;

    [ObservableProperty]
    private bool _isChecking;

    [ObservableProperty]
    private string _checkingMessage = "";

    [ObservableProperty]
    private ObservableCollection<ToolchainInfo> _toolchains = new();

    public string StatusSummary
    {
        get
        {
            var available = Toolchains.Count(t => t.IsAvailable);
            var total = Toolchains.Count;
            return $"{available}/{total} toolchains available";
        }
    }

    public ToolchainStatusViewModel(Window window)
    {
        _window = window;
        InitializeToolchains();
        _ = CheckAllToolchainsAsync();
    }

    private void InitializeToolchains()
    {
        Toolchains.Add(new ToolchainInfo("dotnet", ".NET SDK", "C#, F#"));
        Toolchains.Add(new ToolchainInfo("rustc", "Rust Compiler", "Rust"));
        Toolchains.Add(new ToolchainInfo("cargo", "Cargo", "Rust"));
        Toolchains.Add(new ToolchainInfo("gcc", "GCC", "C, C++"));
        Toolchains.Add(new ToolchainInfo("clang", "Clang/LLVM", "C, C++, Objective-C"));
        Toolchains.Add(new ToolchainInfo("go", "Go", "Go"));
        Toolchains.Add(new ToolchainInfo("python3", "Python 3", "Python"));
        Toolchains.Add(new ToolchainInfo("node", "Node.js", "JavaScript, TypeScript"));
        Toolchains.Add(new ToolchainInfo("zig", "Zig", "Zig"));
        Toolchains.Add(new ToolchainInfo("nim", "Nim", "Nim"));
        Toolchains.Add(new ToolchainInfo("julia", "Julia", "Julia"));
        Toolchains.Add(new ToolchainInfo("ghc", "GHC", "Haskell"));
        Toolchains.Add(new ToolchainInfo("javac", "Java Compiler", "Java"));
        Toolchains.Add(new ToolchainInfo("kotlinc", "Kotlin Compiler", "Kotlin"));
        Toolchains.Add(new ToolchainInfo("ruby", "Ruby", "Ruby"));
        Toolchains.Add(new ToolchainInfo("lua", "Lua", "Lua"));
        Toolchains.Add(new ToolchainInfo("perl", "Perl", "Perl"));
        Toolchains.Add(new ToolchainInfo("php", "PHP", "PHP"));
        Toolchains.Add(new ToolchainInfo("erl", "Erlang", "Erlang"));
        Toolchains.Add(new ToolchainInfo("elixir", "Elixir", "Elixir"));
        Toolchains.Add(new ToolchainInfo("nasm", "NASM", "Assembly"));
        Toolchains.Add(new ToolchainInfo("swiftc", "Swift", "Swift"));
        Toolchains.Add(new ToolchainInfo("gfortran", "GNU Fortran", "Fortran"));
        Toolchains.Add(new ToolchainInfo("cobc", "GnuCOBOL", "COBOL"));
    }

    [RelayCommand]
    private async Task Refresh()
    {
        await CheckAllToolchainsAsync();
    }

    private async Task CheckAllToolchainsAsync()
    {
        IsChecking = true;

        foreach (var toolchain in Toolchains)
        {
            CheckingMessage = $"Checking {toolchain.Name}...";
            await CheckToolchainAsync(toolchain);
        }

        IsChecking = false;
        CheckingMessage = "";
        OnPropertyChanged(nameof(StatusSummary));
    }

    private async Task CheckToolchainAsync(ToolchainInfo toolchain)
    {
        try
        {
            var versionArg = toolchain.Command switch
            {
                "dotnet" => "--version",
                "go" => "version",
                "julia" => "--version",
                "erl" => "+V",
                _ => "--version"
            };

            var psi = new ProcessStartInfo
            {
                FileName = toolchain.Command,
                Arguments = versionArg,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true
            };

            using var process = Process.Start(psi);
            if (process == null)
            {
                SetToolchainNotFound(toolchain);
                return;
            }

            var output = await process.StandardOutput.ReadToEndAsync();
            var error = await process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync();

            if (process.ExitCode == 0 || !string.IsNullOrEmpty(output))
            {
                var version = ParseVersion(output + error, toolchain.Command);
                SetToolchainAvailable(toolchain, version);
                
                // Try to find path
                toolchain.Path = await GetToolchainPathAsync(toolchain.Command);
            }
            else
            {
                SetToolchainNotFound(toolchain);
            }
        }
        catch
        {
            SetToolchainNotFound(toolchain);
        }
    }

    private static string ParseVersion(string output, string command)
    {
        var lines = output.Split('\n', StringSplitOptions.RemoveEmptyEntries);
        if (lines.Length == 0) return "Unknown version";

        var firstLine = lines[0].Trim();
        
        // Extract version number
        var versionPatterns = new[]
        {
            @"(\d+\.\d+\.\d+)",
            @"version\s+(\d+\.\d+)",
            @"(\d+\.\d+)"
        };

        foreach (var pattern in versionPatterns)
        {
            var match = System.Text.RegularExpressions.Regex.Match(firstLine, pattern);
            if (match.Success)
            {
                return $"v{match.Groups[1].Value}";
            }
        }

        return firstLine.Length > 50 ? firstLine[..50] + "..." : firstLine;
    }

    private static async Task<string> GetToolchainPathAsync(string command)
    {
        try
        {
            var whichCommand = OperatingSystem.IsWindows() ? "where" : "which";
            var psi = new ProcessStartInfo
            {
                FileName = whichCommand,
                Arguments = command,
                RedirectStandardOutput = true,
                UseShellExecute = false,
                CreateNoWindow = true
            };

            using var process = Process.Start(psi);
            if (process == null) return "";

            var output = await process.StandardOutput.ReadToEndAsync();
            await process.WaitForExitAsync();

            var path = output.Split('\n', StringSplitOptions.RemoveEmptyEntries).FirstOrDefault()?.Trim();
            return path ?? "";
        }
        catch
        {
            return "";
        }
    }

    private static void SetToolchainAvailable(ToolchainInfo toolchain, string version)
    {
        toolchain.IsAvailable = true;
        toolchain.Version = version;
        toolchain.StatusText = "Available";
        toolchain.StatusIcon = "✓";
        toolchain.StatusColor = new SolidColorBrush(Color.Parse("#00B894"));
        toolchain.StatusBackground = new SolidColorBrush(Color.Parse("#1E3A2F"));
    }

    private static void SetToolchainNotFound(ToolchainInfo toolchain)
    {
        toolchain.IsAvailable = false;
        toolchain.Version = "Not installed";
        toolchain.StatusText = "Not Found";
        toolchain.StatusIcon = "✕";
        toolchain.StatusColor = new SolidColorBrush(Color.Parse("#636E72"));
        toolchain.StatusBackground = new SolidColorBrush(Color.Parse("#2D3436"));
        toolchain.Path = "";
    }

    [RelayCommand]
    private void Close()
    {
        _window.Close();
    }
}

public class ToolchainInfo : ObservableObject
{
    private bool _isAvailable;
    private string _version = "Checking...";
    private string _statusText = "Checking";
    private string _statusIcon = "⏳";
    private string _path = "";
    private IBrush _statusColor = new SolidColorBrush(Color.Parse("#FDCB6E"));
    private IBrush _statusBackground = new SolidColorBrush(Color.Parse("#3D3A28"));

    public string Command { get; set; }
    public string Name { get; set; }
    public string Languages { get; set; }

    public bool IsAvailable
    {
        get => _isAvailable;
        set => SetProperty(ref _isAvailable, value);
    }

    public string Version
    {
        get => _version;
        set => SetProperty(ref _version, value);
    }

    public string StatusText
    {
        get => _statusText;
        set => SetProperty(ref _statusText, value);
    }

    public string StatusIcon
    {
        get => _statusIcon;
        set => SetProperty(ref _statusIcon, value);
    }

    public string Path
    {
        get => _path;
        set => SetProperty(ref _path, value);
    }

    public IBrush StatusColor
    {
        get => _statusColor;
        set => SetProperty(ref _statusColor, value);
    }

    public IBrush StatusBackground
    {
        get => _statusBackground;
        set => SetProperty(ref _statusBackground, value);
    }

    public ToolchainInfo(string command, string name, string languages)
    {
        Command = command;
        Name = name;
        Languages = languages;
    }
}
