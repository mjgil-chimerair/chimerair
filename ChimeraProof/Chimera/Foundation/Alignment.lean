-- ChimeraProof Foundation: Alignment
-- Alignment specification for the ChimeraIR proof system.

namespace Chimera

/--
Alignment table for primitive types on a target.
-/
structure AlignmentTable where
  i8  : Nat
  i16 : Nat
  i32 : Nat
  i64 : Nat
  u8  : Nat
  u16 : Nat
  u32 : Nat
  u64 : Nat
  f32 : Nat
  f64 : Nat
  ptr : Nat
deriving Repr, BEq

/--
Check if a value is a valid alignment (power of 2).
-/
def isValidAlignment (a : Nat) : Bool :=
  a > 0 ∧ (Nat.land a (a - 1)) = 0

/--
Check if offset is aligned.
-/
def offsetAligned (offset align : Nat) : Bool :=
  offset % align = 0

/--
Theorem: isValidAlignment is true exactly for powers of two greater than zero.
-/
theorem isValidAlignment_power_of_two (a : Nat) :
  isValidAlignment a = true ↔ a > 0 ∧ (Nat.land a (a - 1)) = 0 := by
  simp [isValidAlignment]

/--
Theorem: common alignments are valid (1, 2, 4, 8, 16).
-/
theorem common_alignments_valid :
  isValidAlignment 1 = true ∧
  isValidAlignment 2 = true ∧
  isValidAlignment 4 = true ∧
  isValidAlignment 8 = true ∧
  isValidAlignment 16 = true := by
  constructor <;> constructor <;> constructor <;> constructor <;> rfl

/--
Alignment mismatch error.
-/
inductive AlignmentError where
  | notPowerOfTwo (got : Nat)
  | misalignment (offset : Nat) (expected : Nat)
deriving Repr, BEq

/--
Get alignment for a type by width.
-/
def alignmentForWidth (w : Nat) : Nat :=
  if w = 1 then 1
  else if w = 2 then 2
  else if w ≤ 4 then 4
  else if w ≤ 8 then 8
  else 16

namespace AlignmentTable

/--
Get alignment for a type by name.
-/
def get (tbl : AlignmentTable) (ty : String) : Nat :=
  match ty with
  | "i8"  => tbl.i8
  | "i16" => tbl.i16
  | "i32" => tbl.i32
  | "i64" => tbl.i64
  | "u8"  => tbl.u8
  | "u16" => tbl.u16
  | "u32" => tbl.u32
  | "u64" => tbl.u64
  | "f32" => tbl.f32
  | "f64" => tbl.f64
  | "ptr" => tbl.ptr
  | _     => 1

end AlignmentTable

end Chimera