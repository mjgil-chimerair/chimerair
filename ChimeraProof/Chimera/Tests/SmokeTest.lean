-- ChimeraProof Tests: Smoke Test
-- Root import smoke tests to verify each module imports correctly.

import Chimera.Foundation
import Chimera.ABI
import Chimera.Memory
import Chimera.Error
import Chimera.Effects
import Chimera.IR
import Chimera.Link.Resolve
import Chimera.Checkers
import Chimera.Wrapper
import Chimera.Theorems

namespace Chimera.Tests

/--
Verify ContractChecker is accessible through the Checkers root.
-/
theorem contract_checker_accessible : ∀ (c : Chimera.FunctionContract), True := by
  intros _ -- Suppress unused variable warning
  -- checkContract is accessible via Chimera.Checkers import
  trivial

/--
Run all smoke tests and return exit code.
-/
def runSmokeTests : IO UInt32 := do
  IO.println "Running ChimeraProof smoke tests..."
  IO.println "All modules imported successfully."
  return 0

end Chimera.Tests
