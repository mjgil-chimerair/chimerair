-- ChimeraProof Link: Safety
-- Formal safety invariants for link modes.

import Chimera.Foundation
import Chimera.ABI
import Chimera.Link.Component
import Chimera.Link.Resolve

namespace Chimera

/--
A link mode is safe if all required obligations for that mode are met.
-/
def isLinkModeSafe (mode : LinkMode) (edge : AbiEdge) (plan : LinkPlan) : Prop :=
  match mode with
  | .directLink | .staticLink =>
      -- Every symbol in the edge must be in the plan and have a valid resolution
      ∀ sym ∈ edge.symbols, ∃ entry ∈ plan.entries, entry.import_ == sym
  | .generatedWrapper =>
      -- Must have a wrapper and proof, and proof must be verified
      -- (This is modeled by the existence of a valid resolution in the plan for the wrapper symbols)
      ∀ sym ∈ edge.symbols, ∃ entry ∈ plan.entries, entry.import_ == sym
  | .runtimeDlopen =>
      -- Symbols are NOT linked directly, but must be available at runtime
      True
  | .dynamicLink =>
      -- Symbols linked against shared lib
      ∀ sym ∈ edge.symbols, ∃ entry ∈ plan.entries, entry.import_ == sym

/--
Theorem: If symbols are resolved via resolveSymbols, then directLink is safe.
-/
theorem resolve_implies_direct_link_safe (tbl : SymbolTable) (edge : AbiEdge) (h_mode : edge.mode = .directLink) :
  (∃ plan_list, ∃ plan, resolveSymbols tbl = .ok plan_list ∧ plan = { entries := plan_list.map (fun r => { import_ := r.symbol, resolvedTo := r }) }) → 
  ∃ plan, isLinkModeSafe .directLink edge plan := by
  sorry -- OPEN: the direct-link safety theorem still needs a full proof over resolveSymbols.

end Chimera
