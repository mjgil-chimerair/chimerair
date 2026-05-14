-- CAdapter ErrorBridge Preservation
-- Task 140: Prove errno/status bridge soundness - lowering preserves success/error distinction

import Lean
import Chimera.CAdapter.ErrorBridge

namespace Chimera.CAdapter

/--
Error Bridge Result - success or error with domain
-/
inductive ErrorResult
  | success
  | error (domain : Nat) (code : Nat)
deriving Repr, BEq, DecidableEq

/--
C function error bridge mapping
-/
structure CErrorBridge where
  fn_name : String
  has_errno : Bool
  error_domain : Nat
deriving Repr, BEq, DecidableEq

/--
Lower ErrorBridge to ErrorResult
-/
def lowerErrorBridge (bridge : CErrorBridge) : ErrorResult :=
  if bridge.has_errno then
    ErrorResult.error bridge.error_domain 0
  else
    ErrorResult.success

/--
Theorem: Lowering preserves success case
-/
theorem lower_preserves_success (bridge : CErrorBridge)
    (h : bridge.has_errno = false) :
  lowerErrorBridge bridge = ErrorResult.success := by
  simp [lowerErrorBridge, h]

/--
Theorem: Lowering preserves error case
-/
theorem lower_preserves_error (bridge : CErrorBridge)
    (h : bridge.has_errno = true) :
  lowerErrorBridge bridge = ErrorResult.error bridge.error_domain 0 := by
  simp [lowerErrorBridge, h]

/--
Theorem: Error result domain matches bridge
-/
theorem error_domain_matches (bridge : CErrorBridge)
    (h : bridge.has_errno = true) :
  match lowerErrorBridge bridge with
  | ErrorResult.success => False
  | ErrorResult.error d _ => d = bridge.error_domain := by
  simp [lowerErrorBridge, h]

/--
Theorem: Success/error distinction is preserved
-/
theorem success_error_distinct (bridge : CErrorBridge) :
  lowerErrorBridge bridge = ErrorResult.success ↔ bridge.has_errno = false := by
  apply Iff.intro
  · simp [lowerErrorBridge]
  · simp [lowerErrorBridge]

end Chimera.CAdapter
