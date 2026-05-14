-- ChimeraProof Error: Status
-- Status code representation for error bridging.

namespace Chimera

/--
Chimera status codes.
-/
inductive Status where
  | ok      -- 0, success
  | err     -- non-zero, error
deriving Repr, BEq

/--
Status code as used in FFI.
-/
structure StatusCode where
  value : Int32
deriving Repr, BEq

namespace StatusCode

/--
Success status (0).
-/
def success : StatusCode := ⟨0⟩

/--
Error status (non-zero).
Input code must be non-zero (error code 0 is not valid).
-/
def error (code : Nat) (h : code ≠ 0) : StatusCode := ⟨Int32.ofNat (code % (2^31 - 1) + 1)⟩

/--
Check if status is success.
-/
def isOk (s : StatusCode) : Bool := s.value = 0

/--
Check if status is error.
-/
def isErr (s : StatusCode) : Bool := s.value ≠ 0

end StatusCode

end Chimera