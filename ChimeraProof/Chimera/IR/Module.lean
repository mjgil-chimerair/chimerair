-- ChimeraProof IR: Module
-- ChimeraIR module representation.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Contract
import Chimera.Memory

namespace Chimera

/--
Type declaration.
-/
structure TypeDecl where
  name : Symbol
  ty : ChType
  size : Nat
  align : Nat

/--
Layout declaration with full field details.
Hash field is for cache/diagnostics only - NOT for proof obligations.
Proof must use structural equality of fields, not hash.
-/
structure DeclaredLayout where
  name : Symbol
  size : Nat
  align : Nat
  hash : Nat  -- cache/diagnostics only, not proof
  fields : List DeclaredField

/--
Declared field with name, offset, physical type, size, and alignment.
-/
structure DeclaredField where
  name : String
  offset : Nat
  ty : PhysType
  size : Nat
  align : Nat

/--
Import specification.
-/
structure Import where
  symbol : Symbol
  contract : FunctionContract

/--
Export specification.
-/
structure Export where
  symbol : Symbol
  contract : FunctionContract

/--
ChimeraIR module.
-/
structure Module where
  abiVersion : String
  moduleName : Symbol
  language : SourceLanguage
  target : Target
  exports : List Export
  imports : List Import
  types : List TypeDecl
  layouts : List DeclaredLayout

namespace Module

/--
Check if module is empty.
-/
def isEmpty (m : Module) : Bool :=
  m.exports.isEmpty ∧ m.imports.isEmpty ∧ m.types.isEmpty

/--
Get all symbols defined in this module.
-/
def definedSymbols (m : Module) : List Symbol :=
  m.exports.map (·.symbol)

/--
Get all symbols imported by this module.
-/
def importedSymbols (m : Module) : List Symbol :=
  m.imports.map (·.symbol)

end Module

/--
Structural equality for DeclaredLayout - used for proofs, NOT hash.
Hash can be used as a quick diagnostic filter but NOT for proof obligations.
-/
def DeclaredLayout.structurallyEquals (a b : DeclaredLayout) : Bool :=
  match a, b with
  | ⟨aName, aSize, aAlign, _, _⟩, ⟨bName, bSize, bAlign, _, _⟩ =>
      aName == bName && aSize == bSize && aAlign == bAlign

/--
Structural equality for DeclaredField - used for proofs.
-/
def DeclaredField.structurallyEquals (a b : DeclaredField) : Bool :=
  match a, b with
  | ⟨aName, aOffset, aTy, aSize, aAlign⟩, ⟨bName, bOffset, bTy, bSize, bAlign⟩ =>
      aName == bName && aOffset == bOffset && aTy == bTy && aSize == bSize && aAlign == bAlign

end Chimera
