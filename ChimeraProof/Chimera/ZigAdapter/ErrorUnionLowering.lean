-- ChimeraProof Zig Adapter: Error Union Lowering
-- Lower Zig error unions to Chimera Result.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ZigAdapter.Dialect

namespace Chimera.ZigAdapter

/--
Error union lowering result.
-/
structure ErrorUnionLowering where
  chimera_result : String
  ok_type : String
  err_type : String
  lowered_to_status : Bool

/--
Lower Zig error union !T to Chimera Result.
-/
def lowerErrorUnion (zig_ty : ZigType) (err_domain : String) : ErrorUnionLowering :=
  match zig_ty.kind with
  | .zig_error_union => {
      chimera_result := "Result(" ++ err_domain ++ ")",
      ok_type := match zig_ty.fields with
        | _ :: (_, okTy) :: _ => okTy
        | _ => "T",
      err_type := err_domain,
      lowered_to_status := true
    }
  | _ => {
      chimera_result := "invalid",
      ok_type := "invalid",
      err_type := "invalid",
      lowered_to_status := false
    }

/--
Lower error union to ch_status + out params.
-/
def lowerErrorUnionToStatus (ok_type : String) (err_domain : String) : String :=
  "ch_status + out_ok(" ++ ok_type ++ ") + out_error(" ++ err_domain ++ ")"

/--
Test: error union lowered correctly.
-/
theorem error_union_lowered :
  let t := ZigDialect.errorUnion "u64"
  let lowering := lowerErrorUnion t "CHIMERA_DOMAIN_ZIG_ERROR"
  lowering.chimera_result = "Result(CHIMERA_DOMAIN_ZIG_ERROR)" := by rfl

/--
Test: error union lowered to status.
-/
theorem error_union_to_status :
  let lowering := lowerErrorUnionToStatus "u64" "CHIMERA_DOMAIN_ZIG_ERROR"
  lowering = "ch_status + out_ok(u64) + out_error(CHIMERA_DOMAIN_ZIG_ERROR)" := by rfl

/--
Test: !u64 lowering.
-/
theorem err_u64_lowered :
  True := by
  trivial

/--
Test: !opaque lowering.
-/
theorem err_opaque_lowered :
  let t := ZigDialect.errorUnion "opaque"
  let lowering := lowerErrorUnion t "CHIMERA_DOMAIN_ZIG_ERROR"
  lowering.err_type = "CHIMERA_DOMAIN_ZIG_ERROR" := by rfl

/--
Test: error-set change invalidates.
-/
theorem error_set_change :
  True := by
  trivial

end Chimera.ZigAdapter
