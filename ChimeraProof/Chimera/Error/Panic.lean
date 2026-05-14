-- ChimeraProof Error: Panic
-- Panic policy and boundary outcome modeling.

import Chimera.Foundation
import Chimera.ABI
import Chimera.Memory

namespace Chimera

/--
Panic payload.
-/
structure PanicPayload where
  message : String
  file    : String
  line    : Nat
deriving Repr, BEq

/--
Boundary outcome after a function call.
-/
inductive BoundaryOutcome where
  | returned : Pointer → BoundaryOutcome
  | errored : Pointer → BoundaryOutcome
  | panicked : PanicPayload → BoundaryOutcome
  | aborted
  | unwound
deriving Repr, BEq

/--
Boundary exit safety predicate.
-/
def BoundaryExitSafe (policy : PanicPolicy) (outcome : BoundaryOutcome) : Bool :=
  match outcome with
  | .aborted => policy == .abort
  | .panicked _ =>
    match policy with
    | .catchUnwind => true
    | .abort => true
    | .forbidden => false
  | .unwound => false
  | _ => true

namespace BoundaryOutcome

/--
Check if outcome is safe under abort policy.
-/
def safeUnderAbort : BoundaryOutcome → Bool
  | .returned _ => true
  | .errored _ => true
  | .aborted => true
  | _ => false

/--
Check if outcome is safe under forbidden policy.
-/
def safeUnderForbidden : BoundaryOutcome → Bool
  | .returned _ => true
  | .errored _ => true
  | _ => false

/--
Check if outcome is safe under catch policy.
-/
def safeUnderCatch : BoundaryOutcome → Bool
  | .returned _ => true
  | .errored _ => true
  | .panicked _ => true
  | .aborted => false
  | .unwound => false

end BoundaryOutcome

/--
Theorem: abort policy allows aborted outcome.
-/
theorem abort_allows_aborted (outcome : BoundaryOutcome)
  (h : outcome = .aborted) :
  BoundaryExitSafe .abort outcome = true := by
  cases h
  rfl

/--
Theorem: forbidden policy rejects panicked outcome.
-/
theorem forbidden_rejects_panic (payload : PanicPayload) :
  BoundaryExitSafe .forbidden (.panicked payload) = false := by
  rfl

/--
Theorem: catch policy converts panic to safe.
-/
theorem catch_allows_panic (payload : PanicPayload) :
  BoundaryExitSafe .catchUnwind (.panicked payload) = true := by
  rfl

end Chimera
