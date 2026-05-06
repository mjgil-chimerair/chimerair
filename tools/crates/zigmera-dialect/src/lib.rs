//! Zig dialect for AIR-to-dialect lowering.
//!
//! Provides semantic modeling for:
//! - Zig types (integers, floats, pointers, slices, arrays, structs, unions, enums)
//! - Zig control flow (branch, loop, switch, break/continue, return, try/catch)
//! - Zig memory operations (pointer deref, address-of, loads, stores)
//! - Error unions and error sets
//! - Generics and comptime
//!
//! Tasks 75, 76, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95: Base Zig dialect, AIR lowering, MLIR emission, primitives, slices, optionals, error unions, structs, ownership, effects, panic policy, generics

pub mod comprehensive_lowering;
pub mod control_flow;
pub mod effects;
pub mod error_handling;
pub mod error_unions;
pub mod generics;
pub mod memory;
pub mod mlir_emitter;
pub mod operations;
pub mod optionals;
pub mod ownership;
pub mod primitives;
pub mod slices;
pub mod structs;
pub mod types;
pub mod verifier;

pub use comprehensive_lowering::{
    ComptimeTracker, InstantiationTracker, PanicBoundaryRegistry, UnifiedLoweringContext,
};
pub use control_flow::{Block, BlockId, ControlFlowGraph};
pub use effects::{Effect, EffectLowering, PanicPolicy};
pub use error_handling::{ErrorSetModel, ErrorUnionModel};
pub use error_unions::ErrorUnionLowering;
pub use generics::{ComptimeModel, GenericModel};
pub use memory::{AddressSpace, MemoryModel, PointerModel};
pub use mlir_emitter::MlirEmitter;
pub use operations::{ZigInstruction, ZigOp};
pub use optionals::{LoweredOptional, OptionalLowering};
pub use ownership::{Lifetime, Ownership, OwnershipContext, OwnershipLowering};
pub use primitives::PrimitiveLowering;
pub use slices::{LoweredSlice, SliceLowering};
pub use structs::{LoweredEnum, LoweredStruct, LoweredUnion, StructLowering};
pub use types::{SourceLoc, ZigType, ZigTypeKind};
pub use verifier::{DialectVerifier, VerifierError};

use zigmera_diagnostics::{Diag, DiagBag};

/// A Zig dialect module containing lowered AIR
#[derive(Debug, Clone)]
pub struct DialectModule {
    /// Module name
    pub name: String,
    /// Source file path
    pub source_path: String,
    /// Type definitions
    pub types: Vec<ZigType>,
    /// Functions in this module
    pub functions: Vec<DialectFunction>,
    /// External function imports (extern declarations)
    pub extern_fns: Vec<ExternFn>,
    /// Diagnostics from lowering
    diags: DiagBag,
}

/// External function import (Task 22)
#[derive(Debug, Clone)]
pub struct ExternFn {
    pub name: String,
    pub abi: String,
    pub params: Vec<u64>,
    pub return_type: Option<u64>,
}

impl DialectModule {
    /// Create a new dialect module
    pub fn new(name: String, source_path: String) -> Self {
        Self {
            name,
            source_path,
            types: Vec::new(),
            functions: Vec::new(),
            extern_fns: Vec::new(),
            diags: DiagBag::new(),
        }
    }

    /// Add a function to this module
    pub fn add_function(&mut self, func: DialectFunction) {
        self.functions.push(func);
    }

    /// Add a type to this module
    pub fn add_type(&mut self, ty: ZigType) {
        self.types.push(ty);
    }

    /// Add an extern function import (Task 22)
    pub fn add_extern_fn(&mut self, ext_fn: ExternFn) {
        self.extern_fns.push(ext_fn);
    }

    /// Add a diagnostic
    pub fn add_diag(&mut self, diag: Diag) {
        self.diags.push(diag);
    }

    /// Check if module has errors
    pub fn has_errors(&self) -> bool {
        self.diags.has_errors()
    }

    /// Get diagnostics
    pub fn diagnostics(&self) -> &DiagBag {
        &self.diags
    }

    /// Add an unsupported diagnostic for async/frame features
    pub fn add_unsupported_feature(&mut self, feature: &str, location: Option<SourceLoc>) {
        let mut diag = Diag::new(zigmera_diagnostics::DiagCode::LoweringAsyncNotSupported);
        diag.message = format!("{} feature is not yet supported", feature);
        if let Some(loc) = location {
            diag.file = Some(format!("file_{}", loc.file_id));
            diag.line = Some(loc.line);
            diag.column = Some(loc.column);
        }
        self.add_diag(diag);
    }
}

/// A lowered Zig function
#[derive(Debug, Clone)]
pub struct DialectFunction {
    pub id: u64,
    pub name: String,
    pub type_id: u64,
    pub blocks: Vec<Block>,
    pub is_exported: bool,
    pub callconv: String,
    pub params: Vec<u64>,
    pub return_type: Option<u64>,
}

impl DialectFunction {
    pub fn new(name: String, id: u64, type_id: u64) -> Self {
        Self {
            id,
            name,
            type_id,
            blocks: Vec::new(),
            is_exported: false,
            callconv: "C".to_string(),
            params: Vec::new(),
            return_type: None,
        }
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn set_export(&mut self, exported: bool) {
        self.is_exported = exported;
    }

    pub fn set_callconv(&mut self, callconv: &str) {
        self.callconv = callconv.to_string();
    }

    /// Check if function has any unsupported operations
    pub fn check_unsupported(&self) -> Vec<ZigOp> {
        self.blocks
            .iter()
            .flat_map(|b| b.instructions.iter())
            .filter(|i| !i.op.is_supported())
            .map(|i| i.op.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialect_module_creation() {
        let module = DialectModule::new("test".to_string(), "test.zig".to_string());
        assert_eq!(module.name, "test");
        assert!(!module.has_errors());
        assert!(module.functions.is_empty());
    }

    #[test]
    fn test_dialect_function_creation() {
        let func = DialectFunction::new("add".to_string(), 1, 100);
        assert_eq!(func.name, "add");
        assert_eq!(func.id, 1);
        assert!(!func.is_exported);
    }

    #[test]
    fn test_block_creation() {
        let block = Block::new(0);
        assert_eq!(block.id, 0);
        assert!(block.instructions.is_empty());
    }

    #[test]
    fn test_zig_type_integer() {
        let ty = ZigType::integer(32, true);
        assert!(matches!(ty.kind, ZigTypeKind::Int { .. }));
        assert_eq!(ty.sizeof(), 4);
    }

    #[test]
    fn test_zig_type_pointer() {
        let ty = ZigType::pointer(64);
        assert!(matches!(ty.kind, ZigTypeKind::Pointer));
        assert_eq!(ty.sizeof(), 8);
    }

    #[test]
    fn test_zig_op_from_air() {
        let op = ZigOp::from_air_str("add");
        assert!(matches!(op, ZigOp::Add));

        let op = ZigOp::from_air_str("load");
        assert!(matches!(op, ZigOp::Load));

        let op = ZigOp::from_air_str("br");
        assert!(matches!(op, ZigOp::Br));

        let op = ZigOp::from_air_str("unknown");
        assert!(matches!(op, ZigOp::Unknown));
    }

    #[test]
    fn test_unsupported_feature_diagnostic() {
        let mut module = DialectModule::new("test".to_string(), "test.zig".to_string());
        module.add_unsupported_feature(
            "async",
            Some(SourceLoc {
                file_id: 1,
                line: 10,
                column: 5,
            }),
        );
        assert!(module.has_errors());
        let diags = module.diagnostics();
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_dialect_function_check_unsupported() {
        let mut func = DialectFunction::new("test".to_string(), 1, 0);
        let mut block = Block::new(0);
        block.add_instruction(ZigInstruction::new(0, ZigOp::Await));
        block.add_instruction(ZigInstruction::new(1, ZigOp::Add));
        func.add_block(block);

        let unsupported = func.check_unsupported();
        assert_eq!(unsupported.len(), 1);
        assert!(matches!(unsupported[0], ZigOp::Await));
    }
}
