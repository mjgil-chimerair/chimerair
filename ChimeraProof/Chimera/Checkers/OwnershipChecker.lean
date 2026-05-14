-- ChimeraProof Checkers: Ownership Checker
-- Executable ownership boundary validation.

import Chimera.Foundation
import Chimera.ABI
import Chimera.Memory
import Chimera.Memory.Permission

namespace Chimera

/--
Ownership check error.
-/
inductive OwnershipCheckError where
  | doubleOwn (block : BlockId)
  | writeBorrowAlias (block : BlockId)
  | borrowEscapes (block : BlockId)
  | droppedUse (block : BlockId)
  | movedUse (block : BlockId)
  | noOwner (block : BlockId)
  | callLifetimeReturn (ty : ChType)
deriving Repr, BEq

/--
Check if a resource list has no double ownership.
-/
def checkNoDoubleOwn (resources : List Resource) : Except OwnershipCheckError Unit :=
  go resources []
where
  go : List Resource → List BlockId → Except OwnershipCheckError Unit
    | [], _ => .ok ()
    | r :: rest, owners =>
      if r.permission.transfersOwnership then
        if owners.contains r.block then
          .error (.doubleOwn r.block)
        else
          go rest (r.block :: owners)
      else
        go rest owners

/--
Check if a resource list has no write borrow aliasing.
-/
def checkNoWriteBorrowAlias (resources : List Resource) : Except OwnershipCheckError Unit :=
  go resources []
where
  go : List Resource → List BlockId → Except OwnershipCheckError Unit
    | [], _ => .ok ()
    | r :: rest, writeBlocks =>
      match r.permission with
      | .writeBorrow =>
        if writeBlocks.contains r.block then
          .error (.writeBorrowAlias r.block)
        else
          go rest (r.block :: writeBlocks)
      | _ => go rest writeBlocks

/--
Check call-lifetime borrows don't escape in return type.
A call-lifetime borrow inside a result, owned wrapper, slice, or string escapes.
-/
def checkNoCallLifetimeEscape (retTy : ChType) : Except OwnershipCheckError Unit :=
  if containsEscapingBorrow retTy then
    .error (.callLifetimeReturn retTy)
  else
    .ok ()

/--
Full ownership check on a call state.
-/
def checkOwnership (s : CallState) (c : CallContract) : Except OwnershipCheckError Unit := do
  checkNoDoubleOwn s.resources
  checkNoWriteBorrowAlias s.resources
  .ok ()

end Chimera
