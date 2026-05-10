//! Schema integration for C cache
//!
//! Bridges the cache layer (`chimera-c-cache`) with the schema layer (`chimera-c-schema`)
//! to ensure deterministic identity for `.csnap`, `.cdep`, and `.castpack` artifacts.
//!
//! PR 3: Finalize authoritative C semantic artifact identity
//! - lock deterministic schema/version/checksum/compiler identity
//! - ensure include, macro, target, and generated-header inputs are reflected in cache identity

use chimera_c_schema::{
    ArtifactHeader, CastPack, CdepGraph, CsnapSnapshot, CURRENT_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// C semantic artifact identity that ties together .csnap, .cdep, .castpack
///
/// This structure captures the complete identity of C semantic artifacts,
/// ensuring that cache keys are deterministic and include all inputs that
/// affect semantic validity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSemanticArtifactIdentity {
    /// Schema version for the artifact format
    pub schema_version: u32,
    /// Compiler identity (executable, version, triple, sysroot)
    pub compiler_identity: SchemaCompilerIdentity,
    /// Target triple
    pub target_triple: String,
    /// C standard used
    pub c_standard: String,
    /// Compile flags hash
    pub compile_flags_hash: String,
    /// Include graph hash (all headers)
    pub include_graph_hash: String,
    /// Macro state hash (active macros, conditional branches)
    pub macro_state_hash: String,
    /// Generated header hash (if any)
    pub generated_header_hash: Option<String>,
    /// Semantic snapshot checksum (.csnap)
    pub csnap_checksum: Option<String>,
    /// Dependency graph checksum (.cdep)
    pub cdep_checksum: Option<String>,
    /// AST/type/layout package checksum (.castpack)
    pub castpack_checksum: Option<String>,
}

impl CSemanticArtifactIdentity {
    /// Create a new semantic artifact identity
    pub fn new(
        compiler_identity: SchemaCompilerIdentity,
        target_triple: String,
        c_standard: String,
        compile_flags_hash: String,
        include_graph_hash: String,
        macro_state_hash: String,
    ) -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            compiler_identity,
            target_triple,
            c_standard,
            compile_flags_hash,
            include_graph_hash,
            macro_state_hash,
            generated_header_hash: None,
            csnap_checksum: None,
            cdep_checksum: None,
            castpack_checksum: None,
        }
    }

    /// Set the generated header hash
    pub fn with_generated_header(mut self, hash: String) -> Self {
        self.generated_header_hash = Some(hash);
        self
    }

    /// Set the .csnap checksum
    pub fn with_csnap_checksum(mut self, checksum: String) -> Self {
        self.csnap_checksum = Some(checksum);
        self
    }

    /// Set the .cdep checksum
    pub fn with_cdep_checksum(mut self, checksum: String) -> Self {
        self.cdep_checksum = Some(checksum);
        self
    }

    /// Set the .castpack checksum
    pub fn with_castpack_checksum(mut self, checksum: String) -> Self {
        self.castpack_checksum = Some(checksum);
        self
    }

    /// Compute the overall semantic state ID
    ///
    /// This combines all checksums to produce a single ID that changes
    /// when any semantic fact changes.
    pub fn compute_semantic_state_id(&self) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();

        hasher.update(&self.schema_version.to_le_bytes());
        hasher.update(self.compiler_identity.executable.as_bytes());
        hasher.update(self.compiler_identity.version.as_bytes());
        hasher.update(self.target_triple.as_bytes());
        hasher.update(self.c_standard.as_bytes());
        hasher.update(self.compile_flags_hash.as_bytes());
        hasher.update(self.include_graph_hash.as_bytes());
        hasher.update(self.macro_state_hash.as_bytes());

        if let Some(ref h) = self.generated_header_hash {
            hasher.update(h.as_bytes());
        }
        if let Some(ref h) = self.csnap_checksum {
            hasher.update(h.as_bytes());
        }
        if let Some(ref h) = self.cdep_checksum {
            hasher.update(h.as_bytes());
        }
        if let Some(ref h) = self.castpack_checksum {
            hasher.update(h.as_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }

    /// Check if all authoritative artifacts are present
    pub fn has_all_artifacts(&self) -> bool {
        self.csnap_checksum.is_some()
            && self.cdep_checksum.is_some()
            && self.castpack_checksum.is_some()
    }
}

/// Compiler identity for schema integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCompilerIdentity {
    pub executable: String,
    pub version: String,
    pub target_triple: String,
    pub sysroot: Option<String>,
    pub resource_dir: Option<String>,
    pub clang_version: Option<String>,
}

impl SchemaCompilerIdentity {
    /// Create from clang version string
    pub fn from_clang(executable: &str, clang_version: &str, target_triple: &str) -> Self {
        Self {
            executable: executable.to_string(),
            version: clang_version.to_string(),
            target_triple: target_triple.to_string(),
            sysroot: None,
            resource_dir: None,
            clang_version: Some(clang_version.to_string()),
        }
    }

    /// With sysroot
    pub fn with_sysroot(mut self, sysroot: String) -> Self {
        self.sysroot = Some(sysroot);
        self
    }

    /// With resource directory
    pub fn with_resource_dir(mut self, resource_dir: String) -> Self {
        self.resource_dir = Some(resource_dir);
        self
    }

    /// Compute a hash of the compiler identity
    pub fn compute_identity_hash(&self) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();

        hasher.update(self.executable.as_bytes());
        hasher.update(self.version.as_bytes());
        hasher.update(self.target_triple.as_bytes());

        if let Some(ref s) = self.sysroot {
            hasher.update(s.as_bytes());
        }
        if let Some(ref r) = self.resource_dir {
            hasher.update(r.as_bytes());
        }
        if let Some(ref c) = self.clang_version {
            hasher.update(c.as_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }
}

/// Compute the include graph hash from a list of header info
pub fn compute_include_graph_hash<H: HeaderHashInput>(headers: &[H]) -> String {
    use blake3::Hasher;
    let mut hasher = Hasher::new();

    // Collect and sort header paths for determinism
    let mut sorted: Vec<(String, Option<String>)> = headers
        .iter()
        .map(|h| {
            (
                h.path().to_string(),
                h.content_hash().map(|s| s.to_string()),
            )
        })
        .collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    for (path, content_hash) in sorted {
        hasher.update(path.as_bytes());
        if let Some(h) = content_hash {
            hasher.update(h.as_bytes());
        }
    }

    hasher.finalize().to_hex()[..16].to_string()
}

/// Input for computing header hash
pub trait HeaderHashInput {
    fn path(&self) -> &str;
    fn content_hash(&self) -> Option<&str>;
}

impl HeaderHashInput for (&str, Option<&str>) {
    fn path(&self) -> &str {
        self.0
    }
    fn content_hash(&self) -> Option<&str> {
        self.1
    }
}

impl HeaderHashInput for chimera_c_schema::HeaderInfo {
    fn path(&self) -> &str {
        &self.path
    }
    fn content_hash(&self) -> Option<&str> {
        Some(&self.content_hash)
    }
}

/// Compute the macro state hash from active macros and conditional branches
pub fn compute_macro_state_hash<M: AsRef<str>, B: ConditionalBranchInput>(
    active_macros: &[M],
    conditional_branches: &[B],
) -> String {
    use blake3::Hasher;
    let mut hasher = Hasher::new();

    // Sort macros for determinism
    let mut sorted_macros: Vec<String> = active_macros
        .iter()
        .map(|m| m.as_ref().to_string())
        .collect();
    sorted_macros.sort();
    for macro_name in &sorted_macros {
        hasher.update(macro_name.as_bytes());
    }

    // Sort conditional branches by macro name
    let mut sorted_branches: Vec<(String, String)> = conditional_branches
        .iter()
        .map(|b| (b.macro_name().to_string(), b.condition().to_string()))
        .collect();
    sorted_branches.sort();
    for (macro_name, condition) in sorted_branches {
        hasher.update(macro_name.as_bytes());
        hasher.update(condition.as_bytes());
    }

    hasher.finalize().to_hex()[..16].to_string()
}

/// Input for computing conditional branch hash
pub trait ConditionalBranchInput {
    fn macro_name(&self) -> &str;
    fn condition(&self) -> &str;
}

impl ConditionalBranchInput for chimera_c_schema::ConditionalBranch {
    fn macro_name(&self) -> &str {
        &self.macro_name
    }
    fn condition(&self) -> &str {
        &self.condition
    }
}

/// Compute checksum for a .csnap artifact
pub fn compute_csnap_checksum(snapshot: &CsnapSnapshot) -> String {
    snapshot.compute_checksum()
}

/// Compute checksum for a .cdep artifact
pub fn compute_cdep_checksum(graph: &CdepGraph) -> String {
    graph.compute_checksum()
}

/// Compute checksum for a .castpack artifact
pub fn compute_castpack_checksum(pack: &CastPack) -> String {
    pack.compute_checksum()
}

/// Verify that an artifact header is valid for the current schema version
pub fn verify_artifact_header(header: &ArtifactHeader) -> Result<(), SchemaIdentityError> {
    use chimera_c_schema::SchemaError;

    header.validate().map_err(|e| match e {
        SchemaError::InvalidMagic(_) => SchemaIdentityError::InvalidMagic,
        SchemaError::UnsupportedVersion(v) => SchemaIdentityError::UnsupportedVersion(v),
        _ => SchemaIdentityError::Other(e.to_string()),
    })
}

/// Errors for schema identity operations
#[derive(Debug, thiserror::Error)]
pub enum SchemaIdentityError {
    #[error("invalid magic bytes")]
    InvalidMagic,
    #[error("unsupported schema version: {0}")]
    UnsupportedVersion(u32),
    #[error("other schema error: {0}")]
    Other(String),
}

/// Compute deterministic cache key for C artifacts
///
/// This combines all identity components to produce a stable cache key
/// that changes only when semantic content changes.
pub fn compute_artifact_cache_key(identity: &CSemanticArtifactIdentity) -> String {
    use blake3::Hasher;
    let mut hasher = Hasher::new();

    // Schema version
    hasher.update(&identity.schema_version.to_le_bytes());

    // Compiler identity
    hasher.update(
        identity
            .compiler_identity
            .compute_identity_hash()
            .as_bytes(),
    );

    // Target triple
    hasher.update(identity.target_triple.as_bytes());

    // C standard
    hasher.update(identity.c_standard.as_bytes());

    // Compile flags
    hasher.update(identity.compile_flags_hash.as_bytes());

    // Include graph
    hasher.update(identity.include_graph_hash.as_bytes());

    // Macro state
    hasher.update(identity.macro_state_hash.as_bytes());

    // Generated header
    if let Some(ref h) = identity.generated_header_hash {
        hasher.update(h.as_bytes());
    }

    // Semantic checksums
    if let Some(ref h) = identity.csnap_checksum {
        hasher.update(h.as_bytes());
    }
    if let Some(ref h) = identity.cdep_checksum {
        hasher.update(h.as_bytes());
    }
    if let Some(ref h) = identity.castpack_checksum {
        hasher.update(h.as_bytes());
    }

    hasher.finalize().to_hex()[..32].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_compiler_identity() -> SchemaCompilerIdentity {
        SchemaCompilerIdentity::from_clang("clang", "15.0.0", "x86_64-unknown-linux-gnu")
    }

    #[test]
    fn test_semantic_artifact_identity_new() {
        let identity = CSemanticArtifactIdentity::new(
            make_test_compiler_identity(),
            "x86_64-unknown-linux-gnu".to_string(),
            "c11".to_string(),
            "flags_hash".to_string(),
            "include_hash".to_string(),
            "macro_hash".to_string(),
        );

        assert_eq!(identity.schema_version, 1);
        assert!(identity.csnap_checksum.is_none());
        assert!(!identity.has_all_artifacts());
    }

    #[test]
    fn test_semantic_artifact_identity_with_checksums() {
        let identity = CSemanticArtifactIdentity::new(
            make_test_compiler_identity(),
            "x86_64-unknown-linux-gnu".to_string(),
            "c11".to_string(),
            "flags_hash".to_string(),
            "include_hash".to_string(),
            "macro_hash".to_string(),
        )
        .with_csnap_checksum("csnap_hash_abc".to_string())
        .with_cdep_checksum("cdep_hash_def".to_string())
        .with_castpack_checksum("castpack_hash_ghi".to_string());

        assert!(identity.has_all_artifacts());
    }

    #[test]
    fn test_compute_semantic_state_id() {
        let identity = CSemanticArtifactIdentity::new(
            make_test_compiler_identity(),
            "x86_64-unknown-linux-gnu".to_string(),
            "c11".to_string(),
            "flags_hash".to_string(),
            "include_hash".to_string(),
            "macro_hash".to_string(),
        )
        .with_csnap_checksum("csnap_hash_abc".to_string())
        .with_cdep_checksum("cdep_hash_def".to_string())
        .with_castpack_checksum("castpack_hash_ghi".to_string());

        let state_id = identity.compute_semantic_state_id();
        assert_eq!(state_id.len(), 64); // blake3 hex
    }

    #[test]
    fn test_semantic_state_id_changes_with_csnap() {
        let identity1 = CSemanticArtifactIdentity::new(
            make_test_compiler_identity(),
            "x86_64-unknown-linux-gnu".to_string(),
            "c11".to_string(),
            "flags_hash".to_string(),
            "include_hash".to_string(),
            "macro_hash".to_string(),
        )
        .with_csnap_checksum("csnap_hash_v1".to_string());

        let identity2 = CSemanticArtifactIdentity::new(
            make_test_compiler_identity(),
            "x86_64-unknown-linux-gnu".to_string(),
            "c11".to_string(),
            "flags_hash".to_string(),
            "include_hash".to_string(),
            "macro_hash".to_string(),
        )
        .with_csnap_checksum("csnap_hash_v2".to_string());

        assert_ne!(
            identity1.compute_semantic_state_id(),
            identity2.compute_semantic_state_id()
        );
    }

    #[test]
    fn test_schema_compiler_identity_hash() {
        let id1 = make_test_compiler_identity();
        let id2 = SchemaCompilerIdentity::from_clang("clang", "15.0.0", "x86_64-unknown-linux-gnu");

        assert_eq!(id1.compute_identity_hash(), id2.compute_identity_hash());
    }

    #[test]
    fn test_schema_compiler_identity_different_version() {
        let id1 = SchemaCompilerIdentity::from_clang("clang", "15.0.0", "x86_64-unknown-linux-gnu");
        let id2 = SchemaCompilerIdentity::from_clang("clang", "16.0.0", "x86_64-unknown-linux-gnu");

        assert_ne!(id1.compute_identity_hash(), id2.compute_identity_hash());
    }

    #[test]
    fn test_schema_compiler_identity_with_sysroot() {
        let id1 = SchemaCompilerIdentity::from_clang("clang", "15.0.0", "x86_64-unknown-linux-gnu")
            .with_sysroot("/usr/lib/clang/15".to_string());

        let id2 = SchemaCompilerIdentity::from_clang("clang", "15.0.0", "x86_64-unknown-linux-gnu")
            .with_sysroot("/usr/lib/clang/16".to_string());

        assert_ne!(id1.compute_identity_hash(), id2.compute_identity_hash());
    }

    #[test]
    fn test_compute_include_graph_hash() {
        #[derive(Clone)]
        struct TestHeader<'a>(&'a str, Option<&'a str>);

        impl HeaderHashInput for TestHeader<'_> {
            fn path(&self) -> &str {
                self.0
            }
            fn content_hash(&self) -> Option<&str> {
                self.1
            }
        }

        let headers = vec![
            TestHeader("/usr/include/stdio.h", Some("hash_stdio")),
            TestHeader("/usr/include/stdlib.h", Some("hash_stdlib")),
        ];

        let hash1 = compute_include_graph_hash(&headers);
        let hash2 = compute_include_graph_hash(&headers);

        // Same input produces same hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 16);
    }

    #[test]
    fn test_compute_include_graph_hash_deterministic_order() {
        #[derive(Clone)]
        struct TestHeader<'a>(&'a str, Option<&'a str>);

        impl HeaderHashInput for TestHeader<'_> {
            fn path(&self) -> &str {
                self.0
            }
            fn content_hash(&self) -> Option<&str> {
                self.1
            }
        }

        // Same headers in different order should produce same hash
        let headers1 = vec![
            TestHeader("/usr/include/stdio.h", Some("hash_stdio")),
            TestHeader("/usr/include/stdlib.h", Some("hash_stdlib")),
        ];

        let headers2 = vec![
            TestHeader("/usr/include/stdlib.h", Some("hash_stdlib")),
            TestHeader("/usr/include/stdio.h", Some("hash_stdio")),
        ];

        let hash1 = compute_include_graph_hash(&headers1);
        let hash2 = compute_include_graph_hash(&headers2);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_macro_state_hash() {
        let active_macros = vec!["MAX", "BUFFER_SIZE"];

        struct TestBranch<'a>(&'a str, &'a str);

        impl ConditionalBranchInput for TestBranch<'_> {
            fn macro_name(&self) -> &str {
                self.0
            }
            fn condition(&self) -> &str {
                self.1
            }
        }

        let conditional_branches = vec![TestBranch("__x86_64__", "__x86_64__")];

        let hash = compute_macro_state_hash(&active_macros, &conditional_branches);
        assert_eq!(hash.len(), 16);
    }

    #[test]
    fn test_compute_macro_state_hash_deterministic() {
        struct TestBranch<'a>(&'a str, &'a str);

        impl ConditionalBranchInput for TestBranch<'_> {
            fn macro_name(&self) -> &str {
                self.0
            }
            fn condition(&self) -> &str {
                self.1
            }
        }

        let macros1 = vec!["MAX", "BUFFER_SIZE"];
        let branches1: Vec<TestBranch> = vec![];

        let macros2 = vec!["BUFFER_SIZE", "MAX"]; // Different order
        let branches2: Vec<TestBranch> = vec![];

        let hash1 = compute_macro_state_hash(&macros1, &branches1);
        let hash2 = compute_macro_state_hash(&macros2, &branches2);

        // Sorted order produces same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_artifact_cache_key() {
        let identity = CSemanticArtifactIdentity::new(
            make_test_compiler_identity(),
            "x86_64-unknown-linux-gnu".to_string(),
            "c11".to_string(),
            "flags_hash".to_string(),
            "include_hash".to_string(),
            "macro_hash".to_string(),
        )
        .with_csnap_checksum("csnap_hash".to_string())
        .with_cdep_checksum("cdep_hash".to_string())
        .with_castpack_checksum("castpack_hash".to_string());

        let cache_key = compute_artifact_cache_key(&identity);
        assert_eq!(cache_key.len(), 32); // blake3 hex (16 bytes = 32 hex chars)
    }

    #[test]
    fn test_compute_artifact_cache_key_deterministic() {
        let identity1 = CSemanticArtifactIdentity::new(
            make_test_compiler_identity(),
            "x86_64-unknown-linux-gnu".to_string(),
            "c11".to_string(),
            "flags_hash".to_string(),
            "include_hash".to_string(),
            "macro_hash".to_string(),
        )
        .with_csnap_checksum("csnap_hash".to_string());

        let identity2 = CSemanticArtifactIdentity::new(
            make_test_compiler_identity(),
            "x86_64-unknown-linux-gnu".to_string(),
            "c11".to_string(),
            "flags_hash".to_string(),
            "include_hash".to_string(),
            "macro_hash".to_string(),
        )
        .with_csnap_checksum("csnap_hash".to_string());

        assert_eq!(
            compute_artifact_cache_key(&identity1),
            compute_artifact_cache_key(&identity2)
        );
    }

    #[test]
    fn test_verify_artifact_header_valid() {
        let header = ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0");
        assert!(verify_artifact_header(&header).is_ok());
    }

    #[test]
    fn test_verify_artifact_header_invalid_magic() {
        let header = ArtifactHeader {
            magic: [0, 0, 0, 0],
            schema_version: 1,
            producer_version: "0.1.0".to_string(),
            target: "x86_64-unknown-linux-gnu".to_string(),
            source_language: "c".to_string(),
        };
        assert!(matches!(
            verify_artifact_header(&header),
            Err(SchemaIdentityError::InvalidMagic)
        ));
    }

    #[test]
    fn test_with_generated_header() {
        let identity = CSemanticArtifactIdentity::new(
            make_test_compiler_identity(),
            "x86_64-unknown-linux-gnu".to_string(),
            "c11".to_string(),
            "flags_hash".to_string(),
            "include_hash".to_string(),
            "macro_hash".to_string(),
        )
        .with_generated_header("generated_hash".to_string());

        assert!(identity.generated_header_hash.is_some());
        assert_eq!(identity.generated_header_hash.unwrap(), "generated_hash");
    }

    #[test]
    fn test_identity_includes_all_artifacts_check() {
        let mut identity = CSemanticArtifactIdentity::new(
            make_test_compiler_identity(),
            "x86_64-unknown-linux-gnu".to_string(),
            "c11".to_string(),
            "flags_hash".to_string(),
            "include_hash".to_string(),
            "macro_hash".to_string(),
        );

        // No artifacts yet
        assert!(!identity.has_all_artifacts());

        identity.csnap_checksum = Some("csnap".to_string());
        assert!(!identity.has_all_artifacts());

        identity.cdep_checksum = Some("cdep".to_string());
        assert!(!identity.has_all_artifacts());

        identity.castpack_checksum = Some("castpack".to_string());
        assert!(identity.has_all_artifacts());
    }
}
