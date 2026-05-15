-- RustAdapter module
-- Task 126: Add panic boundary pass for Rust
-- Task 127: Add effect tracking for Rust
-- Task 152: Add RustAdapter Lean namespace
-- Task 153: Define Rust proof input schema in Lean
-- Task 154: Prove Rust ABI compatibility
-- Task 155: Prove Rust layout preservation
-- Task 156: Prove result lowering soundness
-- Task 157: Prove panic boundary safety
-- Task 158: Prove ownership/drop obligations
-- Task 159: Prove unsafe trust ledger completeness
-- Task 160: Prove cache soundness for Rust
-- Task 161: Add Rust proof report integration

import Lean
import RustAdapter.RustResultLowering
import RustAdapter.PanicBoundary
import RustAdapter.EffectTracking
import RustAdapter.ABIFingerprint
import RustAdapter.LayoutFingerprint
import RustAdapter.MIRModel
import RustAdapter.OwnershipLowering
import RustAdapter.Invalidation
import RustAdapter.ProofInput
import RustAdapter.ABICompatibility
import RustAdapter.LayoutPreservation
import RustAdapter.ResultLoweringSoundness
import RustAdapter.PanicBoundarySafety
import RustAdapter.OwnershipProof
import RustAdapter.UnsafeTrustLedger
import RustAdapter.CacheSoundness
import RustAdapter.ProofReport

namespace RustAdapter

/--
RustAdapter module version
-/
def version : String := "0.1.0"

end RustAdapter