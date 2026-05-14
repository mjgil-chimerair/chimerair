-- ChimeraProof CAdapter module
-- Task 136: Add `Chimera/CAdapter` Lean namespace

import Chimera.Foundation
import Chimera.CAdapter.Snapshot
import Chimera.CAdapter.DependencyGraph
import Chimera.CAdapter.Dialect
import Chimera.CAdapter.ProofBridge
import Chimera.CAdapter.ProofInput
import Chimera.CAdapter.LayoutPreservation
import Chimera.CAdapter.Architecture
import Chimera.CAdapter.ABIFingerprint
import Chimera.CAdapter.LayoutFingerprint
import Chimera.CAdapter.ABIPreservation
import Chimera.CAdapter.ErrorBridge
import Chimera.CAdapter.ErrorPreservation
import Chimera.CAdapter.PointerContracts
import Chimera.CAdapter.AllocatorContracts
import Chimera.CAdapter.CachePreservation
import Chimera.CAdapter.WrapperPreservation
import Chimera.CAdapter.ProofReport
import Chimera.CAdapter.MLIRAttributes
import Chimera.CAdapter.LayoutMaterialization
import Chimera.CAdapter.PointerVerification
import Chimera.CAdapter.ResultLowering
import Chimera.CAdapter.AllocatorVerification
import Chimera.CAdapter.EffectDeclarations
import Chimera.CAdapter.ProofObligations
import Chimera.CAdapter.COriginLowering
import Chimera.CAdapter.Varargs
import Chimera.CAdapter.Cache
import Chimera.CAdapter.Invalidation

namespace Chimera.CAdapter

/--
CAdapter module version
-/
def version : String := "0.1.0"

end Chimera.CAdapter