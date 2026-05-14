-- ChimeraProof Memory: Block and Heap
-- Memory block and heap representation.

import Chimera.Foundation
import Chimera.ABI.Type
import Chimera.Foundation.FinMap

namespace Chimera

/--
Block ID uniquely identifies a memory block.
-/
structure BlockId where
  id : Nat
deriving Repr, BEq, Hashable, Inhabited

/--
Block state in the heap.
-/
inductive BlockState where
  | live      -- Block is allocated and accessible
  | moved     -- Block ownership has been moved out
  | dropped   -- Block has been dropped/freed
deriving Repr, BEq

/--
Memory block with typed content.
-/
structure Block where
  id        : BlockId
  size      : Nat
  align     : Nat
  allocator : Symbol  -- allocator that created this block
  state     : BlockState
deriving Repr, BEq

/--
Abstract heap using typed FinMap for blocks.
Using typed keys prevents accidental Nat-based mixing between domains.
-/
structure Heap where
  blocks : TypedFinMap BlockId Block
deriving Repr

namespace Heap

/--
Empty heap.
-/
def empty : Heap := ⟨TypedFinMap.empty BlockId Block⟩

/--
Find a block by ID.
-/
def findBlock? (h : Heap) (id : BlockId) : Option Block :=
  h.blocks.find? id

/--
Insert or update a block.
-/
def insert (h : Heap) (id : BlockId) (block : Block) : Heap :=
  ⟨h.blocks.insert id block⟩

/--
Allocate a new block.
-/
def alloc (h : Heap) (id : BlockId) (size align : Nat) (alloc : Symbol) : Heap :=
  let block := ⟨id, size, align, alloc, .live⟩
  h.insert id block

/--
Set block state to moved.
-/
def markMoved (h : Heap) (id : BlockId) : Heap :=
  match h.findBlock? id with
  | some b => h.insert id { b with state := .moved }
  | none => h

/--
Set block state to dropped.
-/
def markDropped (h : Heap) (id : BlockId) : Heap :=
  match h.findBlock? id with
  | some b => h.insert id { b with state := .dropped }
  | none => h

/--
Check if block is live.
-/
def isLive (h : Heap) (id : BlockId) : Bool :=
  match h.findBlock? id with
  | some b => b.state == .live
  | none => false

/--
Check if block exists (in any state).
-/
def contains? (h : Heap) (id : BlockId) : Bool :=
  h.blocks.contains id

/--
Get block if it exists.
-/
def getBlock? (h : Heap) (id : BlockId) : Option Block :=
  h.findBlock? id

/--
Delete a block from the heap.
-/
def delete (h : Heap) (id : BlockId) : Heap :=
  ⟨h.blocks.erase id⟩

/--
Well-formed heap predicate.

A heap is well-formed if:
1. All block IDs are unique (no duplicate keys)
2. All alignments are valid (power of two, > 0)
3. Live blocks have non-zero size
4. Allocator consistency (allocator is recorded)
5. Stored block state is coherent with the heap key
-/
structure WellFormedHeap (h : Heap) : Prop where
  /-- All block IDs are unique in the map. -/
  unique_ids : ∀ (a b : BlockId) (block_a block_b : Block),
    h.blocks.find? a = some block_a →
    h.blocks.find? b = some block_b →
    a = b → block_a = block_b
  /-- All alignments are valid (power of two and non-zero). -/
  valid_alignments : ∀ (id : BlockId) (b : Block),
    h.findBlock? id = some b →
    isValidAlignment b.align = true
  /-- Live blocks have non-zero size. -/
  live_nonzero_size : ∀ (id : BlockId) (b : Block),
    h.findBlock? id = some b →
    b.state = .live → b.size > 0
  /-- Allocator is recorded for all blocks. -/
  has_allocator : ∀ (id : BlockId) (b : Block),
    h.findBlock? id = some b →
    b.allocator.name ≠ ""
  /-- Stored block ID matches the key used in the heap. -/
  state_coherent : ∀ (id : BlockId) (b : Block),
    h.findBlock? id = some b →
    b.id = id

/--
Verify that a heap satisfies well-formedness conditions.
Uses isValidAlignment for proper power-of-two check.
-/
def isWellFormed (h : Heap) : Bool :=
  h.blocks.entries.all (fun (id, b) =>
    (id == b.id) &&
    isValidAlignment b.align &&
    (b.state != .live || b.size > 0) &&
    !b.allocator.name.isEmpty
  )

end Heap

end Chimera
