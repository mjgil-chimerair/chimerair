-- ChimeraProof Checkers: Allocator Checker
-- Executable allocator validation.

import Chimera.Foundation
import Chimera.ABI
import Chimera.Memory
import Chimera.Memory.Allocator

namespace Chimera

/--
Allocator check error.
-/
inductive AllocatorCheckError where
  | mismatchedAllocator (block : BlockId)
  | noDropFunction (ty : ChType)
  | invalidDrop (block : BlockId)
deriving Repr, BEq

/--
Check if allocation matches free.
-/
def checkAllocFreeMatch
  (reg : AllocRegistry)
  (block : BlockId)
  (allocId : AllocatorId) :
  Except AllocatorCheckError Unit := do
  match reg.findAllocator? block with
  | some aid =>
    if aid == allocId then .ok ()
    else .error (.mismatchedAllocator block)
  | none => .ok ()

/--
Check that a type has a drop path if it requires one.
-/
def checkDropPath
  (dropReg : DropRegistry)
  (allocReg : AllocRegistry)
  (ty : ChType)
  (block : BlockId) :
  Except AllocatorCheckError Unit := do
  match dropReg.findDropFn? ty with
  | some df =>
    match allocReg.findAllocator? block with
    | some aid =>
      match df.allocator with
      | some dfAlloc =>
        if aid == dfAlloc then .ok ()
        else .error (.mismatchedAllocator block)
      | none => .ok ()
    | none => .ok ()
  | none => .ok ()

/--
Check owned opaque return has drop function.
-/
def checkOwnedOpaqueHasDrop
  (reg : DropRegistry)
  (ty : ChType) :
  Except AllocatorCheckError Unit := do
  match ty with
  | .owned (.opaque _) =>
    match reg.findDropFn? ty with
    | some _ => .ok ()
    | none => .error (.noDropFunction ty)
  | _ => .ok ()

end Chimera