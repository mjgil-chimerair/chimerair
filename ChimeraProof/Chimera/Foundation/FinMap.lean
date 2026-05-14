-- ChimeraProof Foundation: FinMap
-- Finite map for the ChimeraIR proof system.
-- Typed version with explicit key type parameter.

namespace Chimera

/--
TypedFinMap is a finite map from key type κ to value type α.
The key type must provide a decidable equality instance.
Callers must ensure key uniqueness via proper projection
from domain types (BlockId, AllocatorId, Symbol).
-/
structure TypedFinMap (κ : Type) [BEq κ] (α : Type) where
  entries : List (κ × α)
deriving Repr

namespace TypedFinMap

/--
Empty TypedFinMap.
-/
def empty (κ : Type) [BEq κ] (α : Type) : TypedFinMap κ α := ⟨[]⟩

/--
Look up a key (internal recursive function).
-/
def findCore (κ : Type) [BEq κ] {α : Type} (entries : List (κ × α)) (k : κ) : Option α :=
  match entries with
  | [] => none
  | (k', v) :: rest =>
    if k' == k then some v else findCore κ rest k

/--
Look up a key.
-/
def find? {κ : Type} [BEq κ] {α : Type} (m : TypedFinMap κ α) (k : κ) : Option α :=
  findCore κ m.entries k

/--
Insert a key-value pair (left-biased, replaces existing key).
-/
def insert {κ : Type} [BEq κ] {α : Type} (m : TypedFinMap κ α) (k : κ) (v : α) : TypedFinMap κ α :=
  ⟨(k, v) :: m.entries.filter (fun (k', _) => !(k' == k))⟩

/--
Remove a key.
-/
def erase {κ : Type} [BEq κ] {α : Type} (m : TypedFinMap κ α) (k : κ) : TypedFinMap κ α :=
  ⟨m.entries.filter (fun (k', _) => !(k' == k))⟩

/--
Check if key exists.
-/
def contains {κ : Type} [BEq κ] {α : Type} (m : TypedFinMap κ α) (k : κ) : Bool :=
  (find? m k).isSome

/--
Get all keys.
-/
def keys {κ : Type} [BEq κ] {α : Type} (m : TypedFinMap κ α) : List κ :=
  m.entries.map Prod.fst

/--
Get all values.
-/
def values {κ : Type} [BEq κ] {α : Type} (m : TypedFinMap κ α) : List α :=
  m.entries.map Prod.snd

/--
Number of entries.
-/
def size {κ : Type} [BEq κ] {α : Type} (m : TypedFinMap κ α) : Nat :=
  m.entries.length

/--
Map over values.
-/
def map {κ : Type} [BEq κ] {α β : Type} (f : α → β) (m : TypedFinMap κ α) : TypedFinMap κ β :=
  ⟨m.entries.map (fun (k, v) => (k, f v))⟩

/--
Union of two maps (left bias).
-/
def union {κ : Type} [BEq κ] {α : Type} (a b : TypedFinMap κ α) : TypedFinMap κ α :=
  ⟨a.entries ++ b.entries⟩

/--
Insert with duplicate check - returns error if key already exists.
This enforces uniqueness policy for TypedFinMap keys.
-/
def insertNoDup {κ : Type} [BEq κ] {α : Type} (m : TypedFinMap κ α) (k : κ) (v : α) : Except String (TypedFinMap κ α) :=
  match find? m k with
  | some _ => .error "Key already exists"
  | none => .ok ⟨(k, v) :: m.entries⟩

/--
Theorem: find? after insert returns the inserted value.
-/
theorem find_after_insert {κ : Type} [BEq κ] [ReflBEq κ] {α : Type} (m : TypedFinMap κ α) (k : κ) (v : α) :
  (m.insert k v).find? k = some v := by
  simp [insert, findCore, find?]

private theorem findCore_filter_eq_none {κ : Type} [BEq κ] {α : Type}
  (entries : List (κ × α)) (k : κ) :
  findCore κ (entries.filter (fun (k', _) => !(k' == k))) k = none := by
  induction entries with
  | nil =>
      simp [findCore]
  | cons entry rest ih =>
      cases entry with
      | mk k' v =>
          by_cases h : k' == k
          · simp [h, ih]
          · have hk : (k' == k) = false := by
                cases hEq : (k' == k) <;> simp [hEq] at h ⊢
            simp [findCore, hk, ih]

/--
Theorem: find? after erase returns none.
-/
theorem find_after_erase {κ : Type} [BEq κ] {α : Type} (m : TypedFinMap κ α) (k : κ) :
  (m.erase k).find? k = none := by
  simpa [erase, find?] using findCore_filter_eq_none m.entries k

/--
Theorem: insert with insertNoDup succeeds when key not present.
-/
theorem insertNoDup_ok {κ : Type} [BEq κ] {α : Type} (m : TypedFinMap κ α) (k : κ) (v : α) :
  ¬ (m.contains k) → (m.insertNoDup k v).isOk = true := by
  intros h
  simp [contains] at h
  cases hFind : find? m k with
  | none =>
      rw [insertNoDup, hFind]
      rfl
  | some value =>
      simp [hFind] at h

end TypedFinMap

/--
Legacy FinMap (Nat-keyed) for backward compatibility.
New code should use TypedFinMap for type-safe key operations.
-/
structure FinMap (α : Type) where
  entries : List (Nat × α)
deriving Repr

namespace FinMap

/--
Empty FinMap.
-/
def empty (α : Type) : FinMap α := ⟨[]⟩

/--
Look up a key (internal recursive function).
-/
def findCore (entries : List (Nat × α)) (k : Nat) : Option α :=
  match entries with
  | [] => none
  | (k', v) :: rest =>
    if k' = k then some v else findCore rest k

/--
Look up a key.
-/
def find? (m : FinMap α) (k : Nat) : Option α :=
  findCore m.entries k

/--
Insert a key-value pair (left-biased, replaces existing key).
-/
def insert (m : FinMap α) (k : Nat) (v : α) : FinMap α :=
  ⟨(k, v) :: m.entries.filter (fun (k', _) => k' ≠ k)⟩

/--
Remove a key.
-/
def erase (m : FinMap α) (k : Nat) : FinMap α :=
  ⟨m.entries.filter (fun (k', _) => k' ≠ k)⟩

/--
Check if key exists.
-/
def contains (m : FinMap α) (k : Nat) : Bool :=
  (find? m k).isSome

/--
Get all keys.
-/
def keys (m : FinMap α) : List Nat :=
  m.entries.map Prod.fst

/--
Get all values.
-/
def values (m : FinMap α) : List α :=
  m.entries.map Prod.snd

/--
Number of entries.
-/
def size (m : FinMap α) : Nat :=
  m.entries.length

/--
Map over values.
-/
def map {α β : Type} (f : α → β) (m : FinMap α) : FinMap β :=
  ⟨m.entries.map (fun (k, v) => (k, f v))⟩

/--
Union of two maps (left bias).
-/
def union (a b : FinMap α) : FinMap α :=
  ⟨a.entries ++ b.entries⟩

/--
Insert with duplicate check - returns error if key already exists.
This enforces uniqueness policy for FinMap keys.
-/
def insertNoDup (m : FinMap α) (k : Nat) (v : α) : Except String (FinMap α) :=
  match find? m k with
  | some _ => .error s!"Key {k} already exists"
  | none => .ok ⟨(k, v) :: m.entries⟩

/--
Theorem: find? after insert returns the inserted value.
-/
theorem find_after_insert {α : Type} (m : FinMap α) (k : Nat) (v : α) :
  (m.insert k v).find? k = some v := by
  simp [insert, findCore, find?]

private theorem findCore_filter_ne_none {α : Type}
  (entries : List (Nat × α)) (k : Nat) :
  findCore (entries.filter (fun (k', _) => !decide (k' = k))) k = none := by
  induction entries with
  | nil =>
      simp [findCore]
  | cons entry rest ih =>
      cases entry with
      | mk k' v =>
          by_cases h : k' = k
          · simp [h, ih]
          · have hk : decide (k' = k) = false := by
                simp [h]
            simp [findCore, h, hk, ih]

/--
Theorem: find? after erase returns none.
-/
theorem find_after_erase {α : Type} (m : FinMap α) (k : Nat) :
  (m.erase k).find? k = none := by
  simpa [erase, find?] using findCore_filter_ne_none m.entries k

end FinMap

end Chimera
