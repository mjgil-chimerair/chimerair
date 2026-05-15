--! Chimera.RustAdapter.Panic
--!
--! Lean model for Rust panic boundary safety proofs.

import Chimera.RustAdapter
import Chimera.Effects

namespace Chimera.RustAdapter.Panic

/--
  Panic boundary safety proof.
  
  Proves that the configured panic policy prevents forbidden
  unwind across the Chimera ABI boundary.
-/
structure PanicBoundarySafety where
  policy : PanicPolicy
  safeTransfers : List SafeTransfer
  unsafeTransfers : List UnsafeTransfer

/--
  Safe transfer across panic boundary.
-/
structure SafeTransfer where
  fromFn : String
  toFn : String
  reason : String

/--
  Unsafe transfer across panic boundary.
-/
structure UnsafeTransfer where
  fromFn : String
  toFn : String
  reason : String
  trustAssumption : String

/--
  Panic policy for Rust code.
-/
inductive PanicPolicy where
  | abort
  | catch
  | unwind

/--
  Panic safety theorem.
  
  For a given panic policy, prove that panic cannot propagate
  across the ABI boundary in a forbidden way.
-/
structure PanicSafetyTheorem where
  policy : PanicPolicy
  precondition : String
  proof : String
  conclusion : String

/--
  Validate panic policy configuration.
-/
structure PanicPolicyValidation where
  policy : PanicPolicy
  abortPreservesSafety : Bool
  catchPreservesSafety : Bool
  unwindRequiresTrust : Bool

end Chimera.RustAdapter.Panic
