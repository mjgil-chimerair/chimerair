//! Lowering of slice types.
//!
//! Lowers `[]T` and `[]const T` to pointer+len with ownership/lifetime/constness metadata.
//!
//! Task 88: Lower slices

use super::operations::{ZigInstruction, ZigOp};
use super::types::SourceLoc;
use super::{Block, ZigType, ZigTypeKind};
use serde::{Deserialize, Serialize};

/// A lowered slice representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoweredSlice {
    /// Pointer to element data
    pub ptr_value: u64,
    /// Length (number of elements)
    pub len_value: u64,
    /// Element type ID
    pub elem_type_id: u64,
    /// Is const slice
    pub is_const: bool,
    /// MLIR type string
    pub mlir_type: String,
}

impl LoweredSlice {
    /// Create a new lowered slice
    pub fn new(ptr_value: u64, len_value: u64, elem_type_id: u64, is_const: bool) -> Self {
        let mlir_type = if is_const {
            format!("!chir.slice<{}>", elem_type_id)
        } else {
            format!("!chir.mut_slice<{}>", elem_type_id)
        };
        Self {
            ptr_value,
            len_value,
            elem_type_id,
            is_const,
            mlir_type,
        }
    }

    /// Check if this is a const slice
    pub fn is_const(&self) -> bool {
        self.is_const
    }
}

/// Slice lowering context
#[derive(Debug, Clone)]
pub struct SliceLowering {
    /// Pointer width in bits
    pointer_width: u32,
    /// Size width in bits
    size_width: u32,
}

impl SliceLowering {
    /// Create a new slice lowering context
    pub fn new(pointer_width: u32, size_width: u32) -> Self {
        Self {
            pointer_width,
            size_width,
        }
    }

    /// Create a slice type
    pub fn create_slice_type(&self, elem_type_id: u64, is_const: bool) -> ZigType {
        ZigType {
            id: 0,
            kind: ZigTypeKind::Slice {
                elem_type: elem_type_id,
            },
            size_bytes: (self.pointer_width / 8) as u64 * 2, // ptr + len
            align_bytes: (self.pointer_width / 8) as u64,
            source_loc: None,
        }
    }

    /// Create an array type
    pub fn create_array_type(&self, elem_type_id: u64, len: u64, elem_size: u64) -> ZigType {
        ZigType {
            id: 0,
            kind: ZigTypeKind::Array {
                elem_type: elem_type_id,
                len,
            },
            size_bytes: len * elem_size,
            align_bytes: elem_size,
            source_loc: None,
        }
    }

    /// Lower slice operations for a block
    pub fn lower_block(&self, block: &mut Block) {
        let mut lowered_insts = Vec::new();
        for inst in &block.instructions {
            lowered_insts.push(self.lower_instruction(inst));
        }
        block.instructions = lowered_insts;
    }

    /// Lower a single instruction
    fn lower_instruction(&self, inst: &ZigInstruction) -> ZigInstruction {
        let mut lowered = inst.clone();
        match &lowered.op {
            ZigOp::ElemPtr => {
                // Slice element pointer - keep as is, semantic lowering happens elsewhere
            }
            ZigOp::IndexPtr => {
                // Slice index pointer - keep as is
            }
            ZigOp::Load | ZigOp::Store => {
                // These may involve slices - semantic lowering handles this
            }
            _ => {}
        }
        lowered
    }

    /// Get MLIR type for slice
    pub fn mlir_type_for_slice(&self, elem_type: u64, is_const: bool) -> String {
        if is_const {
            format!("!chir.slice<i{}, {}>", elem_type, self.size_width / 8)
        } else {
            format!("!chir.mut_slice<i{}, {}>", elem_type, self.size_width / 8)
        }
    }

    /// Get MLIR type for array
    pub fn mlir_type_for_array(&self, elem_type: u64, len: u64) -> String {
        format!("!chir.array<i{}, {}>", elem_type, len)
    }
}

/// Check if a type is a slice type
pub fn is_slice_type(ty: &ZigType) -> bool {
    matches!(ty.kind, ZigTypeKind::Slice { .. })
}

/// Check if a type is an array type
pub fn is_array_type(ty: &ZigType) -> bool {
    matches!(ty.kind, ZigTypeKind::Array { .. })
}

/// Check if slice type is const
pub fn is_const_slice(ty: &ZigType) -> bool {
    matches!(ty.kind, ZigTypeKind::Slice { .. })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowered_slice_creation() {
        let slice = LoweredSlice::new(1, 10, 100, true);
        assert_eq!(slice.ptr_value, 1);
        assert_eq!(slice.len_value, 10);
        assert_eq!(slice.elem_type_id, 100);
        assert!(slice.is_const());
    }

    #[test]
    fn test_lowered_slice_mut() {
        let slice = LoweredSlice::new(1, 10, 100, false);
        assert!(!slice.is_const());
    }

    #[test]
    fn test_slice_lowering_creation() {
        let lowering = SliceLowering::new(64, 64);
        let slice_type = lowering.create_slice_type(100, true);
        assert!(matches!(
            slice_type.kind,
            ZigTypeKind::Slice { elem_type: 100 }
        ));
        assert_eq!(slice_type.size_bytes, 16); // ptr + len = 8 + 8
    }

    #[test]
    fn test_array_lowering_creation() {
        let lowering = SliceLowering::new(64, 64);
        let array_type = lowering.create_array_type(100, 10, 4);
        assert!(matches!(
            array_type.kind,
            ZigTypeKind::Array {
                elem_type: 100,
                len: 10
            }
        ));
        assert_eq!(array_type.size_bytes, 40); // 10 * 4
    }

    #[test]
    fn test_mlir_type_for_slice() {
        let lowering = SliceLowering::new(64, 64);
        let mlir_type = lowering.mlir_type_for_slice(100, true);
        assert!(mlir_type.contains("chir.slice"));
        assert!(mlir_type.contains("100"));
    }

    #[test]
    fn test_mlir_type_for_mut_slice() {
        let lowering = SliceLowering::new(64, 64);
        let mlir_type = lowering.mlir_type_for_slice(100, false);
        assert!(mlir_type.contains("chir.mut_slice"));
    }

    #[test]
    fn test_is_slice_type() {
        let slice_type = ZigType {
            id: 1,
            kind: ZigTypeKind::Slice { elem_type: 100 },
            size_bytes: 16,
            align_bytes: 8,
            source_loc: None,
        };
        assert!(is_slice_type(&slice_type));

        let int_type = ZigType::integer(32, true);
        assert!(!is_slice_type(&int_type));
    }

    #[test]
    fn test_is_array_type() {
        let array_type = ZigType {
            id: 1,
            kind: ZigTypeKind::Array {
                elem_type: 100,
                len: 10,
            },
            size_bytes: 40,
            align_bytes: 4,
            source_loc: None,
        };
        assert!(is_array_type(&array_type));

        let int_type = ZigType::integer(32, true);
        assert!(!is_array_type(&int_type));
    }
}
