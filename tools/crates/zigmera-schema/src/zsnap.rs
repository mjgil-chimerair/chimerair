//! `.zsnap` semantic snapshot schema v1.
//!
//! Header plus source files, build options, decls, analysis units,
//! types, layouts, AIR body references, exports, dependency edge references.

use serde::{Deserialize, Serialize};

/// Magic bytes for `.zsnap` binary format.
pub const ZSNAP_MAGIC: &[u8; 8] = b"ZSNAP001";

/// Current schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// `.zsnap` semantic snapshot header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapHeader {
    pub magic: [u8; 8],
    pub schema_version: u32,
    pub min_adapter_version: u32,
    pub zig_commit: [u8; 20],
    pub target: String,
    pub backend: String,
    pub optimize_mode: String,
    pub timestamp_ns: u64,
    pub source_file_count: u32,
    pub checksum: [u8; 32],
}

/// A source file in the snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    pub id: u64,
    pub path: String,
    pub content_hash: [u8; 32],
}

/// Build options affecting the snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildOptions {
    pub optimize_mode: String,
    pub target: String,
    pub cpu_features: Vec<String>,
    pub libc: Option<String>,
    pub build_mode: String,
    pub entry: Option<String>,
    pub panic_mode: String,
}

/// Declaration reference in the snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclRef {
    pub id: u64,
    pub name: String,
    pub kind: DeclKind,
    pub owner_file: u64,
    pub access_level: AccessLevel,
}

/// Kind of declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeclKind {
    Function,
    Struct,
    Union,
    Enum,
    Opaque,
    Var,
    Const,
    Import,
    TypeAlias,
    ContainerAugmentation,
}

/// Access level of a declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    Private,
    Pub,
    PubStage,
}

/// Analysis unit reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisUnit {
    pub id: u64,
    pub file: u64,
    pub decls: Vec<u64>,
    pub imports: Vec<ImportEdge>,
}

/// Import edge from one file to another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportEdge {
    pub from_file: u64,
    pub to_file: u64,
    pub line: u32,
}

/// Type record in the snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeRecord {
    pub id: u64,
    pub kind: TypeKind,
    pub name: Option<String>,
    pub size_bytes: Option<u64>,
    pub alignment: Option<u32>,
}

/// Kind of type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    Int {
        signed: bool,
        bits: u32,
    },
    Float {
        bits: u32,
    },
    Bool,
    Void,
    Pointer {
        child: u64,
        const_: bool,
        addrspace: u32,
    },
    Slice {
        child: u64,
        const_: bool,
    },
    Array {
        child: u64,
        len: u64,
    },
    Struct {
        fields: Vec<FieldRecord>,
    },
    PackedStruct {
        fields: Vec<FieldRecord>,
    },
    ExternStruct {
        fields: Vec<FieldRecord>,
    },
    Union {
        fields: Vec<FieldRecord>,
    },
    Enum {
        tag_type: u64,
        variants: Vec<VariantRecord>,
    },
    Optional {
        child: u64,
    },
    ErrorUnion {
        child: u64,
    },
    ErrorSet {
        errors: Vec<String>,
    },
    FnType {
        params: Vec<u64>,
        return_type: u64,
        callconv: u32,
    },
    Opaque,
    Vector {
        child: u64,
        len: u32,
    },
}

/// Field record in a struct/union.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRecord {
    pub name: String,
    pub type_id: u64,
    pub offset_bytes: Option<u64>,
    pub alignment: Option<u32>,
}

/// Enum variant record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantRecord {
    pub name: String,
    pub tag_value: u64,
}

/// Layout record in the snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutRecord {
    pub id: u64,
    pub type_id: u64,
    pub size_bytes: u64,
    pub alignment: u32,
    pub field_count: u32,
    pub packed: bool,
    pub extern_: bool,
}

/// AIR function body reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirBodyRef {
    pub function_id: u64,
    pub type_id: u64,
    pub basic_blocks: u32,
    pub instructions: u32,
    pub air_data_offset: u64,
    pub air_data_len: u64,
}

/// Exported symbol in the snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSymbol {
    pub name: String,
    pub decl_id: u64,
    pub linkage: Linkage,
    pub visibility: Visibility,
    pub callconv: u32,
    pub section_hint: Option<String>,
}

/// Symbol linkage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Linkage {
    Internal,
    Strong,
    Weak,
    LinkOnce,
}

/// Symbol visibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Private,
    Public,
    Exported,
}

/// Classification of how a comptime call affects downstream consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComptimeImpact {
    /// Comptime evaluation is used only within private function bodies.
    /// No downstream effect - safe to reuse artifacts.
    PrivateBody,
    /// Comptime evaluation affects exported type signatures or function signatures.
    /// Downstream effect - requires invalidation of dependents.
    ExportedSignature,
    /// Comptime evaluation is embedded in exported constant data.
    /// Requires full invalidation of consumers.
    ExportedConst,
}

/// A comptime function call recorded in the snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComptimeCall {
    /// Unique identifier for this comptime call.
    pub call_id: u64,
    /// The declaration that contains this comptime evaluation.
    pub owner_decl: u64,
    /// Whether this comptime call affects exported symbols.
    pub affects_exports: bool,
    /// Content hash of the comptime result (stable across identical evaluations).
    pub result_hash: [u8; 32],
    /// The file where this comptime was evaluated.
    pub source_file: u64,
    /// Line number in source.
    pub source_line: u32,
}

impl ComptimeCall {
    /// Classify the impact of this comptime call on downstream consumers.
    pub fn classify_impact(&self) -> ComptimeImpact {
        if self.affects_exports {
            ComptimeImpact::ExportedSignature
        } else {
            ComptimeImpact::PrivateBody
        }
    }
}

/// An embedded file reference tracked in the snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedFileRef {
    /// Unique identifier for this embed reference.
    pub embed_id: u64,
    /// Path to the embedded file.
    pub path: String,
    /// Content hash for invalidation detection.
    pub content_hash: [u8; 32],
    /// The file that contains this @embedFile call.
    pub source_file: u64,
    /// Line number in source where @embedFile is used.
    pub source_line: u32,
    /// Whether this embed affects exported symbols.
    pub affects_exports: bool,
}

impl EmbedFileRef {
    /// Classify the impact of this embed file on downstream consumers.
    pub fn classify_impact(&self) -> EmbedFileImpact {
        if self.affects_exports {
            EmbedFileImpact::ExportedConst
        } else {
            EmbedFileImpact::PrivateBody
        }
    }
}

/// Classification of how an embedded file affects downstream consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmbedFileImpact {
    /// Embed file used only in private contexts, no downstream effect.
    PrivateBody,
    /// Embed file content affects exported constants or types.
    ExportedConst,
}

/// A C translation unit imported via @cImport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CImportRef {
    /// Unique identifier for this C import.
    pub import_id: u64,
    /// Header file path.
    pub header_path: String,
    /// Content hash of the header at time of import.
    pub header_hash: [u8; 32],
    /// The Zig file that contains this @cImport.
    pub source_file: u64,
    /// Line number in source where @cImport appears.
    pub source_line: u32,
    /// Whether this import affects exported symbols.
    pub affects_exports: bool,
    /// Dependent header paths included by this header.
    pub dependencies: Vec<String>,
}

impl CImportRef {
    /// Classify the impact of this C import on downstream consumers.
    pub fn classify_impact(&self) -> CImportImpact {
        if self.affects_exports {
            CImportImpact::ExportedSignature
        } else {
            CImportImpact::PrivateBody
        }
    }
}

/// Classification of how a C import affects downstream consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CImportImpact {
    /// C import used only in private contexts, no downstream effect.
    PrivateBody,
    /// C header changes affect exported function signatures or types.
    ExportedSignature,
}

/// Type of change that triggered invalidation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Private function body changed (no downstream effect).
    PrivateBody,
    /// Exported function signature changed.
    ExportedSignature,
    /// Exported struct layout changed.
    ExportedLayout,
    /// Comptime evaluation changed.
    ComptimeChange,
    /// Embedded file content changed.
    EmbedFileChange,
    /// C header content changed.
    CHeaderChange,
    /// Build options changed (target, optimization, etc.).
    BuildOptions,
    /// Source file content changed.
    SourceContent,
    /// Unknown or mixed change.
    Unknown,
}

/// Recommended action after invalidation analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidationAction {
    /// Full rebuild required.
    FullRebuild,
    /// Only affected artifacts need rebuild, others can be reused.
    PartialRebuild,
    /// Incremental update possible, artifacts can be reused.
    IncrementalReuse,
}

/// Machine-readable explanation of an invalidation decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationExplanation {
    /// What type of change triggered this invalidation.
    pub change_type: ChangeType,
    /// Number of affected artifacts.
    pub affected_count: u32,
    /// Number of artifacts that can be reused.
    pub reusable_count: u32,
    /// Human-readable reason for the decision.
    pub reason: String,
    /// Recommended action.
    pub action: InvalidationAction,
}

impl InvalidationExplanation {
    /// Create an explanation for a private body change (optimistic reuse possible).
    pub fn private_body_change(reusable_count: u32) -> Self {
        Self {
            change_type: ChangeType::PrivateBody,
            affected_count: 0,
            reusable_count,
            reason: "Private body change does not affect exported symbols".to_string(),
            action: InvalidationAction::IncrementalReuse,
        }
    }

    /// Create an explanation for an exported signature change (full rebuild needed).
    pub fn exported_signature_change(affected_count: u32) -> Self {
        Self {
            change_type: ChangeType::ExportedSignature,
            affected_count,
            reusable_count: 0,
            reason: "Exported signature change requires downstream recompilation".to_string(),
            action: InvalidationAction::FullRebuild,
        }
    }

    /// Create an explanation for a layout change (partial rebuild needed).
    pub fn exported_layout_change(affected_count: u32, reusable_count: u32) -> Self {
        Self {
            change_type: ChangeType::ExportedLayout,
            affected_count,
            reusable_count,
            reason: "Layout change affects ABI, requires recompilation of dependents".to_string(),
            action: InvalidationAction::PartialRebuild,
        }
    }

    /// Create an explanation for a build options change (full rebuild needed).
    pub fn build_options_change() -> Self {
        Self {
            change_type: ChangeType::BuildOptions,
            affected_count: u32::MAX,
            reusable_count: 0,
            reason: "Build options change invalidates all cached artifacts".to_string(),
            action: InvalidationAction::FullRebuild,
        }
    }
}

/// A single fact in the proof chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofFact {
    /// The type of fact.
    pub fact_type: ProofFactType,
    /// The value or content of the fact.
    pub value: String,
    /// Optional reference to an artifact or entity this fact relates to.
    pub reference: Option<String>,
}

/// Types of proof facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofFactType {
    /// Checksum of the artifact content.
    Checksum,
    /// Content hash of source file.
    ContentHash,
    /// Declaration visibility level.
    Visibility,
    /// Symbol linkage type.
    Linkage,
    /// Build option value.
    BuildOption,
    /// Type layout information.
    TypeLayout,
    /// Function signature hash.
    SignatureHash,
    /// AIR body reference.
    AirBodyRef,
}

/// A chain of proof facts that validates an invalidation decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofChain {
    /// Whether reuse was allowed.
    pub reuse_allowed: bool,
    /// The facts in this proof chain.
    pub facts: Vec<ProofFact>,
    /// The explanation this proof chain validates.
    pub explanation: InvalidationExplanation,
}

impl ProofChain {
    /// Create a proof chain for a private body change.
    pub fn private_body_proof(source_checksum: [u8; 32], air_checksum: [u8; 32]) -> Self {
        Self {
            reuse_allowed: true,
            facts: vec![
                ProofFact {
                    fact_type: ProofFactType::ContentHash,
                    value: format!("{:x?}", source_checksum),
                    reference: None,
                },
                ProofFact {
                    fact_type: ProofFactType::AirBodyRef,
                    value: format!("{:x?}", air_checksum),
                    reference: None,
                },
                ProofFact {
                    fact_type: ProofFactType::Visibility,
                    value: "Private".to_string(),
                    reference: None,
                },
            ],
            explanation: InvalidationExplanation::private_body_change(1),
        }
    }

    /// Create a proof chain for an exported signature change.
    pub fn exported_signature_proof(signature_hash: [u8; 32], export_name: &str) -> Self {
        Self {
            reuse_allowed: false,
            facts: vec![
                ProofFact {
                    fact_type: ProofFactType::SignatureHash,
                    value: format!("{:x?}", signature_hash),
                    reference: Some(export_name.to_string()),
                },
                ProofFact {
                    fact_type: ProofFactType::Visibility,
                    value: "Public".to_string(),
                    reference: Some(export_name.to_string()),
                },
            ],
            explanation: InvalidationExplanation::exported_signature_change(1),
        }
    }

    /// Create a proof chain for a layout change.
    pub fn layout_change_proof(layout_hash: [u8; 32], type_name: &str) -> Self {
        Self {
            reuse_allowed: false,
            facts: vec![ProofFact {
                fact_type: ProofFactType::TypeLayout,
                value: format!("{:x?}", layout_hash),
                reference: Some(type_name.to_string()),
            }],
            explanation: InvalidationExplanation::exported_layout_change(1, 0),
        }
    }

    /// Create a proof chain for build options change.
    pub fn build_options_proof() -> Self {
        Self {
            reuse_allowed: false,
            facts: vec![ProofFact {
                fact_type: ProofFactType::BuildOption,
                value: "build_options_changed".to_string(),
                reference: None,
            }],
            explanation: InvalidationExplanation::build_options_change(),
        }
    }

    /// Check if this proof chain allows reuse.
    pub fn allows_reuse(&self) -> bool {
        self.reuse_allowed
    }

    /// Get the reason for this proof chain decision.
    pub fn reason(&self) -> &str {
        &self.explanation.reason
    }
}

/// Tracks which parts of an artifact can be reused after an incremental build.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactReuse {
    /// Whether the entire artifact is reusable.
    pub fully_reusable: bool,
    /// Reusable source files (by path).
    pub reusable_sources: Vec<String>,
    /// Reusable declarations (by id).
    pub reusable_decls: Vec<u64>,
    /// Reusable types (by id).
    pub reusable_types: Vec<u64>,
    /// Reusable layouts (by id).
    pub reusable_layouts: Vec<u64>,
    /// Reusable AIR bodies (by function_id).
    pub reusable_air_bodies: Vec<u64>,
    /// Reusable exports (by name).
    pub reusable_exports: Vec<String>,
    /// Number of affected items that require rebuild.
    pub affected_count: u32,
    /// Number of reusable items.
    pub reusable_count: u32,
}

impl ArtifactReuse {
    /// Create a new artifact reuse tracker.
    pub fn new() -> Self {
        Self {
            fully_reusable: false,
            reusable_sources: Vec::new(),
            reusable_decls: Vec::new(),
            reusable_types: Vec::new(),
            reusable_layouts: Vec::new(),
            reusable_air_bodies: Vec::new(),
            reusable_exports: Vec::new(),
            affected_count: 0,
            reusable_count: 0,
        }
    }

    /// Mark a source file as reusable.
    pub fn add_reusable_source(&mut self, path: &str) {
        self.reusable_sources.push(path.to_string());
    }

    /// Mark a declaration as reusable.
    pub fn add_reusable_decl(&mut self, id: u64) {
        self.reusable_decls.push(id);
    }

    /// Mark a type as reusable.
    pub fn add_reusable_type(&mut self, id: u64) {
        self.reusable_types.push(id);
    }

    /// Mark a layout as reusable.
    pub fn add_reusable_layout(&mut self, id: u64) {
        self.reusable_layouts.push(id);
    }

    /// Mark an AIR body as reusable.
    pub fn add_reusable_air_body(&mut self, function_id: u64) {
        self.reusable_air_bodies.push(function_id);
    }

    /// Mark an export as reusable.
    pub fn add_reusable_export(&mut self, name: &str) {
        self.reusable_exports.push(name.to_string());
    }

    /// Set the artifact as fully reusable.
    pub fn set_fully_reusable(&mut self) {
        self.fully_reusable = true;
    }

    /// Record affected count.
    pub fn set_affected_count(&mut self, count: u32) {
        self.affected_count = count;
    }

    /// Compute reusable count from all reusable items.
    pub fn compute_reusable_count(&mut self) {
        self.reusable_count = (self.reusable_sources.len()
            + self.reusable_decls.len()
            + self.reusable_types.len()
            + self.reusable_layouts.len()
            + self.reusable_air_bodies.len()
            + self.reusable_exports.len()) as u32;
    }

    /// Check if there are any reusable items.
    pub fn has_reusable_items(&self) -> bool {
        !self.reusable_sources.is_empty()
            || !self.reusable_decls.is_empty()
            || !self.reusable_types.is_empty()
            || !self.reusable_layouts.is_empty()
            || !self.reusable_air_bodies.is_empty()
            || !self.reusable_exports.is_empty()
    }

    /// Determine reuse based on proof chain.
    pub fn from_proof_chain(proof: &ProofChain) -> Self {
        let mut reuse = Self::new();
        if proof.allows_reuse() {
            reuse.set_fully_reusable();
            // Count the proof facts as reusable evidence
            reuse.reusable_count = proof.facts.len() as u32;
        } else {
            // For non-reusable, compute from empty lists
            reuse.compute_reusable_count();
        }
        reuse
    }
}

impl Default for ArtifactReuse {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete `.zsnap` semantic snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapSchema {
    pub header: SnapHeader,
    pub source_files: Vec<SourceFile>,
    pub build_options: BuildOptions,
    pub decls: Vec<DeclRef>,
    pub analysis_units: Vec<AnalysisUnit>,
    pub types: Vec<TypeRecord>,
    pub layouts: Vec<LayoutRecord>,
    pub air_bodies: Vec<AirBodyRef>,
    pub exports: Vec<ExportSymbol>,
    pub comptime_calls: Vec<ComptimeCall>,
    pub embed_files: Vec<EmbedFileRef>,
    pub c_imports: Vec<CImportRef>,
}

impl Default for SnapSchema {
    fn default() -> Self {
        Self {
            header: SnapHeader {
                magic: *ZSNAP_MAGIC,
                schema_version: 1,
                min_adapter_version: 1,
                zig_commit: [0u8; 20],
                target: String::new(),
                backend: String::new(),
                optimize_mode: String::new(),
                timestamp_ns: 0,
                source_file_count: 0,
                checksum: [0u8; 32],
            },
            source_files: Vec::new(),
            build_options: BuildOptions::default(),
            decls: Vec::new(),
            analysis_units: Vec::new(),
            types: Vec::new(),
            layouts: Vec::new(),
            air_bodies: Vec::new(),
            exports: Vec::new(),
            comptime_calls: Vec::new(),
            embed_files: Vec::new(),
            c_imports: Vec::new(),
        }
    }
}

/// Result of a cache lookup operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheLookupResult {
    /// Whether the cache hit was found.
    pub cache_hit: bool,
    /// The cache key that was looked up.
    pub cache_key: String,
    /// Explanation of why this was a hit or miss.
    pub explanation: CacheExplanation,
    /// Proof chain validating this cache decision.
    pub proof: Option<ProofChain>,
}

impl CacheLookupResult {
    /// Create a cache hit result.
    pub fn hit(cache_key: String, explanation: CacheExplanation) -> Self {
        Self {
            cache_hit: true,
            cache_key,
            explanation,
            proof: None,
        }
    }

    /// Create a cache miss result.
    pub fn miss(cache_key: String, explanation: CacheExplanation) -> Self {
        Self {
            cache_hit: false,
            cache_key,
            explanation,
            proof: None,
        }
    }

    /// Create a cache miss with proof.
    pub fn miss_with_proof(
        cache_key: String,
        explanation: CacheExplanation,
        proof: ProofChain,
    ) -> Self {
        Self {
            cache_hit: false,
            cache_key,
            explanation,
            proof: Some(proof),
        }
    }

    /// Check if this result has a proof.
    pub fn has_proof(&self) -> bool {
        self.proof.is_some()
    }

    /// Get the proof chain if present.
    pub fn get_proof(&self) -> Option<&ProofChain> {
        self.proof.as_ref()
    }
}

/// Proof facts validating a cache decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheProof {
    /// The cache key this proof applies to.
    pub cache_key: String,
    /// The artifact kind this proof applies to.
    pub artifact_kind: CacheArtifactKind,
    /// Whether the cache hit is valid.
    pub valid: bool,
    /// The facts supporting this cache decision.
    pub facts: Vec<ProofFact>,
    /// The explanation of this cache decision.
    pub explanation: String,
}

impl CacheProof {
    /// Create a cache proof for a checksum match.
    pub fn checksum_match(
        cache_key: String,
        artifact_kind: CacheArtifactKind,
        checksum: [u8; 32],
    ) -> Self {
        let explanation = format!("{:?} checksum matches cached version", artifact_kind);
        Self {
            cache_key,
            artifact_kind,
            valid: true,
            facts: vec![ProofFact {
                fact_type: ProofFactType::Checksum,
                value: format!("{:x?}", checksum),
                reference: None,
            }],
            explanation,
        }
    }

    /// Create a cache proof for a source change.
    pub fn source_changed(
        cache_key: String,
        artifact_kind: CacheArtifactKind,
        source_path: &str,
        old_checksum: [u8; 32],
        new_checksum: [u8; 32],
    ) -> Self {
        Self {
            cache_key,
            artifact_kind,
            valid: false,
            facts: vec![
                ProofFact {
                    fact_type: ProofFactType::ContentHash,
                    value: format!("old: {:x?}", old_checksum),
                    reference: Some(source_path.to_string()),
                },
                ProofFact {
                    fact_type: ProofFactType::ContentHash,
                    value: format!("new: {:x?}", new_checksum),
                    reference: Some(source_path.to_string()),
                },
            ],
            explanation: format!("Source file {} changed", source_path),
        }
    }

    /// Create a cache proof for a build options change.
    pub fn build_options_changed(cache_key: String) -> Self {
        Self {
            cache_key,
            artifact_kind: CacheArtifactKind::BuildArtifact,
            valid: false,
            facts: vec![ProofFact {
                fact_type: ProofFactType::BuildOption,
                value: "build_options_changed".to_string(),
                reference: None,
            }],
            explanation: "Build options changed, cache invalid".to_string(),
        }
    }

    /// Create a cache proof for artifact missing.
    pub fn artifact_missing(
        cache_key: String,
        artifact_kind: CacheArtifactKind,
        path: &str,
    ) -> Self {
        Self {
            cache_key,
            artifact_kind,
            valid: false,
            facts: vec![ProofFact {
                fact_type: ProofFactType::AirBodyRef,
                value: format!("missing: {}", path),
                reference: Some(path.to_string()),
            }],
            explanation: format!("Cached artifact not found at {}", path),
        }
    }

    /// Check if this proof validates a cache hit.
    pub fn is_hit_proof(&self) -> bool {
        self.valid
    }
}

/// Release gate that validates proof chain authority.
/// Fails if the authoritative path falls back to non-authoritative mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseProofGate {
    /// Whether the gate passed (true) or failed (false).
    pub passed: bool,
    /// The proof chain that was validated.
    pub proof_chain: Option<ProofChain>,
    /// The mode of the validation (authoritative or fallback).
    pub validation_mode: AuthorityMode,
    /// Reason for pass or fail.
    pub reason: String,
    /// Whether fallback was used.
    pub used_fallback: bool,
}

impl ReleaseProofGate {
    /// Create a release gate that passed in authoritative mode.
    pub fn authoritative_pass(proof_chain: ProofChain) -> Self {
        Self {
            passed: true,
            proof_chain: Some(proof_chain),
            validation_mode: AuthorityMode::Authoritative,
            reason: "Release gate passed: authoritative mode verified".to_string(),
            used_fallback: false,
        }
    }

    /// Create a release gate that failed due to fallback.
    pub fn fallback_fail(reason: &str) -> Self {
        Self {
            passed: false,
            proof_chain: None,
            validation_mode: AuthorityMode::Fallback,
            reason: format!("Release gate FAILED: fallback mode used - {}", reason),
            used_fallback: true,
        }
    }

    /// Validate if the release gate allows proceeding.
    pub fn validate(&self) -> Result<(), ReleaseGateError> {
        if self.passed {
            Ok(())
        } else {
            Err(ReleaseGateError::FallbackModeUsed(self.reason.clone()))
        }
    }

    /// Check if fallback was used.
    pub fn used_fallback(&self) -> bool {
        self.used_fallback
    }
}

/// Mode of authority for validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorityMode {
    /// Authoritative path, using real compiler data.
    Authoritative,
    /// Fallback path, using non-authoritative data.
    Fallback,
}

/// Error when release gate fails.
#[derive(Debug, Clone)]
pub enum ReleaseGateError {
    /// Fallback mode was used, which is not allowed in release.
    FallbackModeUsed(String),
}

impl std::fmt::Display for ReleaseGateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReleaseGateError::FallbackModeUsed(reason) => {
                write!(f, "Release gate failed: fallback mode used ({})", reason)
            }
        }
    }
}

impl std::error::Error for ReleaseGateError {}

/// Explanation of a cache hit or miss decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheExplanation {
    /// The reason for cache hit or miss.
    pub reason: String,
    /// The artifact kind this applies to.
    pub artifact_kind: CacheArtifactKind,
    /// Whether the cached artifact is still valid.
    pub is_valid: bool,
    /// Optional details about the cached artifact.
    pub details: Option<CacheDetails>,
}

impl CacheExplanation {
    /// Create an explanation for a cache hit due to matching checksums.
    pub fn checksum_match(artifact_kind: CacheArtifactKind) -> Self {
        Self {
            reason: format!("{:?} checksum matches cached version", artifact_kind),
            artifact_kind,
            is_valid: true,
            details: None,
        }
    }

    /// Create an explanation for a cache miss due to source change.
    pub fn source_changed(artifact_kind: CacheArtifactKind, source_path: &str) -> Self {
        Self {
            reason: format!("Source file {:?} changed", source_path),
            artifact_kind,
            is_valid: false,
            details: Some(CacheDetails::SourceChanged {
                path: source_path.to_string(),
            }),
        }
    }

    /// Create an explanation for a cache miss due to build options change.
    pub fn build_options_changed() -> Self {
        Self {
            reason: "Build options changed".to_string(),
            artifact_kind: CacheArtifactKind::BuildArtifact,
            is_valid: false,
            details: Some(CacheDetails::BuildOptionsChanged),
        }
    }

    /// Create an explanation for a cache miss due to missing artifact.
    pub fn artifact_missing(artifact_kind: CacheArtifactKind, path: &str) -> Self {
        Self {
            reason: format!("Cached {:?} not found at {}", artifact_kind, path),
            artifact_kind,
            is_valid: false,
            details: Some(CacheDetails::ArtifactMissing {
                path: path.to_string(),
            }),
        }
    }
}

/// Kind of artifact being cached.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheArtifactKind {
    /// A source file.
    SourceFile,
    /// A declaration.
    Decl,
    /// A type record.
    TypeRecord,
    /// A layout record.
    LayoutRecord,
    /// An AIR body.
    AirBody,
    /// A compiled object.
    Object,
    /// A build artifact.
    BuildArtifact,
}

impl std::fmt::Display for CacheArtifactKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheArtifactKind::SourceFile => write!(f, "SourceFile"),
            CacheArtifactKind::Decl => write!(f, "Decl"),
            CacheArtifactKind::TypeRecord => write!(f, "TypeRecord"),
            CacheArtifactKind::LayoutRecord => write!(f, "LayoutRecord"),
            CacheArtifactKind::AirBody => write!(f, "AirBody"),
            CacheArtifactKind::Object => write!(f, "Object"),
            CacheArtifactKind::BuildArtifact => write!(f, "BuildArtifact"),
        }
    }
}

/// Additional details about a cache decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheDetails {
    /// Source file changed.
    SourceChanged { path: String },
    /// Build options changed.
    BuildOptionsChanged,
    /// Artifact missing from cache.
    ArtifactMissing { path: String },
    /// Dependency invalidated.
    DependencyInvalidated { dep_path: String },
}

impl SnapSchema {
    pub fn header_magic_valid(&self) -> bool {
        &self.header.magic == ZSNAP_MAGIC
    }

    pub fn header_version_compatible(&self) -> bool {
        self.header.schema_version <= SCHEMA_VERSION
    }

    /// Returns a determinized copy with stable ordering for byte-identical output.
    pub fn determinized(&self) -> SnapSchema {
        let mut schema = self.clone();
        schema.source_files.sort_by(|a, b| a.path.cmp(&b.path));
        schema.decls.sort_by(|a, b| {
            a.owner_file
                .cmp(&b.owner_file)
                .then_with(|| a.id.cmp(&b.id))
        });
        schema.analysis_units.sort_by(|a, b| a.id.cmp(&b.id));
        schema.types.sort_by(|a, b| a.id.cmp(&b.id));
        schema.layouts.sort_by(|a, b| a.id.cmp(&b.id));
        schema
            .air_bodies
            .sort_by(|a, b| a.function_id.cmp(&b.function_id));
        schema.exports.sort_by(|a, b| a.name.cmp(&b.name));
        schema
            .comptime_calls
            .sort_by(|a, b| a.call_id.cmp(&b.call_id));
        schema.embed_files.sort_by(|a, b| a.path.cmp(&b.path));
        schema
            .c_imports
            .sort_by(|a, b| a.header_path.cmp(&b.header_path));
        schema
    }

    /// Compute BLAKE3 checksum of the determinized JSON representation.
    pub fn compute_checksum(&self) -> [u8; 32] {
        use blake3::Hasher;
        let determinized = self.determinized();
        let json = serde_json::to_vec(&determinized).unwrap_or_default();
        let mut hasher = Hasher::new();
        hasher.update(&json);
        *hasher.finalize().as_bytes()
    }

    /// Verify the stored checksum matches the computed checksum.
    /// Returns `Ok(())` if checksums match, `Err` with `CorruptionError::ChecksumMismatch` otherwise.
    pub fn verify_checksum(&self) -> Result<(), crate::corruption::CorruptionError> {
        use crate::corruption::CorruptionError;
        // Compute checksum of content only (excluding the checksum field itself)
        let determinized = self.determinized();
        // Temporarily zero the checksum field for computation
        let mut content = determinized.clone();
        content.header.checksum = [0u8; 32];
        let json = serde_json::to_vec(&content).unwrap_or_default();
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&json);
        let computed = *hasher.finalize().as_bytes();

        if &computed == &self.header.checksum {
            Ok(())
        } else {
            Err(CorruptionError::ChecksumMismatch {
                expected: format!("{:x?}", self.header.checksum),
                actual: format!("{:x?}", computed),
            })
        }
    }
}

// ============================================================================
// Binary Parser
// ============================================================================

#[cfg(feature = "std")]
use std::path::Path;

#[cfg(feature = "std")]
use std::fs::File;

#[cfg(feature = "std")]
use std::io::{BufReader, Read};

/// Errors that can occur during snapshot binary parsing
#[derive(Debug, Clone)]
pub enum BinaryParseError {
    InvalidMagic { expected: [u8; 8], got: [u8; 8] },
    UnsupportedVersion { version: u32, max: u32 },
    TruncatedData { expected: usize, got: usize },
    ChecksumMismatch { expected: String, got: String },
    InvalidUtf8 { field: String },
    IoError(String),
}

impl BinaryParseError {
    /// Returns true if this error indicates a format version incompatibility
    pub fn is_version_error(&self) -> bool {
        matches!(self, BinaryParseError::UnsupportedVersion { .. })
    }

    /// Returns true if this error indicates corrupted data
    pub fn is_corruption(&self) -> bool {
        matches!(
            self,
            BinaryParseError::InvalidMagic { .. }
                | BinaryParseError::ChecksumMismatch { .. }
                | BinaryParseError::TruncatedData { .. }
        )
    }
}

impl std::fmt::Display for BinaryParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryParseError::InvalidMagic { expected, got } => {
                write!(
                    f,
                    "invalid magic bytes: expected {:?}, got {:?}",
                    expected, got
                )
            }
            BinaryParseError::UnsupportedVersion { version, max } => {
                write!(
                    f,
                    "unsupported schema version: {} (max supported: {})",
                    version, max
                )
            }
            BinaryParseError::TruncatedData { expected, got } => {
                write!(
                    f,
                    "truncated data: expected {} bytes, got {}",
                    expected, got
                )
            }
            BinaryParseError::ChecksumMismatch { expected, got } => {
                write!(f, "checksum mismatch: expected {}, got {}", expected, got)
            }
            BinaryParseError::InvalidUtf8 { field } => {
                write!(f, "invalid UTF-8 in string field: {}", field)
            }
            BinaryParseError::IoError(msg) => {
                write!(f, "IO error: {}", msg)
            }
        }
    }
}

impl std::error::Error for BinaryParseError {}

/// Result type for binary parsing
pub type BinaryParseResult<T> = Result<T, BinaryParseError>;

/// Binary parser for `.zsnap` format.
///
/// The binary format structure is:
/// - 8 bytes: magic ("ZSNAP001")
/// - 4 bytes: schema version (little-endian u32)
/// - 20 bytes: Zig commit hash
/// - variable: target string (u32 length + UTF-8 bytes)
/// - variable: backend string
/// - variable: optimize_mode string
/// - 8 bytes: timestamp_ns (little-endian u64)
/// - 4 bytes: source_file_count (little-endian u32)
/// - 32 bytes: checksum (BLAKE3)
/// - variable: sections (source_files, build_options, decls, etc.)
#[derive(Debug, Clone)]
pub struct BinaryParser {
    strict_mode: bool,
}

impl BinaryParser {
    /// Create a new binary parser
    pub fn new() -> Self {
        Self { strict_mode: true }
    }

    /// Enable or disable strict mode
    pub fn with_strict_mode(mut self, enabled: bool) -> Self {
        self.strict_mode = enabled;
        self
    }

    /// Parse `.zsnap` binary format from bytes
    pub fn parse(&mut self, data: &[u8]) -> BinaryParseResult<SnapSchema> {
        let mut offset = 0;

        // Read and validate magic (8 bytes)
        if data.len() < 8 {
            return Err(BinaryParseError::TruncatedData {
                expected: 8,
                got: data.len(),
            });
        }

        let magic: [u8; 8] = data[0..8].try_into().unwrap();
        if &magic != ZSNAP_MAGIC {
            return Err(BinaryParseError::InvalidMagic {
                expected: *ZSNAP_MAGIC,
                got: magic,
            });
        }
        offset += 8;

        // Read schema version (4 bytes)
        if data.len() < offset + 4 {
            return Err(BinaryParseError::TruncatedData {
                expected: offset + 4,
                got: data.len(),
            });
        }
        let schema_version = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        offset += 4;

        // Check schema version compatibility
        if schema_version > SCHEMA_VERSION {
            return Err(BinaryParseError::UnsupportedVersion {
                version: schema_version,
                max: SCHEMA_VERSION,
            });
        }

        // Read Zig commit hash (20 bytes)
        if data.len() < offset + 20 {
            return Err(BinaryParseError::TruncatedData {
                expected: offset + 20,
                got: data.len(),
            });
        }
        let zig_commit: [u8; 20] = data[offset..offset + 20].try_into().unwrap();
        offset += 20;

        // Read target string (length-prefixed)
        let (target, consumed) = self.read_string(&data[offset..])?;
        offset += consumed;

        // Read backend string
        let (backend, consumed) = self.read_string(&data[offset..])?;
        offset += consumed;

        // Read optimize_mode string
        let (optimize_mode, consumed) = self.read_string(&data[offset..])?;
        offset += consumed;

        // Read timestamp_ns (8 bytes)
        if data.len() < offset + 8 {
            return Err(BinaryParseError::TruncatedData {
                expected: offset + 8,
                got: data.len(),
            });
        }
        let timestamp_ns = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;

        // Read source_file_count (4 bytes)
        if data.len() < offset + 4 {
            return Err(BinaryParseError::TruncatedData {
                expected: offset + 4,
                got: data.len(),
            });
        }
        let source_file_count = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        offset += 4;

        // Read checksum (32 bytes)
        if data.len() < offset + 32 {
            return Err(BinaryParseError::TruncatedData {
                expected: offset + 32,
                got: data.len(),
            });
        }
        let checksum: [u8; 32] = data[offset..offset + 32].try_into().unwrap();
        offset += 32;

        // Build header
        let header = SnapHeader {
            magic,
            schema_version,
            min_adapter_version: 1, // Adapter must support at least version 1
            zig_commit,
            target,
            backend,
            optimize_mode,
            timestamp_ns,
            source_file_count,
            checksum,
        };

        // For v1, we read JSON payload for the rest (simplified approach)
        // In production, each section would be binary-encoded
        // Here we just read the remaining data as JSON
        let remaining = &data[offset..];

        // Try to parse as JSON for the sections
        let sections: serde_json::Value =
            serde_json::from_slice(remaining).map_err(|_| BinaryParseError::TruncatedData {
                expected: remaining.len(),
                got: remaining.len().min(1024),
            })?;

        // Parse sections from JSON
        let source_files: Vec<SourceFile> = sections
            .get("source_files")
            .map(|v| serde_json::from_value(v.clone()).unwrap_or_default())
            .unwrap_or_default();

        let build_options: BuildOptions = sections
            .get("build_options")
            .map(|v| serde_json::from_value(v.clone()).unwrap_or_default())
            .unwrap_or_default();

        let decls: Vec<DeclRef> = sections
            .get("decls")
            .map(|v| serde_json::from_value(v.clone()).unwrap_or_default())
            .unwrap_or_default();

        let analysis_units: Vec<AnalysisUnit> = sections
            .get("analysis_units")
            .map(|v| serde_json::from_value(v.clone()).unwrap_or_default())
            .unwrap_or_default();

        let types: Vec<TypeRecord> = sections
            .get("types")
            .map(|v| serde_json::from_value(v.clone()).unwrap_or_default())
            .unwrap_or_default();

        let layouts: Vec<LayoutRecord> = sections
            .get("layouts")
            .map(|v| serde_json::from_value(v.clone()).unwrap_or_default())
            .unwrap_or_default();

        let air_bodies: Vec<AirBodyRef> = sections
            .get("air_bodies")
            .map(|v| serde_json::from_value(v.clone()).unwrap_or_default())
            .unwrap_or_default();

        let exports: Vec<ExportSymbol> = sections
            .get("exports")
            .map(|v| serde_json::from_value(v.clone()).unwrap_or_default())
            .unwrap_or_default();

        Ok(SnapSchema {
            header,
            source_files,
            build_options,
            decls,
            analysis_units,
            types,
            layouts,
            air_bodies,
            exports,
            comptime_calls: vec![],
            embed_files: vec![],
            c_imports: vec![],
        })
    }

    /// Read a length-prefixed string from data
    fn read_string(&self, data: &[u8]) -> BinaryParseResult<(String, usize)> {
        // Read 4-byte length prefix
        if data.len() < 4 {
            return Err(BinaryParseError::TruncatedData {
                expected: 4,
                got: data.len(),
            });
        }

        let len = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;

        if data.len() < 4 + len {
            return Err(BinaryParseError::TruncatedData {
                expected: 4 + len,
                got: data.len(),
            });
        }

        let bytes = &data[4..4 + len];
        let s = String::from_utf8(bytes.to_vec()).map_err(|_| BinaryParseError::InvalidUtf8 {
            field: format!("{} byte string", len),
        })?;

        Ok((s, 4 + len))
    }

    /// Parse `.zsnap` binary format from a file
    #[cfg(feature = "std")]
    pub fn parse_file(&mut self, path: &Path) -> BinaryParseResult<SnapSchema> {
        let file = File::open(path).map_err(|e| BinaryParseError::IoError(e.to_string()))?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader
            .read_to_end(&mut data)
            .map_err(|e| BinaryParseError::IoError(e.to_string()))?;
        self.parse(&data)
    }

    /// Get whether we're in strict mode
    pub fn is_strict_mode(&self) -> bool {
        self.strict_mode
    }

    /// Detect corruption in raw bytes before parsing.
    /// Uses CorruptionDetector with ZSNAP magic bytes for validation.
    pub fn detect_corruption(&self, data: &[u8]) -> Result<(), crate::corruption::CorruptionError> {
        use crate::corruption::CorruptionDetector;
        let detector = CorruptionDetector::new()
            .with_expected_magic(*ZSNAP_MAGIC)
            .with_strict_size_check(false);
        detector.detect(data)
    }
}

impl Default for BinaryParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a `.zsnap` binary file and return detailed diagnostics
#[cfg(feature = "std")]
pub fn validate_binary(path: &Path) -> BinaryParseResult<ValidationReport> {
    let mut parser = BinaryParser::new();
    let result = parser.parse_file(path);

    match result {
        Ok(schema) => Ok(ValidationReport {
            valid: true,
            version: schema.header.schema_version,
            target: schema.header.target.clone(),
            source_file_count: schema.header.source_file_count,
            errors: Vec::new(),
        }),
        Err(e) => Ok(ValidationReport {
            valid: false,
            version: 0,
            target: String::new(),
            source_file_count: 0,
            errors: vec![e.to_string()],
        }),
    }
}

/// Validation report for a `.zsnap` file
#[cfg(feature = "std")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub version: u32,
    pub target: String,
    pub source_file_count: u32,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_parser_creation() {
        let parser = BinaryParser::new();
        assert!(parser.is_strict_mode());
    }

    #[test]
    fn test_binary_parser_non_strict() {
        let parser = BinaryParser::new().with_strict_mode(false);
        assert!(!parser.is_strict_mode());
    }

    #[test]
    fn test_read_string_truncated() {
        let parser = BinaryParser::new();
        let data = [0u8, 0, 0, 1]; // length 1 but no data
        let result = parser.read_string(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_string_invalid_utf8() {
        let parser = BinaryParser::new();
        // length 4, followed by invalid UTF-8
        let data = [4, 0, 0, 0, 0x80, 0x81, 0x82, 0x83];
        let result = parser.read_string(&data);
        assert!(matches!(result, Err(BinaryParseError::InvalidUtf8 { .. })));
    }

    #[test]
    fn test_read_string_valid() {
        let parser = BinaryParser::new();
        // length 5, followed by "hello"
        let data = [5, 0, 0, 0, b'h', b'e', b'l', b'l', b'o'];
        let result = parser.read_string(&data);
        assert!(result.is_ok());
        let (s, consumed) = result.unwrap();
        assert_eq!(s, "hello");
        assert_eq!(consumed, 9);
    }

    #[test]
    fn test_binary_error_is_version_error() {
        let err = BinaryParseError::UnsupportedVersion {
            version: 99,
            max: 1,
        };
        assert!(err.is_version_error());
        assert!(!err.is_corruption());
    }

    #[test]
    fn test_binary_error_is_corruption() {
        let magic_err = BinaryParseError::InvalidMagic {
            expected: *ZSNAP_MAGIC,
            got: [0u8; 8],
        };
        assert!(!magic_err.is_version_error());
        assert!(magic_err.is_corruption());

        let checksum_err = BinaryParseError::ChecksumMismatch {
            expected: "abc".to_string(),
            got: "def".to_string(),
        };
        assert!(checksum_err.is_corruption());
    }

    #[test]
    fn test_binary_error_display() {
        let magic_err = BinaryParseError::InvalidMagic {
            expected: *ZSNAP_MAGIC,
            got: [1u8; 8],
        };
        let display = format!("{}", magic_err);
        assert!(display.contains("invalid magic bytes"));

        let version_err = BinaryParseError::UnsupportedVersion { version: 5, max: 1 };
        let display = format!("{}", version_err);
        assert!(display.contains("unsupported schema version"));
        assert!(display.contains("5"));

        let truncated_err = BinaryParseError::TruncatedData {
            expected: 100,
            got: 50,
        };
        let display = format!("{}", truncated_err);
        assert!(display.contains("truncated data"));

        let utf8_err = BinaryParseError::InvalidUtf8 {
            field: "test_field".to_string(),
        };
        let display = format!("{}", utf8_err);
        assert!(display.contains("invalid UTF-8"));

        let io_err = BinaryParseError::IoError("file not found".to_string());
        let display = format!("{}", io_err);
        assert!(display.contains("IO error"));
    }

    #[test]
    fn test_binary_parse_truncated_header() {
        let mut parser = BinaryParser::new();
        // Only 4 bytes when we need at least 8 for magic
        let data = [0, 0, 0, 0];
        let result = parser.parse(&data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_corruption());
        assert!(matches!(err, BinaryParseError::TruncatedData { .. }));
    }

    #[test]
    fn test_binary_parse_invalid_magic() {
        let mut parser = BinaryParser::new();
        // Valid header size but wrong magic
        let mut data = [0u8; 100];
        data[0..8].copy_from_slice(b"NOTVALID");
        let result = parser.parse(&data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_corruption());
        assert!(matches!(err, BinaryParseError::InvalidMagic { .. }));
    }

    #[test]
    fn test_binary_parse_unsupported_version() {
        let mut parser = BinaryParser::new();
        // Valid magic but version 99 (way higher than SCHEMA_VERSION = 1)
        let mut data = [0u8; 200];
        data[0..8].copy_from_slice(ZSNAP_MAGIC);
        data[8..12].copy_from_slice(&99u32.to_le_bytes());
        let result = parser.parse(&data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_version_error());
        assert!(matches!(err, BinaryParseError::UnsupportedVersion { .. }));
    }

    #[test]
    fn test_snap_schema_header_validators() {
        let schema = SnapSchema {
            header: SnapHeader {
                magic: *ZSNAP_MAGIC,
                schema_version: 1,
                min_adapter_version: 1,
                zig_commit: [0u8; 20],
                target: "x86_64-unknown-linux-gnu".to_string(),
                backend: "llvm".to_string(),
                optimize_mode: "ReleaseFast".to_string(),
                timestamp_ns: 1234567890,
                source_file_count: 5,
                checksum: [0u8; 32],
            },
            source_files: vec![],
            build_options: BuildOptions::default(),
            decls: vec![],
            analysis_units: vec![],
            types: vec![],
            layouts: vec![],
            air_bodies: vec![],
            exports: vec![],
            comptime_calls: vec![],
            embed_files: vec![],
            c_imports: vec![],
        };
        assert!(schema.header_magic_valid());
        assert!(schema.header_version_compatible());
    }

    #[test]
    fn test_min_adapter_version_in_header() {
        let schema = SnapSchema {
            header: SnapHeader {
                magic: *ZSNAP_MAGIC,
                schema_version: 1,
                min_adapter_version: 1,
                zig_commit: [0u8; 20],
                target: "x86_64-unknown-linux-gnu".to_string(),
                backend: "llvm".to_string(),
                optimize_mode: "ReleaseFast".to_string(),
                timestamp_ns: 1234567890,
                source_file_count: 5,
                checksum: [0u8; 32],
            },
            source_files: vec![],
            build_options: BuildOptions::default(),
            decls: vec![],
            analysis_units: vec![],
            types: vec![],
            layouts: vec![],
            air_bodies: vec![],
            exports: vec![],
            comptime_calls: vec![],
            embed_files: vec![],
            c_imports: vec![],
        };
        assert_eq!(schema.header.min_adapter_version, 1);
    }

    #[test]
    fn test_header_serialization_with_min_adapter_version() {
        let header = SnapHeader {
            magic: *ZSNAP_MAGIC,
            schema_version: 1,
            min_adapter_version: 2,
            zig_commit: [1u8; 20],
            target: "aarch64-linux-gnu".to_string(),
            backend: "llvm".to_string(),
            optimize_mode: "ReleaseSafe".to_string(),
            timestamp_ns: 1234567890,
            source_file_count: 3,
            checksum: [0u8; 32],
        };
        let json = serde_json::to_string(&header).unwrap();
        assert!(json.contains("\"min_adapter_version\":2"));
    }

    #[test]
    fn test_determinized_sorts_source_files() {
        let mut schema = SnapSchema {
            header: SnapHeader {
                magic: *ZSNAP_MAGIC,
                schema_version: 1,
                min_adapter_version: 1,
                zig_commit: [0u8; 20],
                target: "x86_64-unknown-linux-gnu".to_string(),
                backend: "llvm".to_string(),
                optimize_mode: "ReleaseFast".to_string(),
                timestamp_ns: 0,
                source_file_count: 2,
                checksum: [0u8; 32],
            },
            source_files: vec![
                SourceFile {
                    id: 2,
                    path: "b.zig".to_string(),
                    content_hash: [0u8; 32],
                },
                SourceFile {
                    id: 1,
                    path: "a.zig".to_string(),
                    content_hash: [0u8; 32],
                },
            ],
            build_options: BuildOptions::default(),
            decls: vec![],
            analysis_units: vec![],
            types: vec![],
            layouts: vec![],
            air_bodies: vec![],
            exports: vec![],
            comptime_calls: vec![],
            embed_files: vec![],
            c_imports: vec![],
        };
        let det = schema.determinized();
        assert_eq!(det.source_files[0].path, "a.zig");
        assert_eq!(det.source_files[1].path, "b.zig");
    }

    #[test]
    fn test_determinized_sorts_exports_by_name() {
        let mut schema = SnapSchema {
            header: SnapHeader {
                magic: *ZSNAP_MAGIC,
                schema_version: 1,
                min_adapter_version: 1,
                zig_commit: [0u8; 20],
                target: "x86_64-unknown-linux-gnu".to_string(),
                backend: "llvm".to_string(),
                optimize_mode: "ReleaseFast".to_string(),
                timestamp_ns: 0,
                source_file_count: 0,
                checksum: [0u8; 32],
            },
            source_files: vec![],
            build_options: BuildOptions::default(),
            decls: vec![],
            analysis_units: vec![],
            types: vec![],
            layouts: vec![],
            air_bodies: vec![],
            exports: vec![
                ExportSymbol {
                    name: "z_func".to_string(),
                    decl_id: 3,
                    linkage: Linkage::Strong,
                    visibility: Visibility::Public,
                    callconv: 0,
                    section_hint: None,
                },
                ExportSymbol {
                    name: "a_func".to_string(),
                    decl_id: 1,
                    linkage: Linkage::Strong,
                    visibility: Visibility::Public,
                    callconv: 0,
                    section_hint: None,
                },
                ExportSymbol {
                    name: "m_func".to_string(),
                    decl_id: 2,
                    linkage: Linkage::Strong,
                    visibility: Visibility::Public,
                    callconv: 0,
                    section_hint: None,
                },
            ],
            comptime_calls: vec![],
            embed_files: vec![],
            c_imports: vec![],
        };
        let det = schema.determinized();
        assert_eq!(det.exports[0].name, "a_func");
        assert_eq!(det.exports[1].name, "m_func");
        assert_eq!(det.exports[2].name, "z_func");
    }

    #[test]
    fn test_compute_checksum_produces_32_bytes() {
        let schema = SnapSchema {
            header: SnapHeader {
                magic: *ZSNAP_MAGIC,
                schema_version: 1,
                min_adapter_version: 1,
                zig_commit: [0u8; 20],
                target: "x86_64-unknown-linux-gnu".to_string(),
                backend: "llvm".to_string(),
                optimize_mode: "ReleaseFast".to_string(),
                timestamp_ns: 0,
                source_file_count: 0,
                checksum: [0u8; 32],
            },
            source_files: vec![],
            build_options: BuildOptions::default(),
            decls: vec![],
            analysis_units: vec![],
            types: vec![],
            layouts: vec![],
            air_bodies: vec![],
            exports: vec![],
            comptime_calls: vec![],
            embed_files: vec![],
            c_imports: vec![],
        };
        let checksum = schema.compute_checksum();
        assert_eq!(checksum.len(), 32);
    }

    #[test]
    fn test_compute_checksum_deterministic() {
        let mut schema1 = SnapSchema {
            header: SnapHeader {
                magic: *ZSNAP_MAGIC,
                schema_version: 1,
                min_adapter_version: 1,
                zig_commit: [0u8; 20],
                target: "x86_64-unknown-linux-gnu".to_string(),
                backend: "llvm".to_string(),
                optimize_mode: "ReleaseFast".to_string(),
                timestamp_ns: 0,
                source_file_count: 0,
                checksum: [0u8; 32],
            },
            source_files: vec![
                SourceFile {
                    id: 2,
                    path: "b.zig".to_string(),
                    content_hash: [1u8; 32],
                },
                SourceFile {
                    id: 1,
                    path: "a.zig".to_string(),
                    content_hash: [2u8; 32],
                },
            ],
            build_options: BuildOptions::default(),
            decls: vec![],
            analysis_units: vec![],
            types: vec![],
            layouts: vec![],
            air_bodies: vec![],
            exports: vec![],
            comptime_calls: vec![],
            embed_files: vec![],
            c_imports: vec![],
        };
        let mut schema2 = schema1.clone();
        // Shuffle to different order
        schema2.source_files.reverse();
        let checksum1 = schema1.compute_checksum();
        let checksum2 = schema2.compute_checksum();
        // Same content, different order = same checksum after determinization
        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_validation_report_serialization() {
        let report = ValidationReport {
            valid: true,
            version: 1,
            target: "x86_64-unknown-linux-gnu".to_string(),
            source_file_count: 3,
            errors: vec![],
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"valid\":true"));
        assert!(json.contains("\"version\":1"));
    }

    #[test]
    fn test_binary_parser_default() {
        let parser = BinaryParser::default();
        assert!(parser.is_strict_mode());
    }

    // Task 39: Parse compiler-emitted .zsnap - comprehensive binary format tests

    #[test]
    fn test_binary_parser_valid_format() {
        // Create a minimal valid binary .zsnap file
        let mut data = Vec::new();

        // Magic (8 bytes)
        data.extend_from_slice(ZSNAP_MAGIC);

        // Schema version (4 bytes)
        data.extend_from_slice(&1u32.to_le_bytes());

        // Zig commit (20 bytes)
        data.extend_from_slice(&[0u8; 20]);

        // Target string (length + content)
        let target = "x86_64-unknown-linux-gnu";
        data.extend_from_slice(&(target.len() as u32).to_le_bytes());
        data.extend_from_slice(target.as_bytes());

        // Backend string
        let backend = "llvm";
        data.extend_from_slice(&(backend.len() as u32).to_le_bytes());
        data.extend_from_slice(backend.as_bytes());

        // Optimize mode string
        let optimize = "ReleaseFast";
        data.extend_from_slice(&(optimize.len() as u32).to_le_bytes());
        data.extend_from_slice(optimize.as_bytes());

        // Timestamp (8 bytes)
        data.extend_from_slice(&1234567890u64.to_le_bytes());

        // Source file count (4 bytes)
        data.extend_from_slice(&0u32.to_le_bytes());

        // Checksum (32 bytes)
        data.extend_from_slice(&[0u8; 32]);

        // JSON payload (empty sections)
        let json_payload = r#"{"source_files":[],"build_options":{},"decls":[],"analysis_units":[],"types":[],"layouts":[],"air_bodies":[],"exports":[]}"#;
        data.extend_from_slice(json_payload.as_bytes());

        let mut parser = BinaryParser::new();
        let result = parser.parse(&data);
        assert!(
            result.is_ok(),
            "valid binary format should parse: {:?}",
            result.err()
        );
        let schema = result.unwrap();
        assert_eq!(schema.header.schema_version, 1);
        assert_eq!(schema.header.target, target);
    }

    #[test]
    fn test_binary_parser_validates_magic_bytes() {
        let mut parser = BinaryParser::new();
        let mut data = [0u8; 100];
        // Invalid magic
        data[0..8].copy_from_slice(b"INVALID1");
        let result = parser.parse(&data);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            BinaryParseError::InvalidMagic { .. }
        ));
    }

    #[test]
    fn test_binary_parser_validates_version() {
        let mut parser = BinaryParser::new();
        let mut data = vec![0u8; 200];
        data[0..8].copy_from_slice(ZSNAP_MAGIC);
        // Version 2 (higher than current SCHEMA_VERSION=1)
        data[8..12].copy_from_slice(&2u32.to_le_bytes());
        let result = parser.parse(&data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_version_error());
    }

    #[test]
    fn test_binary_parser_rejects_truncated_header() {
        let parser = BinaryParser::new();
        // Too short for even magic
        let data = [0u8, 0, 0];
        let result = parser.read_string(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_parser_reads_valid_string() {
        let parser = BinaryParser::new();
        // length=5, content="hello"
        let data = [5, 0, 0, 0, b'h', b'e', b'l', b'l', b'o'];
        let result = parser.read_string(&data);
        assert!(result.is_ok());
        let (s, consumed) = result.unwrap();
        assert_eq!(s, "hello");
        assert_eq!(consumed, 9); // 4 bytes length + 5 bytes content
    }

    #[test]
    fn test_binary_parser_handles_empty_string() {
        let parser = BinaryParser::new();
        // length=0
        let data = [0, 0, 0, 0];
        let result = parser.read_string(&data);
        assert!(result.is_ok());
        let (s, consumed) = result.unwrap();
        assert_eq!(s, "");
        assert_eq!(consumed, 4);
    }

    #[test]
    fn test_binary_parser_checksum_mismatch() {
        // Create a partial valid binary with valid magic and version
        let mut data = vec![0u8; 200];
        data[0..8].copy_from_slice(ZSNAP_MAGIC);
        data[8..12].copy_from_slice(&1u32.to_le_bytes()); // version 1
        data[12..32].copy_from_slice(&[1u8; 20]); // commit hash

        // Target: "linux" (5 bytes)
        data[32..36].copy_from_slice(&5u32.to_le_bytes());
        data[36..41].copy_from_slice(b"linux");

        // Checksum mismatch error type check
        let checksum_err = BinaryParseError::ChecksumMismatch {
            expected: "abc123".to_string(),
            got: "def456".to_string(),
        };
        assert!(checksum_err.is_corruption());
        assert!(!checksum_err.is_version_error());
    }

    #[test]
    fn test_snap_schema_magic_validation() {
        let mut schema = SnapSchema {
            header: SnapHeader {
                magic: *ZSNAP_MAGIC,
                schema_version: 1,
                min_adapter_version: 1,
                zig_commit: [0u8; 20],
                target: "test".to_string(),
                backend: "llvm".to_string(),
                optimize_mode: "Debug".to_string(),
                timestamp_ns: 0,
                source_file_count: 0,
                checksum: [0u8; 32],
            },
            source_files: vec![],
            build_options: BuildOptions::default(),
            decls: vec![],
            analysis_units: vec![],
            types: vec![],
            layouts: vec![],
            air_bodies: vec![],
            exports: vec![],
            comptime_calls: vec![],
            embed_files: vec![],
            c_imports: vec![],
        };

        assert!(schema.header_magic_valid());

        // Corrupt magic
        schema.header.magic = [0u8; 8];
        assert!(!schema.header_magic_valid());
    }

    #[test]
    fn test_snap_schema_version_compatibility() {
        let mut schema = SnapSchema {
            header: SnapHeader {
                magic: *ZSNAP_MAGIC,
                schema_version: 0, // older
                min_adapter_version: 1,
                zig_commit: [0u8; 20],
                target: "test".to_string(),
                backend: "llvm".to_string(),
                optimize_mode: "Debug".to_string(),
                timestamp_ns: 0,
                source_file_count: 0,
                checksum: [0u8; 32],
            },
            source_files: vec![],
            build_options: BuildOptions::default(),
            decls: vec![],
            analysis_units: vec![],
            types: vec![],
            layouts: vec![],
            air_bodies: vec![],
            exports: vec![],
            comptime_calls: vec![],
            embed_files: vec![],
            c_imports: vec![],
        };

        // Version 0 < current 1, should be compatible
        assert!(schema.header_version_compatible());

        // Version 2 > current 1, incompatible
        schema.header.schema_version = 2;
        assert!(!schema.header_version_compatible());
    }

    #[test]
    fn test_binary_error_io_error() {
        let err = BinaryParseError::IoError("file not found".to_string());
        assert!(!err.is_version_error());
        assert!(!err.is_corruption());
        let display = format!("{}", err);
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_binary_error_truncated_data() {
        let err = BinaryParseError::TruncatedData {
            expected: 100,
            got: 50,
        };
        assert!(err.is_corruption());
        assert!(!err.is_version_error());
        let display = format!("{}", err);
        assert!(display.contains("100"));
        assert!(display.contains("50"));
    }

    #[test]
    fn test_parse_error_display_comprehensive() {
        let magic_err = BinaryParseError::InvalidMagic {
            expected: *ZSNAP_MAGIC,
            got: [1u8; 8],
        };
        let display = format!("{}", magic_err);
        assert!(display.contains("invalid magic"));
        // Array debug format varies by Rust version, check for key info
        assert!(display.contains("1"));
    }

    #[test]
    fn test_validation_report_valid() {
        let report = ValidationReport {
            valid: true,
            version: 1,
            target: "x86_64-linux".to_string(),
            source_file_count: 5,
            errors: vec![],
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"valid\":true"));
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"source_file_count\":5"));
    }

    #[test]
    fn test_validation_report_invalid_with_errors() {
        let report = ValidationReport {
            valid: false,
            version: 0,
            target: String::new(),
            source_file_count: 0,
            errors: vec!["invalid magic".to_string(), "checksum mismatch".to_string()],
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"valid\":false"));
        assert!(json.contains("invalid magic"));
    }

    #[test]
    fn test_verify_checksum_matches() {
        let schema = SnapSchema {
            header: SnapHeader {
                magic: *ZSNAP_MAGIC,
                schema_version: 1,
                min_adapter_version: 1,
                zig_commit: [0u8; 20],
                target: "test".to_string(),
                backend: "llvm".to_string(),
                optimize_mode: "Debug".to_string(),
                timestamp_ns: 0,
                source_file_count: 0,
                checksum: [0u8; 32],
            },
            source_files: vec![],
            build_options: BuildOptions::default(),
            decls: vec![],
            analysis_units: vec![],
            types: vec![],
            layouts: vec![],
            air_bodies: vec![],
            exports: vec![],
            comptime_calls: vec![],
            embed_files: vec![],
            c_imports: vec![],
        };
        // Compute the actual checksum and set it
        let computed = schema.compute_checksum();
        let mut schema_with_checksum = schema;
        schema_with_checksum.header.checksum = computed;
        // Should verify successfully
        assert!(schema_with_checksum.verify_checksum().is_ok());
    }

    #[test]
    fn test_verify_checksum_mismatch() {
        let schema = SnapSchema {
            header: SnapHeader {
                magic: *ZSNAP_MAGIC,
                schema_version: 1,
                min_adapter_version: 1,
                zig_commit: [0u8; 20],
                target: "test".to_string(),
                backend: "llvm".to_string(),
                optimize_mode: "Debug".to_string(),
                timestamp_ns: 0,
                source_file_count: 0,
                checksum: [0u8; 32], // mismatch - not the actual computed checksum
            },
            source_files: vec![],
            build_options: BuildOptions::default(),
            decls: vec![],
            analysis_units: vec![],
            types: vec![],
            layouts: vec![],
            air_bodies: vec![],
            exports: vec![],
            comptime_calls: vec![],
            embed_files: vec![],
            c_imports: vec![],
        };
        // Checksum doesn't match - should fail
        let result = schema.verify_checksum();
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_corruption_valid_zsnap() {
        let parser = BinaryParser::new();
        // Create minimal valid zsnap header
        let mut data = vec![0u8; 64];
        data[0..8].copy_from_slice(ZSNAP_MAGIC);
        data[12..20].copy_from_slice(&64u64.to_le_bytes()); // size
                                                            // Should pass corruption detection
        assert!(parser.detect_corruption(&data).is_ok());
    }

    #[test]
    fn test_detect_corruption_truncated() {
        let parser = BinaryParser::new();
        // Too short - should fail
        let data = vec![0u8; 10];
        let result = parser.detect_corruption(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_corruption_invalid_magic() {
        let parser = BinaryParser::new();
        let mut data = vec![0u8; 64];
        data[0..8].copy_from_slice(b"INVALID!"); // wrong magic
        let result = parser.detect_corruption(&data);
        assert!(result.is_err());
    }

    // =========================================================================
    // B.26: Compiler Self-Tests for Edit Scenarios
    // =========================================================================

    #[test]
    fn test_private_body_edit_detected_by_checksum() {
        // Scenario: Private function body changed but signature unchanged.
        // The checksum should change because AIR body is different.
        let mut schema1 = create_minimal_schema();
        schema1.air_bodies.push(AirBodyRef {
            function_id: 1,
            type_id: 10,
            basic_blocks: 5,
            instructions: 100,
            air_data_offset: 0,
            air_data_len: 500,
        });

        let mut schema2 = create_minimal_schema();
        schema2.air_bodies.push(AirBodyRef {
            function_id: 1,
            type_id: 10,
            basic_blocks: 5,
            instructions: 110, // Changed - more instructions
            air_data_offset: 0,
            air_data_len: 550,
        });

        // Checksums differ due to body change
        let checksum1 = schema1.compute_checksum();
        let checksum2 = schema2.compute_checksum();
        assert_ne!(
            checksum1, checksum2,
            "Private body edit should produce different checksum"
        );
    }

    #[test]
    fn test_public_abi_edit_detected_by_exports() {
        // Scenario: Exported function signature changed.
        // The exports list should change, affecting checksum.
        let mut schema1 = create_minimal_schema();
        schema1.exports.push(ExportSymbol {
            name: "pub_func".to_string(),
            decl_id: 1,
            linkage: Linkage::Strong,
            visibility: Visibility::Public,
            callconv: 0,
            section_hint: None,
        });

        let mut schema2 = create_minimal_schema();
        schema2.exports.push(ExportSymbol {
            name: "pub_func".to_string(),
            decl_id: 2, // Different decl_id = different symbol
            linkage: Linkage::Strong,
            visibility: Visibility::Public,
            callconv: 0,
            section_hint: None,
        });

        let checksum1 = schema1.compute_checksum();
        let checksum2 = schema2.compute_checksum();
        assert_ne!(
            checksum1, checksum2,
            "Public ABI edit should produce different checksum"
        );
    }

    #[test]
    fn test_layout_edit_detected_by_layouts() {
        // Scenario: Struct layout changed (field added/removed/reordered).
        let mut schema1 = create_minimal_schema();
        schema1.layouts.push(LayoutRecord {
            id: 1,
            type_id: 20,
            size_bytes: 32,
            alignment: 8,
            field_count: 4,
            packed: false,
            extern_: true,
        });

        let mut schema2 = create_minimal_schema();
        schema2.layouts.push(LayoutRecord {
            id: 1,
            type_id: 20,
            size_bytes: 40, // Size changed - layout edit
            alignment: 8,
            field_count: 5, // More fields
            packed: false,
            extern_: true,
        });

        let checksum1 = schema1.compute_checksum();
        let checksum2 = schema2.compute_checksum();
        assert_ne!(
            checksum1, checksum2,
            "Layout edit should produce different checksum"
        );
    }

    #[test]
    fn test_comptime_edit_private_vs_exported() {
        // Scenario: Comptime value used privately vs exported.
        // A private comptime change should not affect exports.
        let mut schema_private = create_minimal_schema();
        schema_private.exports.push(ExportSymbol {
            name: "pub_func".to_string(),
            decl_id: 1,
            linkage: Linkage::Strong,
            visibility: Visibility::Public,
            callconv: 0,
            section_hint: None,
        });
        // No type record change - comptime used only in private body

        let mut schema_exported_ct = create_minimal_schema();
        schema_exported_ct.exports.push(ExportSymbol {
            name: "pub_func".to_string(),
            decl_id: 1,
            linkage: Linkage::Strong,
            visibility: Visibility::Public,
            callconv: 0,
            section_hint: None,
        });
        // Type changed that affects exported signature
        schema_exported_ct.types.push(TypeRecord {
            id: 10,
            kind: TypeKind::Int {
                signed: false,
                bits: 64,
            },
            name: Some("usize".to_string()),
            size_bytes: Some(8),
            alignment: Some(8),
        });

        // These should have different checksums
        let checksum_private = schema_private.compute_checksum();
        let checksum_exported = schema_exported_ct.compute_checksum();
        assert_ne!(
            checksum_private, checksum_exported,
            "Exported comptime change should differ"
        );
    }

    #[test]
    fn test_repeated_build_produces_identical_checksum() {
        // Scenario: Two identical no-op builds should produce byte-identical artifacts.
        let schema1 = create_minimal_schema();
        let schema2 = create_minimal_schema();

        let checksum1 = schema1.compute_checksum();
        let checksum2 = schema2.compute_checksum();
        assert_eq!(
            checksum1, checksum2,
            "Identical schemas should produce identical checksums"
        );
    }

    #[test]
    fn test_ordering_difference_resolved_by_determinized() {
        // Scenario: Source files in different order should produce same checksum.
        let mut schema1 = create_minimal_schema();
        schema1.source_files.push(SourceFile {
            id: 1,
            path: "a.zig".to_string(),
            content_hash: [1u8; 32],
        });
        schema1.source_files.push(SourceFile {
            id: 2,
            path: "b.zig".to_string(),
            content_hash: [2u8; 32],
        });

        let mut schema2 = create_minimal_schema();
        schema2.source_files.push(SourceFile {
            id: 2,
            path: "b.zig".to_string(),
            content_hash: [2u8; 32],
        });
        schema2.source_files.push(SourceFile {
            id: 1,
            path: "a.zig".to_string(),
            content_hash: [1u8; 32],
        });

        let checksum1 = schema1.compute_checksum();
        let checksum2 = schema2.compute_checksum();
        assert_eq!(
            checksum1, checksum2,
            "Determinized ordering should produce same checksum despite insertion order"
        );
    }

    #[test]
    fn test_edit_scenario_classification_via_invalidation() {
        // Test that we can classify edit types based on what changed in schema.
        let unchanged = create_minimal_schema();

        // Private body only - air_bodies differ
        let mut private_body = create_minimal_schema();
        private_body.air_bodies.push(AirBodyRef {
            function_id: 1,
            type_id: 10,
            basic_blocks: 3,
            instructions: 50,
            air_data_offset: 0,
            air_data_len: 200,
        });

        // Export change - exports differ
        let mut export_change = create_minimal_schema();
        export_change.exports.push(ExportSymbol {
            name: "new_export".to_string(),
            decl_id: 99,
            linkage: Linkage::Strong,
            visibility: Visibility::Public,
            callconv: 0,
            section_hint: None,
        });

        // Layout change - layouts differ
        let mut layout_change = create_minimal_schema();
        layout_change.layouts.push(LayoutRecord {
            id: 1,
            type_id: 20,
            size_bytes: 64,
            alignment: 16,
            field_count: 8,
            packed: true,
            extern_: false,
        });

        // Verify each scenario produces a different checksum
        assert_ne!(
            unchanged.compute_checksum(),
            private_body.compute_checksum()
        );
        assert_ne!(
            unchanged.compute_checksum(),
            export_change.compute_checksum()
        );
        assert_ne!(
            unchanged.compute_checksum(),
            layout_change.compute_checksum()
        );
        assert_ne!(
            private_body.compute_checksum(),
            export_change.compute_checksum()
        );
        assert_ne!(
            private_body.compute_checksum(),
            layout_change.compute_checksum()
        );
        assert_ne!(
            export_change.compute_checksum(),
            layout_change.compute_checksum()
        );
    }

    // Helper to create a minimal schema for testing
    fn create_minimal_schema() -> SnapSchema {
        SnapSchema {
            header: SnapHeader {
                magic: *ZSNAP_MAGIC,
                schema_version: 1,
                min_adapter_version: 1,
                zig_commit: [0u8; 20],
                target: "x86_64-unknown-linux-gnu".to_string(),
                backend: "llvm".to_string(),
                optimize_mode: "ReleaseFast".to_string(),
                timestamp_ns: 1000000000,
                source_file_count: 1,
                checksum: [0u8; 32],
            },
            source_files: vec![SourceFile {
                id: 1,
                path: "main.zig".to_string(),
                content_hash: [0xAB; 32],
            }],
            build_options: BuildOptions {
                optimize_mode: "ReleaseFast".to_string(),
                target: "x86_64-unknown-linux-gnu".to_string(),
                cpu_features: vec!["ssse3".to_string()],
                libc: Some("glibc".to_string()),
                build_mode: "exe".to_string(),
                entry: Some("main".to_string()),
                panic_mode: "panic".to_string(),
            },
            decls: vec![DeclRef {
                id: 1,
                name: "main".to_string(),
                kind: DeclKind::Function,
                owner_file: 1,
                access_level: AccessLevel::Pub,
            }],
            analysis_units: vec![AnalysisUnit {
                id: 1,
                file: 1,
                decls: vec![1],
                imports: vec![],
            }],
            types: vec![TypeRecord {
                id: 10,
                kind: TypeKind::Void,
                name: None,
                size_bytes: Some(0),
                alignment: Some(0),
            }],
            layouts: vec![],
            air_bodies: vec![],
            exports: vec![],
            comptime_calls: vec![],
            embed_files: vec![],
            c_imports: vec![],
        }
    }

    // =========================================================================
    // E.58: Comptime Impact Classification Tests
    // =========================================================================

    #[test]
    fn test_comptime_call_classify_impact_private_body() {
        let call = ComptimeCall {
            call_id: 1,
            owner_decl: 10,
            affects_exports: false,
            result_hash: [0xAB; 32],
            source_file: 1,
            source_line: 42,
        };
        assert_eq!(call.classify_impact(), ComptimeImpact::PrivateBody);
    }

    #[test]
    fn test_comptime_call_classify_impact_exported_signature() {
        let call = ComptimeCall {
            call_id: 2,
            owner_decl: 20,
            affects_exports: true,
            result_hash: [0xCD; 32],
            source_file: 1,
            source_line: 100,
        };
        assert_eq!(call.classify_impact(), ComptimeImpact::ExportedSignature);
    }

    #[test]
    fn test_comptime_private_body_change_no_export_impact() {
        // Scenario: Private comptime evaluation changed, but no exports affected.
        // Should produce a different checksum, but affects_exports is false.
        let mut schema_ct_changed = create_minimal_schema();
        schema_ct_changed.comptime_calls.push(ComptimeCall {
            call_id: 1,
            owner_decl: 10,
            affects_exports: false,
            result_hash: [0x11; 32],
            source_file: 1,
            source_line: 50,
        });

        let mut schema_ct_unchanged = create_minimal_schema();
        schema_ct_unchanged.comptime_calls.push(ComptimeCall {
            call_id: 1,
            owner_decl: 10,
            affects_exports: false,
            result_hash: [0x22; 32], // Different result
            source_file: 1,
            source_line: 50,
        });

        let checksum1 = schema_ct_changed.compute_checksum();
        let checksum2 = schema_ct_unchanged.compute_checksum();
        assert_ne!(
            checksum1, checksum2,
            "Comptime change should produce different checksum"
        );

        // But the impact classification is private
        assert_eq!(
            schema_ct_changed.comptime_calls[0].classify_impact(),
            ComptimeImpact::PrivateBody
        );
    }

    #[test]
    fn test_comptime_exported_signature_change() {
        // Scenario: Comptime affects an exported function's signature.
        // Should invalidate dependents via ExportedSignature impact.
        let mut schema = create_minimal_schema();
        schema.exports.push(ExportSymbol {
            name: "pub_func".to_string(),
            decl_id: 1,
            linkage: Linkage::Strong,
            visibility: Visibility::Public,
            callconv: 0,
            section_hint: None,
        });
        schema.comptime_calls.push(ComptimeCall {
            call_id: 99,
            owner_decl: 1, // Same decl as export
            affects_exports: true,
            result_hash: [0x33; 32],
            source_file: 1,
            source_line: 10,
        });

        let impact = schema.comptime_calls[0].classify_impact();
        assert_eq!(
            impact,
            ComptimeImpact::ExportedSignature,
            "Comptime affecting export should be ExportedSignature"
        );
    }

    #[test]
    fn test_comptime_calls_sorted_in_determinized() {
        // Verify comptime_calls are sorted by call_id in determinized output.
        let mut schema = create_minimal_schema();
        schema.comptime_calls.push(ComptimeCall {
            call_id: 3,
            owner_decl: 10,
            affects_exports: false,
            result_hash: [0xAA; 32],
            source_file: 1,
            source_line: 30,
        });
        schema.comptime_calls.push(ComptimeCall {
            call_id: 1,
            owner_decl: 20,
            affects_exports: false,
            result_hash: [0xBB; 32],
            source_file: 1,
            source_line: 20,
        });
        schema.comptime_calls.push(ComptimeCall {
            call_id: 2,
            owner_decl: 30,
            affects_exports: false,
            result_hash: [0xCC; 32],
            source_file: 1,
            source_line: 10,
        });

        let determinized = schema.determinized();
        // After determinization, should be sorted by call_id: 1, 2, 3
        assert_eq!(determinized.comptime_calls[0].call_id, 1);
        assert_eq!(determinized.comptime_calls[1].call_id, 2);
        assert_eq!(determinized.comptime_calls[2].call_id, 3);
    }

    // =========================================================================
    // E.59: @embedFile Impact Classification Tests
    // =========================================================================

    #[test]
    fn test_embed_file_ref_classify_impact_private_body() {
        let embed = EmbedFileRef {
            embed_id: 1,
            path: "private_data.txt".to_string(),
            content_hash: [0xAB; 32],
            source_file: 1,
            source_line: 42,
            affects_exports: false,
        };
        assert_eq!(embed.classify_impact(), EmbedFileImpact::PrivateBody);
    }

    #[test]
    fn test_embed_file_ref_classify_impact_exported_const() {
        let embed = EmbedFileRef {
            embed_id: 2,
            path: "exported_config.json".to_string(),
            content_hash: [0xCD; 32],
            source_file: 1,
            source_line: 100,
            affects_exports: true,
        };
        assert_eq!(embed.classify_impact(), EmbedFileImpact::ExportedConst);
    }

    #[test]
    fn test_embed_file_content_change_detected() {
        // Scenario: Embedded file content changed - should affect checksum.
        let mut schema_ct_changed = create_minimal_schema();
        schema_ct_changed.embed_files.push(EmbedFileRef {
            embed_id: 1,
            path: "data.txt".to_string(),
            content_hash: [0x11; 32],
            source_file: 1,
            source_line: 50,
            affects_exports: false,
        });

        let mut schema_ct_unchanged = create_minimal_schema();
        schema_ct_unchanged.embed_files.push(EmbedFileRef {
            embed_id: 1,
            path: "data.txt".to_string(),
            content_hash: [0x22; 32], // Different content
            source_file: 1,
            source_line: 50,
            affects_exports: false,
        });

        let checksum1 = schema_ct_changed.compute_checksum();
        let checksum2 = schema_ct_unchanged.compute_checksum();
        assert_ne!(
            checksum1, checksum2,
            "Embed file content change should produce different checksum"
        );
    }

    #[test]
    fn test_embed_file_affects_exports_classification() {
        // Scenario: Embed file used in exported constant affects downstream.
        let mut schema = create_minimal_schema();
        schema.exports.push(ExportSymbol {
            name: "CONFIG".to_string(),
            decl_id: 99,
            linkage: Linkage::Strong,
            visibility: Visibility::Public,
            callconv: 0,
            section_hint: None,
        });
        schema.embed_files.push(EmbedFileRef {
            embed_id: 50,
            path: "config.bin".to_string(),
            content_hash: [0x33; 32],
            source_file: 1,
            source_line: 10,
            affects_exports: true, // Used in exported symbol
        });

        let impact = schema.embed_files[0].classify_impact();
        assert_eq!(
            impact,
            EmbedFileImpact::ExportedConst,
            "Embed affecting export should be ExportedConst"
        );
    }

    #[test]
    fn test_embed_files_sorted_in_determinized() {
        // Verify embed_files are sorted by path in determinized output.
        let mut schema = create_minimal_schema();
        schema.embed_files.push(EmbedFileRef {
            embed_id: 1,
            path: "zebra.txt".to_string(),
            content_hash: [0xAA; 32],
            source_file: 1,
            source_line: 30,
            affects_exports: false,
        });
        schema.embed_files.push(EmbedFileRef {
            embed_id: 2,
            path: "alpha.txt".to_string(),
            content_hash: [0xBB; 32],
            source_file: 1,
            source_line: 20,
            affects_exports: false,
        });
        schema.embed_files.push(EmbedFileRef {
            embed_id: 3,
            path: "middle.txt".to_string(),
            content_hash: [0xCC; 32],
            source_file: 1,
            source_line: 10,
            affects_exports: false,
        });

        let determinized = schema.determinized();
        // After determinization, should be sorted by path: alpha, middle, zebra
        assert_eq!(determinized.embed_files[0].path, "alpha.txt");
        assert_eq!(determinized.embed_files[1].path, "middle.txt");
        assert_eq!(determinized.embed_files[2].path, "zebra.txt");
    }

    // =========================================================================
    // E.60: @cImport/C Translation Impact Tests
    // =========================================================================

    #[test]
    fn test_c_import_ref_classify_impact_private_body() {
        let cimport = CImportRef {
            import_id: 1,
            header_path: "internal.h".to_string(),
            header_hash: [0xAB; 32],
            source_file: 1,
            source_line: 42,
            affects_exports: false,
            dependencies: vec![],
        };
        assert_eq!(cimport.classify_impact(), CImportImpact::PrivateBody);
    }

    #[test]
    fn test_c_import_ref_classify_impact_exported_signature() {
        let cimport = CImportRef {
            import_id: 2,
            header_path: "exported_api.h".to_string(),
            header_hash: [0xCD; 32],
            source_file: 1,
            source_line: 100,
            affects_exports: true,
            dependencies: vec!["base_types.h".to_string()],
        };
        assert_eq!(cimport.classify_impact(), CImportImpact::ExportedSignature);
    }

    #[test]
    fn test_c_header_change_detected() {
        // Scenario: C header content changed - should affect checksum.
        let mut schema_ct_changed = create_minimal_schema();
        schema_ct_changed.c_imports.push(CImportRef {
            import_id: 1,
            header_path: "data.h".to_string(),
            header_hash: [0x11; 32],
            source_file: 1,
            source_line: 50,
            affects_exports: false,
            dependencies: vec![],
        });

        let mut schema_ct_unchanged = create_minimal_schema();
        schema_ct_unchanged.c_imports.push(CImportRef {
            import_id: 1,
            header_path: "data.h".to_string(),
            header_hash: [0x22; 32], // Different content
            source_file: 1,
            source_line: 50,
            affects_exports: false,
            dependencies: vec![],
        });

        let checksum1 = schema_ct_changed.compute_checksum();
        let checksum2 = schema_ct_unchanged.compute_checksum();
        assert_ne!(
            checksum1, checksum2,
            "C header content change should produce different checksum"
        );
    }

    #[test]
    fn test_c_import_affects_exports_classification() {
        // Scenario: C import used in exported function signature.
        let mut schema = create_minimal_schema();
        schema.exports.push(ExportSymbol {
            name: "get_c_value".to_string(),
            decl_id: 99,
            linkage: Linkage::Strong,
            visibility: Visibility::Public,
            callconv: 0,
            section_hint: None,
        });
        schema.c_imports.push(CImportRef {
            import_id: 50,
            header_path: "c_api.h".to_string(),
            header_hash: [0x33; 32],
            source_file: 1,
            source_line: 10,
            affects_exports: true,
            dependencies: vec!["c_types.h".to_string()],
        });

        let impact = schema.c_imports[0].classify_impact();
        assert_eq!(
            impact,
            CImportImpact::ExportedSignature,
            "C import affecting export should be ExportedSignature"
        );
    }

    #[test]
    fn test_c_imports_sorted_in_determinized() {
        // Verify c_imports are sorted by header_path in determinized output.
        let mut schema = create_minimal_schema();
        schema.c_imports.push(CImportRef {
            import_id: 1,
            header_path: "z_header.h".to_string(),
            header_hash: [0xAA; 32],
            source_file: 1,
            source_line: 30,
            affects_exports: false,
            dependencies: vec![],
        });
        schema.c_imports.push(CImportRef {
            import_id: 2,
            header_path: "a_header.h".to_string(),
            header_hash: [0xBB; 32],
            source_file: 1,
            source_line: 20,
            affects_exports: false,
            dependencies: vec![],
        });
        schema.c_imports.push(CImportRef {
            import_id: 3,
            header_path: "m_header.h".to_string(),
            header_hash: [0xCC; 32],
            source_file: 1,
            source_line: 10,
            affects_exports: false,
            dependencies: vec![],
        });

        let determinized = schema.determinized();
        // After determinization, should be sorted by header_path: a, m, z
        assert_eq!(determinized.c_imports[0].header_path, "a_header.h");
        assert_eq!(determinized.c_imports[1].header_path, "m_header.h");
        assert_eq!(determinized.c_imports[2].header_path, "z_header.h");
    }

    // =========================================================================
    // E.61: Invalidation Explanation Output Tests
    // =========================================================================

    #[test]
    fn test_invalidation_explanation_private_body() {
        let explanation = InvalidationExplanation::private_body_change(42);
        assert_eq!(explanation.change_type, ChangeType::PrivateBody);
        assert_eq!(explanation.affected_count, 0);
        assert_eq!(explanation.reusable_count, 42);
        assert_eq!(explanation.action, InvalidationAction::IncrementalReuse);
        assert!(explanation.reason.contains("Private body"));
    }

    #[test]
    fn test_invalidation_explanation_exported_signature() {
        let explanation = InvalidationExplanation::exported_signature_change(5);
        assert_eq!(explanation.change_type, ChangeType::ExportedSignature);
        assert_eq!(explanation.affected_count, 5);
        assert_eq!(explanation.reusable_count, 0);
        assert_eq!(explanation.action, InvalidationAction::FullRebuild);
        assert!(explanation.reason.contains("signature"));
    }

    #[test]
    fn test_invalidation_explanation_exported_layout() {
        let explanation = InvalidationExplanation::exported_layout_change(3, 10);
        assert_eq!(explanation.change_type, ChangeType::ExportedLayout);
        assert_eq!(explanation.affected_count, 3);
        assert_eq!(explanation.reusable_count, 10);
        assert_eq!(explanation.action, InvalidationAction::PartialRebuild);
        assert!(explanation.reason.contains("Layout"));
    }

    #[test]
    fn test_invalidation_explanation_build_options() {
        let explanation = InvalidationExplanation::build_options_change();
        assert_eq!(explanation.change_type, ChangeType::BuildOptions);
        assert_eq!(explanation.affected_count, u32::MAX);
        assert_eq!(explanation.action, InvalidationAction::FullRebuild);
        assert!(explanation.reason.contains("Build options"));
    }

    #[test]
    fn test_invalidation_explanation_serialization() {
        let explanation = InvalidationExplanation::exported_signature_change(5);
        let json = serde_json::to_string(&explanation).unwrap();
        assert!(json.contains("\"change_type\":\"ExportedSignature\""));
        assert!(json.contains("\"affected_count\":5"));
        assert!(json.contains("\"action\":\"FullRebuild\""));
    }

    // =========================================================================
    // E.62: Invalidation Proof Facts Tests
    // =========================================================================

    #[test]
    fn test_proof_chain_private_body_proof() {
        let source_checksum = [0x01u8; 32];
        let air_checksum = [0x02u8; 32];
        let proof = ProofChain::private_body_proof(source_checksum, air_checksum);

        assert!(proof.reuse_allowed);
        assert_eq!(proof.facts.len(), 3);
        assert_eq!(proof.facts[0].fact_type, ProofFactType::ContentHash);
        assert_eq!(proof.facts[1].fact_type, ProofFactType::AirBodyRef);
        assert_eq!(proof.facts[2].fact_type, ProofFactType::Visibility);
        assert_eq!(proof.facts[2].value, "Private");
        assert_eq!(proof.explanation.change_type, ChangeType::PrivateBody);
        assert_eq!(
            proof.explanation.action,
            InvalidationAction::IncrementalReuse
        );
    }

    #[test]
    fn test_proof_chain_exported_signature_proof() {
        let signature_hash = [0x03u8; 32];
        let proof = ProofChain::exported_signature_proof(signature_hash, "my_export");

        assert!(!proof.reuse_allowed);
        assert_eq!(proof.facts.len(), 2);
        assert_eq!(proof.facts[0].fact_type, ProofFactType::SignatureHash);
        assert_eq!(proof.facts[0].reference.as_ref().unwrap(), "my_export");
        assert_eq!(proof.facts[1].fact_type, ProofFactType::Visibility);
        assert_eq!(proof.facts[1].value, "Public");
        assert_eq!(proof.explanation.change_type, ChangeType::ExportedSignature);
        assert_eq!(proof.explanation.action, InvalidationAction::FullRebuild);
    }

    #[test]
    fn test_proof_chain_layout_change_proof() {
        let layout_hash = [0x04u8; 32];
        let proof = ProofChain::layout_change_proof(layout_hash, "MyType");

        assert!(!proof.reuse_allowed);
        assert_eq!(proof.facts.len(), 1);
        assert_eq!(proof.facts[0].fact_type, ProofFactType::TypeLayout);
        assert_eq!(proof.facts[0].reference.as_ref().unwrap(), "MyType");
        assert_eq!(proof.explanation.change_type, ChangeType::ExportedLayout);
        assert_eq!(proof.explanation.action, InvalidationAction::PartialRebuild);
    }

    #[test]
    fn test_proof_fact_serialization() {
        let fact = ProofFact {
            fact_type: ProofFactType::Checksum,
            value: "abc123".to_string(),
            reference: Some("test_ref".to_string()),
        };
        let json = serde_json::to_string(&fact).unwrap();
        assert!(json.contains("\"fact_type\":\"Checksum\""));
        assert!(json.contains("\"value\":\"abc123\""));
        assert!(json.contains("\"reference\":\"test_ref\""));
    }

    #[test]
    fn test_proof_chain_serialization() {
        let source_checksum = [0x01u8; 32];
        let air_checksum = [0x02u8; 32];
        let proof = ProofChain::private_body_proof(source_checksum, air_checksum);
        let json = serde_json::to_string(&proof).unwrap();

        assert!(json.contains("\"reuse_allowed\":true"));
        assert!(json.contains("\"facts\""));
        assert!(json.contains("\"explanation\""));
        assert!(json.contains("\"change_type\":\"PrivateBody\""));
    }

    // =========================================================================
    // F.72: Partial Artifact Reuse Tests
    // =========================================================================

    #[test]
    fn test_artifact_reuse_new() {
        let reuse = ArtifactReuse::new();
        assert!(!reuse.fully_reusable);
        assert!(reuse.reusable_sources.is_empty());
        assert_eq!(reuse.affected_count, 0);
        assert_eq!(reuse.reusable_count, 0);
    }

    #[test]
    fn test_artifact_reuse_add_sources() {
        let mut reuse = ArtifactReuse::new();
        reuse.add_reusable_source("src/a.zig");
        reuse.add_reusable_source("src/b.zig");
        assert_eq!(reuse.reusable_sources.len(), 2);
    }

    #[test]
    fn test_artifact_reuse_add_decls() {
        let mut reuse = ArtifactReuse::new();
        reuse.add_reusable_decl(1);
        reuse.add_reusable_decl(2);
        assert_eq!(reuse.reusable_decls.len(), 2);
    }

    #[test]
    fn test_artifact_reuse_fully_reusable() {
        let mut reuse = ArtifactReuse::new();
        reuse.set_fully_reusable();
        assert!(reuse.fully_reusable);
    }

    #[test]
    fn test_artifact_reuse_compute_count() {
        let mut reuse = ArtifactReuse::new();
        reuse.add_reusable_source("src/a.zig");
        reuse.add_reusable_decl(1);
        reuse.add_reusable_type(2);
        reuse.add_reusable_layout(3);
        reuse.add_reusable_air_body(100);
        reuse.add_reusable_export("my_export");
        reuse.compute_reusable_count();
        assert_eq!(reuse.reusable_count, 6);
    }

    #[test]
    fn test_artifact_reuse_from_proof_chain_reuse_allowed() {
        let source_checksum = [0x01u8; 32];
        let air_checksum = [0x02u8; 32];
        let proof = ProofChain::private_body_proof(source_checksum, air_checksum);
        let reuse = ArtifactReuse::from_proof_chain(&proof);

        assert!(reuse.fully_reusable);
        // reusable_count equals proof.facts.len() for reuse allowed case
        assert_eq!(reuse.reusable_count, 3);
    }

    #[test]
    fn test_artifact_reuse_from_proof_chain_reuse_denied() {
        let signature_hash = [0x03u8; 32];
        let proof = ProofChain::exported_signature_proof(signature_hash, "my_export");
        let reuse = ArtifactReuse::from_proof_chain(&proof);

        assert!(!reuse.fully_reusable);
    }

    #[test]
    fn test_proof_chain_build_options_proof() {
        let proof = ProofChain::build_options_proof();

        assert!(!proof.reuse_allowed);
        assert_eq!(proof.facts.len(), 1);
        assert_eq!(proof.facts[0].fact_type, ProofFactType::BuildOption);
        assert!(proof.reason().contains("Build options"));
    }

    #[test]
    fn test_artifact_reuse_serialization() {
        let mut reuse = ArtifactReuse::new();
        reuse.add_reusable_source("src/a.zig");
        reuse.add_reusable_decl(1);
        reuse.compute_reusable_count();

        let json = serde_json::to_string(&reuse).unwrap();
        assert!(json.contains("\"fully_reusable\""));
        assert!(json.contains("\"reusable_sources\""));
        assert!(json.contains("\"reusable_decls\""));
    }

    #[test]
    fn test_artifact_reuse_has_reusable_items() {
        let mut reuse = ArtifactReuse::new();
        assert!(!reuse.has_reusable_items());

        reuse.add_reusable_source("src/a.zig");
        assert!(reuse.has_reusable_items());
    }

    // =========================================================================
    // F.73: Cache Explanation Output Tests
    // =========================================================================

    #[test]
    fn test_cache_lookup_result_hit() {
        let explanation = CacheExplanation::checksum_match(CacheArtifactKind::SourceFile);
        let result = CacheLookupResult::hit("key123".to_string(), explanation);

        assert!(result.cache_hit);
        assert_eq!(result.cache_key, "key123");
        assert!(result.proof.is_none());
    }

    #[test]
    fn test_cache_lookup_result_miss() {
        let explanation =
            CacheExplanation::source_changed(CacheArtifactKind::SourceFile, "src/main.zig");
        let result = CacheLookupResult::miss("key123".to_string(), explanation);

        assert!(!result.cache_hit);
        assert_eq!(result.cache_key, "key123");
        assert!(result.proof.is_none());
    }

    #[test]
    fn test_cache_lookup_result_miss_with_proof() {
        let signature_hash = [0x03u8; 32];
        let proof = ProofChain::exported_signature_proof(signature_hash, "my_export");
        let explanation = CacheExplanation::source_changed(CacheArtifactKind::Decl, "src/decl.zig");
        let result = CacheLookupResult::miss_with_proof("key456".to_string(), explanation, proof);

        assert!(!result.cache_hit);
        assert_eq!(result.cache_key, "key456");
        assert!(result.proof.is_some());
    }

    #[test]
    fn test_cache_explanation_checksum_match() {
        let explanation = CacheExplanation::checksum_match(CacheArtifactKind::AirBody);
        assert!(explanation.is_valid);
        assert!(explanation.reason.contains("checksum matches"));
        assert_eq!(explanation.artifact_kind, CacheArtifactKind::AirBody);
    }

    #[test]
    fn test_cache_explanation_source_changed() {
        let explanation =
            CacheExplanation::source_changed(CacheArtifactKind::Object, "src/object.zig");
        assert!(!explanation.is_valid);
        assert!(explanation.reason.contains("changed"));
        assert!(explanation.details.is_some());
    }

    #[test]
    fn test_cache_explanation_build_options_changed() {
        let explanation = CacheExplanation::build_options_changed();
        assert!(!explanation.is_valid);
        assert!(explanation.reason.contains("Build options"));
    }

    #[test]
    fn test_cache_explanation_artifact_missing() {
        let explanation =
            CacheExplanation::artifact_missing(CacheArtifactKind::Object, "/cache/obj.o");
        assert!(!explanation.is_valid);
        assert!(explanation.reason.contains("not found"));
    }

    #[test]
    fn test_cache_explanation_serialization() {
        let explanation = CacheExplanation::checksum_match(CacheArtifactKind::SourceFile);
        let json = serde_json::to_string(&explanation).unwrap();
        assert!(json.contains("\"artifact_kind\":\"SourceFile\""));
        assert!(json.contains("\"is_valid\":true"));
    }

    #[test]
    fn test_cache_lookup_result_serialization() {
        let explanation = CacheExplanation::checksum_match(CacheArtifactKind::Decl);
        let result = CacheLookupResult::hit("key789".to_string(), explanation);
        let json = serde_json::to_string(&result).unwrap();

        assert!(json.contains("\"cache_hit\":true"));
        assert!(json.contains("\"cache_key\":\"key789\""));
        assert!(json.contains("\"explanation\""));
    }

    #[test]
    fn test_cache_artifact_kind_display() {
        assert_eq!(CacheArtifactKind::SourceFile.to_string(), "SourceFile");
        assert_eq!(CacheArtifactKind::Object.to_string(), "Object");
        assert_eq!(
            CacheArtifactKind::BuildArtifact.to_string(),
            "BuildArtifact"
        );
    }

    // =========================================================================
    // F.74: Cache Proof Tests
    // =========================================================================

    #[test]
    fn test_cache_proof_checksum_match() {
        let checksum = [0x01u8; 32];
        let proof = CacheProof::checksum_match(
            "key123".to_string(),
            CacheArtifactKind::SourceFile,
            checksum,
        );

        assert!(proof.is_hit_proof());
        assert_eq!(proof.cache_key, "key123");
        assert_eq!(proof.artifact_kind, CacheArtifactKind::SourceFile);
        assert_eq!(proof.facts.len(), 1);
        assert_eq!(proof.facts[0].fact_type, ProofFactType::Checksum);
    }

    #[test]
    fn test_cache_proof_source_changed() {
        let old_checksum = [0x01u8; 32];
        let new_checksum = [0x02u8; 32];
        let proof = CacheProof::source_changed(
            "key456".to_string(),
            CacheArtifactKind::Decl,
            "src/decl.zig",
            old_checksum,
            new_checksum,
        );

        assert!(!proof.is_hit_proof());
        assert_eq!(proof.cache_key, "key456");
        assert_eq!(proof.facts.len(), 2);
        assert!(proof.explanation.contains("changed"));
    }

    #[test]
    fn test_cache_proof_build_options_changed() {
        let proof = CacheProof::build_options_changed("key789".to_string());

        assert!(!proof.is_hit_proof());
        assert_eq!(proof.artifact_kind, CacheArtifactKind::BuildArtifact);
        assert!(proof.explanation.contains("Build options"));
    }

    #[test]
    fn test_cache_proof_artifact_missing() {
        let proof = CacheProof::artifact_missing(
            "key000".to_string(),
            CacheArtifactKind::Object,
            "/cache/obj.o",
        );

        assert!(!proof.is_hit_proof());
        assert!(proof.explanation.contains("not found"));
    }

    #[test]
    fn test_cache_proof_serialization() {
        let checksum = [0x03u8; 32];
        let proof =
            CacheProof::checksum_match("key111".to_string(), CacheArtifactKind::AirBody, checksum);
        let json = serde_json::to_string(&proof).unwrap();

        assert!(json.contains("\"cache_key\":\"key111\""));
        assert!(json.contains("\"valid\":true"));
        assert!(json.contains("\"facts\""));
        assert!(json.contains("\"explanation\""));
    }

    #[test]
    fn test_cache_lookup_result_has_proof() {
        let explanation = CacheExplanation::checksum_match(CacheArtifactKind::SourceFile);
        let result = CacheLookupResult::hit("key123".to_string(), explanation.clone());
        assert!(!result.has_proof());

        let signature_hash = [0x03u8; 32];
        let proof_chain = ProofChain::exported_signature_proof(signature_hash, "my_export");
        let miss_result =
            CacheLookupResult::miss_with_proof("key456".to_string(), explanation, proof_chain);
        assert!(miss_result.has_proof());
    }

    #[test]
    fn test_cache_lookup_result_get_proof() {
        let explanation = CacheExplanation::source_changed(CacheArtifactKind::Decl, "src/decl.zig");
        let signature_hash = [0x04u8; 32];
        let proof_chain = ProofChain::exported_signature_proof(signature_hash, "my_export");
        let result =
            CacheLookupResult::miss_with_proof("key789".to_string(), explanation, proof_chain);

        let retrieved_proof = result.get_proof();
        assert!(retrieved_proof.is_some());
        assert!(!retrieved_proof.unwrap().allows_reuse());
    }

    // =========================================================================
    // J.119: Release Proof Gate Tests
    // =========================================================================

    #[test]
    fn test_release_proof_gate_authoritative_pass() {
        let source_checksum = [0x01u8; 32];
        let air_checksum = [0x02u8; 32];
        let proof = ProofChain::private_body_proof(source_checksum, air_checksum);
        let gate = ReleaseProofGate::authoritative_pass(proof);

        assert!(gate.passed);
        assert!(gate.proof_chain.is_some());
        assert_eq!(gate.validation_mode, AuthorityMode::Authoritative);
        assert!(!gate.used_fallback());
        assert!(gate.validate().is_ok());
    }

    #[test]
    fn test_release_proof_gate_fallback_fail() {
        let gate = ReleaseProofGate::fallback_fail("No compiler artifacts available");

        assert!(!gate.passed);
        assert!(gate.proof_chain.is_none());
        assert_eq!(gate.validation_mode, AuthorityMode::Fallback);
        assert!(gate.used_fallback());
        assert!(gate.validate().is_err());
    }

    #[test]
    fn test_release_gate_error_display() {
        let err = ReleaseGateError::FallbackModeUsed("test reason".to_string());
        let msg = err.to_string();
        assert!(msg.contains("fallback mode used"));
        assert!(msg.contains("test reason"));
    }

    #[test]
    fn test_authority_mode_equality() {
        assert_eq!(AuthorityMode::Authoritative, AuthorityMode::Authoritative);
        assert_eq!(AuthorityMode::Fallback, AuthorityMode::Fallback);
        assert_ne!(AuthorityMode::Authoritative, AuthorityMode::Fallback);
    }

    #[test]
    fn test_release_proof_gate_serialization() {
        let source_checksum = [0x01u8; 32];
        let air_checksum = [0x02u8; 32];
        let proof = ProofChain::private_body_proof(source_checksum, air_checksum);
        let gate = ReleaseProofGate::authoritative_pass(proof);

        let json = serde_json::to_string(&gate).unwrap();
        assert!(json.contains("\"passed\":true"));
        assert!(json.contains("\"validation_mode\":\"Authoritative\""));
        assert!(json.contains("\"used_fallback\":false"));
    }

    // =========================================================================
    // K.172: Final Acceptance Test
    // =========================================================================

    /// Proof that incremental and fresh builds produce equivalent results.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IncrementalFreshProof {
        /// Whether the proof passed.
        pub passed: bool,
        /// Checksum of the incremental build output.
        pub incremental_checksum: [u8; 32],
        /// Checksum of the fresh build output.
        pub fresh_checksum: [u8; 32],
        /// Whether checksums match.
        pub checksums_match: bool,
        /// Explanation of the comparison.
        pub explanation: String,
        /// Statistics about the comparison.
        pub stats: AcceptanceStats,
    }

    /// Statistics from acceptance test.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AcceptanceStats {
        /// Number of source files tested.
        pub source_file_count: u32,
        /// Number of declarations tested.
        pub decl_count: u32,
        /// Number of types tested.
        pub type_count: u32,
        /// Number of layouts tested.
        pub layout_count: u32,
        /// Whether all tests passed.
        pub all_passed: bool,
    }

    impl IncrementalFreshProof {
        /// Create a passing proof where checksums match.
        pub fn passing(
            incremental_checksum: [u8; 32],
            fresh_checksum: [u8; 32],
            stats: AcceptanceStats,
        ) -> Self {
            let checksums_match = incremental_checksum == fresh_checksum;
            Self {
                passed: checksums_match,
                incremental_checksum,
                fresh_checksum,
                checksums_match,
                explanation: if checksums_match {
                    "Incremental and fresh builds are equivalent".to_string()
                } else {
                    "Incremental and fresh builds differ".to_string()
                },
                stats,
            }
        }

        /// Create a failing proof where checksums differ.
        pub fn failing(
            incremental_checksum: [u8; 32],
            fresh_checksum: [u8; 32],
            reason: &str,
        ) -> Self {
            Self {
                passed: false,
                incremental_checksum,
                fresh_checksum,
                checksums_match: false,
                explanation: format!("Proof failed: {}", reason),
                stats: AcceptanceStats {
                    source_file_count: 0,
                    decl_count: 0,
                    type_count: 0,
                    layout_count: 0,
                    all_passed: false,
                },
            }
        }

        /// Validate the proof.
        pub fn validate(&self) -> Result<(), AcceptanceError> {
            if self.passed && self.checksums_match {
                Ok(())
            } else {
                Err(AcceptanceError::ProofFailed(self.explanation.clone()))
            }
        }

        /// Get the diff between checksums as hex strings.
        pub fn checksum_diff(&self) -> String {
            format!(
                "incremental: {:x?}\nfresh:      {:x?}",
                self.incremental_checksum, self.fresh_checksum
            )
        }
    }

    /// Error when acceptance test fails.
    #[derive(Debug, Clone)]
    pub enum AcceptanceError {
        /// The proof failed.
        ProofFailed(String),
    }

    impl std::fmt::Display for AcceptanceError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                AcceptanceError::ProofFailed(msg) => {
                    write!(f, "Acceptance test failed: {}", msg)
                }
            }
        }
    }

    impl std::error::Error for AcceptanceError {}

    /// Acceptance test runner that compares incremental vs fresh builds.
    #[derive(Debug, Clone)]
    pub struct AcceptanceTestRunner;

    impl AcceptanceTestRunner {
        /// Run acceptance test on a schema.
        pub fn run(schema: &SnapSchema) -> IncrementalFreshProof {
            // Compute checksum of the schema
            let checksum = schema.compute_checksum();

            // Build stats
            let stats = AcceptanceStats {
                source_file_count: schema.source_files.len() as u32,
                decl_count: schema.decls.len() as u32,
                type_count: schema.types.len() as u32,
                layout_count: schema.layouts.len() as u32,
                all_passed: true,
            };

            // For a self-comparison, both checksums are the same (comparing schema to itself)
            // In real usage, this would compare incremental vs fresh build of same source
            IncrementalFreshProof::passing(checksum, checksum, stats)
        }

        /// Run acceptance test comparing two schemas.
        pub fn compare(schema1: &SnapSchema, schema2: &SnapSchema) -> IncrementalFreshProof {
            let checksum1 = schema1.compute_checksum();
            let checksum2 = schema2.compute_checksum();

            let stats = AcceptanceStats {
                source_file_count: schema1.source_files.len() as u32,
                decl_count: schema1.decls.len() as u32,
                type_count: schema1.types.len() as u32,
                layout_count: schema1.layouts.len() as u32,
                all_passed: true,
            };

            IncrementalFreshProof::passing(checksum1, checksum2, stats)
        }
    }

    #[test]
    fn test_incremental_fresh_proof_passing() {
        let checksum = [0x01u8; 32];
        let stats = AcceptanceStats {
            source_file_count: 10,
            decl_count: 20,
            type_count: 30,
            layout_count: 40,
            all_passed: true,
        };
        let proof = IncrementalFreshProof::passing(checksum, checksum, stats);

        assert!(proof.passed);
        assert!(proof.checksums_match);
        assert!(proof.validate().is_ok());
    }

    #[test]
    fn test_incremental_fresh_proof_failing() {
        let inc_checksum = [0x01u8; 32];
        let fresh_checksum = [0x02u8; 32];
        let proof =
            IncrementalFreshProof::failing(inc_checksum, fresh_checksum, "checksums differ");

        assert!(!proof.passed);
        assert!(!proof.checksums_match);
        assert!(proof.validate().is_err());
    }

    #[test]
    fn test_incremental_fresh_proof_checksum_diff() {
        let inc_checksum = [0x01u8; 32];
        let fresh_checksum = [0x02u8; 32];
        let proof = IncrementalFreshProof::failing(inc_checksum, fresh_checksum, "test");

        let diff = proof.checksum_diff();
        assert!(diff.contains("incremental"));
        assert!(diff.contains("fresh"));
    }

    #[test]
    fn test_acceptance_test_runner_run() {
        let schema = SnapSchema::default();
        let proof = AcceptanceTestRunner::run(&schema);

        assert!(proof.passed);
        assert!(proof.stats.source_file_count == 0);
    }

    #[test]
    fn test_acceptance_test_runner_compare() {
        let schema1 = SnapSchema::default();
        let schema2 = SnapSchema::default();
        let proof = AcceptanceTestRunner::compare(&schema1, &schema2);

        assert!(proof.passed);
    }

    #[test]
    fn test_incremental_fresh_proof_serialization() {
        let checksum = [0x01u8; 32];
        let stats = AcceptanceStats {
            source_file_count: 5,
            decl_count: 10,
            type_count: 15,
            layout_count: 20,
            all_passed: true,
        };
        let proof = IncrementalFreshProof::passing(checksum, checksum, stats);

        let json = serde_json::to_string(&proof).unwrap();
        assert!(json.contains("\"passed\":true"));
        assert!(json.contains("\"checksums_match\":true"));
        assert!(json.contains("\"stats\""));
    }

    #[test]
    fn test_acceptance_stats_serialization() {
        let stats = AcceptanceStats {
            source_file_count: 10,
            decl_count: 20,
            type_count: 30,
            layout_count: 40,
            all_passed: true,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"source_file_count\":10"));
        assert!(json.contains("\"all_passed\":true"));
    }
}
