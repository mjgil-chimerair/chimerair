-- CAdapter LayoutFingerprint module
-- Task 136: Add `Chimera/CAdapter` Lean namespace

import Lean

namespace Chimera.CAdapter

/--
C Layout Fingerprint - struct/layout ABI fingerprint
-/
structure LayoutFingerprint where
  struct_name : String
  size_bytes : Nat
deriving Inhabited

end Chimera.CAdapter