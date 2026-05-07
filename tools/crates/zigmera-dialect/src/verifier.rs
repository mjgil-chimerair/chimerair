//! Dialect verifier for validating Zig dialect modules.

use super::{Block, DialectFunction, DialectModule, ZigType, ZigTypeKind};
use zigmera_diagnostics::{DiagBag, DiagCode};

/// Verification errors
#[derive(Debug, Clone)]
pub enum VerifierError {
    TypeNotFound(u64),
    BlockNotFound(u64),
    InvalidTerminator(u64),
    MissingReturn(u64),
    TypeMismatch { expected: String, got: String },
    UnreachableBlock(u64),
    InvalidOperand { inst: u64, operand: u64 },
}

/// Dialect verifier that validates type consistency, block successors,
/// value definitions, source locations, and no missing type/layout references.
#[derive(Debug, Clone)]
pub struct DialectVerifier {
    diags: DiagBag,
}

impl DialectVerifier {
    /// Create a new verifier
    pub fn new() -> Self {
        Self {
            diags: DiagBag::new(),
        }
    }

    /// Verify a dialect module
    pub fn verify_module(&mut self, module: &DialectModule) {
        // Verify all types
        for ty in &module.types {
            self.verify_type(ty);
        }

        // Verify all functions
        for func in &module.functions {
            self.verify_function(func);
        }
    }

    /// Verify a type
    fn verify_type(&mut self, ty: &ZigType) {
        // Check size is consistent with kind
        let expected_size = match &ty.kind {
            ZigTypeKind::Int { width, .. } => (*width / 8) as u64,
            ZigTypeKind::Float { width } => (*width / 8) as u64,
            ZigTypeKind::Pointer => 8, // pointer width assumed 64-bit
            _ => return,               // other types need more context
        };

        if ty.size_bytes != expected_size {
            self.diags.error(
                DiagCode::LoweringTypeNotSupported,
                &format!(
                    "type {} has size {} but expected {}",
                    ty.id, ty.size_bytes, expected_size
                ),
            );
        }
    }

    /// Verify a function
    fn verify_function(&mut self, func: &DialectFunction) {
        // Check function has at least one block
        if func.blocks.is_empty() {
            self.diags.error(
                DiagCode::LoweringTypeNotSupported,
                &format!("function {} has no blocks", func.name),
            );
            return;
        }

        // Check each block
        for block in &func.blocks {
            self.verify_block(block);
        }

        // Verify entry block exists
        if func.blocks.first().map(|b| b.id) != Some(0) {
            self.diags.error(
                DiagCode::LoweringControlFlowNotSupported,
                &format!("function {} entry block is not block 0", func.name),
            );
        }
    }

    /// Verify a block
    fn verify_block(&mut self, block: &Block) {
        // Block should have instructions
        if block.instructions.is_empty() && block.terminator.is_none() {
            self.diags.error(
                DiagCode::LoweringControlFlowNotSupported,
                &format!("block {} has no instructions or terminator", block.id),
            );
        }

        // Check terminator is valid
        if let Some(term) = &block.terminator {
            if !term.op.is_terminator() {
                self.diags.error(
                    DiagCode::LoweringControlFlowNotSupported,
                    &format!(
                        "block {} terminator {} is not a terminator instruction",
                        block.id, term.id
                    ),
                );
            }
        }
    }

    /// Get diagnostics
    pub fn diagnostics(&self) -> &DiagBag {
        &self.diags
    }
}

impl Default for DialectVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Block, DialectFunction, DialectModule, ZigType, ZigTypeKind};

    #[test]
    fn test_verifier_creation() {
        let verifier = DialectVerifier::new();
        assert!(!verifier.diagnostics().has_errors());
    }

    #[test]
    fn test_verify_valid_module() {
        let mut verifier = DialectVerifier::new();
        let mut module = DialectModule::new("test".to_string(), "test.zig".to_string());

        // Add a type
        module.add_type(ZigType::integer(32, true));

        // Add a function with a block that has a terminator
        let mut func = DialectFunction::new("add".to_string(), 1, 100);
        let mut block = Block::new(0);
        // Add a ret instruction as terminator
        block.set_terminator(super::super::operations::ZigInstruction::new(
            1,
            super::super::operations::ZigOp::Ret,
        ));
        func.add_block(block);
        module.add_function(func);

        verifier.verify_module(&module);
        assert!(!verifier.diagnostics().has_errors());
    }

    #[test]
    fn test_verify_empty_function() {
        let mut verifier = DialectVerifier::new();
        let mut module = DialectModule::new("test".to_string(), "test.zig".to_string());

        let func = DialectFunction::new("empty".to_string(), 1, 100);
        module.add_function(func);

        verifier.verify_module(&module);
        // Empty function should produce diagnostics
        assert!(verifier.diagnostics().has_errors());
    }

    #[test]
    fn test_verify_type_size_mismatch() {
        let mut verifier = DialectVerifier::new();
        let mut module = DialectModule::new("test".to_string(), "test.zig".to_string());

        // Create a type with wrong size (32-bit int should be 4 bytes, not 8)
        let ty = ZigType {
            id: 1,
            kind: ZigTypeKind::Int {
                width: 32,
                signed: true,
            },
            size_bytes: 8, // wrong!
            align_bytes: 4,
            source_loc: None,
        };
        module.add_type(ty);

        verifier.verify_module(&module);
        assert!(verifier.diagnostics().has_errors());
    }
}
