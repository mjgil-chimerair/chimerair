-- ChimeraProof Tests: Effects Tests
-- Tests for effect inference and lattice.

import Chimera.Effects
import Chimera.ABI

namespace Chimera.Test

namespace EffectInferenceTest

theorem owned_infers_alloc_dealloc :
    inferFromType (.owned .i32) = { mayAlloc := true, mayDealloc := true } := by
  rfl

theorem rawptr_infers_raw :
    inferFromType (.rawptr .i32) = { mayTouchRaw := true } := by
  rfl

theorem primitive_infurses_nothing :
    inferFromType .i32 = {} := by
  rfl

theorem result_merges_effects :
    inferFromType (.result (.owned .i32) (.rawptr .i32)) =
      { mayAlloc := true, mayDealloc := true, mayTouchRaw := true } := by
  rfl

theorem infer_to_effect_set :
    InferredEffects.toEffectSet { mayAlloc := true, mayPanic := true, threadSafe := true } =
      [.threadSafe, .mayPanic, .mayAlloc] := by
  rfl

theorem merge_combines_effects :
    InferredEffects.merge { mayAlloc := true, threadSafe := true }
      { mayPanic := true, threadSafe := false } =
      { mayAlloc := true, mayPanic := true, threadSafe := false } := by
  rfl

end EffectInferenceTest

namespace EffectLatticeTest

theorem canonicalize_removes_dup :
    EffectSet.canonicalize [.mayAlloc, .mayAlloc, .pure] = [.pure, .mayAlloc] := by
  rfl

theorem canonicalize_idempotent :
    EffectSet.canonicalize (EffectSet.canonicalize [.mayAlloc, .mayAlloc, .pure]) =
      EffectSet.canonicalize [.mayAlloc, .mayAlloc, .pure] := by
  exact EffectLatticeLaws.canonicalize_idempotent _

theorem same_effects_same_canonical :
    EffectSet.canonicalize [.mayAlloc, .pure, .mayAlloc] =
      EffectSet.canonicalize [.pure, .mayAlloc] := by
  exact EffectLatticeLaws.EffectSubset_antisymmetric _ _
    (by
      intro e hMem
      cases e <;> simp [memberEffect] at hMem ⊢)
    (by
      intro e hMem
      cases e <;> simp [memberEffect] at hMem ⊢)

theorem empty_is_canonical :
    EffectSet.canonicalize [] = [] := by
  rfl

theorem subset_reflexive :
    EffectSubset [.pure, .mayAlloc] [.pure, .mayAlloc] := by
  exact EffectLatticeLaws.EffectSubset_reflexive _

theorem subset_transitive :
    EffectSubset [.pure] [.pure, .mayAlloc] →
      EffectSubset [.pure, .mayAlloc] [.pure, .mayAlloc, .mayPanic] →
      EffectSubset [.pure] [.pure, .mayAlloc, .mayPanic] := by
  intro hAB hBC
  exact EffectLatticeLaws.EffectSubset_transitive _ _ _ hAB hBC

theorem compose_contains_left_operand :
    EffectSubset [.pure] (composeEffectSets [[.pure], [.mayAlloc]]) := by
  exact EffectLatticeLaws.composeEffectSets_contains_left _ _

theorem compose_contains_right_operand :
    EffectSubset [.mayAlloc] (composeEffectSets [[.pure], [.mayAlloc]]) := by
  exact EffectLatticeLaws.composeEffectSets_contains_right _ _

end EffectLatticeTest

namespace PureEffectTest

theorem pure_is_pure : isPure [.pure] = true := by
  rfl

theorem alloc_not_pure : isPure [.mayAlloc] = false := by
  rfl

theorem mixed_not_pure : isPure [.pure, .mayAlloc] = false := by
  rfl

theorem multiple_not_pure : isPure [.pure, .pure] = false := by
  rfl

theorem pure_no_alloc :
    memberEffect [.pure] .mayAlloc = false := by
  rfl

end PureEffectTest

end Chimera.Test
