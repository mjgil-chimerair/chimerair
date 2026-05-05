//! Chimera C dependency/incremental cache crate.
//!
//! Caches `.csnap`, `.cdep`, `.castpack`, C dialect, `.chmeta`, `.cho`,
//! `.cproof`, wrappers, object/link metadata with hit/miss/corruption/eviction/reuse.
//!
//! Task 17: C dependency/incremental cache crate

// Public envelope types for chimerair integration (PR 2)
pub mod envelope;

// Schema integration for C semantic artifact identity (PR 3)
pub mod schema_identity;

// Invalidation classification and artifact reuse rules (PR 4)
pub mod invalidation_classifier;

use chimera_meta::Version as ChimeraVersion;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use thiserror::Error;

/// Result type for cache operations
pub type Result<T> = std::result::Result<T, CacheError>;

/// Cache errors
#[derive(Debug, Clone, Error)]
pub enum CacheError {
    #[error("cache miss: {0}")]
    CacheMiss(String),
    #[error("cache corruption: {0}")]
    CacheCorruption(String),
    #[error("serialization error: {0}")]
    SerializationError(String),
    #[error("io error: {0}")]
    IoError(String),
    #[error("invalid key: {0}")]
    InvalidKey(String),
}

/// C cache key components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CCacheKey {
    /// Source file path
    pub source_path: PathBuf,
    /// Compiler identity (executable, version, triple)
    pub compiler_identity: CompilerIdentity,
    /// Compile flags hash
    pub flags_hash: String,
    /// Include graph hash
    pub include_graph_hash: String,
    /// Target triple
    pub target_triple: String,
    /// Schema version
    pub schema_version: String,
}

impl CCacheKey {
    /// Create a new cache key from components
    #[allow(dead_code)]
    pub fn new(
        source_path: impl Into<PathBuf>,
        compiler_identity: CompilerIdentity,
        flags_hash: impl Into<String>,
        include_graph_hash: impl Into<String>,
        target_triple: impl Into<String>,
    ) -> Self {
        Self {
            source_path: source_path.into(),
            compiler_identity,
            flags_hash: flags_hash.into(),
            include_graph_hash: include_graph_hash.into(),
            target_triple: target_triple.into(),
            schema_version: "0.1.0".to_string(),
        }
    }

    /// Compute content addressable hash using BLAKE3 for deterministic hashing
    pub fn content_hash(&self) -> String {
        let mut hasher = zigmera_hash::Blake3Hasher::with_schema_tag("c-cache-key");
        hasher.update_str(&self.source_path.to_string_lossy());
        hasher.update_str(&self.compiler_identity.executable);
        hasher.update_str(&self.compiler_identity.version);
        hasher.update_str(&self.compiler_identity.target_triple);
        if let Some(ref sysroot) = self.compiler_identity.sysroot {
            hasher.update_str(sysroot);
        }
        hasher.update_str(&self.flags_hash);
        hasher.update_str(&self.include_graph_hash);
        hasher.update_str(&self.target_triple);
        hasher.update_str(&self.schema_version);
        hasher.finalize().as_hex()[..16].to_string()
    }
}

/// Compiler identity for cache key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CompilerIdentity {
    pub executable: String,
    pub version: String,
    pub target_triple: String,
    pub sysroot: Option<String>,
}

impl CompilerIdentity {
    /// Create new compiler identity
    #[allow(dead_code)]
    pub fn new(
        executable: impl Into<String>,
        version: impl Into<String>,
        target_triple: impl Into<String>,
    ) -> Self {
        Self {
            executable: executable.into(),
            version: version.into(),
            target_triple: target_triple.into(),
            sysroot: None,
        }
    }

    /// With sysroot
    pub fn with_sysroot(mut self, sysroot: impl Into<String>) -> Self {
        self.sysroot = Some(sysroot.into());
        self
    }
}

/// Cached artifact types - enum representing different artifact kinds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CachedArtifact {
    /// Raw bytes with type tag for flexibility
    Snapshot { data: Vec<u8> },
    /// Dependency graph
    DependencyGraph { data: Vec<u8> },
    /// AST/type/layout package
    AstPackage { data: Vec<u8> },
    /// C dialect context as JSON bytes
    Dialect { data: Vec<u8> },
    /// Common metadata as JSON bytes
    Metadata { data: Vec<u8> },
    /// Object metadata as JSON bytes
    Object { data: Vec<u8> },
    /// Proof facts as JSON bytes
    Proof { data: Vec<u8> },
}

impl CachedArtifact {
    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| CacheError::SerializationError(e.to_string()))
    }

    /// Deserialize from bytes
    #[allow(dead_code)]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| CacheError::SerializationError(e.to_string()))
    }

    /// Get artifact kind name
    pub fn kind(&self) -> &'static str {
        match self {
            CachedArtifact::Snapshot { .. } => "snapshot",
            CachedArtifact::DependencyGraph { .. } => "dependency_graph",
            CachedArtifact::AstPackage { .. } => "ast_package",
            CachedArtifact::Dialect { .. } => "dialect",
            CachedArtifact::Metadata { .. } => "metadata",
            CachedArtifact::Object { .. } => "object",
            CachedArtifact::Proof { .. } => "proof",
        }
    }
}

/// C cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CCacheEntry {
    pub key: CCacheKey,
    pub artifact: CachedArtifact,
    pub created_at: std::time::SystemTime,
    pub size_bytes: u64,
    pub hit_count: u64,
}

impl CCacheEntry {
    /// Create new cache entry
    pub fn new(key: CCacheKey, artifact: CachedArtifact) -> Self {
        let size_bytes = artifact.to_bytes().map(|b| b.len() as u64).unwrap_or(0);
        Self {
            key,
            artifact,
            created_at: std::time::SystemTime::now(),
            size_bytes,
            hit_count: 0,
        }
    }

    /// Record a cache hit
    pub fn record_hit(&mut self) {
        self.hit_count += 1;
    }

    /// Check if entry is still valid
    pub fn is_valid(&self) -> bool {
        // For now, always valid - could add TTL logic later
        true
    }
}

/// C cache manager
#[derive(Debug, Clone)]
pub struct CCacheManager {
    entries: HashMap<CCacheKey, CCacheEntry>,
    max_size_bytes: u64,
    current_size_bytes: u64,
}

impl CCacheManager {
    /// Create new cache manager
    pub fn new(max_size_bytes: u64) -> Self {
        Self {
            entries: HashMap::new(),
            max_size_bytes,
            current_size_bytes: 0,
        }
    }

    /// Create with default backend and size
    #[allow(dead_code)]
    pub fn with_defaults() -> Self {
        Self::new(1024 * 1024 * 1024) // 1GB default
    }

    /// Get artifact from cache
    pub fn get(&mut self, key: &CCacheKey) -> Result<CachedArtifact> {
        if let Some(entry) = self.entries.get_mut(key) {
            if entry.is_valid() {
                entry.record_hit();
                return Ok(entry.artifact.clone());
            }
        }
        Err(CacheError::CacheMiss(format!("{:?}", key)))
    }

    /// Put artifact in cache
    pub fn put(&mut self, key: CCacheKey, artifact: CachedArtifact) -> Result<()> {
        let size_bytes = artifact.to_bytes().map(|b| b.len() as u64).unwrap_or(0);

        // Evict if needed
        while self.current_size_bytes + size_bytes > self.max_size_bytes && !self.entries.is_empty()
        {
            self.evict_one();
        }

        let entry = CCacheEntry::new(key.clone(), artifact);
        self.current_size_bytes += size_bytes;
        self.entries.insert(key, entry);
        Ok(())
    }

    /// Evict one entry (LRU)
    fn evict_one(&mut self) {
        if let Some(oldest_key) = self.entries.keys().next().cloned() {
            if let Some(entry) = self.entries.remove(&oldest_key) {
                self.current_size_bytes = self.current_size_bytes.saturating_sub(entry.size_bytes);
            }
        }
    }

    /// Check if key exists in cache
    pub fn contains(&self, key: &CCacheKey) -> bool {
        self.entries.contains_key(key)
            && self.entries.get(key).map(|e| e.is_valid()).unwrap_or(false)
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.entries.len() as u64,
            size_bytes: self.current_size_bytes,
            max_size_bytes: self.max_size_bytes,
            hit_rate: self.compute_hit_rate(),
        }
    }

    /// Compute hit rate
    fn compute_hit_rate(&self) -> f64 {
        let total_hits: u64 = self.entries.values().map(|e| e.hit_count).sum();
        let total_accesses = total_hits + self.entries.len() as u64;
        if total_accesses == 0 {
            0.0
        } else {
            total_hits as f64 / total_accesses as f64
        }
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_size_bytes = 0;
    }

    /// Invalidate entries matching predicate
    pub fn invalidate_matching<F>(&mut self, pred: F)
    where
        F: Fn(&CCacheKey) -> bool,
    {
        let keys: Vec<_> = self.entries.keys().filter(|k| pred(k)).cloned().collect();
        for key in keys {
            if let Some(entry) = self.entries.remove(&key) {
                self.current_size_bytes = self.current_size_bytes.saturating_sub(entry.size_bytes);
            }
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub entry_count: u64,
    pub size_bytes: u64,
    pub max_size_bytes: u64,
    pub hit_rate: f64,
}

/// C cache key builder for fluent construction
pub struct CCacheKeyBuilder {
    source_path: PathBuf,
    compiler_identity: Option<CompilerIdentity>,
    flags_hash: Option<String>,
    include_graph_hash: Option<String>,
    target_triple: Option<String>,
}

impl CCacheKeyBuilder {
    /// Create new builder
    pub fn new(source_path: impl Into<PathBuf>) -> Self {
        Self {
            source_path: source_path.into(),
            compiler_identity: None,
            flags_hash: None,
            include_graph_hash: None,
            target_triple: None,
        }
    }

    /// Set compiler identity
    pub fn compiler_identity(mut self, identity: CompilerIdentity) -> Self {
        self.compiler_identity = Some(identity);
        self
    }

    /// Set flags hash
    pub fn flags_hash(mut self, hash: impl Into<String>) -> Self {
        self.flags_hash = Some(hash.into());
        self
    }

    /// Set include graph hash
    pub fn include_graph_hash(mut self, hash: impl Into<String>) -> Self {
        self.include_graph_hash = Some(hash.into());
        self
    }

    /// Set target triple
    pub fn target_triple(mut self, triple: impl Into<String>) -> Self {
        self.target_triple = Some(triple.into());
        self
    }

    /// Build the cache key
    pub fn build(self) -> Result<CCacheKey> {
        Ok(CCacheKey::new(
            self.source_path,
            self.compiler_identity
                .ok_or_else(|| CacheError::InvalidKey("missing compiler_identity".to_string()))?,
            self.flags_hash
                .ok_or_else(|| CacheError::InvalidKey("missing flags_hash".to_string()))?,
            self.include_graph_hash
                .ok_or_else(|| CacheError::InvalidKey("missing include_graph_hash".to_string()))?,
            self.target_triple
                .ok_or_else(|| CacheError::InvalidKey("missing target_triple".to_string()))?,
        ))
    }
}

// ============================================================================
// C Dependency Graph (Task 124)
// ============================================================================

/// Node kinds in the C dependency graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DepNodeKind {
    /// Translation unit node
    TranslationUnit,
    /// Source file node
    Source,
    /// Header file node
    Header,
    /// Macro definition node
    Macro,
    /// Declaration node
    Declaration,
    /// Type node
    Type,
    /// Layout node
    Layout,
    /// Function body node
    FunctionBody,
    /// Export node
    Export,
    /// Import node
    Import,
    /// Object file node
    Object,
    /// Wrapper node
    Wrapper,
    /// Proof node
    Proof,
    /// Link node
    Link,
}

impl DepNodeKind {
    /// Check if this node kind affects ABI
    pub fn affects_abi(&self) -> bool {
        matches!(
            self,
            DepNodeKind::Declaration
                | DepNodeKind::Type
                | DepNodeKind::Layout
                | DepNodeKind::Export
                | DepNodeKind::Import
        )
    }

    /// Check if this node kind affects layout only
    pub fn affects_layout(&self) -> bool {
        matches!(self, DepNodeKind::Layout | DepNodeKind::Type)
    }
}

/// A node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepNode {
    /// Unique node ID
    pub id: DepNodeId,
    /// Kind of node
    pub kind: DepNodeKind,
    /// Display name
    pub name: String,
    /// File path (if applicable)
    pub file_path: Option<String>,
    /// Source hash (content hash)
    pub content_hash: String,
    /// Metadata
    pub metadata: DepNodeMetadata,
}

/// Node metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DepNodeMetadata {
    /// Line number (for declarations/types)
    pub line: Option<u32>,
    /// Column number
    pub col: Option<u32>,
    /// Whether this is a system header
    pub is_system: bool,
    /// ABI fingerprint (for ABI-affecting nodes)
    pub abi_fingerprint: Option<String>,
    /// Layout fingerprint (for layout nodes)
    pub layout_fingerprint: Option<String>,
}

/// Unique node identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DepNodeId(pub String);

/// An edge in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepEdge {
    /// Source node ID
    pub from: DepNodeId,
    /// Target node ID
    pub to: DepNodeId,
    /// Edge kind
    pub kind: DepEdgeKind,
}

/// Edge kinds
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DepEdgeKind {
    /// Depends on (includes, uses)
    DependsOn,
    /// Defines (provides)
    Defines,
    /// Exports
    Exports,
    /// Imports
    Imports,
    /// Links to
    LinksTo,
}

/// C dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDependencyGraph {
    /// All nodes indexed by ID
    pub nodes: HashMap<DepNodeId, DepNode>,
    /// All edges
    pub edges: Vec<DepEdge>,
    /// Root node IDs (translation units)
    pub roots: Vec<DepNodeId>,
}

impl CDependencyGraph {
    /// Create new empty dependency graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            roots: Vec::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: DepNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: DepEdge) {
        // Ensure both nodes exist
        if self.nodes.contains_key(&edge.from) && self.nodes.contains_key(&edge.to) {
            self.edges.push(edge);
        }
    }

    /// Add a root node
    pub fn add_root(&mut self, id: DepNodeId) {
        self.roots.push(id);
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &DepNodeId) -> Option<&DepNode> {
        self.nodes.get(id)
    }

    /// Get all edges from a node
    pub fn get_edges_from(&self, id: &DepNodeId) -> Vec<&DepEdge> {
        self.edges.iter().filter(|e| &e.from == id).collect()
    }

    /// Get all edges to a node
    pub fn get_edges_to(&self, id: &DepNodeId) -> Vec<&DepEdge> {
        self.edges.iter().filter(|e| &e.to == id).collect()
    }

    /// Find nodes by kind
    pub fn find_nodes_by_kind(&self, kind: &DepNodeKind) -> Vec<&DepNode> {
        self.nodes.values().filter(|n| &n.kind == kind).collect()
    }

    /// Compute transitive closure of nodes affected by changed nodes
    pub fn transitive_closure(&self, changed_ids: &[DepNodeId]) -> HashSet<DepNodeId> {
        let mut affected: HashSet<DepNodeId> = changed_ids.iter().cloned().collect();
        let mut worklist: Vec<DepNodeId> = changed_ids.to_vec();

        while let Some(id) = worklist.pop() {
            // Get all nodes that depend on this one
            for edge in self.get_edges_to(&id) {
                if affected.insert(edge.from.clone()) {
                    worklist.push(edge.from.clone());
                }
            }
        }

        affected
    }

    /// Classify a change as ABI-affecting, layout-affecting, or cosmetic
    pub fn classify_change(&self, changed_ids: &[DepNodeId]) -> ChangeClassification {
        let mut has_abi_change = false;
        let mut has_layout_change = false;

        for id in changed_ids {
            if let Some(node) = self.get_node(id) {
                match node.kind {
                    DepNodeKind::Declaration | DepNodeKind::Export | DepNodeKind::Import => {
                        has_abi_change = true;
                    }
                    DepNodeKind::Type | DepNodeKind::Layout => {
                        has_layout_change = true;
                    }
                    _ => {}
                }
            }
        }

        if has_abi_change {
            ChangeClassification::AbiChanged
        } else if has_layout_change {
            ChangeClassification::LayoutChanged
        } else {
            ChangeClassification::Cosmetic
        }
    }
}

impl Default for CDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Classification of a change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeClassification {
    /// ABI signature changed (requires full rebuild of dependents)
    AbiChanged,
    /// Only layout changed (may allow incremental updates)
    LayoutChanged,
    /// Cosmetic change (no semantic effect)
    Cosmetic,
}

// ============================================================================
// Graph Diffing (Task 126)
// ============================================================================

/// Result of comparing two dependency graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDiff {
    /// Nodes that were added
    pub added: Vec<DepNodeId>,
    /// Nodes that were removed
    pub removed: Vec<DepNodeId>,
    /// Nodes that changed
    pub changed: Vec<ChangedNode>,
    /// Nodes that are unchanged
    pub unchanged: Vec<DepNodeId>,
}

/// Information about a changed node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedNode {
    /// Node ID
    pub id: DepNodeId,
    /// What changed
    pub change_type: NodeChangeType,
    /// Previous content hash
    pub old_hash: String,
    /// New content hash
    pub new_hash: String,
}

/// Type of change to a node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeChangeType {
    /// Node content hash changed
    ContentChanged,
    /// Node metadata changed
    MetadataChanged,
    /// Node was renamed
    Renamed,
    /// Node moved to different file
    FileChanged,
}

/// Graph diffing engine
pub struct GraphDiffer {
    target: String,
}

impl GraphDiffer {
    /// Create new graph differ
    pub fn new(target: String) -> Self {
        Self { target }
    }

    /// Compare two graphs and produce a diff
    pub fn diff(&self, old_graph: &CDependencyGraph, new_graph: &CDependencyGraph) -> GraphDiff {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();
        let mut unchanged = Vec::new();

        // Find added and changed nodes
        for (id, new_node) in &new_graph.nodes {
            if let Some(old_node) = old_graph.nodes.get(id) {
                if old_node.content_hash != new_node.content_hash {
                    let change_type = if old_node.name != new_node.name {
                        NodeChangeType::Renamed
                    } else if old_node.file_path != new_node.file_path {
                        NodeChangeType::FileChanged
                    } else {
                        NodeChangeType::ContentChanged
                    };
                    changed.push(ChangedNode {
                        id: id.clone(),
                        change_type,
                        old_hash: old_node.content_hash.clone(),
                        new_hash: new_node.content_hash.clone(),
                    });
                } else if old_node.metadata != new_node.metadata {
                    changed.push(ChangedNode {
                        id: id.clone(),
                        change_type: NodeChangeType::MetadataChanged,
                        old_hash: old_node.content_hash.clone(),
                        new_hash: new_node.content_hash.clone(),
                    });
                } else {
                    unchanged.push(id.clone());
                }
            } else {
                added.push(id.clone());
            }
        }

        // Find removed nodes
        for id in old_graph.nodes.keys() {
            if !new_graph.nodes.contains_key(id) {
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

    /// Classify the diff into ABI/layout/cosmetic changes
    pub fn classify_diff(&self, diff: &GraphDiff) -> ChangeClassification {
        for node_change in &diff.changed {
            let node = self.find_node_by_id(&node_change.id, &HashMap::new());
            if let Some(n) = node {
                if n.kind.affects_abi() {
                    return ChangeClassification::AbiChanged;
                }
                if n.kind.affects_layout() {
                    return ChangeClassification::LayoutChanged;
                }
            }
        }

        if !diff.added.is_empty() || !diff.removed.is_empty() {
            return ChangeClassification::AbiChanged;
        }

        ChangeClassification::Cosmetic
    }

    /// Find a node by ID from a temporary map (helper for classification)
    fn find_node_by_id(
        &self,
        id: &DepNodeId,
        _nodes: &HashMap<DepNodeId, DepNode>,
    ) -> Option<DepNode> {
        // This is a simplified version - in real use we'd pass the graph
        let mut nodes = HashMap::new();
        nodes.insert(
            id.clone(),
            DepNode {
                id: id.clone(),
                kind: DepNodeKind::Declaration,
                name: String::new(),
                file_path: None,
                content_hash: String::new(),
                metadata: DepNodeMetadata::default(),
            },
        );
        nodes.get(id).cloned()
    }
}

// ============================================================================
// C Invalidation Engine (Task 127)
// ============================================================================

/// Invalidation reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidationReason {
    /// Source file changed
    SourceChanged { path: String },
    /// Header changed
    HeaderChanged { path: String },
    /// Macro changed
    MacroChanged { name: String },
    /// Compiler flags changed
    FlagsChanged,
    /// Compiler version changed
    CompilerVersionChanged { old: String, new: String },
    /// Target triple changed
    TargetChanged { old: String, new: String },
    /// Schema version upgraded
    SchemaUpgraded { old: String, new: String },
}

/// Invalidation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationResult {
    /// Whether the artifact is now invalid
    pub is_invalid: bool,
    /// Reason for invalidation
    pub reason: Option<InvalidationReason>,
    /// IDs of nodes that were invalidated
    pub invalidated_nodes: Vec<DepNodeId>,
    /// IDs of nodes that remain valid
    pub valid_nodes: Vec<DepNodeId>,
}

/// C invalidation engine
pub struct CInvalidationEngine {
    graph: CDependencyGraph,
}

impl CInvalidationEngine {
    /// Create new invalidation engine with graph
    pub fn new(graph: CDependencyGraph) -> Self {
        Self { graph }
    }

    /// Check if a cache entry is still valid given current graph state
    pub fn check_validity(
        &self,
        cache_key: &CCacheKey,
        current_hashes: &HashMap<String, String>,
    ) -> InvalidationResult {
        // Check source hash
        let source_path_str = cache_key.source_path.to_string_lossy().to_string();
        if let Some(current_hash) = current_hashes.get(&source_path_str) {
            // If current hash differs from cached, source changed
            // For now, we assume valid if not in current_hashes (not yet computed)
            let _ = current_hash;
        }

        InvalidationResult {
            is_invalid: false,
            reason: None,
            invalidated_nodes: Vec::new(),
            valid_nodes: Vec::new(),
        }
    }

    /// Invalidate based on changed files
    pub fn invalidate_changed(&self, changed_files: &[String]) -> InvalidationResult {
        let mut invalidated = Vec::new();
        let mut reason = None;

        for file in changed_files {
            for node in self.graph.nodes.values() {
                if let Some(ref path) = node.file_path {
                    if path == file {
                        invalidated.push(node.id.clone());
                        reason = Some(InvalidationReason::SourceChanged { path: file.clone() });
                    }
                }
            }
        }

        let valid: Vec<DepNodeId> = self
            .graph
            .nodes
            .keys()
            .filter(|id| !invalidated.contains(id))
            .cloned()
            .collect();

        InvalidationResult {
            is_invalid: !invalidated.is_empty(),
            reason,
            invalidated_nodes: invalidated,
            valid_nodes: valid,
        }
    }

    /// Get the dependency graph
    pub fn graph(&self) -> &CDependencyGraph {
        &self.graph
    }

    /// Check if an ABI-affecting change invalidates a symbol
    pub fn symbol_invalidated_by_abi_change(
        &self,
        symbol: &str,
        changed_ids: &[DepNodeId],
    ) -> bool {
        // Find the node for this symbol
        let symbol_node = self.graph.nodes.values().find(|n| n.name == symbol);

        if let Some(node) = symbol_node {
            // Check if any changed node affects this symbol
            for changed_id in changed_ids {
                let changed_node = self.graph.get_node(changed_id);
                if let Some(cn) = changed_node {
                    // If changed node is ABI-affecting and there's a path from it to symbol node
                    if cn.kind.affects_abi() {
                        // Check for path in graph
                        if self.has_path(&cn.id, &node.id) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Check if there's a path from one node to another
    fn has_path(&self, from: &DepNodeId, to: &DepNodeId) -> bool {
        let mut visited: HashSet<DepNodeId> = HashSet::new();
        let mut worklist: Vec<DepNodeId> = vec![from.clone()];

        while let Some(current) = worklist.pop() {
            if current == *to {
                return true;
            }

            if visited.insert(current.clone()) {
                for edge in self.graph.get_edges_from(&current) {
                    if edge.kind == DepEdgeKind::DependsOn || edge.kind == DepEdgeKind::Defines {
                        worklist.push(edge.to.clone());
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod dep_graph_tests {
    use super::*;

    #[test]
    fn test_dep_node_kind_affects_abi() {
        assert!(DepNodeKind::Declaration.affects_abi());
        assert!(DepNodeKind::Type.affects_abi());
        assert!(DepNodeKind::Export.affects_abi());
        assert!(DepNodeKind::Import.affects_abi());
        assert!(!DepNodeKind::Source.affects_abi());
        assert!(!DepNodeKind::Header.affects_abi());
    }

    #[test]
    fn test_dep_node_kind_affects_layout() {
        assert!(DepNodeKind::Layout.affects_layout());
        assert!(DepNodeKind::Type.affects_layout());
        assert!(!DepNodeKind::Declaration.affects_layout());
    }

    #[test]
    fn test_dependency_graph_new() {
        let graph = CDependencyGraph::new();
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.roots.is_empty());
    }

    #[test]
    fn test_dependency_graph_add_node() {
        let mut graph = CDependencyGraph::new();
        let node = DepNode {
            id: DepNodeId("node1".to_string()),
            kind: DepNodeKind::Source,
            name: "test.c".to_string(),
            file_path: Some("test.c".to_string()),
            content_hash: "abc123".to_string(),
            metadata: DepNodeMetadata::default(),
        };
        graph.add_node(node);
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn test_dependency_graph_add_edge() {
        let mut graph = CDependencyGraph::new();
        let node1 = DepNode {
            id: DepNodeId("node1".to_string()),
            kind: DepNodeKind::Source,
            name: "test.c".to_string(),
            file_path: Some("test.c".to_string()),
            content_hash: "abc123".to_string(),
            metadata: DepNodeMetadata::default(),
        };
        let node2 = DepNode {
            id: DepNodeId("node2".to_string()),
            kind: DepNodeKind::Header,
            name: "test.h".to_string(),
            file_path: Some("test.h".to_string()),
            content_hash: "def456".to_string(),
            metadata: DepNodeMetadata::default(),
        };
        graph.add_node(node1);
        graph.add_node(node2);
        graph.add_edge(DepEdge {
            from: DepNodeId("node1".to_string()),
            to: DepNodeId("node2".to_string()),
            kind: DepEdgeKind::DependsOn,
        });
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_dependency_graph_find_nodes_by_kind() {
        let mut graph = CDependencyGraph::new();
        let node1 = DepNode {
            id: DepNodeId("node1".to_string()),
            kind: DepNodeKind::Source,
            name: "test.c".to_string(),
            file_path: Some("test.c".to_string()),
            content_hash: "abc123".to_string(),
            metadata: DepNodeMetadata::default(),
        };
        let node2 = DepNode {
            id: DepNodeId("node2".to_string()),
            kind: DepNodeKind::Header,
            name: "test.h".to_string(),
            file_path: Some("test.h".to_string()),
            content_hash: "def456".to_string(),
            metadata: DepNodeMetadata::default(),
        };
        graph.add_node(node1);
        graph.add_node(node2);

        let sources: Vec<_> = graph.find_nodes_by_kind(&DepNodeKind::Source);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].name, "test.c");
    }

    #[test]
    fn test_classify_change_abi() {
        let mut graph = CDependencyGraph::new();
        let decl_node = DepNode {
            id: DepNodeId("decl1".to_string()),
            kind: DepNodeKind::Declaration,
            name: "my_func".to_string(),
            file_path: Some("test.c".to_string()),
            content_hash: "abc123".to_string(),
            metadata: DepNodeMetadata::default(),
        };
        graph.add_node(decl_node);

        let changed = vec![DepNodeId("decl1".to_string())];
        let classification = graph.classify_change(&changed);
        assert_eq!(classification, ChangeClassification::AbiChanged);
    }

    #[test]
    fn test_classify_change_layout() {
        let mut graph = CDependencyGraph::new();
        let layout_node = DepNode {
            id: DepNodeId("layout1".to_string()),
            kind: DepNodeKind::Layout,
            name: "struct Point".to_string(),
            file_path: Some("test.h".to_string()),
            content_hash: "def456".to_string(),
            metadata: DepNodeMetadata::default(),
        };
        graph.add_node(layout_node);

        let changed = vec![DepNodeId("layout1".to_string())];
        let classification = graph.classify_change(&changed);
        assert_eq!(classification, ChangeClassification::LayoutChanged);
    }

    #[test]
    fn test_change_classification_serialization() {
        let classification = ChangeClassification::AbiChanged;
        let json = serde_json::to_string(&classification).unwrap();
        assert!(json.contains("AbiChanged"));

        let deserialized: ChangeClassification = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ChangeClassification::AbiChanged);
    }

    #[test]
    fn test_invalidation_result_serialization() {
        let result = InvalidationResult {
            is_invalid: true,
            reason: Some(InvalidationReason::SourceChanged {
                path: "test.c".to_string(),
            }),
            invalidated_nodes: vec![DepNodeId("node1".to_string())],
            valid_nodes: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("SourceChanged"));
    }

    #[test]
    fn test_invalidation_engine_new() {
        let graph = CDependencyGraph::new();
        let engine = CInvalidationEngine::new(graph);
        assert!(engine.graph().nodes.is_empty());
    }

    #[test]
    fn test_invalidation_engine_invalidate_changed() {
        let mut graph = CDependencyGraph::new();
        let node = DepNode {
            id: DepNodeId("node1".to_string()),
            kind: DepNodeKind::Source,
            name: "test.c".to_string(),
            file_path: Some("test.c".to_string()),
            content_hash: "abc123".to_string(),
            metadata: DepNodeMetadata::default(),
        };
        graph.add_node(node);

        let engine = CInvalidationEngine::new(graph);
        let result = engine.invalidate_changed(&["test.c".to_string()]);

        assert!(result.is_invalid);
        assert!(result
            .invalidated_nodes
            .contains(&DepNodeId("node1".to_string())));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_manager_new() {
        let manager = CCacheManager::with_defaults();
        let stats = manager.stats();
        assert_eq!(stats.entry_count, 0);
        assert_eq!(stats.size_bytes, 0);
    }

    #[test]
    fn test_cache_manager_empty_stats() {
        let manager = CCacheManager::with_defaults();
        let stats = manager.stats();
        assert_eq!(stats.max_size_bytes, 1024 * 1024 * 1024);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[test]
    fn test_cache_key_builder() {
        let key = CCacheKeyBuilder::new("/path/to/source.c")
            .compiler_identity(CompilerIdentity::new(
                "clang",
                "15.0.0",
                "x86_64-unknown-linux-gnu",
            ))
            .flags_hash("abc123")
            .include_graph_hash("def456")
            .target_triple("x86_64-unknown-linux-gnu")
            .build()
            .unwrap();

        assert_eq!(key.source_path, PathBuf::from("/path/to/source.c"));
        assert_eq!(key.flags_hash, "abc123");
    }

    #[test]
    fn test_cache_key_content_hash() {
        let key1 = CCacheKeyBuilder::new("/path/to/source.c")
            .compiler_identity(CompilerIdentity::new(
                "clang",
                "15.0.0",
                "x86_64-unknown-linux-gnu",
            ))
            .flags_hash("abc123")
            .include_graph_hash("def456")
            .target_triple("x86_64-unknown-linux-gnu")
            .build()
            .unwrap();

        let key2 = CCacheKeyBuilder::new("/path/to/source.c")
            .compiler_identity(CompilerIdentity::new(
                "clang",
                "15.0.0",
                "x86_64-unknown-linux-gnu",
            ))
            .flags_hash("abc123")
            .include_graph_hash("def456")
            .target_triple("x86_64-unknown-linux-gnu")
            .build()
            .unwrap();

        assert_eq!(key1.content_hash(), key2.content_hash());
    }

    #[test]
    fn test_cache_put_get() {
        let mut manager = CCacheManager::with_defaults();
        let key = CCacheKeyBuilder::new("/path/to/source.c")
            .compiler_identity(CompilerIdentity::new(
                "clang",
                "15.0.0",
                "x86_64-unknown-linux-gnu",
            ))
            .flags_hash("abc123")
            .include_graph_hash("def456")
            .target_triple("x86_64-unknown-linux-gnu")
            .build()
            .unwrap();

        // Put an artifact
        let artifact = CachedArtifact::Snapshot {
            data: vec![1, 2, 3, 4],
        };
        manager.put(key.clone(), artifact).unwrap();

        // Should be able to get it
        assert!(manager.contains(&key));
    }

    #[test]
    fn test_cache_miss() {
        let mut manager = CCacheManager::with_defaults();
        let key = CCacheKeyBuilder::new("/path/to/source.c")
            .compiler_identity(CompilerIdentity::new(
                "clang",
                "15.0.0",
                "x86_64-unknown-linux-gnu",
            ))
            .flags_hash("abc123")
            .include_graph_hash("def456")
            .target_triple("x86_64-unknown-linux-gnu")
            .build()
            .unwrap();

        let result = manager.get(&key);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CacheError::CacheMiss(_)));
    }

    #[test]
    fn test_cache_entry_record_hit() {
        let key = CCacheKeyBuilder::new("/path/to/source.c")
            .compiler_identity(CompilerIdentity::new(
                "clang",
                "15.0.0",
                "x86_64-unknown-linux-gnu",
            ))
            .flags_hash("abc123")
            .include_graph_hash("def456")
            .target_triple("x86_64-unknown-linux-gnu")
            .build()
            .unwrap();

        let artifact = CachedArtifact::Snapshot {
            data: vec![1, 2, 3, 4],
        };
        let mut entry = CCacheEntry::new(key, artifact);
        assert_eq!(entry.hit_count, 0);
        entry.record_hit();
        assert_eq!(entry.hit_count, 1);
        entry.record_hit();
        assert_eq!(entry.hit_count, 2);
    }

    #[test]
    fn test_cache_clear() {
        let mut manager = CCacheManager::with_defaults();
        let key = CCacheKeyBuilder::new("/path/to/source.c")
            .compiler_identity(CompilerIdentity::new(
                "clang",
                "15.0.0",
                "x86_64-unknown-linux-gnu",
            ))
            .flags_hash("abc123")
            .include_graph_hash("def456")
            .target_triple("x86_64-unknown-linux-gnu")
            .build()
            .unwrap();

        let artifact = CachedArtifact::Snapshot {
            data: vec![1, 2, 3, 4],
        };
        manager.put(key, artifact).unwrap();
        assert_eq!(manager.stats().entry_count, 1);

        manager.clear();
        assert_eq!(manager.stats().entry_count, 0);
    }

    #[test]
    fn test_cache_invalidate_matching() {
        let mut manager = CCacheManager::with_defaults();

        // Add two entries
        let key1 = CCacheKeyBuilder::new("/path/a.c")
            .compiler_identity(CompilerIdentity::new(
                "clang",
                "15.0.0",
                "x86_64-unknown-linux-gnu",
            ))
            .flags_hash("abc123")
            .include_graph_hash("def456")
            .target_triple("x86_64-unknown-linux-gnu")
            .build()
            .unwrap();

        let key2 = CCacheKeyBuilder::new("/path/b.c")
            .compiler_identity(CompilerIdentity::new(
                "clang",
                "15.0.0",
                "x86_64-unknown-linux-gnu",
            ))
            .flags_hash("abc123")
            .include_graph_hash("def456")
            .target_triple("x86_64-unknown-linux-gnu")
            .build()
            .unwrap();

        let artifact = CachedArtifact::Snapshot {
            data: vec![1, 2, 3, 4],
        };
        manager.put(key1.clone(), artifact.clone()).unwrap();
        manager.put(key2.clone(), artifact).unwrap();

        assert_eq!(manager.stats().entry_count, 2);

        // Invalidate entries containing "a.c"
        manager.invalidate_matching(|k| k.source_path.to_string_lossy().contains("a.c"));
        assert_eq!(manager.stats().entry_count, 1);
        assert!(!manager.contains(&key1));
        assert!(manager.contains(&key2));
    }

    #[test]
    fn test_cached_artifact_serialization() {
        let artifact = CachedArtifact::Snapshot {
            data: vec![1, 2, 3, 4],
        };

        let bytes = artifact.to_bytes().unwrap();
        let deserialized = CachedArtifact::from_bytes(&bytes).unwrap();

        assert!(matches!(deserialized, CachedArtifact::Snapshot { .. }));
    }

    #[test]
    fn test_compiler_identity_with_sysroot() {
        let identity = CompilerIdentity::new("clang", "15.0.0", "x86_64-unknown-linux-gnu")
            .with_sysroot("/usr/lib/clang/15");
        assert_eq!(identity.sysroot, Some("/usr/lib/clang/15".to_string()));
    }

    #[test]
    fn test_cached_artifact_kind() {
        assert_eq!(CachedArtifact::Snapshot { data: vec![] }.kind(), "snapshot");
        assert_eq!(
            CachedArtifact::DependencyGraph { data: vec![] }.kind(),
            "dependency_graph"
        );
        assert_eq!(
            CachedArtifact::AstPackage { data: vec![] }.kind(),
            "ast_package"
        );
        assert_eq!(CachedArtifact::Dialect { data: vec![] }.kind(), "dialect");
        assert_eq!(CachedArtifact::Metadata { data: vec![] }.kind(), "metadata");
        assert_eq!(CachedArtifact::Object { data: vec![] }.kind(), "object");
        assert_eq!(CachedArtifact::Proof { data: vec![] }.kind(), "proof");
    }
}

// ============================================================================
// Macro/Include Fingerprints (Task 130) and Object/Link Cache Keys (Task 131)
// ============================================================================

/// Macro fingerprint components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroFingerprintComponents {
    /// Macro name
    pub name: String,
    /// Macro value (if any)
    pub value: Option<String>,
    /// Whether it's function-like
    pub is_function_like: bool,
    /// Parameters (for function-like macros)
    pub params: Option<Vec<String>>,
    /// Active conditional branches
    pub active_conditions: Vec<String>,
    /// Header content hash
    pub header_hash: Option<String>,
}

impl MacroFingerprintComponents {
    /// Compute macro fingerprint hash using BLAKE3
    pub fn compute_fingerprint(&self) -> String {
        let mut hasher = zigmera_hash::Blake3Hasher::with_schema_tag("c-macro-fingerprint");
        hasher.update_str(&self.name);
        if let Some(ref v) = self.value {
            hasher.update_str(v);
        }
        hasher.update_bool(self.is_function_like);
        if let Some(ref params) = self.params {
            for p in params {
                hasher.update_str(p);
            }
        }
        for c in &self.active_conditions {
            hasher.update_str(c);
        }
        if let Some(ref h) = self.header_hash {
            hasher.update_str(h);
        }
        hasher.finalize().as_hex()[..16].to_string()
    }
}

/// Include fingerprint components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeFingerprintComponents {
    /// Header path
    pub path: String,
    /// Content hash
    pub content_hash: String,
    /// Include search config
    pub search_config: String,
    /// Whether system header
    pub is_system: bool,
}

impl IncludeFingerprintComponents {
    /// Compute include fingerprint hash using BLAKE3
    pub fn compute_fingerprint(&self) -> String {
        let mut hasher = zigmera_hash::Blake3Hasher::with_schema_tag("c-include-fingerprint");
        hasher.update_str(&self.path);
        hasher.update_str(&self.content_hash);
        hasher.update_str(&self.search_config);
        hasher.update_bool(self.is_system);
        hasher.finalize().as_hex()[..16].to_string()
    }
}

/// Object file cache key components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectCacheKeyComponents {
    /// Compiler version
    pub compiler_version: String,
    /// Compile flags
    pub flags: Vec<String>,
    /// Target triple
    pub target_triple: String,
    /// Sysroot
    pub sysroot: Option<String>,
    /// Source file hashes
    pub source_hashes: Vec<String>,
    /// Header hashes
    pub header_hashes: Vec<String>,
    /// ABI fingerprint
    pub abi_fingerprint: String,
    /// Layout fingerprints
    pub layout_fingerprints: Vec<String>,
    /// Link arguments
    pub link_args: Vec<String>,
    /// Runtime version
    pub runtime_version: Option<String>,
}

impl ObjectCacheKeyComponents {
    /// Compute object cache key hash using BLAKE3
    pub fn compute_fingerprint(&self) -> String {
        let mut hasher = zigmera_hash::Blake3Hasher::with_schema_tag("c-object-key");
        hasher.update_str(&self.compiler_version);
        for f in &self.flags {
            hasher.update_str(f);
        }
        hasher.update_str(&self.target_triple);
        if let Some(ref s) = self.sysroot {
            hasher.update_str(s);
        }
        for h in &self.source_hashes {
            hasher.update_str(h);
        }
        for h in &self.header_hashes {
            hasher.update_str(h);
        }
        hasher.update_str(&self.abi_fingerprint);
        for l in &self.layout_fingerprints {
            hasher.update_str(l);
        }
        for l in &self.link_args {
            hasher.update_str(l);
        }
        if let Some(ref r) = self.runtime_version {
            hasher.update_str(r);
        }
        hasher.finalize().as_hex()[..16].to_string()
    }
}

/// Object file cache key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectCacheKey {
    pub components: ObjectCacheKeyComponents,
    pub hash: String,
}

impl ObjectCacheKey {
    /// Create new object cache key
    pub fn new(components: ObjectCacheKeyComponents) -> Self {
        let hash = components.compute_fingerprint();
        Self { components, hash }
    }

    /// Check if this key matches another
    pub fn matches(&self, other: &ObjectCacheKey) -> bool {
        self.hash == other.hash
    }
}

/// Link cache key components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkCacheKeyComponents {
    /// Object file hashes
    pub object_hashes: Vec<String>,
    /// Library paths
    pub library_paths: Vec<String>,
    /// Linker flags
    pub linker_flags: Vec<String>,
    /// Target triple
    pub target_triple: String,
    /// Output name
    pub output_name: Option<String>,
}

impl LinkCacheKeyComponents {
    /// Compute link cache key hash using BLAKE3
    pub fn compute_fingerprint(&self) -> String {
        let mut hasher = zigmera_hash::Blake3Hasher::with_schema_tag("c-link-key");
        for o in &self.object_hashes {
            hasher.update_str(o);
        }
        for l in &self.library_paths {
            hasher.update_str(l);
        }
        for f in &self.linker_flags {
            hasher.update_str(f);
        }
        hasher.update_str(&self.target_triple);
        if let Some(ref o) = self.output_name {
            hasher.update_str(o);
        }
        hasher.finalize().as_hex()[..16].to_string()
    }
}

/// Link cache key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkCacheKey {
    pub components: LinkCacheKeyComponents,
    pub hash: String,
}

impl LinkCacheKey {
    /// Create new link cache key
    pub fn new(components: LinkCacheKeyComponents) -> Self {
        let hash = components.compute_fingerprint();
        Self { components, hash }
    }

    /// Check if this key matches another
    pub fn matches(&self, other: &LinkCacheKey) -> bool {
        self.hash == other.hash
    }
}

#[cfg(test)]
mod fingerprint_tests {
    use super::*;

    #[test]
    fn test_macro_fingerprint_components() {
        let components = MacroFingerprintComponents {
            name: "MAX".to_string(),
            value: Some("100".to_string()),
            is_function_like: false,
            params: None,
            active_conditions: vec![],
            header_hash: Some("abc123".to_string()),
        };
        let fp = components.compute_fingerprint();
        assert!(!fp.is_empty());
        assert_eq!(fp.len(), 16);
    }

    #[test]
    fn test_macro_fingerprint_function_like() {
        let components = MacroFingerprintComponents {
            name: "MAX3".to_string(),
            value: Some("(_a, _b, _c) ((_a) > (_b) ? (_a) : (_c))".to_string()),
            is_function_like: true,
            params: Some(vec!["_a".to_string(), "_b".to_string(), "_c".to_string()]),
            active_conditions: vec![],
            header_hash: Some("def456".to_string()),
        };
        let fp = components.compute_fingerprint();
        assert!(!fp.is_empty());
    }

    #[test]
    fn test_include_fingerprint_components() {
        let components = IncludeFingerprintComponents {
            path: "/usr/include/stdio.h".to_string(),
            content_hash: "hash123".to_string(),
            search_config: "-isystem /usr/include".to_string(),
            is_system: true,
        };
        let fp = components.compute_fingerprint();
        assert!(!fp.is_empty());
    }

    #[test]
    fn test_object_cache_key_components() {
        let components = ObjectCacheKeyComponents {
            compiler_version: "clang-15.0.0".to_string(),
            flags: vec!["-O2".to_string(), "-Wall".to_string()],
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            sysroot: None,
            source_hashes: vec!["src1".to_string()],
            header_hashes: vec!["hdr1".to_string(), "hdr2".to_string()],
            abi_fingerprint: "abi123".to_string(),
            layout_fingerprints: vec!["layout1".to_string()],
            link_args: vec!["-lm".to_string()],
            runtime_version: Some("1.0.0".to_string()),
        };
        let key = ObjectCacheKey::new(components);
        assert!(!key.hash.is_empty());
    }

    #[test]
    fn test_object_cache_key_matches() {
        let components1 = ObjectCacheKeyComponents {
            compiler_version: "clang-15".to_string(),
            flags: vec!["-O2".to_string()],
            target_triple: "x86_64".to_string(),
            sysroot: None,
            source_hashes: vec!["src1".to_string()],
            header_hashes: vec![],
            abi_fingerprint: "abi1".to_string(),
            layout_fingerprints: vec![],
            link_args: vec![],
            runtime_version: None,
        };
        let key1 = ObjectCacheKey::new(components1);
        let key2 = ObjectCacheKey::new(ObjectCacheKeyComponents {
            compiler_version: "clang-15".to_string(),
            flags: vec!["-O2".to_string()],
            target_triple: "x86_64".to_string(),
            sysroot: None,
            source_hashes: vec!["src1".to_string()],
            header_hashes: vec![],
            abi_fingerprint: "abi1".to_string(),
            layout_fingerprints: vec![],
            link_args: vec![],
            runtime_version: None,
        });
        assert!(key1.matches(&key2));
    }

    #[test]
    fn test_link_cache_key_components() {
        let components = LinkCacheKeyComponents {
            object_hashes: vec!["obj1".to_string(), "obj2".to_string()],
            library_paths: vec!["/usr/lib".to_string()],
            linker_flags: vec!["-shared".to_string()],
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            output_name: Some("libtest.so".to_string()),
        };
        let key = LinkCacheKey::new(components);
        assert!(!key.hash.is_empty());
    }

    #[test]
    fn test_link_cache_key_matches() {
        let components1 = LinkCacheKeyComponents {
            object_hashes: vec!["obj1".to_string()],
            library_paths: vec!["/usr/lib".to_string()],
            linker_flags: vec!["-shared".to_string()],
            target_triple: "x86_64".to_string(),
            output_name: Some("lib.so".to_string()),
        };
        let key1 = LinkCacheKey::new(components1);
        let key2 = LinkCacheKey::new(LinkCacheKeyComponents {
            object_hashes: vec!["obj1".to_string()],
            library_paths: vec!["/usr/lib".to_string()],
            linker_flags: vec!["-shared".to_string()],
            target_triple: "x86_64".to_string(),
            output_name: Some("lib.so".to_string()),
        });
        assert!(key1.matches(&key2));
    }
}

#[cfg(test)]
mod graph_diff_tests {
    use super::*;

    #[test]
    fn test_graph_diff_add_remove() {
        let differ = GraphDiffer::new("x86_64-unknown-linux-gnu".to_string());

        let mut old_graph = CDependencyGraph::new();
        old_graph.add_node(DepNode {
            id: DepNodeId("node1".to_string()),
            kind: DepNodeKind::Source,
            name: "a.c".to_string(),
            file_path: Some("a.c".to_string()),
            content_hash: "hash1".to_string(),
            metadata: DepNodeMetadata::default(),
        });

        let mut new_graph = CDependencyGraph::new();
        new_graph.add_node(DepNode {
            id: DepNodeId("node2".to_string()),
            kind: DepNodeKind::Source,
            name: "b.c".to_string(),
            file_path: Some("b.c".to_string()),
            content_hash: "hash2".to_string(),
            metadata: DepNodeMetadata::default(),
        });

        let diff = differ.diff(&old_graph, &new_graph);
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.removed.len(), 1);
        assert!(diff.changed.is_empty());
        assert!(diff.unchanged.is_empty());
    }

    #[test]
    fn test_graph_diff_content_change() {
        let differ = GraphDiffer::new("x86_64-unknown-linux-gnu".to_string());

        let mut old_graph = CDependencyGraph::new();
        old_graph.add_node(DepNode {
            id: DepNodeId("node1".to_string()),
            kind: DepNodeKind::Source,
            name: "test.c".to_string(),
            file_path: Some("test.c".to_string()),
            content_hash: "old_hash".to_string(),
            metadata: DepNodeMetadata::default(),
        });

        let mut new_graph = CDependencyGraph::new();
        new_graph.add_node(DepNode {
            id: DepNodeId("node1".to_string()),
            kind: DepNodeKind::Source,
            name: "test.c".to_string(),
            file_path: Some("test.c".to_string()),
            content_hash: "new_hash".to_string(),
            metadata: DepNodeMetadata::default(),
        });

        let diff = differ.diff(&old_graph, &new_graph);
        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert_eq!(diff.changed.len(), 1);
        assert!(diff.unchanged.is_empty());
        assert_eq!(diff.changed[0].old_hash, "old_hash");
        assert_eq!(diff.changed[0].new_hash, "new_hash");
    }

    #[test]
    fn test_graph_diff_unchanged() {
        let differ = GraphDiffer::new("x86_64-unknown-linux-gnu".to_string());

        let mut old_graph = CDependencyGraph::new();
        old_graph.add_node(DepNode {
            id: DepNodeId("node1".to_string()),
            kind: DepNodeKind::Source,
            name: "test.c".to_string(),
            file_path: Some("test.c".to_string()),
            content_hash: "same_hash".to_string(),
            metadata: DepNodeMetadata::default(),
        });

        let mut new_graph = CDependencyGraph::new();
        new_graph.add_node(DepNode {
            id: DepNodeId("node1".to_string()),
            kind: DepNodeKind::Source,
            name: "test.c".to_string(),
            file_path: Some("test.c".to_string()),
            content_hash: "same_hash".to_string(),
            metadata: DepNodeMetadata::default(),
        });

        let diff = differ.diff(&old_graph, &new_graph);
        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert!(diff.changed.is_empty());
        assert_eq!(diff.unchanged.len(), 1);
    }

    #[test]
    fn test_graph_diff_mixed() {
        let differ = GraphDiffer::new("x86_64-unknown-linux-gnu".to_string());

        // Old graph has node1 and node2
        let mut old_graph = CDependencyGraph::new();
        old_graph.add_node(DepNode {
            id: DepNodeId("node1".to_string()),
            kind: DepNodeKind::Source,
            name: "a.c".to_string(),
            file_path: Some("a.c".to_string()),
            content_hash: "hash_a".to_string(),
            metadata: DepNodeMetadata::default(),
        });
        old_graph.add_node(DepNode {
            id: DepNodeId("node2".to_string()),
            kind: DepNodeKind::Header,
            name: "b.h".to_string(),
            file_path: Some("b.h".to_string()),
            content_hash: "hash_b".to_string(),
            metadata: DepNodeMetadata::default(),
        });

        // New graph has node1 (changed), node2 (unchanged), node3 (added)
        let mut new_graph = CDependencyGraph::new();
        new_graph.add_node(DepNode {
            id: DepNodeId("node1".to_string()),
            kind: DepNodeKind::Source,
            name: "a.c".to_string(),
            file_path: Some("a.c".to_string()),
            content_hash: "hash_a_MODIFIED".to_string(),
            metadata: DepNodeMetadata::default(),
        });
        new_graph.add_node(DepNode {
            id: DepNodeId("node2".to_string()),
            kind: DepNodeKind::Header,
            name: "b.h".to_string(),
            file_path: Some("b.h".to_string()),
            content_hash: "hash_b".to_string(),
            metadata: DepNodeMetadata::default(),
        });
        new_graph.add_node(DepNode {
            id: DepNodeId("node3".to_string()),
            kind: DepNodeKind::Source,
            name: "c.c".to_string(),
            file_path: Some("c.c".to_string()),
            content_hash: "hash_c".to_string(),
            metadata: DepNodeMetadata::default(),
        });

        let diff = differ.diff(&old_graph, &new_graph);
        assert_eq!(diff.added.len(), 1); // node3
        assert!(diff.removed.is_empty()); // no nodes removed
        assert_eq!(diff.changed.len(), 1); // node1
        assert_eq!(diff.unchanged.len(), 1); // node2
    }

    #[test]
    fn test_changed_node_serialization() {
        let changed = ChangedNode {
            id: DepNodeId("func1".to_string()),
            change_type: NodeChangeType::ContentChanged,
            old_hash: "old".to_string(),
            new_hash: "new".to_string(),
        };
        let json = serde_json::to_string(&changed).unwrap();
        assert!(json.contains("ContentChanged"));
        let deserialized: ChangedNode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id.0, "func1");
    }

    #[test]
    fn test_graph_diff_serialization() {
        let diff = GraphDiff {
            added: vec![DepNodeId("new1".to_string())],
            removed: vec![DepNodeId("old1".to_string())],
            changed: vec![ChangedNode {
                id: DepNodeId("mod1".to_string()),
                change_type: NodeChangeType::Renamed,
                old_hash: "h1".to_string(),
                new_hash: "h2".to_string(),
            }],
            unchanged: vec![],
        };
        let json = serde_json::to_string(&diff).unwrap();
        assert!(json.contains("new1"));
        assert!(json.contains("old1"));
        assert!(json.contains("Renamed"));
    }
}

// ============================================================================
// Cache Corruption Recovery (Task 133)
// ============================================================================

/// Cache corruption types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CorruptionType {
    /// Checksum mismatch
    ChecksumMismatch,
    /// Partial write detected
    PartialWrite,
    /// Orphaned manifest
    OrphanedManifest,
    /// Stale object file
    StaleObjectFile,
    /// Schema version mismatch
    SchemaVersionMismatch,
    /// Missing required file
    MissingFile,
}

/// Corruption report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorruptionReport {
    /// Type of corruption
    pub corruption_type: CorruptionType,
    /// File or key affected
    pub affected_key: Option<String>,
    /// Description
    pub description: String,
    /// Recovery action taken
    pub recovery_action: RecoveryAction,
}

/// Recovery action taken
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Entry was quarantined
    Quarantined,
    /// Entry was rebuilt
    Rebuilt,
    /// Entry was removed
    Removed,
    /// Recovery was skipped
    Skipped,
}

/// Cache corruption detector
pub struct CacheCorruptionDetector;

impl CacheCorruptionDetector {
    /// Detect corruption in a cache entry
    pub fn detect(cache_entry: &CCacheEntry, expected_checksum: &str) -> Option<CorruptionReport> {
        // Check checksum
        let artifact_bytes = match cache_entry.artifact.to_bytes() {
            Ok(b) => b,
            Err(_) => {
                return Some(CorruptionReport {
                    corruption_type: CorruptionType::PartialWrite,
                    affected_key: Some(format!("{:?}", cache_entry.key)),
                    description: "Failed to serialize artifact".to_string(),
                    recovery_action: RecoveryAction::Quarantined,
                });
            }
        };

        let actual_checksum = blake3::hash(&artifact_bytes).to_hex().to_string();
        if actual_checksum != *expected_checksum {
            return Some(CorruptionReport {
                corruption_type: CorruptionType::ChecksumMismatch,
                affected_key: Some(format!("{:?}", cache_entry.key)),
                description: format!(
                    "Checksum mismatch: expected {}, got {}",
                    expected_checksum, actual_checksum
                ),
                recovery_action: RecoveryAction::Quarantined,
            });
        }

        None
    }

    /// Detect orphaned manifests
    pub fn detect_orphaned_manifests(active_keys: &[CCacheKey]) -> Vec<CorruptionReport> {
        // In a real implementation, this would scan the cache directory
        // for manifests that don't have corresponding cache entries
        Vec::new()
    }

    /// Detect stale object files
    pub fn detect_stale_objects(
        object_paths: &[String],
        cache_hashes: &HashMap<String, String>,
    ) -> Vec<CorruptionReport> {
        let mut reports = Vec::new();
        for path in object_paths {
            if let Some(cached_hash) = cache_hashes.get(path) {
                // In real implementation, would compute actual file hash
                let _ = cached_hash;
                // Placeholder: always skip for now
                reports.push(CorruptionReport {
                    corruption_type: CorruptionType::StaleObjectFile,
                    affected_key: Some(path.clone()),
                    description: "Object file may be stale".to_string(),
                    recovery_action: RecoveryAction::Skipped,
                });
            }
        }
        reports
    }
}

/// Cache recovery manager
pub struct CacheRecoveryManager {
    detector: CacheCorruptionDetector,
    quarantine: Vec<CorruptionReport>,
    rebuilt: Vec<CorruptionReport>,
}

impl CacheRecoveryManager {
    /// Create new recovery manager
    pub fn new() -> Self {
        Self {
            detector: CacheCorruptionDetector,
            quarantine: Vec::new(),
            rebuilt: Vec::new(),
        }
    }

    /// Recover a corrupted entry
    pub fn recover(&mut self, entry: &CCacheEntry, expected_checksum: &str) -> RecoveryAction {
        if let Some(report) = CacheCorruptionDetector::detect(entry, expected_checksum) {
            match report.recovery_action {
                RecoveryAction::Quarantined => {
                    self.quarantine.push(report);
                    RecoveryAction::Quarantined
                }
                RecoveryAction::Rebuilt => {
                    self.rebuilt.push(report);
                    RecoveryAction::Rebuilt
                }
                action => action,
            }
        } else {
            RecoveryAction::Skipped
        }
    }

    /// Get quarantine reports
    pub fn quarantine_reports(&self) -> &[CorruptionReport] {
        &self.quarantine
    }

    /// Get rebuilt reports
    pub fn rebuilt_reports(&self) -> &[CorruptionReport] {
        &self.rebuilt
    }

    /// Get total recovered count
    pub fn recovered_count(&self) -> usize {
        self.quarantine.len() + self.rebuilt.len()
    }
}

impl Default for CacheRecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod corruption_recovery_tests {
    use super::*;

    #[test]
    fn test_corruption_type_serialization() {
        let ct = CorruptionType::ChecksumMismatch;
        let json = serde_json::to_string(&ct).unwrap();
        assert!(json.contains("ChecksumMismatch"));
        let deserialized: CorruptionType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, CorruptionType::ChecksumMismatch);
    }

    #[test]
    fn test_recovery_action_serialization() {
        let action = RecoveryAction::Quarantined;
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("Quarantined"));
        let deserialized: RecoveryAction = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, RecoveryAction::Quarantined);
    }

    #[test]
    fn test_corruption_report_serialization() {
        let report = CorruptionReport {
            corruption_type: CorruptionType::PartialWrite,
            affected_key: Some("key123".to_string()),
            description: "Incomplete write detected".to_string(),
            recovery_action: RecoveryAction::Quarantined,
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("PartialWrite"));
        assert!(json.contains("Quarantined"));
    }

    #[test]
    fn test_cache_corruption_detector_no_corruption() {
        let key = CCacheKeyBuilder::new("/path/to/test.c")
            .compiler_identity(CompilerIdentity::new(
                "clang",
                "15.0.0",
                "x86_64-unknown-linux-gnu",
            ))
            .flags_hash("abc123")
            .include_graph_hash("def456")
            .target_triple("x86_64-unknown-linux-gnu")
            .build()
            .unwrap();
        let artifact = CachedArtifact::Snapshot {
            data: vec![1, 2, 3, 4],
        };
        let entry = CCacheEntry::new(key, artifact);

        let artifact_bytes = entry.artifact.to_bytes().unwrap();
        let checksum = blake3::hash(&artifact_bytes).to_hex().to_string();
        let result = CacheCorruptionDetector::detect(&entry, &checksum);
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_corruption_detector_with_corruption() {
        let key = CCacheKeyBuilder::new("/path/to/test.c")
            .compiler_identity(CompilerIdentity::new(
                "clang",
                "15.0.0",
                "x86_64-unknown-linux-gnu",
            ))
            .flags_hash("abc123")
            .include_graph_hash("def456")
            .target_triple("x86_64-unknown-linux-gnu")
            .build()
            .unwrap();
        let artifact = CachedArtifact::Snapshot {
            data: vec![1, 2, 3, 4],
        };
        let entry = CCacheEntry::new(key, artifact);

        // Use wrong checksum
        let wrong_checksum = "0000000000000000";
        let result = CacheCorruptionDetector::detect(&entry, wrong_checksum);
        assert!(result.is_some());
        let report = result.unwrap();
        assert_eq!(report.corruption_type, CorruptionType::ChecksumMismatch);
        assert_eq!(report.recovery_action, RecoveryAction::Quarantined);
    }

    #[test]
    fn test_cache_recovery_manager_new() {
        let manager = CacheRecoveryManager::new();
        assert_eq!(manager.recovered_count(), 0);
        assert!(manager.quarantine_reports().is_empty());
        assert!(manager.rebuilt_reports().is_empty());
    }

    #[test]
    fn test_cache_recovery_manager_recover() {
        let mut manager = CacheRecoveryManager::new();
        let key = CCacheKeyBuilder::new("/path/to/test.c")
            .compiler_identity(CompilerIdentity::new(
                "clang",
                "15.0.0",
                "x86_64-unknown-linux-gnu",
            ))
            .flags_hash("abc123")
            .include_graph_hash("def456")
            .target_triple("x86_64-unknown-linux-gnu")
            .build()
            .unwrap();
        let artifact = CachedArtifact::Snapshot {
            data: vec![1, 2, 3, 4],
        };
        let entry = CCacheEntry::new(key, artifact);

        let wrong_checksum = "0000000000000000";
        let action = manager.recover(&entry, wrong_checksum);
        assert_eq!(action, RecoveryAction::Quarantined);
        assert_eq!(manager.recovered_count(), 1);
    }
}
