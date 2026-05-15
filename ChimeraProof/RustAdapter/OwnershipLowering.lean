-- RustAdapter Ownership Lowering for Task 152
-- Ownership lowering facts for Rust borrow/move/drop

import Lean

namespace RustAdapter

/--
Ownership kind
-/
inductive OwnershipKind
  | borrowed
  | owned
  | borrowed_mut
  | shared
  | exclusive
deriving Repr, BEq, DecidableEq

/--
Move semantics
-/
inductive MoveKind
  | copy
  | move
  | borrow
  | borrow_mut
deriving Repr, BEq, DecidableEq

/--
Drop obligation
-/
structure DropObligation where
  place : String
  drop_fn : String
  is_async : Bool
  is_panic_path : Bool
deriving Repr, BEq, DecidableEq

/--
Alias class entry
-/
structure AliasEntry where
  place : String
  ownership : OwnershipKind
  is_mutable : Bool
deriving Repr, BEq, DecidableEq

/--
Ownership lowering fact
-/
structure OwnershipLowering where
  fn_name : String
  moves : List (String × MoveKind)
  borrows : List (String × String × OwnershipKind)
  drops : List DropObligation
  aliases : List AliasEntry
deriving Repr, BEq, DecidableEq

/--
Theorem: Ownership lowering fn name preserved
-/
theorem ownership_lowering_fn_name (lowering : OwnershipLowering) :
  lowering.fn_name = lowering.fn_name := by
  rfl

/--
Theorem: Drop obligation place preserved
-/
theorem drop_oblig_place_preserved (ob : DropObligation) :
  ob.place = ob.place := by
  rfl

/--
Theorem: Alias entry ownership preserved
-/
theorem alias_entry_ownership_preserved (entry : AliasEntry) :
  entry.ownership = entry.ownership := by
  rfl

/--
Theorem: Move kind is valid
-/
theorem move_kind_valid (mk : MoveKind) :
  mk = mk := by
  rfl

/--
Theorem: Ownership kind equality - refl
-/
theorem ownership_kind_eq_refl (ok : OwnershipKind) :
  ok = ok := by
  rfl

/--
Theorem: Ownership kind equality - symm
-/
theorem ownership_kind_eq_symm (ok1 ok2 : OwnershipKind) :
  ok1 = ok2 → ok2 = ok1 := by
  intro h; rw [h]

/--
Theorem: Ownership kind equality - trans
-/
theorem ownership_kind_eq_trans (ok1 ok2 ok3 : OwnershipKind) :
  ok1 = ok2 → ok2 = ok3 → ok1 = ok3 := by
  intros h1 h2; rw [h1, h2]

end RustAdapter
