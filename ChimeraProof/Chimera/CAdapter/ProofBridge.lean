-- CAdapter ProofBridge module
-- Task 136: Add `Chimera/CAdapter` Lean namespace

import Lean

namespace Chimera.CAdapter

/--
C Proof Bridge - maps C proof obligations to Lean theorems
-/
structure ProofBridge where
  layout_proofs : List LayoutProof
  signature_proofs : List SignatureProof
deriving Inhabited

/--
Layout proof obligation
-/
structure LayoutProof where
  struct_name : String
  size_bytes : Nat
  proved : Bool
deriving Inhabited

/--
Signature proof obligation
-/
structure SignatureProof where
  function_name : String
  proved : Bool
deriving Inhabited

/--
Nullability status
-/
inductive Nullability where
  | nullable
  | nonnull
deriving Inhabited

/--
Ownership semantics
-/
inductive Ownership where
  | borrowed
  | owned
deriving Inhabited

end Chimera.CAdapter