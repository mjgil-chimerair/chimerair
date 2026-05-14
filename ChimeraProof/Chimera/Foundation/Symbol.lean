namespace Chimera

/--
A Symbol represents a named entity in the IR.
Symbols have a namespace and a name, with an optional FQN for ABI rendering.
-/
structure Symbol where
  ns : String
  name : String
deriving Repr, BEq, Hashable

namespace Symbol

/--
Fully-qualified name for ABI rendering.
Returns "ns::name" or just "name" if namespace is empty.
-/
def fqn (s : Symbol) : String :=
  if s.ns.isEmpty then s.name else s.ns ++ "::" ++ s.name

/--
Create a simple symbol with no namespace.
-/
def simple (name : String) : Symbol :=
  ⟨"", name⟩

/--
Create a namespaced symbol.
-/
def namespaced (ns name : String) : Symbol :=
  ⟨ns, name⟩

/--
Check if symbol has a namespace.
-/
def hasNamespace (s : Symbol) : Bool :=
  ¬ s.ns.isEmpty

/--
Reserved namespace prefixes that cannot be used.
-/
def isReserved (s : Symbol) : Bool :=
  s.ns = "chimera" ∨ s.ns = "llvm" ∨ s.ns = "mlir"

/--
Check if symbol name is valid (non-empty).
-/
def isValidName (s : Symbol) : Bool :=
  ¬ s.name.isEmpty

/--
Create an anonymous symbol (for internal use only).
-/
def anonymous : Symbol :=
  ⟨"", "anon"⟩

end Symbol

inductive SymbolStrength : Type where
| strong
| weak
| linkonce
deriving BEq

inductive SymbolVisibility : Type where
| vis_private
| vis_public
| vis_exported
deriving BEq

end Chimera
