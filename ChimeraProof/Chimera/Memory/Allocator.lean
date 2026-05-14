-- ChimeraProof Memory: Allocator
-- Allocator model and allocation tracking.

import Chimera.Foundation
import Chimera.Memory.Block

namespace Chimera

/--
Allocator identifier.
-/
structure AllocatorId where
  name : Symbol
deriving Repr, BEq, Hashable

/--
Allocation record.
-/
structure AllocationRecord where
  block     : BlockId
  allocator : AllocatorId
  size      : Nat
  align     : Nat
deriving Repr, BEq

/--
Allocation registry.
-/
structure AllocRegistry where
  records : List AllocationRecord
deriving Repr, BEq

namespace AllocRegistry

/--
Empty registry.
-/
def empty : AllocRegistry := ⟨[]⟩

/--
Register an allocation (allows duplicates for first-match semantics).
-/
def register (r : AllocRegistry) (rec : AllocationRecord) : AllocRegistry :=
  ⟨rec :: r.records⟩

/--
Find the allocator for a block.
-/
def findAllocator? (r : AllocRegistry) (block : BlockId) : Option AllocatorId :=
  r.records.find? (fun rec => rec.block == block) |>.map (·.allocator)

/--
Register an allocation, rejecting duplicate block IDs.
Returns error if block ID already registered.
-/
def registerNoDup (r : AllocRegistry) (rec : AllocationRecord) : Except String AllocRegistry :=
  match findAllocator? r rec.block with
  | some _ => .error s!"Block {rec.block.id} already registered"
  | none => .ok ⟨rec :: r.records⟩

/--
Check if two blocks share the same allocator.
-/
def sameAllocator (r : AllocRegistry) (a b : BlockId) : Bool :=
  match findAllocator? r a with
  | some aid_a =>
    match findAllocator? r b with
    | some aid_b => aid_a == aid_b
    | none => false
  | none => false

end AllocRegistry

/--
Drop function registration.
-/
structure DropFn where
  symbol : Symbol
  inputType : ChType
  allocator : Option AllocatorId
deriving Repr, BEq

/--
Drop registry.
-/
structure DropRegistry where
  drops : List DropFn
deriving Repr, BEq

namespace DropRegistry

/--
Empty drop registry.
-/
def empty : DropRegistry := ⟨[]⟩

/--
Register a drop function (allows duplicates for first-match semantics).
-/
def register (r : DropRegistry) (df : DropFn) : DropRegistry :=
  ⟨df :: r.drops⟩

/--
Find drop function for a type.
-/
def findDropFn? (r : DropRegistry) (ty : ChType) : Option DropFn :=
  r.drops.find? (fun df => df.inputType == ty)

/--
Register a drop function, rejecting duplicate types.
Returns error if type already registered.
-/
def registerNoDup (r : DropRegistry) (df : DropFn) : Except String DropRegistry :=
  match findDropFn? r df.inputType with
  | some _ => .error "Drop for type already registered"
  | none => .ok ⟨df :: r.drops⟩

end DropRegistry

/--
Check if a type has a registered drop path.
-/
def HasDropPath (reg : DropRegistry) (ty : ChType) : Prop :=
  match ty with
  | .opaque _ => (DropRegistry.findDropFn? reg ty).isSome
  | .owned inner => HasDropPath reg inner
  | .result ok err => HasDropPath reg ok ∧ HasDropPath reg err
  | _ => False

end Chimera
