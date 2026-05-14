-- CAdapter Varargs module
-- Task 136: Add `Chimera/CAdapter` Lean namespace

import Lean

namespace Chimera.CAdapter

/--
C Varargs representation
-/
structure VarargsInfo where
  function_name : String
  is_varargs : Bool
deriving Inhabited

end Chimera.CAdapter