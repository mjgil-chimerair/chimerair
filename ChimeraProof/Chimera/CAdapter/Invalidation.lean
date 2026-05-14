-- CAdapter Invalidation module
-- Task 136: Add `Chimera/CAdapter` Lean namespace

import Lean

namespace Chimera.CAdapter

/--
C Invalidation event types
-/
inductive InvalidationKind where
  | header_changed
  | layout_changed
  | abi_changed
deriving Inhabited

end Chimera.CAdapter