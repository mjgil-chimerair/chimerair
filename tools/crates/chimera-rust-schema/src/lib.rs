//! Chimera Rust schema definitions for artifact formats.
//!
//! This crate defines the schema for Rust-specific artifacts:
//! - `.rsnap`: Semantic snapshot containing crate graph, items, exports
//! - `.rdep`: Dependency graph for incremental cache invalidation
//! - `.rmirpack`: MIR body package with normalized rustc IDs
//! - `.rchmeta`: Rust-specific metadata before common `.chmeta`
//! - `.rchproof`: Rust proof facts for Lean bridge

use serde::{Deserialize, Serialize};

/// Magic bytes for Rust artifacts: b"CHRS" (Chimera Rust Schema)
pub const RUST_ARTIFACT_MAGIC: [u8; 4] = [0x43, 0x48, 0x52, 0x53];

/// Schema version for all Rust artifacts
pub const CURRENT_SCHEMA_VERSION: u32 = 2;

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
            magic: RUST_ARTIFACT_MAGIC,
            schema_version: CURRENT_SCHEMA_VERSION,
            producer_version: producer_version.to_string(),
            target: target.to_string(),
            source_language: "rust".to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), SchemaError> {
        if self.magic != RUST_ARTIFACT_MAGIC {
            return Err(SchemaError::InvalidMagic(self.magic));
        }
        if self.schema_version < 1 {
            return Err(SchemaError::IncompatibleVersion(self.schema_version, 1));
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
// .rsnap - Semantic Snapshot
// =============================================================================

/// Semantic snapshot containing crate graph, items, and exports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsnapSnapshot {
    pub header: ArtifactHeader,
    pub checksum: String,
    pub rustc_version: String,
    pub crate_graph: CrateGraph,
    pub items: Vec<RsnapItem>,
    pub exports: Vec<RsnapExport>,
    pub source_files: Vec<SourceFile>,
}

impl RsnapSnapshot {
    pub fn compute_checksum(&self) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        // Canonical JSON serialization for stable hashing
        let bytes = serde_json::to_vec(self).unwrap();
        hasher.update(&bytes);
        hasher.finalize().to_hex().to_string()
    }
}

/// Crate graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateNode {
    pub id: CrateId,
    pub name: String,
    #[serde(default)]
    pub package_name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub source_kind: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub source_ref: Option<String>,
    pub edition: String,
    pub crate_type: CrateType,
    pub dependency_crates: Vec<CrateId>,
    pub extern_prelude: Vec<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default = "default_true")]
    pub default_features: bool,
    #[serde(default)]
    pub optional: bool,
}

fn default_true() -> bool {
    true
}

/// Stable crate identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CrateId(pub u64);

/// Crate type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrateType {
    Library,
    Binary,
    Cdylib,
    Rlib,
    ProcMacro,
}

/// Crate graph containing all dependency nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateGraph {
    pub root: CrateId,
    pub nodes: Vec<CrateNode>,
}

/// Item table entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsnapItem {
    pub id: ItemId,
    pub def_path: String,
    pub kind: ItemKind,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
    pub generics: Option<Generics>,
    pub where_clauses: Vec<WhereClause>,
}

/// Stable item identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u64);

/// Kind of item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemKind {
    Function,
    Static,
    Constant,
    Struct,
    Enum,
    Union,
    Trait,
    Impl,
    Type,
    Module,
    ExternBlock,
}

/// Item visibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visibility {
    pub rank: VisibilityRank,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisibilityRank {
    Private,
    PubCrate,
    PubRestricted,
    Pub,
    PubSuper,
    PubSelf,
}

/// Item attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub path: String,
    pub tokens: String,
}

/// Generic parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generics {
    pub lifetimes: Vec<String>,
    pub type_params: Vec<TypeParam>,
    pub const_params: Vec<ConstParam>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeParam {
    pub name: String,
    pub bounds: Vec<TraitBound>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstParam {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereClause {
    pub bound: TraitBound,
}

/// Trait bound
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitBound {
    pub path: String,
    pub generic_args: Vec<String>,
}

/// Export table entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsnapExport {
    pub symbol: String,
    pub item_id: ItemId,
    pub abi: String,
    pub linkage: Linkage,
}

/// Symbol linkage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Linkage {
    None,
    External,
    Weak,
    LinkOnce,
}

/// Source file reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub content_hash: String,
}

// =============================================================================
// .rdep - Dependency Graph
// =============================================================================

/// Dependency graph for incremental cache invalidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdepGraph {
    pub header: ArtifactHeader,
    pub checksum: String,
    pub nodes: Vec<DepNode>,
    pub edges: Vec<DepEdge>,
}

impl RdepGraph {
    pub fn compute_checksum(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        let bytes = serde_json::to_vec(self).unwrap();
        hasher.update(&bytes);
        hasher.finalize().to_hex().to_string()
    }
}

/// Dependency node kinds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepNodeKind {
    Source,
    Item,
    Type,
    Layout,
    MirBody,
    GenericInstantiation,
    ConstEval,
    Export,
    Object,
    Wrapper,
    Proof,
}

/// Dependency node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepNode {
    pub id: DepNodeId,
    pub kind: DepNodeKind,
    pub fingerprint: String,
    pub stable_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DepNodeId(pub u64);

/// Dependency edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepEdge {
    pub from: DepNodeId,
    pub to: DepNodeId,
    pub kind: DepEdgeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DepEdgeKind {
    DependsOn,
    Monomorphizes,
    Instantiates,
    Provides,
    Requires,
}

// =============================================================================
// .rmirpack - MIR Package
// =============================================================================

/// MIR body package containing normalized rustc IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmirPack {
    pub header: ArtifactHeader,
    pub checksum: String,
    pub types: Vec<TypeDef>,
    pub layouts: Vec<LayoutDef>,
    pub bodies: Vec<MirBody>,
    pub constants: Vec<ConstDef>,
}

impl RmirPack {
    pub fn compute_checksum(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        let bytes = serde_json::to_vec(self).unwrap();
        hasher.update(&bytes);
        hasher.finalize().to_hex().to_string()
    }
}

/// MIR body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirBody {
    pub item_id: ItemId,
    pub locals: Vec<LocalDef>,
    pub blocks: Vec<BasicBlock>,
}

/// Local definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDef {
    pub index: u32,
    pub ty: TypeRef,
    pub is_return_slot: bool,
    pub is_arg: bool,
}

/// Basic block in MIR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    pub index: u32,
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

/// MIR statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Assign { place: Place, value: Rvalue },
    StorageLive(u32),
    StorageDead(u32),
    SetDiscriminant { place: Place, variant_index: u32 },
    Deinit(Place),
    Retag(Place),
    FakeRead(Place),
}

/// Place in MIR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    pub local: u32,
    pub projection: Vec<Projection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Projection {
    Deref,
    Field(u32),
    Index(LocalDef),
    Downcast(u32),
    SubSlice { from: u32, to: Option<u32> },
}

/// Rvalue in MIR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Rvalue {
    Use(Place),
    Copy(Place),
    Move(Place),
    Borrow(BorrowKind, Place),
    AddressOf(bool, Place), // (is_mut, place) - raw address operation
    Aggregate(AggregateKind, Vec<Place>),
    Cast(CastKind, Place, TypeRef),
    BinOp(BinOp, Place, Place),
    UnOp(UnOp, Place),
}

/// Borrow kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BorrowKind {
    Shared,
    Mut,
    TwoPhaseMut,
    Shallow,
}

/// Aggregate kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregateKind {
    Tuple,
    Struct(String),
    Enum(String, u32),
    Union(String),
    Array,
}

/// Cast kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CastKind {
    Transmute,
    IntToInt,
    FloatToInt,
    IntToFloat,
    FloatToFloat,
    PtrToPtr,
    FnPtrToPtr,
}

/// Binary operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
    Offset,
}

/// Unary operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnOp {
    Not,
    Neg,
}

/// MIR terminator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Terminator {
    Goto {
        target: u32,
    },
    SwitchInt {
        discr: Place,
        targets: Vec<u32>,
        otherwise: u32,
    },
    Return,
    Call {
        func: Place,
        args: Vec<Place>,
        destination: Place,
        target: Option<u32>,
        cleanup: Option<u32>,
    },
    Drop {
        place: Place,
        target: u32,
        unwind: Option<u32>,
        replace: bool,
    },
    Assert {
        cond: Place,
        expected: bool,
        target: u32,
        cleanup: Option<u32>,
        msg: String,
    },
    Abort,
    Resume,
    Unreachable,
    Yield,
}

/// Type reference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeRef(pub u32);

/// Type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeDef {
    Primitive(PrimitiveType),
    Ref(TypeRef, BorrowKind),
    RawPtr(TypeRef, bool),
    Array(TypeRef, u64),
    Slice(TypeRef),
    Tuple(Vec<TypeRef>),
    Adt {
        name: String,
        variants: Vec<AdtVariant>,
        repr: AdtRepr,
    },
    FnPtr {
        params: Vec<TypeRef>,
        ret: TypeRef,
    },
    Closure {
        upvars: Vec<TypeRef>,
    },
    TraitObject {
        traits: Vec<String>,
    },
    AssociatedType {
        trait_id: ItemId,
        name: String,
    },
    Alias(TypeAliasKind),
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveType {
    Bool,
    Char,
    Str,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    F32,
    F64,
    Unit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdtVariant {
    pub name: String,
    pub fields: Vec<TypeRef>,
    pub discriminant: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdtRepr {
    pub kind: AdtReprKind,
    pub pack: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdtReprKind {
    C,
    Transparent,
    Rust,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeAliasKind {
    Weak,
    Rigid,
}

/// Layout definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutDef {
    pub ty: TypeRef,
    pub size: u64,
    pub align: u32,
    pub kind: LayoutKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutKind {
    Primitive,
    Struct {
        fields: Vec<FieldLayout>,
    },
    Enum {
        niche: Option<NicheLayout>,
        variants: Vec<VariantLayout>,
    },
    Union {
        variants: Vec<VariantLayout>,
    },
    Vector {
        element: TypeRef,
        count: u64,
    },
    FatPtr {
        data: TypeRef,
        meta: TypeRef,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLayout {
    pub field_idx: u32,
    pub offset: u64,
    pub ty: TypeRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheLayout {
    pub offset: u64,
    pub size: u64,
    pub valid_range: (u64, u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantLayout {
    pub index: u32,
    pub offset: Option<u64>,
    pub size: Option<u64>,
}

/// Constant definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstDef {
    pub id: ConstId,
    pub ty: TypeRef,
    pub kind: ConstKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConstId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstKind {
    Scalar(u64),
    ZeroSized,
    Aggregate(Vec<u8>),
    Generic(String),
    PromotedMir(MirBody),
}

// =============================================================================
// .rchmeta - Rust-specific Metadata
// =============================================================================

/// Rust-specific metadata before conversion to common `.chmeta`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RchMeta {
    pub header: ArtifactHeader,
    pub checksum: String,
    pub crate_info: CrateInfo,
    pub layout_facts: Vec<LayoutFact>,
    pub abi_facts: Vec<AbiFact>,
    pub ownership_facts: Vec<OwnershipFact>,
    pub effect_facts: Vec<EffectFact>,
    pub trust_assumptions: Vec<TrustAssumption>,
}

impl RchMeta {
    pub fn compute_checksum(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        let bytes = serde_json::to_vec(self).unwrap();
        hasher.update(&bytes);
        hasher.finalize().to_hex().to_string()
    }
}

/// Crate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub root: ItemId,
    pub panic_policy: PanicPolicy,
}

/// Panic policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanicPolicy {
    Unwind,
    Abort,
}

/// Layout fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutFact {
    pub item_id: ItemId,
    pub size: u64,
    pub align: u32,
    pub fields: Vec<FieldFact>,
    pub proof_hash: String,
}

/// Field fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldFact {
    pub name: Option<String>,
    pub offset: u64,
    pub ty: TypeRef,
}

/// ABI fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiFact {
    pub symbol: String,
    pub abi: String,
    pub calling_convention: CallingConvention,
    pub params: Vec<AbiParam>,
    pub ret: Option<AbiParam>,
    pub panic_policy: PanicPolicy,
    pub proof_hash: String,
}

/// Calling convention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallingConvention {
    C,
    CUnwind,
    Rust,
    RustCall,
    Intrinsic,
    Other(String),
}

/// ABI parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiParam {
    pub ty: TypeRef,
    pub passing: PassingConvention,
    pub by_val: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PassingConvention {
    ByValue,
    ByRef,
    ByHint,
}

/// Ownership fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipFact {
    pub item_id: ItemId,
    pub kind: OwnershipKind,
    pub drop_kind: DropKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnershipKind {
    Owned,
    Borrowed {
        mutable: bool,
        region: Option<RegionKind>,
    },
    Raw,
    Static,
}

/// Region kind approximation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegionKind {
    Call,
    Local,
    Static,
}

/// Drop kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DropKind {
    ManuallyDrop,
    TrivialDrop,
    NeedsDrop,
    TrackedDrop,
}

/// Effect fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectFact {
    pub item_id: ItemId,
    pub effects: EffectSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectSet {
    pub may_panic: bool,
    pub may_alloc: bool,
    pub may_dealloc: bool,
    pub may_ffi: bool,
    pub may_error: bool,
    pub may_block: bool,
    pub is_unsafe: bool,
    pub thread_affecting: bool,
}

/// Trust assumption for unsafe operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAssumption {
    pub span: String,
    pub kind: TrustKind,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustKind {
    UnsafeOperation,
    RawPtrDeref,
    FFICall,
    MutableStatic,
    UnionAccess,
    InlineAsm,
    UnsafeFn,
}

// =============================================================================
// .rchproof - Rust Proof Facts
// =============================================================================

/// Rust-specific proof facts for Lean bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RchProof {
    pub header: ArtifactHeader,
    pub checksum: String,
    pub layout_proofs: Vec<LayoutProof>,
    pub abi_proofs: Vec<AbiProof>,
    pub ownership_proofs: Vec<OwnershipProof>,
    pub panic_proofs: Vec<PanicProof>,
    pub result_proofs: Vec<ResultProof>,
    pub unsafe_proofs: Vec<UnsafeProof>,
    pub wrapper_proofs: Vec<WrapperProof>,
    pub cache_proofs: Vec<CacheProof>,
}

impl RchProof {
    pub fn compute_checksum(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        let bytes = serde_json::to_vec(self).unwrap();
        hasher.update(&bytes);
        hasher.finalize().to_hex().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutProof {
    pub item_id: ItemId,
    pub proof_hash: String,
    pub obligations: Vec<LayoutObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutObligation {
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiProof {
    pub symbol: String,
    pub proof_hash: String,
    pub obligations: Vec<AbiObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiObligation {
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipProof {
    pub item_id: ItemId,
    pub proof_hash: String,
    pub obligations: Vec<OwnershipObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipObligation {
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicProof {
    pub item_id: ItemId,
    pub policy: PanicPolicy,
    pub proof_hash: String,
    pub obligations: Vec<PanicObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicObligation {
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultProof {
    pub symbol: String,
    pub proof_hash: String,
    pub obligations: Vec<ResultObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultObligation {
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeProof {
    pub item_id: ItemId,
    pub proof_hash: String,
    pub obligations: Vec<UnsafeObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeObligation {
    pub span: String,
    pub claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrapperProof {
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
pub struct CacheProof {
    pub cache_key: String,
    pub proof_hash: String,
    pub obligations: Vec<CacheObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheObligation {
    pub claim: String,
    pub evidence: String,
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
    fn test_checksum_computation() {
        let snapshot = RsnapSnapshot {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            rustc_version: "1.70.0".to_string(),
            crate_graph: CrateGraph {
                root: CrateId(0),
                nodes: vec![],
            },
            items: vec![],
            exports: vec![],
            source_files: vec![],
        };

        let checksum = snapshot.compute_checksum();
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // blake3 hex length
    }

    #[test]
    fn test_rdep_graph_checksum() {
        let graph = RdepGraph {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            nodes: vec![],
            edges: vec![],
        };

        let checksum = graph.compute_checksum();
        assert!(!checksum.is_empty());
    }

    #[test]
    fn test_rmirpack_checksum() {
        let pack = RmirPack {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            types: vec![],
            layouts: vec![],
            bodies: vec![],
            constants: vec![],
        };

        let checksum = pack.compute_checksum();
        assert!(!checksum.is_empty());
    }

    #[test]
    fn test_rchmeta_checksum() {
        let meta = RchMeta {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            crate_info: CrateInfo {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                edition: "2021".to_string(),
                root: ItemId(0),
                panic_policy: PanicPolicy::Abort,
            },
            layout_facts: vec![],
            abi_facts: vec![],
            ownership_facts: vec![],
            effect_facts: vec![],
            trust_assumptions: vec![],
        };

        let checksum = meta.compute_checksum();
        assert!(!checksum.is_empty());
    }

    #[test]
    fn test_rchproof_checksum() {
        let proof = RchProof {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            layout_proofs: vec![],
            abi_proofs: vec![],
            ownership_proofs: vec![],
            panic_proofs: vec![],
            result_proofs: vec![],
            unsafe_proofs: vec![],
            wrapper_proofs: vec![],
            cache_proofs: vec![],
        };

        let checksum = proof.compute_checksum();
        assert!(!checksum.is_empty());
    }

    #[test]
    fn test_version_rejected_if_too_new() {
        let header = ArtifactHeader {
            magic: RUST_ARTIFACT_MAGIC,
            schema_version: 999,
            producer_version: "0.1.0".to_string(),
            target: "x86_64-unknown-linux-gnu".to_string(),
            source_language: "rust".to_string(),
        };

        assert!(matches!(
            header.validate(),
            Err(SchemaError::UnsupportedVersion(999))
        ));
    }

    // =============================================================================
    // Golden artifact and roundtrip tests
    // =============================================================================

    #[test]
    fn test_rsnap_roundtrip() {
        let snapshot = RsnapSnapshot {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            rustc_version: "1.75.0".to_string(),
            crate_graph: CrateGraph {
                root: CrateId(0),
                nodes: vec![CrateNode {
                    id: CrateId(0),
                    name: "test_crate".to_string(),
                    package_name: Some("test-crate".to_string()),
                    version: Some("0.1.0".to_string()),
                    source_kind: Some("path".to_string()),
                    source: Some("/workspace/test-crate".to_string()),
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
                def_path: "test_crate::add".to_string(),
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
                abi: "Rust".to_string(),
                linkage: Linkage::None,
            }],
            source_files: vec![SourceFile {
                path: "src/lib.rs".to_string(),
                content_hash: "abc123".to_string(),
            }],
        };

        let bytes = serde_json::to_vec(&snapshot).unwrap();
        let deserialized: RsnapSnapshot = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(deserialized.header.schema_version, 2);
        assert_eq!(deserialized.rustc_version, "1.75.0");
        assert_eq!(deserialized.crate_graph.nodes.len(), 1);
        assert_eq!(
            deserialized.crate_graph.nodes[0].source_kind.as_deref(),
            Some("path")
        );
        assert_eq!(
            deserialized.crate_graph.nodes[0].source.as_deref(),
            Some("/workspace/test-crate")
        );
        assert_eq!(deserialized.items.len(), 1);
        assert_eq!(deserialized.exports.len(), 1);
        assert_eq!(deserialized.exports[0].symbol, "add");
    }

    #[test]
    fn test_rdep_roundtrip() {
        let graph = RdepGraph {
            header: ArtifactHeader::new("aarch64-apple-darwin", "0.2.0"),
            checksum: String::new(),
            nodes: vec![
                DepNode {
                    id: DepNodeId(0),
                    kind: DepNodeKind::Source,
                    fingerprint: "fingerprint0".to_string(),
                    stable_id: "stable0".to_string(),
                },
                DepNode {
                    id: DepNodeId(1),
                    kind: DepNodeKind::Item,
                    fingerprint: "fingerprint1".to_string(),
                    stable_id: "stable1".to_string(),
                },
            ],
            edges: vec![DepEdge {
                from: DepNodeId(0),
                to: DepNodeId(1),
                kind: DepEdgeKind::DependsOn,
            }],
        };

        let bytes = serde_json::to_vec(&graph).unwrap();
        let deserialized: RdepGraph = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(deserialized.nodes.len(), 2);
        assert_eq!(deserialized.edges.len(), 1);
        assert!(matches!(deserialized.edges[0].kind, DepEdgeKind::DependsOn));
    }

    #[test]
    fn test_rmirpack_roundtrip() {
        let pack = RmirPack {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            types: vec![TypeDef::Primitive(PrimitiveType::I32)],
            layouts: vec![LayoutDef {
                ty: TypeRef(0),
                size: 4,
                align: 4,
                kind: LayoutKind::Primitive,
            }],
            bodies: vec![],
            constants: vec![],
        };

        let bytes = serde_json::to_vec(&pack).unwrap();
        let deserialized: RmirPack = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(deserialized.layouts.len(), 1);
        assert_eq!(deserialized.layouts[0].size, 4);
    }

    #[test]
    fn test_rchmeta_roundtrip() {
        let meta = RchMeta {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            crate_info: CrateInfo {
                name: "my_lib".to_string(),
                version: "1.0.0".to_string(),
                edition: "2021".to_string(),
                root: ItemId(0),
                panic_policy: PanicPolicy::Unwind,
            },
            layout_facts: vec![],
            abi_facts: vec![],
            ownership_facts: vec![],
            effect_facts: vec![],
            trust_assumptions: vec![],
        };

        let bytes = serde_json::to_vec(&meta).unwrap();
        let deserialized: RchMeta = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(deserialized.crate_info.name, "my_lib");
        assert_eq!(deserialized.crate_info.panic_policy, PanicPolicy::Unwind);
    }

    #[test]
    fn test_rchproof_roundtrip() {
        let proof = RchProof {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            layout_proofs: vec![LayoutProof {
                item_id: ItemId(5),
                proof_hash: "hashabc".to_string(),
                obligations: vec![LayoutObligation {
                    claim: "size_is_8".to_string(),
                    evidence: "std::mem::size_of::<MyStruct>()".to_string(),
                }],
            }],
            abi_proofs: vec![],
            ownership_proofs: vec![],
            panic_proofs: vec![],
            result_proofs: vec![],
            unsafe_proofs: vec![],
            wrapper_proofs: vec![],
            cache_proofs: vec![],
        };

        let bytes = serde_json::to_vec(&proof).unwrap();
        let deserialized: RchProof = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(deserialized.layout_proofs.len(), 1);
        assert_eq!(
            deserialized.layout_proofs[0].obligations[0].claim,
            "size_is_8"
        );
    }

    #[test]
    fn test_incompatible_version() {
        let header = ArtifactHeader {
            magic: RUST_ARTIFACT_MAGIC,
            schema_version: 0,
            producer_version: "0.1.0".to_string(),
            target: "x86_64-unknown-linux-gnu".to_string(),
            source_language: "rust".to_string(),
        };

        // Version 0 is incompatible with minimum version 1
        assert!(matches!(
            header.validate(),
            Err(SchemaError::IncompatibleVersion(0, 1))
        ));
    }

    #[test]
    fn test_invalid_magic_bytes() {
        let header = ArtifactHeader {
            magic: [0x41, 0x42, 0x43, 0x44], // "ABCD" instead of "CHRS"
            schema_version: 1,
            producer_version: "0.1.0".to_string(),
            target: "x86_64-unknown-linux-gnu".to_string(),
            source_language: "rust".to_string(),
        };

        assert!(matches!(
            header.validate(),
            Err(SchemaError::InvalidMagic(_))
        ));
    }

    #[test]
    fn test_future_schema_version_rejected() {
        // Version 3 is not yet supported
        let header = ArtifactHeader {
            magic: RUST_ARTIFACT_MAGIC,
            schema_version: 3,
            producer_version: "0.1.0".to_string(),
            target: "x86_64-unknown-linux-gnu".to_string(),
            source_language: "rust".to_string(),
        };

        assert!(matches!(
            header.validate(),
            Err(SchemaError::UnsupportedVersion(3))
        ));
    }

    #[test]
    fn test_corrupted_data_detection() {
        // Create a valid snapshot
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

        let mut bytes = serde_json::to_vec(&snapshot).unwrap();
        // Corrupt the data by modifying a byte in the middle
        if bytes.len() > 10 {
            bytes[10] ^= 0xFF;
        }

        // Verify checksum mismatch would be detected
        let deserialized: Result<RsnapSnapshot, _> = serde_json::from_slice(&bytes);
        // Either deserialization fails or checksum validation would fail
        // We check that the error is detectable
        assert!(
            deserialized.is_err() || {
                let d = deserialized.unwrap();
                d.compute_checksum() != snapshot.compute_checksum()
            }
        );
    }

    #[test]
    fn test_golden_rsnap_with_full_items() {
        let snapshot = RsnapSnapshot {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            rustc_version: "1.75.0".to_string(),
            crate_graph: CrateGraph {
                root: CrateId(0),
                nodes: vec![CrateNode {
                    id: CrateId(0),
                    name: "golden_test".to_string(),
                    package_name: None,
                    version: None,
                    source_kind: None,
                    source: None,
                    source_ref: None,
                    edition: "2021".to_string(),
                    crate_type: CrateType::Library,
                    dependency_crates: vec![],
                    extern_prelude: vec!["std".to_string()],
                    features: vec![],
                    default_features: true,
                    optional: false,
                }],
            },
            items: vec![
                RsnapItem {
                    id: ItemId(0),
                    def_path: "golden_test::StructA".to_string(),
                    kind: ItemKind::Struct,
                    visibility: Visibility {
                        rank: VisibilityRank::Pub,
                        path: None,
                    },
                    attributes: vec![Attribute {
                        path: "repr".to_string(),
                        tokens: "C".to_string(),
                    }],
                    generics: Some(Generics {
                        lifetimes: vec![],
                        type_params: vec![TypeParam {
                            name: "T".to_string(),
                            bounds: vec![],
                        }],
                        const_params: vec![],
                    }),
                    where_clauses: vec![],
                },
                RsnapItem {
                    id: ItemId(1),
                    def_path: "golden_test::function_with_repr".to_string(),
                    kind: ItemKind::Function,
                    visibility: Visibility {
                        rank: VisibilityRank::Pub,
                        path: None,
                    },
                    attributes: vec![
                        Attribute {
                            path: "repr".to_string(),
                            tokens: "C".to_string(),
                        },
                        Attribute {
                            path: "export_name".to_string(),
                            tokens: "\"internal_name\"".to_string(),
                        },
                    ],
                    generics: None,
                    where_clauses: vec![],
                },
            ],
            exports: vec![RsnapExport {
                symbol: "internal_name".to_string(),
                item_id: ItemId(1),
                abi: "C".to_string(),
                linkage: Linkage::External,
            }],
            source_files: vec![SourceFile {
                path: "src/lib.rs".to_string(),
                content_hash: "def456".to_string(),
            }],
        };

        let json = serde_json::to_string_pretty(&snapshot).unwrap();
        let deserialized: RsnapSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.items.len(), 2);
        assert_eq!(deserialized.exports[0].symbol, "internal_name");
        assert_eq!(deserialized.exports[0].abi, "C");
    }

    #[test]
    fn test_golden_rdep_with_all_node_kinds() {
        let graph = RdepGraph {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            nodes: vec![
                DepNode {
                    id: DepNodeId(0),
                    kind: DepNodeKind::Source,
                    fingerprint: "s0".to_string(),
                    stable_id: "src0".to_string(),
                },
                DepNode {
                    id: DepNodeId(1),
                    kind: DepNodeKind::Item,
                    fingerprint: "s1".to_string(),
                    stable_id: "item0".to_string(),
                },
                DepNode {
                    id: DepNodeId(2),
                    kind: DepNodeKind::Type,
                    fingerprint: "s2".to_string(),
                    stable_id: "type0".to_string(),
                },
                DepNode {
                    id: DepNodeId(3),
                    kind: DepNodeKind::Layout,
                    fingerprint: "s3".to_string(),
                    stable_id: "layout0".to_string(),
                },
                DepNode {
                    id: DepNodeId(4),
                    kind: DepNodeKind::MirBody,
                    fingerprint: "s4".to_string(),
                    stable_id: "mir0".to_string(),
                },
                DepNode {
                    id: DepNodeId(5),
                    kind: DepNodeKind::GenericInstantiation,
                    fingerprint: "s5".to_string(),
                    stable_id: "gen0".to_string(),
                },
                DepNode {
                    id: DepNodeId(6),
                    kind: DepNodeKind::ConstEval,
                    fingerprint: "s6".to_string(),
                    stable_id: "const0".to_string(),
                },
                DepNode {
                    id: DepNodeId(7),
                    kind: DepNodeKind::Export,
                    fingerprint: "s7".to_string(),
                    stable_id: "export0".to_string(),
                },
                DepNode {
                    id: DepNodeId(8),
                    kind: DepNodeKind::Object,
                    fingerprint: "s8".to_string(),
                    stable_id: "obj0".to_string(),
                },
                DepNode {
                    id: DepNodeId(9),
                    kind: DepNodeKind::Wrapper,
                    fingerprint: "s9".to_string(),
                    stable_id: "wrap0".to_string(),
                },
                DepNode {
                    id: DepNodeId(10),
                    kind: DepNodeKind::Proof,
                    fingerprint: "s10".to_string(),
                    stable_id: "proof0".to_string(),
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
                DepEdge {
                    from: DepNodeId(2),
                    to: DepNodeId(3),
                    kind: DepEdgeKind::Requires,
                },
                DepEdge {
                    from: DepNodeId(3),
                    to: DepNodeId(4),
                    kind: DepEdgeKind::Monomorphizes,
                },
                DepEdge {
                    from: DepNodeId(4),
                    to: DepNodeId(5),
                    kind: DepEdgeKind::Instantiates,
                },
            ],
        };

        let bytes = serde_json::to_vec(&graph).unwrap();
        let deserialized: RdepGraph = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(deserialized.nodes.len(), 11);
        assert_eq!(deserialized.edges.len(), 5);
    }

    #[test]
    fn test_golden_rmirpack_with_struct_layout() {
        let pack = RmirPack {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            types: vec![
                TypeDef::Adt {
                    name: "MyStruct".to_string(),
                    variants: vec![AdtVariant {
                        name: "MyStruct".to_string(),
                        fields: vec![TypeRef(1), TypeRef(2)],
                        discriminant: None,
                    }],
                    repr: AdtRepr {
                        kind: AdtReprKind::C,
                        pack: None,
                    },
                },
                TypeDef::Primitive(PrimitiveType::I32),
                TypeDef::Primitive(PrimitiveType::F64),
            ],
            layouts: vec![LayoutDef {
                ty: TypeRef(0),
                size: 16,
                align: 8,
                kind: LayoutKind::Struct {
                    fields: vec![
                        FieldLayout {
                            field_idx: 0,
                            offset: 0,
                            ty: TypeRef(1),
                        },
                        FieldLayout {
                            field_idx: 1,
                            offset: 8,
                            ty: TypeRef(2),
                        },
                    ],
                },
            }],
            bodies: vec![],
            constants: vec![],
        };

        let bytes = serde_json::to_vec(&pack).unwrap();
        let deserialized: RmirPack = serde_json::from_slice(&bytes).unwrap();

        if let TypeDef::Adt {
            name,
            variants,
            repr: _,
        } = &deserialized.types[0]
        {
            assert_eq!(name, "MyStruct");
            assert_eq!(variants.len(), 1);
            assert_eq!(variants[0].fields.len(), 2);
        } else {
            panic!("Expected Adt type");
        }
    }

    #[test]
    fn test_golden_rchproof_with_all_proof_types() {
        let proof = RchProof {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            layout_proofs: vec![LayoutProof {
                item_id: ItemId(1),
                proof_hash: "layout_hash".to_string(),
                obligations: vec![LayoutObligation {
                    claim: "struct_layout_matches".to_string(),
                    evidence: "offset_assert!(field_a, 0)".to_string(),
                }],
            }],
            abi_proofs: vec![AbiProof {
                symbol: "my_extern_fn".to_string(),
                proof_hash: "abi_hash".to_string(),
                obligations: vec![AbiObligation {
                    claim: "abi_is_c".to_string(),
                    evidence: "extern \"C\"".to_string(),
                }],
            }],
            ownership_proofs: vec![OwnershipProof {
                item_id: ItemId(2),
                proof_hash: "own_hash".to_string(),
                obligations: vec![OwnershipObligation {
                    claim: "no_drop_needed".to_string(),
                    evidence: "Copy trait implemented".to_string(),
                }],
            }],
            panic_proofs: vec![PanicProof {
                item_id: ItemId(3),
                policy: PanicPolicy::Abort,
                proof_hash: "panic_hash".to_string(),
                obligations: vec![PanicObligation {
                    claim: "never_panics".to_string(),
                    evidence: "#[track_caller]".to_string(),
                }],
            }],
            result_proofs: vec![ResultProof {
                symbol: "fallible_fn".to_string(),
                proof_hash: "result_hash".to_string(),
                obligations: vec![ResultObligation {
                    claim: "returns_result".to_string(),
                    evidence: "-> Result<(), Error>".to_string(),
                }],
            }],
            unsafe_proofs: vec![UnsafeProof {
                item_id: ItemId(4),
                proof_hash: "unsafe_hash".to_string(),
                obligations: vec![UnsafeObligation {
                    span: "src/lib.rs:10".to_string(),
                    claim: "unsafe_precondition".to_string(),
                    evidence: "ptr::read() called".to_string(),
                }],
            }],
            wrapper_proofs: vec![WrapperProof {
                wrapper_id: "wrap_abc123".to_string(),
                proof_hash: "wrapper_hash".to_string(),
                obligations: vec![WrapperObligation {
                    claim: "wrapper_valid".to_string(),
                    evidence: "abi_matches".to_string(),
                }],
            }],
            cache_proofs: vec![CacheProof {
                cache_key: "key_xyz".to_string(),
                proof_hash: "cache_hash".to_string(),
                obligations: vec![CacheObligation {
                    claim: "cache_reusable".to_string(),
                    evidence: "fingerprint matches".to_string(),
                }],
            }],
        };

        let bytes = serde_json::to_vec(&proof).unwrap();
        let deserialized: RchProof = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(deserialized.layout_proofs.len(), 1);
        assert_eq!(deserialized.abi_proofs.len(), 1);
        assert_eq!(deserialized.ownership_proofs.len(), 1);
        assert_eq!(deserialized.panic_proofs.len(), 1);
        assert_eq!(deserialized.result_proofs.len(), 1);
        assert_eq!(deserialized.unsafe_proofs.len(), 1);
        assert_eq!(deserialized.wrapper_proofs.len(), 1);
        assert_eq!(deserialized.cache_proofs.len(), 1);
    }

    #[test]
    fn test_checksum_deterministic_across_serializations() {
        let snapshot1 = RsnapSnapshot {
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

        let snapshot2 = RsnapSnapshot {
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

        // Same content should produce same checksum
        let hash1 = snapshot1.compute_checksum();
        let hash2 = snapshot2.compute_checksum();
        assert_eq!(hash1, hash2);

        // Different content should produce different checksum
        let mut snapshot3 = snapshot2.clone();
        snapshot3.rustc_version = "1.76.0".to_string();
        let hash3 = snapshot3.compute_checksum();
        assert_ne!(hash2, hash3);
    }
}
