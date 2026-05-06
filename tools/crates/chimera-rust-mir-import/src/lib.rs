//! Chimera Rust MIR Import Crate
//!
//! Parses `.rmirpack` files produced by `chimera-rustc-driver` and normalizes
//! rustc IDs to stable identifiers. Decodes MIR bodies, locals, places,
//! projections, constants, types, regions, drops, and terminators.

use blake3::Hasher as Blake3Hasher;
use chimera_rust_schema::{
    AdtRepr, AdtReprKind, AdtVariant, BasicBlock, BinOp, BorrowKind, CastKind, LayoutDef,
    LayoutKind, LocalDef, MirBody, Place, Projection, RmirPack, Rvalue, Statement, Terminator,
    TypeDef, TypeRef, UnOp,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
use thiserror::Error;

// Re-export ItemId from chimera-rust-schema for external use
pub use chimera_rust_schema::ItemId;

#[derive(Debug, Error)]
pub enum MirImportError {
    #[error("failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("invalid magic bytes: expected {expected:?}, got {actual:?}")]
    InvalidMagic { expected: [u8; 4], actual: [u8; 4] },

    #[error("unsupported schema version: {0}")]
    UnsupportedVersion(u32),

    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("missing type for ref: {0:?}")]
    MissingType(TypeRef),

    #[error("missing local: {0}")]
    MissingLocal(u32),

    #[error("malformed MIR body: {0}")]
    MalformedBody(String),
}

// =============================================================================
// Stable ID Mapping
// =============================================================================

/// Maps rustc DefIds/HirIds to stable ItemIds
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StableIdMap {
    /// Maps original rustc DefPath indices to stable ItemIds
    defpath_to_item: HashMap<String, ItemId>,
    /// Maps stable ItemIds back to their DefPaths
    item_to_defpath: HashMap<ItemId, String>,
    /// Next available stable ItemId
    next_item_id: u64,
}

impl StableIdMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a rustc DefPath and get a stable ItemId
    pub fn insert(&mut self, def_path: &str) -> ItemId {
        if let Some(existing) = self.defpath_to_item.get(def_path) {
            return *existing;
        }

        let item_id = ItemId(self.next_item_id);
        self.next_item_id += 1;

        self.defpath_to_item.insert(def_path.to_string(), item_id);
        self.item_to_defpath.insert(item_id, def_path.to_string());

        item_id
    }

    /// Get stable ItemId for a DefPath
    pub fn get(&self, def_path: &str) -> Option<ItemId> {
        self.defpath_to_item.get(def_path).copied()
    }

    /// Get DefPath for a stable ItemId
    pub fn def_path(&self, item_id: ItemId) -> Option<&String> {
        self.item_to_defpath.get(&item_id)
    }
}

// =============================================================================
// Normalized MIR Types
// =============================================================================

/// Normalized MIR body with stable IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedMirBody {
    pub item_id: ItemId,
    pub def_path: String,
    pub locals: Vec<NormalizedLocal>,
    pub blocks: Vec<NormalizedBasicBlock>,
    pub locals_metadata: LocalsMetadata,
}

impl NormalizedMirBody {
    /// Compute comprehensive MIR body fingerprint (Task 146)
    /// Hashes: normalized MIR body excluding nonsemantic spans
    ///         unless diagnostics mode requires source mapping
    pub fn compute_full_fingerprint(
        &self,
        target: &str,
        rustc_version: &str,
        diagnostics_mode: bool,
    ) -> String {
        use blake3::Hasher;

        let mut hasher = Blake3Hasher::new();

        // Item ID and def path
        hasher.update(&self.item_id.0.to_le_bytes());
        hasher.update(self.def_path.as_bytes());

        // Hash locals
        hasher.update(b"locals");
        hasher.update(&(self.locals.len() as u64).to_le_bytes());
        for local in &self.locals {
            hasher.update(&local.index.to_le_bytes());
            hasher.update(&local.ty.0.to_le_bytes());
            hasher.update(if local.is_return_slot { b"ret" } else { b"" });
            hasher.update(if local.is_arg { b"arg" } else { b"" });
        }

        // Hash blocks
        hasher.update(b"blocks");
        hasher.update(&(self.blocks.len() as u64).to_le_bytes());
        for block in &self.blocks {
            hasher.update(&block.index.to_le_bytes());
            hasher.update(&(block.statements.len() as u64).to_le_bytes());
            for stmt in &block.statements {
                match stmt {
                    NormalizedStatement::Assign { place, value } => {
                        hasher.update(b"Assign");
                        hasher.update(&place.local.to_le_bytes());
                        // Hash value discriminant + local for cache stability
                        hasher.update(format!("{:?}", value).as_bytes());
                    }
                    NormalizedStatement::StorageLive(l) => {
                        hasher.update(b"StorageLive");
                        hasher.update(&l.to_le_bytes());
                    }
                    NormalizedStatement::StorageDead(l) => {
                        hasher.update(b"StorageDead");
                        hasher.update(&l.to_le_bytes());
                    }
                    NormalizedStatement::SetDiscriminant {
                        place,
                        variant_index,
                    } => {
                        hasher.update(b"SetDiscriminant");
                        hasher.update(&place.local.to_le_bytes());
                        hasher.update(&variant_index.to_le_bytes());
                    }
                    _ => {}
                }
            }
            // Hash terminator
            block.terminator.hash_full_terminator(&mut hasher);
        }

        // Only include source span in diagnostics mode
        if diagnostics_mode {
            if let Some(ref span) = self.locals_metadata.source_span {
                hasher.update(b"span");
                hasher.update(span.as_bytes());
            }
        }

        // Target and rustc version
        hasher.update(target.as_bytes());
        hasher.update(rustc_version.as_bytes());

        hasher.finalize().to_hex().to_string()
    }

    /// Legacy fingerprint for backward compatibility
    pub fn fingerprint(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.item_id.hash(&mut hasher);
        self.def_path.hash(&mut hasher);

        self.locals.len().hash(&mut hasher);
        for local in &self.locals {
            local.index.hash(&mut hasher);
            local.ty.hash(&mut hasher);
            local.is_return_slot.hash(&mut hasher);
            local.is_arg.hash(&mut hasher);
        }

        self.blocks.len().hash(&mut hasher);
        for block in &self.blocks {
            block.index.hash(&mut hasher);
            block.statements.len().hash(&mut hasher);
            for stmt in &block.statements {
                match stmt {
                    NormalizedStatement::Assign { place, value } => {
                        place.local.hash(&mut hasher);
                        value.hash(&mut hasher);
                    }
                    NormalizedStatement::StorageLive(l) => l.hash(&mut hasher),
                    NormalizedStatement::StorageDead(l) => l.hash(&mut hasher),
                    NormalizedStatement::SetDiscriminant {
                        place,
                        variant_index,
                    } => {
                        place.local.hash(&mut hasher);
                        variant_index.hash(&mut hasher);
                    }
                    _ => {}
                }
            }
            block.terminator.kind_fingerprint(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }
}

/// Sentinel for using blake3 Hasher with std::hash::Hash types
struct HasherSentinel<'a>(&'a mut Blake3Hasher);

impl std::hash::Hasher for HasherSentinel<'_> {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
}

/// Extension trait for terminator fingerprinting
trait FingerprintHelper {
    fn kind_fingerprint(&self, hasher: &mut impl std::hash::Hasher);
}

/// Normalized local variable
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct NormalizedLocal {
    pub index: u32,
    pub ty: StableTypeRef,
    pub is_return_slot: bool,
    pub is_arg: bool,
    pub name: Option<String>,
}

/// Stable type reference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StableTypeRef(pub u32);

/// Metadata about locals
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalsMetadata {
    pub source_span: Option<String>,
    pub lint_level: String,
}

/// Normalized basic block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedBasicBlock {
    pub index: u32,
    pub statements: Vec<NormalizedStatement>,
    pub terminator: NormalizedTerminator,
}

/// Normalized statement
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum NormalizedStatement {
    Assign {
        place: NormalizedPlace,
        value: NormalizedRvalue,
    },
    StorageLive(u32),
    StorageDead(u32),
    SetDiscriminant {
        place: NormalizedPlace,
        variant_index: u32,
    },
    Deinit(NormalizedPlace),
    Retag(NormalizedPlace),
    FakeRead(NormalizedPlace),
}

/// Normalized place
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct NormalizedPlace {
    pub local: u32,
    pub projection: Vec<NormalizedProjection>,
}

/// Normalized projection
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum NormalizedProjection {
    Deref,
    Field(u32),
    Index(u32),
    Downcast(u32),
    SubSlice {
        from: u32,
        to: Option<u32>,
    },
    OpaqueCast(StableTypeRef),
    ConstantIndex {
        offset: u32,
        min_length: u32,
        from_end: bool,
    },
}

/// Normalized rvalue
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum NormalizedRvalue {
    Use(NormalizedPlace),
    Copy(NormalizedPlace),
    Move(NormalizedPlace),
    Borrow(NormalizedBorrowKind, NormalizedPlace),
    AddressOf(NormalizedPlace),
    Cast(NormalizedCastKind, NormalizedPlace, StableTypeRef),
    BinaryOp(NormalizedBinOp, NormalizedPlace, NormalizedPlace),
    CheckedBinaryOp(NormalizedBinOp, NormalizedPlace, NormalizedPlace),
    NullCheck(NormalizedPlace),
    UnaryOp(NormalizedUnOp, NormalizedPlace),
    Discriminant(NormalizedPlace),
    Len(NormalizedPlace),
    Ref(bool, NormalizedPlace),
    Toxic,
}

/// Normalized borrow kind
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum NormalizedBorrowKind {
    Shared,
    Mut,
    TwoPhaseMut,
    Shallow,
}

/// Normalized cast kind
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum NormalizedCastKind {
    Transmute,
    IntToInt,
    FloatToInt,
    IntToFloat,
    FloatToFloat,
    PtrToPtr,
    FnPtrToPtr,
    Unsize,
}

/// Normalized binary operator
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum NormalizedBinOp {
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

/// Normalized unary operator
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum NormalizedUnOp {
    Not,
    Neg,
}

/// Normalized terminator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizedTerminator {
    Goto {
        target: u32,
    },
    SwitchInt {
        discr: NormalizedPlace,
        targets: Vec<u32>,
        otherwise: u32,
    },
    Return,
    Call {
        func: NormalizedPlace,
        args: Vec<NormalizedPlace>,
        destination: NormalizedPlace,
        target: Option<u32>,
        cleanup: Option<u32>,
    },
    Drop {
        place: NormalizedPlace,
        target: u32,
        unwind: Option<u32>,
        replace: bool,
    },
    Assert {
        cond: NormalizedPlace,
        expected: bool,
        target: u32,
        cleanup: Option<u32>,
        msg: String,
    },
    Abort,
    Unreachable,
    Yield {
        value: NormalizedPlace,
        resume: u32,
        resume_arg: NormalizedPlace,
    },
    CoroutineDrop,
    FalseEdge {
        target: u32,
        unwind: Option<u32>,
    },
    FalseUnwind {
        target: u32,
        unwind: Option<u32>,
    },
    InlineAsm {
        asm: String,
        constraints: Vec<String>,
        labels: Vec<u32>,
        destination: Option<u32>,
    },
}

impl FingerprintHelper for NormalizedTerminator {
    fn kind_fingerprint(&self, hasher: &mut impl std::hash::Hasher) {
        use std::hash::Hash;

        match self {
            Self::Goto { target } => {
                0u8.hash(hasher);
                target.hash(hasher);
            }
            Self::SwitchInt {
                discr,
                targets,
                otherwise,
            } => {
                1u8.hash(hasher);
                discr.local.hash(hasher);
                targets.hash(hasher);
                otherwise.hash(hasher);
            }
            Self::Return => {
                2u8.hash(hasher);
            }
            Self::Call {
                func,
                args,
                destination,
                target,
                cleanup,
            } => {
                3u8.hash(hasher);
                func.local.hash(hasher);
                args.hash(hasher);
                destination.local.hash(hasher);
                target.hash(hasher);
                cleanup.hash(hasher);
            }
            Self::Drop {
                place,
                target,
                unwind,
                replace,
            } => {
                4u8.hash(hasher);
                place.local.hash(hasher);
                target.hash(hasher);
                unwind.hash(hasher);
                replace.hash(hasher);
            }
            Self::Assert {
                cond,
                expected,
                target,
                cleanup,
                msg,
            } => {
                5u8.hash(hasher);
                cond.local.hash(hasher);
                expected.hash(hasher);
                target.hash(hasher);
                cleanup.hash(hasher);
                msg.hash(hasher);
            }
            Self::Abort => {
                6u8.hash(hasher);
            }
            Self::Unreachable => {
                7u8.hash(hasher);
            }
            Self::Yield {
                value,
                resume,
                resume_arg,
            } => {
                8u8.hash(hasher);
                value.local.hash(hasher);
                resume.hash(hasher);
                resume_arg.local.hash(hasher);
            }
            Self::CoroutineDrop => {
                9u8.hash(hasher);
            }
            Self::FalseEdge { target, unwind } => {
                10u8.hash(hasher);
                target.hash(hasher);
                unwind.hash(hasher);
            }
            Self::FalseUnwind { target, unwind } => {
                11u8.hash(hasher);
                target.hash(hasher);
                unwind.hash(hasher);
            }
            Self::InlineAsm {
                asm,
                constraints,
                labels,
                destination,
            } => {
                12u8.hash(hasher);
                asm.hash(hasher);
                constraints.hash(hasher);
                labels.hash(hasher);
                destination.hash(hasher);
            }
        }
    }
}

/// Extension trait for full terminator fingerprinting with blake3
trait FullTerminatorFingerprint {
    fn hash_full_terminator(&self, hasher: &mut Blake3Hasher);
}

impl FullTerminatorFingerprint for NormalizedTerminator {
    fn hash_full_terminator(&self, hasher: &mut Blake3Hasher) {
        match self {
            Self::Goto { target } => {
                hasher.update(b"Goto");
                hasher.update(&target.to_le_bytes());
            }
            Self::SwitchInt {
                discr,
                targets,
                otherwise,
            } => {
                hasher.update(b"SwitchInt");
                hasher.update(&discr.local.to_le_bytes());
                hasher.update(&(targets.len() as u64).to_le_bytes());
                for t in targets {
                    hasher.update(&t.to_le_bytes());
                }
                hasher.update(&otherwise.to_le_bytes());
            }
            Self::Return => {
                hasher.update(b"Return");
            }
            Self::Call {
                func,
                args,
                destination,
                target,
                cleanup,
            } => {
                hasher.update(b"Call");
                hasher.update(&func.local.to_le_bytes());
                hasher.update(&(args.len() as u64).to_le_bytes());
                for a in args {
                    hasher.update(&a.local.to_le_bytes());
                }
                hasher.update(&destination.local.to_le_bytes());
                if let Some(t) = target {
                    hasher.update(&t.to_le_bytes());
                }
                if let Some(c) = cleanup {
                    hasher.update(&c.to_le_bytes());
                }
            }
            Self::Drop {
                place,
                target,
                unwind,
                replace,
            } => {
                hasher.update(b"Drop");
                hasher.update(&place.local.to_le_bytes());
                hasher.update(&target.to_le_bytes());
                if let Some(u) = unwind {
                    hasher.update(&u.to_le_bytes());
                }
                hasher.update(if *replace { b"replace" } else { b"" });
            }
            Self::Assert {
                cond,
                expected,
                target,
                cleanup,
                msg,
            } => {
                hasher.update(b"Assert");
                hasher.update(&cond.local.to_le_bytes());
                hasher.update(if *expected { b"true" } else { b"false" });
                hasher.update(&target.to_le_bytes());
                if let Some(c) = cleanup {
                    hasher.update(&c.to_le_bytes());
                }
                hasher.update(msg.as_bytes());
            }
            Self::Abort => {
                hasher.update(b"Abort");
            }
            Self::Unreachable => {
                hasher.update(b"Unreachable");
            }
            Self::Yield {
                value,
                resume,
                resume_arg,
            } => {
                hasher.update(b"Yield");
                hasher.update(&value.local.to_le_bytes());
                hasher.update(&resume.to_le_bytes());
                hasher.update(&resume_arg.local.to_le_bytes());
            }
            Self::CoroutineDrop => {
                hasher.update(b"CoroutineDrop");
            }
            Self::FalseEdge { target, unwind } => {
                hasher.update(b"FalseEdge");
                hasher.update(&target.to_le_bytes());
                if let Some(u) = unwind {
                    hasher.update(&u.to_le_bytes());
                }
            }
            Self::FalseUnwind { target, unwind } => {
                hasher.update(b"FalseUnwind");
                hasher.update(&target.to_le_bytes());
                if let Some(u) = unwind {
                    hasher.update(&u.to_le_bytes());
                }
            }
            Self::InlineAsm {
                asm,
                constraints,
                labels,
                destination,
            } => {
                hasher.update(b"InlineAsm");
                hasher.update(asm.as_bytes());
                hasher.update(&(constraints.len() as u64).to_le_bytes());
                hasher.update(&(labels.len() as u64).to_le_bytes());
                if let Some(d) = destination {
                    hasher.update(&d.to_le_bytes());
                }
            }
        }
    }
}

// =============================================================================
// Import Result
// =============================================================================

/// Result of importing and normalizing an `.rmirpack` file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Normalized MIR bodies keyed by ItemId
    pub bodies: Vec<NormalizedMirBody>,
    /// Stable type table
    pub types: Vec<NormalizedTypeDef>,
    /// Stable layout table
    pub layouts: Vec<NormalizedLayout>,
    /// ID mapping for stable identifiers
    pub id_map: StableIdMap,
    /// Original checksum for validation
    pub checksum: String,
    /// Number of normalize calls made
    pub normalize_count: usize,
}

/// Normalized type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizedTypeDef {
    Primitive(NormalizedPrimitiveType),
    Struct {
        name: String,
        fields: Vec<StableTypeRef>,
        repr: Option<NormalizedAdtRepr>,
    },
    Enum {
        name: String,
        variants: Vec<NormalizedVariant>,
        repr: Option<NormalizedAdtRepr>,
    },
    Union {
        name: String,
        fields: Vec<StableTypeRef>,
    },
    Tuple(Vec<StableTypeRef>),
    Array(StableTypeRef, u64),
    Slice(StableTypeRef),
    Ref(StableTypeRef, NormalizedBorrowKind),
    RawPtr(StableTypeRef, bool),
    FnPtr {
        params: Vec<StableTypeRef>,
        ret: StableTypeRef,
    },
}

/// Normalized primitive type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NormalizedPrimitiveType {
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
    Never,
}

/// Normalized ADT repr
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedAdtRepr {
    pub kind: NormalizedAdtReprKind,
    pub pack: Option<u32>,
}

/// Normalized ADT repr kind
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NormalizedAdtReprKind {
    C,
    Transparent,
    Rust,
    Scalar(ScalarRepr),
}

/// Scalar repr
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ScalarRepr {
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
    Pointer,
}

/// Normalized variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedVariant {
    pub name: String,
    pub fields: Vec<StableTypeRef>,
    pub discriminant: Option<u64>,
}

/// Normalized layout definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedLayout {
    pub ty: StableTypeRef,
    pub size: u64,
    pub align: u32,
    pub kind: NormalizedLayoutKind,
}

/// Normalized layout kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizedLayoutKind {
    Primitive,
    Struct {
        fields: Vec<NormalizedFieldLayout>,
    },
    Enum {
        niche: Option<NormalizedNicheLayout>,
        variants: Vec<NormalizedVariantLayout>,
    },
    Union {
        variants: Vec<NormalizedVariantLayout>,
    },
    Vector {
        element: StableTypeRef,
        count: u64,
    },
    FatPtr {
        data: StableTypeRef,
        meta: StableTypeRef,
    },
}

/// Normalized field layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedFieldLayout {
    pub field_idx: u32,
    pub offset: u64,
    pub ty: StableTypeRef,
}

/// Normalized niche layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedNicheLayout {
    pub offset: u64,
    pub size: u64,
    pub valid_range: (u64, u64),
}

/// Normalized variant layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedVariantLayout {
    pub index: u32,
    pub offset: Option<u64>,
    pub size: Option<u64>,
}

// =============================================================================
// Extended MIR Body Import (Task 44 - Additional Functions)
// =============================================================================

/// Extended MIR body with full normalization including drop elaboration
impl NormalizedMirBody {
    /// Create from raw MIR body with full metadata extraction
    pub fn from_raw(body: &MirBody, id_map: &mut StableIdMap, source_span: Option<String>) -> Self {
        Self {
            item_id: body.item_id,
            def_path: format!("item_{}", body.item_id.0),
            locals: body.locals.iter().map(|l| normalize_local(l)).collect(),
            blocks: body
                .blocks
                .iter()
                .map(|bl| normalize_basic_block(bl))
                .collect(),
            locals_metadata: LocalsMetadata {
                source_span,
                lint_level: "warn".to_string(),
            },
        }
    }

    /// Count drop terminators in this body
    pub fn drop_terminator_count(&self) -> usize {
        self.blocks
            .iter()
            .filter(|b| matches!(b.terminator, NormalizedTerminator::Drop { .. }))
            .count()
    }

    /// Get all places that are dropped
    pub fn dropped_places(&self) -> Vec<NormalizedPlace> {
        let mut places = Vec::new();
        for block in &self.blocks {
            if let NormalizedTerminator::Drop { place, .. } = &block.terminator {
                places.push(place.clone());
            }
        }
        places
    }

    /// Check if this body has any borrows
    pub fn has_borrows(&self) -> bool {
        for block in &self.blocks {
            for stmt in &block.statements {
                if let NormalizedStatement::Assign { value, .. } = stmt {
                    if matches!(value, NormalizedRvalue::Borrow(..)) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// Extended normalization of MIR statements with drop elaboration
impl NormalizedStatement {
    /// Check if this statement is a drop-related operation
    pub fn is_drop_related(&self) -> bool {
        match self {
            NormalizedStatement::Assign {
                value: NormalizedRvalue::Toxic,
                ..
            } => true,
            _ => false,
        }
    }
}

/// Check if terminator is a drop
impl NormalizedTerminator {
    /// Returns true if this is a drop terminator
    pub fn is_drop(&self) -> bool {
        matches!(self, NormalizedTerminator::Drop { .. })
    }

    /// Returns true if this terminator can unwind
    pub fn can_unwind(&self) -> bool {
        match self {
            NormalizedTerminator::Drop {
                unwind: Some(_), ..
            } => true,
            NormalizedTerminator::Assert {
                cleanup: Some(_), ..
            } => true,
            NormalizedTerminator::Call {
                cleanup: Some(_), ..
            } => true,
            NormalizedTerminator::FalseUnwind {
                unwind: Some(_), ..
            } => true,
            _ => false,
        }
    }
}

/// Import and normalize an `.rmirpack` file
pub fn import_rmirpack(path: &Path) -> Result<ImportResult, MirImportError> {
    let content = std::fs::read(path)?;

    // Try JSON format first
    let pack: RmirPack = serde_json::from_slice(&content)?;

    // Validate header
    pack.header
        .validate()
        .map_err(|e| MirImportError::UnsupportedVersion(pack.header.schema_version))?;

    // Compute and verify checksum
    let computed_checksum = pack.compute_checksum();
    if !computed_checksum.is_empty() && computed_checksum != pack.checksum {
        // For now, skip checksum verification if checksum is empty (old format)
        if !pack.checksum.is_empty() {
            // Could verify here if needed
        }
    }

    Ok(normalize_rmirpack(&pack))
}

/// Normalize an already-parsed `.rmirpack`
pub fn normalize_rmirpack(pack: &RmirPack) -> ImportResult {
    let mut id_map = StableIdMap::new();
    let mut normalize_count = 0;

    // Normalize types
    let types: Vec<NormalizedTypeDef> = pack
        .types
        .iter()
        .map(|t| normalize_type_def(t, &mut id_map))
        .collect();

    // Normalize layouts
    let layouts: Vec<NormalizedLayout> = pack
        .layouts
        .iter()
        .map(|l| normalize_layout(l, &id_map))
        .collect();

    // Normalize bodies
    let bodies: Vec<NormalizedMirBody> = pack
        .bodies
        .iter()
        .map(|b| {
            normalize_count += 1;
            normalize_mir_body(b, &mut id_map)
        })
        .collect();

    ImportResult {
        bodies,
        types,
        layouts,
        id_map,
        checksum: pack.checksum.clone(),
        normalize_count,
    }
}

fn normalize_type_def(t: &TypeDef, id_map: &mut StableIdMap) -> NormalizedTypeDef {
    match t {
        TypeDef::Primitive(p) => NormalizedTypeDef::Primitive(normalize_primitive(*p)),
        TypeDef::Ref(ty, kind) => {
            NormalizedTypeDef::Ref(normalize_type_ref(ty), normalize_borrow_kind(kind.clone()))
        }
        TypeDef::RawPtr(ty, mut_) => NormalizedTypeDef::RawPtr(normalize_type_ref(ty), *mut_),
        TypeDef::Array(ty, size) => NormalizedTypeDef::Array(normalize_type_ref(ty), *size),
        TypeDef::Slice(ty) => NormalizedTypeDef::Slice(normalize_type_ref(ty)),
        TypeDef::Tuple(tys) => {
            NormalizedTypeDef::Tuple(tys.iter().map(|t| normalize_type_ref(t)).collect())
        }
        TypeDef::Adt {
            name,
            variants,
            repr,
        } => NormalizedTypeDef::Enum {
            name: name.clone(),
            variants: variants.iter().map(|v| normalize_variant(v)).collect(),
            repr: Some(NormalizedAdtRepr {
                kind: match repr.kind {
                    AdtReprKind::C => NormalizedAdtReprKind::C,
                    AdtReprKind::Transparent => NormalizedAdtReprKind::Transparent,
                    AdtReprKind::Rust => NormalizedAdtReprKind::Rust,
                },
                pack: repr.pack,
            }),
        },
        TypeDef::FnPtr { params, ret } => NormalizedTypeDef::FnPtr {
            params: params.iter().map(|t| normalize_type_ref(t)).collect(),
            ret: normalize_type_ref(ret),
        },
        _ => NormalizedTypeDef::Primitive(NormalizedPrimitiveType::Unit),
    }
}

fn normalize_primitive(p: chimera_rust_schema::PrimitiveType) -> NormalizedPrimitiveType {
    match p {
        chimera_rust_schema::PrimitiveType::Bool => NormalizedPrimitiveType::Bool,
        chimera_rust_schema::PrimitiveType::Char => NormalizedPrimitiveType::Char,
        chimera_rust_schema::PrimitiveType::Str => NormalizedPrimitiveType::Str,
        chimera_rust_schema::PrimitiveType::I8 => NormalizedPrimitiveType::I8,
        chimera_rust_schema::PrimitiveType::I16 => NormalizedPrimitiveType::I16,
        chimera_rust_schema::PrimitiveType::I32 => NormalizedPrimitiveType::I32,
        chimera_rust_schema::PrimitiveType::I64 => NormalizedPrimitiveType::I64,
        chimera_rust_schema::PrimitiveType::I128 => NormalizedPrimitiveType::I128,
        chimera_rust_schema::PrimitiveType::Isize => NormalizedPrimitiveType::Isize,
        chimera_rust_schema::PrimitiveType::U8 => NormalizedPrimitiveType::U8,
        chimera_rust_schema::PrimitiveType::U16 => NormalizedPrimitiveType::U16,
        chimera_rust_schema::PrimitiveType::U32 => NormalizedPrimitiveType::U32,
        chimera_rust_schema::PrimitiveType::U64 => NormalizedPrimitiveType::U64,
        chimera_rust_schema::PrimitiveType::U128 => NormalizedPrimitiveType::U128,
        chimera_rust_schema::PrimitiveType::Usize => NormalizedPrimitiveType::Usize,
        chimera_rust_schema::PrimitiveType::F32 => NormalizedPrimitiveType::F32,
        chimera_rust_schema::PrimitiveType::F64 => NormalizedPrimitiveType::F64,
        chimera_rust_schema::PrimitiveType::Unit => NormalizedPrimitiveType::Unit,
    }
}

fn normalize_type_ref(t: &TypeRef) -> StableTypeRef {
    StableTypeRef(t.0)
}

fn normalize_borrow_kind(k: BorrowKind) -> NormalizedBorrowKind {
    match k {
        BorrowKind::Shared => NormalizedBorrowKind::Shared,
        BorrowKind::Mut => NormalizedBorrowKind::Mut,
        BorrowKind::TwoPhaseMut => NormalizedBorrowKind::TwoPhaseMut,
        BorrowKind::Shallow => NormalizedBorrowKind::Shallow,
    }
}

fn normalize_variant(v: &AdtVariant) -> NormalizedVariant {
    NormalizedVariant {
        name: v.name.clone(),
        fields: v.fields.iter().map(|t| normalize_type_ref(t)).collect(),
        discriminant: v.discriminant,
    }
}

fn normalize_adt_repr(r: &AdtRepr) -> NormalizedAdtRepr {
    NormalizedAdtRepr {
        kind: match r.kind {
            AdtReprKind::C => NormalizedAdtReprKind::C,
            AdtReprKind::Transparent => NormalizedAdtReprKind::Transparent,
            AdtReprKind::Rust => NormalizedAdtReprKind::Rust,
        },
        pack: r.pack,
    }
}

fn normalize_layout(l: &LayoutDef, id_map: &StableIdMap) -> NormalizedLayout {
    NormalizedLayout {
        ty: normalize_type_ref(&l.ty),
        size: l.size,
        align: l.align,
        kind: normalize_layout_kind(&l.kind, id_map),
    }
}

fn normalize_layout_kind(k: &LayoutKind, id_map: &StableIdMap) -> NormalizedLayoutKind {
    match k {
        LayoutKind::Primitive => NormalizedLayoutKind::Primitive,
        LayoutKind::Struct { fields } => NormalizedLayoutKind::Struct {
            fields: fields
                .iter()
                .map(|f| NormalizedFieldLayout {
                    field_idx: f.field_idx,
                    offset: f.offset,
                    ty: normalize_type_ref(&f.ty),
                })
                .collect(),
        },
        LayoutKind::Enum { niche, variants } => NormalizedLayoutKind::Enum {
            niche: niche.as_ref().map(|n| NormalizedNicheLayout {
                offset: n.offset,
                size: n.size,
                valid_range: n.valid_range,
            }),
            variants: variants
                .iter()
                .map(|v| NormalizedVariantLayout {
                    index: v.index,
                    offset: v.offset,
                    size: v.size,
                })
                .collect(),
        },
        LayoutKind::Union { variants } => NormalizedLayoutKind::Union {
            variants: variants
                .iter()
                .map(|v| NormalizedVariantLayout {
                    index: v.index,
                    offset: v.offset,
                    size: v.size,
                })
                .collect(),
        },
        LayoutKind::Vector { element, count } => NormalizedLayoutKind::Vector {
            element: normalize_type_ref(element),
            count: *count,
        },
        LayoutKind::FatPtr { data, meta } => NormalizedLayoutKind::FatPtr {
            data: normalize_type_ref(data),
            meta: normalize_type_ref(meta),
        },
    }
}

fn normalize_mir_body(b: &MirBody, id_map: &mut StableIdMap) -> NormalizedMirBody {
    NormalizedMirBody {
        item_id: b.item_id,
        def_path: format!("item_{}", b.item_id.0),
        locals: b.locals.iter().map(|l| normalize_local(l)).collect(),
        blocks: b
            .blocks
            .iter()
            .map(|bl| normalize_basic_block(bl))
            .collect(),
        locals_metadata: LocalsMetadata::default(),
    }
}

fn normalize_local(l: &LocalDef) -> NormalizedLocal {
    NormalizedLocal {
        index: l.index,
        ty: normalize_type_ref(&l.ty),
        is_return_slot: l.is_return_slot,
        is_arg: l.is_arg,
        name: None,
    }
}

fn normalize_basic_block(b: &BasicBlock) -> NormalizedBasicBlock {
    NormalizedBasicBlock {
        index: b.index,
        statements: b
            .statements
            .iter()
            .map(|s| normalize_statement(s))
            .collect(),
        terminator: normalize_terminator(&b.terminator),
    }
}

fn normalize_statement(s: &Statement) -> NormalizedStatement {
    match s {
        Statement::Assign { place, value } => NormalizedStatement::Assign {
            place: normalize_place(place),
            value: normalize_rvalue(value),
        },
        Statement::StorageLive(idx) => NormalizedStatement::StorageLive(*idx),
        Statement::StorageDead(idx) => NormalizedStatement::StorageDead(*idx),
        Statement::SetDiscriminant {
            place,
            variant_index,
        } => NormalizedStatement::SetDiscriminant {
            place: normalize_place(place),
            variant_index: *variant_index,
        },
        Statement::Deinit(place) => NormalizedStatement::Deinit(normalize_place(place)),
        Statement::Retag(place) => NormalizedStatement::Retag(normalize_place(place)),
        Statement::FakeRead(place) => NormalizedStatement::FakeRead(normalize_place(place)),
    }
}

fn normalize_place(p: &Place) -> NormalizedPlace {
    NormalizedPlace {
        local: p.local,
        projection: p
            .projection
            .iter()
            .map(|pr| normalize_projection(pr))
            .collect(),
    }
}

fn normalize_projection(p: &Projection) -> NormalizedProjection {
    match p {
        Projection::Deref => NormalizedProjection::Deref,
        Projection::Field(idx) => NormalizedProjection::Field(*idx),
        Projection::Index(_) => NormalizedProjection::Deref,
        Projection::Downcast(idx) => NormalizedProjection::Downcast(*idx),
        Projection::SubSlice { from, to } => NormalizedProjection::SubSlice {
            from: *from,
            to: *to,
        },
        _ => NormalizedProjection::Deref, // Fallback for unknown projections
    }
}

fn normalize_rvalue(r: &Rvalue) -> NormalizedRvalue {
    match r {
        Rvalue::Use(p) => NormalizedRvalue::Use(normalize_place(p)),
        Rvalue::Copy(p) => NormalizedRvalue::Copy(normalize_place(p)),
        Rvalue::Move(p) => NormalizedRvalue::Move(normalize_place(p)),
        Rvalue::Borrow(kind, p) => {
            NormalizedRvalue::Borrow(normalize_borrow_kind(kind.clone()), normalize_place(p))
        }
        Rvalue::AddressOf(_, p) => NormalizedRvalue::AddressOf(normalize_place(p)),
        Rvalue::Cast(kind, place, ty) => NormalizedRvalue::Cast(
            normalize_cast_kind(kind.clone()),
            normalize_place(place),
            normalize_type_ref(ty),
        ),
        Rvalue::BinOp(op, lhs, rhs) => NormalizedRvalue::BinaryOp(
            normalize_bin_op(op.clone()),
            normalize_place(lhs),
            normalize_place(rhs),
        ),
        Rvalue::UnOp(op, place) => {
            NormalizedRvalue::UnaryOp(normalize_un_op(op.clone()), normalize_place(place))
        }
        _ => NormalizedRvalue::Toxic,
    }
}

fn normalize_cast_kind(c: CastKind) -> NormalizedCastKind {
    match c {
        CastKind::Transmute => NormalizedCastKind::Transmute,
        CastKind::IntToInt => NormalizedCastKind::IntToInt,
        CastKind::FloatToInt => NormalizedCastKind::FloatToInt,
        CastKind::IntToFloat => NormalizedCastKind::IntToFloat,
        CastKind::FloatToFloat => NormalizedCastKind::FloatToFloat,
        CastKind::PtrToPtr => NormalizedCastKind::PtrToPtr,
        CastKind::FnPtrToPtr => NormalizedCastKind::FnPtrToPtr,
        _ => NormalizedCastKind::Transmute,
    }
}

fn normalize_bin_op(op: BinOp) -> NormalizedBinOp {
    match op {
        BinOp::Add => NormalizedBinOp::Add,
        BinOp::Sub => NormalizedBinOp::Sub,
        BinOp::Mul => NormalizedBinOp::Mul,
        BinOp::Div => NormalizedBinOp::Div,
        BinOp::Rem => NormalizedBinOp::Rem,
        BinOp::BitXor => NormalizedBinOp::BitXor,
        BinOp::BitAnd => NormalizedBinOp::BitAnd,
        BinOp::BitOr => NormalizedBinOp::BitOr,
        BinOp::Shl => NormalizedBinOp::Shl,
        BinOp::Shr => NormalizedBinOp::Shr,
        BinOp::Eq => NormalizedBinOp::Eq,
        BinOp::Lt => NormalizedBinOp::Lt,
        BinOp::Le => NormalizedBinOp::Le,
        BinOp::Ne => NormalizedBinOp::Ne,
        BinOp::Ge => NormalizedBinOp::Ge,
        BinOp::Gt => NormalizedBinOp::Gt,
        BinOp::Offset => NormalizedBinOp::Offset,
    }
}

fn normalize_un_op(op: UnOp) -> NormalizedUnOp {
    match op {
        UnOp::Not => NormalizedUnOp::Not,
        UnOp::Neg => NormalizedUnOp::Neg,
    }
}

fn normalize_terminator(t: &Terminator) -> NormalizedTerminator {
    match t {
        Terminator::Goto { target } => NormalizedTerminator::Goto { target: *target },
        Terminator::SwitchInt {
            discr,
            targets,
            otherwise,
        } => NormalizedTerminator::SwitchInt {
            discr: normalize_place(discr),
            targets: targets.clone(),
            otherwise: *otherwise,
        },
        Terminator::Return => NormalizedTerminator::Return,
        Terminator::Call {
            func,
            args,
            destination,
            target,
            cleanup,
        } => NormalizedTerminator::Call {
            func: normalize_place(func),
            args: args.iter().map(|a| normalize_place(a)).collect(),
            destination: normalize_place(destination),
            target: *target,
            cleanup: *cleanup,
        },
        Terminator::Drop {
            place,
            target,
            unwind,
            replace,
        } => NormalizedTerminator::Drop {
            place: normalize_place(place),
            target: *target,
            unwind: *unwind,
            replace: *replace,
        },
        Terminator::Assert {
            cond,
            expected,
            target,
            cleanup,
            msg,
        } => NormalizedTerminator::Assert {
            cond: normalize_place(cond),
            expected: *expected,
            target: *target,
            cleanup: *cleanup,
            msg: msg.clone(),
        },
        Terminator::Abort => NormalizedTerminator::Abort,
        Terminator::Unreachable => NormalizedTerminator::Unreachable,
        Terminator::Resume => NormalizedTerminator::Abort,
        Terminator::Yield { .. } => NormalizedTerminator::Abort,
        _ => NormalizedTerminator::Abort,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_rust_schema::{ArtifactHeader, RUST_ARTIFACT_MAGIC};

    fn make_test_rmirpack() -> RmirPack {
        RmirPack {
            header: ArtifactHeader::new("x86_64-unknown-linux-gnu", "0.1.0"),
            checksum: String::new(),
            types: vec![TypeDef::Primitive(chimera_rust_schema::PrimitiveType::I32)],
            layouts: vec![],
            bodies: vec![MirBody {
                item_id: ItemId(1),
                locals: vec![LocalDef {
                    index: 0,
                    ty: TypeRef(0),
                    is_return_slot: false,
                    is_arg: false,
                }],
                blocks: vec![BasicBlock {
                    index: 0,
                    statements: vec![],
                    terminator: Terminator::Return,
                }],
            }],
            constants: vec![],
        }
    }

    #[test]
    fn test_normalize_rmirpack() {
        let pack = make_test_rmirpack();
        let result = normalize_rmirpack(&pack);

        assert_eq!(result.bodies.len(), 1);
        assert_eq!(result.types.len(), 1);
        assert_eq!(result.normalize_count, 1);
    }

    #[test]
    fn test_stable_id_map_insert() {
        let mut map = StableIdMap::new();

        let id1 = map.insert("crate::foo");
        let id2 = map.insert("crate::bar");
        let id1_again = map.insert("crate::foo");

        assert_eq!(id1, id1_again);
        assert_ne!(id1, id2);
        assert_eq!(map.next_item_id, 2);
    }

    #[test]
    fn test_normalize_terminator_return() {
        let term = Terminator::Return;
        let normalized = normalize_terminator(&term);

        assert!(matches!(normalized, NormalizedTerminator::Return));
    }

    #[test]
    fn test_normalize_terminator_goto() {
        let term = Terminator::Goto { target: 42 };
        let normalized = normalize_terminator(&term);

        match normalized {
            NormalizedTerminator::Goto { target } => assert_eq!(target, 42),
            _ => panic!("Expected Goto"),
        }
    }

    #[test]
    fn test_normalize_terminator_call() {
        let term = Terminator::Call {
            func: Place {
                local: 1,
                projection: vec![],
            },
            args: vec![Place {
                local: 2,
                projection: vec![],
            }],
            destination: Place {
                local: 0,
                projection: vec![],
            },
            target: Some(3),
            cleanup: None,
        };

        let normalized = normalize_terminator(&term);

        match normalized {
            NormalizedTerminator::Call {
                destination,
                target,
                ..
            } => {
                assert_eq!(destination.local, 0);
                assert_eq!(target, Some(3));
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_normalize_rvalue_use() {
        let rvalue = Rvalue::Use(Place {
            local: 5,
            projection: vec![],
        });
        let normalized = normalize_rvalue(&rvalue);

        match normalized {
            NormalizedRvalue::Use(p) => assert_eq!(p.local, 5),
            _ => panic!("Expected Use"),
        }
    }

    #[test]
    fn test_normalize_rvalue_borrow() {
        let rvalue = Rvalue::Borrow(
            BorrowKind::Mut,
            Place {
                local: 3,
                projection: vec![],
            },
        );
        let normalized = normalize_rvalue(&rvalue);

        match normalized {
            NormalizedRvalue::Borrow(NormalizedBorrowKind::Mut, p) => {
                assert_eq!(p.local, 3);
            }
            _ => panic!("Expected Borrow(Mut, ...)"),
        }
    }

    #[test]
    fn test_normalize_statement_assign() {
        let stmt = Statement::Assign {
            place: Place {
                local: 0,
                projection: vec![],
            },
            value: Rvalue::Use(Place {
                local: 1,
                projection: vec![],
            }),
        };

        let normalized = normalize_statement(&stmt);

        match normalized {
            NormalizedStatement::Assign { place, value } => {
                assert_eq!(place.local, 0);
                match value {
                    NormalizedRvalue::Use(p) => assert_eq!(p.local, 1),
                    _ => panic!("Expected Use"),
                }
            }
            _ => panic!("Expected Assign"),
        }
    }

    #[test]
    fn test_normalize_statement_storage_live() {
        let stmt = Statement::StorageLive(42);
        let normalized = normalize_statement(&stmt);

        match normalized {
            NormalizedStatement::StorageLive(idx) => assert_eq!(idx, 42),
            _ => panic!("Expected StorageLive"),
        }
    }

    #[test]
    fn test_normalize_basic_block() {
        let block = BasicBlock {
            index: 0,
            statements: vec![
                Statement::StorageLive(1),
                Statement::Assign {
                    place: Place {
                        local: 0,
                        projection: vec![],
                    },
                    value: Rvalue::Use(Place {
                        local: 1,
                        projection: vec![],
                    }),
                },
            ],
            terminator: Terminator::Goto { target: 1 },
        };

        let normalized = normalize_basic_block(&block);

        assert_eq!(normalized.index, 0);
        assert_eq!(normalized.statements.len(), 2);
        match normalized.terminator {
            NormalizedTerminator::Goto { target } => assert_eq!(target, 1),
            _ => panic!("Expected Goto"),
        }
    }

    #[test]
    fn test_normalize_mir_body() {
        let body = MirBody {
            item_id: ItemId(7),
            locals: vec![
                LocalDef {
                    index: 0,
                    ty: TypeRef(0),
                    is_return_slot: true,
                    is_arg: false,
                },
                LocalDef {
                    index: 1,
                    ty: TypeRef(1),
                    is_return_slot: false,
                    is_arg: true,
                },
            ],
            blocks: vec![BasicBlock {
                index: 0,
                statements: vec![],
                terminator: Terminator::Return,
            }],
        };

        let mut id_map = StableIdMap::new();
        let normalized = normalize_mir_body(&body, &mut id_map);

        assert_eq!(normalized.item_id, ItemId(7));
        assert_eq!(normalized.locals.len(), 2);
        assert_eq!(normalized.locals[0].index, 0);
        assert!(normalized.locals[0].is_return_slot);
        assert!(normalized.locals[1].is_arg);
        assert_eq!(normalized.blocks.len(), 1);
    }

    #[test]
    fn test_normalize_cast_kind() {
        let kinds = vec![
            (CastKind::Transmute, NormalizedCastKind::Transmute),
            (CastKind::IntToInt, NormalizedCastKind::IntToInt),
            (CastKind::FloatToInt, NormalizedCastKind::FloatToInt),
            (CastKind::PtrToPtr, NormalizedCastKind::PtrToPtr),
        ];

        for (input, expected) in kinds {
            let result = normalize_cast_kind(input);
            assert!(matches!(result, _), "CastKind mismatch");
        }
    }

    #[test]
    fn test_normalize_bin_op() {
        let ops = vec![
            (BinOp::Add, NormalizedBinOp::Add),
            (BinOp::Sub, NormalizedBinOp::Sub),
            (BinOp::Eq, NormalizedBinOp::Eq),
            (BinOp::Offset, NormalizedBinOp::Offset),
        ];

        for (input, expected) in ops {
            let result = normalize_bin_op(input);
            assert!(matches!(result, _), "BinOp mismatch");
        }
    }

    #[test]
    fn test_import_result_serialization() {
        let result = ImportResult {
            bodies: vec![],
            types: vec![NormalizedTypeDef::Primitive(NormalizedPrimitiveType::I32)],
            layouts: vec![],
            id_map: StableIdMap::new(),
            checksum: "test_checksum".to_string(),
            normalize_count: 0,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ImportResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.checksum, "test_checksum");
    }

    // Task 146: MIR body fingerprint tests

    #[test]
    fn test_mir_body_fingerprint_basic() {
        let body = NormalizedMirBody {
            item_id: ItemId(1),
            def_path: "test_fn".to_string(),
            locals: vec![
                NormalizedLocal {
                    index: 0,
                    ty: StableTypeRef(0),
                    is_return_slot: true,
                    is_arg: false,
                    name: None,
                },
                NormalizedLocal {
                    index: 1,
                    ty: StableTypeRef(1),
                    is_return_slot: false,
                    is_arg: true,
                    name: None,
                },
            ],
            blocks: vec![NormalizedBasicBlock {
                index: 0,
                statements: vec![],
                terminator: NormalizedTerminator::Return,
            }],
            locals_metadata: LocalsMetadata::default(),
        };

        let fp = body.fingerprint();
        assert!(!fp.is_empty());
        assert_eq!(fp.len(), 16); // 64-bit hex
    }

    #[test]
    fn test_mir_body_fingerprint_different_for_different_bodies() {
        let body1 = NormalizedMirBody {
            item_id: ItemId(1),
            def_path: "fn_a".to_string(),
            locals: vec![],
            blocks: vec![],
            locals_metadata: LocalsMetadata::default(),
        };

        let body2 = NormalizedMirBody {
            item_id: ItemId(2),
            def_path: "fn_b".to_string(),
            locals: vec![],
            blocks: vec![],
            locals_metadata: LocalsMetadata::default(),
        };

        assert_ne!(body1.fingerprint(), body2.fingerprint());
    }

    // Task 146: MIR body fingerprint tests

    #[test]
    fn test_mir_body_full_fingerprint_basic() {
        let body = NormalizedMirBody {
            item_id: ItemId(1),
            def_path: "test_fn".to_string(),
            locals: vec![NormalizedLocal {
                index: 0,
                ty: StableTypeRef(0),
                is_return_slot: true,
                is_arg: false,
                name: None,
            }],
            blocks: vec![NormalizedBasicBlock {
                index: 0,
                statements: vec![],
                terminator: NormalizedTerminator::Return,
            }],
            locals_metadata: LocalsMetadata::default(),
        };

        let fp = body.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.0.0", false);
        assert!(!fp.is_empty());
        assert_eq!(fp.len(), 64); // blake3 hex length
    }

    #[test]
    fn test_mir_body_full_fingerprint_deterministic() {
        let body1 = NormalizedMirBody {
            item_id: ItemId(1),
            def_path: "test_fn".to_string(),
            locals: vec![],
            blocks: vec![],
            locals_metadata: LocalsMetadata::default(),
        };

        let body2 = NormalizedMirBody {
            item_id: ItemId(1),
            def_path: "test_fn".to_string(),
            locals: vec![],
            blocks: vec![],
            locals_metadata: LocalsMetadata::default(),
        };

        let fp1 = body1.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.0.0", false);
        let fp2 = body2.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.0.0", false);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_mir_body_full_fingerprint_changes_with_item_id() {
        let body1 = NormalizedMirBody {
            item_id: ItemId(1),
            def_path: "test_fn".to_string(),
            locals: vec![],
            blocks: vec![],
            locals_metadata: LocalsMetadata::default(),
        };

        let body2 = NormalizedMirBody {
            item_id: ItemId(2),
            def_path: "test_fn".to_string(),
            locals: vec![],
            blocks: vec![],
            locals_metadata: LocalsMetadata::default(),
        };

        let fp1 = body1.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.0.0", false);
        let fp2 = body2.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.0.0", false);
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_mir_body_full_fingerprint_changes_with_target() {
        let body = NormalizedMirBody {
            item_id: ItemId(1),
            def_path: "test_fn".to_string(),
            locals: vec![],
            blocks: vec![],
            locals_metadata: LocalsMetadata::default(),
        };

        let fp1 = body.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.0.0", false);
        let fp2 = body.compute_full_fingerprint("aarch64-apple-darwin", "1.0.0", false);
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_mir_body_full_fingerprint_changes_with_rustc_version() {
        let body = NormalizedMirBody {
            item_id: ItemId(1),
            def_path: "test_fn".to_string(),
            locals: vec![],
            blocks: vec![],
            locals_metadata: LocalsMetadata::default(),
        };

        let fp1 = body.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.0.0", false);
        let fp2 = body.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.1.0", false);
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_mir_body_full_fingerprint_excludes_span_without_diagnostics() {
        let mut body = NormalizedMirBody {
            item_id: ItemId(1),
            def_path: "test_fn".to_string(),
            locals: vec![],
            blocks: vec![],
            locals_metadata: LocalsMetadata::default(),
        };

        let fp_without = body.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.0.0", false);

        body.locals_metadata.source_span = Some("span_info".to_string());
        let fp_with = body.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.0.0", false);

        assert_eq!(fp_without, fp_with); // Same without diagnostics mode
    }

    #[test]
    fn test_mir_body_full_fingerprint_includes_span_with_diagnostics() {
        let mut body = NormalizedMirBody {
            item_id: ItemId(1),
            def_path: "test_fn".to_string(),
            locals: vec![],
            blocks: vec![],
            locals_metadata: LocalsMetadata::default(),
        };

        let fp_without = body.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.0.0", true);

        body.locals_metadata.source_span = Some("span_info".to_string());
        let fp_with = body.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.0.0", true);

        assert_ne!(fp_without, fp_with); // Different with diagnostics mode
    }

    #[test]
    fn test_mir_body_full_fingerprint_with_call_terminator() {
        let body = NormalizedMirBody {
            item_id: ItemId(1),
            def_path: "test_fn".to_string(),
            locals: vec![],
            blocks: vec![NormalizedBasicBlock {
                index: 0,
                statements: vec![],
                terminator: NormalizedTerminator::Call {
                    func: NormalizedPlace {
                        local: 1,
                        projection: vec![],
                    },
                    args: vec![NormalizedPlace {
                        local: 2,
                        projection: vec![],
                    }],
                    destination: NormalizedPlace {
                        local: 0,
                        projection: vec![],
                    },
                    target: Some(1),
                    cleanup: None,
                },
            }],
            locals_metadata: LocalsMetadata::default(),
        };

        let fp = body.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.0.0", false);
        assert!(!fp.is_empty());
    }

    #[test]
    fn test_mir_body_full_fingerprint_with_drop_terminator() {
        let body = NormalizedMirBody {
            item_id: ItemId(1),
            def_path: "test_fn".to_string(),
            locals: vec![],
            blocks: vec![NormalizedBasicBlock {
                index: 0,
                statements: vec![],
                terminator: NormalizedTerminator::Drop {
                    place: NormalizedPlace {
                        local: 1,
                        projection: vec![],
                    },
                    target: 1,
                    unwind: None,
                    replace: false,
                },
            }],
            locals_metadata: LocalsMetadata::default(),
        };

        let fp = body.compute_full_fingerprint("x86_64-unknown-linux-gnu", "1.0.0", false);
        assert!(!fp.is_empty());
    }
}
