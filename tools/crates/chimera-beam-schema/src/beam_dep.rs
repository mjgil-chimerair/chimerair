//! `.beam_dep` BEAM dependency graph schema v1.
//!
//! Captures module dependency edges and their kinds.

use serde::{Deserialize, Serialize};

/// Magic bytes for `.beam_dep` binary format.
pub const BEAM_DEP_MAGIC: &[u8; 8] = b"BeamDep1";

/// Dependency kind describing the relationship between modules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DepKind {
    /// Direct function call from one module to another.
    FunctionCall,
    /// A process spawn linking two modules.
    Spawn,
    /// Bidirectional link between processes.
    Link,
    /// Unidirectional monitor between processes.
    Monitor,
    /// Registration of a name in the registry.
    Register,
    /// Code loading or hot replacement.
    CodeLoad,
    /// Message passing between processes.
    MessagePassing,
    /// NIF (Native Implemented Function) call.
    NifCall,
    /// Port communication.
    Port,
}

/// Edge direction in the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeDirection {
    /// Directed edge from source to target.
    Directed,
    /// Bidirectional edge (for links).
    Bidirectional,
}

/// A dependency edge between two modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamDepEdge {
    pub from_module: String,
    pub to_module: String,
    pub kind: DepKind,
    pub direction: EdgeDirection,
    pub detail: Option<String>,
}

impl BeamDepEdge {
    pub fn new(
        from_module: impl Into<String>,
        to_module: impl Into<String>,
        kind: DepKind,
    ) -> Self {
        BeamDepEdge {
            from_module: from_module.into(),
            to_module: to_module.into(),
            kind,
            direction: EdgeDirection::Directed,
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn bidirectional(mut self) -> Self {
        self.direction = EdgeDirection::Bidirectional;
        self
    }
}

/// A node in the dependency graph (represents a module).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamDepNode {
    pub module_name: String,
    pub kind: NodeKind,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
}

/// Kind of node in the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeKind {
    /// Regular BEAM module.
    Module,
    /// Application callback module.
    Application,
    /// Supervisor module.
    Supervisor,
    /// Worker module.
    Worker,
    /// GenServer module.
    GenServer,
    /// GenStatem module.
    GenStatem,
    /// Port module.
    Port,
}

/// `.beam_dep` dependency graph header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamDepHeader {
    pub magic: [u8; 8],
    pub schema_version: u32,
    pub min_adapter_version: u32,
    pub node_count: u32,
    pub edge_count: u32,
    pub checksum: [u8; 32],
}

impl BeamDepHeader {
    pub fn new(node_count: u32, edge_count: u32) -> Self {
        BeamDepHeader {
            magic: *BEAM_DEP_MAGIC,
            schema_version: 1,
            min_adapter_version: 1,
            node_count,
            edge_count,
            checksum: [0u8; 32],
        }
    }
}

/// Full BEAM dependency graph schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamDepSchema {
    pub header: BeamDepHeader,
    pub nodes: Vec<BeamDepNode>,
    pub edges: Vec<BeamDepEdge>,
}

impl BeamDepSchema {
    pub fn new() -> Self {
        BeamDepSchema {
            header: BeamDepHeader::new(0, 0),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: BeamDepNode) {
        self.header.node_count += 1;
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: BeamDepEdge) {
        self.header.edge_count += 1;
        self.edges.push(edge);
    }
}

impl Default for BeamDepSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beam_dep_header_new() {
        let header = BeamDepHeader::new(5, 10);
        assert_eq!(header.node_count, 5);
        assert_eq!(header.edge_count, 10);
        assert_eq!(header.magic, *BEAM_DEP_MAGIC);
    }

    #[test]
    fn test_beam_dep_edge_creation() {
        let edge = BeamDepEdge::new("module_a", "module_b", DepKind::FunctionCall);
        assert_eq!(edge.from_module, "module_a");
        assert_eq!(edge.to_module, "module_b");
        assert!(matches!(edge.kind, DepKind::FunctionCall));
        assert!(edge.detail.is_none());
    }

    #[test]
    fn test_beam_dep_edge_with_detail() {
        let edge = BeamDepEdge::new("module_a", "module_b", DepKind::Spawn)
            .with_detail("spawning worker process");
        assert_eq!(edge.detail, Some("spawning worker process".to_string()));
    }

    #[test]
    fn test_beam_dep_edge_bidirectional() {
        let edge = BeamDepEdge::new("module_a", "module_b", DepKind::Link).bidirectional();
        assert_eq!(edge.direction, EdgeDirection::Bidirectional);
    }

    #[test]
    fn test_node_kind_serialization() {
        let kind = NodeKind::GenServer;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"GenServer\"");
    }

    #[test]
    fn test_dep_kind_serialization() {
        let kind = DepKind::NifCall;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"NifCall\"");
    }

    #[test]
    fn test_beam_dep_schema_add_node() {
        let mut schema = BeamDepSchema::new();
        let node = BeamDepNode {
            module_name: "my_module".to_string(),
            kind: NodeKind::Module,
            dependencies: vec![],
            dependents: vec![],
        };
        schema.add_node(node);
        assert_eq!(schema.header.node_count, 1);
        assert_eq!(schema.nodes.len(), 1);
    }

    #[test]
    fn test_beam_dep_schema_add_edge() {
        let mut schema = BeamDepSchema::new();
        let edge = BeamDepEdge::new("a", "b", DepKind::FunctionCall);
        schema.add_edge(edge);
        assert_eq!(schema.header.edge_count, 1);
        assert_eq!(schema.edges.len(), 1);
    }
}
