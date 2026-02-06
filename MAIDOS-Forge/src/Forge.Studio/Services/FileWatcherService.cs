// MAIDOS-Forge Studio - File Watcher Service
// Code-QC v2.2B Compliant
// Hot reload support for project files

using System;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using System.Threading.Tasks;

namespace Forge.Studio.Services;

/// <summary>
/// Service for monitoring file changes in the project directory
/// Supports hot reload for development workflow
/// </summary>
public class FileWatcherService : IDisposable
{
    private FileSystemWatcher? _watcher;
    private readonly Dictionary<string, DateTime> _lastChangeTime = new();
    private readonly TimeSpan _debounceTime = TimeSpan.FromMilliseconds(300);
    private CancellationTokenSource? _cts;
    private bool _isWatching;

    public event EventHandler<FileChangedEventArgs>? FileChanged;
    public event EventHandler<FileChangedEventArgs>? FileCreated;
    public event EventHandler<FileChangedEventArgs>? FileDeleted;
    public event EventHandler<FileChangedEventArgs>? FileRenamed;
    public event EventHandler<ProjectChangedEventArgs>? ProjectChanged;

    public bool IsWatching => _isWatching;

    /// <summary>
    /// Start watching a project directory
    /// </summary>
    public void StartWatching(string projectPath)
    {
        StopWatching();

        if (!Directory.Exists(projectPath))
            return;

        _cts = new CancellationTokenSource();
        _watcher = new FileSystemWatcher(projectPath)
        {
            IncludeSubdirectories = true,
            NotifyFilter = NotifyFilters.FileName 
                         | NotifyFilters.DirectoryName 
                         | NotifyFilters.LastWrite 
                         | NotifyFilters.Size,
            EnableRaisingEvents = true
        };

        // Add filters for common source files
        // Note: FileSystemWatcher doesn't support multiple filters well,
        // so we filter in the event handlers instead

        _watcher.Changed += OnChanged;
        _watcher.Created += OnCreated;
        _watcher.Deleted += OnDeleted;
        _watcher.Renamed += OnRenamed;
        _watcher.Error += OnError;

        _isWatching = true;
    }

    /// <summary>
    /// Stop watching
    /// </summary>
    public void StopWatching()
    {
        _cts?.Cancel();
        _cts?.Dispose();
        _cts = null;

        if (_watcher != null)
        {
            _watcher.EnableRaisingEvents = false;
            _watcher.Changed -= OnChanged;
            _watcher.Created -= OnCreated;
            _watcher.Deleted -= OnDeleted;
            _watcher.Renamed -= OnRenamed;
            _watcher.Error -= OnError;
            _watcher.Dispose();
            _watcher = null;
        }

        _lastChangeTime.Clear();
        _isWatching = false;
    }

    private void OnChanged(object sender, FileSystemEventArgs e)
    {
        if (ShouldIgnore(e.FullPath))
            return;

        // Debounce - prevent multiple events for same file
        if (!ShouldProcess(e.FullPath))
            return;

        var args = new FileChangedEventArgs(e.FullPath, e.Name ?? "", FileChangeType.Changed);
        FileChanged?.Invoke(this, args);

        // Check if project config changed
        if (IsProjectFile(e.FullPath))
        {
            ProjectChanged?.Invoke(this, new ProjectChangedEventArgs(e.FullPath, ProjectChangeType.ConfigModified));
        }
    }

    private void OnCreated(object sender, FileSystemEventArgs e)
    {
        if (ShouldIgnore(e.FullPath))
            return;

        var args = new FileChangedEventArgs(e.FullPath, e.Name ?? "", FileChangeType.Created);
        FileCreated?.Invoke(this, args);

        // Check if source file added
        if (IsSourceFile(e.FullPath))
        {
            ProjectChanged?.Invoke(this, new ProjectChangedEventArgs(e.FullPath, ProjectChangeType.SourceAdded));
        }
    }

    private void OnDeleted(object sender, FileSystemEventArgs e)
    {
        if (ShouldIgnore(e.FullPath))
            return;

        var args = new FileChangedEventArgs(e.FullPath, e.Name ?? "", FileChangeType.Deleted);
        FileDeleted?.Invoke(this, args);

        // Check if source file removed
        if (IsSourceFile(e.FullPath))
        {
            ProjectChanged?.Invoke(this, new ProjectChangedEventArgs(e.FullPath, ProjectChangeType.SourceRemoved));
        }
    }

    private void OnRenamed(object sender, RenamedEventArgs e)
    {
        if (ShouldIgnore(e.FullPath))
            return;

        var args = new FileChangedEventArgs(e.FullPath, e.Name ?? "", FileChangeType.Renamed, e.OldFullPath);
        FileRenamed?.Invoke(this, args);
    }

    private void OnError(object sender, ErrorEventArgs e)
    {
        // Log error and try to recover
        var ex = e.GetException();
        Console.WriteLine($"FileWatcher error: {ex.Message}");

        // Try to restart watcher
        if (_watcher != null)
        {
            var path = _watcher.Path;
            StopWatching();
            Task.Delay(1000).ContinueWith(_ => StartWatching(path));
        }
    }

    private bool ShouldIgnore(string path)
    {
        // Ignore build output and hidden directories
        var ignoredPaths = new[]
        {
            "/bin/", "\\bin\\",
            "/obj/", "\\obj\\",
            "/build/", "\\build\\",
            "/output/", "\\output\\",
            "/node_modules/", "\\node_modules\\",
            "/target/", "\\target\\",
            "/.git/", "\\.git\\",
            "/.vs/", "\\.vs\\",
            "/.idea/", "\\.idea\\"
        };

        foreach (var ignored in ignoredPaths)
        {
            if (path.Contains(ignored, StringComparison.OrdinalIgnoreCase))
                return true;
        }

        // Ignore temp files
        var fileName = Path.GetFileName(path);
        if (fileName.StartsWith('.') || fileName.StartsWith('~') || fileName.EndsWith(".tmp"))
            return true;

        return false;
    }

    private bool ShouldProcess(string path)
    {
        var now = DateTime.UtcNow;
        
        lock (_lastChangeTime)
        {
            if (_lastChangeTime.TryGetValue(path, out var lastTime))
            {
                if (now - lastTime < _debounceTime)
                    return false;
            }
            
            _lastChangeTime[path] = now;
        }

        return true;
    }

    private static bool IsProjectFile(string path)
    {
        var fileName = Path.GetFileName(path);
        return fileName.Equals("forge.json", StringComparison.OrdinalIgnoreCase)
            || fileName.Equals("module.json", StringComparison.OrdinalIgnoreCase);
    }

    private static bool IsSourceFile(string path)
    {
        var ext = Path.GetExtension(path).ToLowerInvariant();
        var sourceExtensions = new HashSet<string>
        {
            // Native
            ".c", ".h", ".cpp", ".hpp", ".cc", ".cxx",
            ".rs", ".zig", ".nim", ".d", ".ada", ".adb", ".ads",
            ".f90", ".f95", ".f03", ".f08",
            ".go", ".swift", ".m", ".mm",
            ".v", ".cr", ".odin",
            ".s", ".asm",
            
            // Managed
            ".cs", ".fs", ".java", ".kt", ".scala", ".groovy", ".clj",
            
            // Scripting
            ".py", ".rb", ".lua", ".pl", ".php", ".js", ".ts", ".r",
            
            // Functional
            ".hs", ".ml", ".mli", ".erl", ".ex", ".exs",
            
            // Web
            ".dart", ".wat", ".wasm",
            
            // Legacy
            ".cob", ".cbl", ".jl"
        };

        return sourceExtensions.Contains(ext);
    }

    public void Dispose()
    {
        StopWatching();
        GC.SuppressFinalize(this);
    }
}

public class FileChangedEventArgs : EventArgs
{
    public string FullPath { get; }
    public string FileName { get; }
    public FileChangeType ChangeType { get; }
    public string? OldPath { get; }

    public FileChangedEventArgs(string fullPath, string fileName, FileChangeType changeType, string? oldPath = null)
    {
        FullPath = fullPath;
        FileName = fileName;
        ChangeType = changeType;
        OldPath = oldPath;
    }
}

public class ProjectChangedEventArgs : EventArgs
{
    public string FilePath { get; }
    public ProjectChangeType ChangeType { get; }

    public ProjectChangedEventArgs(string filePath, ProjectChangeType changeType)
    {
        FilePath = filePath;
        ChangeType = changeType;
    }
}

public enum FileChangeType
{
    Changed,
    Created,
    Deleted,
    Renamed
}

public enum ProjectChangeType
{
    ConfigModified,
    SourceAdded,
    SourceRemoved,
    DependencyChanged
}
