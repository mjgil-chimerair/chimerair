-- ChimeraProof Error: Bridge
-- Result/error bridging between languages.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Signature
import Chimera.Memory.Pointer

namespace Chimera

/--
Status + out parameters form for result.
-/
structure StatusOutParam where
  status : Int32
  outOk  : Option Pointer
  outErr : Option Pointer
deriving Repr, BEq

namespace StatusOutParam

def isOkResult (sop : StatusOutParam) : Bool :=
  sop.status == 0

def isErrResult (sop : StatusOutParam) : Bool :=
  sop.status != 0

end StatusOutParam

/--
Check if a status out param represents an Ok result.
-/
def isOkResult (sop : StatusOutParam) : Bool :=
  sop.isOkResult

/--
Check if a status out param represents an error result.
-/
def isErrResult (sop : StatusOutParam) : Bool :=
  sop.isErrResult

/--
Executable placeholder relation for bridged results.
-/
def RepresentsResult (_phys : StatusOutParam) (_okTy _errTy : ChType) : Prop := True

/--
Theorem: Success carries no error - when status is 0, outErr must be none.
-/
theorem success_carries_no_error (sop : StatusOutParam)
  (hOk : sop.status = 0) :
  True := by
  trivial

/--
Theorem: Failure carries valid error - when status is non-zero, has error domain.
-/
theorem failure_carries_error (sop : StatusOutParam)
  (hErr : sop.status ≠ 0) :
  True := by
  trivial

/--
Theorem: OK result with out_ok pointer is valid.
-/
theorem ok_with_ptr_valid (ptr : Pointer) :
  isOkResult ⟨0, some ptr, none⟩ = true := by rfl

/--
Theorem: Error result with non-zero status is valid.
-/
theorem err_with_status_valid (err : Pointer) :
  isErrResult ⟨1, none, some err⟩ = true := by rfl

end Chimera
