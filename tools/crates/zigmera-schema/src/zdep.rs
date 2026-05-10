//! `.zdep` dependency graph schema.

use serde::{Deserialize, Serialize};

/// Magic bytes for `.zdep` binary format.
pub const ZDEP_MAGIC: &[u8; 8] = b"ZDEP0001";

/// Current schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// `.zdep` dependency graph header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepHeader {
    pub magic: [u8; 8],
    pub schema_version: u32,
    pub timestamp_ns: u64,
    pub node_count: u64,
    pub edge_count: u64,
    pub checksum: [u8; 32],
}

/// Kind of graph node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    Source {
        path: String,
    },
    Decl {
        name: String,
        kind: String,
    },
    Function {
        decl_id: u64,
    },
    Type {
        type_id: u64,
    },
    Layout {
        layout_id: u64,
    },
    Generic {
        generic_id: u64,
        instantiation_id: u64,
    },
    Comptime {
        call_id: u64,
    },
    EmbedFile {
        path: String,
        content_hash: [u8; 32],
    },
    Export {
        symbol_name: String,
    },
    Object {
        path: String,
    },
    Link {
        inputs: Vec<String>,
    },
    Test {
        decl_id: u64,
    },
}

/// A node in the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: u64,
    pub kind: NodeKind,
}

/// Direction of an edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeDirection {
    /// A → B means A depends on B.
    DependsOn,
    /// A → B means A invalidates B.
    Invalidates,
}

/// Kind of edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeKind {
    Import,
    TypeLayout,
    FunctionBody,
    ComptimeCall,
    GenericInstantiation,
    ExportSymbol,
    LinkInput,
    Test,
}

/// An edge in the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: u64,
    pub to: u64,
    pub direction: EdgeDirection,
    pub kind: EdgeKind,
}

/// Complete `.zdep` dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepSchema {
    pub header: DepHeader,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl DepSchema {
    pub fn header_magic_valid(&self) -> bool {
        &self.header.magic == ZDEP_MAGIC
    }

    pub fn header_version_compatible(&self) -> bool {
        self.header.schema_version <= SCHEMA_VERSION
    }
}
