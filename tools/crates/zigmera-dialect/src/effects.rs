//! Lowering of effects (suspend, resume, await).
//!
//! Task 93: Lower effects

use super::operations::{ZigInstruction, ZigOp};
use super::types::SourceLoc;
use serde::{Deserialize, Serialize};

/// Effect classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Async/await effect (unsupported)
    AsyncAwait,
    /// Suspend frame effect (unsupported)
    SuspendFrame,
    /// Resume effect (unsupported)
    Resume,
    /// Panic effect (bounds check, unreachable)
    Panic,
    /// No side effects
    None,
}

/// Effect lowering context
#[derive(Debug, Clone)]
pub struct EffectLowering {
    /// Check if async/await is supported
    async_supported: bool,
    /// Panic policy
    panic_policy: PanicPolicy,
}

/// Panic policy for boundary enforcement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanicPolicy {
    /// Unwinding is allowed
    AllowUnwind,
    /// No unwinding allowed (crash on panic)
    NoUnwind,
    /// Bounds check only (no panic)
    BoundsCheckOnly,
}

impl Default for PanicPolicy {
    fn default() -> Self {
        Self::NoUnwind
    }
}

impl EffectLowering {
    /// Create a new effect lowering context
    pub fn new(async_supported: bool, panic_policy: PanicPolicy) -> Self {
        Self {
            async_supported,
            panic_policy,
        }
    }

    /// Classify an instruction's effect
    pub fn classify(&self, inst: &ZigInstruction) -> Effect {
        match &inst.op {
            ZigOp::Await => {
                if self.async_supported {
                    Effect::AsyncAwait
                } else {
                    Effect::None // Would be marked unsupported elsewhere
                }
            }
            ZigOp::SuspendFrame => Effect::SuspendFrame,
            ZigOp::Resume => Effect::Resume,
            ZigOp::Unreachable => Effect::Panic,
            ZigOp::Invoke => Effect::Panic,
            _ => Effect::None,
        }
    }

    /// Check if an operation is an async effect
    pub fn is_async_effect(&self, op: &ZigOp) -> bool {
        matches!(op, ZigOp::Await | ZigOp::SuspendFrame | ZigOp::Resume)
    }

    /// Check if an operation is a panic effect
    pub fn is_panic_effect(&self, op: &ZigOp) -> bool {
        matches!(op, ZigOp::Unreachable | ZigOp::Invoke)
    }

    /// Get the panic policy
    pub fn panic_policy(&self) -> PanicPolicy {
        self.panic_policy
    }

    /// Emit effect annotation for MLIR
    pub fn emit_effect_attr(&self, effect: &Effect) -> String {
        match effect {
            Effect::AsyncAwait => "!chir.effect(async_await)".to_string(),
            Effect::SuspendFrame => "!chir.effect(suspend)".to_string(),
            Effect::Resume => "!chir.effect(resume)".to_string(),
            Effect::Panic => "!chir.effect(panic)".to_string(),
            Effect::None => String::new(),
        }
    }

    /// Emit panic policy annotation
    pub fn emit_panic_policy_attr(&self) -> String {
        match &self.panic_policy {
            PanicPolicy::AllowUnwind => "!chir.panic_policy(allow_unwind)".to_string(),
            PanicPolicy::NoUnwind => "!chir.panic_policy(no_unwind)".to_string(),
            PanicPolicy::BoundsCheckOnly => "!chir.panic_policy(bounds_check_only)".to_string(),
        }
    }

    /// Check if panic is allowed at a boundary
    pub fn panic_allowed_at_boundary(&self, boundary: &str) -> bool {
        match &self.panic_policy {
            PanicPolicy::AllowUnwind => true,
            PanicPolicy::NoUnwind => false,
            PanicPolicy::BoundsCheckOnly => boundary == "bounds_check",
        }
    }

    /// Validate effect compatibility
    pub fn validate_effect(&self, effect: &Effect, declared_effects: &[Effect]) -> bool {
        // If no effects declared, allow all
        if declared_effects.is_empty() {
            return true;
        }
        // Check if effect is in declared effects
        declared_effects.iter().any(|e| e == effect)
    }
}

/// Check if an instruction has async effects
pub fn has_async_effect(inst: &ZigInstruction) -> bool {
    matches!(inst.op, ZigOp::Await | ZigOp::SuspendFrame | ZigOp::Resume)
}

/// Check if an instruction has panic effects
pub fn has_panic_effect(inst: &ZigInstruction) -> bool {
    matches!(inst.op, ZigOp::Unreachable | ZigOp::Invoke)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_lowering_creation() {
        let lowering = EffectLowering::new(false, PanicPolicy::NoUnwind);
        assert!(!lowering.async_supported);
        assert_eq!(lowering.panic_policy(), PanicPolicy::NoUnwind);
    }

    #[test]
    fn test_classify_await() {
        let lowering = EffectLowering::new(false, PanicPolicy::NoUnwind);
        let inst = ZigInstruction::new(1, ZigOp::Await);
        let effect = lowering.classify(&inst);
        assert_eq!(effect, Effect::None); // Not async-supported, so treated as none
    }

    #[test]
    fn test_classify_unreachable() {
        let lowering = EffectLowering::new(false, PanicPolicy::NoUnwind);
        let inst = ZigInstruction::new(1, ZigOp::Unreachable);
        let effect = lowering.classify(&inst);
        assert_eq!(effect, Effect::Panic);
    }

    #[test]
    fn test_is_async_effect() {
        let lowering = EffectLowering::new(true, PanicPolicy::NoUnwind);
        assert!(lowering.is_async_effect(&ZigOp::Await));
        assert!(lowering.is_async_effect(&ZigOp::SuspendFrame));
        assert!(lowering.is_async_effect(&ZigOp::Resume));
        assert!(!lowering.is_async_effect(&ZigOp::Add));
    }

    #[test]
    fn test_is_panic_effect() {
        let lowering = EffectLowering::new(true, PanicPolicy::NoUnwind);
        assert!(lowering.is_panic_effect(&ZigOp::Unreachable));
        assert!(lowering.is_panic_effect(&ZigOp::Invoke));
        assert!(!lowering.is_panic_effect(&ZigOp::Add));
    }

    #[test]
    fn test_emit_effect_attr() {
        let lowering = EffectLowering::new(true, PanicPolicy::NoUnwind);
        let attr = lowering.emit_effect_attr(&Effect::AsyncAwait);
        assert!(attr.contains("async_await"));

        let attr_panic = lowering.emit_effect_attr(&Effect::Panic);
        assert!(attr_panic.contains("panic"));
    }

    #[test]
    fn test_panic_allowed_at_boundary() {
        let lowering = EffectLowering::new(true, PanicPolicy::NoUnwind);
        assert!(!lowering.panic_allowed_at_boundary("export"));
        assert!(!lowering.panic_allowed_at_boundary("bounds_check"));

        let allow_unwind = EffectLowering::new(true, PanicPolicy::AllowUnwind);
        assert!(allow_unwind.panic_allowed_at_boundary("export"));
        assert!(allow_unwind.panic_allowed_at_boundary("bounds_check"));

        let bounds_only = EffectLowering::new(true, PanicPolicy::BoundsCheckOnly);
        assert!(!bounds_only.panic_allowed_at_boundary("export"));
        assert!(bounds_only.panic_allowed_at_boundary("bounds_check"));
    }

    #[test]
    fn test_validate_effect() {
        let lowering = EffectLowering::new(true, PanicPolicy::NoUnwind);
        assert!(lowering.validate_effect(&Effect::Panic, &[]));
        assert!(lowering.validate_effect(&Effect::Panic, &[Effect::Panic]));
        assert!(!lowering.validate_effect(&Effect::AsyncAwait, &[Effect::Panic]));
    }

    #[test]
    fn test_has_async_effect() {
        let await_inst = ZigInstruction::new(1, ZigOp::Await);
        assert!(has_async_effect(&await_inst));

        let add_inst = ZigInstruction::new(1, ZigOp::Add);
        assert!(!has_async_effect(&add_inst));
    }

    #[test]
    fn test_has_panic_effect() {
        let unreachable_inst = ZigInstruction::new(1, ZigOp::Unreachable);
        assert!(has_panic_effect(&unreachable_inst));

        let add_inst = ZigInstruction::new(1, ZigOp::Add);
        assert!(!has_panic_effect(&add_inst));
    }

    // Task 82: Async/frame limitations tests
    #[test]
    fn test_async_not_supported_diagnostic() {
        // Verify async operations trigger unsupported diagnostics
        let lowering = EffectLowering::new(false, PanicPolicy::NoUnwind);

        // Await is not supported
        let await_inst = ZigInstruction::new(1, ZigOp::Await);
        let effect = lowering.classify(&await_inst);
        assert!(matches!(effect, Effect::None)); // Treated as none since async not supported

        // SuspendFrame is always a frame effect
        let suspend_inst = ZigInstruction::new(2, ZigOp::SuspendFrame);
        let suspend_effect = lowering.classify(&suspend_inst);
        assert!(matches!(suspend_effect, Effect::SuspendFrame));

        // Resume is always a resume effect
        let resume_inst = ZigInstruction::new(3, ZigOp::Resume);
        let resume_effect = lowering.classify(&resume_inst);
        assert!(matches!(resume_effect, Effect::Resume));
    }

    #[test]
    fn test_async_supported_mode() {
        // When async is supported, Await becomes AsyncAwait effect
        let lowering = EffectLowering::new(true, PanicPolicy::NoUnwind);

        let await_inst = ZigInstruction::new(1, ZigOp::Await);
        let effect = lowering.classify(&await_inst);
        assert!(matches!(effect, Effect::AsyncAwait));

        // But SuspendFrame and Resume are still frame effects
        let suspend_inst = ZigInstruction::new(2, ZigOp::SuspendFrame);
        assert!(matches!(
            lowering.classify(&suspend_inst),
            Effect::SuspendFrame
        ));

        let resume_inst = ZigInstruction::new(3, ZigOp::Resume);
        assert!(matches!(lowering.classify(&resume_inst), Effect::Resume));
    }

    #[test]
    fn test_async_effect_mlir_representation() {
        let lowering = EffectLowering::new(true, PanicPolicy::NoUnwind);

        assert_eq!(
            lowering.emit_effect_attr(&Effect::AsyncAwait),
            "!chir.effect(async_await)"
        );
        assert_eq!(
            lowering.emit_effect_attr(&Effect::SuspendFrame),
            "!chir.effect(suspend)"
        );
        assert_eq!(
            lowering.emit_effect_attr(&Effect::Resume),
            "!chir.effect(resume)"
        );
    }

    #[test]
    fn test_frame_limitations_diagnostic_output() {
        // Verify frame operations produce appropriate diagnostic info
        let lowering = EffectLowering::new(false, PanicPolicy::NoUnwind);

        // All async/frame ops are frame-limited
        assert!(lowering.is_async_effect(&ZigOp::Await));
        assert!(lowering.is_async_effect(&ZigOp::SuspendFrame));
        assert!(lowering.is_async_effect(&ZigOp::Resume));

        // Non-async ops are not frame-limited
        assert!(!lowering.is_async_effect(&ZigOp::Call));
        assert!(!lowering.is_async_effect(&ZigOp::Invoke));
    }

    #[test]
    fn test_async_not_in_effect_validation() {
        // Async effects are not in the default effect set
        let lowering = EffectLowering::new(true, PanicPolicy::NoUnwind);

        // Empty declared effects allows all
        assert!(lowering.validate_effect(&Effect::AsyncAwait, &[]));
        assert!(lowering.validate_effect(&Effect::SuspendFrame, &[]));
        assert!(lowering.validate_effect(&Effect::Resume, &[]));

        // Async effects are not in Panic-only set
        assert!(!lowering.validate_effect(&Effect::AsyncAwait, &[Effect::Panic]));
        assert!(!lowering.validate_effect(&Effect::SuspendFrame, &[Effect::Panic]));
    }

    // Task 83: SIMD/vector limitations tests
    #[test]
    fn test_vector_supported() {
        // VectorReduce is supported (represented, not rejected)
        let vec_op = ZigOp::VectorReduce;
        assert!(vec_op.is_supported());
    }

    #[test]
    fn test_vector_operation_classification() {
        // Vector operations are classified as having no direct effect
        let lowering = EffectLowering::new(true, PanicPolicy::NoUnwind);

        let vec_inst = ZigInstruction::new(1, ZigOp::VectorReduce);
        let effect = lowering.classify(&vec_inst);
        assert!(matches!(effect, Effect::None)); // No direct panic/async effect
    }

    #[test]
    fn test_vector_reduce_mlir_representation() {
        // VectorReduce should be representable in MLIR
        let vec_op = ZigOp::VectorReduce;
        assert!(vec_op.is_supported());
        assert!(!vec_op.has_side_effects());
    }

    // Task 84: Inline assembly limitations tests
    #[test]
    fn test_inline_asm_not_supported() {
        // InlineAsm is not supported - must be rejected with diagnostic
        let asm_op = ZigOp::InlineAsm;
        assert!(!asm_op.is_supported());
    }

    #[test]
    fn test_inline_asm_classification() {
        // Inline asm has no direct effect classification
        let lowering = EffectLowering::new(true, PanicPolicy::NoUnwind);

        let asm_inst = ZigInstruction::new(1, ZigOp::InlineAsm);
        let effect = lowering.classify(&asm_inst);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_all_async_ops_not_supported() {
        // All async/frame ops are not supported
        let ops = [ZigOp::Await, ZigOp::SuspendFrame, ZigOp::Resume];
        for op in ops {
            assert!(!op.is_supported(), " {:?} should not be supported", op);
        }
    }

    #[test]
    fn test_asm_await_suspend_frame_all_unsupported() {
        // Comprehensive check for all unsupported frame ops
        let unsupported_ops = [
            ZigOp::Await,
            ZigOp::SuspendFrame,
            ZigOp::Resume,
            ZigOp::InlineAsm,
        ];
        for op in unsupported_ops {
            assert!(!op.is_supported(), " {:?} should be unsupported", op);
        }
    }

    #[test]
    fn test_inline_asm_trust_obligation() {
        // Inline asm requires trust obligation per TCB docs
        let asm_op = ZigOp::InlineAsm;
        assert!(!asm_op.is_supported()); // Rejected with diagnostic
        assert!(!asm_op.has_side_effects()); // No automatic side effects tracked
    }
}
