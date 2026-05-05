//! Chimera C schema definitions for artifact formats.
//!
//! This crate defines the schema for C-specific artifacts:
//! - `.csnap`: Semantic snapshot containing headers, source files, declarations
//! - `.cdep`: Dependency graph for incremental cache invalidation
//! - `.castpack`: C AST/type/layout package with declarations, types, layouts
//! - `.cchmeta`: C-specific metadata before common `.chmeta`
//! - `.cchproof`: C proof facts for Lean bridge
//!
//! Task 9: C-specific artifact schema crate

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Canonical serializer for deterministic artifact encoding.
///
/// This module provides canonical serialization to ensure byte-for-byte
/// repeatability across builds, which is essential for cache invalidation
/// and content-addressable storage.
pub mod canonical {
    use serde::Serialize;

    /// Options for canonical serialization
    #[derive(Debug, Clone, Default)]
    pub struct SerializeOptions {
        /// Whether to include whitespace (default: false for compact)
        pub include_whitespace: bool,
    }

    impl SerializeOptions {
        pub fn new() -> Self {
            Self::default()
        }
    }

    /// Serialize a value to a canonical JSON string.
    ///
    /// Uses `serde_json::to_string` which produces deterministic output
    /// for the same input data. The output is NOT sorted - for that
    /// use `compute_deterministic_hash` which sorts bytes before hashing.
    pub fn to_canonical_string<T: Serialize + ?Sized>(
        value: &T,
    ) -> Result<String, serde_json::Error> {
        serde_json::to_string(value)
    }

    /// Serialize a value to compact JSON bytes.
    pub fn to_canonical_bytes<T: Serialize + ?Sized>(
        value: &T,
    ) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(value)
    }
}

/// Compute a deterministic content hash for an artifact.
///
/// Uses BLAKE3 for fast, secure hashing. The serialization uses
/// standard JSON which is deterministic for the same content,
/// and additional sorting is applied before hashing to ensure
/// order-independent repeatability.
pub fn compute_deterministic_hash<T: Serialize + ?Sized>(
    value: &T,
) -> Result<String, DeterministicHashError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|e| DeterministicHashError::SerializationError(e.to_string()))?;

    // Sort bytes deterministically before hashing for true repeatability
    let mut sorted_bytes = bytes;
    sorted_bytes.sort();

    Ok(blake3::hash(&sorted_bytes).to_hex().to_string())
}

/// Error type for deterministic hashing
#[derive(Debug, thiserror::Error)]
pub enum DeterministicHashError {
    #[error("serialization error: {0}")]
    SerializationError(String),
}

/// Artifact migration policy for schema version transitions.
///
/// Handles reject/migrate/accept rules for major/minor schema versions.
/// Each artifact type has its own migration rules.
pub mod migration {
    use super::SchemaError;

    /// Migration result type
    pub type Result<T> = std::result::Result<T, SchemaError>;

    /// Migration action to take for an artifact
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum MigrationAction {
        /// Accept the artifact as-is (compatible version)
        Accept,
        /// Migrate the artifact to current version
        Migrate,
        /// Reject the artifact (incompatible version)
        Reject { reason: String },
    }

    /// Migration policy for schema versions
    #[derive(Debug, Clone)]
    pub struct MigrationPolicy {
        /// Current schema version
        pub current_version: u32,
        /// Minimum supported version (older versions are rejected)
        pub min_supported_version: u32,
    }

    impl MigrationPolicy {
        /// Create a new migration policy
        pub fn new(current_version: u32) -> Self {
            let min_supported_version = if current_version <= 100 {
                // For v1.x (version 100), accept anything (v0.x and v1.x)
                0
            } else {
                // For v2.x+ (version 200+), require previous major version minimum
                // e.g., v2.0 (200) requires v1.x (100) minimum
                ((current_version / 100) - 1) * 100
            };
            Self {
                current_version,
                min_supported_version,
            }
        }

        /// Create with explicit minimum supported version
        pub fn with_min_version(current_version: u32, min_version: u32) -> Self {
            Self {
                current_version,
                min_supported_version: min_version,
            }
        }

        /// Determine migration action for a given schema version
        pub fn get_action(&self, artifact_version: u32) -> MigrationAction {
            if artifact_version < self.min_supported_version {
                MigrationAction::Reject {
                    reason: format!(
                        "Schema version {} is too old, minimum supported is {}",
                        artifact_version, self.min_supported_version
                    ),
                }
            } else if artifact_version == self.current_version {
                MigrationAction::Accept
            } else if artifact_version > self.current_version {
                // Future version - we can't know how to migrate
                MigrationAction::Reject {
                    reason: format!(
                        "Schema version {} is from a newer version, current is {}",
                        artifact_version, self.current_version
                    ),
                }
            } else {
                // artifact_version < current_version but >= min_supported
                MigrationAction::Migrate
            }
        }

        /// Check if migration is needed
        pub fn needs_migration(&self, artifact_version: u32) -> bool {
            matches!(self.get_action(artifact_version), MigrationAction::Migrate)
        }

        /// Check if version is compatible (accept or migrate)
        pub fn is_compatible(&self, artifact_version: u32) -> bool {
            !matches!(
                self.get_action(artifact_version),
                MigrationAction::Reject { .. }
            )
        }
    }

    /// Default migration policy for C artifacts
    pub fn default_policy() -> MigrationPolicy {
        MigrationPolicy::new(super::CURRENT_SCHEMA_VERSION)
    }
}

/// Magic bytes for C artifacts: b"CHCS" (Chimera C Schema)
pub const C_ARTIFACT_MAGIC: [u8; 4] = [0x43, 0x48, 0x43, 0x53];

/// Schema version for all C artifacts
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Artifact magic bytes and version header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactHeader {
    pub magic: [u8; 4],
    pub schema_version: u32,
    pub producer_version: String,
    pub target: String,
    pub source_language: String,
}

impl ArtifactHeader {
    pub fn new(target: &str, producer_version: &str) -> Self {
        Self {
            magic: C_ARTIFACT_MAGIC,
            schema_version: CURRENT_SCHEMA_VERSION,
            producer_version: producer_version.to_string(),
            target: target.to_string(),
            source_language: "c".to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), SchemaError> {
        if self.magic != C_ARTIFACT_MAGIC {
            return Err(SchemaError::InvalidMagic(self.magic));
        }
        if self.schema_version > CURRENT_SCHEMA_VERSION {
            return Err(SchemaError::UnsupportedVersion(self.schema_version));
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("invalid magic bytes: {0:?}")]
    InvalidMagic([u8; 4]),
    #[error("unsupported schema version: {0}")]
    UnsupportedVersion(u32),
    #[error("incompatible version {0}, expected {1}")]
    IncompatibleVersion(u32, u32),
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("corrupted data: {0}")]
    Corrupted(String),
}

// =============================================================================
// C Diagnostic Codes
// =============================================================================

/// C-specific diagnostic codes for parse, clang extraction, include, macro, ABI,
/// layout, pointer, varargs, errno, callback, unsafe/trust, link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum CDiagnosticCode {
    // Parse errors (E1xxx)
    ParseUnexpectedToken = 1000,
    ParseExpectedToken = 1001,
    ParseInvalidDeclaration = 1002,
    ParseInvalidType = 1003,
    ParseIncompleteType = 1004,
    ParseDuplicateDeclaration = 1005,

    // Clang extraction errors (E2xxx)
    ClangExtractFailed = 2000,
    ClangAstError = 2001,
    ClangParseError = 2002,
    ClangIncludeNotFound = 2003,
    ClangMacroExpansionError = 2004,

    // Include errors (E3xxx)
    IncludeNotFound = 3000,
    IncludeRecursive = 3001,
    IncludeMalformed = 3002,

    // Macro errors (E4xxx)
    MacroUndefined = 4000,
    MacroInvalid = 4001,
    MacroExpansionFailed = 4002,
    MacroConditionalBranch = 4003,

    // ABI errors (E5xxx)
    AbiMismatch = 5000,
    AbiCallingConventionMismatch = 5001,
    AbiVarargsUnsupported = 5002,
    AbiFunctionPointerMismatch = 5003,

    // Layout errors (E6xxx)
    LayoutSizeMismatch = 6000,
    LayoutAlignMismatch = 6001,
    LayoutFieldOffsetMismatch = 6002,
    LayoutBitfieldOverflow = 6003,
    LayoutFlexibleArrayInvalid = 6004,
    LayoutPackedAttributeInvalid = 6005,

    // Pointer errors (E7xxx)
    PointerNullabilityMismatch = 7000,
    PointerAliasingViolation = 7001,
    PointerRestrictViolation = 7002,
    PointerNullableViolation = 7003,

    // Varargs errors (E8xxx)
    VarargsDirectUnsafe = 8000,
    VarargsMissingWrapper = 8001,

    // Errno errors (E9xxx)
    ErrnoMappingMissing = 9000,
    ErrnoInvalidCode = 9001,
    ErrnoConventionMismatch = 9002,

    // Callback errors (F1xxx)
    CallbackCallingConventionMismatch = 11000,
    CallbackLifetimeViolation = 11001,
    CallbackNullabilityViolation = 11002,
    CallbackUserDataViolation = 11003,
    CallbackPanicPolicyViolation = 11004,

    // Unsafe/Trust errors (F2xxx)
    UnsafeOperationUndocumented = 12000,
    TrustAssumptionViolation = 12001,
    UnsafePtrDereference = 12002,
    UnsafeUnionAccess = 12003,

    // Link errors (F3xxx)
    LinkSymbolNotFound = 13000,
    LinkDuplicateSymbol = 13001,
    LinkIncompatibleArchive = 13002,
}

impl CDiagnosticCode {
    pub fn code(&self) -> u32 {
        *self as u32
    }

    pub fn category(&self) -> &'static str {
        let code = *self as u32;
        if code >= 1000 && code <= 1005 {
            "parse"
        } else if code >= 2000 && code <= 2004 {
            "clang"
        } else if code >= 3000 && code <= 3002 {
            "include"
        } else if code >= 4000 && code <= 4003 {
            "macro"
        } else if code >= 5000 && code <= 5003 {
            "abi"
        } else if code >= 6000 && code <= 6005 {
            "layout"
        } else if code >= 7000 && code <= 7003 {
            "pointer"
        } else if code >= 8000 && code <= 8001 {
            "varargs"
        } else if code >= 9000 && code <= 9002 {
            "errno"
        } else if code >= 11000 && code <= 11004 {
            "callback"
        } else if code >= 12000 && code <= 12003 {
            "unsafe"
        } else if code >= 13000 && code <= 13002 {
            "link"
        } else {
            "unknown"
        }
    }
}

// =============================================================================
// C Target Information
// =============================================================================

/// Target information for C compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CTarget {
    pub triple: String,
    pub arch: String,
    pub os: String,
    pub env: String,
    pub libc: Option<String>,
    pub clang_version: Option<String>,
    pub resource_dir: Option<String>,
    pub sysroot: Option<String>,
    pub pointer_width: u32,
    pub size_of_ptr: u32,
    pub size_of_int: u32,
    pub size_of_long: u32,
    pub size_of_long_long: u32,
    pub size_of_float: u32,
    pub size_of_double: u32,
    pub size_of_long_double: u32,
    pub size_of_void: u32,
    pub int64_aligned: u32,
    pub long_long_aligned: u32,
    pub double_aligned: u32,
    pub long_double_aligned: u32,
    pub long_double_size: u32,
    pub big_endian: bool,
    pub c_standard: CStandard,
    /// Clang-provided facts that are trusted until independently verified
    pub clang_trust_facts: Vec<String>,
}

/// C standard version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CStandard {
    C89,
    C90,
    C99,
    C11,
    C17,
    C23,
    Gnuc,
}

impl Default for CStandard {
    fn default() -> Self {
        CStandard::C11
    }
}

// =============================================================================
// C Manifest Schema
// =============================================================================

/// C module manifest describing the C project being processed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CManifest {
    pub language: String,
    pub headers: Vec<String>,
    pub sources: Vec<String>,
    pub compile_database: Option<String>,
    pub include_dirs: Vec<IncludeDir>,
    pub macro_defs: Vec<MacroDef>,
    pub standard: CStandard,
    pub sysroot: Option<String>,
    pub linker_libs: Vec<String>,
    pub target: Option<String>,
    pub adapter_mode: CAdapterMode,
}

/// Include directory with search path and priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeDir {
    pub path: String,
    pub is_system: bool,
    pub is_framework: bool,
}

/// Macro definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDef {
    pub name: String,
    pub value: Option<String>,
    pub is_function_like: bool,
    pub params: Option<Vec<String>>,
}

/// C adapter modes for processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CAdapterMode {
    /// Clang is authoritative for all facts including layout
    ClangAuthoritative,
    /// Only surface-level validation, no layout authority
    SurfaceOnly,
    /// Header-only processing, no source bodies
    HeaderOnly,
    /// Source lowering with function bodies
    SourceLowering,
    /// Metadata only, no actual processing
    MetadataOnly,
}

impl Default for CAdapterMode {
    fn default() -> Self {
        CAdapterMode::ClangAuthoritative
    }
}

// =============================================================================
// .csnap - Semantic Snapshot
// =============================================================================

/// C semantic snapshot containing header graph, source files, declarations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsnapSnapshot {
    pub header: ArtifactHeader,
    pub checksum: String,
    pub clang_version: String,
    pub target: CTarget,
    pub headers: Vec<HeaderInfo>,
    pub source_files: Vec<SourceFileInfo>,
    pub declarations: Vec<Declaration>,
    pub exports: Vec<ExportSymbol>,
    pub imports: Vec<ImportSymbol>,
    pub compile_flags: Vec<String>,
    pub active_macros: Vec<String>,
    pub conditional_branches: Vec<ConditionalBranch>,
}

impl CsnapSnapshot {
    pub fn compute_checksum(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        let bytes = serde_json::to_vec(self).unwrap();
        hasher.update(&bytes);
        hasher.finalize().to_hex().to_string()
    }
}

/// Header file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderInfo {
    pub path: String,
    pub content_hash: String,
    pub size: u64,
    pub mtime: u64,
    pub include_guard: Option<String>,
    pub includes: Vec<String>,
    pub macro_defs: Vec<String>,
    pub is_system: bool,
    pub is_generated: bool,
}

/// Source file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFileInfo {
    pub path: String,
    pub content_hash: String,
    pub size: u64,
    pub mtime: u64,
    pub translation_unit: TranslationUnit,
}

/// Translation unit for a C source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationUnit {
    pub id: TUId,
    pub source_file: String,
    pub header_dependencies: Vec<String>,
    pub macro_dependencies: Vec<String>,
    pub declarations: Vec<Declaration>,
}

/// Stable TU identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TUId(pub u64);

/// Conditional compilation branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalBranch {
    pub macro_name: String,
    pub condition: String,
    pub is_active: bool,
}

// =============================================================================
// C Declaration Types
// =============================================================================

/// C declaration kinds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeclarationKind {
    Function(FunctionDecl),
    GlobalVariable(GlobalVarDecl),
    StructDecl(StructDecl),
    UnionDecl(UnionDecl),
    EnumDecl(EnumDecl),
    TypedefDecl(TypedefDecl),
    EnumConstant(EnumConstant),
    MacroDecl(MacroDecl),
}

/// Function declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDecl {
    pub id: DeclId,
    pub name: String,
    pub linkage: Linkage,
    pub storage_class: StorageClass,
    pub calling_convention: String,
    pub params: Vec<ParamDecl>,
    pub return_type: TypeRef,
    pub attributes: Vec<CAttribute>,
    pub source_span: SourceSpan,
    pub is_definition: bool,
    pub is_inline: bool,
    pub has_body: bool,
}

/// Parameter declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDecl {
    pub name: String,
    pub typ: TypeRef,
    pub has_default: bool,
}

/// Global variable declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalVarDecl {
    pub id: DeclId,
    pub name: String,
    pub linkage: Linkage,
    pub storage_class: StorageClass,
    pub typ: TypeRef,
    pub initializer: Option<Initializer>,
    pub source_span: SourceSpan,
}

/// Struct declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDecl {
    pub id: DeclId,
    pub name: Option<String>,
    pub fields: Vec<FieldDecl>,
    pub is_packed: bool,
    pub pack_align: Option<u32>,
    pub is_incomplete: bool,
    pub source_span: SourceSpan,
}

/// Field declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDecl {
    pub name: String,
    pub typ: TypeRef,
    pub bitfield_width: Option<u32>,
    pub offset: u64,
    pub size: u64,
    pub align: u32,
}

/// Union declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionDecl {
    pub id: DeclId,
    pub name: Option<String>,
    pub variants: Vec<FieldDecl>,
    pub size: u64,
    pub align: u32,
    pub is_incomplete: bool,
    pub source_span: SourceSpan,
}

/// Enum declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDecl {
    pub id: DeclId,
    pub name: Option<String>,
    pub underlying_type: Option<TypeRef>,
    pub constants: Vec<EnumConstant>,
    pub is_incomplete: bool,
    pub source_span: SourceSpan,
}

/// Enum constant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumConstant {
    pub id: DeclId,
    pub name: String,
    pub value: Option<i64>,
    pub source_span: SourceSpan,
}

/// Typedef declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedefDecl {
    pub id: DeclId,
    pub name: String,
    pub underlying_type: TypeRef,
    pub source_span: SourceSpan,
}

/// Macro declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDecl {
    pub id: DeclId,
    pub name: String,
    pub value: Option<String>,
    pub is_function_like: bool,
    pub params: Option<Vec<String>>,
    pub source_span: SourceSpan,
}

/// Declaration identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeclId(pub u64);

impl DeclId {
    /// Compute a stable ID for a declaration based on its characteristics
    /// This ID is deterministic and does not depend on transient data like memory addresses
    pub fn compute_stable_id(
        name: &str,
        declaration_spelling: &str,
        type_shape: &str,
        target: &str,
        schema_version: u32,
    ) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(name.as_bytes());
        hasher.update(declaration_spelling.as_bytes());
        hasher.update(type_shape.as_bytes());
        hasher.update(target.as_bytes());
        hasher.update(&schema_version.to_le_bytes());
        hasher.finalize().to_hex().to_string()
    }
}

/// Source span for location tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpan {
    pub file: String,
    pub line: u32,
    pub col: u32,
    pub byte_offset: u64,
    pub byte_length: u64,
}

/// Linkage classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Linkage {
    None,
    Internal,
    External,
    Weak,
}

/// Storage class specifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageClass {
    None,
    Auto,
    Static,
    Extern,
    Register,
    ThreadLocal,
}

/// C attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CAttribute {
    pub name: String,
    pub args: Option<Vec<String>>,
}

/// Initializer for global variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Initializer {
    Zero,
    Constant(String),
    Expr(String),
}

/// Export symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSymbol {
    pub symbol: String,
    pub decl_id: DeclId,
    pub abi: String,
    pub linkage: Linkage,
}

/// Import symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSymbol {
    pub symbol: String,
    pub signature: String,
    pub abi: String,
    pub source_lang: SourceLanguage,
}

// =============================================================================
// .cdep - Dependency Graph
// =============================================================================

/// C dependency graph for incremental cache invalidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdepGraph {
    pub header: ArtifactHeader,
    pub checksum: String,
    pub nodes: Vec<CDepNode>,
    pub edges: Vec<CDepEdge>,
}

impl CdepGraph {
    pub fn compute_checksum(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        let bytes = serde_json::to_vec(self).unwrap();
        hasher.update(&bytes);
        hasher.finalize().to_hex().to_string()
    }
}

/// Dependency node kinds for C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CDepNodeKind {
    TranslationUnit,
    Source,
    Header,
    Macro,
    Declaration,
    Type,
    Layout,
    FunctionBody,
    Export,
    Import,
    Object,
    Wrapper,
    Proof,
    Link,
}

/// Dependency node in the C graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDepNode {
    pub id: CDepNodeId,
    pub kind: CDepNodeKind,
    pub fingerprint: String,
    pub stable_id: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CDepNodeId(pub u64);

/// Dependency edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDepEdge {
    pub from: CDepNodeId,
    pub to: CDepNodeId,
    pub kind: CDepEdgeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CDepEdgeKind {
    DependsOn,
    Includes,
    Defines,
    Exports,
    Imports,
    Specializes,
}

/// Graph mutation types for invalidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphMutation {
    Added(CDepNodeId),
    Removed(CDepNodeId),
    Changed { id: CDepNodeId, kind: ChangeKind },
    Unchanged(CDepNodeId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeKind {
    AbiChanged,
    LayoutChanged,
    MacroChanged,
    IncludeChanged,
    ExportChanged,
}

// =============================================================================
// .castpack - C AST/Type/Layout Package
// =============================================================================

/// C AST/type/layout package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastPack {
    pub header: ArtifactHeader,
    pub checksum: String,
    pub declarations: Vec<Declaration>,
    pub types: Vec<TypeDef>,
    pub layouts: Vec<LayoutDef>,
    pub symbol_table: SymbolTable,
    pub macro_provenance: MacroProvenance,
    pub diagnostics: Vec<Diagnostic>,
}

impl CastPack {
    pub fn compute_checksum(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        let bytes = serde_json::to_vec(self).unwrap();
        hasher.update(&bytes);
        hasher.finalize().to_hex().to_string()
    }
}

/// Symbol table mapping names to declarations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTable {
    pub functions: HashMap<String, DeclId>,
    pub globals: HashMap<String, DeclId>,
    pub structs: HashMap<String, DeclId>,
    pub unions: HashMap<String, DeclId>,
    pub enums: HashMap<String, DeclId>,
    pub typedefs: HashMap<String, DeclId>,
    pub macros: HashMap<String, DeclId>,
}

/// Macro provenance tracking macro expansions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroProvenance {
    pub expansions: Vec<MacroExpansion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroExpansion {
    pub macro_id: DeclId,
    pub expanded_at: SourceSpan,
    pub arguments: Option<Vec<String>>,
    pub expansion_text: String,
}

/// C type reference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeRef(pub u32);

/// C type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeDef {
    Primitive(CPrimitiveType),
    Pointer(TypeRef, PointerKind),
    Array(TypeRef, u64),
    FunctionPointer {
        params: Vec<TypeRef>,
        ret: Option<TypeRef>,
        cconv: String,
    },
    Struct(TypeRef),
    Union(TypeRef),
    Enum(TypeRef),
    Typedef(TypeRef),
    Volatile(TypeRef),
    Atomic(TypeRef),
    Incomplete,
}

/// Primitive C types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CPrimitiveType {
    Void,
    Bool,
    Char,
    SChar,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    LongLong,
    ULongLong,
    Float,
    Double,
    LongDouble,
    FloatComplex,
    DoubleComplex,
    LongDoubleComplex,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
}

/// Pointer kind with nullability and constness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerKind {
    Raw,
    Nullable,
    NonNull,
    Borrow,
    BorrowMut,
}

/// C layout definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutDef {
    pub ty: TypeRef,
    pub size: u64,
    pub align: u32,
    pub fields: Vec<FieldOffset>,
    pub bitfields: Vec<BitfieldDef>,
    pub is_packed: bool,
    pub pack_align: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOffset {
    pub name: String,
    pub offset: u64,
    pub typ: TypeRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitfieldDef {
    pub name: String,
    pub container_offset: u64,
    pub bit_offset: u8,
    pub bit_width: u8,
    pub typ: TypeRef,
    pub is_signed: bool,
}

/// Diagnostic from C processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: CDiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub location: SourceSpan,
    pub notes: Vec<DiagnosticNote>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticNote {
    pub message: String,
    pub location: Option<SourceSpan>,
}

// =============================================================================
// .cchmeta - C-specific Metadata
// =============================================================================

/// C-specific metadata before conversion to common `.chmeta`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CchMeta {
    pub header: ArtifactHeader,
    pub checksum: String,
    pub declaration_provenance: Vec<DeclProvenance>,
    pub c_abi_facts: Vec<CAbiFact>,
    pub layout_facts: Vec<CLayoutFact>,
    pub macro_dependencies: Vec<MacroDep>,
    pub include_dependencies: Vec<IncludeDep>,
    pub trust_assumptions: Vec<CTrustAssumption>,
}

impl CchMeta {
    pub fn compute_checksum(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        let bytes = serde_json::to_vec(self).unwrap();
        hasher.update(&bytes);
        hasher.finalize().to_hex().to_string()
    }
}

/// Declaration provenance tracking source location and extraction method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclProvenance {
    pub decl_id: DeclId,
    pub extracted_by: ExtractionMethod,
    pub source_location: SourceSpan,
    pub macro_expansion_stack: Vec<SourceSpan>,
    pub is_trusted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtractionMethod {
    ClangAst,
    ClangAstDump,
    TreeSitterFallback,
}

/// C ABI fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CAbiFact {
    pub symbol: String,
    pub cconv: String,
    pub params: Vec<AbiParamInfo>,
    pub ret: Option<AbiRetInfo>,
    pub varargs: bool,
    pub proof_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiParamInfo {
    pub position: u32,
    pub passing: PassingConvention,
    pub by_val: bool,
    pub align: u32,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiRetInfo {
    pub passing: PassingConvention,
    pub align: u32,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PassingConvention {
    Direct,
    ByReference,
    Split,
    Ignore,
}

/// C layout fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLayoutFact {
    pub type_name: String,
    pub size: u64,
    pub align: u32,
    pub fields: Vec<LayoutFieldFact>,
    pub bitfields: Vec<BitfieldFact>,
    pub is_packed: bool,
    pub proof_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutFieldFact {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub align: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitfieldFact {
    pub name: String,
    pub container_offset: u64,
    pub bit_offset: u8,
    pub bit_width: u8,
    pub is_signed: bool,
}

/// Macro dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDep {
    pub macro_name: String,
    pub value_hash: String,
    pub controlling_conditions: Vec<String>,
    pub affects_abi: bool,
}

/// Include dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeDep {
    pub header_path: String,
    pub content_hash: String,
    pub is_system: bool,
    pub is_generated: bool,
}

/// C trust assumption for Clang-provided facts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CTrustAssumption {
    pub kind: CTrustKind,
    pub description: String,
    pub external_ref: Option<String>,
    pub verified_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CTrustKind {
    ClangAstInterpretation,
    LayoutComputedByClang,
    MacroExpansionCorrect,
    TargetTripleCorrect,
    StandardLibraryConformance,
}

// =============================================================================
// .cchproof - C Proof Facts
// =============================================================================

/// C-specific proof facts for Lean bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CchProof {
    pub header: ArtifactHeader,
    pub checksum: String,
    pub layout_proofs: Vec<CLayoutProof>,
    pub signature_proofs: Vec<CSignatureProof>,
    pub pointer_proofs: Vec<CPointerProof>,
    pub alias_proofs: Vec<AliasProof>,
    pub errno_proofs: Vec<ErrnoProof>,
    pub varargs_proofs: Vec<VarargsProof>,
    pub callback_proofs: Vec<CallbackProof>,
    pub wrapper_proofs: Vec<CWrapperProof>,
    pub cache_proofs: Vec<CCacheProof>,
}

impl CchProof {
    pub fn compute_checksum(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        let bytes = serde_json::to_vec(self).unwrap();
        hasher.update(&bytes);
        hasher.finalize().to_hex().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLayoutProof {
    pub type_name: String,
    pub proof_hash: String,
    pub obligations: Vec<LayoutObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutObligation {
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSignatureProof {
    pub symbol: String,
    pub proof_hash: String,
    pub obligations: Vec<SignatureObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureObligation {
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPointerProof {
    pub symbol: String,
    pub proof_hash: String,
    pub obligations: Vec<PointerObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointerObligation {
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasProof {
    pub symbol: String,
    pub proof_hash: String,
    pub obligations: Vec<AliasObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasObligation {
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrnoProof {
    pub symbol: String,
    pub proof_hash: String,
    pub domain: String,
    pub obligations: Vec<ErrnoObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrnoObligation {
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarargsProof {
    pub symbol: String,
    pub proof_hash: String,
    pub obligations: Vec<VarargsObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarargsObligation {
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackProof {
    pub symbol: String,
    pub proof_hash: String,
    pub obligations: Vec<CallbackObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackObligation {
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CWrapperProof {
    pub wrapper_id: String,
    pub proof_hash: String,
    pub obligations: Vec<WrapperObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrapperObligation {
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CCacheProof {
    pub cache_key: String,
    pub proof_hash: String,
    pub obligations: Vec<CacheObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheObligation {
    pub claim: String,
    pub evidence: String,
}

// =============================================================================
// Source Language (for import tracking)
// =============================================================================

/// Source language for cross-language tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SourceLanguage {
    #[serde(alias = "mlir")]
    #[default]
    Chimera,
    C,
    Rust,
    Zig,
    Wasm,
}

impl SourceLanguage {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "chimera" | "mlir" => Some(SourceLanguage::Chimera),
            "c" => Some(SourceLanguage::C),
            "rust" => Some(SourceLanguage::Rust),
            "zig" => Some(SourceLanguage::Zig),
            "wasm" => Some(SourceLanguage::Wasm),
            _ => None,
        }
    }
}

// =============================================================================
// Declaration wrapper for unified handling
// =============================================================================

/// Declaration with kind-specific payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Declaration {
    pub id: DeclId,
    pub kind: DeclarationKind,
}

impl Declaration {
    pub fn name(&self) -> &str {
        match &self.kind {
            DeclarationKind::Function(f) => &f.name,
            DeclarationKind::GlobalVariable(g) => &g.name,
            DeclarationKind::StructDecl(s) => s.name.as_deref().unwrap_or("(anonymous)"),
            DeclarationKind::UnionDecl(u) => u.name.as_deref().unwrap_or("(anonymous)"),
            DeclarationKind::EnumDecl(e) => e.name.as_deref().unwrap_or("(anonymous)"),
            DeclarationKind::TypedefDecl(t) => &t.name,
            DeclarationKind::EnumConstant(e) => &e.name,
            DeclarationKind::MacroDecl(m) => &m.name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_header_validation() {
        let header = ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0");
        header.validate().unwrap();

        let mut bad_header = header;
        bad_header.magic = [0, 0, 0, 0];
        assert!(bad_header.validate().is_err());
    }

    #[test]
    fn test_csnapsnapshot_checksum() {
        let snapshot = CsnapSnapshot {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            clang_version: "17.0.0".to_string(),
            target: CTarget {
                triple: "x86_64-unknown-linux-gnu".to_string(),
                arch: "x86_64".to_string(),
                os: "linux".to_string(),
                env: "gnu".to_string(),
                libc: Some("glibc".to_string()),
                clang_version: Some("17.0.0".to_string()),
                resource_dir: None,
                sysroot: None,
                pointer_width: 64,
                size_of_ptr: 8,
                size_of_int: 4,
                size_of_long: 8,
                size_of_long_long: 8,
                size_of_float: 4,
                size_of_double: 8,
                size_of_long_double: 16,
                size_of_void: 1,
                int64_aligned: 8,
                long_long_aligned: 8,
                double_aligned: 8,
                long_double_aligned: 16,
                long_double_size: 16,
                big_endian: false,
                c_standard: CStandard::C11,
                clang_trust_facts: vec![],
            },
            headers: vec![],
            source_files: vec![],
            declarations: vec![],
            exports: vec![],
            imports: vec![],
            compile_flags: vec![],
            active_macros: vec![],
            conditional_branches: vec![],
        };

        let checksum = snapshot.compute_checksum();
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // blake3 hex length
    }

    #[test]
    fn test_cdep_graph_checksum() {
        let graph = CdepGraph {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            nodes: vec![],
            edges: vec![],
        };

        let checksum = graph.compute_checksum();
        assert!(!checksum.is_empty());
    }

    #[test]
    fn test_castpack_checksum() {
        let pack = CastPack {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            declarations: vec![],
            types: vec![],
            layouts: vec![],
            symbol_table: SymbolTable {
                functions: HashMap::new(),
                globals: HashMap::new(),
                structs: HashMap::new(),
                unions: HashMap::new(),
                enums: HashMap::new(),
                typedefs: HashMap::new(),
                macros: HashMap::new(),
            },
            macro_provenance: MacroProvenance { expansions: vec![] },
            diagnostics: vec![],
        };

        let checksum = pack.compute_checksum();
        assert!(!checksum.is_empty());
    }

    #[test]
    fn test_cchmeta_checksum() {
        let meta = CchMeta {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            declaration_provenance: vec![],
            c_abi_facts: vec![],
            layout_facts: vec![],
            macro_dependencies: vec![],
            include_dependencies: vec![],
            trust_assumptions: vec![],
        };

        let checksum = meta.compute_checksum();
        assert!(!checksum.is_empty());
    }

    #[test]
    fn test_cchproof_checksum() {
        let proof = CchProof {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            layout_proofs: vec![],
            signature_proofs: vec![],
            pointer_proofs: vec![],
            alias_proofs: vec![],
            errno_proofs: vec![],
            varargs_proofs: vec![],
            callback_proofs: vec![],
            wrapper_proofs: vec![],
            cache_proofs: vec![],
        };

        let checksum = proof.compute_checksum();
        assert!(!checksum.is_empty());
    }

    #[test]
    fn test_version_rejected_if_too_new() {
        let header = ArtifactHeader {
            magic: C_ARTIFACT_MAGIC,
            schema_version: 999,
            producer_version: "0.1.0".to_string(),
            target: "x86_64-unknown-linux-gnu".to_string(),
            source_language: "c".to_string(),
        };

        assert!(matches!(
            header.validate(),
            Err(SchemaError::UnsupportedVersion(999))
        ));
    }

    #[test]
    fn test_diagnostic_code_categories() {
        assert_eq!(CDiagnosticCode::ParseUnexpectedToken.category(), "parse");
        assert_eq!(CDiagnosticCode::ClangExtractFailed.category(), "clang");
        assert_eq!(CDiagnosticCode::IncludeNotFound.category(), "include");
        assert_eq!(CDiagnosticCode::MacroUndefined.category(), "macro");
        assert_eq!(CDiagnosticCode::AbiMismatch.category(), "abi");
        assert_eq!(CDiagnosticCode::LayoutSizeMismatch.category(), "layout");
        assert_eq!(
            CDiagnosticCode::PointerNullabilityMismatch.category(),
            "pointer"
        );
        assert_eq!(CDiagnosticCode::VarargsDirectUnsafe.category(), "varargs");
        assert_eq!(CDiagnosticCode::ErrnoMappingMissing.category(), "errno");
        assert_eq!(
            CDiagnosticCode::CallbackCallingConventionMismatch.category(),
            "callback"
        );
        assert_eq!(
            CDiagnosticCode::UnsafeOperationUndocumented.category(),
            "unsafe"
        );
        assert_eq!(CDiagnosticCode::LinkSymbolNotFound.category(), "link");
    }

    #[test]
    fn test_adapter_mode_default() {
        let mode = CAdapterMode::default();
        assert_eq!(mode, CAdapterMode::ClangAuthoritative);
    }

    #[test]
    fn test_c_standard_default() {
        let standard = CStandard::default();
        assert_eq!(standard, CStandard::C11);
    }

    #[test]
    fn test_source_language_parse() {
        assert_eq!(SourceLanguage::parse("c"), Some(SourceLanguage::C));
        assert_eq!(SourceLanguage::parse("C"), Some(SourceLanguage::C));
        assert_eq!(SourceLanguage::parse("rust"), Some(SourceLanguage::Rust));
        assert_eq!(SourceLanguage::parse("zig"), Some(SourceLanguage::Zig));
        assert_eq!(SourceLanguage::parse("unknown"), None);
    }

    #[test]
    fn test_c_artifact_magic() {
        assert_eq!(C_ARTIFACT_MAGIC, [0x43, 0x48, 0x43, 0x53]);
    }

    #[test]
    fn test_decl_id_compute_stable_id() {
        let id1 = DeclId::compute_stable_id(
            "my_func",
            "int (*)(int, int)",
            "fn(i32, i32) -> i32",
            "x86_64-unknown-linux-gnu",
            1,
        );
        let id2 = DeclId::compute_stable_id(
            "my_func",
            "int (*)(int, int)",
            "fn(i32, i32) -> i32",
            "x86_64-unknown-linux-gnu",
            1,
        );
        // Same inputs should produce same ID
        assert_eq!(id1, id2);
        assert!(!id1.is_empty());
    }

    #[test]
    fn test_decl_id_stable_id_different_inputs() {
        let id1 = DeclId::compute_stable_id(
            "my_func",
            "int (*)(int, int)",
            "fn(i32, i32) -> i32",
            "x86_64-unknown-linux-gnu",
            1,
        );
        let id2 = DeclId::compute_stable_id(
            "other_func",
            "int (*)(int, int)",
            "fn(i32, i32) -> i32",
            "x86_64-unknown-linux-gnu",
            1,
        );
        // Different name should produce different ID
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_decl_id_stable_id_different_target() {
        let id1 = DeclId::compute_stable_id(
            "my_func",
            "int (*)(int, int)",
            "fn(i32, i32) -> i32",
            "x86_64-unknown-linux-gnu",
            1,
        );
        let id2 = DeclId::compute_stable_id(
            "my_func",
            "int (*)(int, int)",
            "fn(i32, i32) -> i32",
            "aarch64-unknown-linux-gnu",
            1,
        );
        // Different target should produce different ID
        assert_ne!(id1, id2);
    }

    // =============================================================================
    // Deterministic Encoding Tests (Task 51)
    // =============================================================================

    #[test]
    fn test_deterministic_hash_same_input() {
        let data = CsnapSnapshot {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            clang_version: "17.0.0".to_string(),
            target: CTarget {
                triple: "x86_64-unknown-linux-gnu".to_string(),
                arch: "x86_64".to_string(),
                os: "linux".to_string(),
                env: "gnu".to_string(),
                libc: Some("glibc".to_string()),
                clang_version: Some("17.0.0".to_string()),
                resource_dir: None,
                sysroot: None,
                pointer_width: 64,
                size_of_ptr: 8,
                size_of_int: 4,
                size_of_long: 8,
                size_of_long_long: 8,
                size_of_float: 4,
                size_of_double: 8,
                size_of_long_double: 16,
                size_of_void: 1,
                int64_aligned: 8,
                long_long_aligned: 8,
                double_aligned: 8,
                long_double_aligned: 16,
                long_double_size: 16,
                big_endian: false,
                c_standard: CStandard::C11,
                clang_trust_facts: vec![],
            },
            headers: vec![],
            source_files: vec![],
            declarations: vec![],
            exports: vec![],
            imports: vec![],
            compile_flags: vec![],
            active_macros: vec![],
            conditional_branches: vec![],
        };

        let hash1 = compute_deterministic_hash(&data).unwrap();
        let hash2 = compute_deterministic_hash(&data).unwrap();

        // Same input should produce same hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // blake3 hex length
    }

    #[test]
    fn test_deterministic_hash_different_order() {
        // Create two snapshots with same content but built in different order
        let mut data1 = CsnapSnapshot {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            clang_version: "17.0.0".to_string(),
            target: CTarget {
                triple: "x86_64-unknown-linux-gnu".to_string(),
                arch: "x86_64".to_string(),
                os: "linux".to_string(),
                env: "gnu".to_string(),
                libc: Some("glibc".to_string()),
                clang_version: Some("17.0.0".to_string()),
                resource_dir: None,
                sysroot: None,
                pointer_width: 64,
                size_of_ptr: 8,
                size_of_int: 4,
                size_of_long: 8,
                size_of_long_long: 8,
                size_of_float: 4,
                size_of_double: 8,
                size_of_long_double: 16,
                size_of_void: 1,
                int64_aligned: 8,
                long_long_aligned: 8,
                double_aligned: 8,
                long_double_aligned: 16,
                long_double_size: 16,
                big_endian: false,
                c_standard: CStandard::C11,
                clang_trust_facts: vec![],
            },
            headers: vec![],
            source_files: vec![],
            declarations: vec![],
            exports: vec![],
            imports: vec![],
            compile_flags: vec![],
            active_macros: vec![],
            conditional_branches: vec![],
        };

        let mut data2 = CsnapSnapshot {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            clang_version: "17.0.0".to_string(),
            target: CTarget {
                triple: "x86_64-unknown-linux-gnu".to_string(),
                arch: "x86_64".to_string(),
                os: "linux".to_string(),
                env: "gnu".to_string(),
                libc: Some("glibc".to_string()),
                clang_version: Some("17.0.0".to_string()),
                resource_dir: None,
                sysroot: None,
                pointer_width: 64,
                size_of_ptr: 8,
                size_of_int: 4,
                size_of_long: 8,
                size_of_long_long: 8,
                size_of_float: 4,
                size_of_double: 8,
                size_of_long_double: 16,
                size_of_void: 1,
                int64_aligned: 8,
                long_long_aligned: 8,
                double_aligned: 8,
                long_double_aligned: 16,
                long_double_size: 16,
                big_endian: false,
                c_standard: CStandard::C11,
                clang_trust_facts: vec![],
            },
            headers: vec![],
            source_files: vec![],
            declarations: vec![],
            exports: vec![],
            imports: vec![],
            compile_flags: vec![],
            active_macros: vec![],
            conditional_branches: vec![],
        };

        let hash1 = compute_deterministic_hash(&data1).unwrap();
        let hash2 = compute_deterministic_hash(&data2).unwrap();

        // Same data built at different times should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_deterministic_hash_different_content() {
        let data1 = CsnapSnapshot {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            clang_version: "17.0.0".to_string(),
            target: CTarget {
                triple: "x86_64-unknown-linux-gnu".to_string(),
                arch: "x86_64".to_string(),
                os: "linux".to_string(),
                env: "gnu".to_string(),
                libc: Some("glibc".to_string()),
                clang_version: Some("17.0.0".to_string()),
                resource_dir: None,
                sysroot: None,
                pointer_width: 64,
                size_of_ptr: 8,
                size_of_int: 4,
                size_of_long: 8,
                size_of_long_long: 8,
                size_of_float: 4,
                size_of_double: 8,
                size_of_long_double: 16,
                size_of_void: 1,
                int64_aligned: 8,
                long_long_aligned: 8,
                double_aligned: 8,
                long_double_aligned: 16,
                long_double_size: 16,
                big_endian: false,
                c_standard: CStandard::C11,
                clang_trust_facts: vec![],
            },
            headers: vec![],
            source_files: vec![],
            declarations: vec![],
            exports: vec![],
            imports: vec![],
            compile_flags: vec![],
            active_macros: vec![],
            conditional_branches: vec![],
        };

        let mut data2 = data1.clone();
        data2.clang_version = "18.0.0".to_string();

        let hash1 = compute_deterministic_hash(&data1).unwrap();
        let hash2 = compute_deterministic_hash(&data2).unwrap();

        // Different content should produce different hash
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_cdep_graph_deterministic_hash() {
        let graph1 = CdepGraph {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            nodes: vec![],
            edges: vec![],
        };

        let graph2 = CdepGraph {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            nodes: vec![],
            edges: vec![],
        };

        let hash1 = compute_deterministic_hash(&graph1).unwrap();
        let hash2 = compute_deterministic_hash(&graph2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_castpack_deterministic_hash() {
        let pack1 = CastPack {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            declarations: vec![],
            types: vec![],
            layouts: vec![],
            symbol_table: SymbolTable {
                functions: HashMap::new(),
                globals: HashMap::new(),
                structs: HashMap::new(),
                unions: HashMap::new(),
                enums: HashMap::new(),
                typedefs: HashMap::new(),
                macros: HashMap::new(),
            },
            macro_provenance: MacroProvenance { expansions: vec![] },
            diagnostics: vec![],
        };

        let pack2 = CastPack {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            declarations: vec![],
            types: vec![],
            layouts: vec![],
            symbol_table: SymbolTable {
                functions: HashMap::new(),
                globals: HashMap::new(),
                structs: HashMap::new(),
                unions: HashMap::new(),
                enums: HashMap::new(),
                typedefs: HashMap::new(),
                macros: HashMap::new(),
            },
            macro_provenance: MacroProvenance { expansions: vec![] },
            diagnostics: vec![],
        };

        let hash1 = compute_deterministic_hash(&pack1).unwrap();
        let hash2 = compute_deterministic_hash(&pack2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_cchmeta_deterministic_hash() {
        let meta1 = CchMeta {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            declaration_provenance: vec![],
            c_abi_facts: vec![],
            layout_facts: vec![],
            macro_dependencies: vec![],
            include_dependencies: vec![],
            trust_assumptions: vec![],
        };

        let meta2 = CchMeta {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            declaration_provenance: vec![],
            c_abi_facts: vec![],
            layout_facts: vec![],
            macro_dependencies: vec![],
            include_dependencies: vec![],
            trust_assumptions: vec![],
        };

        let hash1 = compute_deterministic_hash(&meta1).unwrap();
        let hash2 = compute_deterministic_hash(&meta2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_cchproof_deterministic_hash() {
        let proof1 = CchProof {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            layout_proofs: vec![],
            signature_proofs: vec![],
            pointer_proofs: vec![],
            alias_proofs: vec![],
            errno_proofs: vec![],
            varargs_proofs: vec![],
            callback_proofs: vec![],
            wrapper_proofs: vec![],
            cache_proofs: vec![],
        };

        let proof2 = CchProof {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            layout_proofs: vec![],
            signature_proofs: vec![],
            pointer_proofs: vec![],
            alias_proofs: vec![],
            errno_proofs: vec![],
            varargs_proofs: vec![],
            callback_proofs: vec![],
            wrapper_proofs: vec![],
            cache_proofs: vec![],
        };

        let hash1 = compute_deterministic_hash(&proof1).unwrap();
        let hash2 = compute_deterministic_hash(&proof2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_deterministic_hash_with_headers() {
        // Test that deterministic hash works with populated headers
        let data = CsnapSnapshot {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            clang_version: "17.0.0".to_string(),
            target: CTarget {
                triple: "x86_64-unknown-linux-gnu".to_string(),
                arch: "x86_64".to_string(),
                os: "linux".to_string(),
                env: "gnu".to_string(),
                libc: Some("glibc".to_string()),
                clang_version: Some("17.0.0".to_string()),
                resource_dir: None,
                sysroot: None,
                pointer_width: 64,
                size_of_ptr: 8,
                size_of_int: 4,
                size_of_long: 8,
                size_of_long_long: 8,
                size_of_float: 4,
                size_of_double: 8,
                size_of_long_double: 16,
                size_of_void: 1,
                int64_aligned: 8,
                long_long_aligned: 8,
                double_aligned: 8,
                long_double_aligned: 16,
                long_double_size: 16,
                big_endian: false,
                c_standard: CStandard::C11,
                clang_trust_facts: vec!["sizeof_long_is_8".to_string()],
            },
            headers: vec![HeaderInfo {
                path: "/usr/include/stdio.h".to_string(),
                content_hash: "abc123".to_string(),
                size: 1024,
                mtime: 1234567890,
                include_guard: Some("stdio_h".to_string()),
                includes: vec![],
                macro_defs: vec![],
                is_system: true,
                is_generated: false,
            }],
            source_files: vec![],
            declarations: vec![],
            exports: vec![],
            imports: vec![],
            compile_flags: vec!["-O2".to_string()],
            active_macros: vec![],
            conditional_branches: vec![],
        };

        let hash1 = compute_deterministic_hash(&data).unwrap();
        let hash2 = compute_deterministic_hash(&data).unwrap();

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    // =============================================================================
    // Artifact Migration Policy Tests (Task 52)
    // =============================================================================

    #[test]
    fn test_migration_policy_accept_current_version() {
        use super::migration::MigrationPolicy;
        let policy = MigrationPolicy::new(1);
        let action = policy.get_action(1);
        assert!(matches!(action, super::migration::MigrationAction::Accept));
    }

    #[test]
    fn test_migration_policy_reject_too_old() {
        use super::migration::MigrationPolicy;
        let policy = MigrationPolicy::new(200); // version 2.x
        let action = policy.get_action(50); // version 0.50 - way too old
        assert!(matches!(
            action,
            super::migration::MigrationAction::Reject { .. }
        ));
    }

    #[test]
    fn test_migration_policy_reject_future_version() {
        use super::migration::MigrationPolicy;
        let policy = MigrationPolicy::new(1);
        let action = policy.get_action(999);
        assert!(matches!(
            action,
            super::migration::MigrationAction::Reject { .. }
        ));
    }

    #[test]
    fn test_migration_policy_migrate_older_compatible() {
        use super::migration::MigrationPolicy;
        let policy = MigrationPolicy::new(200); // version 2.0
        let action = policy.get_action(100); // version 1.0 - should be migratable
        assert!(matches!(action, super::migration::MigrationAction::Migrate));
    }

    #[test]
    fn test_migration_policy_default_policy() {
        use super::migration::{default_policy, MigrationPolicy};
        let policy = default_policy();
        // Current schema version is 1
        assert_eq!(policy.current_version, 1);
        // min_supported should be 0 since 1/1000*100 = 0 for version 1
        assert!(policy.is_compatible(1));
    }

    #[test]
    fn test_migration_policy_is_compatible() {
        use super::migration::MigrationPolicy;
        let policy = MigrationPolicy::new(200);
        assert!(policy.is_compatible(200));
        assert!(policy.is_compatible(150)); // version 1.50 - migratable
        assert!(!policy.is_compatible(50)); // version 0.50 - too old
        assert!(!policy.is_compatible(999));
    }

    #[test]
    fn test_migration_policy_needs_migration() {
        use super::migration::MigrationPolicy;
        let policy = MigrationPolicy::new(200);
        assert!(!policy.needs_migration(200));
        assert!(policy.needs_migration(100));
        assert!(!policy.needs_migration(50));
    }

    #[test]
    fn test_migration_policy_with_explicit_min_version() {
        use super::migration::MigrationPolicy;
        let policy = MigrationPolicy::with_min_version(200, 100);
        // version 50 is below our explicit min of 100
        let action = policy.get_action(50);
        assert!(matches!(
            action,
            super::migration::MigrationAction::Reject { .. }
        ));
        // version 100 should be migratable (to 200)
        let action = policy.get_action(100);
        assert!(matches!(action, super::migration::MigrationAction::Migrate));
        // version 200 should be accepted
        let action = policy.get_action(200);
        assert!(matches!(action, super::migration::MigrationAction::Accept));
    }

    #[test]
    fn test_migration_action_display() {
        use super::migration::MigrationAction;
        let reject = MigrationAction::Reject {
            reason: "too old".to_string(),
        };
        let action_str = format!("{:?}", reject);
        assert!(action_str.contains("Reject"));
        assert!(action_str.contains("too old"));
    }
}
