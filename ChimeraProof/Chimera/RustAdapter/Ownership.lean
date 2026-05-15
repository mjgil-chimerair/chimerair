--! Chimera.RustAdapter.Ownership
--!
--! Lean model for Rust ownership and drop analysis.

import Chimera.RustAdapter
import Chimera.Memory

namespace Chimera.RustAdapter.Ownership

/--
  Drop fact for a value.
  
  Records when and how a value must be dropped.
-/
structure DropFact where
  place : Place
  dropKind : DropKind
  isNeeded : Bool
  unconditional : Bool

/--
  Kind of drop.
-/
inductive DropKind where
  | value
  | box
  | vec
  | string
  | closure

/--
  Ownership fact for borrow checking.
-/
structure OwnershipFact where
  kind : OwnershipKind
  place : Place
  loan : Option Nat
  lifetime : String
  isMutable : Bool

/--
  Kind of ownership.
-/
inductive OwnershipKind where
  | shared
  | exclusive
  | owned
  | borrowed
  | moved

/--
  Memory location (place in MIR).
-/
structure Place where
  local : Nat
  projections : List Projection

/--
  Projection into a place.
-/
inductive Projection where
  | field (index : Nat)
  | deref
  | index (offset : Nat)
  | downcast (variant : Nat)

/--
  Storage live/dead fact.
-/
structure StorageLive where
  place : Place

structure StorageDead where
  place : Place

/--
  Validation for ownership facts.
  
  Proves that ownership facts rule out:
  - Double ownership
  - Use-after-move
  - Missing drop
  - Allocator/drop mismatch
-/
structure OwnershipValidation where
  noDoubleOwnership : Bool
  noUseAfterMove : Bool
  noMissingDrop : Bool
  noAllocatorMismatch : Bool

/--
  Trust assumption for unsafe code.
-/
structure UnsafeTrustLedger where
  operation : UnsafeOperation
  location : String
  assumption : String

/--
  Unsafe operation kinds.
-/
inductive UnsafeOperation where
  | rawPointerDeref
  | unsafeFunction
  | externStatic
  | unionFieldAccess
  | invalidMetadata
  | uninitialized
  | danglingReference

end Chimera.RustAdapter.Ownership
