//! C Invalidation Classification for chimera-c-cache
//!
//! Implements the classification of C source changes into invalidation kinds
//! and the rules for object/wrapper/proof/link artifact reuse.
//!
//! PR 4: Finalize C invalidation classifications
//! - implementation-only vs header/macro/layout/ABI change rules
//! - object/wrapper/proof/link reuse rules

use crate::envelope::CArtifactKind;
use crate::{CDependencyGraph, CachedArtifact, DepEdgeKind, DepNode, DepNodeId, DepNodeKind};

/// Classification of what kind of change occurred
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeClassification {
    /// Only implementation bodies changed (no downstream effect)
    ImplementationOnly,
    /// Header surface changed (affects consumers)
    HeaderSurface,
    /// Macro or conditional compilation changed
    MacroCondition,
    /// Layout changed (affects ABI)
    Layout,
    /// Compiler, target, or sysroot changed
    CompilerTarget,
    /// Generated header changed
    GeneratedHeader,
    /// Unknown change
    Unknown,
}

impl ChangeClassification {
    /// Convert to the envelope's CInvalidationKind
    pub fn to_invalidation_kind(&self) -> crate::envelope::CInvalidationKind {
        use crate::envelope::CInvalidationKind;
        match self {
            ChangeClassification::ImplementationOnly => CInvalidationKind::ImplementationOnly,
            ChangeClassification::HeaderSurface => CInvalidationKind::HeaderSurface,
            ChangeClassification::MacroCondition => CInvalidationKind::MacroCondition,
            ChangeClassification::Layout => CInvalidationKind::Layout,
            ChangeClassification::CompilerTarget => CInvalidationKind::CompilerTarget,
            ChangeClassification::GeneratedHeader => CInvalidationKind::GeneratedHeader,
            ChangeClassification::Unknown => CInvalidationKind::ImplementationOnly,
        }
    }

    /// Check if this change affects downstream consumers
    pub fn affects_downstream(&self) -> bool {
        matches!(
            self,
            ChangeClassification::HeaderSurface
                | ChangeClassification::Layout
                | ChangeClassification::CompilerTarget
                | ChangeClassification::GeneratedHeader
        )
    }

    /// Check if this is a cosmetic change requiring no rebuild
    pub fn is_cosmetic(&self) -> bool {
        matches!(
            self,
            ChangeClassification::ImplementationOnly | ChangeClassification::Unknown
        )
    }
}

/// Classifier for determining what kind of invalidation occurred
pub struct InvalidationClassifier<'a> {
    graph: &'a CDependencyGraph,
}

impl<'a> InvalidationClassifier<'a> {
    /// Create a new classifier with the dependency graph
    pub fn new(graph: &'a CDependencyGraph) -> Self {
        Self { graph }
    }

    /// Classify a set of changed node IDs into a change classification
    pub fn classify(&self, changed_ids: &[DepNodeId]) -> ChangeClassification {
        if changed_ids.is_empty() {
            return ChangeClassification::Unknown;
        }

        let mut has_abi = false;
        let mut has_layout = false;
        let mut has_header = false;
        let mut has_macro = false;
        let mut has_source_only = true;

        for id in changed_ids {
            if let Some(node) = self.graph.get_node(id) {
                let classification = self.classify_node(node);

                match classification {
                    ChangeClassification::Layout => {
                        has_layout = true;
                        has_abi = true;
                        has_source_only = false;
                    }
                    ChangeClassification::HeaderSurface => {
                        has_header = true;
                        has_source_only = false;
                    }
                    ChangeClassification::MacroCondition => {
                        has_macro = true;
                        has_source_only = false;
                    }
                    ChangeClassification::ImplementationOnly => {
                        // Keep source_only true
                    }
                    ChangeClassification::CompilerTarget => {
                        return ChangeClassification::CompilerTarget;
                    }
                    ChangeClassification::GeneratedHeader => {
                        return ChangeClassification::GeneratedHeader;
                    }
                    ChangeClassification::Unknown => {
                        has_source_only = false;
                    }
                }
            }
        }

        // Priority order: CompilerTarget > GeneratedHeader > Layout > HeaderSurface > Macro > ImplementationOnly
        if has_abi || has_layout {
            ChangeClassification::Layout
        } else if has_header {
            ChangeClassification::HeaderSurface
        } else if has_macro {
            ChangeClassification::MacroCondition
        } else if has_source_only {
            ChangeClassification::ImplementationOnly
        } else {
            ChangeClassification::Unknown
        }
    }

    /// Classify a single node
    fn classify_node(&self, node: &DepNode) -> ChangeClassification {
        match node.kind {
            DepNodeKind::Header => {
                // Check if it's a generated header
                if node.name.contains("_generated")
                    || node.name.contains("_gen.h")
                    || node
                        .file_path
                        .as_ref()
                        .map(|p| p.contains("_generated"))
                        .unwrap_or(false)
                {
                    ChangeClassification::GeneratedHeader
                } else {
                    ChangeClassification::HeaderSurface
                }
            }
            DepNodeKind::Macro => ChangeClassification::MacroCondition,
            DepNodeKind::Layout => ChangeClassification::Layout,
            DepNodeKind::Type => ChangeClassification::Layout,
            DepNodeKind::Declaration => ChangeClassification::Layout,
            DepNodeKind::Export => ChangeClassification::HeaderSurface,
            DepNodeKind::Import => ChangeClassification::HeaderSurface,
            DepNodeKind::Source => {
                // Check if this source only has body changes (not exposed in headers)
                if self.is_impl_only_source(node) {
                    ChangeClassification::ImplementationOnly
                } else {
                    ChangeClassification::HeaderSurface
                }
            }
            DepNodeKind::FunctionBody => ChangeClassification::ImplementationOnly,
            DepNodeKind::TranslationUnit => ChangeClassification::ImplementationOnly,
            DepNodeKind::Object | DepNodeKind::Wrapper | DepNodeKind::Proof | DepNodeKind::Link => {
                ChangeClassification::ImplementationOnly
            }
        }
    }

    /// Check if a source node only contains implementation (not exported)
    fn is_impl_only_source(&self, node: &DepNode) -> bool {
        // Check if this source file has any exports
        for edge in self.graph.get_edges_from(&node.id) {
            if edge.kind == DepEdgeKind::Exports {
                return false;
            }
        }
        true
    }
}

/// Reuse decision for an artifact based on invalidation kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReuseDecision {
    /// Artifact can be reused as-is
    Reuse,
    /// Artifact must be rebuilt
    Rebuild,
    /// Artifact is stale but can be used as reference for incremental rebuild
    Incremental,
}

impl ReuseDecision {
    pub fn can_reuse(&self) -> bool {
        matches!(self, ReuseDecision::Reuse)
    }
}

/// Artifact reuse rules based on change classification
pub struct ArtifactReuseRules;

impl ArtifactReuseRules {
    /// Determine reuse decision for an artifact based on its kind and the change classification
    pub fn reuse_decision(
        artifact_kind: CArtifactKind,
        classification: ChangeClassification,
    ) -> ReuseDecision {
        // Object files: reused only if implementation-only change
        if matches!(artifact_kind, CArtifactKind::Object) {
            match classification {
                ChangeClassification::ImplementationOnly => ReuseDecision::Reuse,
                ChangeClassification::HeaderSurface
                | ChangeClassification::MacroCondition
                | ChangeClassification::Layout
                | ChangeClassification::GeneratedHeader => ReuseDecision::Rebuild,
                ChangeClassification::CompilerTarget => ReuseDecision::Rebuild,
                ChangeClassification::Unknown => ReuseDecision::Rebuild,
            }
        }
        // Wrapper files: depend on whether ABI surface changed
        else if matches!(artifact_kind, CArtifactKind::Wrapper) {
            match classification {
                ChangeClassification::ImplementationOnly => ReuseDecision::Reuse,
                ChangeClassification::Layout
                | ChangeClassification::HeaderSurface
                | ChangeClassification::GeneratedHeader => ReuseDecision::Rebuild,
                ChangeClassification::MacroCondition => {
                    // Macros can affect wrappers, so rebuild
                    ReuseDecision::Rebuild
                }
                ChangeClassification::CompilerTarget => ReuseDecision::Rebuild,
                ChangeClassification::Unknown => ReuseDecision::Rebuild,
            }
        }
        // Proof files: depend on whether anything affecting correctness changed
        else if matches!(artifact_kind, CArtifactKind::Cproof) {
            match classification {
                ChangeClassification::ImplementationOnly => ReuseDecision::Incremental,
                ChangeClassification::Layout
                | ChangeClassification::HeaderSurface
                | ChangeClassification::MacroCondition
                | ChangeClassification::GeneratedHeader => ReuseDecision::Rebuild,
                ChangeClassification::CompilerTarget => ReuseDecision::Rebuild,
                ChangeClassification::Unknown => ReuseDecision::Rebuild,
            }
        }
        // Link artifacts: depend on all object files being stable
        else if matches!(artifact_kind, CArtifactKind::Link) {
            match classification {
                ChangeClassification::ImplementationOnly => ReuseDecision::Reuse,
                ChangeClassification::Layout
                | ChangeClassification::HeaderSurface
                | ChangeClassification::MacroCondition
                | ChangeClassification::GeneratedHeader
                | ChangeClassification::CompilerTarget
                | ChangeClassification::Unknown => ReuseDecision::Rebuild,
            }
        }
        // Semantic artifacts (.csnap, .cdep, .castpack): always rebuilt on any semantic change
        else if matches!(
            artifact_kind,
            CArtifactKind::Csnap | CArtifactKind::Cdep | CArtifactKind::CastPack
        ) {
            ReuseDecision::Rebuild
        }
        // CDialect, Cmeta: rebuilt on semantic changes
        else if matches!(
            artifact_kind,
            CArtifactKind::CDialect | CArtifactKind::Cmeta
        ) {
            match classification {
                ChangeClassification::ImplementationOnly => ReuseDecision::Reuse,
                _ => ReuseDecision::Rebuild,
            }
        }
        // Default: rebuild
        else {
            ReuseDecision::Rebuild
        }
    }

    /// Check if a specific artifact can be reused given the change classification
    pub fn can_reuse_artifact(
        artifact_kind: CArtifactKind,
        classification: ChangeClassification,
    ) -> bool {
        Self::reuse_decision(artifact_kind, classification).can_reuse()
    }
}

/// Result of analyzing artifact reuse
#[derive(Debug, Clone)]
pub struct ArtifactReuseAnalysis {
    /// The classification of the change
    pub change_classification: ChangeClassification,
    /// Whether the change affects downstream consumers
    pub affects_downstream: bool,
    /// Human-readable explanation
    pub explanation: String,
}

impl ArtifactReuseAnalysis {
    /// Create a new analysis result
    pub fn new(
        classification: ChangeClassification,
        changed_artifact_kinds: &[CArtifactKind],
    ) -> Self {
        let affects_downstream = classification.affects_downstream();
        let explanation = Self::build_explanation(classification, changed_artifact_kinds);

        Self {
            change_classification: classification,
            affects_downstream,
            explanation,
        }
    }

    fn build_explanation(
        classification: ChangeClassification,
        changed_kinds: &[CArtifactKind],
    ) -> String {
        use CArtifactKind::*;
        match classification {
            ChangeClassification::ImplementationOnly => {
                format!(
                    "Only private implementation bodies changed ({:?}). Downstream consumers not affected.",
                    changed_kinds
                )
            }
            ChangeClassification::HeaderSurface => {
                format!(
                    "Header surface changed ({:?}). All consumers must be rebuilt.",
                    changed_kinds
                )
            }
            ChangeClassification::MacroCondition => {
                format!(
                    "Macro or conditional compilation changed ({:?}). Recompile required.",
                    changed_kinds
                )
            }
            ChangeClassification::Layout => {
                format!(
                    "Type layout changed ({:?}). ABI-affecting. All consumers must be rebuilt.",
                    changed_kinds
                )
            }
            ChangeClassification::CompilerTarget => {
                format!(
                    "Compiler, target, or sysroot changed ({:?}). Full rebuild required.",
                    changed_kinds
                )
            }
            ChangeClassification::GeneratedHeader => {
                format!(
                    "Generated header changed ({:?}). Downstream consumers must be rebuilt.",
                    changed_kinds
                )
            }
            ChangeClassification::Unknown => {
                format!(
                    "Unknown change detected ({:?}). Conservative rebuild triggered.",
                    changed_kinds
                )
            }
        }
    }
}

/// Compute which artifacts to rebuild given changed nodes
pub fn compute_stale_artifacts(
    graph: &CDependencyGraph,
    changed_ids: &[DepNodeId],
    available_artifacts: &[(String, CArtifactKind)],
) -> Vec<(String, CArtifactKind, ReuseDecision)> {
    let classifier = InvalidationClassifier::new(graph);
    let classification = classifier.classify(changed_ids);

    available_artifacts
        .iter()
        .map(|(id, kind)| {
            let decision = ArtifactReuseRules::reuse_decision(*kind, classification);
            (id.clone(), *kind, decision)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_graph() -> CDependencyGraph {
        let mut graph = CDependencyGraph::new();

        // Add a header node
        graph.add_node(DepNode {
            id: DepNodeId(1.to_string()),
            kind: DepNodeKind::Header,
            name: "types.h".to_string(),
            file_path: Some("include/types.h".to_string()),
            content_hash: "hash1".to_string(),
            metadata: crate::DepNodeMetadata::default(),
        });

        // Add a source node
        graph.add_node(DepNode {
            id: DepNodeId(2.to_string()),
            kind: DepNodeKind::Source,
            name: "impl.c".to_string(),
            file_path: Some("src/impl.c".to_string()),
            content_hash: "hash2".to_string(),
            metadata: crate::DepNodeMetadata::default(),
        });

        // Add export edge
        graph.add_edge(crate::DepEdge {
            from: DepNodeId(2.to_string()),
            to: DepNodeId(1.to_string()),
            kind: DepEdgeKind::Exports,
        });

        graph
    }

    #[test]
    fn test_change_classification_affects_downstream() {
        assert!(ChangeClassification::HeaderSurface.affects_downstream());
        assert!(ChangeClassification::Layout.affects_downstream());
        assert!(ChangeClassification::CompilerTarget.affects_downstream());
        assert!(ChangeClassification::GeneratedHeader.affects_downstream());
        assert!(!ChangeClassification::ImplementationOnly.affects_downstream());
        assert!(!ChangeClassification::Unknown.affects_downstream());
    }

    #[test]
    fn test_change_classification_is_cosmetic() {
        assert!(ChangeClassification::ImplementationOnly.is_cosmetic());
        assert!(ChangeClassification::Unknown.is_cosmetic());
        assert!(!ChangeClassification::HeaderSurface.is_cosmetic());
        assert!(!ChangeClassification::Layout.is_cosmetic());
    }

    #[test]
    fn test_invalidation_classifier_header_change() {
        let graph = make_test_graph();
        let classifier = InvalidationClassifier::new(&graph);

        // Change the header
        let classification = classifier.classify(&[DepNodeId(1.to_string())]);
        assert_eq!(classification, ChangeClassification::HeaderSurface);
        assert!(classification.affects_downstream());
    }

    #[test]
    fn test_invalidation_classifier_impl_only() {
        let graph = make_test_graph();
        let classifier = InvalidationClassifier::new(&graph);

        // Change the source (has exports, so not impl-only in this case)
        let classification = classifier.classify(&[DepNodeId(2.to_string())]);
        // Source with exports is not impl-only
        assert!(!classification.is_cosmetic());
    }

    #[test]
    fn test_invalidation_classifier_empty() {
        let graph = make_test_graph();
        let classifier = InvalidationClassifier::new(&graph);

        let classification = classifier.classify(&[]);
        assert_eq!(classification, ChangeClassification::Unknown);
    }

    #[test]
    fn test_reuse_decision_object_impl_only() {
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Object,
            ChangeClassification::ImplementationOnly,
        );
        assert_eq!(decision, ReuseDecision::Reuse);
    }

    #[test]
    fn test_reuse_decision_object_header_change() {
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Object,
            ChangeClassification::HeaderSurface,
        );
        assert_eq!(decision, ReuseDecision::Rebuild);
    }

    #[test]
    fn test_reuse_decision_object_layout() {
        let decision =
            ArtifactReuseRules::reuse_decision(CArtifactKind::Object, ChangeClassification::Layout);
        assert_eq!(decision, ReuseDecision::Rebuild);
    }

    #[test]
    fn test_reuse_decision_wrapper_impl_only() {
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Wrapper,
            ChangeClassification::ImplementationOnly,
        );
        assert_eq!(decision, ReuseDecision::Reuse);
    }

    #[test]
    fn test_reuse_decision_wrapper_layout() {
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Wrapper,
            ChangeClassification::Layout,
        );
        assert_eq!(decision, ReuseDecision::Rebuild);
    }

    #[test]
    fn test_reuse_decision_proof_impl_only() {
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Cproof,
            ChangeClassification::ImplementationOnly,
        );
        assert_eq!(decision, ReuseDecision::Incremental);
    }

    #[test]
    fn test_reuse_decision_link_impl_only() {
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Link,
            ChangeClassification::ImplementationOnly,
        );
        assert_eq!(decision, ReuseDecision::Reuse);
    }

    #[test]
    fn test_reuse_decision_csnap_always_rebuild() {
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Csnap,
            ChangeClassification::ImplementationOnly,
        );
        assert_eq!(decision, ReuseDecision::Rebuild);
    }

    #[test]
    fn test_reuse_decision_cdep_always_rebuild() {
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Cdep,
            ChangeClassification::ImplementationOnly,
        );
        assert_eq!(decision, ReuseDecision::Rebuild);
    }

    #[test]
    fn test_reuse_decision_castpack_always_rebuild() {
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::CastPack,
            ChangeClassification::ImplementationOnly,
        );
        assert_eq!(decision, ReuseDecision::Rebuild);
    }

    #[test]
    fn test_reuse_decision_cmeta_impl_only() {
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Cmeta,
            ChangeClassification::ImplementationOnly,
        );
        assert_eq!(decision, ReuseDecision::Reuse);
    }

    #[test]
    fn test_reuse_decision_cdialect_impl_only() {
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::CDialect,
            ChangeClassification::ImplementationOnly,
        );
        assert_eq!(decision, ReuseDecision::Reuse);
    }

    #[test]
    fn test_can_reuse_artifact() {
        assert!(ArtifactReuseRules::can_reuse_artifact(
            CArtifactKind::Object,
            ChangeClassification::ImplementationOnly
        ));
        assert!(!ArtifactReuseRules::can_reuse_artifact(
            CArtifactKind::Object,
            ChangeClassification::Layout
        ));
    }

    #[test]
    fn test_compute_stale_artifacts() {
        let graph = make_test_graph();
        let artifacts = vec![
            ("obj1".to_string(), CArtifactKind::Object),
            ("wrapper1".to_string(), CArtifactKind::Wrapper),
        ];

        let result = compute_stale_artifacts(&graph, &[DepNodeId(2.to_string())], &artifacts);

        // Source change (not impl-only since it has exports) -> rebuild all
        for (_, _, decision) in &result {
            assert_eq!(*decision, ReuseDecision::Rebuild);
        }
    }

    #[test]
    fn test_change_classification_to_invalidation_kind() {
        assert_eq!(
            ChangeClassification::ImplementationOnly.to_invalidation_kind(),
            crate::envelope::CInvalidationKind::ImplementationOnly
        );
        assert_eq!(
            ChangeClassification::HeaderSurface.to_invalidation_kind(),
            crate::envelope::CInvalidationKind::HeaderSurface
        );
        assert_eq!(
            ChangeClassification::Layout.to_invalidation_kind(),
            crate::envelope::CInvalidationKind::Layout
        );
        assert_eq!(
            ChangeClassification::MacroCondition.to_invalidation_kind(),
            crate::envelope::CInvalidationKind::MacroCondition
        );
        assert_eq!(
            ChangeClassification::CompilerTarget.to_invalidation_kind(),
            crate::envelope::CInvalidationKind::CompilerTarget
        );
        assert_eq!(
            ChangeClassification::GeneratedHeader.to_invalidation_kind(),
            crate::envelope::CInvalidationKind::GeneratedHeader
        );
    }

    #[test]
    fn test_artifact_reuse_analysis() {
        let analysis = ArtifactReuseAnalysis::new(
            ChangeClassification::HeaderSurface,
            &[CArtifactKind::Object],
        );

        assert_eq!(
            analysis.change_classification,
            ChangeClassification::HeaderSurface
        );
        assert!(analysis.affects_downstream);
        assert!(analysis.explanation.contains("Header surface"));
    }

    #[test]
    fn test_artifact_reuse_analysis_impl_only() {
        let analysis = ArtifactReuseAnalysis::new(
            ChangeClassification::ImplementationOnly,
            &[CArtifactKind::Object],
        );

        assert_eq!(
            analysis.change_classification,
            ChangeClassification::ImplementationOnly
        );
        assert!(!analysis.affects_downstream);
        assert!(analysis
            .explanation
            .contains("private implementation bodies"));
    }

    #[test]
    fn test_reuse_decision_compiler_target() {
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Object,
            ChangeClassification::CompilerTarget,
        );
        assert_eq!(decision, ReuseDecision::Rebuild);

        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Link,
            ChangeClassification::CompilerTarget,
        );
        assert_eq!(decision, ReuseDecision::Rebuild);
    }

    // **PR 6**: Differential and cross-language invalidation tests

    fn make_impl_only_graph() -> CDependencyGraph {
        // Graph where source has no exports (impl-only)
        let mut graph = CDependencyGraph::new();

        // Add a source node with no exports
        graph.add_node(DepNode {
            id: DepNodeId(1.to_string()),
            kind: DepNodeKind::Source,
            name: "impl.c".to_string(),
            file_path: Some("src/impl.c".to_string()),
            content_hash: "hash1".to_string(),
            metadata: crate::DepNodeMetadata::default(),
        });

        // No export edge - this source only has private implementation

        graph
    }

    #[test]
    fn test_impl_only_change_does_not_affect_downstream() {
        // Private implementation body change - downstream should not be rebuilt
        let graph = make_impl_only_graph();
        let classifier = InvalidationClassifier::new(&graph);

        // Source-only change (impl.c body, no exports changed)
        let classification = classifier.classify(&[DepNodeId(1.to_string())]);
        assert!(classification.is_cosmetic());
        assert!(!classification.affects_downstream());
    }

    #[test]
    fn test_header_change_affects_downstream() {
        // Header surface change - downstream must be rebuilt
        let graph = make_test_graph();
        let classifier = InvalidationClassifier::new(&graph);

        // Header change affects consumers
        let classification = classifier.classify(&[DepNodeId(1.to_string())]);
        assert!(!classification.is_cosmetic());
        assert!(classification.affects_downstream());
    }

    #[test]
    fn test_layout_change_requires_rebuild_all() {
        // Layout change affects ABI - all consumers must rebuild
        let mut graph = CDependencyGraph::new();

        // Add a type node (layout-affecting)
        graph.add_node(DepNode {
            id: DepNodeId(3.to_string()),
            kind: DepNodeKind::Type,
            name: "MyStruct".to_string(),
            file_path: Some("include/types.h".to_string()),
            content_hash: "hash3".to_string(),
            metadata: crate::DepNodeMetadata::default(),
        });

        let classifier = InvalidationClassifier::new(&graph);
        let classification = classifier.classify(&[DepNodeId(3.to_string())]);

        assert_eq!(classification, ChangeClassification::Layout);
        assert!(classification.affects_downstream());
    }

    #[test]
    fn test_macro_change_affects_downstream() {
        // Macro change can affect compilation conditions
        let mut graph = CDependencyGraph::new();

        graph.add_node(DepNode {
            id: DepNodeId(4.to_string()),
            kind: DepNodeKind::Macro,
            name: "FEATURE_FLAG".to_string(),
            file_path: Some("include/config.h".to_string()),
            content_hash: "hash4".to_string(),
            metadata: crate::DepNodeMetadata::default(),
        });

        let classifier = InvalidationClassifier::new(&graph);
        let classification = classifier.classify(&[DepNodeId(4.to_string())]);

        assert_eq!(classification, ChangeClassification::MacroCondition);
        // Macro changes can affect downstream
        assert!(!classification.is_cosmetic());
    }

    #[test]
    fn test_multiple_changes_priority_layout() {
        // Layout changes have highest priority
        let mut graph = CDependencyGraph::new();

        graph.add_node(DepNode {
            id: DepNodeId(1.to_string()),
            kind: DepNodeKind::Header,
            name: "types.h".to_string(),
            file_path: Some("include/types.h".to_string()),
            content_hash: "hash1".to_string(),
            metadata: crate::DepNodeMetadata::default(),
        });

        graph.add_node(DepNode {
            id: DepNodeId(5.to_string()),
            kind: DepNodeKind::Type,
            name: "MyStruct".to_string(),
            file_path: Some("include/types.h".to_string()),
            content_hash: "hash5".to_string(),
            metadata: crate::DepNodeMetadata::default(),
        });

        let classifier = InvalidationClassifier::new(&graph);

        // Both header and type changed - should classify as Layout
        let classification =
            classifier.classify(&[DepNodeId(1.to_string()), DepNodeId(5.to_string())]);

        assert_eq!(classification, ChangeClassification::Layout);
    }

    #[test]
    fn test_object_reuse_only_on_impl_only() {
        // Object files can only be reused on implementation-only changes
        assert!(ArtifactReuseRules::can_reuse_artifact(
            CArtifactKind::Object,
            ChangeClassification::ImplementationOnly
        ));

        assert!(!ArtifactReuseRules::can_reuse_artifact(
            CArtifactKind::Object,
            ChangeClassification::HeaderSurface
        ));
        assert!(!ArtifactReuseRules::can_reuse_artifact(
            CArtifactKind::Object,
            ChangeClassification::Layout
        ));
        assert!(!ArtifactReuseRules::can_reuse_artifact(
            CArtifactKind::Object,
            ChangeClassification::MacroCondition
        ));
    }

    #[test]
    fn test_link_artifact_on_impl_only() {
        // Link artifact can be reused only on implementation-only changes
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Link,
            ChangeClassification::ImplementationOnly,
        );
        assert_eq!(decision, ReuseDecision::Reuse);

        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Link,
            ChangeClassification::HeaderSurface,
        );
        assert_eq!(decision, ReuseDecision::Rebuild);
    }

    #[test]
    fn test_proof_artifact_incremental_on_impl_only() {
        // Proof artifacts can be used incrementally on impl-only changes
        let decision = ArtifactReuseRules::reuse_decision(
            CArtifactKind::Cproof,
            ChangeClassification::ImplementationOnly,
        );
        assert_eq!(decision, ReuseDecision::Incremental);
    }

    #[test]
    fn test_wrapper_rebuild_on_any_abi_change() {
        // Wrappers must rebuild on any ABI-affecting change
        assert!(!ArtifactReuseRules::can_reuse_artifact(
            CArtifactKind::Wrapper,
            ChangeClassification::Layout
        ));
        assert!(!ArtifactReuseRules::can_reuse_artifact(
            CArtifactKind::Wrapper,
            ChangeClassification::HeaderSurface
        ));
        assert!(!ArtifactReuseRules::can_reuse_artifact(
            CArtifactKind::Wrapper,
            ChangeClassification::GeneratedHeader
        ));
    }

    #[test]
    fn test_computed_stale_artifacts_impl_only() {
        // Use impl-only graph where source has no exports
        let graph = make_impl_only_graph();
        let artifacts = vec![
            ("main.o".to_string(), CArtifactKind::Object),
            ("lib.o".to_string(), CArtifactKind::Object),
            ("app.chwrap".to_string(), CArtifactKind::Wrapper),
            ("final".to_string(), CArtifactKind::Link),
        ];

        // Source-only change - objects can be reused, wrappers and links rebuilt
        let result = compute_stale_artifacts(
            &graph,
            &[DepNodeId(1.to_string())], // impl.c (no exports) changed
            &artifacts,
        );

        let decisions: Vec<_> = result.iter().map(|(_, _, d)| *d).collect();
        // On impl-only change: object and wrapper can be reused, link can be reused
        assert_eq!(decisions[0], ReuseDecision::Reuse); // main.o
        assert_eq!(decisions[2], ReuseDecision::Reuse); // wrapper - impl-only change affects nothing exported
        assert_eq!(decisions[3], ReuseDecision::Reuse); // link - nothing changed that affects final artifact
    }

    #[test]
    fn test_computed_stale_artifacts_header_change() {
        let graph = make_test_graph();
        let artifacts = vec![
            ("main.o".to_string(), CArtifactKind::Object),
            ("app.chwrap".to_string(), CArtifactKind::Wrapper),
        ];

        // Header change - everything must rebuild
        let result = compute_stale_artifacts(
            &graph,
            &[DepNodeId(1.to_string())], // types.h changed
            &artifacts,
        );

        for (_, _, decision) in result {
            assert_eq!(decision, ReuseDecision::Rebuild);
        }
    }

    #[test]
    fn test_generated_header_change() {
        // Generated headers like _generated.h affect downstream
        let mut graph = CDependencyGraph::new();

        graph.add_node(DepNode {
            id: DepNodeId(1.to_string()),
            kind: DepNodeKind::Header,
            name: "types_generated.h".to_string(),
            file_path: Some("include/types_generated.h".to_string()),
            content_hash: "hash1".to_string(),
            metadata: crate::DepNodeMetadata::default(),
        });

        let classifier = InvalidationClassifier::new(&graph);
        let classification = classifier.classify(&[DepNodeId(1.to_string())]);

        assert_eq!(classification, ChangeClassification::GeneratedHeader);
        assert!(classification.affects_downstream());
    }
}
