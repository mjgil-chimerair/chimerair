-- ChimeraProof Memory: Drop
-- Drop semantics and ownership consumption.

import Chimera.Foundation
import Chimera.ABI
import Chimera.Memory.Block
import Chimera.Memory.Permission
import Chimera.Memory.Allocator

namespace Chimera

/--
Drop result.
-/
inductive DropResult where
  | success : Heap → DropResult
  | error : ChError → DropResult

namespace DropResult

def isError : DropResult → Bool
  | .success _ => false
  | .error _ => true

end DropResult

/--
Execute a drop, consuming ownership.
On success, the block state becomes .dropped.
On error (block not found, already dropped, already moved), ownership is unchanged.
-/
def executeDrop
  (heap : Heap)
  (block : BlockId)
  (dropFn : Option DropFn)
  (allocReg : AllocRegistry) :
  DropResult :=
  match heap.getBlock? block with
  | none => .error (ChError.allocator s!"block {block.id} not found")
  | some b =>
    match b.state with
    | .dropped => .error (ChError.allocator s!"block {block.id} already dropped")
    | .moved => .error (ChError.ownership s!"block {block.id} already moved")
    | .live =>
      match dropFn with
      | some _df =>
        .success (heap.markDropped block)
      | none =>
        .success (heap.markDropped block)

/--
Theorem: executeDrop on a live block succeeds with .dropped state.
-/
theorem executeDrop_live_succeeds
  (heap : Heap) (block : BlockId) (dropFn : Option DropFn) (allocReg : AllocRegistry)
  (b : Block)
  (hFind : heap.getBlock? block = some b)
  (hLive : b.state = .live) :
  executeDrop heap block dropFn allocReg = .success (heap.markDropped block) := by
  simp [executeDrop, hFind, hLive]

/--
Theorem: executeDrop on already-dropped block fails.
-/
theorem executeDrop_already_dropped
  (heap : Heap) (block : BlockId) (dropFn : Option DropFn) (allocReg : AllocRegistry)
  (b : Block)
  (hFind : heap.getBlock? block = some b)
  (hDropped : b.state = .dropped) :
  executeDrop heap block dropFn allocReg =
    .error (ChError.allocator s!"block {block.id} already dropped") := by
  simp [executeDrop, hFind, hDropped]

/--
Theorem: executeDrop on already-moved block fails.
-/
theorem executeDrop_already_moved
  (heap : Heap) (block : BlockId) (dropFn : Option DropFn) (allocReg : AllocRegistry)
  (b : Block)
  (hFind : heap.getBlock? block = some b)
  (hMoved : b.state = .moved) :
  executeDrop heap block dropFn allocReg =
    .error (ChError.ownership s!"block {block.id} already moved") := by
  simp [executeDrop, hFind, hMoved]

/--
Successful drop consumes the block: it is no longer live and the stored state is `.dropped`.
-/
theorem executeDrop_consumes_ownership
  (heap : Heap) (block : BlockId) (dropFn : Option DropFn) (allocReg : AllocRegistry)
  (heap' : Heap)
  (h : executeDrop heap block dropFn allocReg = .success heap') :
  heap'.findBlock? block = (heap.markDropped block).findBlock? block ∧
    heap'.isLive block = false := by
  cases hGet : heap.getBlock? block with
  | none =>
      simp [executeDrop, hGet] at h
  | some b =>
      cases hState : b.state with
      | dropped =>
          simp [executeDrop, hGet, hState] at h
      | moved =>
          simp [executeDrop, hGet, hState] at h
      | live =>
          cases dropFn <;> simp [executeDrop, hGet, hState] at h
          · cases h
            constructor <;> rfl
          · cases h
            constructor <;> rfl

/--
After a successful drop, a second drop on the same block fails with the canonical double-drop error.
-/
theorem executeDrop_double_drop_impossible
  (heap : Heap) (block : BlockId) (dropFn : Option DropFn) (allocReg : AllocRegistry)
  (heap' : Heap)
  (h : executeDrop heap block dropFn allocReg = .success heap') :
  executeDrop heap' block dropFn allocReg =
    .error (ChError.allocator s!"block {block.id} already dropped") := by
  have hConsumed := executeDrop_consumes_ownership heap block dropFn allocReg heap' h
  have hFind : heap'.getBlock? block = some { (heap.findBlock? block).getD ⟨block, 0, 1, Symbol.simple "", .dropped⟩ with state := .dropped } := by
    cases hMark : heap.markDropped block |>.findBlock? block with
    | none =>
        simp [Heap.findBlock?] at hConsumed
    | some droppedBlock =>
        simpa [Heap.getBlock?, Heap.findBlock?] using hConsumed.1
  cases hGet : heap'.getBlock? block with
  | none =>
      simp [executeDrop, hGet] at hFind
  | some b =>
      have hDropped : b.state = .dropped := by
        simpa [Heap.getBlock?, Heap.findBlock?] using hFind
      exact executeDrop_already_dropped heap' block dropFn allocReg b hGet hDropped

end Chimera
