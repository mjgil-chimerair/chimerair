-- CAdapter Architecture module
-- Task 136: Add `Chimera/CAdapter` Lean namespace

import Lean

namespace Chimera.CAdapter

/--
C Architecture information
-/
structure Architecture where
  name : String
  pointer_width : Nat
deriving Inhabited, Repr, BEq, DecidableEq

end Chimera.CAdapter