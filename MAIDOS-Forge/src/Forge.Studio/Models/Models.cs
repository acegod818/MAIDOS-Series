// MAIDOS-Forge Studio - Models
// Code-QC v2.2B Compliant

using System.Collections.Generic;

namespace Forge.Studio.Models;

public class ModuleInfo
{
    public string Name { get; set; } = "";
    public string Language { get; set; } = "";
    public string Path { get; set; } = "";
    public List<string> Dependencies { get; set; } = new();
    public Dictionary<string, string> Options { get; set; } = new();
}

public class ProjectInfo
{
    public string Name { get; set; } = "";
    public string Path { get; set; } = "";
    public string Version { get; set; } = "1.0.0";
    public List<ModuleInfo> Modules { get; set; } = new();
}

public class BuildResult
{
    public bool Success { get; set; }
    public string Output { get; set; } = "";
    public List<string> Artifacts { get; set; } = new();
    public double ElapsedSeconds { get; set; }
}
