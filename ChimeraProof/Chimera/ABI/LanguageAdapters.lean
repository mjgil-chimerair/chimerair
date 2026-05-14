-- ChimeraProof ABI: Language Adapters
-- Models for C, Rust, and Zig boundary crossing.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Type

namespace Chimera

/--
Rust adapter rules - what types can cross extern "C" boundaries.
-/
inductive RustAdapterRule where
  | noNativeRustTypes     -- Vec, String, Box, Rc, Arc cannot cross
  | reprCRequired        -- Types must be repr(C)
  | noDynamicDispatch    -- No trait objects
  | resultLowering        -- Result<T, E> lowers to ch_status + out params
  | panicPolicyRespected -- extern functions must respect panic policy

/--
Rust adapter model - validates types for Rust FFI boundary.
-/
structure RustAdapter where
  rules : List RustAdapterRule

namespace RustAdapter

/--
Default Rust adapter with standard rules.
-/
def default : RustAdapter := {
  rules := [.noNativeRustTypes, .reprCRequired, .noDynamicDispatch, .resultLowering, .panicPolicyRespected]
}

/--
Check if a type is allowed for Rust FFI.
-/
def isAllowedType (adapter : RustAdapter) (ty : ChType) : Bool :=
  match ty with
  | .owned _ => false  -- Owned Rust types cannot cross
  | .slice _ _ => false  -- Rust slices cannot cross
  | .str _ _ => false  -- Rust strings cannot cross
  | .result _ _ => true  -- Result must be lowered
  | _ => ty.isPrimitive ∨ ty.isCCompatible

/--
Theorem: primitive types are allowed.
-/
theorem primitive_allowed (adapter : RustAdapter) :
  adapter.isAllowedType .i32 = true := by
  simp [isAllowedType]

/--
Theorem: owned types are rejected.
-/
theorem owned_rejected (adapter : RustAdapter) :
  adapter.isAllowedType (.owned .i32) = false := by
  simp [isAllowedType]

/--
Theorem: slices are rejected.
-/
theorem slice_rejected (adapter : RustAdapter) :
  adapter.isAllowedType (.slice .u8 .owned) = false := by
  simp [isAllowedType]

end RustAdapter

/--
Zig adapter rules - what types can cross export fn boundaries.
-/
inductive ZigAdapterRule where
  | noDirectSlices      -- Zig slices cannot cross directly
  | noErrorUnions      -- Zig error unions must lower to result
  | externStruct       -- Types must be extern struct
  | noRuntimeBits     -- No dependent-type bits

/--
Zig adapter model - validates types for Zig FFI boundary.
-/
structure ZigAdapter where
  rules : List ZigAdapterRule

namespace ZigAdapter

/--
Default Zig adapter with standard rules.
-/
def default : ZigAdapter := {
  rules := [.noDirectSlices, .noErrorUnions, .externStruct, .noRuntimeBits]
}

/--
Check if a type is allowed for Zig FFI.
-/
def isAllowedType (adapter : ZigAdapter) (ty : ChType) : Bool :=
  match ty with
  | .slice _ _ => false  -- Zig slices cannot cross
  | .str _ _ => false  -- Zig strings cannot cross
  | .result _ _ => true  -- Error union lowered
  | _ => ty.isPrimitive ∨ ty.isCCompatible

/--
Theorem: primitive types are allowed.
-/
theorem primitive_allowed (adapter : ZigAdapter) :
  adapter.isAllowedType .i32 = true := by
  simp [isAllowedType]

/--
Theorem: slices are rejected.
-/
theorem slice_rejected (adapter : ZigAdapter) :
  adapter.isAllowedType (.slice .u8 .owned) = false := by
  simp [isAllowedType]

end ZigAdapter

end Chimera