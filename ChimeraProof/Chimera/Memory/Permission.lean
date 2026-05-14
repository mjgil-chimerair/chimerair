-- ChimeraProof Memory: Permission
-- Memory permissions for borrow checking.

import Chimera.Foundation
import Chimera.ABI.Type
import Chimera.Memory.Block

namespace Chimera

/--
Permission on a memory block.
-/
inductive Permission where
  | own       -- Full ownership, can drop
  | readBorrow -- Read-only borrow
  | writeBorrow -- Mutable borrow
  | raw       -- Raw pointer, no tracking
deriving Repr, BEq, Hashable

/--
Ownership state of a resource.
-/
inductive OwnershipState where
  | valid       -- Resource is valid and accessible
  | borrowed    -- Resource is lent out as borrow
  | moved       -- Ownership has been transferred
  | dropped     -- Resource has been dropped
deriving Repr, BEq

/--
Resource: a block with a permission, lifetime, type, and ownership state.
-/
structure Resource where
  block      : BlockId
  permission : Permission
  lifetime   : Lifetime
  ty         : ChType    -- Type of the resource
  ownership  : OwnershipState  -- Current ownership state
deriving Repr, BEq

namespace Permission

/--
Check if permission allows reading.
-/
def allowsRead : Permission → Bool
  | .own => true
  | .readBorrow => true
  | .writeBorrow => true
  | .raw => false

/--
Check if permission allows writing.
-/
def allowsWrite : Permission → Bool
  | .own => true
  | .writeBorrow => true
  | .raw => false
  | .readBorrow => false

/--
Check if permission is exclusive (mutable).
-/
def isExclusive : Permission → Bool
  | .own => true
  | .writeBorrow => true
  | _ => false

/--
Check if permission transfers ownership.
-/
def transfersOwnership : Permission → Bool
  | .own => true
  | _ => false

end Permission

namespace OwnershipState

/--
Check if resource with this ownership state is accessible.
-/
def isAccessible : OwnershipState → Bool
  | .valid => true
  | .borrowed => true
  | .moved => false
  | .dropped => false

/--
Check if resource can be dropped in this state.
-/
def canDrop : OwnershipState → Bool
  | .valid => true
  | .borrowed => false  -- Cannot drop while borrowed
  | .moved => false
  | .dropped => false

/--
Check if ownership can be transferred from this state.
-/
def canTransfer : OwnershipState → Bool
  | .valid => true
  | .borrowed => false  -- Cannot transfer while borrowed
  | .moved => false
  | .dropped => false

end OwnershipState

namespace Resource

/--
Create a resource with full ownership.
-/
def owned (block : BlockId) (ty : ChType) (lifetime : Lifetime) : Resource :=
  ⟨block, .own, lifetime, ty, .valid⟩

/--
Create a read borrow resource.
-/
def readBorrow (block : BlockId) (ty : ChType) (lifetime : Lifetime) : Resource :=
  ⟨block, .readBorrow, lifetime, ty, .borrowed⟩

/--
Create a write borrow resource.
-/
def writeBorrow (block : BlockId) (ty : ChType) (lifetime : Lifetime) : Resource :=
  ⟨block, .writeBorrow, lifetime, ty, .borrowed⟩

/--
Create a raw pointer resource.
-/
def raw (block : BlockId) (ty : ChType) : Resource :=
  ⟨block, .raw, .call, ty, .valid⟩

/--
Mark resource as moved.
-/
def markMoved (r : Resource) : Resource :=
  { r with permission := .own, ownership := .moved }

/--
Mark resource as dropped.
-/
def markDropped (r : Resource) : Resource :=
  { r with ownership := .dropped }

/--
Check if resource is live (valid or borrowed).
-/
def isLive (r : Resource) : Bool :=
  r.ownership == .valid || r.ownership == .borrowed

/--
Check if resource can be read.
-/
def canRead (r : Resource) : Bool :=
  r.permission.allowsRead && r.ownership.isAccessible

/--
Check if resource can be written.
-/
def canWrite (r : Resource) : Bool :=
  r.permission.allowsWrite && r.ownership.isAccessible

/--
Check if two resources may alias.
-/
def mayAlias (a b : Resource) : Bool :=
  a.block == b.block && a.permission.isExclusive && b.permission.isExclusive

/--
Check if two resources must not alias.
-/
def mustNotAlias (a b : Resource) : Bool :=
  a.block != b.block || !a.permission.isExclusive || !b.permission.isExclusive

/--
Theorem: Dropping a value makes ownership unavailable.

When a resource is marked as dropped, it can no longer be accessed
or transferred. This prevents double-drop.
-/
theorem drop_makes_unavailable (r : Resource) : r.markDropped.ownership = .dropped := by
  simp [Resource.markDropped]

/--
Theorem: Dropped resource is not live.

A dropped resource has ownership = .dropped, which is not live.
-/
theorem dropped_not_live (r : Resource) : r.markDropped.isLive = false := by
  simp [Resource.markDropped, Resource.isLive]

/--
Theorem: Cannot read from dropped resource.

A dropped resource cannot be read because ownership.isAccessible is false.
-/
theorem dropped_cannot_read (r : Resource) : r.markDropped.canRead = false := by
  simp [Resource.markDropped, Resource.canRead, OwnershipState.isAccessible]

/--
Theorem: Cannot write to dropped resource.

A dropped resource cannot be written because ownership.isAccessible is false.
-/
theorem dropped_cannot_write (r : Resource) : r.markDropped.canWrite = false := by
  simp [Resource.markDropped, Resource.canWrite, OwnershipState.isAccessible]

/--
Theorem: Cannot transfer dropped resource.

A dropped resource cannot transfer ownership - canTransfer is false.
-/
theorem dropped_cannot_transfer (r : Resource) : r.markDropped.ownership.canTransfer = false := by
  simp [Resource.markDropped, OwnershipState.canTransfer]

/--
Theorem: Moved resource is not live.

A moved resource has ownership = .moved, which is not live.
-/
theorem moved_not_live (r : Resource) : r.markMoved.isLive = false := by
  simp [Resource.markMoved, Resource.isLive]

/--
Theorem: Moved resource cannot transfer again.

Once moved, a resource cannot transfer ownership again.
-/
theorem moved_cannot_transfer (r : Resource) : r.markMoved.ownership.canTransfer = false := by
  simp [Resource.markMoved, OwnershipState.canTransfer]

end Resource

end Chimera
