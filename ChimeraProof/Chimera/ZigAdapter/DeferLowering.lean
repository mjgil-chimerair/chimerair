-- ChimeraProof Zig Adapter: Defer Lowering
-- Lower Zig defer/errdefer to cleanup/drop obligations in ChimeraIR.

import Chimera.Foundation

namespace Chimera.ZigAdapter

/--
Defer kind.
-/
inductive DeferKind
  | defer_block     -- success-path cleanup
  | errdefer_block   -- error-path cleanup
deriving Inhabited, Repr, BEq, DecidableEq

/--
Cleanup obligation from defer lowering.
-/
structure CleanupObligation where
  kind : DeferKind
  cleanup_expr : String
  order : Nat
deriving Inhabited, Repr, BEq, DecidableEq

/--
Defer lowering result.
-/
structure DeferLoweringResult where
  success_cleanup : List CleanupObligation
  error_cleanup : List CleanupObligation
deriving Repr, BEq, DecidableEq

/--
Ownership facts required when an owned resource crosses a boundary.
-/
structure OwnershipBoundaryFact where
  resource : String
  crossesBoundary : Bool
  hasDropMetadata : Bool
  hasAllocatorMetadata : Bool
deriving Repr, BEq, DecidableEq

/--
Lower Zig defer to cleanup obligation.
-/
def lowerDefer (defer_kind : DeferKind) (cleanup_expr : String) (order : Nat) : CleanupObligation :=
  ⟨defer_kind, cleanup_expr, order⟩

/--
Lower success-path defer.
-/
def lowerSuccessDefer (expr : String) (order : Nat) : DeferLoweringResult := {
  success_cleanup := [lowerDefer .defer_block expr order],
  error_cleanup := []
}

/--
Lower errdefer (error-path cleanup).
-/
def lowerErrdefer (expr : String) (order : Nat) : DeferLoweringResult := {
  success_cleanup := [],
  error_cleanup := [lowerDefer .errdefer_block expr order]
}

/--
Merge multiple defer lowerings.
-/
def mergeDefers (results : List DeferLoweringResult) : DeferLoweringResult :=
  {
    success_cleanup := results.foldl (fun acc r => acc ++ r.success_cleanup) [],
    error_cleanup := results.foldl (fun acc r => acc ++ r.error_cleanup) []
  }

/--
Owned resources may only cross a boundary when the required cleanup metadata is present.
-/
def ownedBoundarySafe (fact : OwnershipBoundaryFact) : Bool :=
  if fact.crossesBoundary then
    fact.hasDropMetadata && fact.hasAllocatorMetadata
  else
    true

/--
Test: defer produces success cleanup.
-/
theorem defer_produces_success_cleanup :
  let result := lowerSuccessDefer "cleanup()" 1
  result.success_cleanup.length = 1 := by rfl

/--
Test: errdefer produces error cleanup.
-/
theorem errdefer_produces_error_cleanup :
  let result := lowerErrdefer "err_cleanup()" 2
  result.error_cleanup.length = 1 := by rfl

/--
Test: errdefer has no success cleanup.
-/
theorem errdefer_no_success_cleanup :
  let result := lowerErrdefer "err_cleanup()" 1
  result.success_cleanup = [] := by rfl

/--
Test: merge combines success cleanups.
-/
theorem merge_combines_success :
  let r1 := lowerSuccessDefer "cleanup1()" 1
  let r2 := lowerSuccessDefer "cleanup2()" 2
  let merged := mergeDefers [r1, r2]
  merged.success_cleanup.length = 2 := by rfl

/--
Test: merge preserves error cleanups.
-/
theorem merge_preserves_error :
  let r1 := lowerErrdefer "err1()" 1
  let r2 := lowerErrdefer "err2()" 2
  let merged := mergeDefers [r1, r2]
  merged.error_cleanup.length = 2 := by rfl

/--
Test: defer order preserved.
-/
theorem defer_order_preserved :
  let result := lowerSuccessDefer "cleanup()" 5
  result.success_cleanup[0]!.order = 5 := by rfl

/--
Task 118 scenario: merged defer obligations preserve cleanup order.
-/
theorem merged_defer_order_preserved :
  let r1 := lowerSuccessDefer "cleanup1()" 1
  let r2 := lowerSuccessDefer "cleanup2()" 2
  let merged := mergeDefers [r1, r2]
  merged.success_cleanup = [
    lowerDefer .defer_block "cleanup1()" 1,
    lowerDefer .defer_block "cleanup2()" 2
  ] := by
  native_decide

/--
Task 118 scenario: errdefer cleanup remains on the error path only.
-/
theorem errdefer_error_path_only :
  let result := lowerErrdefer "err_cleanup()" 3
  result.success_cleanup = [] ∧
    result.error_cleanup = [lowerDefer .errdefer_block "err_cleanup()" 3] := by
  constructor <;> native_decide

/--
Task 118 scenario: owned resources crossing a boundary without metadata are rejected.
-/
theorem owned_resource_without_metadata_rejected :
  let fact : OwnershipBoundaryFact := {
    resource := "buffer"
    crossesBoundary := true
    hasDropMetadata := false
    hasAllocatorMetadata := false
  }
  ownedBoundarySafe fact = false := by
  native_decide

/--
Task 118 scenario: owned resources crossing a boundary with required metadata are accepted.
-/
theorem owned_resource_with_metadata_allowed :
  let fact : OwnershipBoundaryFact := {
    resource := "buffer"
    crossesBoundary := true
    hasDropMetadata := true
    hasAllocatorMetadata := true
  }
  ownedBoundarySafe fact = true := by
  native_decide

/--
Task 118 summary theorem: defer ordering and ownership-boundary checks preserve
cleanup obligations and reject metadata-free owned boundary crossings.
-/
theorem zig_ownership_defer_soundness_surface :
  (let r1 := lowerSuccessDefer "cleanup1()" 1
   let r2 := lowerSuccessDefer "cleanup2()" 2
   let merged := mergeDefers [r1, r2]
   merged.success_cleanup = [
     lowerDefer .defer_block "cleanup1()" 1,
     lowerDefer .defer_block "cleanup2()" 2
   ]) ∧
    (let result := lowerErrdefer "err_cleanup()" 3
     result.success_cleanup = [] ∧
       result.error_cleanup = [lowerDefer .errdefer_block "err_cleanup()" 3]) ∧
    (let fact : OwnershipBoundaryFact := {
      resource := "buffer"
      crossesBoundary := true
      hasDropMetadata := false
      hasAllocatorMetadata := false
    }
    ownedBoundarySafe fact = false) ∧
    (let fact : OwnershipBoundaryFact := {
      resource := "buffer"
      crossesBoundary := true
      hasDropMetadata := true
      hasAllocatorMetadata := true
    }
    ownedBoundarySafe fact = true) := by
  constructor
  · native_decide
  constructor
  · constructor <;> native_decide
  constructor <;> native_decide

end Chimera.ZigAdapter
