-- CAdapter Dialect module
-- C dialect representation in Lean

import Lean

namespace Chimera.CAdapter

/--
C Dialect type representation
-/
inductive CType where
  | void
  | char
  | int
  | pointer (inner : CType)
deriving Inhabited

/--
C Dialect declaration
-/
inductive CDecl where
  | function (name : String) (ret : CType)
  | variable (name : String) (type : CType)
deriving Inhabited

end Chimera.CAdapter