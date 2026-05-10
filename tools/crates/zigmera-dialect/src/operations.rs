//! Zig operations for AIR lowering.

use super::types::SourceLoc;
use serde::{Deserialize, Serialize};

/// Zig AIR operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZigOp {
    // Control flow
    /// Unconditional branch
    Br,
    /// Conditional branch
    BrCond,
    /// Switch statement
    Switch,
    /// Return value
    Ret,
    /// Return void
    RetVoid,
    /// Function call
    Call,
    /// Indirect call (function pointer)
    CallIndirect,
    /// Invoke (with error handling)
    Invoke,
    /// Unreachable
    Unreachable,

    // Memory operations
    /// Load from memory
    Load,
    /// Store to memory
    Store,
    /// Element address
    ElemPtr,
    /// Field address
    FieldPtr,
    /// Index pointer
    IndexPtr,
    /// Address of
    AddrOf,
    /// Allocate local
    Alloca,
    /// Allocate global
    AllocGlobal,

    // Arithmetic
    /// Addition
    Add,
    /// Subtraction
    Sub,
    /// Multiplication
    Mul,
    /// Division
    Div,
    /// Modulo
    Mod,
    /// Remainder
    Rem,

    // Bit operations
    /// Bitwise AND
    And,
    /// Bitwise OR
    Or,
    /// Bitwise XOR
    Xor,
    /// Left shift
    Shl,
    /// Right shift
    Shr,

    // Comparison
    /// Equal
    Eq,
    /// Not equal
    Ne,
    /// Signed less than
    Slt,
    /// Signed less or equal
    Sle,
    /// Signed greater than
    Sgt,
    /// Signed greater or equal
    Sge,
    /// Unsigned less than
    Ult,
    /// Unsigned less or equal
    Ule,
    /// Unsigned greater than
    Ugt,
    /// Unsigned greater or equal
    Uge,

    // Type conversions
    /// Zero extend
    Zext,
    /// Sign extend
    Sext,
    /// Truncate
    Trunc,
    /// Bitcast
    Bitcast,
    /// Int to pointer
    IntToPtr,
    /// Pointer to int
    PtrToInt,
    /// Float to int
    FpToInt,
    /// Int to float
    IntToFp,
    /// Float to float
    FpTrunc,
    /// Float extend
    FpExtend,

    // Aggregate operations
    /// Extract value
    ExtractValue,
    /// Insert value
    InsertValue,
    /// Extract element
    ExtractElement,
    /// Insert element
    InsertElement,

    // Error handling
    /// Wrap in error union
    WrapErr,
    /// Unwrap error union (payload or error)
    UnwrapErr,
    /// Check if error
    IsErr,
    /// Merge errors
    MergeErr,
    /// Try (propagate error or continue with payload)
    Try,
    /// Catch (handle error case)
    Catch,
    /// Errdefer marker (cleanup on error path)
    ErrdeferMarker,

    // Synchronization
    /// Atomic load
    AtomicLoad,
    /// Atomic store
    AtomicStore,
    /// Fence
    Fence,

    // Async/frame (unsupported)
    /// Suspend frame
    SuspendFrame,
    /// Resume
    Resume,
    /// Await
    Await,

    // Comptime
    /// Compile-time evaluation marker
    Comptime,
    /// Type coercion
    Coerce,
    /// Runtime type info
    RuntimeTypeInfo,

    // Vector/SIMD (may be unsupported)
    /// Vector reduction
    VectorReduce,

    // Inline assembly (unsupported or opaque)
    /// Inline assembly
    InlineAsm,

    // Unknown operation (for unsupported)
    Unknown,
}

impl ZigOp {
    /// Create from AIR operation string
    pub fn from_air_str(op: &str) -> Self {
        match op {
            // Control flow
            "br" => ZigOp::Br,
            "br_cond" => ZigOp::BrCond,
            "switch" => ZigOp::Switch,
            "ret" => ZigOp::Ret,
            "ret_void" => ZigOp::RetVoid,
            "call" => ZigOp::Call,
            "call_indirect" => ZigOp::CallIndirect,
            "invoke" => ZigOp::Invoke,
            "unreachable" => ZigOp::Unreachable,

            // Memory
            "load" => ZigOp::Load,
            "store" => ZigOp::Store,
            "elem_ptr" => ZigOp::ElemPtr,
            "field_ptr" => ZigOp::FieldPtr,
            "index_ptr" => ZigOp::IndexPtr,
            "addr_of" => ZigOp::AddrOf,
            "alloca" => ZigOp::Alloca,
            "alloc_global" => ZigOp::AllocGlobal,

            // Arithmetic
            "add" => ZigOp::Add,
            "sub" => ZigOp::Sub,
            "mul" => ZigOp::Mul,
            "div" => ZigOp::Div,
            "mod" => ZigOp::Mod,
            "rem" => ZigOp::Rem,

            // Bit operations
            "and" => ZigOp::And,
            "or" => ZigOp::Or,
            "xor" => ZigOp::Xor,
            "shl" => ZigOp::Shl,
            "shr" => ZigOp::Shr,

            // Comparison
            "eq" => ZigOp::Eq,
            "ne" => ZigOp::Ne,
            "slt" => ZigOp::Slt,
            "sle" => ZigOp::Sle,
            "sgt" => ZigOp::Sgt,
            "sge" => ZigOp::Sge,
            "ult" => ZigOp::Ult,
            "ule" => ZigOp::Ule,
            "ugt" => ZigOp::Ugt,
            "uge" => ZigOp::Uge,

            // Type conversions
            "zext" => ZigOp::Zext,
            "sext" => ZigOp::Sext,
            "trunc" => ZigOp::Trunc,
            "bitcast" => ZigOp::Bitcast,
            "int_to_ptr" => ZigOp::IntToPtr,
            "ptr_to_int" => ZigOp::PtrToInt,
            "fp_to_int" => ZigOp::FpToInt,
            "int_to_fp" => ZigOp::IntToFp,
            "fp_trunc" => ZigOp::FpTrunc,
            "fp_extend" => ZigOp::FpExtend,

            // Aggregate
            "extract_value" => ZigOp::ExtractValue,
            "insert_value" => ZigOp::InsertValue,
            "extract_element" => ZigOp::ExtractElement,
            "insert_element" => ZigOp::InsertElement,

            // Error handling
            "wrap_err" => ZigOp::WrapErr,
            "unwrap_err" => ZigOp::UnwrapErr,
            "is_err" => ZigOp::IsErr,
            "merge_err" => ZigOp::MergeErr,
            "try" => ZigOp::Try,
            "catch" => ZigOp::Catch,
            "errdefer_marker" => ZigOp::ErrdeferMarker,

            // Atomics
            "atomic_load" => ZigOp::AtomicLoad,
            "atomic_store" => ZigOp::AtomicStore,
            "fence" => ZigOp::Fence,

            // Async/frame
            "suspend_frame" => ZigOp::SuspendFrame,
            "resume" => ZigOp::Resume,
            "await" => ZigOp::Await,

            // Comptime
            "comptime" => ZigOp::Comptime,
            "coerce" => ZigOp::Coerce,
            "runtime_type_info" => ZigOp::RuntimeTypeInfo,

            // Vector
            "vector_reduce" => ZigOp::VectorReduce,

            // Inline asm
            "inline_asm" => ZigOp::InlineAsm,

            _ => ZigOp::Unknown,
        }
    }

    /// Create from AIR op enum variant
    pub fn from_air_op(op: &str) -> Self {
        Self::from_air_str(op)
    }

    /// Check if this is a supported operation
    pub fn is_supported(&self) -> bool {
        !matches!(
            self,
            ZigOp::SuspendFrame | ZigOp::Resume | ZigOp::Await | ZigOp::InlineAsm | ZigOp::Unknown
        )
    }

    /// Check if this operation has side effects
    pub fn has_side_effects(&self) -> bool {
        matches!(
            self,
            ZigOp::Store
                | ZigOp::AtomicStore
                | ZigOp::Call
                | ZigOp::CallIndirect
                | ZigOp::Invoke
                | ZigOp::Alloca
                | ZigOp::AllocGlobal
                | ZigOp::Fence
        )
    }

    /// Check if this is a terminator
    pub fn is_terminator(&self) -> bool {
        matches!(
            self,
            ZigOp::Br
                | ZigOp::BrCond
                | ZigOp::Switch
                | ZigOp::Ret
                | ZigOp::RetVoid
                | ZigOp::Unreachable
                | ZigOp::Invoke
        )
    }
}

/// An AIR instruction lowered to Zig dialect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirInst {
    pub id: u64,
    pub op: String,
    pub result_type: Option<u64>,
    pub operands: Vec<u64>,
    pub source_loc: Option<SourceLoc>,
}

impl AirInst {
    /// Create a new instruction
    pub fn new(id: u64, op: &str) -> Self {
        Self {
            id,
            op: op.to_string(),
            result_type: None,
            operands: Vec::new(),
            source_loc: None,
        }
    }

    /// Add an operand
    pub fn with_operand(mut self, operand: u64) -> Self {
        self.operands.push(operand);
        self
    }

    /// Set result type
    pub fn with_result_type(mut self, ty: u64) -> Self {
        self.result_type = Some(ty);
        self
    }
}

/// A lowered Zig instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigInstruction {
    pub id: u64,
    pub op: ZigOp,
    pub result_type: Option<u64>,
    pub operands: Vec<u64>,
    pub source_loc: Option<SourceLoc>,
}

impl ZigInstruction {
    /// Create a new instruction
    pub fn new(id: u64, op: ZigOp) -> Self {
        Self {
            id,
            op,
            result_type: None,
            operands: Vec::new(),
            source_loc: None,
        }
    }

    /// Add an operand
    pub fn add_operand(&mut self, operand: u64) {
        self.operands.push(operand);
    }

    /// Set result type
    pub fn set_result_type(&mut self, ty: u64) {
        self.result_type = Some(ty);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zig_op_from_air() {
        assert!(matches!(ZigOp::from_air_str("add"), ZigOp::Add));
        assert!(matches!(ZigOp::from_air_str("load"), ZigOp::Load));
        assert!(matches!(ZigOp::from_air_str("br"), ZigOp::Br));
        assert!(matches!(ZigOp::from_air_str("call"), ZigOp::Call));
    }

    #[test]
    fn test_zig_op_is_supported() {
        assert!(ZigOp::Add.is_supported());
        assert!(ZigOp::Load.is_supported());
        assert!(ZigOp::Br.is_supported());
        assert!(!ZigOp::InlineAsm.is_supported());
        assert!(!ZigOp::Await.is_supported());
        assert!(!ZigOp::Unknown.is_supported());
    }

    #[test]
    fn test_zig_op_has_side_effects() {
        assert!(ZigOp::Store.has_side_effects());
        assert!(ZigOp::Call.has_side_effects());
        assert!(!ZigOp::Add.has_side_effects());
        assert!(!ZigOp::Load.has_side_effects());
    }

    #[test]
    fn test_zig_op_is_terminator() {
        assert!(ZigOp::Ret.is_terminator());
        assert!(ZigOp::Br.is_terminator());
        assert!(ZigOp::Unreachable.is_terminator());
        assert!(!ZigOp::Add.is_terminator());
    }

    #[test]
    fn test_air_inst_creation() {
        let inst = AirInst::new(1, "add")
            .with_operand(2)
            .with_operand(3)
            .with_result_type(100);
        assert_eq!(inst.id, 1);
        assert_eq!(inst.op, "add");
        assert_eq!(inst.operands.len(), 2);
        assert_eq!(inst.result_type, Some(100));
    }
}
