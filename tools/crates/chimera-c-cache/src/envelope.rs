//! C Build Result Envelope for chimerair integration
//!
//! This module defines the canonical C build result envelope that `chimerair`
//! consumes to determine stale/reusable artifact sets and downstream invalidation.
//!
//! # Design
//!
//! The envelope is language-agnostic at the `chimerair` level, but contains
//! C-specific fields for C-origin modules.
//!
//! # Exit Criteria (from c-incremental-ownership-plan.md)
//!
//! - semantic state ID
//! - stale/reusable artifact sets
//! - public-surface summary
//! - explanation records

use crate::CompilerIdentity;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Build status for the C frontend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CBuildStatus {
    /// Build completed successfully with all artifacts
    Success,
    /// Build completed but fell back to surface-only mode (non-authoritative)
    FallbackSurfaceOnly,
    /// Build failed
    Failed,
}

/// Kind of invalidation detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CInvalidationKind {
    /// Implementation-only source change (no downstream effect)
    ImplementationOnly,
    /// Header surface changed (affects consumers)
    HeaderSurface,
    /// Macro or conditional-compilation changed
    MacroCondition,
    /// Layout changed (affects ABI)
    Layout,
    /// Compiler, target, or sysroot changed
    CompilerTarget,
    /// Generated-header changed
    GeneratedHeader,
}

/// Explanation of why a build result was produced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CBuildExplanation {
    /// What kind of invalidation was detected
    pub invalidation_kind: CInvalidationKind,
    /// Which nodes in the dependency graph changed
    pub changed_nodes: Vec<String>,
    /// Whether this change affects downstream consumers
    pub affects_downstream: bool,
    /// Plaintext explanation suitable for display
    pub human_readable: String,
    /// Files that triggered invalidation
    pub invalidated_files: Vec<PathBuf>,
}

/// A stale or reusable artifact reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CArtifactRef {
    /// Unique stable ID for this artifact
    pub stable_id: String,
    /// What kind of artifact this is
    pub kind: CArtifactKind,
    /// Path where the artifact is stored (if cached)
    pub path: Option<PathBuf>,
    /// Fingerprint of the artifact content
    pub fingerprint: String,
}

/// Kind of C artifact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CArtifactKind {
    /// Semantic snapshot (.csnap)
    Csnap,
    /// Dependency graph (.cdep)
    Cdep,
    /// AST/type/layout package (.castpack)
    CastPack,
    /// C dialect context (.cdialect)
    CDialect,
    /// C metadata (.chmeta)
    Cmeta,
    /// C proof facts (.cproof)
    Cproof,
    /// Object file (.o)
    Object,
    /// Wrapper file
    Wrapper,
    /// Link artifact
    Link,
}

impl CArtifactKind {
    /// Get file extension for this artifact kind
    pub fn extension(&self) -> &'static str {
        match self {
            CArtifactKind::Csnap => ".csnap",
            CArtifactKind::Cdep => ".cdep",
            CArtifactKind::CastPack => ".castpack",
            CArtifactKind::CDialect => ".cdialect",
            CArtifactKind::Cmeta => ".chmeta",
            CArtifactKind::Cproof => ".cproof",
            CArtifactKind::Object => ".o",
            CArtifactKind::Wrapper => ".chwrap",
            CArtifactKind::Link => ".link",
        }
    }

    /// Check if this artifact kind is part of the authoritative semantic path
    pub fn is_authoritative(&self) -> bool {
        matches!(
            self,
            CArtifactKind::Csnap | CArtifactKind::Cdep | CArtifactKind::CastPack
        )
    }
}

/// Public surface summary from a C translation unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPublicSurface {
    /// Exported function declarations
    pub exported_functions: Vec<CExportSummary>,
    /// Exported global variables
    pub exported_globals: Vec<CExportSummary>,
    /// ABI-affecting types (structs, enums, unions)
    pub abi_types: Vec<CTypeSummary>,
    /// Layout-affecting items
    pub layout_items: Vec<String>,
    /// Header files that define the public surface
    pub public_headers: Vec<PathBuf>,
}

/// Summary of an exported item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CExportSummary {
    /// Symbol name
    pub symbol: String,
    /// Declaration location
    pub declaration_file: PathBuf,
    pub declaration_line: u32,
    /// ABI of this export (e.g., "C", "stdcall")
    pub abi: String,
    /// Signature as string
    pub signature: String,
    /// Whether this affects layout
    pub affects_layout: bool,
}

/// Summary of a type that affects ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CTypeSummary {
    /// Type name (struct, enum, union tag)
    pub name: String,
    /// Definition location
    pub definition_file: PathBuf,
    pub definition_line: u32,
    /// Size and alignment if known
    pub size_bytes: Option<u64>,
    pub alignment_bytes: Option<u32>,
    /// Whether layout is affected by changes
    pub affects_layout: bool,
}

/// The canonical C build result envelope
///
/// This is the main output of the C frontend that `chimerair` consumes
/// to make scheduling and invalidation decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CBuildResult {
    /// Schema version for this envelope format
    pub schema_version: String,
    /// Semantic state identifier (changes when any semantic fact changes)
    pub semantic_state_id: String,
    /// Build status
    pub status: CBuildStatus,
    /// Explanation of the build result
    pub explanation: Option<CBuildExplanation>,
    /// Artifacts that were produced and are reusable
    pub reusable_artifacts: Vec<CArtifactRef>,
    /// Artifacts that are stale and need rebuild
    pub stale_artifacts: Vec<CArtifactRef>,
    /// Public surface summary
    pub public_surface: Option<CPublicSurface>,
    /// Compiler identity used for this build
    pub compiler_identity: CompilerIdentity,
    /// Target triple
    pub target_triple: String,
    /// Compile flags used
    pub compile_flags: Vec<String>,
    /// Include graph hash
    pub include_graph_hash: String,
    /// Whether this was an authoritative (Clang-derived) build
    pub is_authoritative: bool,
    /// Fingerprint of the complete build inputs
    pub build_input_fingerprint: String,
    /// Timestamp of the build
    pub timestamp_secs: u64,
}

impl CBuildResult {
    /// Create a new C build result envelope
    pub fn new(
        compiler_identity: CompilerIdentity,
        target_triple: String,
        compile_flags: Vec<String>,
        include_graph_hash: String,
        is_authoritative: bool,
        semantic_state_id: String,
        build_input_fingerprint: String,
    ) -> Self {
        Self {
            schema_version: "0.1.0".to_string(),
            semantic_state_id,
            status: CBuildStatus::Success,
            explanation: None,
            reusable_artifacts: Vec::new(),
            stale_artifacts: Vec::new(),
            public_surface: None,
            compiler_identity,
            target_triple,
            compile_flags,
            include_graph_hash,
            is_authoritative,
            build_input_fingerprint,
            timestamp_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    /// Set the build status
    pub fn with_status(mut self, status: CBuildStatus) -> Self {
        self.status = status;
        self
    }

    /// Set the explanation
    pub fn with_explanation(mut self, explanation: CBuildExplanation) -> Self {
        self.explanation = Some(explanation);
        self
    }

    /// Add a reusable artifact
    pub fn add_reusable(mut self, artifact: CArtifactRef) -> Self {
        self.reusable_artifacts.push(artifact);
        self
    }

    /// Add a stale artifact
    pub fn add_stale(mut self, artifact: CArtifactRef) -> Self {
        self.stale_artifacts.push(artifact);
        self
    }

    /// Set the public surface
    pub fn with_public_surface(mut self, surface: CPublicSurface) -> Self {
        self.public_surface = Some(surface);
        self
    }

    /// Compute the semantic state ID from artifacts
    pub fn compute_semantic_state_id(
        csnap_hash: Option<&str>,
        cdep_hash: Option<&str>,
        castpack_hash: Option<&str>,
    ) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();

        if let Some(h) = csnap_hash {
            hasher.update(h.as_bytes());
        }
        if let Some(h) = cdep_hash {
            hasher.update(h.as_bytes());
        }
        if let Some(h) = castpack_hash {
            hasher.update(h.as_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }

    /// Check if this is a no-op rebuild (all artifacts reusable)
    pub fn is_noop_rebuild(&self) -> bool {
        self.stale_artifacts.is_empty() && !self.reusable_artifacts.is_empty()
    }

    /// Check if this build has any reusable C artifacts
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
        compiler_identity: CompilerIdentity,
        target_triple: String,
        compile_flags: Vec<String>,
        include_graph_hash: String,
        build_input_fingerprint: String,
    ) -> Self {
        Self::new(
            compiler_identity,
            target_triple,
            compile_flags,
            include_graph_hash,
            false,
            String::new(),
            build_input_fingerprint,
        )
        .with_status(CBuildStatus::FallbackSurfaceOnly)
    }

    /// Check if the build used the authoritative Clang path
    pub fn is_clang_authoritative(&self) -> bool {
        self.is_authoritative && self.status == CBuildStatus::Success
    }
}

/// Compute fingerprint for build inputs
pub fn compute_build_input_fingerprint(
    source_files: &[&PathBuf],
    compile_flags: &[String],
    include_hashes: &[String],
    compiler_identity: &CompilerIdentity,
    target_triple: &str,
) -> String {
    use blake3::Hasher;
    let mut hasher = Hasher::new();

    for file in source_files {
        hasher.update(file.to_string_lossy().as_bytes());
    }

    for flag in compile_flags {
        hasher.update(flag.as_bytes());
    }

    for h in include_hashes {
        hasher.update(h.as_bytes());
    }

    hasher.update(compiler_identity.executable.as_bytes());
    hasher.update(compiler_identity.version.as_bytes());
    hasher.update(target_triple.as_bytes());

    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_compiler_identity() -> CompilerIdentity {
        CompilerIdentity::new("clang", "15.0.0", "x86_64-unknown-linux-gnu")
    }

    #[test]
    fn test_build_result_new() {
        let compiler = make_test_compiler_identity();
        let result = CBuildResult::new(
            compiler,
            "x86_64-unknown-linux-gnu".to_string(),
            vec!["-O2".to_string()],
            "include_hash_abc".to_string(),
            true,
            "state_abc123".to_string(),
            "input_fp".to_string(),
        );

        assert_eq!(result.target_triple, "x86_64-unknown-linux-gnu");
        assert_eq!(result.semantic_state_id, "state_abc123");
        assert!(result.is_authoritative);
        assert_eq!(result.status, CBuildStatus::Success);
    }

    #[test]
    fn test_build_result_add_artifacts() {
        let compiler = make_test_compiler_identity();
        let result = CBuildResult::new(
            compiler,
            "x86_64-unknown-linux-gnu".to_string(),
            vec![],
            "hash".to_string(),
            true,
            "state_abc".to_string(),
            "input_fp".to_string(),
        )
        .add_reusable(CArtifactRef {
            stable_id: "csnap_0".to_string(),
            kind: CArtifactKind::Csnap,
            path: Some(PathBuf::from(".cache/csnap_0.csnap")),
            fingerprint: "fp_csnap".to_string(),
        })
        .add_stale(CArtifactRef {
            stable_id: "object_0".to_string(),
            kind: CArtifactKind::Object,
            path: Some(PathBuf::from("build/object_0.o")),
            fingerprint: "fp_obj".to_string(),
        });

        assert_eq!(result.reusable_artifacts.len(), 1);
        assert_eq!(result.stale_artifacts.len(), 1);
    }

    #[test]
    fn test_build_result_is_noop() {
        let compiler = make_test_compiler_identity();
        let result = CBuildResult::new(
            compiler,
            "x86_64-unknown-linux-gnu".to_string(),
            vec![],
            "hash".to_string(),
            true,
            "state".to_string(),
            "input".to_string(),
        )
        .add_reusable(CArtifactRef {
            stable_id: "csnap_0".to_string(),
            kind: CArtifactKind::Csnap,
            path: Some(PathBuf::from(".cache/csnap_0.csnap")),
            fingerprint: "fp".to_string(),
        });

        assert!(result.is_noop_rebuild());
    }

    #[test]
    fn test_build_result_not_noop_with_stale() {
        let compiler = make_test_compiler_identity();
        let result = CBuildResult::new(
            compiler,
            "x86_64-unknown-linux-gnu".to_string(),
            vec![],
            "hash".to_string(),
            true,
            "state".to_string(),
            "input".to_string(),
        )
        .add_stale(CArtifactRef {
            stable_id: "object_0".to_string(),
            kind: CArtifactKind::Object,
            path: None,
            fingerprint: "fp".to_string(),
        });

        assert!(!result.is_noop_rebuild());
    }

    #[test]
    fn test_build_result_fallback() {
        let compiler = make_test_compiler_identity();
        let result = CBuildResult::fallback_surface_only(
            compiler,
            "aarch64-apple-darwin".to_string(),
            vec!["-O2".to_string()],
            "include_hash".to_string(),
            "fallback_fp".to_string(),
        );

        assert_eq!(result.status, CBuildStatus::FallbackSurfaceOnly);
        assert!(!result.is_authoritative);
    }

    #[test]
    fn test_invalidation_kind_serialization() {
        let kinds = vec![
            CInvalidationKind::ImplementationOnly,
            CInvalidationKind::HeaderSurface,
            CInvalidationKind::MacroCondition,
            CInvalidationKind::Layout,
            CInvalidationKind::CompilerTarget,
            CInvalidationKind::GeneratedHeader,
        ];

        for kind in kinds {
            let json = serde_json::to_string(&kind).unwrap();
            let parsed: CInvalidationKind = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, kind);
        }
    }

    #[test]
    fn test_artifact_kind_extensions() {
        assert_eq!(CArtifactKind::Csnap.extension(), ".csnap");
        assert_eq!(CArtifactKind::Cdep.extension(), ".cdep");
        assert_eq!(CArtifactKind::CastPack.extension(), ".castpack");
        assert_eq!(CArtifactKind::CDialect.extension(), ".cdialect");
        assert_eq!(CArtifactKind::Cmeta.extension(), ".chmeta");
        assert_eq!(CArtifactKind::Cproof.extension(), ".cproof");
        assert_eq!(CArtifactKind::Object.extension(), ".o");
        assert_eq!(CArtifactKind::Wrapper.extension(), ".chwrap");
        assert_eq!(CArtifactKind::Link.extension(), ".link");
    }

    #[test]
    fn test_artifact_kind_is_authoritative() {
        assert!(CArtifactKind::Csnap.is_authoritative());
        assert!(CArtifactKind::Cdep.is_authoritative());
        assert!(CArtifactKind::CastPack.is_authoritative());
        assert!(!CArtifactKind::Object.is_authoritative());
        assert!(!CArtifactKind::Wrapper.is_authoritative());
    }

    #[test]
    fn test_compute_semantic_state_id() {
        let state_id = CBuildResult::compute_semantic_state_id(
            Some("csnap_hash"),
            Some("cdep_hash"),
            Some("castpack_hash"),
        );
        assert_eq!(state_id.len(), 64); // blake3 hex
    }

    #[test]
    fn test_compute_semantic_state_id_partial() {
        // With only csnap
        let state_id = CBuildResult::compute_semantic_state_id(Some("csnap_hash"), None, None);
        assert_eq!(state_id.len(), 64);

        // With none
        let state_id = CBuildResult::compute_semantic_state_id(None, None, None);
        assert_eq!(state_id.len(), 64); // Empty hasher still produces output
    }

    #[test]
    fn test_all_artifact_ids() {
        let compiler = make_test_compiler_identity();
        let result = CBuildResult::new(
            compiler,
            "x86_64-unknown-linux-gnu".to_string(),
            vec![],
            "hash".to_string(),
            true,
            "state".to_string(),
            "input".to_string(),
        )
        .add_reusable(CArtifactRef {
            stable_id: "reusable_0".to_string(),
            kind: CArtifactKind::Csnap,
            path: None,
            fingerprint: "fp_0".to_string(),
        })
        .add_stale(CArtifactRef {
            stable_id: "stale_0".to_string(),
            kind: CArtifactKind::Object,
            path: None,
            fingerprint: "fp_1".to_string(),
        });

        let ids = result.all_artifact_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"reusable_0".to_string()));
        assert!(ids.contains(&"stale_0".to_string()));
    }

    #[test]
    fn test_explanation_serialization() {
        let explanation = CBuildExplanation {
            invalidation_kind: CInvalidationKind::Layout,
            changed_nodes: vec!["struct_Point".to_string(), "type_size_0".to_string()],
            affects_downstream: true,
            human_readable: "Layout of Point struct changed".to_string(),
            invalidated_files: vec![PathBuf::from("types.h")],
        };

        let json = serde_json::to_string_pretty(&explanation).unwrap();
        let parsed: CBuildExplanation = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.invalidation_kind, CInvalidationKind::Layout);
        assert_eq!(parsed.changed_nodes.len(), 2);
        assert_eq!(parsed.invalidated_files.len(), 1);
    }

    #[test]
    fn test_public_surface_serialization() {
        let surface = CPublicSurface {
            exported_functions: vec![CExportSummary {
                symbol: "add".to_string(),
                declaration_file: PathBuf::from("math.h"),
                declaration_line: 10,
                abi: "C".to_string(),
                signature: "int add(int, int)".to_string(),
                affects_layout: false,
            }],
            exported_globals: vec![],
            abi_types: vec![CTypeSummary {
                name: "Point".to_string(),
                definition_file: PathBuf::from("types.h"),
                definition_line: 5,
                size_bytes: Some(8),
                alignment_bytes: Some(4),
                affects_layout: true,
            }],
            layout_items: vec!["struct Point".to_string()],
            public_headers: vec![PathBuf::from("types.h"), PathBuf::from("math.h")],
        };

        let json = serde_json::to_string(&surface).unwrap();
        let parsed: CPublicSurface = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.exported_functions.len(), 1);
        assert_eq!(parsed.abi_types.len(), 1);
        assert_eq!(parsed.public_headers.len(), 2);
    }

    #[test]
    fn test_build_result_roundtrip() {
        let compiler = make_test_compiler_identity();
        let result = CBuildResult::new(
            compiler,
            "x86_64-unknown-linux-gnu".to_string(),
            vec!["-O2".to_string(), "-Wall".to_string()],
            "include_hash_xyz".to_string(),
            true,
            "sem_state_abc".to_string(),
            "build_input_fp".to_string(),
        )
        .with_status(CBuildStatus::Success)
        .with_explanation(CBuildExplanation {
            invalidation_kind: CInvalidationKind::ImplementationOnly,
            changed_nodes: vec!["func_impl".to_string()],
            affects_downstream: false,
            human_readable: "Only implementation body changed".to_string(),
            invalidated_files: vec![PathBuf::from("impl.c")],
        })
        .add_reusable(CArtifactRef {
            stable_id: "csnap_0".to_string(),
            kind: CArtifactKind::Csnap,
            path: Some(PathBuf::from(".cache/csnap_0.csnap")),
            fingerprint: "fp_csnap".to_string(),
        });

        let json = serde_json::to_string(&result).unwrap();
        let parsed: CBuildResult = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.semantic_state_id, "sem_state_abc");
        assert_eq!(parsed.status, CBuildStatus::Success);
        assert!(parsed.explanation.is_some());
        assert_eq!(parsed.reusable_artifacts.len(), 1);
        assert!(parsed.is_authoritative);
    }

    #[test]
    fn test_build_result_is_clang_authoritative() {
        let compiler = make_test_compiler_identity();

        // Authoritative + Success = true
        let result = CBuildResult::new(
            compiler.clone(),
            "x86_64-unknown-linux-gnu".to_string(),
            vec![],
            "hash".to_string(),
            true,
            "state".to_string(),
            "input".to_string(),
        );
        assert!(result.is_clang_authoritative());

        // Authoritative + Fallback = false (not truly authoritative)
        let result = CBuildResult::fallback_surface_only(
            compiler.clone(),
            "x86_64-unknown-linux-gnu".to_string(),
            vec![],
            "hash".to_string(),
            "input".to_string(),
        );
        assert!(!result.is_clang_authoritative());

        // Non-authoritative = false
        let result = CBuildResult::new(
            compiler,
            "x86_64-unknown-linux-gnu".to_string(),
            vec![],
            "hash".to_string(),
            false,
            "state".to_string(),
            "input".to_string(),
        );
        assert!(!result.is_clang_authoritative());
    }

    #[test]
    fn test_compute_build_input_fingerprint() {
        let compiler = make_test_compiler_identity();
        let main_c = PathBuf::from("src/main.c");
        let types_h = PathBuf::from("src/types.h");
        let sources = vec![&main_c, &types_h];
        let flags = vec!["-O2".to_string(), "-Wall".to_string()];
        let include_hashes = vec!["hash1".to_string(), "hash2".to_string()];

        let fp = compute_build_input_fingerprint(
            &sources,
            &flags,
            &include_hashes,
            &compiler,
            "x86_64-unknown-linux-gnu",
        );

        assert_eq!(fp.len(), 64); // blake3 hex
    }

    #[test]
    fn test_has_reusable_artifacts() {
        let compiler = make_test_compiler_identity();
        let result = CBuildResult::new(
            compiler,
            "x86_64-unknown-linux-gnu".to_string(),
            vec![],
            "hash".to_string(),
            true,
            "state".to_string(),
            "input".to_string(),
        );

        assert!(!result.has_reusable_artifacts());

        let result = result.add_reusable(CArtifactRef {
            stable_id: "csnap_0".to_string(),
            kind: CArtifactKind::Csnap,
            path: Some(PathBuf::from(".cache/csnap_0.csnap")),
            fingerprint: "fp".to_string(),
        });

        assert!(result.has_reusable_artifacts());
    }
}
