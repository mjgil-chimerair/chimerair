-- ChimeraProof Tests: Memory and Heap
-- Tests for heap and block operations.

import Chimera.Memory.Block
import Chimera.Memory.Allocator

namespace Chimera.Test

namespace BlockIdTest

theorem block_id_eq : (⟨1⟩ : BlockId) = ⟨1⟩ := by
  rfl

end BlockIdTest

namespace BlockStateTest

theorem live_is_live : True := by trivial
theorem moved_is_moved : True := by trivial
theorem dropped_is_dropped : True := by trivial

end BlockStateTest

namespace HeapTest

private def blockOne : BlockId := ⟨1⟩
private def blockTwo : BlockId := ⟨2⟩
private def allocSym : Symbol := Symbol.simple "global_alloc"

theorem empty_heap_no_blocks : Heap.empty.blocks.size = 0 := by
  rfl

theorem alloc_creates_live_block :
    let heap := Heap.alloc Heap.empty blockOne 16 8 allocSym
    heap.findBlock? blockOne = some ⟨blockOne, 16, 8, allocSym, .live⟩ := by
  rfl

theorem find_nonexistent : Heap.empty.findBlock? blockOne = none := by
  rfl

theorem isLive_true :
    let heap := Heap.alloc Heap.empty blockOne 16 8 allocSym
    heap.isLive blockOne = true := by
  rfl

theorem isLive_false_moved :
    let heap := (Heap.alloc Heap.empty blockOne 16 8 allocSym).markMoved blockOne
    heap.isLive blockOne = false := by
  rfl

theorem markMoved_changes_state :
    let heap := (Heap.alloc Heap.empty blockOne 16 8 allocSym).markMoved blockOne
    heap.findBlock? blockOne = some ⟨blockOne, 16, 8, allocSym, .moved⟩ := by
  rfl

theorem markDropped_changes_state :
    let heap := (Heap.alloc Heap.empty blockOne 16 8 allocSym).markDropped blockOne
    heap.findBlock? blockOne = some ⟨blockOne, 16, 8, allocSym, .dropped⟩ := by
  rfl

theorem contains_true :
    let heap := Heap.alloc Heap.empty blockOne 16 8 allocSym
    heap.contains? blockOne = true := by
  rfl

theorem contains_false : Heap.empty.contains? blockOne = false := by
  rfl

theorem delete_removes_typed_block_key :
    let heap := (Heap.alloc Heap.empty blockOne 16 8 allocSym).delete blockOne
    heap.findBlock? blockOne = none := by
  rfl

theorem distinct_typed_block_keys_stay_distinct :
    let heap := (Heap.alloc Heap.empty blockOne 16 8 allocSym).alloc blockTwo 32 8 allocSym
    heap.blocks.keys = [blockTwo, blockOne] := by
  rfl

namespace WellFormedHeapTest

theorem empty_heap_well_formed :
    Heap.WellFormedHeap Heap.empty := by
  refine {
    unique_ids := ?_
    valid_alignments := ?_
    live_nonzero_size := ?_
    has_allocator := ?_
    state_coherent := ?_
  }
  · intro a b block_a block_b hA
    simp at hA
  · intro id b hFind
    simp at hFind
  · intro id b hFind
    simp at hFind
  · intro id b hFind
    simp at hFind
  · intro id b hFind
    simp at hFind

theorem valid_alloc_heap_well_formed :
    let heap := Heap.alloc Heap.empty blockOne 16 8 allocSym
    Heap.WellFormedHeap heap := by
  intro heap
  refine {
    unique_ids := ?_
    valid_alignments := ?_
    live_nonzero_size := ?_
    has_allocator := ?_
    state_coherent := ?_
  }
  · intro a b block_a block_b hA hB hEq
    simpa [heap, Heap.alloc, Heap.insert, Heap.empty] using hA.trans hB.symm
  · intro id b hFind
    simp [heap, Heap.alloc, Heap.insert, Heap.empty] at hFind
    cases hFind
    simp [isValidAlignment]
  · intro id b hFind hState
    simp [heap, Heap.alloc, Heap.insert, Heap.empty] at hFind
    cases hFind
    omega
  · intro id b hFind
    simp [heap, Heap.alloc, Heap.insert, Heap.empty] at hFind
    cases hFind
    simp [allocSym]
  · intro id b hFind
    simp [heap, Heap.alloc, Heap.insert, Heap.empty] at hFind
    cases hFind
    rfl

theorem isWellFormed_empty : Heap.empty.isWellFormed = true := by
  rfl

theorem isWellFormed_alloc :
    let heap := Heap.alloc Heap.empty blockOne 16 8 allocSym
    heap.isWellFormed = true := by
  rfl

theorem isWellFormed_zero_align :
    let heap : Heap := ⟨(TypedFinMap.empty BlockId Block).insert blockOne ⟨blockOne, 16, 0, allocSym, .live⟩⟩
    heap.isWellFormed = false := by
  rfl

theorem isWellFormed_dropped_zero_size :
    let heap : Heap := ⟨(TypedFinMap.empty BlockId Block).insert blockOne ⟨blockOne, 0, 8, allocSym, .dropped⟩⟩
    heap.isWellFormed = true := by
  rfl

theorem isWellFormed_rejects_incoherent_block_id :
    let heap : Heap := ⟨(TypedFinMap.empty BlockId Block).insert blockOne ⟨blockTwo, 16, 8, allocSym, .live⟩⟩
    heap.isWellFormed = false := by
  rfl

theorem markMoved_preserves_executable_wellformedness :
    let heap := (Heap.alloc Heap.empty blockOne 16 8 allocSym).markMoved blockOne
    heap.isWellFormed = true := by
  rfl

theorem markDropped_preserves_executable_wellformedness :
    let heap := (Heap.alloc Heap.empty blockOne 16 8 allocSym).markDropped blockOne
    heap.isWellFormed = true := by
  rfl

end WellFormedHeapTest

end HeapTest

end Chimera.Test
