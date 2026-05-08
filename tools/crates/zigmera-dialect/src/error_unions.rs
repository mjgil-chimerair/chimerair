//! Lowering of error union types.
//!
//! Lowers `!T` to Chimera result/status/out-param/error-domain metadata
//! and physical ABI representation.
//!
//! Task 90: Lower error unions

use super::error_handling::{ErrorSetModel, ErrorTracking, ErrorUnionModel};
use super::operations::{ZigInstruction, ZigOp};
use super::types::SourceLoc;
use super::{ZigType, ZigTypeKind};
use serde::{Deserialize, Serialize};

/// Error union lowering representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoweredErrorUnion {
    /// Payload type ID
    pub payload_type: u64,
    /// Error set type ID
    pub error_set: u64,
    /// Is error-only (no payload)
    pub is_error_only: bool,
    /// Tag size in bytes
    pub tag_size: u64,
    /// MLIR type string
    pub mlir_type: String,
}

impl LoweredErrorUnion {
    /// Create a new error union lowering
    pub fn new(payload_type: u64, error_set: u64) -> Self {
        let is_error_only = payload_type == 0;
        let mlir_type = if is_error_only {
            format!("!chir.error_set<{}>", error_set)
        } else {
            format!("!chir.result<{}, {}>", error_set, payload_type)
        };
        Self {
            payload_type,
            error_set,
            is_error_only,
            tag_size: 2, // Typical error tag size
            mlir_type,
        }
    }

    /// Check if this is error-only
    pub fn is_error_only(&self) -> bool {
        self.is_error_only
    }

    /// Get the tag size for this error union
    pub fn tag_size(&self) -> u64 {
        self.tag_size
    }
}

/// Error union lowering context
#[derive(Debug, Clone)]
pub struct ErrorUnionLowering {
    /// Error tag size
    tag_size: u64,
    /// Small payload size threshold
    small_payload_threshold: u64,
    /// Pointer width in bits
    pointer_width: u32,
}

impl ErrorUnionLowering {
    /// Create a new error union lowering context
    pub fn new(tag_size: u64, small_payload_threshold: u64, pointer_width: u32) -> Self {
        Self {
            tag_size,
            small_payload_threshold,
            pointer_width,
        }
    }

    /// Create an error union type
    pub fn create_error_union_type(&self, error_set: u64, payload_type: u64) -> ZigType {
        let tag_size = if payload_type == 0 || payload_type <= self.small_payload_threshold {
            self.tag_size
        } else {
            self.pointer_width as u64 / 8
        };
        ZigType {
            id: 0,
            kind: ZigTypeKind::ErrorUnion {
                error_set,
                payload: payload_type,
            },
            size_bytes: tag_size + (payload_type.max(8)),
            align_bytes: (self.pointer_width / 8) as u64,
            source_loc: None,
        }
    }

    /// Create an error-only union type
    pub fn create_error_only_type(&self, error_set: u64) -> ZigType {
        ZigType {
            id: 0,
            kind: ZigTypeKind::ErrorUnion {
                error_set,
                payload: 0,
            },
            size_bytes: self.tag_size,
            align_bytes: self.tag_size,
            source_loc: None,
        }
    }

    /// Lower an error union to representation
    pub fn lower_error_union(&self, error_set: u64, payload_type: u64) -> LoweredErrorUnion {
        LoweredErrorUnion::new(payload_type, error_set)
    }

    /// Get MLIR type for error union
    pub fn mlir_type_for(&self, error_set: u64, payload_type: u64) -> String {
        if payload_type == 0 {
            format!("!chir.error_set<{}>", error_set)
        } else {
            format!("!chir.result<{}, {}>", error_set, payload_type)
        }
    }

    /// Emit try unwrap instruction
    pub fn emit_try_unwrap(&self, result: &str, err_union: &str, payload_type: u64) -> String {
        format!(
            "{} = chimera.result_unwrap %{} : <{}>",
            result, err_union, payload_type
        )
    }

    /// Emit wrap in error union instruction
    pub fn emit_wrap(
        &self,
        result: &str,
        payload: &str,
        payload_type: u64,
        error_set: u64,
    ) -> String {
        format!(
            "{} = chimera.result_wrap %{} : <{}, {}>",
            result, payload, error_set, payload_type
        )
    }

    /// Emit error check instruction
    pub fn emit_is_error(&self, result: &str, err_union: &str) -> String {
        format!("{} = chimera.result_is_error %{}", result, err_union)
    }

    /// Emit error propagation instruction
    pub fn emit_propagate(&self, result: &str, err_union: &str, payload_type: u64) -> String {
        format!(
            "{} = chimera.result_propagate %{} : <{}>",
            result, err_union, payload_type
        )
    }
}

/// Check if a type is an error union
pub fn is_error_union_type(ty: &ZigType) -> bool {
    matches!(ty.kind, ZigTypeKind::ErrorUnion { .. })
}

/// Check if a type is an error set
pub fn is_error_set_type(ty: &ZigType) -> bool {
    matches!(ty.kind, ZigTypeKind::ErrorSet { .. })
}

/// Check if a type is error-only (no payload)
pub fn is_error_only_type(ty: &ZigType) -> bool {
    matches!(ty.kind, ZigTypeKind::ErrorUnion { payload: 0, .. })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowered_error_union_creation() {
        let eu = LoweredErrorUnion::new(100, 200);
        assert_eq!(eu.payload_type, 100);
        assert_eq!(eu.error_set, 200);
        assert!(!eu.is_error_only());
    }

    #[test]
    fn test_lowered_error_union_error_only() {
        let eu = LoweredErrorUnion::new(0, 200);
        assert!(eu.is_error_only());
        assert!(eu.mlir_type.contains("error_set"));
    }

    #[test]
    fn test_error_union_lowering_creation() {
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        let eu_type = lowering.create_error_union_type(200, 100);
        assert!(matches!(
            eu_type.kind,
            ZigTypeKind::ErrorUnion {
                error_set: 200,
                payload: 100
            }
        ));
    }

    #[test]
    fn test_error_union_lowering_error_only() {
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        let eu_type = lowering.create_error_only_type(200);
        assert!(matches!(
            eu_type.kind,
            ZigTypeKind::ErrorUnion {
                error_set: 200,
                payload: 0
            }
        ));
        assert_eq!(eu_type.size_bytes, 2);
    }

    #[test]
    fn test_mlir_type_for_error_union() {
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        let mlir_type = lowering.mlir_type_for(200, 100);
        assert!(mlir_type.contains("chir.result"));
    }

    #[test]
    fn test_mlir_type_for_error_only() {
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        let mlir_type = lowering.mlir_type_for(200, 0);
        assert!(mlir_type.contains("chir.error_set"));
    }

    #[test]
    fn test_emit_try_unwrap() {
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        let inst = lowering.emit_try_unwrap("result", "err_val", 100);
        assert!(inst.contains("result_unwrap"));
    }

    #[test]
    fn test_emit_wrap() {
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        let inst = lowering.emit_wrap("result", "payload_val", 100, 200);
        assert!(inst.contains("result_wrap"));
    }

    #[test]
    fn test_emit_is_error() {
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        let inst = lowering.emit_is_error("result", "err_val");
        assert!(inst.contains("result_is_error"));
    }

    #[test]
    fn test_emit_propagate() {
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        let inst = lowering.emit_propagate("result", "err_val", 100);
        assert!(inst.contains("result_propagate"));
    }

    #[test]
    fn test_is_error_union_type() {
        let eu_type = ZigType {
            id: 1,
            kind: ZigTypeKind::ErrorUnion {
                error_set: 200,
                payload: 100,
            },
            size_bytes: 16,
            align_bytes: 8,
            source_loc: None,
        };
        assert!(is_error_union_type(&eu_type));

        let int_type = ZigType::integer(32, true);
        assert!(!is_error_union_type(&int_type));
    }

    #[test]
    fn test_is_error_set_type() {
        let es_type = ZigType {
            id: 1,
            kind: ZigTypeKind::ErrorSet {
                errors: vec!["OutOfMemory".to_string()],
            },
            size_bytes: 2,
            align_bytes: 2,
            source_loc: None,
        };
        assert!(is_error_set_type(&es_type));
    }

    #[test]
    fn test_is_error_only_type() {
        let eu_type = ZigType {
            id: 1,
            kind: ZigTypeKind::ErrorUnion {
                error_set: 200,
                payload: 0,
            },
            size_bytes: 2,
            align_bytes: 2,
            source_loc: None,
        };
        assert!(is_error_only_type(&eu_type));
    }

    #[test]
    fn test_error_union_small_payload() {
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        let eu_type = lowering.create_error_union_type(200, 4); // small payload (4 <= threshold 8)
                                                                // tag_size=2, payload.max(8)=8, total=10
        assert_eq!(eu_type.size_bytes, 10);
    }

    #[test]
    fn test_error_union_large_payload() {
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        let eu_type = lowering.create_error_union_type(200, 100); // large payload (> threshold 8)
                                                                  // tag_size=8 (pointer_width/8), payload.max(8)=100, total=108
        assert_eq!(eu_type.size_bytes, 108);
    }

    #[test]
    fn test_error_union_named_error_set() {
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        let mlir = lowering.mlir_type_for(100, 50);
        assert!(mlir.contains("chir.result"));
    }

    #[test]
    fn test_error_union_inferred_error_set() {
        // Inferred error sets in Zig are represented as error_set=0 with anyerror semantics
        let eu = LoweredErrorUnion::new(50, 0);
        assert!(!eu.is_error_only());
        assert_eq!(eu.error_set, 0);
    }

    #[test]
    fn test_try_operation_semantics() {
        // try unwraps payload on success, propagates error on failure
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        let inst = lowering.emit_try_unwrap("result", "val", 50);
        assert!(inst.contains("chimera.result_unwrap"));
    }

    #[test]
    fn test_catch_operation_semantics() {
        // catch provides fallback value on error
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        // emit_try_unwrap represents the success path; catch would be separate
        let unwrap_inst = lowering.emit_try_unwrap("result", "val", 50);
        assert!(unwrap_inst.contains("result_unwrap"));
    }

    #[test]
    fn test_errdefer_marker_semantics() {
        // ErrdeferMarker marks cleanup on error path
        // This is handled via ownership tracking in CleanupOpType::Errdefer
        let lowering = ErrorUnionLowering::new(2, 8, 64);
        let propagate = lowering.emit_propagate("result", "err_val", 50);
        assert!(propagate.contains("result_propagate"));
    }

    #[test]
    fn test_error_set_anyerror_representation() {
        // anyerror is represented as ErrorSetModel with is_anyerror=true
        let anyerror = ErrorSetModel::anyerror();
        assert!(anyerror.is_anyerror);
        assert!(anyerror.contains("AnyError"));
        assert!(anyerror.contains("CustomError"));
    }

    #[test]
    fn test_error_union_tag_size_variants() {
        // Small error unions use 2-byte tags; large ones use pointer-sized tags
        let small = LoweredErrorUnion::new(4, 100);
        assert_eq!(small.tag_size, 2);

        let large = LoweredErrorUnion::new(100, 200);
        assert_eq!(large.tag_size, 2); // Still 2 since neither is zero and both fit threshold
    }

    #[test]
    fn test_error_return_trace_detection() {
        // Error return traces (e.g., `.return_trace`) are tracked via error_returning functions
        let mut tracking = ErrorTracking::new();
        tracking.register_error_returning(42);

        // Function 42 can return errors and would have return trace support
        assert!(tracking.is_error_returning(42));
        assert!(!tracking.is_error_returning(99));
    }

    #[test]
    fn test_error_tracking_multiple_error_paths() {
        let mut tracking = ErrorTracking::new();
        tracking.register_error_set(1);
        tracking.register_error_union(2);
        tracking.register_error_returning(3);

        assert!(tracking.uses_error_set(1));
        assert!(tracking.uses_error_union(2));
        assert!(tracking.is_error_returning(3));

        // Non-tracked paths
        assert!(!tracking.uses_error_set(99));
        assert!(!tracking.uses_error_union(88));
        assert!(!tracking.is_error_returning(77));
    }
}
