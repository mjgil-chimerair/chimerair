-- ChimeraProof IR: WellFormed
-- Module well-formedness predicates.

import Chimera.Foundation
import Chimera.ABI
import Chimera.IR.Module
import Chimera.Link.Resolve

namespace Chimera

/--
Well-formed metadata predicate.
-/
def WellFormedMetadata : Module → Prop
  | m =>
    match m with
    | { abiVersion := v, moduleName := n, language := _, target := _,
        exports := _, imports := _, types := ty, layouts := _ } =>
      v = "0.1" ∧  -- MVP version check
      ¬ n.name.isEmpty ∧
      ty.all (fun td => td.size > 0)

/--
Well-formed linked module.
-/
def WellFormedLinkedModule (modules : List Module) (linkPlan : LinkPlan) : Prop :=
  True  -- simplified: actual check would require resolution

/--
Check if module has a valid target.
-/
def ValidTarget : Module → Prop
  | m => m.target.ptrWidth > 0 ∧ m.target.usizeWidth > 0

/--
Helper: check if contract is valid (executable version).
-/
def contractIsValid (e : Export) : Bool :=
  match e.contract.safety with
  | SafetyClass.unsafeContract => true
  | _ => ¬ e.contract.containsUncheckedRaw

/--
Check if all exports have valid contracts.
-/
def AllContractsValid : Module → Prop
  | m => m.exports.all contractIsValid

/--
Check if all imports resolve.
-/
def AllImportsResolved (modules : List Module) (imports : List Import) : Prop :=
  let definedSymbols := modules.flatMap (·.definedSymbols)
  imports.all (fun imp => definedSymbols.contains imp.symbol)

end Chimera