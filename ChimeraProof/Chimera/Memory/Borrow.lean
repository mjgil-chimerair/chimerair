-- ChimeraProof Memory: Borrow
-- Borrow checking logic.

import Chimera.Foundation
import Chimera.ABI
import Chimera.Memory.Block
import Chimera.Memory.Permission
import Chimera.Memory.Ownership

namespace Chimera

/--
Create a borrow resource.
-/
def makeBorrow (block : BlockId) (isMut : Bool) (lifetime : Lifetime) : Resource :=
  ⟨block, if isMut then .writeBorrow else .readBorrow, lifetime, .unit, .borrowed⟩

/--
Consume an owned resource (move it).
-/
def consumeOwned (r : Resource) : Resource :=
  match r.permission with
  | .own => { r with permission := .own }
  | _ => r

/--
Check if a borrow is valid in the current state.
-/
def validBorrow (s : CallState) (r : Resource) : Bool :=
  match r.permission with
  | .readBorrow =>
    let ownerLive := s.heap.isLive r.block
    let noWriteAlias := !s.resources.any (fun r' =>
      r'.block == r.block && r'.permission == .writeBorrow)
    ownerLive && noWriteAlias
  | .writeBorrow =>
    let ownerLive := s.heap.isLive r.block
    let noOther := !s.resources.any (fun r' =>
      r'.block == r.block && r' != r)
    ownerLive && noOther
  | .own => s.heap.isLive r.block
  | .raw => true

end Chimera
