//! Lowering of optional types.
//!
//! Lowers `?*T` as nullable pointer and non-pointer `?T` as target-specific
//! tagged representation or wrapper result.
//!
//! Task 89: Lower optionals

use super::operations::{ZigInstruction, ZigOp};
use super::types::SourceLoc;
use super::{ZigType, ZigTypeKind};
use serde::{Deserialize, Serialize};

/// Optional lowering representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoweredOptional {
    /// Payload type ID (0 for none)
    pub payload_type: u64,
    /// Is nullable pointer variant
    pub is_nullable_ptr: bool,
    /// Tag size in bytes (for non-pointer optionals)
    pub tag_size: u64,
    /// MLIR type string
    pub mlir_type: String,
}

impl LoweredOptional {
    /// Create a nullable pointer optional
    pub fn nullable_pointer(payload_type: u64) -> Self {
        Self {
            payload_type,
            is_nullable_ptr: true,
            tag_size: 0,
            mlir_type: format!("!chir.nullable_ptr<{}>", payload_type),
        }
    }

    /// Create a tagged optional
    pub fn tagged(payload_type: u64, tag_size: u64) -> Self {
        Self {
            payload_type,
            is_nullable_ptr: false,
            tag_size,
            mlir_type: format!("!chir.optional<{}, {}>", payload_type, tag_size),
        }
    }

    /// Check if this is a nullable pointer
    pub fn is_nullable_ptr(&self) -> bool {
        self.is_nullable_ptr
    }
}

/// Optional lowering context
#[derive(Debug, Clone)]
pub struct OptionalLowering {
    /// Tag size for non-pointer optionals
    tag_size: u64,
    /// Pointer width in bits
    pointer_width: u32,
}

impl OptionalLowering {
    /// Create a new optional lowering context
    pub fn new(tag_size: u64, pointer_width: u32) -> Self {
        Self {
            tag_size,
            pointer_width,
        }
    }

    /// Create an optional type
    pub fn create_optional_type(&self, inner_type_id: u64) -> ZigType {
        ZigType {
            id: 0,
            kind: ZigTypeKind::Optional {
                inner: inner_type_id,
            },
            size_bytes: (self.pointer_width / 8) as u64 + self.tag_size,
            align_bytes: (self.pointer_width / 8) as u64,
            source_loc: None,
        }
    }

    /// Check if a type should be lowered as nullable pointer
    pub fn is_nullable_ptr_type(&self, ty: &ZigType) -> bool {
        matches!(ty.kind, ZigTypeKind::Pointer)
    }

    /// Lower an optional type to representation
    pub fn lower_optional(&self, ty: &ZigType) -> LoweredOptional {
        if self.is_nullable_ptr_type(ty) {
            if let ZigTypeKind::Pointer = &ty.kind {
                return LoweredOptional::nullable_pointer(ty.id);
            }
        }
        LoweredOptional::tagged(ty.id, self.tag_size)
    }

    /// Get MLIR type for optional
    pub fn mlir_type_for(&self, inner_type: u64, is_nullable_ptr: bool) -> String {
        if is_nullable_ptr {
            format!("!chir.nullable_ptr<{}>", inner_type)
        } else {
            format!("!chir.optional<{}, {}>", inner_type, self.tag_size)
        }
    }

    /// Emit optional unwrap instruction
    pub fn emit_unwrap(&self, result: &str, optional: &str, inner_type: u64) -> String {
        format!(
            "{} = chimera.unwrap_optional %{} : <{}>",
            result, optional, inner_type
        )
    }

    /// Emit optional wrap instruction
    pub fn emit_wrap(&self, result: &str, payload: &str, inner_type: u64) -> String {
        format!(
            "{} = chimera.wrap_optional %{} : <{}>",
            result, payload, inner_type
        )
    }

    /// Emit null check
    pub fn emit_is_null(&self, result: &str, optional: &str) -> String {
        format!("{} = chimera.optional_is_null %{}", result, optional)
    }
}

/// Check if a type is optional
pub fn is_optional_type(ty: &ZigType) -> bool {
    matches!(ty.kind, ZigTypeKind::Optional { .. })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nullable_pointer_optional() {
        let opt = LoweredOptional::nullable_pointer(100);
        assert!(opt.is_nullable_ptr());
        assert_eq!(opt.payload_type, 100);
        assert!(opt.mlir_type.contains("nullable_ptr"));
    }

    #[test]
    fn test_tagged_optional() {
        let opt = LoweredOptional::tagged(100, 1);
        assert!(!opt.is_nullable_ptr());
        assert_eq!(opt.payload_type, 100);
        assert_eq!(opt.tag_size, 1);
        assert!(opt.mlir_type.contains("optional"));
    }

    #[test]
    fn test_optional_lowering_creation() {
        let lowering = OptionalLowering::new(1, 64);
        let opt_type = lowering.create_optional_type(100);
        assert!(matches!(
            opt_type.kind,
            ZigTypeKind::Optional { inner: 100 }
        ));
        assert_eq!(opt_type.size_bytes, 9); // ptr (8) + tag (1)
    }

    #[test]
    fn test_is_nullable_ptr_type() {
        let lowering = OptionalLowering::new(1, 64);
        let ptr_type = ZigType::pointer(64);
        assert!(lowering.is_nullable_ptr_type(&ptr_type));

        let int_type = ZigType::integer(32, true);
        assert!(!lowering.is_nullable_ptr_type(&int_type));
    }

    #[test]
    fn test_lower_optional_ptr() {
        let lowering = OptionalLowering::new(1, 64);
        let ptr_type = ZigType::pointer(64);
        let lowered = lowering.lower_optional(&ptr_type);
        assert!(lowered.is_nullable_ptr());
    }

    #[test]
    fn test_mlir_type_for_nullable_ptr() {
        let lowering = OptionalLowering::new(1, 64);
        let mlir_type = lowering.mlir_type_for(100, true);
        assert!(mlir_type.contains("nullable_ptr"));
    }

    #[test]
    fn test_mlir_type_for_tagged() {
        let lowering = OptionalLowering::new(1, 64);
        let mlir_type = lowering.mlir_type_for(100, false);
        assert!(mlir_type.contains("optional"));
    }

    #[test]
    fn test_emit_unwrap() {
        let lowering = OptionalLowering::new(1, 64);
        let inst = lowering.emit_unwrap("result", "opt_val", 100);
        assert!(inst.contains("unwrap_optional"));
        assert!(inst.contains("result"));
    }

    #[test]
    fn test_emit_wrap() {
        let lowering = OptionalLowering::new(1, 64);
        let inst = lowering.emit_wrap("result", "payload_val", 100);
        assert!(inst.contains("wrap_optional"));
    }

    #[test]
    fn test_emit_is_null() {
        let lowering = OptionalLowering::new(1, 64);
        let inst = lowering.emit_is_null("result", "opt_val");
        assert!(inst.contains("optional_is_null"));
    }

    #[test]
    fn test_is_optional_type() {
        let opt_type = ZigType {
            id: 1,
            kind: ZigTypeKind::Optional { inner: 100 },
            size_bytes: 9,
            align_bytes: 8,
            source_loc: None,
        };
        assert!(is_optional_type(&opt_type));

        let int_type = ZigType::integer(32, true);
        assert!(!is_optional_type(&int_type));
    }
}
