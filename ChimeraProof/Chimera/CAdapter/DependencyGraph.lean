-- CAdapter Dependency Graph module
-- Task 136: Add `Chimera/CAdapter` Lean namespace

import Lean

namespace Chimera.CAdapter

inductive NodeKind where
  | source
  | header
  | macro
deriving Repr, BEq, DecidableEq

structure DependencyNode where
  id : String
  kind : NodeKind
deriving Repr, BEq

structure DependencyGraph where
  nodes : List DependencyNode
deriving Repr, BEq

end Chimera.CAdapter