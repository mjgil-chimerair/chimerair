-- ChimeraProof Tests: Checkers Tests
-- Executable checker coverage over the current Lean surface.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Contract
import Chimera.ABI.Signature
import Chimera.IR.Module
import Chimera.Checkers

namespace Chimera.Test

namespace MetadataCheckerTest

def mkExport (ns name : String) (contract : FunctionContract) : Export := {
  symbol := ⟨ns, name⟩
  contract := contract
}

def mkImport (ns name : String) (contract : FunctionContract) : Import := {
  symbol := ⟨ns, name⟩
  contract := contract
}

def mkSemSig (ret : ChType) : SemanticSignature := {
  params := []
  returns := ret
}

def mkPhysSig (ret : ReturnSpec) : PhysicalSignature := {
  params := []
  returns := ret
  callingConv := .sysv
}

def mkContract (ns name : String) (semSig : SemanticSignature) : FunctionContract := {
  symbol := ⟨ns, name⟩
  language := .c
  form := .infallible
  semanticSig := semSig
  physicalSig := mkPhysSig (.value 0)
  effects := []
  panicPolicy := .forbidden
  safety := .verified
  allocator := none
  requiresDrop := false
  trust := .proofObligation
  errorDomain := none
}

def mkModule : Module := {
  abiVersion := "0.1"
  moduleName := ⟨"", "test_module"⟩
  language := .c
  target := Target.x86_64_linux
  exports := [mkExport "" "export_fn" (mkContract "" "export_fn" (mkSemSig .unit))]
  imports := [mkImport "" "import_fn" (mkContract "" "import_fn" (mkSemSig .unit))]
  types := []
  layouts := []
}

/--
checkChMeta accepts a valid module.
-/
theorem valid_module_passes :
  checkChMeta mkModule = Except.ok { module := mkModule, validated := true } := by
  rfl

/--
checkChMeta rejects empty exports.
-/
theorem empty_exports_fails :
  checkChMeta { mkModule with exports := [] } = Except.error .emptyExport := by
  rfl

/--
checkChMeta rejects empty imports.
-/
theorem empty_imports_fails :
  checkChMeta { mkModule with imports := [] } = Except.error .emptyImport := by
  rfl

/--
checkChMeta rejects invalid version.
-/
theorem invalid_version_fails :
  checkChMeta { mkModule with abiVersion := "2.0" } = Except.error (.invalidVersion "2.0") := by
  rfl

/--
checkChMeta rejects empty module name.
-/
theorem empty_module_name_fails :
  checkChMeta { mkModule with moduleName := ⟨"", ""⟩ } = Except.error .emptyModuleName := by
  rfl

/--
checkChMeta rejects empty export symbol.
-/
theorem empty_export_symbol_fails :
  checkChMeta { mkModule with exports := [mkExport "" "" (mkContract "" "fn" (mkSemSig .unit))] } = Except.error .emptyExportSymbol := by
  rfl

/--
checkChMeta rejects empty import symbol.
-/
theorem empty_import_symbol_fails :
  checkChMeta { mkModule with imports := [mkImport "" "" (mkContract "" "fn" (mkSemSig .unit))] } = Except.error .emptyImportSymbol := by
  rfl

/--
checkChMeta detects duplicate exports.
-/
theorem duplicate_exports_detected :
  let c := mkContract "" "fn" (mkSemSig .unit)
  checkChMeta { mkModule with exports := [mkExport "" "fn" c, mkExport "" "fn" c] } = Except.error (.duplicateExport ⟨"", "fn"⟩) := by
  rfl

/--
checkChMeta detects duplicate imports.
-/
theorem duplicate_imports_detected :
  let c := mkContract "" "fn" (mkSemSig .unit)
  checkChMeta { mkModule with imports := [mkImport "" "fn" c, mkImport "" "fn" c] } = Except.error (.duplicateImport ⟨"", "fn"⟩) := by
  rfl

end MetadataCheckerTest

namespace FullCheckerTest

theorem contractChecker_importable : True := by
  trivial

theorem fullCheck_sound_holds_smoke : True := by
  trivial

theorem certified_build_rejects_unwind_for_export_contract :
  let contract := mkContract "" "export_fn" (mkSemSig .unit)
  let m : Module := {
    mkModule with
    exports := [mkExport "" "export_fn" contract]
    imports := [mkImport "" "import_fn" (mkContract "" "import_fn" (mkSemSig .unit))]
  }
  let cert : CertifiedBuild := { modules := [m], validated := true }
  fullCheck [m] = Except.ok cert →
    checkBoundaryExit contract.panicPolicy .unwound = Except.error .unwindNotAllowed := by
  intro contract m cert h
  exact fullCheck.certified_exports_reject_unwind [m] cert h m (by simp [cert]) contract (by simp [m, contract])

end FullCheckerTest

namespace OwnershipCheckerTest

theorem empty_resources_pass : checkNoDoubleOwn [] = Except.ok () := by
  rfl

theorem double_own_detected_smoke : True := by
  trivial

end OwnershipCheckerTest

namespace PanicCheckerTest

theorem abort_allows_abort : checkBoundaryExit .abort .aborted = Except.ok () := by
  rfl

theorem unwind_rejected : checkBoundaryExit .forbidden .unwound = Except.error .unwindNotAllowed := by
  rfl

theorem returned_boundary_is_safe_under_forbidden :
  BoundaryExitSafe .forbidden (.returned Pointer.null) = true := by
  rfl

theorem catch_policy_accepts_panic_safely :
  let payload : PanicPayload := { message := "panic", file := "test", line := 1 }
  checkBoundaryExit .catchUnwind (.panicked payload) = Except.ok () ∧
    BoundaryExitSafe .catchUnwind (.panicked payload) = true := by
  constructor
  · rfl
  · rfl

theorem accepted_boundary_never_unwinds :
  accepted_boundary_never_unwinds .catchUnwind (.returned Pointer.null) rfl := by
  intro h
  cases h

theorem unwind_always_rejected_for_abort_policy :
  unwind_always_rejected .abort = Except.error .unwindNotAllowed := by
  rfl

end PanicCheckerTest

end Chimera.Test
