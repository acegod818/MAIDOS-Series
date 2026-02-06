// MAIDOS-Forge Studio - New Project ViewModel
// Code-QC v2.2B Compliant

using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Text.RegularExpressions;
using Avalonia.Controls;
using Avalonia.Platform.Storage;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Forge.Studio.Services;

namespace Forge.Studio.ViewModels;

public partial class NewProjectViewModel : ViewModelBase
{
    private readonly Window _window;
    private readonly ProjectService _projectService;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(CanCreate))]
    private string _projectName = "";

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(CanCreate))]
    private string _projectPath = "";

    [ObservableProperty]
    private ProjectTemplate? _selectedTemplate;

    [ObservableProperty]
    private string _validationMessage = "";

    [ObservableProperty]
    private ObservableCollection<ProjectTemplate> _templates = new();

    [ObservableProperty]
    private ObservableCollection<InitialModule> _initialModules = new();

    [ObservableProperty]
    private ObservableCollection<string> _availableLanguages = new();

    public bool CanCreate => !string.IsNullOrWhiteSpace(ProjectName) 
                           && !string.IsNullOrWhiteSpace(ProjectPath)
                           && IsValidProjectName(ProjectName);

    public string? CreatedProjectPath { get; private set; }

    public NewProjectViewModel(Window window, ProjectService projectService)
    {
        _window = window;
        _projectService = projectService;

        // Default path
        ProjectPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
            "projects");

        InitializeTemplates();
        InitializeLanguages();
    }

    private void InitializeTemplates()
    {
        Templates.Add(new ProjectTemplate
        {
            Icon = "📦",
            Name = "Empty Project",
            Description = "Start with a blank slate. Add modules manually as needed.",
            Languages = "Any",
            ModuleCount = 0
        });

        Templates.Add(new ProjectTemplate
        {
            Icon = "🦀",
            Name = "Rust + C# Interop",
            Description = "High-performance Rust core with C# bindings. Ideal for game engines, native libraries with managed interfaces.",
            Languages = "Rust, C#",
            ModuleCount = 2
        });

        Templates.Add(new ProjectTemplate
        {
            Icon = "🌐",
            Name = "Full-Stack Web",
            Description = "TypeScript frontend, Go backend, with shared types. Modern web application architecture.",
            Languages = "TypeScript, Go",
            ModuleCount = 3
        });

        Templates.Add(new ProjectTemplate
        {
            Icon = "🔬",
            Name = "Scientific Computing",
            Description = "Python for data analysis, Julia for numerical computation, C for performance-critical code.",
            Languages = "Python, Julia, C",
            ModuleCount = 3
        });

        Templates.Add(new ProjectTemplate
        {
            Icon = "🎮",
            Name = "Game Engine",
            Description = "C++ engine core, Lua scripting, C# tools. Classic game development architecture.",
            Languages = "C++, Lua, C#",
            ModuleCount = 3
        });

        Templates.Add(new ProjectTemplate
        {
            Icon = "🤖",
            Name = "Embedded Systems",
            Description = "C for hardware interface, Rust for safe systems code, Python for tooling.",
            Languages = "C, Rust, Python",
            ModuleCount = 3
        });

        SelectedTemplate = Templates[0];
    }

    private void InitializeLanguages()
    {
        var languages = new[]
        {
            "C", "C++", "C#", "Rust", "Go", "Python", "TypeScript", "JavaScript",
            "Java", "Kotlin", "Scala", "Swift", "Zig", "Nim", "D", "Haskell",
            "OCaml", "F#", "Ruby", "Lua", "Julia", "R", "Erlang", "Elixir"
        };

        foreach (var lang in languages)
        {
            AvailableLanguages.Add(lang);
        }
    }

    private static bool IsValidProjectName(string name)
    {
        return Regex.IsMatch(name, @"^[a-z][a-z0-9\-]*$");
    }

    [RelayCommand]
    private async System.Threading.Tasks.Task Browse()
    {
        var folders = await _window.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
        {
            Title = "Select Project Location",
            AllowMultiple = false,
        });

        var folder = folders.Count > 0 ? folders[0] : null;
        var path = folder?.TryGetLocalPath();
        if (!string.IsNullOrEmpty(path))
            ProjectPath = path;
    }

    [RelayCommand]
    private void AddModule()
    {
        InitialModules.Add(new InitialModule
        {
            Name = $"module{InitialModules.Count + 1}",
            Language = "C#"
        });
    }

    [RelayCommand]
    private void RemoveModule(InitialModule? module)
    {
        if (module != null)
        {
            InitialModules.Remove(module);
        }
    }

    [RelayCommand]
    private void Cancel()
    {
        _window.Close(null);
    }

    [RelayCommand]
    private void Create()
    {
        if (!CanCreate)
            return;

        // Validate project name
        if (!IsValidProjectName(ProjectName))
        {
            ValidationMessage = "Project name must start with a letter and contain only lowercase letters, numbers, and hyphens.";
            return;
        }

        // Create full path
        var fullPath = Path.Combine(ProjectPath, ProjectName);

        // Check if directory exists
        if (Directory.Exists(fullPath))
        {
            ValidationMessage = "A project with this name already exists at this location.";
            return;
        }

        try
        {
            // Create project
            _projectService.CreateNewProject(fullPath, ProjectName);

            // Add initial modules based on template or user selection
            if (SelectedTemplate != null && SelectedTemplate.ModuleCount > 0)
            {
                AddTemplateModules(SelectedTemplate);
            }

            foreach (var module in InitialModules)
            {
                if (!string.IsNullOrWhiteSpace(module.Name))
                {
                    _projectService.AddModule(module.Name, module.Language, module.Name);
                }
            }

            _projectService.SaveProject();

            CreatedProjectPath = fullPath;
            _window.Close(fullPath);
        }
        catch (Exception ex)
        {
            ValidationMessage = $"Failed to create project: {ex.Message}";
        }
    }

    private void AddTemplateModules(ProjectTemplate template)
    {
        switch (template.Name)
        {
            case "Rust + C# Interop":
                _projectService.AddModule("core", "Rust", "core");
                _projectService.AddModule("bindings", "C#", "bindings");
                break;

            case "Full-Stack Web":
                _projectService.AddModule("frontend", "TypeScript", "frontend");
                _projectService.AddModule("backend", "Go", "backend");
                _projectService.AddModule("shared", "TypeScript", "shared");
                break;

            case "Scientific Computing":
                _projectService.AddModule("analysis", "Python", "analysis");
                _projectService.AddModule("compute", "Julia", "compute");
                _projectService.AddModule("native", "C", "native");
                break;

            case "Game Engine":
                _projectService.AddModule("engine", "C++", "engine");
                _projectService.AddModule("scripts", "Lua", "scripts");
                _projectService.AddModule("editor", "C#", "editor");
                break;

            case "Embedded Systems":
                _projectService.AddModule("hal", "C", "hal");
                _projectService.AddModule("firmware", "Rust", "firmware");
                _projectService.AddModule("tools", "Python", "tools");
                break;
        }
    }
}

public class ProjectTemplate
{
    public string Icon { get; set; } = "";
    public string Name { get; set; } = "";
    public string Description { get; set; } = "";
    public string Languages { get; set; } = "";
    public int ModuleCount { get; set; }
}

public class InitialModule : ObservableObject
{
    private string _name = "";
    private string _language = "C#";

    public string Name
    {
        get => _name;
        set => SetProperty(ref _name, value);
    }

    public string Language
    {
        get => _language;
        set => SetProperty(ref _language, value);
    }
}
