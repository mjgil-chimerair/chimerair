//! Lowering of primitive ABI types.
//!
//! Maps Zig integers, floats, booleans, void, and pointers to Chimera physical
//! and semantic types with target width/alignment.
//!
//! Task 87: Lower primitive ABI types

use super::operations::{AirInst, ZigInstruction, ZigOp};
use super::types::SourceLoc;
use super::{Block, DialectFunction, DialectModule, ZigType, ZigTypeKind};

/// Result of lowering a primitive type
#[derive(Debug, Clone)]
pub struct LoweredPrimitive {
    /// The lowered Zig type
    pub zig_type: ZigType,
    /// MLIR type string (e.g., "i32", "f64", "i8")
    pub mlir_type: String,
}

/// Lower primitive types from AIR to Zig dialect
#[derive(Debug, Clone)]
pub struct PrimitiveLowering {
    /// Target pointer width in bits
    pointer_width: u32,
    /// Target size width in bits
    size_width: u32,
}

impl PrimitiveLowering {
    /// Create a new primitive lowering for a target
    pub fn new(pointer_width: u32, size_width: u32) -> Self {
        Self {
            pointer_width,
            size_width,
        }
    }

    /// Lower an integer type
    pub fn lower_int(&self, width: u32, signed: bool) -> LoweredPrimitive {
        let zig_type = ZigType {
            id: 0,
            kind: ZigTypeKind::Int { width, signed },
            size_bytes: (width / 8) as u64,
            align_bytes: (width / 8) as u64,
            source_loc: None,
        };
        let mlir_type = if signed {
            format!("i{}", width)
        } else {
            format!("u{}", width)
        };
        LoweredPrimitive {
            zig_type,
            mlir_type,
        }
    }

    /// Lower a float type
    pub fn lower_float(&self, width: u32) -> LoweredPrimitive {
        let zig_type = ZigType {
            id: 0,
            kind: ZigTypeKind::Float { width },
            size_bytes: (width / 8) as u64,
            align_bytes: (width / 8) as u64,
            source_loc: None,
        };
        let mlir_type = format!("f{}", width);
        LoweredPrimitive {
            zig_type,
            mlir_type,
        }
    }

    /// Lower a boolean type
    pub fn lower_bool(&self) -> LoweredPrimitive {
        let zig_type = ZigType {
            id: 0,
            kind: ZigTypeKind::Bool,
            size_bytes: 1,
            align_bytes: 1,
            source_loc: None,
        };
        LoweredPrimitive {
            zig_type,
            mlir_type: "i8".to_string(),
        }
    }

    /// Lower a void type
    pub fn lower_void(&self) -> LoweredPrimitive {
        let zig_type = ZigType {
            id: 0,
            kind: ZigTypeKind::Void,
            size_bytes: 0,
            align_bytes: 0,
            source_loc: None,
        };
        LoweredPrimitive {
            zig_type,
            mlir_type: "()".to_string(),
        }
    }

    /// Lower a pointer type
    pub fn lower_pointer(&self) -> LoweredPrimitive {
        let zig_type = ZigType {
            id: 0,
            kind: ZigTypeKind::Pointer,
            size_bytes: (self.pointer_width / 8) as u64,
            align_bytes: (self.pointer_width / 8) as u64,
            source_loc: None,
        };
        LoweredPrimitive {
            zig_type,
            mlir_type: format!(
                "!chir.ptr<i{}, {}>",
                self.pointer_width,
                self.pointer_width / 8
            ),
        }
    }

    /// Get MLIR type for a Zig type kind
    pub fn mlir_type_for(&self, kind: &ZigTypeKind) -> String {
        match kind {
            ZigTypeKind::Int {
                width,
                signed: true,
            } => format!("i{}", width),
            ZigTypeKind::Int {
                width,
                signed: false,
            } => format!("u{}", width),
            ZigTypeKind::Float { width } => format!("f{}", width),
            ZigTypeKind::Bool => "i8".to_string(),
            ZigTypeKind::Void => "()".to_string(),
            ZigTypeKind::Pointer => format!(
                "!chir.ptr<i{}, {}>",
                self.pointer_width,
                self.pointer_width / 8
            ),
            _ => "?".to_string(),
        }
    }
}

/// Extension trait for AirInst to lower to ZigInstruction
pub trait AirInstLowering {
    /// Lower an AIR instruction to a Zig instruction
    fn lower_to_zig(&self, type_map: &dyn Fn(u64) -> Option<ZigType>) -> ZigInstruction;
}

impl AirInstLowering for AirInst {
    fn lower_to_zig(&self, type_map: &dyn Fn(u64) -> Option<ZigType>) -> ZigInstruction {
        let op = ZigOp::from_air_str(&self.op);
        let result_type = self
            .result_type
            .and_then(|tid| type_map(tid).map(|t| t.size_bytes));
        let mut zig_inst = ZigInstruction::new(self.id, op);
        zig_inst.result_type = self.result_type;
        zig_inst.operands = self.operands.clone();
        zig_inst.source_loc = self.source_loc.clone();
        zig_inst
    }
}

/// Lower a function containing primitive operations
pub fn lower_function_primitives(func: &mut DialectFunction) {
    for block in &mut func.blocks {
        let mut lowered_insts = Vec::new();
        for inst in &block.instructions {
            let lowered = lower_instruction_primitive(inst);
            lowered_insts.push(lowered);
        }
        block.instructions = lowered_insts;
    }
}

/// Lower a single instruction with primitive type handling
fn lower_instruction_primitive(inst: &ZigInstruction) -> ZigInstruction {
    let mut lowered = inst.clone();
    if let Some(result_type_id) = lowered.result_type {
        lowered.result_type = Some(result_type_id);
    }
    lowered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_lowering_int() {
        let lowering = PrimitiveLowering::new(64, 64);
        let result = lowering.lower_int(32, true);
        assert!(matches!(
            result.zig_type.kind,
            ZigTypeKind::Int {
                width: 32,
                signed: true
            }
        ));
        assert_eq!(result.mlir_type, "i32");
    }

    #[test]
    fn test_primitive_lowering_unsigned_int() {
        let lowering = PrimitiveLowering::new(64, 64);
        let result = lowering.lower_int(32, false);
        assert!(matches!(
            result.zig_type.kind,
            ZigTypeKind::Int {
                width: 32,
                signed: false
            }
        ));
        assert_eq!(result.mlir_type, "u32");
    }

    #[test]
    fn test_primitive_lowering_float() {
        let lowering = PrimitiveLowering::new(64, 64);
        let result = lowering.lower_float(64);
        assert!(matches!(
            result.zig_type.kind,
            ZigTypeKind::Float { width: 64 }
        ));
        assert_eq!(result.mlir_type, "f64");
    }

    #[test]
    fn test_primitive_lowering_bool() {
        let lowering = PrimitiveLowering::new(64, 64);
        let result = lowering.lower_bool();
        assert!(matches!(result.zig_type.kind, ZigTypeKind::Bool));
        assert_eq!(result.mlir_type, "i8");
    }

    #[test]
    fn test_primitive_lowering_void() {
        let lowering = PrimitiveLowering::new(64, 64);
        let result = lowering.lower_void();
        assert!(matches!(result.zig_type.kind, ZigTypeKind::Void));
        assert_eq!(result.mlir_type, "()");
    }

    #[test]
    fn test_primitive_lowering_pointer() {
        let lowering = PrimitiveLowering::new(64, 64);
        let result = lowering.lower_pointer();
        assert!(matches!(result.zig_type.kind, ZigTypeKind::Pointer));
        assert_eq!(result.mlir_type, "!chir.ptr<i64, 8>");
    }

    #[test]
    fn test_mlir_type_for_int() {
        let lowering = PrimitiveLowering::new(64, 64);
        let kind = ZigTypeKind::Int {
            width: 64,
            signed: true,
        };
        assert_eq!(lowering.mlir_type_for(&kind), "i64");
    }

    #[test]
    fn test_air_inst_lowering() {
        let lowering = PrimitiveLowering::new(64, 64);
        let air_inst = AirInst::new(1, "add")
            .with_operand(2)
            .with_operand(3)
            .with_result_type(100);
        let type_map = |_: u64| Some(ZigType::integer(32, true));
        let zig_inst = air_inst.lower_to_zig(&type_map);
        assert!(matches!(zig_inst.op, ZigOp::Add));
        assert_eq!(zig_inst.operands.len(), 2);
    }
}
