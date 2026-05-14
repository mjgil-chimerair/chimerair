-- ChimeraProof Checkers: Panic Checker
-- Executable panic policy validation.

import Chimera.Foundation
import Chimera.ABI
import Chimera.Error
import Chimera.Error.Panic

namespace Chimera

/--
Panic check error.
-/
inductive PanicCheckError where
  | unwindNotAllowed
  | panicWithWrongPolicy (policy : PanicPolicy)
  | abortWithoutAbortable
deriving Repr, BEq

/--
Check boundary exit is safe under policy.
-/
def checkBoundaryExit
  (policy : PanicPolicy)
  (outcome : BoundaryOutcome) :
  Except PanicCheckError Unit := do
  match outcome with
  | .unwound =>
    .error .unwindNotAllowed
  | .panicked payload =>
    match policy with
    | .forbidden =>
      .error (.panicWithWrongPolicy policy)
    | _ => .ok ()
  | .aborted =>
    match policy with
    | .abort => .ok ()
    | _ => .error (.panicWithWrongPolicy policy)
  | _ => .ok ()  -- returned, errored are always safe

/--
Check panic policy is valid.
-/
def checkPanicPolicy (policy : PanicPolicy) : Except PanicCheckError Unit :=
  match policy with
  | .abort | .catchUnwind | .forbidden => .ok ()

/--
Theorem: checkBoundaryExit is sound.
-/
theorem checkBoundaryExit_sound
  (policy : PanicPolicy)
  (outcome : BoundaryOutcome)
  (h : checkBoundaryExit policy outcome = Except.ok ()) :
  BoundaryExitSafe policy outcome = true := by
  cases outcome <;> simp [checkBoundaryExit, BoundaryExitSafe] at h ⊢

/--
Accepted boundaries never allow an unwind outcome.
-/
theorem accepted_boundary_never_unwinds
  (policy : PanicPolicy)
  (outcome : BoundaryOutcome)
  (h : checkBoundaryExit policy outcome = Except.ok ()) :
  outcome ≠ .unwound := by
  intro hEq
  cases hEq
  simp [checkBoundaryExit] at h

/--
Unwind is rejected for every panic policy.
-/
theorem unwind_always_rejected (policy : PanicPolicy) :
  checkBoundaryExit policy .unwound = Except.error .unwindNotAllowed := by
  rfl

end Chimera
