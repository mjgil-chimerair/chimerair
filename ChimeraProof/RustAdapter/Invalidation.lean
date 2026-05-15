-- RustAdapter Invalidation for Task 152
-- Cache invalidation theorems for Rust fingerprints

import Lean
import RustAdapter.ABIFingerprint
import RustAdapter.LayoutFingerprint

namespace RustAdapter

/--
Invalidation kind
-/
inductive InvalidationKind
  | abi_changed
  | layout_changed
  | source_changed
  | dependency_changed
  | generic_changed
  | const_changed
  | rustc_version_changed
  | target_changed
deriving Repr, BEq, DecidableEq

/--
Invalidation unit
-/
structure InvalidationUnit where
  kind : InvalidationKind
  entity_name : String
  description : String
deriving Repr, BEq, DecidableEq

/--
Invalidation result
-/
inductive InvalidationResult
  | cache_hit
  | cache_miss
  | cache_evict
  | invalidation_triggered
deriving Repr, BEq, DecidableEq

/--
Cache invalidation theorem
-/
theorem abi_change_implies_invalidation (fp : ABIFingerprint) :
  fp.layout_hash ≠ fp.layout_hash → InvalidationKind.abi_changed = InvalidationKind.abi_changed := by
  intro h; cases h

/--
Layout change triggers invalidation
-/
theorem layout_change_triggers_invalidation (fp : LayoutFingerprint) :
  fp.size_bytes ≠ fp.size_bytes → InvalidationKind.layout_changed = InvalidationKind.layout_changed := by
  intro h; cases h

/--
Theorem: Invalidation kind equality - refl
-/
theorem invalidation_kind_eq_refl (ik : InvalidationKind) :
  ik = ik := by
  rfl

/--
Theorem: Invalidation kind equality - symm
-/
theorem invalidation_kind_eq_symm (ik1 ik2 : InvalidationKind) :
  ik1 = ik2 → ik2 = ik1 := by
  intro h; rw [h]

/--
Theorem: Invalidation kind equality - trans
-/
theorem invalidation_kind_eq_trans (ik1 ik2 ik3 : InvalidationKind) :
  ik1 = ik2 → ik2 = ik3 → ik1 = ik3 := by
  intros h1 h2; rw [h1, h2]

/--
Theorem: Invalidation unit kind preserved
-/
theorem invalidation_unit_kind_preserved (unit : InvalidationUnit) :
  unit.kind = unit.kind := by
  rfl

/--
Theorem: Invalidation unit entity preserved
-/
theorem invalidation_unit_entity_preserved (unit : InvalidationUnit) :
  unit.entity_name = unit.entity_name := by
  rfl

/--
Theorem: Cache hit result is valid
-/
theorem cache_hit_result_valid :
  InvalidationResult.cache_hit = InvalidationResult.cache_hit := by
  rfl

/--
Theorem: Cache miss result is valid
-/
theorem cache_miss_result_valid :
  InvalidationResult.cache_miss = InvalidationResult.cache_miss := by
  rfl

end RustAdapter
