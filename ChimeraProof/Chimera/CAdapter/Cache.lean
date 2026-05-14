-- CAdapter Cache module
-- Task 136: Add `Chimera/CAdapter` Lean namespace

import Lean

namespace Chimera.CAdapter

/--
C Cache key components
-/
structure CacheKeyComponents where
  source_file : String
  target_triple : String
deriving Inhabited

/--
Cached artifact types
-/
inductive CachedArtifactType where
  | snapshot
  | castpack
deriving Inhabited

end Chimera.CAdapter