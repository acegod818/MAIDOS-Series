// MAIDOS-Forge Dependency Analyzer
// UEP v1.7B Compliant - Zero Technical Debt

using Forge.Core.Config;

namespace Forge.Core.Build;

/// <summary>
/// Dependency analysis result
/// </summary>
/// <impl>
/// APPROACH: Encapsulates DAG construction and cycle detection results
/// CALLS: N/A (pure data)
/// EDGES: CycleChain is non-empty when HasCycle is true, Graph is always available
/// </impl>
public sealed class DependencyAnalysisResult
{
    public bool HasCycle { get; }
    public IReadOnlyList<string> CycleChain { get; }
    public DependencyGraph Graph { get; }
    public string Error { get; }

    private DependencyAnalysisResult(bool hasCycle, IReadOnlyList<string> cycleChain, 
        DependencyGraph graph, string error)
    {
        HasCycle = hasCycle;
        CycleChain = cycleChain;
        Graph = graph;
        Error = error;
    }

    public static DependencyAnalysisResult Success(DependencyGraph graph) 
        => new(false, Array.Empty<string>(), graph, string.Empty);

    public static DependencyAnalysisResult CycleDetected(IReadOnlyList<string> cycle, DependencyGraph graph) 
        => new(true, cycle, graph, $"Circular dependency detected: {string.Join(" → ", cycle)}");

    public static DependencyAnalysisResult Failure(string error) 
        => new(false, Array.Empty<string>(), new DependencyGraph(new Dictionary<string, DependencyNode>()), error);
}

/// <summary>
/// Dependency graph node
/// </summary>
/// <impl>
/// APPROACH: Represents a single module node in the DAG
/// CALLS: N/A (pure data)
/// EDGES: Dependencies can be an empty list, InDegree is used for topological sorting
/// </impl>
public sealed class DependencyNode
{
    public string Name { get; }
    public ValidatedModuleConfig Module { get; }
    public List<string> Dependencies { get; }
    public int InDegree { get; set; }

    public DependencyNode(string name, ValidatedModuleConfig module)
    {
        Name = name;
        Module = module;
        Dependencies = new List<string>(module.Config.Dependencies);
        InDegree = 0;
    }
}

/// <summary>
/// Dependency graph (DAG)
/// </summary>
/// <impl>
/// APPROACH: Adjacency list representation for storing module dependencies
/// CALLS: N/A (pure data)
/// EDGES: Nodes dictionary keyed by module name
/// </impl>
public sealed class DependencyGraph
{
    private readonly Dictionary<string, DependencyNode> _nodes;

    public IReadOnlyDictionary<string, DependencyNode> Nodes => _nodes;
    public int NodeCount => _nodes.Count;

    public DependencyGraph(Dictionary<string, DependencyNode> nodes)
    {
        _nodes = nodes;
    }

    /// <summary>
    /// Get the node for a specified module
    /// </summary>
    /// <impl>
    /// APPROACH: Dictionary lookup
    /// CALLS: Dictionary.TryGetValue()
    /// EDGES: Returns null if module does not exist
    /// </impl>
    public DependencyNode? GetNode(string moduleName)
    {
        return _nodes.TryGetValue(moduleName, out var node) ? node : null;
    }
}

/// <summary>
/// Dependency analyzer - Builds DAG and detects circular dependencies
/// </summary>
/// <impl>
/// APPROACH: Builds adjacency list DAG from module configs, uses DFS to detect cycles
/// CALLS: ValidatedForgeConfig, ValidatedModuleConfig
/// EDGES: Empty module list returns empty graph, missing dependency reports error, circular dependency reports error
/// </impl>
public static class DependencyAnalyzer
{
    /// <summary>
    /// Analyze project dependencies
    /// </summary>
    /// <impl>
    /// APPROACH: Iterates all modules to build nodes and detect circular dependencies
    /// CALLS: BuildGraph(), DetectCycle()
    /// EDGES: Empty modules returns empty graph, non-existent dependency returns failure
    /// </impl>
    public static DependencyAnalysisResult Analyze(ValidatedForgeConfig config)
    {
        if (config.Modules.Count == 0)
        {
            return DependencyAnalysisResult.Success(
                new DependencyGraph(new Dictionary<string, DependencyNode>()));
        }

        // Build node dictionary
        var nodes = new Dictionary<string, DependencyNode>(StringComparer.OrdinalIgnoreCase);
        
        foreach (var module in config.Modules)
        {
            var node = new DependencyNode(module.Config.Name, module);
            nodes[module.Config.Name] = node;
        }

        // Verify all dependencies exist
        foreach (var node in nodes.Values)
        {
            foreach (var dep in node.Dependencies)
            {
                if (!nodes.ContainsKey(dep))
                {
                    return DependencyAnalysisResult.Failure(
                        $"Module '{node.Name}' depends on '{dep}' which does not exist");
                }
            }
        }

        // Calculate in-degree (how many modules depend on each)
        foreach (var node in nodes.Values)
        {
            foreach (var dep in node.Dependencies)
            {
                nodes[dep].InDegree++;
            }
        }

        var graph = new DependencyGraph(nodes);

        // Detect circular dependencies
        var cycleResult = DetectCycle(nodes);
        if (cycleResult.Count > 0)
        {
            return DependencyAnalysisResult.CycleDetected(cycleResult, graph);
        }

        return DependencyAnalysisResult.Success(graph);
    }

    /// <summary>
    /// Detect circular dependencies using DFS
    /// </summary>
    /// <impl>
    /// APPROACH: Three-color marking DFS - white (unvisited), gray (visiting), black (finished)
    /// CALLS: DfsVisit() recursively
    /// EDGES: Encountering a gray node indicates a cycle, returns cycle path
    /// </impl>
    private static List<string> DetectCycle(Dictionary<string, DependencyNode> nodes)
    {
        // 0 = white (unvisited), 1 = gray (visiting), 2 = black (finished)
        var colors = new Dictionary<string, int>(StringComparer.OrdinalIgnoreCase);
        var parent = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);

        foreach (var nodeName in nodes.Keys)
        {
            colors[nodeName] = 0;
        }

        foreach (var nodeName in nodes.Keys)
        {
            if (colors[nodeName] == 0)
            {
                var cyclePath = DfsVisit(nodeName, nodes, colors, parent);
                if (cyclePath.Count > 0)
                {
                    return cyclePath;
                }
            }
        }

        return new List<string>();
    }

    /// <summary>
    /// DFS visit a single node
    /// </summary>
    /// <impl>
    /// APPROACH: Mark as gray, recursively visit dependencies, mark black when done
    /// CALLS: Self-recursive
    /// EDGES: Rebuilds cycle path when a gray node is encountered
    /// </impl>
    private static List<string> DfsVisit(string nodeName, 
        Dictionary<string, DependencyNode> nodes,
        Dictionary<string, int> colors,
        Dictionary<string, string> parent)
    {
        colors[nodeName] = 1; // Mark as gray (visiting)

        var node = nodes[nodeName];
        foreach (var dep in node.Dependencies)
        {
            if (colors[dep] == 1)
            {
                // Cycle detected - rebuild path
                var cycle = new List<string> { dep };
                var current = nodeName;
                while (current != dep)
                {
                    cycle.Add(current);
                    if (!parent.TryGetValue(current, out current!))
                    {
                        break;
                    }
                }
                cycle.Add(dep);
                cycle.Reverse();
                return cycle;
            }

            if (colors[dep] == 0)
            {
                parent[dep] = nodeName;
                var cyclePath = DfsVisit(dep, nodes, colors, parent);
                if (cyclePath.Count > 0)
                {
                    return cyclePath;
                }
            }
        }

        colors[nodeName] = 2; // Mark as black (finished)
        return new List<string>();
    }
}
