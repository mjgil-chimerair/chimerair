//! Zig type definitions for dialect lowering.

use serde::{Deserialize, Serialize};

/// Zig type kinds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZigTypeKind {
    /// Integer types: (width in bits, signed)
    Int { width: u32, signed: bool },
    /// Floating point types
    Float { width: u32 },
    /// Boolean
    Bool,
    /// Void type
    Void,
    /// Pointer types: (address space, width)
    Pointer,
    /// Slice types: (element type ID)
    Slice { elem_type: u64 },
    /// Array types: (element type ID, length)
    Array { elem_type: u64, len: u64 },
    /// Struct types: (field type IDs, field offsets, field names, packed, extern)
    Struct {
        field_types: Vec<u64>,
        field_offsets: Vec<u64>,
        field_names: Vec<String>,
        packed: bool,
        is_extern: bool,
    },
    /// Union types
    Union { variants: Vec<(String, u64)> },
    /// Enum types
    Enum {
        tag_type: u64,
        variants: Vec<String>,
    },
    /// Optional types
    Optional { inner: u64 },
    /// Error set types
    ErrorSet { errors: Vec<String> },
    /// Error union types: error || T
    ErrorUnion { error_set: u64, payload: u64 },
    /// Function types
    Fn {
        params: Vec<u64>,
        return_type: Option<u64>,
        callconv: String,
    },
    /// Vector types for SIMD
    Vector { elem_type: u64, len: u32 },
    /// Opaque types (for asm, extern)
    Opaque,
    /// Type (type-level)
    Type,
}

/// A Zig type with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigType {
    /// Type ID (from AIR type table)
    pub id: u64,
    /// Type kind
    pub kind: ZigTypeKind,
    /// Size in bytes (0 for unsized)
    pub size_bytes: u64,
    /// Alignment in bytes
    pub align_bytes: u64,
    /// Source location
    pub source_loc: Option<SourceLoc>,
}

/// Source location for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLoc {
    pub file_id: u32,
    pub line: u32,
    pub column: u32,
}

impl ZigType {
    /// Create an integer type
    pub fn integer(width: u32, signed: bool) -> Self {
        Self {
            id: 0,
            kind: ZigTypeKind::Int { width, signed },
            size_bytes: (width / 8) as u64,
            align_bytes: (width / 8) as u64,
            source_loc: None,
        }
    }

    /// Create a pointer type
    pub fn pointer(width: u32) -> Self {
        Self {
            id: 0,
            kind: ZigTypeKind::Pointer,
            size_bytes: (width / 8) as u64,
            align_bytes: (width / 8) as u64,
            source_loc: None,
        }
    }

    /// Get the size of this type
    pub fn sizeof(&self) -> u64 {
        self.size_bytes
    }

    /// Get the alignment of this type
    pub fn alignof(&self) -> u64 {
        self.align_bytes
    }

    /// Check if this is a scalar type
    pub fn is_scalar(&self) -> bool {
        matches!(
            self.kind,
            ZigTypeKind::Int { .. }
                | ZigTypeKind::Float { .. }
                | ZigTypeKind::Bool
                | ZigTypeKind::Pointer
        )
    }

    /// Check if this type has interior mutability
    pub fn has_interior_mutability(&self) -> bool {
        matches!(
            self.kind,
            ZigTypeKind::Slice { .. }
                | ZigTypeKind::Array { .. }
                | ZigTypeKind::Struct { .. }
                | ZigTypeKind::Union { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_type() {
        let ty = ZigType::integer(32, true);
        assert!(matches!(
            ty.kind,
            ZigTypeKind::Int {
                width: 32,
                signed: true
            }
        ));
        assert_eq!(ty.sizeof(), 4);
    }

    #[test]
    fn test_pointer_type() {
        let ty = ZigType::pointer(64);
        assert!(matches!(ty.kind, ZigTypeKind::Pointer));
        assert_eq!(ty.sizeof(), 8);
    }

    #[test]
    fn test_scalar_check() {
        let int_ty = ZigType::integer(32, true);
        assert!(int_ty.is_scalar());

        let ptr_ty = ZigType::pointer(64);
        assert!(ptr_ty.is_scalar());
    }

    #[test]
    fn test_struct_type_creation() {
        let ty = ZigType {
            id: 1,
            kind: ZigTypeKind::Struct {
                field_types: vec![1, 2],
                field_offsets: vec![0, 8],
                field_names: vec!["x".to_string(), "y".to_string()],
                packed: false,
                is_extern: false,
            },
            size_bytes: 16,
            align_bytes: 8,
            source_loc: None,
        };
        assert!(!ty.is_scalar());
        assert!(ty.has_interior_mutability());
    }
}
