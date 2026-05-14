-- ChimeraProof Foundation: Bytes
-- Byte operations for the proof system.

namespace Chimera

/--
Byte-level operations.
-/
structure Byte where
  value : Nat
deriving BEq

namespace Byte

/--
Create a byte from a natural number.
-/
def ofNat (n : Nat) : Byte :=
  ⟨n % 256⟩

/--
Get the numeric value.
-/
def toNat (b : Byte) : Nat := b.value

/--
Theorem: Byte.ofNat produces a value in the valid range [0, 256).
-/
theorem ofNat_value_bound (n : Nat) : (Byte.ofNat n).value < 256 := by
  simp [ofNat]
  apply Nat.mod_lt
  simp

/--
Equality.
-/
theorem eq_of_value_eq {a b : Byte} : a.value = b.value → a = b := by
  intro h
  cases a with | mk a_val =>
  cases b with | mk b_val =>
  simp at h
  simp [h]

end Byte

/--
Bytes is a sequence of bytes.
-/
structure Bytes where
  data : List Byte

namespace Bytes

/--
Empty bytes.
-/
def empty : Bytes := ⟨[]⟩

/--
Length of bytes.
-/
def length (b : Bytes) : Nat := b.data.length

/--
Append bytes.
-/
def append (a b : Bytes) : Bytes := ⟨a.data ++ b.data⟩

/--
Get byte at index.
-/
def get? (b : Bytes) (i : Nat) : Option Byte := b.data[i]?

/--
Create from a list of natural numbers.
-/
def fromNatList (xs : List Nat) : Bytes :=
  ⟨xs.map Byte.ofNat⟩

/--
Convert to a list of natural numbers.
-/
def toNatList (b : Bytes) : List Nat :=
  b.data.map Byte.toNat

end Bytes

end Chimera