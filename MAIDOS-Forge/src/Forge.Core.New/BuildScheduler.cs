// MAIDOS-Forge Build Scheduler
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Core.Config;

namespace Forge.Core.Build;

/// <summary>
/// Build layer - Modules within the same layer can be compiled in parallel
/// </summary>
/// <impl>
/// APPROACH: Encapsulates a set of modules that can be compiled in parallel
/// CALLS: N/A (pure data)
/// EDGES: Modules can be empty but not null
/// </impl>
public sealed class BuildLayer
{
    public int Level { get; }
    public IReadOnlyList<ValidatedModuleConfig> Modules { get; }

    public BuildLayer(int level, IReadOnlyList<ValidatedModuleConfig> modules)
    {
        Level = level;
        Modules = modules;
    }
}

/// <summary>
/// Build schedule - Contains modules sorted by layer
/// </summary>
/// <impl>
/// APPROACH: Encapsulates topological sort results, organizing modules by layer
/// CALLS: N/A (pure data)
/// EDGES: Layers are ordered by dependency, Layer 0 has no dependencies
/// </impl>
public sealed class BuildSchedule
{
    public IReadOnlyList<BuildLayer> Layers { get; }
    public int TotalModules { get; }
    public int MaxParallelism { get; }

    public BuildSchedule(IReadOnlyList<BuildLayer> layers)
    {
        Layers = layers;
        TotalModules = layers.Sum(l => l.Modules.Count);
        MaxParallelism = layers.Count > 0 ? layers.Max(l => l.Modules.Count) : 0;
    }

    /// <summary>
    /// Get flattened compilation order (for single-threaded use)
    /// </summary>
    /// <impl>
    /// APPROACH: Flatten all modules by layer
    /// CALLS: SelectMany()
    /// EDGES: Empty schedule returns empty list
    /// </impl>
    public IEnumerable<ValidatedModuleConfig> GetFlattenedOrder()
    {
        return Layers.SelectMany(l => l.Modules);
    }
}

/// <summary>
/// Schedule result
/// </summary>
/// <impl>
/// APPROACH: Result pattern encapsulating success/failure
/// CALLS: N/A (pure data)
/// EDGES: IsSuccess and Error are mutually exclusive
/// </impl>
public readonly struct ScheduleResult
{
    public bool IsSuccess { get; }
    public BuildSchedule? Schedule { get; }
    public string Error { get; }

    private ScheduleResult(bool isSuccess, BuildSchedule? schedule, string error)
    {
        IsSuccess = isSuccess;
        Schedule = schedule;
        Error = error;
    }

    public static ScheduleResult Success(BuildSchedule schedule)
        => new(true, schedule, string.Empty);

    public static ScheduleResult Failure(string error)
        => new(false, null, error);
}

/// <summary>
/// Build scheduler - Uses Kahn's algorithm for topological sorting and build layer assignment
/// </summary>
/// <impl>
/// APPROACH: Kahn's algorithm - Starting from nodes with in-degree 0, remove layer by layer and update in-degrees
/// CALLS: DependencyGraph
/// EDGES: Empty graph returns empty schedule, circular dependency returns failure
/// </impl>
public static class BuildScheduler
{
    /// <summary>
    /// Generate build schedule from dependency graph
    /// </summary>
    /// <impl>
    /// APPROACH: Kahn's algorithm topological sort with layer tracking
    /// CALLS: DependencyGraph.Nodes
    /// EDGES: Empty graph returns empty schedule, unable to complete sort (cycle) returns failure
    /// </impl>
    public static ScheduleResult CreateSchedule(DependencyGraph graph)
    {
        if (graph.NodeCount == 0)
        {
            return ScheduleResult.Success(new BuildSchedule(Array.Empty<BuildLayer>()));
        }

        // Copy in-degree information (avoid modifying the original graph)
        var inDegree = new Dictionary<string, int>(StringComparer.OrdinalIgnoreCase);
        foreach (var (name, node) in graph.Nodes)
        {
            inDegree[name] = node.Dependencies.Count;
        }

        var layers = new List<BuildLayer>();
        var processed = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        var level = 0;

        while (processed.Count < graph.NodeCount)
        {
            // Find all nodes with in-degree 0 at the current level
            var currentLevel = new List<ValidatedModuleConfig>();

            foreach (var (name, degree) in inDegree)
            {
                if (degree == 0 && !processed.Contains(name))
                {
                    var node = graph.GetNode(name);
                    if (node != null)
                    {
                        currentLevel.Add(node.Module);
                    }
                }
            }

            // If no nodes with in-degree 0 but there are still unprocessed ones, there is a cycle
            if (currentLevel.Count == 0 && processed.Count < graph.NodeCount)
            {
                var remaining = graph.Nodes.Keys
                    .Where(k => !processed.Contains(k))
                    .ToList();
                return ScheduleResult.Failure(
                    $"Cannot schedule remaining modules (circular dependency?): {string.Join(", ", remaining)}");
            }

            if (currentLevel.Count > 0)
            {
                // Mark current level nodes as processed
                foreach (var module in currentLevel)
                {
                    processed.Add(module.Config.Name);

                    // Update in-degree for modules that depend on this module
                    foreach (var (name, node) in graph.Nodes)
                    {
                        if (node.Dependencies.Contains(module.Config.Name, StringComparer.OrdinalIgnoreCase))
                        {
                            inDegree[name]--;
                        }
                    }
                }

                layers.Add(new BuildLayer(level, currentLevel));
                level++;
            }
        }

        return ScheduleResult.Success(new BuildSchedule(layers));
    }

    /// <summary>
    /// Generate schedule directly from dependency analysis result
    /// </summary>
    /// <impl>
    /// APPROACH: Delegates to CreateSchedule(DependencyGraph)
    /// CALLS: CreateSchedule()
    /// EDGES: Returns corresponding failure when dependency analysis has errors
    /// </impl>
    public static ScheduleResult CreateSchedule(DependencyAnalysisResult analysisResult)
    {
        if (analysisResult.HasCycle)
        {
            return ScheduleResult.Failure(analysisResult.Error);
        }

        if (!string.IsNullOrEmpty(analysisResult.Error))
        {
            return ScheduleResult.Failure(analysisResult.Error);
        }

        return CreateSchedule(analysisResult.Graph);
    }
}
