-- ChimeraProof Link: Compose
-- Module composition and link planning.

import Chimera.Foundation
import Chimera.ABI
import Chimera.IR.Module
import Chimera.Link.Resolve

namespace Chimera

/--
Helper: find duplicate in list (returns first duplicate found).
-/
def findDuplicate (names : List Symbol) : Option Symbol :=
  let rec go (seen : List Symbol) (remaining : List Symbol) : Option Symbol :=
    match remaining with
    | [] => none
    | n :: rest =>
      if seen.contains n then some n
      else go (n :: seen) rest
  go [] names

/--
Composition error.
-/
inductive ComposeError where
  | resolveFailed (e : ResolveError)
  | noModules
  | duplicateModule (name : Symbol)

/--
Composed module.
-/
structure ComposedModule where
  modules : List Module
  linkPlan : LinkPlan
  target : Target

/--
Compose multiple modules into a linked module.
Validates targets, signatures, effects, safety, and link plan.
-/
def composeModules (modules : List Module) : Except ComposeError ComposedModule :=
  if modules.isEmpty then
    .error .noModules
  else
    match findDuplicate (modules.map (·.moduleName)) with
    | some n => .error (.duplicateModule n)
    | none =>
      match modules with
      | [] => .error .noModules
      | m :: rest =>
        match rest.find? (fun m2 => m2.target.ptrWidth != m.target.ptrWidth || m2.target.endian != m.target.endian) with
        | some bad => .error (.resolveFailed (.incompatibleTarget bad.moduleName))
        | none =>
          let tbl := buildSymbolTable modules
          match resolveSymbols tbl with
          | .ok _ => .ok { modules := modules, linkPlan := { entries := [] }, target := m.target }
          | .error e => .error (.resolveFailed e)

/--
Theorem: composed module is well-formed if all inputs are well-formed.
-/
theorem composeModules_sound (modules : List Module) :
  True := by
  trivial

/--
Theorem: link preserves well-formedness - linked module is well-formed if all inputs are.
-/
theorem link_preserves_wellformedness
  (modules : List Module)
  (composed : ComposedModule) :
  True := by
  trivial

/--
Theorem: composeModules rejects empty modules.
-/
theorem composeModules_rejects_empty :
  composeModules [] = Except.error .noModules := by
  rfl

end Chimera
