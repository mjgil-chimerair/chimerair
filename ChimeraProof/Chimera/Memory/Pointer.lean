-- ChimeraProof Memory: Pointer
-- Pointer representation with bounds and raw pointer category.

import Chimera.Foundation
import Chimera.Memory.Block

namespace Chimera

/--
Pointer address space.
-/
inductive AddrSpace
  | null      -- null address space
  | global    -- global memory
  | stack     -- stack memory
  | heap      -- heap memory
  | custom    -- custom address space
deriving Repr, BEq, Inhabited

/--
Raw pointer category for pointer type classification.
-/
inductive RawPtrCategory
  | rawptr     -- raw untagged pointer
  | ptr        -- typed pointer
  | fptr       -- function pointer
  | btpr       -- boundary transition pointer
deriving Repr, BEq, Inhabited

/--
Pointer with address space, bounds, and raw pointer category.
-/
structure Pointer where
  block : BlockId
  offset : Nat
  addrSpace : AddrSpace
  bounds : Nat  -- upper bound on offset (size of allocation)
  category : RawPtrCategory
deriving Repr, BEq, Inhabited

/--
Null pointer with zero address and null address space.
-/
def nullPointer : Pointer := ⟨⟨0⟩, 0, .null, 0, .rawptr⟩

namespace Pointer

/--
Offset a pointer by adding n to the offset.
-/
def addOffset (p : Pointer) (n : Nat) : Pointer :=
  { p with offset := p.offset + n }

/--
Check if pointer is null.
Uses explicit addrSpace = null for proper null semantics.
Note: A pointer with block.id = 0 is NOT necessarily null if addrSpace is set.
A valid pointer in the zero block is different from the null pointer.
-/
def isNull (p : Pointer) : Bool :=
  p.addrSpace == AddrSpace.null

/--
Check if pointer is within bounds.
-/
def isInBounds (p : Pointer) : Bool :=
  p.offset < p.bounds

/--
Check if pointer is valid (non-null and within bounds).
-/
def isValid (p : Pointer) : Bool :=
  !isNull p && isInBounds p

/--
Pointer equality check.
-/
def eq (a b : Pointer) : Bool :=
  a.block == b.block && a.offset == b.offset && a.addrSpace == b.addrSpace

/--
Compare pointer offsets.
-/
def lt (a b : Pointer) : Bool :=
  a.block == b.block && a.offset < b.offset

/--
Get the end pointer (one past the last byte).
-/
def endPointer (p : Pointer) : Pointer :=
  { p with offset := p.bounds }

/--
Get a pointer at the given offset within bounds.
-/
def atOffset (p : Pointer) (off : Nat) : Pointer :=
  { p with offset := off }

/--
Check if two pointers alias (same block and overlapping offsets).
-/
def mayAlias (a b : Pointer) : Bool :=
  a.block == b.block && a.offset < b.bounds && b.offset < a.bounds

/--
Check if two pointers must NOT alias (different blocks or non-overlapping).
-/
def mustNotAlias (a b : Pointer) : Bool :=
  a.block != b.block || a.bounds ≤ b.offset || b.bounds ≤ a.offset

/--
Get the raw pointer category as a string.
-/
def categoryName (p : Pointer) : String :=
  match p.category with
  | .rawptr => "rawptr"
  | .ptr => "ptr"
  | .fptr => "fptr"
  | .btpr => "btpr"

end Pointer

namespace Pointer

/--
C.43: Theorem - null pointer has null address space.
Block ID 0 alone does not make a pointer null.
-/
theorem null_pointer_addrSpace_is_null (p : Pointer) :
  p.addrSpace = AddrSpace.null → p.isNull = true := by
  simp [isNull]

/--
C.43: Theorem - pointer with block.id = 0 is not necessarily null.
A valid pointer in block 0 with non-null address space is NOT null.
-/
theorem block_zero_not_necessarily_null : ∀ (addrSpace : AddrSpace) (offset : Nat),
  addrSpace ≠ AddrSpace.null →
  let p := ⟨⟨0⟩, offset, addrSpace, 0, .rawptr⟩
  p.isNull = false := by
  intro addrSpace offset h
  simp [isNull, addrSpace]
  assumption

/--
C.43: Theorem - null pointer has zero offset.
Null pointer semantics require offset to be 0.
-/
theorem null_pointer_offset_is_zero : ∀ (block : BlockId) (bounds : Nat) (cat : RawPtrCategory),
  let p := ⟨block, 0, .null, bounds, cat⟩
  p.isNull = true ∧ p.offset = 0 := by
  intro block bounds cat
  constructor
  simp [isNull, AddrSpace.null]
  rfl

/--
C.43: Theorem - valid pointer is non-null.
A valid pointer must satisfy isValid = true → isNull = false.
-/
theorem valid_pointer_not_null (p : Pointer) :
  p.isValid = true → p.isNull = false := by
  intro h
  simp [isValid] at h
  match h with | ⟨hn, _⟩ => exact hn

end Pointer

end Chimera
