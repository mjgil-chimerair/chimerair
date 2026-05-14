-- CAdapter Snapshot module
-- Task 136: Add `Chimera/CAdapter` Lean namespace

import Lean

namespace Chimera.CAdapter

/--
C Declaration kind
-/
inductive DeclarationKind where
  | function
  | variable
  | struct_
  | union_
  | enum_
  | typedef_
deriving Repr, BEq, DecidableEq

/--
C Target information
-/
structure Target where
  triple : String
  pointer_width : Nat
deriving Repr, BEq

/--
C Declaration
-/
structure Declaration where
  name : String
  kind : DeclarationKind
deriving Repr, BEq

/--
C Semantic Snapshot
Represents the extracted C AST/types/decls from Clang
-/
structure Snapshot where
  source_files : List String
  headers : List String
  declarations : List Declaration
  target : Target
deriving Repr, BEq

/--
Empty snapshot
-/
def emptySnapshot : Snapshot := ⟨[], [], [], { triple := "", pointer_width := 0 }⟩

end Chimera.CAdapter