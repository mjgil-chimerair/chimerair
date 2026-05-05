//! Dependency Graph for Zig Source Analysis
//!
//! **AUTHORITY STATUS**: This module is NON-AUTHORITATIVE.
//!
//! For production builds, graph population and diffing is owned by `zigmera-lowering`.
//! This module exists for fixtures, fallback scenarios, and test validation.
//!
//! This module provides a dependency graph that tracks relationships between:
//! - Files and their declarations
//! - Declarations and their dependencies
//! - Functions, types, layouts, comptime values
//! - Embed files and their consumers
//! - Exports and link artifacts
//!
//! The graph enables efficient invalidation analysis when source files change.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Unique identifier for graph nodes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(kind: NodeKind, name: &str) -> Self {
        Self(format!("{:?}:{}", kind, name))
    }

    pub fn file(path: &str) -> Self {
        Self::new(NodeKind::File, path)
    }

    pub fn func(name: &str) -> Self {
        Self::new(NodeKind::Function, name)
    }

    pub fn typ(name: &str) -> Self {
        Self::new(NodeKind::Type, name)
    }

    pub fn struct_(name: &str) -> Self {
        Self::new(NodeKind::Struct, name)
    }

    pub fn layout(name: &str) -> Self {
        Self::new(NodeKind::Layout, name)
    }

    pub fn comptime(name: &str) -> Self {
        Self::new(NodeKind::Comptime, name)
    }

    pub fn embed(path: &str) -> Self {
        Self::new(NodeKind::Embed, path)
    }

    pub fn export(name: &str) -> Self {
        Self::new(NodeKind::Export, name)
    }

    pub fn link(name: &str) -> Self {
        Self::new(NodeKind::LinkArtifact, name)
    }

    pub fn parse(s: &str) -> Option<Self> {
        if s.contains(':') {
            Some(Self(s.to_string()))
        } else {
            None
        }
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Kind of graph node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    /// Source file
    File,
    /// Function declaration
    Function,
    /// Type declaration
    Type,
    /// Struct declaration
    Struct,
    /// Layout information
    Layout,
    /// Comptime value or function
    Comptime,
    /// Embedded file
    Embed,
    /// Export declaration
    Export,
    /// Link artifact
    LinkArtifact,
}

/// Classification of a node change
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeClassification {
    /// Private body edit - doesn't affect public ABI
    BodyEdit,
    /// Public signature edit - affects downstream
    SignatureEdit,
    /// Import/source file edit
    ImportEdit,
    /// Type or layout change
    TypeOrLayoutChange,
    /// Other change
    Other,
    /// Unknown node
    Unknown,
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeKind::File => write!(f, "file"),
            NodeKind::Function => write!(f, "fn"),
            NodeKind::Type => write!(f, "type"),
            NodeKind::Struct => write!(f, "struct"),
            NodeKind::Layout => write!(f, "layout"),
            NodeKind::Comptime => write!(f, "comptime"),
            NodeKind::Embed => write!(f, "embed"),
            NodeKind::Export => write!(f, "export"),
            NodeKind::LinkArtifact => write!(f, "link"),
        }
    }
}

/// Edge kind in the dependency graph
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    /// Node A defines node B
    Defines,
    /// Node A uses node B
    Uses,
    /// Node A references node B (weaker than Uses)
    References,
    /// Node A is contained in node B
    Contains,
    /// Node A exports node B
    Exports,
    /// Node A links to node B
    LinksTo,
    /// Node A invalidates node B (change in A requires rebuild of B)
    Invalidates,
}

/// Edge in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
    /// Optional metadata about the dependency
    pub metadata: Option<String>,
}

impl Edge {
    pub fn new(from: NodeId, to: NodeId, kind: EdgeKind) -> Self {
        Self {
            from,
            to,
            kind,
            metadata: None,
        }
    }

    pub fn with_metadata(from: NodeId, to: NodeId, kind: EdgeKind, metadata: &str) -> Self {
        Self {
            from,
            to,
            kind,
            metadata: Some(metadata.to_string()),
        }
    }
}

/// Graph node with metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub name: String,
    pub source_location: Option<SourceLocation>,
    pub metadata: HashMap<String, String>,
}

impl Node {
    pub fn new(id: NodeId, kind: NodeKind, name: &str) -> Self {
        Self {
            id,
            kind,
            name: name.to_string(),
            source_location: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_location(mut self, loc: SourceLocation) -> Self {
        self.source_location = Some(loc);
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Source location for a node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl SourceLocation {
    pub fn new(file: &str, line: u32, column: u32) -> Self {
        Self {
            file: file.to_string(),
            line,
            column,
        }
    }
}

/// The dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// All nodes in the graph
    pub nodes: HashMap<NodeId, Node>,
    /// All edges in the graph
    edges: Vec<Edge>,
    /// Index from node to its outgoing edges
    pub outgoing: HashMap<NodeId, Vec<usize>>,
    /// Index from node to its incoming edges
    incoming: HashMap<NodeId, Vec<usize>>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: Node) -> &Node {
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        self.outgoing.entry(id.clone()).or_default();
        self.incoming.entry(id.clone()).or_default();
        self.nodes.get(&id).expect("node should exist")
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: Edge) {
        let from = edge.from.clone();
        let to = edge.to.clone();

        // Ensure both nodes exist
        if !self.nodes.contains_key(&from) {
            self.add_node(Node::new(from.clone(), NodeKind::File, ""));
        }
        if !self.nodes.contains_key(&to) {
            self.add_node(Node::new(to.clone(), NodeKind::Type, ""));
        }

        // Add the edge
        let idx = self.edges.len();
        self.edges.push(edge);

        // Update indexes
        self.outgoing.get_mut(&from).unwrap().push(idx);
        self.incoming.get_mut(&to).unwrap().push(idx);
    }

    /// Find or add a node - returns (existing_or_new_node_id, was_already_present)
    ///
    /// This provides stable ID semantics: repeated declarations, generic
    /// instantiations, and type/layout nodes get the same stable ID.
    pub fn find_or_add_node(&mut self, node: Node) -> (NodeId, bool) {
        let id = node.id.clone();
        if self.nodes.contains_key(&id) {
            return (id, true); // Already existed
        }
        self.add_node(node);
        (id, false) // Just created
    }

    /// Check if a node with the given ID already exists
    pub fn has_node(&self, id: &NodeId) -> bool {
        self.nodes.contains_key(id)
    }

    /// Get or create a node by kind and name, with stable ID semantics
    /// Returns (NodeId, was_already_present)
    pub fn get_or_create(&mut self, kind: NodeKind, name: &str) -> (NodeId, bool) {
        let id = NodeId::new(kind.clone(), name);
        if self.nodes.contains_key(&id) {
            return (id, true); // Already existed
        }
        let node = Node::new(id.clone(), kind, name);
        self.add_node(node);
        (id, false) // Just created
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Get all outgoing edges from a node
    pub fn outgoing_edges(&self, id: &NodeId) -> Vec<&Edge> {
        self.outgoing
            .get(id)
            .map(|indices| indices.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    /// Get all incoming edges to a node
    pub fn incoming_edges(&self, id: &NodeId) -> Vec<&Edge> {
        self.incoming
            .get(id)
            .map(|indices| indices.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    /// Get all nodes of a specific kind
    pub fn nodes_by_kind(&self, kind: &NodeKind) -> Vec<&Node> {
        self.nodes.values().filter(|n| &n.kind == kind).collect()
    }

    /// Find all nodes that depend on the given node (reverse traversal)
    pub fn dependents(&self, id: &NodeId) -> Vec<NodeId> {
        self.incoming_edges(id)
            .iter()
            .map(|e| e.from.clone())
            .collect()
    }

    /// Find all nodes that the given node depends on (forward traversal)
    pub fn dependencies(&self, id: &NodeId) -> Vec<NodeId> {
        self.outgoing_edges(id)
            .iter()
            .map(|e| e.to.clone())
            .collect()
    }

    /// Check if a node has any dependencies
    pub fn has_dependencies(&self, id: &NodeId) -> bool {
        !self.outgoing_edges(id).is_empty()
    }

    /// Check if a node is depended upon by others
    pub fn has_dependents(&self, id: &NodeId) -> bool {
        !self.incoming_edges(id).is_empty()
    }

    /// Get all functions in the graph
    pub fn functions(&self) -> Vec<&Node> {
        self.nodes_by_kind(&NodeKind::Function)
    }

    /// Get all types in the graph
    pub fn types(&self) -> Vec<&Node> {
        self.nodes_by_kind(&NodeKind::Type)
    }

    /// Get all exports in the graph
    pub fn exports(&self) -> Vec<&Node> {
        self.nodes_by_kind(&NodeKind::Export)
    }

    /// Get all struct nodes
    pub fn structs(&self) -> Vec<&Node> {
        self.nodes_by_kind(&NodeKind::Struct)
    }

    /// Get all embed nodes
    pub fn embeds(&self) -> Vec<&Node> {
        self.nodes_by_kind(&NodeKind::Embed)
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Iterate over all nodes
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    /// Iterate over all edges
    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter()
    }

    /// Compute topological order of nodes
    pub fn topological_sort(&self) -> Vec<NodeId> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();

        fn visit(
            graph: &DependencyGraph,
            node: &NodeId,
            visited: &mut HashSet<NodeId>,
            result: &mut Vec<NodeId>,
        ) {
            if visited.contains(node) {
                return;
            }
            visited.insert(node.clone());

            for edge in graph.outgoing_edges(node) {
                visit(graph, &edge.to, visited, result);
            }

            result.push(node.clone());
        }

        for node_id in self.nodes.keys() {
            visit(self, node_id, &mut visited, &mut result);
        }

        result
    }

    /// Compare this graph with another graph and classify nodes
    ///
    /// Returns a diff showing what changed between the two graphs.
    pub fn diff<'a>(&'a self, other: &'a DependencyGraph) -> GraphDiff<'a> {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();
        let mut unchanged = Vec::new();

        // Find added and changed nodes
        for node in other.nodes.values() {
            if let Some(old_node) = self.nodes.get(&node.id) {
                if old_node != node {
                    changed.push(node);
                } else {
                    unchanged.push(node);
                }
            } else {
                added.push(node);
            }
        }

        // Find removed nodes
        for id in self.nodes.keys() {
            if !other.nodes.contains_key(id) {
                removed.push(id.clone());
            }
        }

        GraphDiff {
            added,
            removed,
            changed,
            unchanged,
        }
    }

    /// Classify a node change as body edit, signature edit, or import edit
    pub fn classify_change(&self, node_id: &NodeId) -> ChangeClassification {
        if let Some(node) = self.get_node(node_id) {
            match node.kind {
                NodeKind::Function => {
                    // Check if this is an exported function (signature change matters)
                    if self.has_outgoing_edge(node_id, &EdgeKind::Exports) {
                        ChangeClassification::SignatureEdit
                    } else {
                        ChangeClassification::BodyEdit
                    }
                }
                NodeKind::File => ChangeClassification::ImportEdit,
                NodeKind::Type | NodeKind::Struct | NodeKind::Layout => {
                    ChangeClassification::TypeOrLayoutChange
                }
                _ => ChangeClassification::Other,
            }
        } else {
            ChangeClassification::Unknown
        }
    }

    /// Check if node has outgoing edge of given kind
    fn has_outgoing_edge(&self, node_id: &NodeId, kind: &EdgeKind) -> bool {
        self.outgoing_edges(node_id).iter().any(|e| &e.kind == kind)
    }

    /// Get affected exports when a declaration changes
    pub fn affected_exports(&self, decl_id: &NodeId) -> Vec<NodeId> {
        let mut affected = Vec::new();
        let mut visited = HashSet::new();
        let mut to_visit = vec![decl_id.clone()];

        while let Some(current) = to_visit.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            for edge in self.outgoing_edges(&current) {
                if edge.kind == EdgeKind::Exports && !affected.contains(&edge.to) {
                    affected.push(edge.to.clone());
                }
            }

            for dependent in self.dependents(&current) {
                if !visited.contains(&dependent) {
                    to_visit.push(dependent);
                }
            }

            for edge in self.outgoing_edges(&current) {
                if edge.kind == EdgeKind::Exports {
                    if !visited.contains(&edge.to) {
                        to_visit.push(edge.to.clone());
                    }
                }
            }
        }

        affected
    }

    /// Debug: print the graph structure
    pub fn debug_print(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DependencyGraph {{")?;
        writeln!(f, "  nodes ({}):", self.node_count())?;
        for node in self.nodes.values() {
            writeln!(f, "    {:?}: {}", node.id, node.name)?;
        }
        writeln!(f, "  edges ({}):", self.edge_count())?;
        for edge in &self.edges {
            writeln!(f, "    {:?} --{:?}--> {:?}", edge.from, edge.kind, edge.to)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of comparing two graphs
#[derive(Debug, Clone)]
pub struct GraphDiff<'a> {
    /// Nodes that were added (in new but not in old)
    pub added: Vec<&'a Node>,
    /// Nodes that were removed (in old but not in new)
    pub removed: Vec<NodeId>,
    /// Nodes that changed (same ID but different content)
    pub changed: Vec<&'a Node>,
    /// Nodes that are unchanged
    pub unchanged: Vec<&'a Node>,
}

/// Builder for constructing dependency graphs from snapshot data
pub struct GraphBuilder {
    graph: DependencyGraph,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
        }
    }

    /// Add a file node
    pub fn add_file(&mut self, path: &str) -> &mut Self {
        let id = NodeId::file(path);
        let node = Node::new(id, NodeKind::File, path);
        self.graph.add_node(node);
        self
    }

    /// Add a function node
    pub fn add_function(&mut self, name: &str, file: &str, line: u32) -> &mut Self {
        let id = NodeId::func(name);
        let loc = SourceLocation::new(file, line, 1);
        let node = Node::new(id, NodeKind::Function, name).with_location(loc);
        self.graph.add_node(node);
        self
    }

    /// Add a type node
    pub fn add_type(&mut self, name: &str) -> &mut Self {
        let id = NodeId::typ(name);
        let node = Node::new(id, NodeKind::Type, name);
        self.graph.add_node(node);
        self
    }

    /// Add a struct node
    pub fn add_struct(&mut self, name: &str) -> &mut Self {
        let id = NodeId::struct_(name);
        let node = Node::new(id, NodeKind::Struct, name);
        self.graph.add_node(node);
        self
    }

    /// Add a layout node
    pub fn add_layout(&mut self, name: &str, size: u64, align: u64) -> &mut Self {
        let id = NodeId::layout(name);
        let node = Node::new(id, NodeKind::Layout, name)
            .with_metadata("size", &size.to_string())
            .with_metadata("align", &align.to_string());
        self.graph.add_node(node);
        self
    }

    /// Add a comptime node
    pub fn add_comptime(&mut self, name: &str) -> &mut Self {
        let id = NodeId::comptime(name);
        let node = Node::new(id, NodeKind::Comptime, name);
        self.graph.add_node(node);
        self
    }

    /// Add an embed node
    pub fn add_embed(&mut self, path: &str, size: u64) -> &mut Self {
        let id = NodeId::embed(path);
        let node = Node::new(id, NodeKind::Embed, path).with_metadata("size", &size.to_string());
        self.graph.add_node(node);
        self
    }

    /// Add an export node
    pub fn add_export(&mut self, name: &str, symbol: &str) -> &mut Self {
        let id = NodeId::export(name);
        let node = Node::new(id, NodeKind::Export, name).with_metadata("symbol", symbol);
        self.graph.add_node(node);
        self
    }

    /// Add a link artifact node
    pub fn add_link_artifact(&mut self, name: &str) -> &mut Self {
        let id = NodeId::link(name);
        let node = Node::new(id, NodeKind::LinkArtifact, name);
        self.graph.add_node(node);
        self
    }

    /// Add a "uses" edge
    pub fn add_uses(&mut self, user: &NodeId, used: &NodeId) -> &mut Self {
        self.graph
            .add_edge(Edge::new(user.clone(), used.clone(), EdgeKind::Uses));
        self
    }

    /// Add a "defines" edge
    pub fn add_defines(&mut self, container: &NodeId, defined: &NodeId) -> &mut Self {
        self.graph.add_edge(Edge::new(
            container.clone(),
            defined.clone(),
            EdgeKind::Defines,
        ));
        self
    }

    /// Add an "exports" edge
    pub fn add_exports(&mut self, exporter: &NodeId, exported: &NodeId) -> &mut Self {
        self.graph.add_edge(Edge::new(
            exporter.clone(),
            exported.clone(),
            EdgeKind::Exports,
        ));
        self
    }

    /// Add a "links_to" edge
    pub fn add_links_to(&mut self, from: &NodeId, to: &NodeId) -> &mut Self {
        self.graph
            .add_edge(Edge::new(from.clone(), to.clone(), EdgeKind::LinksTo));
        self
    }

    /// Add a "references" edge
    pub fn add_references(&mut self, referrer: &NodeId, referred: &NodeId) -> &mut Self {
        self.graph.add_edge(Edge::new(
            referrer.clone(),
            referred.clone(),
            EdgeKind::References,
        ));
        self
    }

    /// Finish building and return the graph
    pub fn build(self) -> DependencyGraph {
        self.graph
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SnapSchema Integration
// ============================================================================

use zigmera_schema::zsnap::{DeclKind, Linkage, SnapSchema, Visibility};

/// Error types for graph population from snapshot
#[derive(Debug, Clone)]
pub enum GraphPopulationError {
    MissingDeclaration(String),
    InvalidNodeKind(String),
    IoError(String),
}

impl GraphPopulationError {
    pub fn is_missing_error(&self) -> bool {
        matches!(self, GraphPopulationError::MissingDeclaration(_))
    }
}

impl std::fmt::Display for GraphPopulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphPopulationError::MissingDeclaration(s) => {
                write!(f, "missing declaration: {}", s)
            }
            GraphPopulationError::InvalidNodeKind(s) => {
                write!(f, "invalid node kind: {}", s)
            }
            GraphPopulationError::IoError(s) => {
                write!(f, "IO error: {}", s)
            }
        }
    }
}

impl std::error::Error for GraphPopulationError {}

/// Builder extension for populating graph from SnapSchema
impl GraphBuilder {
    /// Populate graph from a compiler-emitted SnapSchema
    ///
    /// This converts compiler-emitted analysis-unit IDs and dependency edges
    /// into `zdep` nodes/edges without guessing from source text.
    pub fn from_snapshot(
        mut self,
        snapshot: &SnapSchema,
    ) -> Result<DependencyGraph, GraphPopulationError> {
        // Add all source files as nodes
        for source_file in &snapshot.source_files {
            let id = NodeId::file(&source_file.path);
            let node = Node::new(id, NodeKind::File, &source_file.path).with_metadata(
                "content_hash",
                &format!("{:?}", &source_file.content_hash[..8]),
            );
            self.graph.add_node(node);
        }

        // Add declarations as nodes
        for decl in &snapshot.decls {
            let node_kind = match decl.kind {
                DeclKind::Function => NodeKind::Function,
                DeclKind::Struct => NodeKind::Struct,
                DeclKind::Union => NodeKind::Type,
                DeclKind::Enum => NodeKind::Type,
                DeclKind::Opaque => NodeKind::Type,
                DeclKind::Var => NodeKind::Type,
                DeclKind::Const => NodeKind::Type,
                DeclKind::Import => NodeKind::File,
                DeclKind::TypeAlias => NodeKind::Type,
                DeclKind::ContainerAugmentation => NodeKind::Type,
            };

            let id = NodeId::new(node_kind.clone(), &decl.name);
            let node_id_for_edge = id.clone();
            let mut node = Node::new(id, node_kind, &decl.name);

            // Add access level as metadata
            let access_level = match decl.access_level {
                zigmera_schema::zsnap::AccessLevel::Private => "private",
                zigmera_schema::zsnap::AccessLevel::Pub => "pub",
                zigmera_schema::zsnap::AccessLevel::PubStage => "pub_stage",
            };
            node.metadata
                .insert("access_level".to_string(), access_level.to_string());

            // Link to owner file
            let owner_file_id = NodeId::file(&format!("file_id:{}", decl.owner_file));
            self.graph
                .add_node(Node::new(owner_file_id.clone(), NodeKind::File, ""));
            self.graph.add_edge(Edge::new(
                owner_file_id,
                node_id_for_edge,
                EdgeKind::Contains,
            ));

            self.graph.add_node(node);
        }

        // Add analysis units and their import edges
        for unit in &snapshot.analysis_units {
            let _unit_id = NodeId::new(NodeKind::File, &format!("unit:{}", unit.id));

            // Add edges for imports
            for import in &unit.imports {
                let from_id = NodeId::file(&format!("file_id:{}", import.from_file));
                let to_id = NodeId::file(&format!("file_id:{}", import.to_file));

                // Create the edge - from imports to (the target file is used by from)
                self.graph.add_edge(Edge::with_metadata(
                    from_id,
                    to_id,
                    EdgeKind::References,
                    &format!("line:{}", import.line),
                ));
            }
        }

        // Add type nodes from type table
        for type_record in &snapshot.types {
            if let Some(ref name) = type_record.name {
                let id = NodeId::typ(name);
                let mut node = Node::new(id.clone(), NodeKind::Type, name);

                if let Some(size) = type_record.size_bytes {
                    node.metadata.insert("size".to_string(), size.to_string());
                }
                if let Some(align) = type_record.alignment {
                    node.metadata
                        .insert("alignment".to_string(), align.to_string());
                }

                self.graph.add_node(node);
            }
        }

        // Add layout nodes
        for layout in &snapshot.layouts {
            let id = NodeId::layout(&format!("layout:{}", layout.id));
            let mut node = Node::new(
                id.clone(),
                NodeKind::Layout,
                &format!("layout:{}", layout.id),
            );

            node.metadata
                .insert("size".to_string(), layout.size_bytes.to_string());
            node.metadata
                .insert("alignment".to_string(), layout.alignment.to_string());
            node.metadata
                .insert("packed".to_string(), layout.packed.to_string());
            node.metadata
                .insert("extern".to_string(), layout.extern_.to_string());

            self.graph.add_node(node);

            // Link layout to its type
            let type_id = NodeId::typ(&format!("type_id:{}", layout.type_id));
            self.graph
                .add_edge(Edge::new(type_id, id, EdgeKind::Defines));
        }

        // Add export symbols
        for export in &snapshot.exports {
            let id = NodeId::export(&export.name);
            let mut node = Node::new(id.clone(), NodeKind::Export, &export.name);

            node.metadata
                .insert("symbol".to_string(), export.name.clone());
            node.metadata
                .insert("decl_id".to_string(), export.decl_id.to_string());

            let linkage = match export.linkage {
                Linkage::Internal => "internal",
                Linkage::Strong => "strong",
                Linkage::Weak => "weak",
                Linkage::LinkOnce => "linkonce",
            };
            node.metadata
                .insert("linkage".to_string(), linkage.to_string());

            let visibility = match export.visibility {
                Visibility::Private => "private",
                Visibility::Public => "public",
                Visibility::Exported => "exported",
            };
            node.metadata
                .insert("visibility".to_string(), visibility.to_string());

            self.graph.add_node(node);

            // Link to the declaration it exports
            let decl_id = NodeId::new(NodeKind::Function, &format!("decl:{}", export.decl_id));
            self.graph
                .add_edge(Edge::new(decl_id, id, EdgeKind::Exports));
        }

        Ok(self.graph)
    }
}

/// Result type for graph population
pub type GraphPopulationResult<T> = Result<T, GraphPopulationError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_creation() {
        let id = NodeId::func("my_function");
        assert_eq!(id.0, "Function:my_function");

        let id = NodeId::typ("MyType");
        assert_eq!(id.0, "Type:MyType");
    }

    #[test]
    fn test_add_nodes_and_edges() {
        let mut graph = DependencyGraph::new();

        graph.add_node(Node::new(
            NodeId::file("test.zig"),
            NodeKind::File,
            "test.zig",
        ));
        graph.add_node(Node::new(NodeId::func("add"), NodeKind::Function, "add"));

        graph.add_edge(Edge::new(
            NodeId::file("test.zig"),
            NodeId::func("add"),
            EdgeKind::Defines,
        ));

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_dependencies_and_dependents() {
        let mut graph = DependencyGraph::new();

        let file = NodeId::file("test.zig");
        let func = NodeId::func("add");
        let ret_type = NodeId::typ("c_int");

        graph.add_node(Node::new(file.clone(), NodeKind::File, "test.zig"));
        graph.add_node(Node::new(func.clone(), NodeKind::Function, "add"));
        graph.add_node(Node::new(ret_type.clone(), NodeKind::Type, "c_int"));

        graph.add_edge(Edge::new(file.clone(), func.clone(), EdgeKind::Defines));
        graph.add_edge(Edge::new(func.clone(), ret_type.clone(), EdgeKind::Uses));

        let deps = graph.dependencies(&func);
        assert!(deps.contains(&ret_type));

        let dependents = graph.dependents(&ret_type);
        assert!(dependents.contains(&func));
    }

    #[test]
    fn test_affected_exports() {
        let mut graph = DependencyGraph::new();

        let file = NodeId::file("test.zig");
        let struct_ = NodeId::struct_("Point");
        let export_fn = NodeId::func("create_point");
        let export = NodeId::export("create_point");

        graph.add_node(Node::new(file.clone(), NodeKind::File, "test.zig"));
        graph.add_node(Node::new(struct_.clone(), NodeKind::Struct, "Point"));
        graph.add_node(Node::new(
            export_fn.clone(),
            NodeKind::Function,
            "create_point",
        ));
        graph.add_node(Node::new(export.clone(), NodeKind::Export, "create_point"));

        graph.add_edge(Edge::new(file.clone(), struct_.clone(), EdgeKind::Defines));
        graph.add_edge(Edge::new(
            file.clone(),
            export_fn.clone(),
            EdgeKind::Defines,
        ));
        graph.add_edge(Edge::new(
            export_fn.clone(),
            struct_.clone(),
            EdgeKind::Uses,
        ));
        graph.add_edge(Edge::new(
            export_fn.clone(),
            export.clone(),
            EdgeKind::Exports,
        ));

        let affected = graph.affected_exports(&struct_);
        assert!(affected.contains(&export));
    }

    #[test]
    fn test_graph_builder() {
        let mut builder = GraphBuilder::new();

        builder.add_file("test.zig");
        builder.add_type("c_int");
        builder.add_function("add", "test.zig", 5);
        builder.add_function("myFunc", "test.zig", 10);

        let func = NodeId::func("add");
        let ret_type = NodeId::typ("c_int");
        builder.add_uses(&func, &ret_type);

        let graph = builder.build();
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = DependencyGraph::new();

        let a = NodeId::func("a");
        let b = NodeId::func("b");
        let c = NodeId::func("c");

        graph.add_node(Node::new(a.clone(), NodeKind::Function, "a"));
        graph.add_node(Node::new(b.clone(), NodeKind::Function, "b"));
        graph.add_node(Node::new(c.clone(), NodeKind::Function, "c"));

        // c depends on b, b depends on a
        graph.add_edge(Edge::new(c.clone(), b.clone(), EdgeKind::Uses));
        graph.add_edge(Edge::new(b.clone(), a.clone(), EdgeKind::Uses));

        let order = graph.topological_sort();
        let a_idx = order.iter().position(|id| id == &a).unwrap();
        let b_idx = order.iter().position(|id| id == &b).unwrap();
        let c_idx = order.iter().position(|id| id == &c).unwrap();

        // a should come before b, b before c
        assert!(a_idx < b_idx);
        assert!(b_idx < c_idx);
    }

    #[test]
    fn test_graph_builder_from_snapshot() {
        use zigmera_schema::zsnap::{
            AccessLevel, BuildOptions, DeclKind, DeclRef, SnapHeader, SourceFile, SCHEMA_VERSION,
            ZSNAP_MAGIC,
        };

        // Build a minimal SnapSchema for testing
        let snapshot = SnapSchema {
            header: SnapHeader {
                magic: *ZSNAP_MAGIC,
                schema_version: SCHEMA_VERSION,
                min_adapter_version: 1,
                zig_commit: [0u8; 20],
                target: "x86_64-linux".to_string(),
                backend: "static".to_string(),
                optimize_mode: "Release".to_string(),
                timestamp_ns: 0,
                source_file_count: 1,
                checksum: [0u8; 32],
            },
            build_options: BuildOptions {
                optimize_mode: "Release".to_string(),
                target: "x86_64-linux".to_string(),
                cpu_features: vec![],
                libc: None,
                build_mode: "Release".to_string(),
                entry: None,
                panic_mode: "unwind".to_string(),
            },
            source_files: vec![SourceFile {
                id: 0,
                path: "test.zig".to_string(),
                content_hash: [0u8; 32],
            }],
            decls: vec![DeclRef {
                id: 0,
                name: "myFunc".to_string(),
                owner_file: 0,
                kind: DeclKind::Function,
                access_level: AccessLevel::Pub,
            }],
            analysis_units: vec![],
            types: vec![],
            layouts: vec![],
            exports: vec![],
            air_bodies: vec![],
            comptime_calls: vec![],
            embed_files: vec![],
            c_imports: vec![],
        };

        let builder = GraphBuilder::new();
        let graph = builder.from_snapshot(&snapshot).expect("should build");

        // Verify file node was created (with path-based ID)
        let file_node = graph.get_node(&NodeId::file("test.zig"));
        assert!(file_node.is_some(), "file node should exist");

        // Verify decl node was created
        let func_node = graph.get_node(&NodeId::new(NodeKind::Function, "myFunc"));
        assert!(func_node.is_some(), "function node should exist");

        // Verify edge from file_id:0 (owner file reference) to the function
        let owner_file_id = NodeId::file("file_id:0");
        let edges = graph.outgoing_edges(&owner_file_id);
        assert_eq!(edges.len(), 1, "expected one edge from owner file");
        assert_eq!(edges[0].kind, EdgeKind::Contains);
    }

    #[test]
    fn test_edge_direction_semantics() {
        // Test edge direction semantics:
        // - dependencies() returns nodes that this node depends on (outgoing edges)
        // - dependents() returns nodes that depend on this node (incoming edges)
        let mut graph = DependencyGraph::new();

        let layout = NodeId::layout("PointLayout");
        let struct_ = NodeId::struct_("Point");

        graph.add_node(Node::new(layout.clone(), NodeKind::Layout, "PointLayout"));
        graph.add_node(Node::new(struct_.clone(), NodeKind::Struct, "Point"));

        // Layout defines the struct's memory layout
        // Edge direction: layout -> struct (layout defines struct)
        graph.add_edge(Edge::new(
            layout.clone(),
            struct_.clone(),
            EdgeKind::Defines,
        ));

        // struct_depends_on would be empty because struct doesn't define anything
        // (outgoing edges from struct)
        let struct_defines = graph.dependencies(&struct_);
        assert!(struct_defines.is_empty(), "struct defines nothing");

        // layout's dependents (incoming edges to layout) is empty
        let layout_dependents_incoming = graph.dependents(&layout);
        assert!(
            layout_dependents_incoming.is_empty(),
            "nothing depends on layout via incoming"
        );

        // But struct depends on layout: struct -> [layout] via incoming
        // Actually looking at the edge: from=layout, to=struct
        // So struct is the target, layout is the source
        // dependents(struct) looks at incoming edges TO struct = [layout]
        let struct_dependents = graph.dependents(&struct_);
        assert!(
            struct_dependents.contains(&layout),
            "struct has layout as dependent via incoming"
        );

        // layout has no dependents since the edge direction is layout -> struct
        // If we want symmetric semantics, we'd also need struct -> layout edge
    }

    #[test]
    fn test_invalidates_edge_direction() {
        // Test Invalidates edge: A -> Invalidates -> B means A invalidates B
        // When A changes, we need to rebuild B
        let mut graph = DependencyGraph::new();

        let type_node = NodeId::typ("MyType");
        let func_node = NodeId::func("usesMyType");

        graph.add_node(Node::new(type_node.clone(), NodeKind::Type, "MyType"));
        graph.add_node(Node::new(
            func_node.clone(),
            NodeKind::Function,
            "usesMyType",
        ));

        // Type change should invalidate the function that uses it
        // Edge direction: type -> func (type invalidates func)
        graph.add_edge(Edge::new(
            type_node.clone(),
            func_node.clone(),
            EdgeKind::Invalidates,
        ));

        // func_node depends on type_node (via outgoing edge from func's perspective)
        // Wait, the edge is type -> func, so from type's perspective func is outgoing
        // But we want to check: when type changes, what gets invalidated?
        // That's the incoming edges to type (none in this case) OR
        // the outgoing edges FROM type which points to func
        let type_invalidates = graph.outgoing_edges(&type_node);
        assert_eq!(type_invalidates.len(), 1);
        assert_eq!(type_invalidates[0].kind, EdgeKind::Invalidates);
        assert_eq!(type_invalidates[0].to, func_node);

        // For the engine: type changes -> invalidate type_invalidates (what type invalidates)
        // This means we traverse OUTGOING Invalidates edges from the changed node
    }

    // Task 53: Duplicate-node stability policy tests

    #[test]
    fn test_find_or_add_node_stable_id() {
        // When same node is added twice, returns (id, true) for existing (was already present)
        let mut graph = DependencyGraph::new();

        let id = NodeId::func("myFunc");
        let node1 = Node::new(id.clone(), NodeKind::Function, "myFunc");
        let node2 = Node::new(id.clone(), NodeKind::Function, "myFunc");

        let (result1_id, already_present1) = graph.find_or_add_node(node1);
        assert_eq!(graph.node_count(), 1);
        assert!(
            !already_present1,
            "first insert should not be already present"
        );

        // Adding again should return (same_id, true) for existing (was already present)
        let (result2_id, already_present2) = graph.find_or_add_node(node2);
        assert_eq!(graph.node_count(), 1, "duplicate should not increase count");
        assert!(already_present2, "second insert should be already present");
        assert_eq!(result1_id, result2_id);
    }

    #[test]
    fn test_duplicate_generic_instantiation() {
        // Generic instantiations with same args should get stable IDs
        let mut graph = DependencyGraph::new();

        // Two instantiations of same generic with same type args
        let inst1_id = NodeId::new(NodeKind::Function, "Vec[i32]_instantiation_1");
        let inst2_id = NodeId::new(NodeKind::Function, "Vec[i32]_instantiation_1");

        graph.add_node(Node::new(
            inst1_id.clone(),
            NodeKind::Function,
            "Vec[i32]_instantiation",
        ));
        assert_eq!(graph.node_count(), 1);

        // Adding same instantiation again - should not create duplicate
        graph.add_node(Node::new(
            inst2_id.clone(),
            NodeKind::Function,
            "Vec[i32]_instantiation",
        ));
        assert_eq!(
            graph.node_count(),
            1,
            "duplicate instantiation should not increase node count"
        );
    }

    #[test]
    fn test_duplicate_type_layout_nodes() {
        // Type/layout nodes with same stable keys should not duplicate
        let mut graph = DependencyGraph::new();

        let layout1_id = NodeId::layout("layout:42");
        let layout2_id = NodeId::layout("layout:42");

        graph.add_node(Node::new(layout1_id.clone(), NodeKind::Layout, "layout:42"));
        assert_eq!(graph.node_count(), 1);

        graph.add_node(Node::new(layout2_id.clone(), NodeKind::Layout, "layout:42"));
        assert_eq!(
            graph.node_count(),
            1,
            "duplicate layout node should not increase count"
        );
    }

    #[test]
    fn test_has_node_check() {
        let mut graph = DependencyGraph::new();
        let id = NodeId::func("exists");
        graph.add_node(Node::new(id.clone(), NodeKind::Function, "exists"));

        assert!(graph.has_node(&id));
        assert!(!graph.has_node(&NodeId::func("does_not_exist")));
    }

    #[test]
    fn test_get_or_create_returns_existing() {
        let mut graph = DependencyGraph::new();

        // First creation - returns (NodeId, was_already_present=false)
        let (id1, already_present1) = graph.get_or_create(NodeKind::Struct, "Point");
        assert_eq!(graph.node_count(), 1);
        assert!(!already_present1, "first call should create new node");

        // Second get_or_create with same kind/name - returns (same NodeId, was_already_present=true)
        let (id2, already_present2) = graph.get_or_create(NodeKind::Struct, "Point");
        assert_eq!(graph.node_count(), 1, "get_or_create should not duplicate");
        assert!(already_present2, "second call should find existing");
        assert_eq!(id1, id2, "should return same NodeId for same kind/name");
    }

    #[test]
    fn test_get_or_create_creates_new() {
        let mut graph = DependencyGraph::new();

        let (id, already_present) = graph.get_or_create(NodeKind::Type, "NewType");
        assert_eq!(graph.node_count(), 1);
        assert!(!already_present, "first call should create new node");
        assert_eq!(id.0, "Type:NewType");
    }

    #[test]
    fn test_find_or_add_with_different_kinds_same_name() {
        // Same name but different kind should create separate nodes
        let mut graph = DependencyGraph::new();

        let func_id = NodeId::new(NodeKind::Function, "MyName");
        let type_id = NodeId::new(NodeKind::Type, "MyName");

        // These are different IDs because kind is part of the ID
        assert_ne!(func_id.0, type_id.0);

        graph.add_node(Node::new(func_id.clone(), NodeKind::Function, "MyName"));
        graph.add_node(Node::new(type_id.clone(), NodeKind::Type, "MyName"));

        // Both should exist since they have different IDs
        assert_eq!(graph.node_count(), 2);
        assert!(graph.has_node(&func_id));
        assert!(graph.has_node(&type_id));
    }

    #[test]
    fn test_duplicate_in_separate_namespaces_files() {
        // Same name in different files should be different nodes
        let mut graph = DependencyGraph::new();

        let decl1 = NodeId::new(NodeKind::Function, "file_a.zig:myFunc");
        let decl2 = NodeId::new(NodeKind::Function, "file_b.zig:myFunc");

        // Different file paths mean different node IDs
        assert_ne!(decl1.0, decl2.0);

        graph.add_node(Node::new(decl1.clone(), NodeKind::Function, "myFunc"));
        graph.add_node(Node::new(decl2.clone(), NodeKind::Function, "myFunc"));

        // Both should exist as separate nodes
        assert_eq!(graph.node_count(), 2);
        assert!(graph.has_node(&decl1));
        assert!(graph.has_node(&decl2));
    }

    // Task 54: Graph diffing tests

    #[test]
    fn test_graph_diff_added_nodes() {
        let mut old_graph = DependencyGraph::new();
        old_graph.add_node(Node::new(
            NodeId::func("func1"),
            NodeKind::Function,
            "func1",
        ));

        let mut new_graph = DependencyGraph::new();
        new_graph.add_node(Node::new(
            NodeId::func("func1"),
            NodeKind::Function,
            "func1",
        ));
        new_graph.add_node(Node::new(
            NodeId::func("func2"),
            NodeKind::Function,
            "func2",
        ));

        let diff = old_graph.diff(&new_graph);
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].name, "func2");
        assert_eq!(diff.removed.len(), 0);
        assert_eq!(diff.changed.len(), 0);
    }

    #[test]
    fn test_graph_diff_removed_nodes() {
        let mut old_graph = DependencyGraph::new();
        old_graph.add_node(Node::new(
            NodeId::func("func1"),
            NodeKind::Function,
            "func1",
        ));
        old_graph.add_node(Node::new(
            NodeId::func("func2"),
            NodeKind::Function,
            "func2",
        ));

        let mut new_graph = DependencyGraph::new();
        new_graph.add_node(Node::new(
            NodeId::func("func1"),
            NodeKind::Function,
            "func1",
        ));

        let diff = old_graph.diff(&new_graph);
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].0, "Function:func2");
        assert_eq!(diff.added.len(), 0);
    }

    #[test]
    fn test_graph_diff_changed_nodes() {
        let mut old_graph = DependencyGraph::new();
        let mut node1 = Node::new(NodeId::func("func1"), NodeKind::Function, "func1");
        node1.metadata.insert("size".to_string(), "100".to_string());
        old_graph.add_node(node1);

        let mut new_graph = DependencyGraph::new();
        let mut node2 = Node::new(NodeId::func("func1"), NodeKind::Function, "func1");
        node2.metadata.insert("size".to_string(), "200".to_string());
        new_graph.add_node(node2);

        let diff = old_graph.diff(&new_graph);
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.changed[0].metadata.get("size").unwrap(), "200");
    }

    #[test]
    fn test_graph_diff_unchanged_nodes() {
        let mut old_graph = DependencyGraph::new();
        old_graph.add_node(Node::new(
            NodeId::func("func1"),
            NodeKind::Function,
            "func1",
        ));

        let mut new_graph = DependencyGraph::new();
        new_graph.add_node(Node::new(
            NodeId::func("func1"),
            NodeKind::Function,
            "func1",
        ));

        let diff = old_graph.diff(&new_graph);
        assert_eq!(diff.unchanged.len(), 1);
        assert_eq!(diff.added.len(), 0);
        assert_eq!(diff.removed.len(), 0);
    }

    #[test]
    fn test_classify_body_edit() {
        let mut graph = DependencyGraph::new();
        // Private function (no Exports edge)
        graph.add_node(Node::new(
            NodeId::func("private_func"),
            NodeKind::Function,
            "private_func",
        ));

        let classification = graph.classify_change(&NodeId::func("private_func"));
        assert_eq!(classification, ChangeClassification::BodyEdit);
    }

    #[test]
    fn test_classify_signature_edit() {
        let mut graph = DependencyGraph::new();
        // Exported function
        graph.add_node(Node::new(
            NodeId::func("exported_func"),
            NodeKind::Function,
            "exported_func",
        ));
        graph.add_node(Node::new(
            NodeId::export("exported_func"),
            NodeKind::Export,
            "exported_func",
        ));
        graph.add_edge(Edge::new(
            NodeId::func("exported_func"),
            NodeId::export("exported_func"),
            EdgeKind::Exports,
        ));

        let classification = graph.classify_change(&NodeId::func("exported_func"));
        assert_eq!(classification, ChangeClassification::SignatureEdit);
    }

    #[test]
    fn test_classify_import_edit() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new(
            NodeId::file("test.zig"),
            NodeKind::File,
            "test.zig",
        ));

        let classification = graph.classify_change(&NodeId::file("test.zig"));
        assert_eq!(classification, ChangeClassification::ImportEdit);
    }

    #[test]
    fn test_classify_type_change() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new(NodeId::typ("MyType"), NodeKind::Type, "MyType"));

        let classification = graph.classify_change(&NodeId::typ("MyType"));
        assert_eq!(classification, ChangeClassification::TypeOrLayoutChange);
    }

    #[test]
    fn test_classify_unknown_node() {
        let graph = DependencyGraph::new();
        let classification = graph.classify_change(&NodeId::func("nonexistent"));
        assert_eq!(classification, ChangeClassification::Unknown);
    }

    // Task 1 (PR 1): Authority boundary documentation test
    #[test]
    fn test_graph_authority_status_documented() {
        let module_doc = include_str!("mod.rs");
        assert!(
            module_doc.contains("NON-AUTHORITATIVE"),
            "graph/mod.rs must document non-authoritative status"
        );
        assert!(
            module_doc.contains("zigmera-lowering"),
            "graph/mod.rs must reference zigmera-lowering as authoritative"
        );
    }
}
