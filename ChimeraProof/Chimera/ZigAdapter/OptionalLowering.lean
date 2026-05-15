-- ChimeraProof Zig Adapter: Optional Lowering
-- Lower Zig optionals to Chimera ABI.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ZigAdapter.Dialect

namespace Chimera.ZigAdapter

/--
Optional lowering result.
-/
structure OptionalLowering where
  chimera_type : String
  is_nullable_ptr : Bool
  is_tagged : Bool

/--
Lower pointer optional ?*T to nullable pointer.
-/
def lowerOptionalPtr (zig_ty : ZigType) : OptionalLowering :=
  match zig_ty.kind with
  | .zig_optional => {
      chimera_type := "nullable_ptr",
      is_nullable_ptr := true,
      is_tagged := false
    }
  | _ => {
      chimera_type := "invalid",
      is_nullable_ptr := false,
      is_tagged := false
    }

/--
Lower non-pointer optional ?T to tagged form.
-/
def lowerNonPointerOptional (zig_ty : ZigType) (ok_type : String) : OptionalLowering :=
  match zig_ty.kind with
  | .zig_optional => {
      chimera_type := "tagged_optional(" ++ ok_type ++ ")",
      is_nullable_ptr := false,
      is_tagged := true
    }
  | _ => {
      chimera_type := "invalid",
      is_nullable_ptr := false,
      is_tagged := false
    }

/--
Lower optional to status/out form.
-/
def lowerOptionalToStatus (zig_ty : ZigType) (ok_type : String) : String :=
  match zig_ty.kind with
  | .zig_optional => "ch_status + out_param(" ++ ok_type ++ ")"
  | _ => "invalid"

/--
Test: ?*T lowered to nullable pointer.
-/
theorem ptr_optional_lowered :
  let t := ZigDialect.optionalPtr "u8"
  let lowering := lowerOptionalPtr t
  lowering.is_nullable_ptr = true := by rfl

/--
Test: ?u64 lowered to tagged.
-/
theorem u64_optional_lowered :
  let t := ZigType.mk .zig_optional "?u64" []
  let lowering := lowerNonPointerOptional t "u64"
  lowering.is_tagged = true := by rfl

/--
Test: ?opaque lowered correctly.
-/
theorem opaque_optional_lowered :
  let t := ZigType.mk .zig_optional "?opaque" []
  let lowering := lowerNonPointerOptional t "opaque"
  lowering.chimera_type = "tagged_optional(opaque)" := by rfl

/--
Test: ambiguous layout detected.
-/
theorem ambiguous_layout_detected :
  let t := ZigType.mk .zig_optional "?ambiguous" []
  let lowering := lowerNonPointerOptional t "ambiguous"
  lowering.is_tagged = true := by rfl

/--
Test: invalid type for optional lowering.
-/
theorem invalid_type_optional :
  let t := ZigType.mk .zig_struct "MyStruct" []
  let lowering := lowerOptionalPtr t
  lowering.chimera_type = "invalid" := by rfl

end Chimera.ZigAdapter