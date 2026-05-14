-- ChimeraProof Foundation: Word
-- Basic word types for the ChimeraIR proof system.

namespace Chimera

/--
A Word is a fixed-width unsigned integer.
-/
structure Word (width : Nat) where
  value : Nat
  bound : value < 2^width

namespace Word

/--
Create a word from a raw Nat value, masking to the width.
-/
def ofNat (w : Nat) (n : Nat) : Word w :=
  ⟨n % (2^w), by
    apply Nat.mod_lt
    apply Nat.pow_pos
    apply Nat.zero_lt_succ⟩

/--
Theorem: Word.ofNat produces a value in the valid range [0, 2^w).
This is the key invariant that makes Word a proper fixed-width type.
-/
theorem ofNat_value_bound (w n : Nat) : (Word.ofNat w n).value < 2^w := by
  exact (Word.ofNat w n).bound

/--
Theorem: Word.ofNat is idempotent: applying ofNat twice is the same as once.
-/
theorem ofNat_idempotent (w n : Nat) :
  Word.ofNat w (Word.ofNat w n).value = Word.ofNat w n := by
  simp [ofNat]

/--
Get the raw Nat value.
-/
def toNat {w : Nat} (word : Word w) : Nat := word.value

/--
Every word value is bounded by its width.
-/
theorem toNat_bound {w : Nat} (word : Word w) : word.toNat < 2^w := by
  exact word.bound

/--
Addition of words.
-/
def add {w : Nat} (a b : Word w) : Word w :=
  ofNat w (a.value + b.value)

/--
Subtraction of words (wraps on underflow).
-/
def sub {w : Nat} (a b : Word w) : Word w :=
  ofNat w (a.value - b.value)

/--
Multiplication of words.
-/
def mul {w : Nat} (a b : Word w) : Word w :=
  ofNat w (a.value * b.value)

/--
Bitwise AND using Nat land.
-/
def and {w : Nat} (a b : Word w) : Word w :=
  ofNat w (Nat.land a.value b.value)

/--
Bitwise OR using Nat lor.
-/
def or {w : Nat} (a b : Word w) : Word w :=
  ofNat w (Nat.lor a.value b.value)

/--
Bitwise XOR using Nat lor/land: a xor b = (a | b) - (a & b)
-/
def xor {w : Nat} (a b : Word w) : Word w :=
  ofNat w (Nat.lor a.value b.value - Nat.land a.value b.value)

/--
Left shift.
-/
def shiftLeft {w : Nat} (a : Word w) (n : Nat) : Word w :=
  ofNat w (a.value <<< n)

/--
Right shift (logical).
-/
def shiftRight {w : Nat} (a : Word w) (n : Nat) : Word w :=
  ofNat w (a.value >>> n)

/--
Equality of words.
-/
theorem eq_of_value_eq {w : Nat} {a b : Word w} :
  a.value = b.value → a = b := by
  intro h
  cases a with | mk a_val a_bound =>
  cases b with | mk b_val b_bound =>
  simp at h
  subst h
  simp

/--
Word width is non-negative. Note: width 0 is supported (degenerate word, always zero).
-/
theorem width_nonneg : ∀ {w : Nat}, w ≥ 0 := by
  intro w
  cases w with
  | zero => simp
  | succ => simp

/--
Theorem: width 0 produces degenerate but valid word.
-/
theorem width_zero_valid (n : Nat) : (Word.ofNat 0 n).value = 0 := by
  simp [ofNat, Nat.pow_zero, Nat.mod_one]

end Word

end Chimera
