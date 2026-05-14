-- ChimeraProof Link: Symbol Table
-- Symbol table management.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Contract
import Chimera.Memory

namespace Chimera

/--
Symbol definition.
-/
structure SymbolDef where
  symbol : Symbol
  contract : FunctionContract
  sourceModule : Symbol
  target : Target
  strength : SymbolStrength
  visibility : SymbolVisibility

/--
Unresolved import reference with module provenance.
-/
structure ImportRef where
  symbol : Symbol
  contract : FunctionContract
  sourceModule : Symbol
  target : Target

/--
Symbol table.
-/
structure SymbolTable where
  defs : List SymbolDef
  imports : List ImportRef

namespace SymbolTable

/--
Empty symbol table.
-/
def empty : SymbolTable := { defs := [], imports := [] }

/--
Add a definition.
-/
def addDef (tbl : SymbolTable) (defn : SymbolDef) : SymbolTable :=
  { defs := defn :: tbl.defs, imports := tbl.imports }

/--
Add an unresolved import.
-/
def addImport (tbl : SymbolTable) (imp : ImportRef) : SymbolTable :=
  { defs := tbl.defs, imports := imp :: tbl.imports }

/--
Find a definition by symbol.
-/
def findDef? (tbl : SymbolTable) (sym : Symbol) : Option SymbolDef :=
  tbl.defs.find? (fun d => d.symbol == sym)

/--
Find all definitions for a symbol (may be multiple with different strengths).
-/
def findDefs (tbl : SymbolTable) (sym : Symbol) : List SymbolDef :=
  tbl.defs.filter (fun d => d.symbol == sym)

end SymbolTable

end Chimera
