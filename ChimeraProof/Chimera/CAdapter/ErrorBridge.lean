-- CAdapter ErrorBridge module
-- Task 136: Add `Chimera/CAdapter` Lean namespace

import Lean

namespace Chimera.CAdapter

/--
C Error Bridge - errno/status mapping representation
-/
structure ErrorBridge where
  function_name : String
  maps_errno : Bool
deriving Inhabited, Repr, BEq, DecidableEq

end Chimera.CAdapter