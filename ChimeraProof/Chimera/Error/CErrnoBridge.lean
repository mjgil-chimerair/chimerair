-- ChimeraProof Error: C Errno Bridge
-- C errno error bridging model.

import Chimera.Foundation
import Chimera.Error.Status
import Chimera.Error.ChError
import Chimera.Error.ErrorDomain

namespace Chimera

/--
C errno mapping metadata.
Maps status codes to canonical errno values.
-/
structure CErrnoMapEntry where
  statusCode : Nat
  errnoValue : Nat
  message : String

/--
C errno bridge - converts status codes to C errors.
-/
structure CErrnoBridge where
  entries : List CErrnoMapEntry

namespace CErrnoBridge

/--
Empty errno bridge.
-/
def empty : CErrnoBridge := ⟨[]⟩

/--
Add a mapping entry.
-/
def addMapping (bridge : CErrnoBridge) (statusCode errnoValue : Nat) (msg : String) : CErrnoBridge :=
  ⟨{ statusCode := statusCode, errnoValue := errnoValue, message := msg } :: bridge.entries⟩

/--
Find errno mapping for a status code.
-/
def findErrno? (bridge : CErrnoBridge) (statusCode : Nat) : Option Nat :=
  bridge.entries.find? (fun e => e.statusCode = statusCode) |>.map (·.errnoValue)

/--
Convert status code to C error domain.
-/
def toErrorDomain (bridge : CErrnoBridge) (statusCode : Nat) (code : Nat) : ChError :=
  match bridge.findErrno? statusCode with
  | some errno =>
    ChError.generic s!"C error: errno {errno}"
  | none =>
    ChError.generic s!"Unknown C error {code}"

end CErrnoBridge

end Chimera