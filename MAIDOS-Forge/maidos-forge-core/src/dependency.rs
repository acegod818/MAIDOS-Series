//! Dependency analysis module

/// Dependency graph
#[derive(Debug)]
pub struct DependencyGraph {
    /// List of nodes
    pub nodes: Vec<DependencyNode>,
}

/// Dependency node
#[derive(Debug)]
pub struct DependencyNode {
    /// Node name
    pub name: String,
    /// List of dependencies
    pub dependencies: Vec<String>,
}

impl DependencyGraph {
    /// Create a new dependency graph.
    pub fn new() -> Self {
        Self {
            nodes: vec![],
        }
    }

    /// Add a node.
    pub fn add_node(&mut self, name: String, dependencies: Vec<String>) {
        self.nodes.push(DependencyNode {
            name,
            dependencies,
        });
    }

    /// Get a node by name.
    pub fn get_node(&self, name: &str) -> Option<&DependencyNode> {
        self.nodes.iter().find(|node| node.name == name)
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}