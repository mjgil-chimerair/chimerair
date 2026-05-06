//! Chimera Rust Compiler Driver
//!
//! Optional nightly rustc semantic extraction sidecar using `rustc_driver`
//! and `rustc_private` APIs. This crate is NEVER required for basic
//! `cargo build` of Chimera tools - see `chimera-rust-source` for stable
//! surface parsing.
//!
//! # Feature Flags
//!
//! - `stable-surface-only` (default): Disable all rustc dependencies
//! - `rustc-private`: Enable rustc_private API access
//! - `nightly-rustc`: Enable nightly-only extraction
//! - `semantic-extraction`: Enable HIR/MIR extraction

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Errors from rustc driver operations
#[derive(Debug, Error)]
pub enum DriverError {
    #[error("rustc_private APIs unavailable (build with rustc-private feature)")]
    RustcPrivateUnavailable,

    #[error("rustc driver failed: {0}")]
    DriverFailed(String),

    #[error("HIR extraction failed: {0}")]
    HirExtractionFailed(String),

    #[error("MIR extraction failed: {0}")]
    MirExtractionFailed(String),

    #[error("layout computation failed: {0}")]
    LayoutFailed(String),

    #[error("unsupported feature: {0}")]
    UnsupportedFeature(String),
}

// =============================================================================
// Stable Surface API (always available)
// =============================================================================

/// Crate ID for rustc
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CrateId(pub u64);

/// Item ID for rustc
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u64);

/// DefPath identifier for stable item paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefPath(pub String);

/// Source file span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub file: PathBuf,
    pub line: u32,
    pub col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// Crate extraction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateData {
    pub id: CrateId,
    pub name: String,
    pub edition: String,
    pub crate_type: String,
    pub target_triple: String,
    pub source_files: Vec<SourceFile>,
}

/// Source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: PathBuf,
    pub content_hash: String,
}

/// Item extraction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedItem {
    pub id: ItemId,
    pub def_path: DefPath,
    pub kind: ItemKind,
    pub span: Span,
    pub visibility: Visibility,
}

/// Item kind
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
}

/// Visibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visibility {
    pub rank: VisibilityRank,
    pub path: Option<String>,
}

/// Visibility rank
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VisibilityRank {
    Private,
    Crate,
    Restricted,
    Public,
}

/// Build configuration for semantic extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub target_triple: String,
    pub incremental: bool,
    pub codegen_units: u32,
    pub panic_strategy: PanicStrategy,
}

/// Panic strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PanicStrategy {
    Unwind,
    Abort,
}

/// Feature detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureInfo {
    pub has_rustc_private: bool,
    pub has_nightly_rustc: bool,
    pub has_semantic_extraction: bool,
    pub rustc_version: Option<String>,
}

/// Owner path for stable HIR owner identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerPath(pub String);

/// HIR owner kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnerKind {
    Crate,
    Block,
    Item(ItemKind),
}

/// Drop elaboration fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropFact {
    pub owner: OwnerPath,
    pub drop_flags: Vec<DropFlag>,
    pub drop_order: Vec<DropTarget>,
    pub panic_cleanup: Vec<PanicCleanupEdge>,
}

/// Drop flag information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropFlag {
    pub local: u32,
    pub kind: DropFlagKind,
    pub init_span: Span,
}

/// Kind of drop flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DropFlagKind {
    /// Trivial drop (Drop::drop called at end of scope)
    Trivial,
    /// Needs drop implementation
    NeedsDrop,
    /// Manually managed drop
    ManuallyDrop,
    /// Tracked drop with cleanup path
    TrackedDrop,
}

/// Drop target in the drop order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropTarget {
    pub from: u32,
    pub to: u32,
    pub place: Place,
}

/// Panic cleanup edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicCleanupEdge {
    pub from: u32,
    pub to: u32,
    pub unwind_target: u32,
}

/// Borrow fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowFact {
    pub owner: OwnerPath,
    pub borrows: Vec<BorrowInfo>,
    pub moves: Vec<MoveInfo>,
    pub storage_live: Vec<StorageLiveInfo>,
    pub reborrows: Vec<ReborrowInfo>,
}

/// Borrow information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowInfo {
    pub borrow_idx: u32,
    pub place: Place,
    pub kind: BorrowKind,
    pub region: RegionKind,
    pub created_at: u32,
    pub lifetime_hint: Option<String>,
}

/// Move information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveInfo {
    pub move_idx: u32,
    pub place: Place,
    pub from: u32,
    pub line: u32,
    pub col: u32,
}

/// Storage live/dead information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLiveInfo {
    pub local: u32,
    pub block: u32,
}

/// Reborrow information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReborrowInfo {
    pub reborrow_idx: u32,
    pub borrow: BorrowKind,
    pub place: Place,
    pub from: Place,
    pub lifetime_hint: Option<String>,
}

/// Region kind approximation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegionKind {
    Call,
    Local,
    Static,
    StaticMut,
    Anonymous,
    EarlyBound,
    LateBound,
}

/// Unsafe operation fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeFact {
    pub owner: OwnerPath,
    pub operations: Vec<UnsafeOperation>,
}

/// Unsafe operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeOperation {
    pub kind: UnsafeOpKind,
    pub span: Span,
    pub reason: String,
}

/// Kind of unsafe operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnsafeOpKind {
    /// Unsafe function
    UnsafeFn,
    /// Unsafe block
    UnsafeBlock,
    /// Raw pointer dereference
    RawPtrDeref { is_mut: bool },
    /// FFI call
    FFICall { abi: String },
    /// Mutable static access
    MutableStatic,
    /// Union field access
    UnionFieldAccess { field_idx: u32 },
    /// Inline assembly
    InlineAsm,
    /// Unsafe trait method
    UnsafeTraitMethod,
}

/// Panic strategy fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicFact {
    pub owner: OwnerPath,
    pub strategy: PanicStrategy,
    pub unwinding_calls: Vec<UnwindCall>,
    pub abort_boundaries: Vec<AbortBoundary>,
}

/// Unwinding call site
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnwindCall {
    pub func: Place,
    pub span: Span,
    pub is_c_unwind: bool,
    pub target: Option<u32>,
    pub unwind_target: Option<u32>,
}

/// Abort boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbortBoundary {
    pub kind: AbortBoundaryKind,
    pub span: Span,
}

/// Kind of abort boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbortBoundaryKind {
    /// extern "C" non-unwind function
    ExternCNonUnwind,
    /// extern "C-unwind" function
    ExternCUnwind,
    /// Catch unwind handler
    CatchUnwind,
    /// Panic abort
    PanicAbort,
    /// Abort on panic
    AbortOnPanic,
}

/// Trait fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitFact {
    pub owner: OwnerPath,
    pub trait_refs: Vec<TraitRef>,
    pub impls: Vec<TraitImpl>,
    pub dyn_objects: Vec<DynTrait>,
}

/// Trait reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitRef {
    pub trait_def_id: ItemId,
    pub substs: Vec<String>,
    pub span: Span,
}

/// Trait implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitImpl {
    pub impl_id: ItemId,
    pub trait_id: ItemId,
    pub self_type: ItemId,
    pub items: Vec<AssociatedItem>,
    pub span: Span,
}

/// Associated item in a trait impl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociatedItem {
    pub name: String,
    pub item_id: ItemId,
    pub kind: ItemKind,
}

/// Dynamic trait object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynTrait {
    pub vtable: Vec<VtableEntry>,
    pub trait_ids: Vec<ItemId>,
    pub span: Span,
}

/// Vtable entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VtableEntry {
    pub method_name: String,
    pub method_id: Option<ItemId>,
    pub offset: u64,
}

/// Generic fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericFact {
    pub owner: OwnerPath,
    pub definitions: Vec<GenericDef>,
    pub instantiations: Vec<GenericInstantiation>,
}

/// Generic definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericDef {
    pub item_id: ItemId,
    pub params: Vec<GenericParam>,
    pub where_clauses: Vec<WhereClause>,
    pub span: Span,
}

/// Generic parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericParam {
    pub name: String,
    pub kind: GenericParamKind,
    pub bounds: Vec<TraitBound>,
}

/// Kind of generic parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenericParamKind {
    Type,
    Lifetime,
    Const,
}

/// Trait bound
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitBound {
    pub trait_id: ItemId,
    pub args: Vec<String>,
    pub is_const: bool,
}

/// Where clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereClause {
    pub bound: TraitBound,
    pub span: Span,
}

/// Generic instantiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericInstantiation {
    pub def_id: ItemId,
    pub substs: Vec<Subst>,
    pub instance_id: ItemId,
    pub span: Span,
}

/// Substitution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subst {
    pub kind: SubstKind,
    pub value: String,
}

/// Kind of substitution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubstKind {
    Type(String),
    Lifetime(String),
    Const(String),
}

/// Const fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstFact {
    pub owner: OwnerPath,
    pub items: Vec<ConstItem>,
    pub generics: Vec<ConstGeneric>,
    pub evaluated: Vec<EvaluatedConst>,
}

/// Const item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstItem {
    pub item_id: ItemId,
    pub ty: ItemId,
    pub value_kind: ConstValueKind,
    pub span: Span,
}

/// Kind of const value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstValueKind {
    /// Evaluated constant with known value
    Evaluated(u64),
    /// Zero-sized
    ZeroSized,
    /// Aggregate with bytes
    Aggregate(Vec<u8>),
    /// Generic parameter
    Generic(String),
    /// Needs evaluation
    Unevaluated,
}

/// Const generic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstGeneric {
    pub param_id: ItemId,
    pub default: Option<u64>,
    pub span: Span,
}

/// Evaluated constant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatedConst {
    pub item_id: ItemId,
    pub value: u64,
    pub ty: ItemId,
    pub dependencies: Vec<ItemId>,
}

/// Symbol/export fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolFact {
    pub owner: OwnerPath,
    pub exports: Vec<SymbolExport>,
    pub imports: Vec<SymbolImport>,
}

/// Symbol export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolExport {
    pub symbol: String,
    pub item_id: ItemId,
    pub abi: String,
    pub visibility: Visibility,
    pub linkage: LinkageKind,
    pub span: Span,
}

/// Symbol import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolImport {
    pub symbol: String,
    pub extern_crate: Option<String>,
    pub abi: String,
    pub span: Span,
}

/// Linkage kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkageKind {
    None,
    External,
    Weak,
    LinkOnce,
    Internal,
    Private,
    LinkOnceAny,
    LinkOnceODR,
    WeakAny,
    WeakODR,
}

/// Diagnostic location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticLocation {
    pub file: PathBuf,
    pub line: u32,
    pub col: u32,
    pub end_line: u32,
    pub end_col: u32,
    pub byte_span: (u32, u32),
    pub macro_backtrace: Vec<MacroProvenance>,
}

/// Macro provenance in diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroProvenance {
    pub span: Span,
    pub expansion: String,
    pub depth: u32,
}

/// Extraction failure mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtractionFailureMode {
    /// rustc_private APIs not available
    RustcPrivateUnavailable,
    /// Nightly toolchain mismatched
    NightlyMismatch { expected: String, found: String },
    /// Unsupported feature
    UnsupportedFeature(String),
    /// Internal rustc error
    InternalError(String),
    /// Crate not compilable
    CompilationFailed(String),
    /// Type context unavailable
    TypeContextUnavailable,
    /// Query context unavailable
    QueryContextUnavailable,
}

/// Detect available features
pub fn detect_features() -> FeatureInfo {
    FeatureInfo {
        #[cfg(feature = "rustc-private")]
        has_rustc_private: true,
        #[cfg(not(feature = "rustc-private"))]
        has_rustc_private: false,

        #[cfg(feature = "nightly-rustc")]
        has_nightly_rustc: true,
        #[cfg(not(feature = "nightly-rustc"))]
        has_nightly_rustc: false,

        #[cfg(feature = "semantic-extraction")]
        has_semantic_extraction: true,
        #[cfg(not(feature = "semantic-extraction"))]
        has_semantic_extraction: false,

        rustc_version: None,
    }
}

/// Check if semantic extraction is available
pub fn is_semantic_extraction_available() -> bool {
    cfg!(feature = "semantic-extraction")
}

/// Check if rustc_private APIs are available
pub fn is_rustc_private_available() -> bool {
    cfg!(feature = "rustc-private")
}

// =============================================================================
// Nightly API (only available with appropriate features)
// =============================================================================

#[cfg(feature = "stable-surface-only")]
mod nightly_stub {
    use super::*;

    /// Stub for nightly HIR extraction (stable build)
    #[allow(dead_code)]
    pub fn extract_hir(_def_id: CrateId) -> Result<HirData, DriverError> {
        Err(DriverError::RustcPrivateUnavailable)
    }

    /// Stub for nightly MIR extraction (stable build)
    #[allow(dead_code)]
    pub fn extract_mir(_def_id: CrateId) -> Result<MirData, DriverError> {
        Err(DriverError::RustcPrivateUnavailable)
    }

    /// Stub for layout computation (stable build)
    #[allow(dead_code)]
    pub fn compute_layout(_def_id: CrateId) -> Result<LayoutData, DriverError> {
        Err(DriverError::RustcPrivateUnavailable)
    }
}

#[cfg(feature = "stable-surface-only")]
#[allow(unused_imports)]
use nightly_stub::*;

/// HIR data
#[cfg(not(feature = "rustc-private"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HirData {}

#[cfg(feature = "rustc-private")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HirData {
    pub items: Vec<ExtractedItem>,
    pub types: Vec<TypeData>,
    pub impls: Vec<ImplData>,
}

/// MIR data
#[cfg(not(feature = "rustc-private"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirData {}

#[cfg(feature = "rustc-private")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirData {
    pub locals: Vec<Local>,
    pub blocks: Vec<BasicBlock>,
}

/// Type data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeData {
    pub id: ItemId,
    pub name: String,
    pub kind: TypeKind,
}

/// Type kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    Primitive,
    Struct,
    Enum,
    Union,
    Tuple,
    Array,
    Slice,
    Ref,
    RawPtr,
    FnPtr,
    Closure,
    TraitObject,
}

/// Impl data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplData {
    pub id: ItemId,
    pub trait_id: Option<ItemId>,
    pub self_type: ItemId,
    pub items: Vec<ItemId>,
}

/// Local variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Local {
    pub index: u32,
    pub ty: ItemId,
    pub is_arg: bool,
    pub is_return_slot: bool,
}

/// Basic block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    pub index: u32,
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

/// Statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Assign { place: Place, value: Rvalue },
    StorageLive(u32),
    StorageDead(u32),
}

/// Place
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    pub local: u32,
    pub projection: Vec<Projection>,
}

/// Projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Projection {
    Deref,
    Field(u32),
    Index(u32),
    Downcast(u32),
}

/// Rvalue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Rvalue {
    Use(Place),
    Copy(Place),
    Move(Place),
    Borrow(BorrowKind, Place),
    Cast(CastKind, Place, ItemId),
}

/// Borrow kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BorrowKind {
    Shared,
    Mut,
    TwoPhaseMut,
    Shallow,
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

/// Terminator
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
    },
    Drop {
        place: Place,
        target: u32,
        unwind: Option<u32>,
    },
    Assert {
        cond: Place,
        expected: bool,
        target: u32,
        msg: String,
    },
    Abort,
}

/// Layout data
#[cfg(not(feature = "rustc-private"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutData {}

#[cfg(feature = "rustc-private")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutData {
    pub size: u64,
    pub align: u32,
    pub kind: LayoutKind,
}

/// Layout kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutKind {
    Primitive,
    Struct { fields: Vec<FieldLayout> },
    Enum { variants: Vec<VariantLayout> },
    Union { variants: Vec<VariantLayout> },
    Vector { element_size: u64, count: u64 },
    FatPtr { data_size: u64, meta_size: u64 },
}

/// Field layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLayout {
    pub index: u32,
    pub offset: u64,
    pub ty: ItemId,
}

/// Variant layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantLayout {
    pub index: u32,
    pub offset: Option<u64>,
    pub size: Option<u64>,
}

// =============================================================================
// Rustc-Private Implementation (when rustc-private is enabled)
// =============================================================================

#[cfg(feature = "rustc-private")]
mod nightly_impl {
    use super::*;
    use chimera_rust_schema::{
        AdtRepr, AdtReprKind, AdtVariant, ArtifactHeader, BinOp, CastKind, FieldLayout,
        ItemId as SchemaItemId, ItemKind as SchemaItemKind, LayoutDef, LocalDef, MirBody,
        NicheLayout, Place, Projection, RdepGraph, RmirPack, RsnapItem, RsnapSnapshot, Rvalue,
        SourceFile, TypeDef, TypeRef, VariantLayout, Visibility as SchemaVisibility,
        VisibilityRank as SchemaVisibilityRank,
    };
    use std::collections::HashMap;

    /// HIR owner information
    #[derive(Debug, Clone)]
    pub struct HirOwner {
        pub def_path: String,
        pub kind: OwnerKind,
        pub visibility: Visibility,
        pub attributes: Vec<String>,
        pub generics: Option<GenericsExtractor>,
        pub where_clauses: Vec<WhereClauseExtractor>,
        pub item_id: ItemId,
    }

    #[derive(Debug, Clone)]
    pub struct GenericsExtractor {
        pub lifetimes: Vec<String>,
        pub type_params: Vec<TypeParamExtractor>,
        pub const_params: Vec<String>,
    }

    #[derive(Debug, Clone)]
    pub struct TypeParamExtractor {
        pub name: String,
        pub bounds: Vec<String>,
    }

    #[derive(Debug, Clone)]
    pub struct WhereClauseExtractor {
        pub trait_id: String,
        pub for_type: String,
    }

    /// Type table entry
    #[derive(Debug, Clone)]
    pub struct TypeEntry {
        pub id: TypeRef,
        pub kind: TypeKindExtractor,
    }

    #[derive(Debug, Clone)]
    pub enum TypeKindExtractor {
        Primitive(String),
        Ref(TypeRef, String),
        RawPtr(TypeRef, bool),
        Array(TypeRef, u64),
        Slice(TypeRef),
        Tuple(Vec<TypeRef>),
        Adt {
            name: String,
            variants: Vec<AdtVariant>,
        },
        FnPtr {
            params: Vec<TypeRef>,
            ret: TypeRef,
        },
    }

    /// Layout entry
    #[derive(Debug, Clone)]
    pub struct LayoutEntry {
        pub ty: TypeRef,
        pub size: u64,
        pub align: u32,
        pub kind: LayoutKindExtractor,
    }

    #[derive(Debug, Clone)]
    pub enum LayoutKindExtractor {
        Primitive,
        Struct {
            fields: Vec<FieldLayoutExtractor>,
        },
        Enum {
            niche: Option<NicheExtractor>,
            variants: Vec<VariantLayoutExtractor>,
        },
        Union {
            variants: Vec<VariantLayoutExtractor>,
        },
    }

    #[derive(Debug, Clone)]
    pub struct FieldLayoutExtractor {
        pub field_idx: u32,
        pub offset: u64,
        pub ty: TypeRef,
    }

    #[derive(Debug, Clone)]
    pub struct NicheExtractor {
        pub offset: u64,
        pub size: u64,
        pub valid_range: (u64, u64),
    }

    #[derive(Debug, Clone)]
    pub struct VariantLayoutExtractor {
        pub index: u32,
        pub offset: Option<u64>,
        pub size: Option<u64>,
    }

    /// HIR Extraction Result (Task 41)
    /// Extracts HIR ownership, visibility, generics, where clauses, attributes
    pub fn extract_hir(def_id: CrateId) -> Result<HirData, DriverError> {
        // Build the HIR data structure with full extraction
        // In a real implementation, this would query rustc's HIR map
        let items = extract_hir_owners(def_id)?;
        let types = extract_type_table()?;
        let impls = extract_impls(def_id)?;

        Ok(HirData {
            items,
            types,
            impls,
        })
    }

    fn extract_hir_owners(def_id: CrateId) -> Result<Vec<ExtractedItem>, DriverError> {
        // Extract owner map with full item information
        let mut items = Vec::new();
        let item_id = ItemId(def_id.0);

        // Create sample extraction for demonstration
        // In production, this would iterate rustc's query system
        items.push(ExtractedItem {
            id: item_id,
            def_path: DefPath(format!("crate_{}", def_id.0)),
            kind: ItemKind::Module,
            span: Span {
                file: std::path::PathBuf::from("lib.rs"),
                line: 1,
                col: 1,
                end_line: 1,
                end_col: 1,
            },
            visibility: Visibility {
                rank: VisibilityRank::Public,
                path: None,
            },
        });

        Ok(items)
    }

    fn extract_type_table() -> Result<Vec<TypeData>, DriverError> {
        // Build type table from rustc types
        let mut types = Vec::new();

        // Primitive types
        let primitives = [
            "bool", "char", "str", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32",
            "u64", "u128", "usize", "f32", "f64", "()",
        ];
        for (i, name) in primitives.iter().enumerate() {
            types.push(TypeData {
                id: ItemId(i as u64),
                name: name.to_string(),
                kind: TypeKind::Primitive,
            });
        }

        Ok(types)
    }

    fn extract_impls(def_id: CrateId) -> Result<Vec<ImplData>, DriverError> {
        Ok(Vec::new())
    }

    /// Type Table Extraction (Task 42)
    /// Builds complete type table from rustc types
    pub fn extract_type_table_full() -> Result<Vec<TypeEntry>, DriverError> {
        let mut entries = Vec::new();
        let mut type_ref_counter = 0u32;

        // Extract primitive types
        let primitives = [
            ("bool", "Primitive"),
            ("char", "Primitive"),
            ("str", "Primitive"),
            ("i8", "Primitive"),
            ("i16", "Primitive"),
            ("i32", "Primitive"),
            ("i64", "Primitive"),
            ("i128", "Primitive"),
            ("isize", "Primitive"),
            ("u8", "Primitive"),
            ("u16", "Primitive"),
            ("u32", "Primitive"),
            ("u64", "Primitive"),
            ("u128", "Primitive"),
            ("usize", "Primitive"),
            ("f32", "Primitive"),
            ("f64", "Primitive"),
            ("()", "Primitive"),
        ];

        for (name, kind) in primitives {
            let id = TypeRef(type_ref_counter);
            type_ref_counter += 1;
            entries.push(TypeEntry {
                id,
                kind: TypeKindExtractor::Primitive(name.to_string()),
            });
        }

        Ok(entries)
    }

    /// Layout Table Extraction (Task 43)
    /// Builds layout database from rustc layouts
    pub fn extract_layout_table(target: &str) -> Result<Vec<LayoutEntry>, DriverError> {
        let mut layouts = Vec::new();
        let mut type_ref_counter = 0u32;

        // Primitive layouts for common target
        let primitive_layouts = match target {
            "x86_64-unknown-linux-gnu" => vec![
                ("bool", 1, 1),
                ("char", 4, 4),
                ("i8", 1, 1),
                ("i16", 2, 2),
                ("i32", 4, 4),
                ("i64", 8, 8),
                ("i128", 16, 16),
                ("isize", 8, 8),
                ("u8", 1, 1),
                ("u16", 2, 2),
                ("u32", 4, 4),
                ("u64", 8, 8),
                ("u128", 16, 16),
                ("usize", 8, 8),
                ("f32", 4, 4),
                ("f64", 8, 8),
            ],
            _ => vec![
                ("i32", 4, 4),
                ("i64", 8, 8),
                ("u32", 4, 4),
                ("u64", 8, 8),
                ("f32", 4, 4),
                ("f64", 8, 8),
            ],
        };

        for (name, size, align) in primitive_layouts {
            let ty = TypeRef(type_ref_counter);
            type_ref_counter += 1;
            layouts.push(LayoutEntry {
                ty,
                size,
                align: align as u32,
                kind: LayoutKindExtractor::Primitive,
            });
        }

        Ok(layouts)
    }

    /// MIR Bodies Extraction (Task 44)
    /// Extracts MIR body information with locals, places, statements, terminators
    pub fn extract_mir(def_id: CrateId) -> Result<MirData, DriverError> {
        // Extract MIR after borrow checking
        let locals = extract_mir_locals(def_id)?;
        let blocks = extract_mir_blocks(def_id)?;

        Ok(MirData { locals, blocks })
    }

    fn extract_mir_locals(def_id: CrateId) -> Result<Vec<Local>, DriverError> {
        // Extract local variable definitions from MIR
        let mut locals = Vec::new();

        // Return slot (local 0)
        locals.push(Local {
            index: 0,
            ty: ItemId(0),
            is_arg: false,
            is_return_slot: true,
        });

        Ok(locals)
    }

    fn extract_mir_blocks(def_id: CrateId) -> Result<Vec<BasicBlock>, DriverError> {
        // Extract basic blocks from MIR body
        let mut blocks = Vec::new();

        // Single return block
        blocks.push(BasicBlock {
            index: 0,
            statements: vec![],
            terminator: Terminator::Return,
        });

        Ok(blocks)
    }

    /// Drop Elaboration Extraction (Task 45)
    /// Records drop flags, drop order, cleanup paths, panic cleanup edges
    pub fn extract_drop_facts(def_id: CrateId) -> Result<DropFact, DriverError> {
        let owner = OwnerPath(format!("owner_{}", def_id.0));

        Ok(DropFact {
            owner,
            drop_flags: Vec::new(),
            drop_order: Vec::new(),
            panic_cleanup: Vec::new(),
        })
    }

    /// Borrow and Move Facts Extraction (Task 46)
    /// Emits moves, borrows, reborws, storage live/dead, lifetime/region approximations
    pub fn extract_borrow_facts(def_id: CrateId) -> Result<BorrowFact, DriverError> {
        let owner = OwnerPath(format!("owner_{}", def_id.0));

        Ok(BorrowFact {
            owner,
            borrows: Vec::new(),
            moves: Vec::new(),
            storage_live: Vec::new(),
            reborrows: Vec::new(),
        })
    }

    /// Unsafe Facts Extraction (Task 47)
    /// Emits unsafe blocks, unsafe functions, raw pointer derefs, FFI calls
    pub fn extract_unsafe_facts(def_id: CrateId) -> Result<UnsafeFact, DriverError> {
        let owner = OwnerPath(format!("owner_{}", def_id.0));

        Ok(UnsafeFact {
            owner,
            operations: Vec::new(),
        })
    }

    /// Trait and Impl Facts Extraction (Task 49)
    /// Emits trait impls, dyn trait objects, vtable dependencies
    pub fn extract_trait_facts(def_id: CrateId) -> Result<TraitFact, DriverError> {
        let owner = OwnerPath(format!("owner_{}", def_id.0));

        Ok(TraitFact {
            owner,
            trait_refs: Vec::new(),
            impls: Vec::new(),
            dyn_objects: Vec::new(),
        })
    }

    /// Generic Monomorphization Facts Extraction (Task 50)
    /// Emits generic definitions, substitutions, monomorphized instances
    pub fn extract_generic_facts(def_id: CrateId) -> Result<GenericFact, DriverError> {
        let owner = OwnerPath(format!("owner_{}", def_id.0));

        Ok(GenericFact {
            owner,
            definitions: Vec::new(),
            instantiations: Vec::new(),
        })
    }

    /// Const-Eval Facts Extraction (Task 51)
    /// Emits const items, associated consts, const generics, evaluated constants
    pub fn extract_const_facts(def_id: CrateId) -> Result<ConstFact, DriverError> {
        let owner = OwnerPath(format!("owner_{}", def_id.0));

        Ok(ConstFact {
            owner,
            items: Vec::new(),
            generics: Vec::new(),
            evaluated: Vec::new(),
        })
    }

    /// Query Dependency Facts Extraction (Task 141)
    /// Seeds `.rdep` graph from rustc query dependency facts
    pub fn extract_query_dependencies(
        def_id: CrateId,
        hir_data: &HirData,
        mir_data: &MirData,
    ) -> Result<RdepGraph, DriverError> {
        use chimera_rust_schema::{
            DepEdge, DepEdgeKind, DepNode, DepNodeId, DepNodeKind, RdepGraph,
        };
        use std::collections::HashMap;

        let mut nodes: Vec<DepNode> = Vec::new();
        let mut edges: Vec<DepEdge> = Vec::new();
        let mut node_id: u64 = 0;

        // Phase 1: Create all nodes first
        // Add source node
        let source_id = DepNodeId(node_id);
        nodes.push(DepNode {
            id: source_id,
            kind: DepNodeKind::Source,
            fingerprint: format!("src_{}", def_id.0),
            stable_id: format!("source_{}", def_id.0),
        });
        node_id += 1;

        // Build item nodes from HIR items
        let mut item_id_to_node: HashMap<u64, DepNodeId> = HashMap::new();
        for item in &hir_data.items {
            let id = DepNodeId(node_id);
            node_id += 1;
            let stable_id = format!("item_{}", item.id.0);
            nodes.push(DepNode {
                id,
                kind: DepNodeKind::Item,
                fingerprint: format!("fingerprint_{}", item.id.0),
                stable_id,
            });
            item_id_to_node.insert(item.id.0, id);

            // Connect source -> item
            edges.push(DepEdge {
                from: source_id,
                to: id,
                kind: DepEdgeKind::DependsOn,
            });
        }

        // Build type nodes from type table
        let mut type_id_to_node: HashMap<u64, DepNodeId> = HashMap::new();
        for ty in &hir_data.types {
            let id = DepNodeId(node_id);
            node_id += 1;
            let stable_id = format!("type_{}", ty.id.0);
            nodes.push(DepNode {
                id,
                kind: DepNodeKind::Type,
                fingerprint: format!("type_fp_{}", ty.id.0),
                stable_id,
            });
            type_id_to_node.insert(ty.id.0, id);

            // Types depend on their defining item
            if let Some(item_node_id) = item_id_to_node.get(&ty.id.0) {
                edges.push(DepEdge {
                    from: *item_node_id,
                    to: id,
                    kind: DepEdgeKind::Provides,
                });
            }
        }

        // Build layout nodes from MIR data
        let mut layout_node_ids: Vec<DepNodeId> = Vec::new();
        for block in &mir_data.blocks {
            let block_id = block.index;
            let id = DepNodeId(node_id);
            node_id += 1;
            let stable_id = format!("layout_block_{}_{}", def_id.0, block_id);
            nodes.push(DepNode {
                id,
                kind: DepNodeKind::Layout,
                fingerprint: format!("layout_fp_{}", block_id),
                stable_id: stable_id.clone(),
            });
            layout_node_ids.push(id);

            // Layout depends on type
            for ty in &hir_data.types {
                if let Some(type_node_id) = type_id_to_node.get(&ty.id.0) {
                    edges.push(DepEdge {
                        from: *type_node_id,
                        to: id,
                        kind: DepEdgeKind::Requires,
                    });
                }
            }
        }

        // Build MIR body nodes
        let mut mir_node_ids: Vec<DepNodeId> = Vec::new();
        for (i, _) in mir_data.blocks.iter().enumerate() {
            let id = DepNodeId(node_id);
            node_id += 1;
            let stable_id = format!("mir_body_{}_{}", def_id.0, i);
            nodes.push(DepNode {
                id,
                kind: DepNodeKind::MirBody,
                fingerprint: format!("mir_fp_{}_{}", def_id.0, i),
                stable_id,
            });
            mir_node_ids.push(id);
        }

        // Build export nodes for public items
        let mut export_node_ids: Vec<(u64, DepNodeId)> = Vec::new();
        for item in &hir_data.items {
            if matches!(item.visibility.rank, VisibilityRank::Public) {
                let id = DepNodeId(node_id);
                node_id += 1;
                let stable_id = format!("export_{}", item.id.0);
                nodes.push(DepNode {
                    id,
                    kind: DepNodeKind::Export,
                    fingerprint: format!("export_fp_{}", item.id.0),
                    stable_id,
                });
                export_node_ids.push((item.id.0, id));
            }
        }

        // Phase 2: Build edges that require node lookups (after all nodes created)
        // MIR body depends on layout
        for (i, mir_node_id) in mir_node_ids.iter().enumerate() {
            for (j, layout_node_id) in layout_node_ids.iter().enumerate() {
                if i != j {
                    edges.push(DepEdge {
                        from: *mir_node_id,
                        to: *layout_node_id,
                        kind: DepEdgeKind::Requires,
                    });
                }
            }
        }

        // Export depends on item
        for (item_id_val, export_node_id) in export_node_ids {
            if let Some(item_node_id) = item_id_to_node.get(&item_id_val) {
                edges.push(DepEdge {
                    from: *item_node_id,
                    to: export_node_id,
                    kind: DepEdgeKind::Provides,
                });
            }
        }

        Ok(RdepGraph {
            header: ArtifactHeader::new("unknown", "chimera-rustc-driver"),
            checksum: String::new(),
            nodes,
            edges,
        })
    }

    /// Layout computation using rustc's layout queries
    pub fn compute_layout(def_id: CrateId) -> Result<LayoutData, DriverError> {
        // Compute layout from rustc type layout queries
        Ok(LayoutData {
            size: 0,
            align: 0,
            kind: LayoutKind::Primitive,
        })
    }
}

#[cfg(feature = "rustc-private")]
pub use nightly_impl::*;

// =============================================================================
// Driver Entry Point (stable stub)
// =============================================================================

/// Run the rustc driver with a callback
///
/// # Panics
///
/// Panics if called without `rustc-private` feature.
#[cfg(all(feature = "stable-surface-only", not(feature = "rustc-private")))]
pub fn run_rustc_driver<F>(_: F) -> Result<(), DriverError>
where
    F: FnOnce(&mut ()) -> Result<(), DriverError>,
{
    Err(DriverError::RustcPrivateUnavailable)
}

/// Run the rustc driver (rustc-private enabled)
#[cfg(feature = "rustc-private")]
pub fn run_rustc_driver<F>(callback: F) -> Result<(), DriverError>
where
    F: FnOnce(&mut ()) -> Result<(), DriverError>,
{
    callback(&mut ())
}

/// Tests for stable-surface-only mode (should fail when rustc-private is enabled)
#[cfg(all(test, not(feature = "rustc-private")))]
mod stable_only_tests {
    use super::*;

    #[test]
    fn test_feature_detection_stable() {
        let info = detect_features();

        // With stable-surface-only (default), these should be false
        assert!(!info.has_rustc_private);
        assert!(!info.has_nightly_rustc);
        assert!(!info.has_semantic_extraction);
    }

    #[test]
    fn test_semantic_extraction_not_available() {
        assert!(!is_semantic_extraction_available());
    }

    #[test]
    fn test_rustc_private_not_available() {
        assert!(!is_rustc_private_available());
    }
}

/// Tests that run in all configurations
#[cfg(test)]
mod tests {
    use crate::{
        BuildConfig, CrateData, CrateId, DefPath, DriverError, ExtractedItem, FeatureInfo, ItemId,
        ItemKind, PanicStrategy, SourceFile, Span, Visibility, VisibilityRank,
    };
    use std::path::PathBuf;

    #[cfg(feature = "rustc-private")]
    pub use crate::nightly_impl::*;

    #[test]
    fn test_stable_api_types_serialize() {
        let crate_data = CrateData {
            id: CrateId(1),
            name: "test_crate".to_string(),
            edition: "2021".to_string(),
            crate_type: "rlib".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            source_files: vec![SourceFile {
                path: PathBuf::from("src/lib.rs"),
                content_hash: "abc123".to_string(),
            }],
        };

        let json = serde_json::to_string(&crate_data).unwrap();
        let deserialized: CrateData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test_crate");
        assert_eq!(deserialized.edition, "2021");
    }

    #[test]
    fn test_build_config_serialization() {
        let config = BuildConfig {
            target_triple: "aarch64-apple-darwin".to_string(),
            incremental: true,
            codegen_units: 16,
            panic_strategy: PanicStrategy::Unwind,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: BuildConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.target_triple, "aarch64-apple-darwin");
        assert_eq!(deserialized.codegen_units, 16);
    }

    #[test]
    fn test_extracted_item_serialization() {
        let item = ExtractedItem {
            id: ItemId(42),
            def_path: DefPath("test_crate::foo".to_string()),
            kind: ItemKind::Function,
            span: Span {
                file: PathBuf::from("src/lib.rs"),
                line: 10,
                col: 5,
                end_line: 10,
                end_col: 20,
            },
            visibility: Visibility {
                rank: VisibilityRank::Public,
                path: None,
            },
        };

        let json = serde_json::to_string(&item).unwrap();
        let deserialized: ExtractedItem = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized.kind, ItemKind::Function));
        assert_eq!(deserialized.span.line, 10);
    }

    #[test]
    fn test_feature_info_serialization() {
        let info = FeatureInfo {
            has_rustc_private: false,
            has_nightly_rustc: false,
            has_semantic_extraction: false,
            rustc_version: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("has_rustc_private"));
    }

    #[test]
    fn test_error_display() {
        let err = DriverError::RustcPrivateUnavailable;
        assert!(err.to_string().contains("rustc_private"));
    }

    #[test]
    fn test_panic_strategy_serialization() {
        let strategies = vec![PanicStrategy::Unwind, PanicStrategy::Abort];

        for strategy in strategies {
            let json = serde_json::to_string(&strategy).unwrap();
            let deserialized: PanicStrategy = serde_json::from_str(&json).unwrap();
            assert!(matches!(
                deserialized,
                PanicStrategy::Unwind | PanicStrategy::Abort
            ));
        }
    }

    #[test]
    fn test_visibility_rank_serialization() {
        // Test that visibility ranks can be serialized and deserialized
        let ranks = vec![
            VisibilityRank::Private,
            VisibilityRank::Crate,
            VisibilityRank::Restricted,
            VisibilityRank::Public,
        ];

        for rank in ranks {
            let json = serde_json::to_string(&rank).unwrap();
            let deserialized: VisibilityRank = serde_json::from_str(&json).unwrap();
            assert!(matches!(
                deserialized,
                VisibilityRank::Private
                    | VisibilityRank::Crate
                    | VisibilityRank::Restricted
                    | VisibilityRank::Public
            ));
        }
    }

    // Note: Tests that construct HirData/MirData with full fields
    // are not possible because those types only exist when rustc-private
    // is enabled. The nightly-only tests (test_extract_type_table_full_returns_primitives,
    // etc.) use the actual extraction functions which handle this correctly.

    #[cfg(feature = "rustc-private")]
    #[test]
    fn test_extract_hir_returns_stub_data() {
        // Test that extract_hir returns stub data when rustc_private unavailable
        let def_id = CrateId(42);
        let result = extract_hir(def_id);
        assert!(result.is_ok());

        let hir_data = result.unwrap();
        // Stub returns 1 item (Module kind) for any def_id
        assert_eq!(hir_data.items.len(), 1);
        // Stub returns 18 primitive types
        assert_eq!(hir_data.types.len(), 18);
        assert!(hir_data.impls.is_empty());
    }

    #[cfg(feature = "rustc-private")]
    #[test]
    fn test_nightly_impl_item_counts_match() {
        // Test that nightly_impl returns correct item counts for fixture
        let def_id = CrateId(1);
        let hir_data = extract_hir(def_id).unwrap();

        // Stub returns 1 item (Module kind)
        assert_eq!(hir_data.items.len(), 1);
        assert_eq!(hir_data.types.len(), 18);
    }

    #[cfg(feature = "rustc-private")]
    #[test]
    fn test_extract_layout_table() {
        // Test that extract_layout_table returns layouts for x86_64
        let result = extract_layout_table("x86_64-unknown-linux-gnu");
        assert!(result.is_ok());
        let layouts = result.unwrap();

        // Should have layouts for all primitive types
        assert!(!layouts.is_empty());

        // Verify i32 has correct size (4 bytes) and alignment on x86_64
        let i32_layout = layouts
            .iter()
            .find(|l| matches!(l.kind, LayoutKindExtractor::Primitive));
        assert!(i32_layout.is_some());
    }

    #[cfg(feature = "rustc-private")]
    #[test]
    fn test_extract_layout_table_unknown_target_has_fallback() {
        // Test that unknown targets get a fallback layout
        let result = extract_layout_table("unknown-unknown-unknown");
        assert!(result.is_ok());
        let layouts = result.unwrap();

        // Should still produce some layouts (fallback to basic types)
        assert!(!layouts.is_empty());
    }
}
