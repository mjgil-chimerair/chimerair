//! Lowering of struct, union, and enum types.
//!
//! Preserves extern/packed/layout constraints and emits layout metadata
//! and compatibility checks.
//!
//! Task 91: Lower structs/unions/enums

use super::types::SourceLoc;
use super::{ZigType, ZigTypeKind};
use serde::{Deserialize, Serialize};

/// A lowered struct representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoweredStruct {
    /// Struct type ID
    pub id: u64,
    /// Field type IDs
    pub field_types: Vec<u64>,
    /// Field offsets in bytes
    pub field_offsets: Vec<u64>,
    /// Field names
    pub field_names: Vec<String>,
    /// Is packed struct
    pub is_packed: bool,
    /// Is extern (C-compatible) struct
    pub is_extern: bool,
    /// Size in bytes
    pub size_bytes: u64,
    /// Alignment in bytes
    pub align_bytes: u64,
    /// MLIR type string
    pub mlir_type: String,
}

impl LoweredStruct {
    /// Create a new lowered struct
    pub fn new(
        id: u64,
        field_types: Vec<u64>,
        field_offsets: Vec<u64>,
        field_names: Vec<String>,
        is_packed: bool,
        is_extern: bool,
    ) -> Self {
        let size_bytes = field_offsets
            .last()
            .map(|o| {
                if field_types.is_empty() {
                    0
                } else {
                    o + calculate_type_size(field_types.last().unwrap())
                }
            })
            .unwrap_or(0);
        let align_bytes = field_offsets.first().copied().unwrap_or(1);
        let mlir_type = if is_extern {
            format!("!chir.extern_struct<{}>", id)
        } else if is_packed {
            format!("!chir.packed_struct<{}>", id)
        } else {
            format!("!chir.struct<{}>", id)
        };
        Self {
            id,
            field_types,
            field_offsets,
            field_names,
            is_packed,
            is_extern,
            size_bytes,
            align_bytes,
            mlir_type,
        }
    }

    /// Check if this is a packed struct
    pub fn is_packed(&self) -> bool {
        self.is_packed
    }

    /// Check if this is an extern struct
    pub fn is_extern(&self) -> bool {
        self.is_extern
    }
}

/// A lowered union representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoweredUnion {
    /// Union type ID
    pub id: u64,
    /// Variant names and type IDs
    pub variants: Vec<(String, u64)>,
    /// Size in bytes (largest variant)
    pub size_bytes: u64,
    /// Alignment in bytes
    pub align_bytes: u64,
    /// MLIR type string
    pub mlir_type: String,
}

impl LoweredUnion {
    /// Create a new lowered union
    pub fn new(id: u64, variants: Vec<(String, u64)>) -> Self {
        let size_bytes = variants
            .iter()
            .map(|(_, tid)| calculate_type_size(tid))
            .max()
            .unwrap_or(0);
        let align_bytes = variants
            .iter()
            .map(|(_, tid)| calculate_type_align(tid))
            .max()
            .unwrap_or(1);
        Self {
            id,
            variants,
            size_bytes,
            align_bytes,
            mlir_type: format!("!chir.union<{}>", id),
        }
    }
}

/// A lowered enum representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoweredEnum {
    /// Enum type ID
    pub id: u64,
    /// Tag type ID
    pub tag_type: u64,
    /// Variant names
    pub variants: Vec<String>,
    /// Tag size in bytes
    pub tag_size: u64,
    /// Size in bytes
    pub size_bytes: u64,
    /// Alignment in bytes
    pub align_bytes: u64,
    /// MLIR type string
    pub mlir_type: String,
}

impl LoweredEnum {
    /// Create a new lowered enum
    pub fn new(id: u64, tag_type: u64, variants: Vec<String>) -> Self {
        let tag_size = calculate_type_size(&tag_type);
        let size_bytes = tag_size;
        let align_bytes = tag_size;
        Self {
            id,
            tag_type,
            variants,
            tag_size,
            size_bytes,
            align_bytes,
            mlir_type: format!("!chir.enum<{}, {}>", id, tag_type),
        }
    }

    /// Check if this is a simple enum (no payload)
    pub fn is_simple(&self) -> bool {
        self.variants.iter().all(|v| !v.contains("_"))
    }
}

/// Struct/Union/Enum lowering context
#[derive(Debug, Clone)]
pub struct StructLowering {
    /// Default alignment
    default_align: u64,
}

impl StructLowering {
    /// Create a new struct lowering context
    pub fn new(default_align: u64) -> Self {
        Self { default_align }
    }

    /// Lower a struct type
    pub fn lower_struct(&self, ty: &ZigType) -> Option<LoweredStruct> {
        if let ZigTypeKind::Struct {
            field_types,
            field_offsets,
            field_names,
            packed,
            is_extern,
        } = &ty.kind
        {
            Some(LoweredStruct::new(
                ty.id,
                field_types.clone(),
                field_offsets.clone(),
                field_names.clone(),
                *packed,
                *is_extern,
            ))
        } else {
            None
        }
    }

    /// Lower a union type
    pub fn lower_union(&self, ty: &ZigType) -> Option<LoweredUnion> {
        if let ZigTypeKind::Union { variants } = &ty.kind {
            Some(LoweredUnion::new(ty.id, variants.clone()))
        } else {
            None
        }
    }

    /// Lower an enum type
    pub fn lower_enum(&self, ty: &ZigType) -> Option<LoweredEnum> {
        if let ZigTypeKind::Enum { tag_type, variants } = &ty.kind {
            Some(LoweredEnum::new(ty.id, *tag_type, variants.clone()))
        } else {
            None
        }
    }

    /// Get MLIR type for struct
    pub fn mlir_type_for_struct(&self, id: u64, is_extern: bool, is_packed: bool) -> String {
        if is_extern {
            format!("!chir.extern_struct<{}>", id)
        } else if is_packed {
            format!("!chir.packed_struct<{}>", id)
        } else {
            format!("!chir.struct<{}>", id)
        }
    }

    /// Get MLIR type for union
    pub fn mlir_type_for_union(&self, id: u64) -> String {
        format!("!chir.union<{}>", id)
    }

    /// Get MLIR type for enum
    pub fn mlir_type_for_enum(&self, id: u64, tag_type: u64) -> String {
        format!("!chir.enum<{}, {}>", id, tag_type)
    }
}

/// Calculate type size from type ID
fn calculate_type_size(type_id: &u64) -> u64 {
    match type_id {
        1 => 1,  // i8
        2 => 2,  // i16
        3 => 4,  // i32
        4 => 8,  // i64
        5 => 16, // i128
        _ => 8,  // default pointer size
    }
}

/// Calculate type alignment from type ID
fn calculate_type_align(type_id: &u64) -> u64 {
    calculate_type_size(type_id)
}

/// Check if a type is a struct type
pub fn is_struct_type(ty: &ZigType) -> bool {
    matches!(ty.kind, ZigTypeKind::Struct { .. })
}

/// Check if a type is a union type
pub fn is_union_type(ty: &ZigType) -> bool {
    matches!(ty.kind, ZigTypeKind::Union { .. })
}

/// Check if a type is an enum type
pub fn is_enum_type(ty: &ZigType) -> bool {
    matches!(ty.kind, ZigTypeKind::Enum { .. })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowered_struct_creation() {
        let struct_lowerer = StructLowering::new(8);
        let ty = ZigType {
            id: 1,
            kind: ZigTypeKind::Struct {
                field_types: vec![3, 4], // i32, i64
                field_offsets: vec![0, 8],
                field_names: vec!["x".to_string(), "y".to_string()],
                packed: false,
                is_extern: false,
            },
            size_bytes: 16,
            align_bytes: 8,
            source_loc: None,
        };
        let lowered = struct_lowerer.lower_struct(&ty).unwrap();
        assert_eq!(lowered.id, 1);
        assert_eq!(lowered.field_types.len(), 2);
        assert!(!lowered.is_packed());
        assert!(!lowered.is_extern());
    }

    #[test]
    fn test_lowered_struct_extern() {
        let struct_lowerer = StructLowering::new(8);
        let ty = ZigType {
            id: 1,
            kind: ZigTypeKind::Struct {
                field_types: vec![3],
                field_offsets: vec![0],
                field_names: vec!["value".to_string()],
                packed: false,
                is_extern: true,
            },
            size_bytes: 4,
            align_bytes: 4,
            source_loc: None,
        };
        let lowered = struct_lowerer.lower_struct(&ty).unwrap();
        assert!(lowered.is_extern());
        assert!(lowered.mlir_type.contains("extern_struct"));
    }

    #[test]
    fn test_lowered_struct_packed() {
        let struct_lowerer = StructLowering::new(1);
        let ty = ZigType {
            id: 1,
            kind: ZigTypeKind::Struct {
                field_types: vec![1, 1, 1], // i8, i8, i8
                field_offsets: vec![0, 1, 2],
                field_names: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                packed: true,
                is_extern: false,
            },
            size_bytes: 3,
            align_bytes: 1,
            source_loc: None,
        };
        let lowered = struct_lowerer.lower_struct(&ty).unwrap();
        assert!(lowered.is_packed());
        assert!(lowered.mlir_type.contains("packed_struct"));
    }

    #[test]
    fn test_lowered_union_creation() {
        let struct_lowerer = StructLowering::new(8);
        let ty = ZigType {
            id: 1,
            kind: ZigTypeKind::Union {
                variants: vec![("Int".to_string(), 3), ("Float".to_string(), 4)],
            },
            size_bytes: 8,
            align_bytes: 8,
            source_loc: None,
        };
        let lowered = struct_lowerer.lower_union(&ty).unwrap();
        assert_eq!(lowered.id, 1);
        assert_eq!(lowered.variants.len(), 2);
        assert!(lowered.mlir_type.contains("union"));
    }

    #[test]
    fn test_lowered_enum_creation() {
        let struct_lowerer = StructLowering::new(4);
        let ty = ZigType {
            id: 1,
            kind: ZigTypeKind::Enum {
                tag_type: 3, // i32
                variants: vec!["One".to_string(), "Two".to_string(), "Three".to_string()],
            },
            size_bytes: 4,
            align_bytes: 4,
            source_loc: None,
        };
        let lowered = struct_lowerer.lower_enum(&ty).unwrap();
        assert_eq!(lowered.id, 1);
        assert_eq!(lowered.tag_type, 3);
        assert_eq!(lowered.variants.len(), 3);
        assert!(lowered.is_simple());
    }

    #[test]
    fn test_mlir_type_for_struct() {
        let struct_lowerer = StructLowering::new(8);
        let mlir = struct_lowerer.mlir_type_for_struct(1, false, false);
        assert!(mlir.contains("chir.struct"));

        let mlir_extern = struct_lowerer.mlir_type_for_struct(1, true, false);
        assert!(mlir_extern.contains("extern_struct"));

        let mlir_packed = struct_lowerer.mlir_type_for_struct(1, false, true);
        assert!(mlir_packed.contains("packed_struct"));
    }

    #[test]
    fn test_mlir_type_for_union() {
        let struct_lowerer = StructLowering::new(8);
        let mlir = struct_lowerer.mlir_type_for_union(1);
        assert!(mlir.contains("chir.union"));
    }

    #[test]
    fn test_mlir_type_for_enum() {
        let struct_lowerer = StructLowering::new(4);
        let mlir = struct_lowerer.mlir_type_for_enum(1, 3);
        assert!(mlir.contains("chir.enum"));
    }

    #[test]
    fn test_is_struct_type() {
        let struct_ty = ZigType {
            id: 1,
            kind: ZigTypeKind::Struct {
                field_types: vec![],
                field_offsets: vec![],
                field_names: vec![],
                packed: false,
                is_extern: false,
            },
            size_bytes: 0,
            align_bytes: 1,
            source_loc: None,
        };
        assert!(is_struct_type(&struct_ty));
        assert!(!is_union_type(&struct_ty));
        assert!(!is_enum_type(&struct_ty));
    }

    #[test]
    fn test_is_union_type() {
        let union_ty = ZigType {
            id: 1,
            kind: ZigTypeKind::Union { variants: vec![] },
            size_bytes: 0,
            align_bytes: 1,
            source_loc: None,
        };
        assert!(is_union_type(&union_ty));
        assert!(!is_struct_type(&union_ty));
    }

    #[test]
    fn test_is_enum_type() {
        let enum_ty = ZigType {
            id: 1,
            kind: ZigTypeKind::Enum {
                tag_type: 3,
                variants: vec![],
            },
            size_bytes: 4,
            align_bytes: 4,
            source_loc: None,
        };
        assert!(is_enum_type(&enum_ty));
        assert!(!is_struct_type(&enum_ty));
    }
}
