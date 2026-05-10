//! `.zairpack` AIR type/layout/function bundle schema v1.

use serde::{Deserialize, Serialize};

/// Magic bytes for `.zairpack` binary format.
pub const ZAIRPACK_MAGIC: &[u8; 8] = b"ZAIRPK01";

/// Current schema version.
pub const SCHEMA_VERSION: u32 = 1;

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

/// `.zairpack` header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirpackHeader {
    pub magic: [u8; 8],
    pub schema_version: u32,
    pub zig_commit: [u8; 20],
    pub target: String,
    pub type_count: u32,
    pub layout_count: u32,
    pub function_count: u32,
    pub checksum: [u8; 32],
}

/// AIR instruction opcode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AirOp {
    // Control flow
    Br {
        dest: u32,
    },
    Ret {
        value: Option<u32>,
    },
    Switch {
        cond: u32,
        default: u32,
        cases: Vec<(u64, u32)>,
    },
    // Memory
    Load {
        ptr: u32,
        ty: u32,
    },
    Store {
        ptr: u32,
        value: u32,
    },
    Alloca {
        ty: u32,
    },
    /// Pointer to element
    Gep {
        base: u32,
        index: u32,
    },
    // Binary ops
    Add {
        lhs: u32,
        rhs: u32,
    },
    Sub {
        lhs: u32,
        rhs: u32,
    },
    Mul {
        lhs: u32,
        rhs: u32,
    },
    Div {
        lhs: u32,
        rhs: u32,
    },
    Rem {
        lhs: u32,
        rhs: u32,
    },
    And {
        lhs: u32,
        rhs: u32,
    },
    Or {
        lhs: u32,
        rhs: u32,
    },
    Xor {
        lhs: u32,
        rhs: u32,
    },
    Shl {
        lhs: u32,
        rhs: u32,
    },
    Shr {
        lhs: u32,
        rhs: u32,
    },
    // Comparison
    ICmp {
        pred: IntPredicate,
        lhs: u32,
        rhs: u32,
    },
    FCmp {
        pred: FloatPredicate,
        lhs: u32,
        rhs: u32,
    },
    // Casts
    Trunc {
        value: u32,
        dest_ty: u32,
    },
    ZExt {
        value: u32,
        dest_ty: u32,
    },
    SExt {
        value: u32,
        dest_ty: u32,
    },
    BitCast {
        value: u32,
        dest_ty: u32,
    },
    IntToPtr {
        value: u32,
        dest_ty: u32,
    },
    PtrToInt {
        value: u32,
        dest_ty: u32,
    },
    FPToUI {
        value: u32,
        dest_ty: u32,
    },
    FPToSI {
        value: u32,
        dest_ty: u32,
    },
    UIToFP {
        value: u32,
        dest_ty: u32,
    },
    SIToFP {
        value: u32,
        dest_ty: u32,
    },
    // Phi
    Phi {
        pairs: Vec<(u32, u32)>,
    },
    // Call
    Call {
        func: u32,
        args: Vec<u32>,
    },
    // Unreachable
    Unreachable,
    // Error union handling
    IsErr {
        value: u32,
    },
    WrapErrUnion {
        payload: u32,
        err_set: u32,
    },
}

/// Integer comparison predicate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntPredicate {
    Eq,
    Ne,
    Slt,
    Sle,
    Sgt,
    Sge,
    Ult,
    Ule,
    Ugt,
    Uge,
}

/// Float comparison predicate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FloatPredicate {
    Oeq,
    Ogt,
    Olt,
    Oge,
    Ole,
    One,
    Ord,
    Uno,
    Ueq,
    Ugx,
    Ule,
    Uge,
}

/// Basic block in AIR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirBlock {
    pub id: u32,
    pub instructions: Vec<AirInst>,
}

/// An AIR instruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirInst {
    pub id: u64,
    pub op: AirOp,
    pub result_type: Option<u32>,
    pub debug_loc: Option<DebugLoc>,
}

/// Debug/source location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugLoc {
    pub file_id: u32,
    pub line: u32,
    pub col: u32,
    pub span_start: u32,
    pub span_end: u32,
}

/// AIR function body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirFunction {
    pub decl_id: u64,
    pub type_id: u64,
    pub blocks: Vec<AirBlock>,
    pub basic_block_count: u32,
    pub instruction_count: u32,
}

/// Complete `.zairpack` bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirpackSchema {
    pub header: AirpackHeader,
    pub types: Vec<TypeRecord>,
    pub layouts: Vec<LayoutRecord>,
    pub functions: Vec<AirFunction>,
}

impl AirpackSchema {
    pub fn header_magic_valid(&self) -> bool {
        &self.header.magic == ZAIRPACK_MAGIC
    }

    pub fn header_version_compatible(&self) -> bool {
        self.header.schema_version <= SCHEMA_VERSION
    }
}
