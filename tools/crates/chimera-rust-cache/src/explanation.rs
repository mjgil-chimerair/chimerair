//! Rust cache explanation for Task 150
//! Provides cache hit/miss/rebuild explanation with key components

use chimera_rust_schema::{DepNodeKind, RdepGraph};
use serde::{Deserialize, Serialize};

/// Cache explanation status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustCacheExplainStatus {
    Hit,
    Miss,
    Rebuild,
}

/// Cache explanation reason
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RustCacheExplainReason {
    CacheHit,
    NoEntry,
    InvalidatedEntry,
    DependencyChanged {
        dependency_kind: String,
        dependency_id: String,
    },
    SchemaMismatch,
    RustcVersionMismatch,
    TargetMismatch,
    FingerprintChanged {
        node_kind: String,
        node_stable_id: String,
    },
}

/// Cache explanation for Rust artifacts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustCacheExplanation {
    pub artifact_kind: String,
    pub cache_key: String,
    pub status: RustCacheExplainStatus,
    pub reason: RustCacheExplainReason,
    pub key_components: RustCacheKeyComponents,
    pub reuse_checks: RustCacheReuseChecks,
}

/// Components that make up a Rust cache key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustCacheKeyComponents {
    pub schema_version: u32,
    pub rustc_version: String,
    pub target_triple: String,
    pub fingerprint: String,
}

/// Reuse checks for cache explanation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustCacheReuseChecks {
    pub cached_entry_valid: bool,
    pub schema_matches: bool,
    pub rustc_version_matches: bool,
    pub target_matches: bool,
    pub fingerprint_matches: bool,
    pub dependency_fingerprints: Vec<RustDependencyFingerprint>,
}

/// Dependency fingerprint entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustDependencyFingerprint {
    pub kind: String,
    pub stable_id: String,
    pub content_hash: String,
}

/// Graph mutation classification for diffing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphMutationKind {
    Added,
    Removed,
    Changed,
    Unchanged,
    ABIChanged,
    LayoutChanged,
    BodyOnlyChanged,
}

/// Result of graph diffing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphDiffResult {
    pub mutations: Vec<GraphMutation>,
    pub total_added: usize,
    pub total_removed: usize,
    pub total_changed: usize,
}

/// A single mutation in the graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphMutation {
    pub kind: GraphMutationKind,
    pub node_kind: String,
    pub stable_id: String,
    pub old_fingerprint: Option<String>,
    pub new_fingerprint: Option<String>,
}

impl RustCacheExplanation {
    /// Create a cache hit explanation
    pub fn hit(artifact_kind: &str, cache_key: &str, components: RustCacheKeyComponents) -> Self {
        Self {
            artifact_kind: artifact_kind.to_string(),
            cache_key: cache_key.to_string(),
            status: RustCacheExplainStatus::Hit,
            reason: RustCacheExplainReason::CacheHit,
            key_components: components,
            reuse_checks: RustCacheReuseChecks {
                cached_entry_valid: true,
                schema_matches: true,
                rustc_version_matches: true,
                target_matches: true,
                fingerprint_matches: true,
                dependency_fingerprints: vec![],
            },
        }
    }

    /// Create a cache miss explanation
    pub fn miss(artifact_kind: &str, cache_key: &str, components: RustCacheKeyComponents) -> Self {
        Self {
            artifact_kind: artifact_kind.to_string(),
            cache_key: cache_key.to_string(),
            status: RustCacheExplainStatus::Miss,
            reason: RustCacheExplainReason::NoEntry,
            key_components: components,
            reuse_checks: RustCacheReuseChecks {
                cached_entry_valid: false,
                schema_matches: true,
                rustc_version_matches: true,
                target_matches: true,
                fingerprint_matches: false,
                dependency_fingerprints: vec![],
            },
        }
    }

    /// Create a rebuild explanation due to dependency change
    pub fn dependency_changed(
        artifact_kind: &str,
        cache_key: &str,
        components: RustCacheKeyComponents,
        dep_kind: &str,
        dep_id: &str,
    ) -> Self {
        Self {
            artifact_kind: artifact_kind.to_string(),
            cache_key: cache_key.to_string(),
            status: RustCacheExplainStatus::Rebuild,
            reason: RustCacheExplainReason::DependencyChanged {
                dependency_kind: dep_kind.to_string(),
                dependency_id: dep_id.to_string(),
            },
            key_components: components,
            reuse_checks: RustCacheReuseChecks {
                cached_entry_valid: false,
                schema_matches: true,
                rustc_version_matches: true,
                target_matches: true,
                fingerprint_matches: false,
                dependency_fingerprints: vec![],
            },
        }
    }
}

/// Compare two RdepGraphs and classify mutations
pub fn diff_graphs(old: &RdepGraph, new: &RdepGraph) -> GraphDiffResult {
    use std::collections::HashMap;

    let mut mutations = Vec::new();
    let mut total_added = 0;
    let mut total_removed = 0;
    let mut total_changed = 0;

    // Build maps for efficient lookup
    let old_nodes: HashMap<&str, _> = old
        .nodes
        .iter()
        .map(|n| (n.stable_id.as_str(), n))
        .collect();
    let new_nodes: HashMap<&str, _> = new
        .nodes
        .iter()
        .map(|n| (n.stable_id.as_str(), n))
        .collect();

    // Find removed and changed nodes
    for (stable_id, old_node) in &old_nodes {
        if let Some(new_node) = new_nodes.get(stable_id) {
            if old_node.fingerprint != new_node.fingerprint {
                let mutation_kind = if is_abi_change(&old_node.kind, &new_node.kind) {
                    GraphMutationKind::ABIChanged
                } else if is_layout_change(&old_node.kind, &new_node.kind) {
                    GraphMutationKind::LayoutChanged
                } else if is_body_only_change(&old_node.kind, &new_node.kind) {
                    GraphMutationKind::BodyOnlyChanged
                } else {
                    GraphMutationKind::Changed
                };
                mutations.push(GraphMutation {
                    kind: mutation_kind,
                    node_kind: format!("{:?}", old_node.kind),
                    stable_id: stable_id.to_string(),
                    old_fingerprint: Some(old_node.fingerprint.clone()),
                    new_fingerprint: Some(new_node.fingerprint.clone()),
                });
                total_changed += 1;
            }
        } else {
            mutations.push(GraphMutation {
                kind: GraphMutationKind::Removed,
                node_kind: format!("{:?}", old_node.kind),
                stable_id: stable_id.to_string(),
                old_fingerprint: Some(old_node.fingerprint.clone()),
                new_fingerprint: None,
            });
            total_removed += 1;
        }
    }

    // Find added nodes
    for (stable_id, new_node) in &new_nodes {
        if !old_nodes.contains_key(stable_id) {
            mutations.push(GraphMutation {
                kind: GraphMutationKind::Added,
                node_kind: format!("{:?}", new_node.kind),
                stable_id: stable_id.to_string(),
                old_fingerprint: None,
                new_fingerprint: Some(new_node.fingerprint.clone()),
            });
            total_added += 1;
        }
    }

    GraphDiffResult {
        mutations,
        total_added,
        total_removed,
        total_changed,
    }
}

/// Check if this is an ABI-affecting change
fn is_abi_change(old_kind: &DepNodeKind, new_kind: &DepNodeKind) -> bool {
    matches!(old_kind, DepNodeKind::Item | DepNodeKind::Export)
        || matches!(new_kind, DepNodeKind::Item | DepNodeKind::Export)
}

/// Check if this is a layout-affecting change
fn is_layout_change(old_kind: &DepNodeKind, new_kind: &DepNodeKind) -> bool {
    matches!(old_kind, DepNodeKind::Layout | DepNodeKind::Type)
        || matches!(new_kind, DepNodeKind::Layout | DepNodeKind::Type)
}

/// Check if this is a body-only change (MIR body changed but not ABI/layout)
fn is_body_only_change(old_kind: &DepNodeKind, new_kind: &DepNodeKind) -> bool {
    matches!(old_kind, DepNodeKind::MirBody) && matches!(new_kind, DepNodeKind::MirBody)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_rust_schema::{ArtifactHeader, DepEdge, DepEdgeKind, DepNode, DepNodeId};

    fn make_test_graph(target: &str) -> RdepGraph {
        RdepGraph {
            header: ArtifactHeader::new(target, "0.1.0"),
            checksum: String::new(),
            nodes: vec![
                DepNode {
                    id: DepNodeId(0),
                    kind: DepNodeKind::Source,
                    fingerprint: "src_fp".to_string(),
                    stable_id: "src0".to_string(),
                },
                DepNode {
                    id: DepNodeId(1),
                    kind: DepNodeKind::Item,
                    fingerprint: "item_fp".to_string(),
                    stable_id: "item0".to_string(),
                },
                DepNode {
                    id: DepNodeId(2),
                    kind: DepNodeKind::Export,
                    fingerprint: "export_fp".to_string(),
                    stable_id: "export0".to_string(),
                },
            ],
            edges: vec![
                DepEdge {
                    from: DepNodeId(0),
                    to: DepNodeId(1),
                    kind: DepEdgeKind::DependsOn,
                },
                DepEdge {
                    from: DepNodeId(1),
                    to: DepNodeId(2),
                    kind: DepEdgeKind::Provides,
                },
            ],
        }
    }

    #[test]
    fn test_cache_explanation_hit() {
        let components = RustCacheKeyComponents {
            schema_version: 1,
            rustc_version: "1.75.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            fingerprint: "abc123".to_string(),
        };
        let explanation =
            RustCacheExplanation::hit("rsnap", "rust/rsnap/1/xxx", components.clone());
        assert!(matches!(explanation.status, RustCacheExplainStatus::Hit));
        assert!(matches!(
            explanation.reason,
            RustCacheExplainReason::CacheHit
        ));
        assert!(explanation.reuse_checks.cached_entry_valid);
    }

    #[test]
    fn test_cache_explanation_miss() {
        let components = RustCacheKeyComponents {
            schema_version: 1,
            rustc_version: "1.75.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            fingerprint: "abc123".to_string(),
        };
        let explanation =
            RustCacheExplanation::miss("rsnap", "rust/rsnap/1/xxx", components.clone());
        assert!(matches!(explanation.status, RustCacheExplainStatus::Miss));
        assert!(matches!(
            explanation.reason,
            RustCacheExplainReason::NoEntry
        ));
        assert!(!explanation.reuse_checks.cached_entry_valid);
    }

    #[test]
    fn test_cache_explanation_dependency_changed() {
        let components = RustCacheKeyComponents {
            schema_version: 1,
            rustc_version: "1.75.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            fingerprint: "abc123".to_string(),
        };
        let explanation = RustCacheExplanation::dependency_changed(
            "rsnap",
            "rust/rsnap/1/xxx",
            components,
            "layout",
            "my_struct",
        );
        assert!(matches!(
            explanation.status,
            RustCacheExplainStatus::Rebuild
        ));
        match &explanation.reason {
            RustCacheExplainReason::DependencyChanged {
                dependency_kind,
                dependency_id,
            } => {
                assert_eq!(dependency_kind, "layout");
                assert_eq!(dependency_id, "my_struct");
            }
            _ => panic!("expected DependencyChanged"),
        }
    }

    #[test]
    fn test_graph_diff_no_changes() {
        let graph1 = make_test_graph("x86_64-unknown-linux-gnu");
        let graph2 = make_test_graph("x86_64-unknown-linux-gnu");

        let result = diff_graphs(&graph1, &graph2);
        assert_eq!(result.total_added, 0);
        assert_eq!(result.total_removed, 0);
        assert_eq!(result.total_changed, 0);
    }

    #[test]
    fn test_graph_diff_node_changed() {
        let mut graph1 = make_test_graph("x86_64-unknown-linux-gnu");
        let mut graph2 = make_test_graph("x86_64-unknown-linux-gnu");

        // Change fingerprint of item node (Item affects ABI)
        graph2.nodes[1].fingerprint = "new_item_fp".to_string();

        let result = diff_graphs(&graph1, &graph2);
        assert_eq!(result.total_changed, 1);
        assert!(result
            .mutations
            .iter()
            .any(|m| m.kind == GraphMutationKind::ABIChanged));
    }

    #[test]
    fn test_graph_diff_node_removed() {
        let mut graph1 = make_test_graph("x86_64-unknown-linux-gnu");
        let graph2 = RdepGraph {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            nodes: vec![DepNode {
                id: DepNodeId(0),
                kind: DepNodeKind::Source,
                fingerprint: "src_fp".to_string(),
                stable_id: "src0".to_string(),
            }],
            edges: vec![],
        };

        let result = diff_graphs(&graph1, &graph2);
        assert_eq!(result.total_removed, 2);
        assert!(result
            .mutations
            .iter()
            .any(|m| m.kind == GraphMutationKind::Removed));
    }

    #[test]
    fn test_graph_diff_node_added() {
        let graph1 = RdepGraph {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            nodes: vec![DepNode {
                id: DepNodeId(0),
                kind: DepNodeKind::Source,
                fingerprint: "src_fp".to_string(),
                stable_id: "src0".to_string(),
            }],
            edges: vec![],
        };
        let mut graph2 = make_test_graph("x86_64-unknown-linux-gnu");

        let result = diff_graphs(&graph1, &graph2);
        assert_eq!(result.total_added, 2);
        assert!(result
            .mutations
            .iter()
            .any(|m| m.kind == GraphMutationKind::Added));
    }
}
