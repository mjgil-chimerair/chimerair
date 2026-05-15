-- ChimeraProof Tests: IR and Metadata Tests
-- Compile-safe theorem smoke tests for IR and metadata modules.

import Chimera.Foundation
import Chimera.ABI
import Chimera.IR
import Chimera.IR.Module
import Chimera.Checkers.MetadataChecker

namespace Chimera.Test

namespace ModuleTest

theorem module_smoke : True := by
  trivial

end ModuleTest

namespace MetadataCheckerTest

theorem metadata_checker_smoke : True := by
  trivial

end MetadataCheckerTest

namespace ChObjectTest

theorem chobject_smoke : True := by
  trivial

end ChObjectTest

namespace ChProofTest

theorem chproof_smoke : True := by
  trivial

end ChProofTest

namespace PassPipelineTest

theorem pipeline_smoke : True := by
  trivial

theorem default_pipeline_covers_all_declared_passes :
    PassPipeline.defaultPipeline.passes.length =
      PassPipeline.defaultPipeline.passEffects.length := by
  rfl

end PassPipelineTest

namespace RunPassTest

theorem run_pass_smoke : True := by
  trivial

private def sampleSemSig (ret : ChType) : SemanticSignature := {
  params := []
  returns := ret
  isVarargs := false
}

private def samplePhysSig : PhysicalSignature := {
  params := []
  returns := .void
  callingConv := .sysv
}

private def exportSym : Symbol := Symbol.simple "export_fn"
private def importSym : Symbol := Symbol.simple "import_fn"
private def allocSym : Symbol := Symbol.simple "global_alloc"

private def baseContract (sym : Symbol) (ret : ChType := .unit) : FunctionContract := {
  symbol := sym
  language := .c
  form := .infallible
  semanticSig := sampleSemSig ret
  physicalSig := samplePhysSig
  effects := [.pure]
  panicPolicy := .catchUnwind
  safety := .verified
  allocator := none
  requiresDrop := false
  trust := .proofObligation
  errorDomain := none
}

private def baseModule : Module := {
  abiVersion := "0.1"
  moduleName := Symbol.simple "pass_test"
  language := .c
  target := Target.x86_64_linux
  exports := [{ symbol := exportSym, contract := baseContract exportSym }]
  imports := [{ symbol := importSym, contract := baseContract importSym }]
  types := []
  layouts := []
}

theorem target_validate_rejects_zero_width :
    PassPipeline.runPass PassPipeline.defaultPipeline .targetValidate
      { baseModule with target := { Target.x86_64_linux with ptrWidth := 0 } } =
      .error { pass := .targetValidate, message := "invalid target widths", location := none } := by
  rfl

theorem metadata_import_rejects_invalid_version :
    PassPipeline.runPass PassPipeline.defaultPipeline .metadataImport
      { baseModule with abiVersion := "2.0" } =
      .error { pass := .metadataImport, message := "invalid version: 2.0", location := none } := by
  rfl

theorem layout_validate_rejects_zero_align_layout :
    let badLayout : DeclaredLayout := {
      name := Symbol.simple "bad_layout"
      size := 8
      align := 0
      hash := 0
      fields := []
    }
    PassPipeline.runPass PassPipeline.defaultPipeline .layoutValidate
      { baseModule with layouts := [badLayout] } =
      .error { pass := .layoutValidate, message := "invalid layout alignment for bad_layout: 0", location := none } := by
  rfl

theorem signature_lower_rejects_missing_allocator_for_alloc_effect :
    let badContract := { baseContract exportSym with effects := [.mayAlloc] }
    let badModule := { baseModule with exports := [{ symbol := exportSym, contract := badContract }] }
    PassPipeline.runPass PassPipeline.defaultPipeline .signatureLower badModule =
      .error { pass := .signatureLower, message := "contract validation failed for export_fn: Chimera.ContractCheckError.missingAllocator { ns := \"\", name := \"export_fn\" }", location := none } := by
  rfl

theorem ownership_verify_rejects_call_lifetime_return :
    let badContract := { baseContract exportSym (.borrow .u32 .call) with physicalSig := { samplePhysSig with returns := .value 1 } }
    let badModule := { baseModule with exports := [{ symbol := exportSym, contract := badContract }] }
    PassPipeline.runPass PassPipeline.defaultPipeline .ownershipVerify badModule =
      .error { pass := .ownershipVerify, message := "ownership validation failed for export_fn: Chimera.OwnershipCheckError.callLifetimeReturn Chimera.ChType.borrow Chimera.ChType.u32 Chimera.Lifetime.call", location := none } := by
  rfl

theorem allocator_verify_rejects_owned_opaque_without_allocator :
    let badContract := { baseContract exportSym (.owned (.opaque (Symbol.simple "Handle"))) with requiresDrop := true }
    let badModule := { baseModule with exports := [{ symbol := exportSym, contract := badContract }] }
    PassPipeline.runPass PassPipeline.defaultPipeline .allocatorVerify badModule =
      .error { pass := .allocatorVerify, message := "allocator required for export_fn", location := none } := by
  rfl

theorem result_verify_rejects_fallible_nonresult_return :
    let badContract := { baseContract exportSym .u32 with form := .fallible, physicalSig := { samplePhysSig with returns := .value 1 } }
    let badModule := { baseModule with exports := [{ symbol := exportSym, contract := badContract }] }
    PassPipeline.runPass PassPipeline.defaultPipeline .resultVerify badModule =
      .error { pass := .resultVerify, message := "fallible contract must return Result for export_fn", location := none } := by
  rfl

theorem panic_verify_rejects_forbidden_panic_surface :
    let badContract := { baseContract exportSym with effects := [.mayPanic], panicPolicy := .forbidden }
    let badModule := { baseModule with exports := [{ symbol := exportSym, contract := badContract }] }
    PassPipeline.runPass PassPipeline.defaultPipeline .panicVerify badModule =
      .error { pass := .panicVerify, message := "panic policy forbids panic surface for export_fn", location := none } := by
  rfl

theorem valid_result_pass_succeeds :
    let okContract := { baseContract exportSym (.result .u32 .error) with
      form := .fallible
      physicalSig := { samplePhysSig with returns := .value 1 }
      effects := [.mayError]
      errorDomain := some .cErrno
    }
    let okImport := { baseContract importSym with effects := [.pure] }
    let okModule := {
      baseModule with
      exports := [{ symbol := exportSym, contract := okContract }]
      imports := [{ symbol := importSym, contract := okImport }]
    }
    PassPipeline.runPass PassPipeline.defaultPipeline .resultVerify okModule = .ok okModule := by
  rfl

end RunPassTest

namespace OperationWellFormedTest

private def reg (name : String) : OperandValue := .register name

private def u32Operand (name : String) : Operand := ⟨.u32, reg name⟩
private def statusOperand (name : String) : Operand := ⟨.status, reg name⟩
private def ownedOperand (name : String) : Operand := ⟨.owned .u32, reg name⟩
private def borrowOperand (name : String) : Operand := ⟨.borrow .u32 .static, reg name⟩
private def callBorrowOperand (name : String) : Operand := ⟨.borrow .u32 .call, reg name⟩
private def rawOperand (name : String) : Operand := ⟨.rawptr .u8, reg name⟩
private def resultOperand (name : String) : Operand := ⟨.result .u32 .error, reg name⟩

theorem call_is_well_formed :
    Operation.isWellFormed
      { kind := .call
        inputs := [u32Operand "arg"]
        outputs := [u32Operand "ret"]
        location := none } = true := by
  rfl

theorem call_requires_nonempty_inputs :
    Operation.isWellFormed
      { kind := .call
        inputs := []
        outputs := [u32Operand "ret"]
        location := none } = false := by
  rfl

theorem call_rejects_direct_result_output :
    Operation.isWellFormed
      { kind := .call
        inputs := [u32Operand "arg"]
        outputs := [resultOperand "ret"]
        location := none } = false := by
  rfl

theorem call_rejects_escaping_borrow_output :
    Operation.isWellFormed
      { kind := .call
        inputs := [u32Operand "arg"]
        outputs := [callBorrowOperand "ret"]
        location := none } = false := by
  rfl

theorem ownership_transfer_requires_droppable_surfaces :
    Operation.isWellFormed
      { kind := .ownershipTransfer
        inputs := [ownedOperand "src"]
        outputs := [ownedOperand "dst"]
        location := none } = true := by
  rfl

theorem ownership_transfer_rejects_nondroppable_input :
    Operation.isWellFormed
      { kind := .ownershipTransfer
        inputs := [u32Operand "src"]
        outputs := [ownedOperand "dst"]
        location := none } = false := by
  rfl

theorem borrow_requires_borrow_output :
    Operation.isWellFormed
      { kind := .borrow
        inputs := [ownedOperand "src"]
        outputs := [borrowOperand "dst"]
        location := none } = true := by
  rfl

theorem borrow_rejects_nonborrow_output :
    Operation.isWellFormed
      { kind := .borrow
        inputs := [ownedOperand "src"]
        outputs := [u32Operand "dst"]
        location := none } = false := by
  rfl

theorem drop_requires_droppable_input :
    Operation.isWellFormed
      { kind := .drop
        inputs := [ownedOperand "value"]
        outputs := []
        location := none } = true := by
  rfl

theorem drop_rejects_nondroppable_input :
    Operation.isWellFormed
      { kind := .drop
        inputs := [u32Operand "value"]
        outputs := []
        location := none } = false := by
  rfl

theorem error_bridge_accepts_result_to_status :
    Operation.isWellFormed
      { kind := .errorBridge
        inputs := [resultOperand "value"]
        outputs := [statusOperand "status"]
        location := none } = true := by
  rfl

theorem error_bridge_rejects_nonbridge_shapes :
    Operation.isWellFormed
      { kind := .errorBridge
        inputs := [u32Operand "value"]
        outputs := [statusOperand "status"]
        location := none } = false := by
  rfl

theorem panic_bridge_accepts_boundary_safe_surfaces :
    Operation.isWellFormed
      { kind := .panicBridge
        inputs := [u32Operand "value"]
        outputs := [statusOperand "status"]
        location := none } = true := by
  rfl

theorem panic_bridge_rejects_result_output :
    Operation.isWellFormed
      { kind := .panicBridge
        inputs := [u32Operand "value"]
        outputs := [resultOperand "status"]
        location := none } = false := by
  rfl

theorem raw_unsafe_call_requires_raw_surface :
    Operation.isWellFormed
      { kind := .rawUnsafeCall
        inputs := [rawOperand "ptr"]
        outputs := []
        location := none } = true := by
  rfl

theorem raw_unsafe_call_rejects_safe_surface_only :
    Operation.isWellFormed
      { kind := .rawUnsafeCall
        inputs := [u32Operand "value"]
        outputs := []
        location := none } = false := by
  rfl

theorem prop_wrapper_tracks_executable_checker :
    WellFormedOperation
      { kind := .drop
        inputs := [ownedOperand "value"]
        outputs := []
        location := none } := by
  rfl

end OperationWellFormedTest

end Chimera.Test
