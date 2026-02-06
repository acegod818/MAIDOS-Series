// MAIDOS-Forge Studio - Settings ViewModel
// Code-QC v2.2B Compliant

using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Text.Json;
using Avalonia.Controls;
using Avalonia.Platform.Storage;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;

namespace Forge.Studio.ViewModels;

public partial class SettingsViewModel : ViewModelBase
{
    private readonly Window _window;
    private readonly string _settingsPath;

    [ObservableProperty]
    private SettingsCategory? _selectedCategory;

    [ObservableProperty]
    private string _statusMessage = "";

    [ObservableProperty]
    private ObservableCollection<SettingsCategory> _categories = new();

    // General
    [ObservableProperty]
    private string _defaultProjectPath = "";

    [ObservableProperty]
    private bool _autoSave = true;

    [ObservableProperty]
    private bool _checkUpdates = true;

    // Build
    [ObservableProperty]
    private string _defaultConfig = "Debug";

    [ObservableProperty]
    private bool _parallelBuild = true;

    [ObservableProperty]
    private int _maxParallelJobs = 4;

    [ObservableProperty]
    private bool _incrementalBuild = true;

    // Appearance
    [ObservableProperty]
    private string _theme = "Dark";

    [ObservableProperty]
    private string _fontSize = "Medium";

    [ObservableProperty]
    private bool _showLineNumbers = true;

    // Plugins
    [ObservableProperty]
    private string _pluginPath = "";

    [ObservableProperty]
    private bool _autoLoadPlugins = true;

    [ObservableProperty]
    private bool _checkPluginUpdates = true;

    // Advanced
    [ObservableProperty]
    private bool _verboseLogging;

    [ObservableProperty]
    private bool _keepArtifacts;

    [ObservableProperty]
    private bool _telemetry = true;

    // Visibility
    public bool IsGeneralVisible => SelectedCategory?.Name == "General";
    public bool IsBuildVisible => SelectedCategory?.Name == "Build";
    public bool IsAppearanceVisible => SelectedCategory?.Name == "Appearance";
    public bool IsPluginsVisible => SelectedCategory?.Name == "Plugins";
    public bool IsAdvancedVisible => SelectedCategory?.Name == "Advanced";

    public SettingsViewModel(Window window)
    {
        _window = window;
        _settingsPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
            ".forge", "settings.json");

        // Default paths
        DefaultProjectPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
            "projects");
        PluginPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
            ".forge", "plugins");

        InitializeCategories();
        LoadSettings();
    }

    private void InitializeCategories()
    {
        Categories.Add(new SettingsCategory("⚙️", "General"));
        Categories.Add(new SettingsCategory("🔨", "Build"));
        Categories.Add(new SettingsCategory("🎨", "Appearance"));
        Categories.Add(new SettingsCategory("🔌", "Plugins"));
        Categories.Add(new SettingsCategory("🔧", "Advanced"));

        SelectedCategory = Categories[0];
    }

    partial void OnSelectedCategoryChanged(SettingsCategory? value)
    {
        OnPropertyChanged(nameof(IsGeneralVisible));
        OnPropertyChanged(nameof(IsBuildVisible));
        OnPropertyChanged(nameof(IsAppearanceVisible));
        OnPropertyChanged(nameof(IsPluginsVisible));
        OnPropertyChanged(nameof(IsAdvancedVisible));
    }

    private void LoadSettings()
    {
        try
        {
            if (!File.Exists(_settingsPath))
                return;

            var json = File.ReadAllText(_settingsPath);
            var settings = JsonSerializer.Deserialize<SettingsData>(json);
            
            if (settings == null) return;

            // General
            DefaultProjectPath = settings.DefaultProjectPath ?? DefaultProjectPath;
            AutoSave = settings.AutoSave;
            CheckUpdates = settings.CheckUpdates;

            // Build
            DefaultConfig = settings.DefaultConfig ?? "Debug";
            ParallelBuild = settings.ParallelBuild;
            MaxParallelJobs = settings.MaxParallelJobs;
            IncrementalBuild = settings.IncrementalBuild;

            // Appearance
            Theme = settings.Theme ?? "Dark";
            FontSize = settings.FontSize ?? "Medium";
            ShowLineNumbers = settings.ShowLineNumbers;

            // Plugins
            PluginPath = settings.PluginPath ?? PluginPath;
            AutoLoadPlugins = settings.AutoLoadPlugins;
            CheckPluginUpdates = settings.CheckPluginUpdates;

            // Advanced
            VerboseLogging = settings.VerboseLogging;
            KeepArtifacts = settings.KeepArtifacts;
            Telemetry = settings.Telemetry;

            StatusMessage = "Settings loaded";
        }
        catch (Exception ex)
        {
            StatusMessage = $"Failed to load settings: {ex.Message}";
        }
    }

    [RelayCommand]
    private async System.Threading.Tasks.Task BrowseProjectPath()
    {
        var folders = await _window.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
        {
            Title = "Select Default Project Location",
            AllowMultiple = false,
        });

        var folder = folders.Count > 0 ? folders[0] : null;
        var path = folder?.TryGetLocalPath();
        if (!string.IsNullOrEmpty(path))
            DefaultProjectPath = path;
    }

    [RelayCommand]
    private async System.Threading.Tasks.Task BrowsePluginPath()
    {
        var folders = await _window.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
        {
            Title = "Select Plugin Directory",
            AllowMultiple = false,
        });

        var folder = folders.Count > 0 ? folders[0] : null;
        var path = folder?.TryGetLocalPath();
        if (!string.IsNullOrEmpty(path))
            PluginPath = path;
    }

    [RelayCommand]
    private void Reset()
    {
        // General
        DefaultProjectPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
            "projects");
        AutoSave = true;
        CheckUpdates = true;

        // Build
        DefaultConfig = "Debug";
        ParallelBuild = true;
        MaxParallelJobs = 4;
        IncrementalBuild = true;

        // Appearance
        Theme = "Dark";
        FontSize = "Medium";
        ShowLineNumbers = true;

        // Plugins
        PluginPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
            ".forge", "plugins");
        AutoLoadPlugins = true;
        CheckPluginUpdates = true;

        // Advanced
        VerboseLogging = false;
        KeepArtifacts = false;
        Telemetry = true;

        StatusMessage = "Settings reset to defaults";
    }

    [RelayCommand]
    private void Save()
    {
        try
        {
            var settings = new SettingsData
            {
                // General
                DefaultProjectPath = DefaultProjectPath,
                AutoSave = AutoSave,
                CheckUpdates = CheckUpdates,

                // Build
                DefaultConfig = DefaultConfig,
                ParallelBuild = ParallelBuild,
                MaxParallelJobs = MaxParallelJobs,
                IncrementalBuild = IncrementalBuild,

                // Appearance
                Theme = Theme,
                FontSize = FontSize,
                ShowLineNumbers = ShowLineNumbers,

                // Plugins
                PluginPath = PluginPath,
                AutoLoadPlugins = AutoLoadPlugins,
                CheckPluginUpdates = CheckPluginUpdates,

                // Advanced
                VerboseLogging = VerboseLogging,
                KeepArtifacts = KeepArtifacts,
                Telemetry = Telemetry
            };

            var dir = Path.GetDirectoryName(_settingsPath);
            if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir))
            {
                Directory.CreateDirectory(dir);
            }

            var json = JsonSerializer.Serialize(settings, new JsonSerializerOptions 
            { 
                WriteIndented = true 
            });
            File.WriteAllText(_settingsPath, json);

            StatusMessage = "Settings saved";
            _window.Close(true);
        }
        catch (Exception ex)
        {
            StatusMessage = $"Failed to save: {ex.Message}";
        }
    }

    [RelayCommand]
    private void Cancel()
    {
        _window.Close(false);
    }
}

public class SettingsCategory
{
    public string Icon { get; set; }
    public string Name { get; set; }

    public SettingsCategory(string icon, string name)
    {
        Icon = icon;
        Name = name;
    }
}

public class SettingsData
{
    // General
    public string? DefaultProjectPath { get; set; }
    public bool AutoSave { get; set; } = true;
    public bool CheckUpdates { get; set; } = true;

    // Build
    public string? DefaultConfig { get; set; } = "Debug";
    public bool ParallelBuild { get; set; } = true;
    public int MaxParallelJobs { get; set; } = 4;
    public bool IncrementalBuild { get; set; } = true;

    // Appearance
    public string? Theme { get; set; } = "Dark";
    public string? FontSize { get; set; } = "Medium";
    public bool ShowLineNumbers { get; set; } = true;

    // Plugins
    public string? PluginPath { get; set; }
    public bool AutoLoadPlugins { get; set; } = true;
    public bool CheckPluginUpdates { get; set; } = true;

    // Advanced
    public bool VerboseLogging { get; set; }
    public bool KeepArtifacts { get; set; }
    public bool Telemetry { get; set; } = true;
}
