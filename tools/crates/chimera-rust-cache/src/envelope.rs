//! Rust Build Result Envelope for chimerair integration
//!
//! This module defines the canonical Rust build result envelope that `chimerair`
//! consumes to determine stale/reusable artifact sets and downstream invalidation.
//!
//! # Design
//!
//! The envelope is language-agnostic at the `chimerair` level, but contains
//! Rust-specific fields for Rust-origin modules.
//!
//! # Exit Criteria (from rust-incremental-ownership-plan.md)
//!
//! - semantic state ID
//! - stale/reusable artifact sets
//! - public-surface summary
//! - explanation records

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use chimera_rust_schema::{ArtifactHeader, CrateGraph, ItemId, RdepGraph, RsnapSnapshot};

/// Build status for the Rust frontend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustBuildStatus {
    /// Build completed successfully with all artifacts
    Success,
    /// Build completed but fell back to surface-only mode
    FallbackSurfaceOnly,
    /// Build failed
    Failed,
}

/// Kind of invalidation detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvalidationKind {
    /// Private body only changed (no downstream effect)
    PrivateBodyOnly,
    /// Exported signature changed
    ExportedSignature,
    /// Layout changed (affects ABI)
    Layout,
    /// Proc-macro expansion changed
    ProcMacro,
    /// Build-script output changed
    BuildScript,
    /// Target or profile changed
    TargetProfile,
}

/// Explanation of why a build result was produced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustBuildExplanation {
    /// What kind of invalidation was detected
    pub invalidation_kind: InvalidationKind,
    /// Which nodes in the dependency graph changed
    pub changed_nodes: Vec<String>,
    /// Whether this change affects downstream consumers
    pub affects_downstream: bool,
    /// Plaintext explanation suitable for display
    pub human_readable: String,
}

/// A stale or reusable artifact reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustArtifactRef {
    /// Unique stable ID for this artifact
    pub stable_id: String,
    /// What kind of artifact this is
    pub kind: RustArtifactKind,
    /// Path where the artifact is stored (if cached)
    pub path: Option<PathBuf>,
    /// Fingerprint of the artifact content
    pub fingerprint: String,
}

/// Kind of Rust artifact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustArtifactKind {
    /// Semantic snapshot (.rsnap)
    Rsnap,
    /// Dependency graph (.rdep)
    Rdep,
    /// MIR package (.rmirpack)
    RmirPack,
    /// Rust metadata (.rchmeta)
    RchMeta,
    /// Rust proof facts (.rchproof)
    RchProof,
    /// Object file (.cho)
    Object,
    /// Link artifact (.link)
    Link,
    /// Wrapper file
    Wrapper,
}

impl RustArtifactKind {
    /// Get file extension for this artifact kind
    pub fn extension(&self) -> &'static str {
        match self {
            RustArtifactKind::Rsnap => ".rsnap",
            RustArtifactKind::Rdep => ".rdep",
            RustArtifactKind::RmirPack => ".rmirpack",
            RustArtifactKind::RchMeta => ".rchmeta",
            RustArtifactKind::RchProof => ".rchproof",
            RustArtifactKind::Object => ".cho",
            RustArtifactKind::Link => ".link",
            RustArtifactKind::Wrapper => ".chwrap",
        }
    }
}

/// Public surface summary from a Rust crate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustPublicSurface {
    /// Name of the crate
    pub crate_name: String,
    /// Version of the crate
    pub version: String,
    /// Exported items (functions, types, etc.)
    pub exports: Vec<RustExportSummary>,
    /// ABIs that this crate exposes
    pub abi_surfaces: Vec<String>,
    /// Layout-affecting items
    pub layout_items: Vec<String>,
}

/// Summary of an exported item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustExportSummary {
    /// Item ID
    pub item_id: ItemId,
    /// Definition path
    pub def_path: String,
    /// ABI of this export
    pub abi: String,
    /// Whether this affects layout
    pub affects_layout: bool,
}

/// The canonical Rust build result envelope
///
/// This is the main output of the Rust frontend that `chimerair` consumes
/// to make scheduling and invalidation decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustBuildResult {
    /// Header for artifact identification
    pub header: ArtifactHeader,
    /// Semantic state identifier (changes when any semantic fact changes)
    pub semantic_state_id: String,
    /// Build status
    pub status: RustBuildStatus,
    /// Explanation of the build result
    pub explanation: Option<RustBuildExplanation>,
    /// Artifacts that were produced and are reusable
    pub reusable_artifacts: Vec<RustArtifactRef>,
    /// Artifacts that are stale and need rebuild
    pub stale_artifacts: Vec<RustArtifactRef>,
    /// Public surface summary
    pub public_surface: Option<RustPublicSurface>,
    /// Reference to the source crate graph
    pub crate_graph: Option<CrateGraph>,
    /// Reference to the dependency graph
    pub rdep_graph: Option<RdepGraph>,
    /// Reference to the semantic snapshot
    pub rsnap_snapshot: Option<RsnapSnapshot>,
    /// Whether this was an authoritative (semantic) build
    pub is_authoritative: bool,
    /// Fingerprint of the complete build inputs
    pub build_input_fingerprint: String,
    /// Timestamp of the build
    pub timestamp_secs: u64,
}

impl RustBuildResult {
    /// Create a new Rust build result envelope
    pub fn new(
        target: &str,
        version: &str,
        is_authoritative: bool,
        semantic_state_id: String,
        build_input_fingerprint: String,
    ) -> Self {
        Self {
            header: ArtifactHeader::new(target, version),
            semantic_state_id,
            status: RustBuildStatus::Success,
            explanation: None,
            reusable_artifacts: Vec::new(),
            stale_artifacts: Vec::new(),
            public_surface: None,
            crate_graph: None,
            rdep_graph: None,
            rsnap_snapshot: None,
            is_authoritative,
            build_input_fingerprint,
            timestamp_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    /// Set the build status
    pub fn with_status(mut self, status: RustBuildStatus) -> Self {
        self.status = status;
        self
    }

    /// Set the explanation
    pub fn with_explanation(mut self, explanation: RustBuildExplanation) -> Self {
        self.explanation = Some(explanation);
        self
    }

    /// Add a reusable artifact
    pub fn add_reusable(mut self, artifact: RustArtifactRef) -> Self {
        self.reusable_artifacts.push(artifact);
        self
    }

    /// Add a stale artifact
    pub fn add_stale(mut self, artifact: RustArtifactRef) -> Self {
        self.stale_artifacts.push(artifact);
        self
    }

    /// Set the public surface
    pub fn with_public_surface(mut self, surface: RustPublicSurface) -> Self {
        self.public_surface = Some(surface);
        self
    }

    /// Set the crate graph reference
    pub fn with_crate_graph(mut self, graph: CrateGraph) -> Self {
        self.crate_graph = Some(graph);
        self
    }

    /// Set the dependency graph reference
    pub fn with_rdep_graph(mut self, graph: RdepGraph) -> Self {
        self.rdep_graph = Some(graph);
        self
    }

    /// Set the semantic snapshot reference
    pub fn with_rsnap_snapshot(mut self, snapshot: RsnapSnapshot) -> Self {
        self.rsnap_snapshot = Some(snapshot);
        self
    }

    /// Compute the semantic state ID from artifacts
    pub fn compute_semantic_state_id(
        rsnap: Option<&RsnapSnapshot>,
        rdep: Option<&RdepGraph>,
    ) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();

        if let Some(snapshot) = rsnap {
            hasher.update(snapshot.compute_checksum().as_bytes());
        }
        if let Some(graph) = rdep {
            hasher.update(graph.compute_checksum().as_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }

    /// Check if this is a no-op rebuild (all artifacts reusable)
    pub fn is_noop_rebuild(&self) -> bool {
        self.stale_artifacts.is_empty() && !self.reusable_artifacts.is_empty()
    }

    /// Check if this build has any reusable Rust artifacts
    pub fn has_reusable_artifacts(&self) -> bool {
        !self.reusable_artifacts.is_empty()
    }

    /// Get all artifact stable IDs
    pub fn all_artifact_ids(&self) -> Vec<String> {
        self.reusable_artifacts
            .iter()
            .chain(self.stale_artifacts.iter())
            .map(|a| a.stable_id.clone())
            .collect()
    }

    /// Create a fallback (surface-only) build result
    pub fn fallback_surface_only(
        target: &str,
        version: &str,
        build_input_fingerprint: String,
    ) -> Self {
        Self::new(
            target,
            version,
            false,
            String::new(),
            build_input_fingerprint,
        )
        .with_status(RustBuildStatus::FallbackSurfaceOnly)
    }
}

/// Convert from Rust artifacts to build result envelope
impl From<RsnapSnapshot> for RustBuildResult {
    fn from(snapshot: RsnapSnapshot) -> Self {
        let semantic_state_id = Self::compute_semantic_state_id(Some(&snapshot), None);
        let build_input_fingerprint = snapshot.compute_checksum();

        Self::new(
            &snapshot.header.target,
            &snapshot.header.producer_version,
            true,
            semantic_state_id.clone(),
            build_input_fingerprint,
        )
        .with_status(RustBuildStatus::Success)
        .with_rsnap_snapshot(snapshot.clone())
        .with_public_surface(RustPublicSurface {
            crate_name: snapshot
                .crate_graph
                .nodes
                .first()
                .map(|n| n.name.clone())
                .unwrap_or_default(),
            version: String::new(),
            exports: snapshot
                .exports
                .iter()
                .map(|e| RustExportSummary {
                    item_id: e.item_id,
                    def_path: format!("{:?}", e.item_id),
                    abi: e.abi.clone(),
                    affects_layout: false,
                })
                .collect(),
            abi_surfaces: snapshot
                .exports
                .iter()
                .filter(|e| e.abi != "Rust")
                .map(|e| e.abi.clone())
                .collect(),
            layout_items: Vec::new(),
        })
    }
}

/// Compute fingerprint for build inputs
pub fn compute_build_input_fingerprint(
    source_files: &[String],
    crate_graph: &CrateGraph,
    build_config: &RustBuildConfig,
) -> String {
    use blake3::Hasher;
    let mut hasher = Hasher::new();

    for file in source_files {
        hasher.update(file.as_bytes());
    }

    hasher.update(serde_json::to_string(crate_graph).unwrap().as_bytes());
    hasher.update(build_config.target.as_bytes());
    hasher.update(build_config.profile.as_bytes());

    hasher.finalize().to_hex().to_string()
}

/// Configuration for Rust builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustBuildConfig {
    /// Target triple
    pub target: String,
    /// Build profile (debug, release)
    pub profile: String,
    /// Whether to enable semantic extraction
    pub semantic_extraction: bool,
    /// rustc version string
    pub rustc_version: String,
    /// Build script outputs (if any)
    pub build_script_outputs: Vec<BuildScriptOutputSummary>,
    /// Proc-macro crate versions
    pub proc_macro_versions: Vec<ProcMacroVersion>,
}

impl Default for RustBuildConfig {
    fn default() -> Self {
        Self {
            target: "x86_64-unknown-linux-gnu".to_string(),
            profile: "debug".to_string(),
            semantic_extraction: false,
            rustc_version: "1.75.0".to_string(),
            build_script_outputs: Vec::new(),
            proc_macro_versions: Vec::new(),
        }
    }
}

/// Summary of build script output for cache key fingerprinting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildScriptOutputSummary {
    /// Build script source file path
    pub script_path: String,
    /// cargo:rustc-cfg flags
    pub rustc_cfg: Vec<String>,
    /// cargo:rustc-link-lib entries
    pub link_libs: Vec<String>,
    /// Environment variables that affect build
    pub env_vars: Vec<String>,
    /// Files that trigger rebuild
    pub rerun_if_changed: Vec<String>,
}

impl BuildScriptOutputSummary {
    /// Compute fingerprint for this build script output
    pub fn fingerprint(&self) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();

        hasher.update(self.script_path.as_bytes());
        for cfg in &self.rustc_cfg {
            hasher.update(cfg.as_bytes());
        }
        for lib in &self.link_libs {
            hasher.update(lib.as_bytes());
        }
        for env in &self.env_vars {
            hasher.update(env.as_bytes());
        }
        for file in &self.rerun_if_changed {
            hasher.update(file.as_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }
}

/// Proc-macro crate version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcMacroVersion {
    pub crate_name: String,
    pub version: String,
    /// Hash of the expanded macro tokens
    pub expanded_token_hash: Option<String>,
}

impl ProcMacroVersion {
    /// Compute fingerprint for this proc-macro
    pub fn fingerprint(&self) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();

        hasher.update(self.crate_name.as_bytes());
        hasher.update(self.version.as_bytes());
        if let Some(ref token_hash) = self.expanded_token_hash {
            hasher.update(token_hash.as_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_rust_schema::{
        CrateId, CrateNode, CrateType, DepEdge, DepEdgeKind, DepNode, DepNodeId, DepNodeKind,
        ItemId, ItemKind, Linkage, RsnapExport, RsnapItem, Visibility, VisibilityRank,
    };

    fn make_test_config() -> RustBuildConfig {
        RustBuildConfig {
            target: "x86_64-unknown-linux-gnu".to_string(),
            profile: "debug".to_string(),
            semantic_extraction: true,
            rustc_version: "1.75.0".to_string(),
            build_script_outputs: vec![],
            proc_macro_versions: vec![],
        }
    }

    #[test]
    fn test_build_result_new() {
        let result = RustBuildResult::new(
            "x86_64-unknown-linux-gnu",
            "0.1.0",
            true,
            "state_abc123".to_string(),
            "input_fp".to_string(),
        );

        assert_eq!(result.header.target, "x86_64-unknown-linux-gnu");
        assert_eq!(result.semantic_state_id, "state_abc123");
        assert!(result.is_authoritative);
        assert_eq!(result.status, RustBuildStatus::Success);
    }

    #[test]
    fn test_build_result_add_artifacts() {
        let result = RustBuildResult::new(
            "x86_64-unknown-linux-gnu",
            "0.1.0",
            true,
            "state_abc".to_string(),
            "input_fp".to_string(),
        )
        .add_reusable(RustArtifactRef {
            stable_id: "obj_0".to_string(),
            kind: RustArtifactKind::Object,
            path: Some(PathBuf::from("build/lib_0.cho")),
            fingerprint: "fp_abc".to_string(),
        })
        .add_stale(RustArtifactRef {
            stable_id: "obj_1".to_string(),
            kind: RustArtifactKind::Object,
            path: Some(PathBuf::from("build/lib_1.cho")),
            fingerprint: "fp_def".to_string(),
        });

        assert_eq!(result.reusable_artifacts.len(), 1);
        assert_eq!(result.stale_artifacts.len(), 1);
    }

    #[test]
    fn test_build_result_is_noop() {
        let result = RustBuildResult::new(
            "x86_64-unknown-linux-gnu",
            "0.1.0",
            true,
            "state".to_string(),
            "input".to_string(),
        )
        .add_reusable(RustArtifactRef {
            stable_id: "obj".to_string(),
            kind: RustArtifactKind::Object,
            path: Some(PathBuf::from("build/lib.cho")),
            fingerprint: "fp".to_string(),
        });

        assert!(result.is_noop_rebuild());
    }

    #[test]
    fn test_build_result_not_noop_with_stale() {
        let result = RustBuildResult::new(
            "x86_64-unknown-linux-gnu",
            "0.1.0",
            true,
            "state".to_string(),
            "input".to_string(),
        )
        .add_stale(RustArtifactRef {
            stable_id: "obj".to_string(),
            kind: RustArtifactKind::Object,
            path: None,
            fingerprint: "fp".to_string(),
        });

        assert!(!result.is_noop_rebuild());
    }

    #[test]
    fn test_build_result_fallback() {
        let result = RustBuildResult::fallback_surface_only(
            "aarch64-apple-darwin",
            "0.2.0",
            "fallback_fp".to_string(),
        );

        assert_eq!(result.status, RustBuildStatus::FallbackSurfaceOnly);
        assert!(!result.is_authoritative);
    }

    #[test]
    fn test_invalidation_kind_serialization() {
        let kinds = vec![
            InvalidationKind::PrivateBodyOnly,
            InvalidationKind::ExportedSignature,
            InvalidationKind::Layout,
            InvalidationKind::ProcMacro,
            InvalidationKind::BuildScript,
            InvalidationKind::TargetProfile,
        ];

        for kind in kinds {
            let json = serde_json::to_string(&kind).unwrap();
            let parsed: InvalidationKind = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, kind);
        }
    }

    #[test]
    fn test_artifact_kind_extensions() {
        assert_eq!(RustArtifactKind::Rsnap.extension(), ".rsnap");
        assert_eq!(RustArtifactKind::Rdep.extension(), ".rdep");
        assert_eq!(RustArtifactKind::RmirPack.extension(), ".rmirpack");
        assert_eq!(RustArtifactKind::RchMeta.extension(), ".rchmeta");
        assert_eq!(RustArtifactKind::RchProof.extension(), ".rchproof");
        assert_eq!(RustArtifactKind::Object.extension(), ".cho");
        assert_eq!(RustArtifactKind::Link.extension(), ".link");
        assert_eq!(RustArtifactKind::Wrapper.extension(), ".chwrap");
    }

    #[test]
    fn test_build_config_default() {
        let config = RustBuildConfig::default();
        assert_eq!(config.target, "x86_64-unknown-linux-gnu");
        assert_eq!(config.profile, "debug");
        assert!(!config.semantic_extraction);
    }

    #[test]
    fn test_compute_build_input_fingerprint() {
        let config = make_test_config();
        let crate_graph = CrateGraph {
            root: CrateId(0),
            nodes: vec![CrateNode {
                id: CrateId(0),
                name: "test_crate".to_string(),
                package_name: None,
                version: None,
                source_kind: None,
                source: None,
                source_ref: None,
                edition: "2021".to_string(),
                crate_type: CrateType::Library,
                dependency_crates: vec![],
                extern_prelude: vec![],
                features: vec![],
                default_features: true,
                optional: false,
            }],
        };

        let fp =
            compute_build_input_fingerprint(&["src/lib.rs".to_string()], &crate_graph, &config);

        assert_eq!(fp.len(), 64); // blake3 hex
    }

    #[test]
    fn test_compute_semantic_state_id() {
        let snapshot = RsnapSnapshot {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            rustc_version: "1.75.0".to_string(),
            crate_graph: CrateGraph {
                root: CrateId(0),
                nodes: vec![],
            },
            items: vec![],
            exports: vec![],
            source_files: vec![],
        };

        let state_id = RustBuildResult::compute_semantic_state_id(Some(&snapshot), None);
        assert_eq!(state_id.len(), 64);
    }

    #[test]
    fn test_all_artifact_ids() {
        let result = RustBuildResult::new(
            "x86_64-unknown-linux-gnu",
            "0.1.0",
            true,
            "state".to_string(),
            "input".to_string(),
        )
        .add_reusable(RustArtifactRef {
            stable_id: "reusable_0".to_string(),
            kind: RustArtifactKind::Object,
            path: None,
            fingerprint: "fp_0".to_string(),
        })
        .add_stale(RustArtifactRef {
            stable_id: "stale_0".to_string(),
            kind: RustArtifactKind::Object,
            path: None,
            fingerprint: "fp_1".to_string(),
        });

        let ids = result.all_artifact_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"reusable_0".to_string()));
        assert!(ids.contains(&"stale_0".to_string()));
    }

    #[test]
    fn test_public_surface_from_snapshot() {
        let snapshot = RsnapSnapshot {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            rustc_version: "1.75.0".to_string(),
            crate_graph: CrateGraph {
                root: CrateId(0),
                nodes: vec![CrateNode {
                    id: CrateId(0),
                    name: "my_lib".to_string(),
                    package_name: None,
                    version: None,
                    source_kind: None,
                    source: None,
                    source_ref: None,
                    edition: "2021".to_string(),
                    crate_type: CrateType::Library,
                    dependency_crates: vec![],
                    extern_prelude: vec![],
                    features: vec![],
                    default_features: true,
                    optional: false,
                }],
            },
            items: vec![RsnapItem {
                id: ItemId(1),
                def_path: "my_lib::add".to_string(),
                kind: ItemKind::Function,
                visibility: Visibility {
                    rank: VisibilityRank::Pub,
                    path: None,
                },
                attributes: vec![],
                generics: None,
                where_clauses: vec![],
            }],
            exports: vec![RsnapExport {
                symbol: "add".to_string(),
                item_id: ItemId(1),
                abi: "C".to_string(),
                linkage: Linkage::External,
            }],
            source_files: vec![],
        };

        let result = RustBuildResult::from(snapshot);

        assert!(result.public_surface.is_some());
        let surface = result.public_surface.unwrap();
        assert_eq!(surface.crate_name, "my_lib");
        assert_eq!(surface.exports.len(), 1);
        assert_eq!(surface.abi_surfaces.len(), 1);
    }

    #[test]
    fn test_explanation_serialization() {
        let explanation = RustBuildExplanation {
            invalidation_kind: InvalidationKind::Layout,
            changed_nodes: vec!["item_0".to_string(), "type_0".to_string()],
            affects_downstream: true,
            human_readable: "Layout of MyStruct changed".to_string(),
        };

        let json = serde_json::to_string_pretty(&explanation).unwrap();
        let parsed: RustBuildExplanation = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.invalidation_kind, InvalidationKind::Layout);
        assert_eq!(parsed.changed_nodes.len(), 2);
    }

    #[test]
    fn test_build_result_roundtrip() {
        let result = RustBuildResult::new(
            "x86_64-unknown-linux-gnu",
            "0.1.0",
            true,
            "sem_state_abc".to_string(),
            "build_input_fp".to_string(),
        )
        .with_status(RustBuildStatus::Success)
        .with_explanation(RustBuildExplanation {
            invalidation_kind: InvalidationKind::PrivateBodyOnly,
            changed_nodes: vec!["mir_body_0".to_string()],
            affects_downstream: false,
            human_readable: "Only private MIR body changed".to_string(),
        })
        .add_reusable(RustArtifactRef {
            stable_id: "rsnap_0".to_string(),
            kind: RustArtifactKind::Rsnap,
            path: Some(PathBuf::from(".cache/rsnap_0.rsnap")),
            fingerprint: "fp_rsnap".to_string(),
        });

        let json = serde_json::to_string(&result).unwrap();
        let parsed: RustBuildResult = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.semantic_state_id, "sem_state_abc");
        assert_eq!(parsed.status, RustBuildStatus::Success);
        assert!(parsed.explanation.is_some());
        assert_eq!(parsed.reusable_artifacts.len(), 1);
    }

    #[test]
    fn test_build_script_output_summary_fingerprint() {
        let summary = BuildScriptOutputSummary {
            script_path: "build.rs".to_string(),
            rustc_cfg: vec!["cfg(debug)".to_string()],
            link_libs: vec!["foo:bar".to_string()],
            env_vars: vec!["FOO=bar".to_string()],
            rerun_if_changed: vec!["config.txt".to_string()],
        };

        let fp = summary.fingerprint();
        assert_eq!(fp.len(), 64); // blake3 hex
    }

    #[test]
    fn test_build_script_output_summary_deterministic() {
        let summary1 = BuildScriptOutputSummary {
            script_path: "build.rs".to_string(),
            rustc_cfg: vec!["cfg(debug)".to_string()],
            link_libs: vec!["foo:bar".to_string()],
            env_vars: vec!["FOO=bar".to_string()],
            rerun_if_changed: vec!["config.txt".to_string()],
        };

        let summary2 = BuildScriptOutputSummary {
            script_path: "build.rs".to_string(),
            rustc_cfg: vec!["cfg(debug)".to_string()],
            link_libs: vec!["foo:bar".to_string()],
            env_vars: vec!["FOO=bar".to_string()],
            rerun_if_changed: vec!["config.txt".to_string()],
        };

        assert_eq!(summary1.fingerprint(), summary2.fingerprint());
    }

    #[test]
    fn test_build_script_output_summary_changes_with_rustc_cfg() {
        let summary1 = BuildScriptOutputSummary {
            script_path: "build.rs".to_string(),
            rustc_cfg: vec!["cfg(debug)".to_string()],
            link_libs: vec![],
            env_vars: vec![],
            rerun_if_changed: vec![],
        };

        let summary2 = BuildScriptOutputSummary {
            script_path: "build.rs".to_string(),
            rustc_cfg: vec!["cfg(release)".to_string()],
            link_libs: vec![],
            env_vars: vec![],
            rerun_if_changed: vec![],
        };

        assert_ne!(summary1.fingerprint(), summary2.fingerprint());
    }

    #[test]
    fn test_proc_macro_version_fingerprint() {
        let pm = ProcMacroVersion {
            crate_name: "serde".to_string(),
            version: "1.0.0".to_string(),
            expanded_token_hash: Some("abc123".to_string()),
        };

        let fp = pm.fingerprint();
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn test_proc_macro_version_deterministic() {
        let pm1 = ProcMacroVersion {
            crate_name: "proc_macro_demo".to_string(),
            version: "2.0.0".to_string(),
            expanded_token_hash: None,
        };

        let pm2 = ProcMacroVersion {
            crate_name: "proc_macro_demo".to_string(),
            version: "2.0.0".to_string(),
            expanded_token_hash: None,
        };

        assert_eq!(pm1.fingerprint(), pm2.fingerprint());
    }

    #[test]
    fn test_proc_macro_version_changes_with_version() {
        let pm1 = ProcMacroVersion {
            crate_name: "my_macro".to_string(),
            version: "1.0.0".to_string(),
            expanded_token_hash: None,
        };

        let pm2 = ProcMacroVersion {
            crate_name: "my_macro".to_string(),
            version: "2.0.0".to_string(),
            expanded_token_hash: None,
        };

        assert_ne!(pm1.fingerprint(), pm2.fingerprint());
    }

    #[test]
    fn test_proc_macro_version_includes_token_hash() {
        let pm_no_hash = ProcMacroVersion {
            crate_name: "macro".to_string(),
            version: "1.0".to_string(),
            expanded_token_hash: None,
        };

        let pm_with_hash = ProcMacroVersion {
            crate_name: "macro".to_string(),
            version: "1.0".to_string(),
            expanded_token_hash: Some("token_hash_xyz".to_string()),
        };

        assert_ne!(pm_no_hash.fingerprint(), pm_with_hash.fingerprint());
    }

    #[test]
    fn test_rust_build_config_with_build_scripts() {
        let config = RustBuildConfig {
            target: "x86_64-unknown-linux-gnu".to_string(),
            profile: "release".to_string(),
            semantic_extraction: true,
            rustc_version: "1.76.0".to_string(),
            build_script_outputs: vec![BuildScriptOutputSummary {
                script_path: "build.rs".to_string(),
                rustc_cfg: vec![],
                link_libs: vec!["foo:libfoo".to_string()],
                env_vars: vec!["BUILD_SCRIPT_VAR=1".to_string()],
                rerun_if_changed: vec![],
            }],
            proc_macro_versions: vec![],
        };

        assert_eq!(config.build_script_outputs.len(), 1);
        assert_eq!(config.build_script_outputs[0].link_libs[0], "foo:libfoo");
    }

    #[test]
    fn test_rust_build_config_with_proc_macros() {
        let config = RustBuildConfig {
            target: "aarch64-apple-darwin".to_string(),
            profile: "debug".to_string(),
            semantic_extraction: true,
            rustc_version: "1.75.0".to_string(),
            build_script_outputs: vec![],
            proc_macro_versions: vec![ProcMacroVersion {
                crate_name: "my_proc_macro".to_string(),
                version: "1.0.0".to_string(),
                expanded_token_hash: Some("hash123".to_string()),
            }],
        };

        assert_eq!(config.proc_macro_versions.len(), 1);
        assert_eq!(config.proc_macro_versions[0].crate_name, "my_proc_macro");
    }

    #[test]
    fn test_rust_build_config_default_empty_scripts_and_macros() {
        let config = RustBuildConfig::default();
        assert!(config.build_script_outputs.is_empty());
        assert!(config.proc_macro_versions.is_empty());
    }
}
