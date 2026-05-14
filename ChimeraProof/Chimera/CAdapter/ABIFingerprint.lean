-- CAdapter ABIFingerprint module
-- Task 136: Add `Chimera/CAdapter` Lean namespace

import Lean

namespace Chimera.CAdapter

/--
C ABI Fingerprint - computed from C declarations/types
-/
structure ABIFingerprint where
  hash : String
  declarations : List String
deriving Inhabited, Repr, BEq, DecidableEq

end Chimera.CAdapter