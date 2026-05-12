//! Component build graph types for ChimeraIR.
//!
//! This module provides the types for representing the build graph:
//! - `ComponentGraph` - the full build graph with nodes and edges
//! - `ComponentNode` - a node representing a component build
//! - `BuildEdge`, `RuntimeEdge`, `ProofEdge`, `WrapperEdge` - edge types
//! - Cycle detection utilities

use crate::{AbiEdge, ComponentId, ComponentKind, Language};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use thiserror::Error;

/// Errors that can occur in graph operations.
#[derive(Debug, Error)]
pub enum GraphError {
    #[error("cycle detected involving component: {0}")]
    CycleDetected(ComponentId),
    #[error("duplicate component ID: {0}")]
    DuplicateComponent(ComponentId),
    #[error("missing component: {0}")]
    MissingComponent(ComponentId),
    #[error("duplicate edge: {0}")]
    DuplicateEdge(String),
    #[error("invalid edge: {0}")]
    InvalidEdge(String),
}

/// Edge kind in the build graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeKind {
    /// Component must be built before dependent
    Build,
    /// Metadata must be extracted before use
    Metadata,
    /// Wrapper must be generated before link
    Wrapper,
    /// Proof must pass before link
    Proof,
    /// Runtime files must be packaged before execution
    Runtime,
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EdgeKind::Build => write!(f, "build"),
            EdgeKind::Metadata => write!(f, "metadata"),
            EdgeKind::Wrapper => write!(f, "wrapper"),
            EdgeKind::Proof => write!(f, "proof"),
            EdgeKind::Runtime => write!(f, "runtime"),
        }
    }
}

/// An edge in the build graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub from: ComponentId,
    /// Target node ID
    pub to: ComponentId,
    /// Edge kind
    pub kind: EdgeKind,
    /// Optional ABI edge reference
    #[serde(default)]
    pub abi_edge: Option<AbiEdge>,
}

impl GraphEdge {
    /// Create a new build edge.
    pub fn build(from: ComponentId, to: ComponentId) -> Self {
        GraphEdge {
            from,
            to,
            kind: EdgeKind::Build,
            abi_edge: None,
        }
    }

    /// Create a new runtime edge.
    pub fn runtime(from: ComponentId, to: ComponentId) -> Self {
        GraphEdge {
            from,
            to,
            kind: EdgeKind::Runtime,
            abi_edge: None,
        }
    }

    /// Create a new metadata edge.
    pub fn metadata(from: ComponentId, to: ComponentId) -> Self {
        GraphEdge {
            from,
            to,
            kind: EdgeKind::Metadata,
            abi_edge: None,
        }
    }

    /// Create a new wrapper edge.
    pub fn wrapper(from: ComponentId, to: ComponentId) -> Self {
        GraphEdge {
            from,
            to,
            kind: EdgeKind::Wrapper,
            abi_edge: None,
        }
    }

    /// Create a new proof edge.
    pub fn proof(from: ComponentId, to: ComponentId) -> Self {
        GraphEdge {
            from,
            to,
            kind: EdgeKind::Proof,
            abi_edge: None,
        }
    }

    /// Create a new edge from an ABI edge.
    pub fn from_abi_edge(abi_edge: AbiEdge, kind: EdgeKind) -> Self {
        GraphEdge {
            from: abi_edge.provider.clone(),
            to: abi_edge.consumer.clone(),
            kind,
            abi_edge: Some(abi_edge),
        }
    }
}

/// A node in the component build graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentNode {
    /// Component ID
    pub id: ComponentId,
    /// Component kind
    pub kind: ComponentKind,
    /// Component language
    pub language: Language,
    /// Whether this is a build target
    pub is_target: bool,
    /// Dependencies (by component ID)
    #[serde(default)]
    pub dependencies: Vec<ComponentId>,
}

impl ComponentNode {
    /// Create a new component node.
    pub fn new(id: ComponentId, kind: ComponentKind, language: Language) -> Self {
        ComponentNode {
            id,
            kind,
            language,
            is_target: false,
            dependencies: Vec::new(),
        }
    }

    /// Mark this node as a build target.
    pub fn set_target(&mut self) {
        self.is_target = true;
    }

    /// Add a dependency.
    pub fn add_dependency(&mut self, dep: ComponentId) {
        if !self.dependencies.contains(&dep) {
            self.dependencies.push(dep);
        }
    }
}

/// The component build graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentGraph {
    /// Nodes indexed by component ID
    nodes: HashMap<ComponentId, ComponentNode>,
    /// Edges
    #[serde(default)]
    edges: Vec<GraphEdge>,
    /// Edge index for fast lookup
    edge_index: HashMap<String, GraphEdge>,
}

impl ComponentGraph {
    /// Create a new empty component graph.
    pub fn new() -> Self {
        ComponentGraph {
            nodes: HashMap::new(),
            edges: Vec::new(),
            edge_index: HashMap::new(),
        }
    }

    /// Add a component node to the graph.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::DuplicateComponent` if a node with the same ID already exists.
    pub fn add_node(&mut self, node: ComponentNode) -> Result<(), GraphError> {
        let id = node.id.clone();
        if self.nodes.contains_key(&id) {
            return Err(GraphError::DuplicateComponent(id));
        }
        self.nodes.insert(id, node);
        Ok(())
    }

    /// Add an edge to the graph.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::InvalidEdge` if either endpoint doesn't exist.
    pub fn add_edge(&mut self, edge: GraphEdge) -> Result<(), GraphError> {
        // Clone values we need after moving edge
        let from_id = edge.from.clone();
        let to_id = edge.to.clone();
        let edge_key = format!("{}:{}:{:?}", from_id, to_id, edge.kind);

        // Verify endpoints exist
        if !self.nodes.contains_key(&from_id) {
            return Err(GraphError::MissingComponent(from_id));
        }
        if !self.nodes.contains_key(&to_id) {
            return Err(GraphError::MissingComponent(to_id));
        }

        // Check for duplicate edge
        if self.edge_index.contains_key(&edge_key) {
            return Err(GraphError::DuplicateEdge(edge_key));
        }

        self.edge_index.insert(edge_key, edge.clone());
        self.edges.push(edge);

        // Update node dependencies
        if let Some(node) = self.nodes.get_mut(&from_id) {
            node.add_dependency(to_id);
        }

        Ok(())
    }

    /// Add an ABI edge as a build edge.
    pub fn add_abi_edge(&mut self, abi_edge: AbiEdge) -> Result<(), GraphError> {
        let edge = GraphEdge::from_abi_edge(abi_edge, EdgeKind::Build);
        self.add_edge(edge)
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: &ComponentId) -> Option<&ComponentNode> {
        self.nodes.get(id)
    }

    /// Get a mutable node by ID.
    pub fn get_node_mut(&mut self, id: &ComponentId) -> Option<&mut ComponentNode> {
        self.nodes.get_mut(id)
    }

    /// Get all nodes.
    pub fn nodes(&self) -> &HashMap<ComponentId, ComponentNode> {
        &self.nodes
    }

    /// Get all edges.
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }

    /// Get edges of a specific kind.
    pub fn edges_of_kind(&self, kind: EdgeKind) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.kind == kind).collect()
    }

    /// Get edges from a specific node.
    pub fn edges_from(&self, from: &ComponentId) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| &e.from == from).collect()
    }

    /// Get edges to a specific node.
    pub fn edges_to(&self, to: &ComponentId) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| &e.to == to).collect()
    }

    /// Check if the graph contains a cycle.
    ///
    /// Uses DFS-based cycle detection.
    pub fn has_cycle(&self) -> bool {
        let mut visited: HashSet<ComponentId> = HashSet::new();
        let mut rec_stack: HashSet<ComponentId> = HashSet::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                if self.detect_cycle_dfs(node_id, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }
        false
    }

    fn detect_cycle_dfs(
        &self,
        node_id: &ComponentId,
        visited: &mut HashSet<ComponentId>,
        rec_stack: &mut HashSet<ComponentId>,
    ) -> bool {
        visited.insert(node_id.clone());
        rec_stack.insert(node_id.clone());

        // Visit all dependencies
        if let Some(node) = self.nodes.get(node_id) {
            for dep_id in &node.dependencies {
                if !visited.contains(dep_id) {
                    if self.detect_cycle_dfs(dep_id, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep_id) {
                    return true;
                }
            }
        }

        rec_stack.remove(node_id);
        false
    }

    /// Get the topological order of nodes.
    ///
    /// Returns `Err` with the cycle if the graph contains a cycle.
    pub fn topological_order(&self) -> Result<Vec<ComponentId>, Vec<ComponentId>> {
        if self.has_cycle() {
            // Find nodes in cycle
            let cycle = self.find_cycle();
            return Err(cycle);
        }

        let mut result = Vec::new();
        let mut visited: HashSet<ComponentId> = HashSet::new();
        let mut temp_mark: HashSet<ComponentId> = HashSet::new();

        fn visit(
            graph: &ComponentGraph,
            node_id: &ComponentId,
            visited: &mut HashSet<ComponentId>,
            temp_mark: &mut HashSet<ComponentId>,
            result: &mut Vec<ComponentId>,
        ) {
            if temp_mark.contains(node_id) {
                return; // Cycle detected (shouldn't happen here)
            }
            if visited.contains(node_id) {
                return;
            }

            temp_mark.insert(node_id.clone());
            if let Some(node) = graph.nodes.get(node_id) {
                for dep_id in &node.dependencies {
                    visit(graph, dep_id, visited, temp_mark, result);
                }
            }
            temp_mark.remove(node_id);
            visited.insert(node_id.clone());
            result.push(node_id.clone());
        }

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                visit(self, node_id, &mut visited, &mut temp_mark, &mut result);
            }
        }

        result.reverse();
        Ok(result)
    }

    /// Find a cycle in the graph (if any).
    fn find_cycle(&self) -> Vec<ComponentId> {
        let mut visited: HashSet<ComponentId> = HashSet::new();
        let mut rec_stack: HashSet<ComponentId> = HashSet::new();
        let mut cycle_path: Vec<ComponentId> = Vec::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                if self.find_cycle_dfs(node_id, &mut visited, &mut rec_stack, &mut cycle_path) {
                    return cycle_path;
                }
            }
        }
        Vec::new()
    }

    fn find_cycle_dfs(
        &self,
        node_id: &ComponentId,
        visited: &mut HashSet<ComponentId>,
        rec_stack: &mut HashSet<ComponentId>,
        cycle_path: &mut Vec<ComponentId>,
    ) -> bool {
        visited.insert(node_id.clone());
        rec_stack.insert(node_id.clone());
        cycle_path.push(node_id.clone());

        if let Some(node) = self.nodes.get(node_id) {
            for dep_id in &node.dependencies {
                if !visited.contains(dep_id) {
                    if self.find_cycle_dfs(dep_id, visited, rec_stack, cycle_path) {
                        return true;
                    }
                } else if rec_stack.contains(dep_id) {
                    cycle_path.push(dep_id.clone());
                    return true;
                }
            }
        }

        cycle_path.pop();
        rec_stack.remove(node_id);
        false
    }

    /// Get build targets (nodes marked as is_target).
    pub fn build_targets(&self) -> Vec<&ComponentNode> {
        self.nodes.values().filter(|n| n.is_target).collect()
    }

    /// Get nodes of a specific language.
    pub fn nodes_by_language(&self, language: Language) -> Vec<&ComponentNode> {
        self.nodes
            .values()
            .filter(|n| n.language == language)
            .collect()
    }

    /// Get the number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Validate the graph.
    ///
    /// Checks for:
    /// - Cycles
    /// - Missing dependencies
    /// - Invalid edges
    pub fn validate(&self) -> Result<(), Vec<GraphError>> {
        let mut errors = Vec::new();

        // Check for cycles
        if self.has_cycle() {
            let cycle = self.find_cycle();
            if let Some(first) = cycle.first() {
                errors.push(GraphError::CycleDetected(first.clone()));
            }
        }

        // Check all dependencies exist
        for (_node_id, node) in &self.nodes {
            for dep_id in &node.dependencies {
                if !self.nodes.contains_key(dep_id) {
                    errors.push(GraphError::MissingComponent(dep_id.clone()));
                }
            }
        }

        // Check all edges reference existing nodes
        for edge in &self.edges {
            if !self.nodes.contains_key(&edge.from) {
                errors.push(GraphError::MissingComponent(edge.from.clone()));
            }
            if !self.nodes.contains_key(&edge.to) {
                errors.push(GraphError::MissingComponent(edge.to.clone()));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_rust_node(id: &str) -> ComponentNode {
        ComponentNode::new(
            ComponentId::new(id),
            ComponentKind::CargoPackage,
            Language::Rust,
        )
    }

    fn create_zig_node(id: &str) -> ComponentNode {
        ComponentNode::new(ComponentId::new(id), ComponentKind::ZigExe, Language::Zig)
    }

    #[test]
    fn test_empty_graph() {
        let graph = ComponentGraph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_nodes() {
        let mut graph = ComponentGraph::new();
        let node1 = create_rust_node("lib");
        let node2 = create_zig_node("cli");

        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();

        assert_eq!(graph.node_count(), 2);
        assert!(graph.get_node(&ComponentId::new("lib")).is_some());
        assert!(graph.get_node(&ComponentId::new("cli")).is_some());
    }

    #[test]
    fn test_duplicate_node_error() {
        let mut graph = ComponentGraph::new();
        let node1 = create_rust_node("lib");
        let node2 = create_rust_node("lib");

        graph.add_node(node1).unwrap();
        assert!(graph.add_node(node2).is_err());
    }

    #[test]
    fn test_add_edges() {
        let mut graph = ComponentGraph::new();
        graph.add_node(create_rust_node("lib")).unwrap();
        graph.add_node(create_zig_node("cli")).unwrap();

        let edge = GraphEdge::build(ComponentId::new("lib"), ComponentId::new("cli"));
        graph.add_edge(edge).unwrap();

        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_missing_node_edge_error() {
        let mut graph = ComponentGraph::new();
        graph.add_node(create_rust_node("lib")).unwrap();

        let edge = GraphEdge::build(ComponentId::new("lib"), ComponentId::new("missing"));
        assert!(graph.add_edge(edge).is_err());
    }

    #[test]
    fn test_no_cycle() {
        let mut graph = ComponentGraph::new();
        graph.add_node(create_rust_node("lib")).unwrap();
        graph.add_node(create_zig_node("cli")).unwrap();
        graph
            .add_node(ComponentNode::new(
                ComponentId::new("c"),
                ComponentKind::CSource,
                Language::C,
            ))
            .unwrap();

        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("lib"),
                ComponentId::new("cli"),
            ))
            .unwrap();
        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("c"),
                ComponentId::new("cli"),
            ))
            .unwrap();

        assert!(!graph.has_cycle());
        let order = graph.topological_order().unwrap();
        // lib and c should come before cli
        assert!(
            index_of(&order, &ComponentId::new("lib")) < index_of(&order, &ComponentId::new("cli"))
        );
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = ComponentGraph::new();
        graph.add_node(create_rust_node("a")).unwrap();
        graph.add_node(create_rust_node("b")).unwrap();
        graph.add_node(create_rust_node("c")).unwrap();

        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("a"),
                ComponentId::new("b"),
            ))
            .unwrap();
        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("b"),
                ComponentId::new("c"),
            ))
            .unwrap();
        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("c"),
                ComponentId::new("a"),
            ))
            .unwrap();

        assert!(graph.has_cycle());
        assert!(graph.topological_order().is_err());
    }

    #[test]
    fn test_build_targets() {
        let mut graph = ComponentGraph::new();
        let mut node1 = create_zig_node("cli");
        node1.set_target();

        graph.add_node(create_rust_node("lib")).unwrap();
        graph.add_node(node1).unwrap();

        let targets = graph.build_targets();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].id.as_str(), "cli");
    }

    #[test]
    fn test_nodes_by_language() {
        let mut graph = ComponentGraph::new();
        graph.add_node(create_rust_node("lib")).unwrap();
        graph.add_node(create_zig_node("cli")).unwrap();

        let rust_nodes = graph.nodes_by_language(Language::Rust);
        let zig_nodes = graph.nodes_by_language(Language::Zig);

        assert_eq!(rust_nodes.len(), 1);
        assert_eq!(zig_nodes.len(), 1);
    }

    #[test]
    fn test_edges_of_kind() {
        let mut graph = ComponentGraph::new();
        graph.add_node(create_rust_node("lib")).unwrap();
        graph.add_node(create_zig_node("cli")).unwrap();

        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("lib"),
                ComponentId::new("cli"),
            ))
            .unwrap();
        graph
            .add_edge(GraphEdge::runtime(
                ComponentId::new("lib"),
                ComponentId::new("cli"),
            ))
            .unwrap();

        let build_edges = graph.edges_of_kind(EdgeKind::Build);
        let runtime_edges = graph.edges_of_kind(EdgeKind::Runtime);

        assert_eq!(build_edges.len(), 1);
        assert_eq!(runtime_edges.len(), 1);
    }

    #[test]
    fn test_validate_success() {
        let mut graph = ComponentGraph::new();
        graph.add_node(create_rust_node("lib")).unwrap();
        graph.add_node(create_zig_node("cli")).unwrap();

        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("lib"),
                ComponentId::new("cli"),
            ))
            .unwrap();

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_validate_missing_dependency() {
        let mut graph = ComponentGraph::new();
        let mut node = create_rust_node("lib");
        node.add_dependency(ComponentId::new("missing"));
        graph.add_node(node).unwrap();

        assert!(graph.validate().is_err());
    }

    #[test]
    fn test_abi_edge_cycle() {
        let mut graph = ComponentGraph::new();
        graph.add_node(create_rust_node("provider_a")).unwrap();
        graph.add_node(create_rust_node("provider_b")).unwrap();
        graph.add_node(create_zig_node("consumer_a")).unwrap();
        graph.add_node(create_zig_node("consumer_b")).unwrap();

        // ABI edge cycle: consumer_a -> provider_a -> consumer_b -> provider_b -> consumer_a
        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("provider_a"),
                ComponentId::new("consumer_a"),
            ))
            .unwrap();
        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("consumer_a"),
                ComponentId::new("provider_b"),
            ))
            .unwrap();
        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("provider_b"),
                ComponentId::new("consumer_b"),
            ))
            .unwrap();
        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("consumer_b"),
                ComponentId::new("provider_a"),
            ))
            .unwrap();

        assert!(graph.has_cycle(), "ABI edge cycle must be detected");
        assert!(
            graph.topological_order().is_err(),
            "cyclic graph must not have topological order"
        );
    }

    #[test]
    fn test_deterministic_topological_order() {
        let mut graph = ComponentGraph::new();

        // Create a diamond dependency: a -> b, a -> c, b -> d, c -> d
        graph.add_node(create_rust_node("a")).unwrap();
        graph.add_node(create_rust_node("b")).unwrap();
        graph.add_node(create_rust_node("c")).unwrap();
        graph.add_node(create_zig_node("d")).unwrap();

        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("a"),
                ComponentId::new("b"),
            ))
            .unwrap();
        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("a"),
                ComponentId::new("c"),
            ))
            .unwrap();
        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("b"),
                ComponentId::new("d"),
            ))
            .unwrap();
        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("c"),
                ComponentId::new("d"),
            ))
            .unwrap();

        // Run topological sort multiple times
        let order1 = graph.topological_order().unwrap();
        let order2 = graph.topological_order().unwrap();
        let order3 = graph.topological_order().unwrap();

        // All orders should be identical
        assert_eq!(order1, order2, "topological order must be deterministic");
        assert_eq!(order2, order3, "topological order must be deterministic");

        // Check DAG constraints: a before b and c, b and c before d
        assert!(
            index_of(&order1, &ComponentId::new("a")) < index_of(&order1, &ComponentId::new("b"))
        );
        assert!(
            index_of(&order1, &ComponentId::new("a")) < index_of(&order1, &ComponentId::new("c"))
        );
        assert!(
            index_of(&order1, &ComponentId::new("b")) < index_of(&order1, &ComponentId::new("d"))
        );
        assert!(
            index_of(&order1, &ComponentId::new("c")) < index_of(&order1, &ComponentId::new("d"))
        );
    }

    #[test]
    fn test_abi_edge_with_abi_edge_struct() {
        let mut graph = ComponentGraph::new();
        graph.add_node(create_rust_node("rust_provider")).unwrap();
        graph.add_node(create_zig_node("zig_consumer")).unwrap();

        // Add an edge using AbiEdge
        let abi_edge = crate::AbiEdge::new(
            ComponentId::new("zig_consumer"),
            ComponentId::new("rust_provider"),
        );

        graph.add_abi_edge(abi_edge).unwrap();

        assert_eq!(graph.edge_count(), 1);
        let edge = &graph.edges()[0];
        assert_eq!(edge.from.as_str(), "rust_provider");
        assert_eq!(edge.to.as_str(), "zig_consumer");
        assert_eq!(edge.kind, EdgeKind::Build);
        assert!(edge.abi_edge.is_some());
    }

    #[test]
    fn test_duplicate_provider_edge() {
        let mut graph = ComponentGraph::new();
        graph.add_node(create_rust_node("provider")).unwrap();
        graph.add_node(create_zig_node("consumer1")).unwrap();
        graph.add_node(create_zig_node("consumer2")).unwrap();

        // Two consumers from same provider is fine
        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("provider"),
                ComponentId::new("consumer1"),
            ))
            .unwrap();
        graph
            .add_edge(GraphEdge::build(
                ComponentId::new("provider"),
                ComponentId::new("consumer2"),
            ))
            .unwrap();

        // But duplicate same edge should fail
        let result = graph.add_edge(GraphEdge::build(
            ComponentId::new("provider"),
            ComponentId::new("consumer1"),
        ));
        assert!(result.is_err(), "duplicate edge should be rejected");
    }

    // Helper to get index in Vec (for topological order test)
    fn index_of(v: &[ComponentId], id: &ComponentId) -> usize {
        v.iter().position(|i| i == id).unwrap_or(0)
    }
}
