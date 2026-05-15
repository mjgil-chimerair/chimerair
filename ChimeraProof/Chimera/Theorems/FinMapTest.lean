-- ChimeraProof Tests: FinMap
-- Tests for FinMap operations.

import Chimera.Foundation.FinMap
import Chimera.Foundation.Symbol
import Chimera.Memory.Block
import Chimera.Memory.Allocator

namespace Chimera.Test

namespace FinMapTest

-- Test: empty map has size 0
theorem empty_size : (FinMap.empty String).size = 0 := by rfl

-- Test: insert into empty map gives size 1
theorem empty_insert_size : (FinMap.empty String |>.insert 1 "a").size = 1 := by rfl

-- Test: find? returns none for key not in empty map
theorem find_empty_none : (FinMap.empty String).find? 1 = none := by rfl

-- Test: find? returns some after insert at same key
theorem insert_find_same : (FinMap.empty String |>.insert 1 "a").find? 1 = some "a" := by rfl

-- Test: contains returns false for empty map
theorem contains_empty_false : (FinMap.empty String).contains 1 = false := by rfl

-- Test: contains returns true for existing key after insert
theorem contains_true : (FinMap.empty String |>.insert 1 "a").contains 1 = true := by rfl

-- Test: keys after insert includes the new key
theorem keys_insert : (FinMap.empty String |>.insert 1 "a").keys = [1] := by rfl

-- Test: values after insert includes the new value
theorem values_insert : (FinMap.empty String |>.insert 1 "a").values = ["a"] := by rfl

-- Test: union combines entries
theorem union_keys : ((FinMap.empty String |>.insert 1 "a").union (FinMap.empty String |>.insert 2 "b")).keys = [1, 2] := by rfl

-- Test: map transforms values
theorem map_values : ((FinMap.empty String |>.insert 1 "a").map (fun s => s.length)).values = [1] := by rfl

end FinMapTest

namespace TypedFinMapTest

private def blockOne : BlockId := ⟨1⟩
private def blockTwo : BlockId := ⟨2⟩
private def allocatorOne : AllocatorId := ⟨Symbol.simple "alloc.one"⟩
private def allocatorTwo : AllocatorId := ⟨Symbol.simple "alloc.two"⟩
private def symbolOne : Symbol := Symbol.namespaced "chimera" "entry"
private def symbolTwo : Symbol := Symbol.namespaced "chimera" "helper"

theorem symbol_key_round_trip :
    ((TypedFinMap.empty Symbol String).insert symbolOne "export").find? symbolOne = some "export" := by
  rfl

theorem block_id_key_round_trip :
    ((TypedFinMap.empty BlockId String).insert blockOne "live").find? blockOne = some "live" := by
  rfl

theorem allocator_id_key_round_trip :
    ((TypedFinMap.empty AllocatorId Nat).insert allocatorOne 64).find? allocatorOne = some 64 := by
  rfl

theorem typed_map_replaces_existing_key :
    let m := (TypedFinMap.empty Symbol String).insert symbolOne "first"
    (m.insert symbolOne "second").find? symbolOne = some "second" := by
  rfl

theorem typed_insert_no_dup_rejects_duplicate_symbol :
    ((TypedFinMap.empty Symbol String).insertNoDup symbolOne "first" >>= fun m =>
      m.insertNoDup symbolOne "second").isError = true := by
  native_decide

theorem typed_union_preserves_typed_keys :
    let lhs := (TypedFinMap.empty BlockId String).insert blockOne "a"
    let rhs := (TypedFinMap.empty BlockId String).insert blockTwo "b"
    (lhs.union rhs).keys = [blockOne, blockTwo] := by
  rfl

theorem typed_allocator_map_keeps_distinct_keys :
    let m := (TypedFinMap.empty AllocatorId Nat).insert allocatorOne 32 |>.insert allocatorTwo 64
    m.keys = [allocatorTwo, allocatorOne] := by
  rfl

theorem typed_find_after_erase_for_block_ids :
    let m := (TypedFinMap.empty BlockId String).insert blockOne "tmp"
    (m.erase blockOne).find? blockOne = none := by
  rfl

theorem typed_symbol_map_values_track_insert_order :
    let m := (TypedFinMap.empty Symbol String).insert symbolOne "entry" |>.insert symbolTwo "helper"
    m.values = ["helper", "entry"] := by
  rfl

end TypedFinMapTest

end Chimera.Test
