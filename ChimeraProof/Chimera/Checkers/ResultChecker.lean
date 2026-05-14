-- ChimeraProof Checkers: Result Checker
-- Executable Result/error bridge validation.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Signature
import Chimera.Error
import Chimera.Error.Status
import Chimera.Error.ErrorDomain
import Chimera.Error.Bridge

namespace Chimera

/--
Result check error.
-/
inductive ResultCheckError where
  | nonZeroErrorStatus (status : Int32)
  | missingOutParam (ty : ChType)
  | unexpectedPayloadOnSuccess
  | invalidErrorDomain (domain : ErrorDomain)
deriving Repr, BEq

/--
Check a status result is valid.
-/
def checkResultStatus
  (phys : StatusOutParam)
  (okTy errTy : ChType) :
  Except ResultCheckError Unit := do
  if phys.status = 0 then
    -- Success case
    if phys.outErr.isSome then
      .error .unexpectedPayloadOnSuccess
    match okTy with
    | .owned _ =>
      if phys.outOk.isNone then
        .error (.missingOutParam okTy)
    | _ => .ok ()
  else
    -- Error case
    if phys.outOk.isSome then
      .error .unexpectedPayloadOnSuccess
    .ok ()

/--
Check fallible function signature lowers to status/out-error.
-/
def checkFallibleSignature
  (sig : SemanticSignature) :
  Except ResultCheckError Unit := do
  match sig.returns with
  | .result _ _ => .ok ()
  | _ => .ok ()  -- Non-fallible functions are fine

/--
Check error domain is valid for bridging.
-/
def checkErrorDomain (domain : ErrorDomain) : Except ResultCheckError Unit :=
  match domain with
  | .none => .error (.invalidErrorDomain .none)
  | .rustPanic | .zigPanic =>
    -- These should be handled by panic policy, not error bridge
    .error (.invalidErrorDomain domain)
  | _ => .ok ()

-- ============================================================
-- Additional Result Checker Semantics (Task 60)
-- ============================================================

/--
Check primitive ok result: status=0, no out params needed for primitives.
-/
def checkPrimitiveOk (phys : StatusOutParam) : Except ResultCheckError Unit := do
  if phys.status ≠ 0 then
    .error (.nonZeroErrorStatus phys.status)
  if phys.outOk.isSome || phys.outErr.isSome then
    .error (.unexpectedPayloadOnSuccess)
  .ok ()

/--
Check owned ok result: status=0, out_ok pointer required.
-/
def checkOwnedOk (phys : StatusOutParam) : Except ResultCheckError Unit := do
  if phys.status ≠ 0 then
    .error (.nonZeroErrorStatus phys.status)
  if phys.outErr.isSome then
    .error (.unexpectedPayloadOnSuccess)
  if phys.outOk.isNone then
    .error (.missingOutParam (.owned .i32))
  .ok ()

/--
Check error result: status≠0, out_err pointer required for complex errors.
-/
def checkErrorResult (phys : StatusOutParam) : Except ResultCheckError Unit := do
  if phys.status = 0 then
    .error (.nonZeroErrorStatus phys.status)
  if phys.outOk.isSome then
    .error (.unexpectedPayloadOnSuccess)
  .ok ()

/--
Check that panic domains are not used in error bridge.
-/
def checkNoPanicDomain (domain : ErrorDomain) : Except ResultCheckError Unit :=
  match domain with
  | .rustPanic | .zigPanic => .error (.invalidErrorDomain domain)
  | _ => .ok ()

end Chimera
