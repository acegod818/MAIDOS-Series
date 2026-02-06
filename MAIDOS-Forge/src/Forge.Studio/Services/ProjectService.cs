// MAIDOS-Forge Studio - Project Service
// Code-QC v2.2B Compliant

using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using Forge.Studio.Models;
using Forge.Studio.ViewModels;

namespace Forge.Studio.Services;

/// <summary>
/// Service for managing Forge projects
/// </summary>
public class ProjectService
{
    private readonly ForgeService _forgeService;
    private ProjectInfo? _currentProject;

    public string CurrentProjectPath { get; private set; } = "";
    public string ProjectName => _currentProject?.Name ?? "";
    public List<ModuleInfo> Modules => _currentProject?.Modules ?? new();

    public ProjectService(ForgeService forgeService)
    {
        _forgeService = forgeService;
    }

    public void LoadProject(string path)
    {
        CurrentProjectPath = path;

        var forgeJsonPath = Path.Combine(path, "forge.json");
        if (!File.Exists(forgeJsonPath))
        {
            // Create a default project
            _currentProject = new ProjectInfo
            {
                Name = Path.GetFileName(path),
                Path = path
            };
            return;
        }

        try
        {
            var json = File.ReadAllText(forgeJsonPath);
            var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;

            _currentProject = new ProjectInfo
            {
                Name = root.TryGetProperty("name", out var name) ? name.GetString() ?? "" : "",
                Path = path,
                Version = root.TryGetProperty("version", out var ver) ? ver.GetString() ?? "1.0.0" : "1.0.0"
            };

            // Load modules
            if (root.TryGetProperty("modules", out var modules))
            {
                foreach (var module in modules.EnumerateArray())
                {
                    var moduleInfo = new ModuleInfo
                    {
                        Name = module.TryGetProperty("name", out var mn) ? mn.GetString() ?? "" : "",
                        Language = module.TryGetProperty("language", out var ml) ? ml.GetString() ?? "" : "",
                        Path = module.TryGetProperty("path", out var mp) ? mp.GetString() ?? "" : ""
                    };

                    if (module.TryGetProperty("depends", out var deps))
                    {
                        foreach (var dep in deps.EnumerateArray())
                        {
                            moduleInfo.Dependencies.Add(dep.GetString() ?? "");
                        }
                    }

                    _currentProject.Modules.Add(moduleInfo);
                }
            }

            // Also scan for module.json files in subdirectories
            ScanForModules(path);
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error loading project: {ex.Message}");
            _currentProject = new ProjectInfo
            {
                Name = Path.GetFileName(path),
                Path = path
            };
        }
    }

    private void ScanForModules(string path)
    {
        if (_currentProject == null) return;

        foreach (var dir in Directory.GetDirectories(path))
        {
            var moduleJson = Path.Combine(dir, "module.json");
            if (File.Exists(moduleJson))
            {
                try
                {
                    var json = File.ReadAllText(moduleJson);
                    var doc = JsonDocument.Parse(json);
                    var root = doc.RootElement;

                    var moduleName = root.TryGetProperty("name", out var n) ? n.GetString() : Path.GetFileName(dir);
                    
                    // Check if already added
                    if (_currentProject.Modules.Exists(m => m.Name == moduleName))
                        continue;

                    var moduleInfo = new ModuleInfo
                    {
                        Name = moduleName ?? "",
                        Language = root.TryGetProperty("language", out var l) ? l.GetString() ?? "" : DetectLanguage(dir),
                        Path = dir
                    };

                    if (root.TryGetProperty("depends", out var deps))
                    {
                        foreach (var dep in deps.EnumerateArray())
                        {
                            moduleInfo.Dependencies.Add(dep.GetString() ?? "");
                        }
                    }

                    _currentProject.Modules.Add(moduleInfo);
                }
                catch
                {
                    // Ignore malformed module.json
                }
            }
        }
    }

    private static string DetectLanguage(string dir)
    {
        // Simple language detection based on file extensions
        var extensions = new Dictionary<string, string>
        {
            [".cs"] = "C#",
            [".rs"] = "Rust",
            [".c"] = "C",
            [".cpp"] = "C++",
            [".go"] = "Go",
            [".py"] = "Python",
            [".ts"] = "TypeScript",
            [".js"] = "JavaScript",
            [".zig"] = "Zig",
            [".nim"] = "Nim",
            [".jl"] = "Julia",
            [".hs"] = "Haskell",
            [".ml"] = "OCaml",
            [".rb"] = "Ruby",
            [".java"] = "Java",
            [".kt"] = "Kotlin",
            [".scala"] = "Scala",
            [".swift"] = "Swift",
            [".d"] = "D",
            [".lua"] = "Lua",
            [".php"] = "PHP",
            [".erl"] = "Erlang",
            [".ex"] = "Elixir"
        };

        foreach (var (ext, lang) in extensions)
        {
            if (Directory.GetFiles(dir, $"*{ext}", SearchOption.AllDirectories).Length > 0)
                return lang;
        }

        return "Unknown";
    }

    public void SaveProject()
    {
        if (_currentProject == null || string.IsNullOrEmpty(CurrentProjectPath))
            return;

        var forgeJsonPath = Path.Combine(CurrentProjectPath, "forge.json");
        
        var projectData = new
        {
            name = _currentProject.Name,
            version = _currentProject.Version,
            modules = _currentProject.Modules.ConvertAll(m => new
            {
                name = m.Name,
                language = m.Language,
                path = m.Path,
                depends = m.Dependencies
            })
        };

        var json = JsonSerializer.Serialize(projectData, new JsonSerializerOptions 
        { 
            WriteIndented = true 
        });
        
        File.WriteAllText(forgeJsonPath, json);
    }

    public void CreateNewProject(string path, string name)
    {
        Directory.CreateDirectory(path);

        _currentProject = new ProjectInfo
        {
            Name = name,
            Path = path,
            Version = "1.0.0"
        };

        CurrentProjectPath = path;
        SaveProject();
    }

    public void AddModule(string name, string language, string relativePath)
    {
        if (_currentProject == null) return;

        var modulePath = Path.Combine(CurrentProjectPath, relativePath);
        Directory.CreateDirectory(modulePath);

        var module = new ModuleInfo
        {
            Name = name,
            Language = language,
            Path = relativePath
        };

        _currentProject.Modules.Add(module);

        // Create module.json
        var moduleJson = new
        {
            name = name,
            language = language,
            depends = Array.Empty<string>()
        };

        var json = JsonSerializer.Serialize(moduleJson, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(Path.Combine(modulePath, "module.json"), json);
    }

    public List<DependencyLink> GetDependencies()
    {
        var links = new List<DependencyLink>();

        if (_currentProject == null) return links;

        foreach (var module in _currentProject.Modules)
        {
            foreach (var dep in module.Dependencies)
            {
                links.Add(new DependencyLink
                {
                    From = module.Name,
                    To = dep
                });
            }
        }

        return links;
    }
}
