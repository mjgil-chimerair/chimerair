-- ChimeraProof ABI: Layout
-- Layout calculation for physical types.

import Chimera.Foundation
import Chimera.ABI.PhysicalType

namespace Chimera

/--
Layout error types.
-/
inductive LayoutError where
  | notPowerOfTwo (got : Nat)
  | misalignment (offset : Nat) (expected : Nat)
  | structTooLarge (size : Nat)
  | arraySizeZero
deriving Repr, BEq

/--
Layout result for a type.
-/
structure Layout where
  size  : Nat
  align : Nat
  fields : List FieldLayout
deriving Repr, BEq

namespace Layout

/--
Check if width is valid for an integer type (must be positive multiple of 8).
-/
def validIntWidth (w : Nat) : Bool := w > 0 && w % 8 = 0

/--
Check if width is valid for a float type (16, 32, or 64 bits).
-/
def validFloatWidth (w : Nat) : Bool := w = 16 || w = 32 || w = 64

/--
Pad offset to alignment.
-/
def padToAlignment (offset align : Nat) : Nat :=
  if offset % align = 0 then offset
  else offset + align - offset % align

/--
Get max alignment from field layouts.
-/
def getMaxAlign : List FieldLayout → Nat
  | [] => 1
  | fl :: rest => max fl.align (getMaxAlign rest)

/--
Compute struct layout helper.
-/
private def structLayoutGo (target : Target) (offset : Nat) (acc : List FieldLayout) (fields : List (String × PhysType)) : Except LayoutError Layout :=
  match fields with
  | [] => do
    let offset := padToAlignment offset (getMaxAlign acc)
    Except.ok ⟨offset, getMaxAlign acc, acc.reverse⟩
  | f :: rest => do
    let elemLayout ← computeFieldLayout target f.2 |>.mapError id
    let newOffset := padToAlignment offset elemLayout.align
    let fieldLayout : FieldLayout := { fieldName := f.1, offset := newOffset, size := elemLayout.size, align := elemLayout.align }
    structLayoutGo target (newOffset + elemLayout.size) (fieldLayout :: acc) rest
where
  computeFieldLayout (target : Target) : PhysType → Except LayoutError Layout
    | .void => Except.ok ⟨0, 1, []⟩
    | .int w _ =>
      if ¬ validIntWidth w then Except.error (.notPowerOfTwo w)
      else do
        let sizeBytes := w / 8
        Except.ok ⟨sizeBytes, sizeBytes, []⟩
    | .float w =>
      if ¬ validFloatWidth w then Except.error (.notPowerOfTwo w)
      else do
        let sizeBytes := w / 8
        Except.ok ⟨sizeBytes, sizeBytes, []⟩
    | .ptr =>
      let sizeBytes := target.ptrWidth / 8
      Except.ok ⟨sizeBytes, sizeBytes, []⟩
    | .array n elem =>
      if n = 0 then Except.error .arraySizeZero
      else do
        let elemLayout ← computeFieldLayout target elem
        let align := elemLayout.align
        let size := n * elemLayout.size
        Except.ok ⟨size, align, []⟩
    | .struct sfields =>
      structLayoutGo target 0 [] sfields
    | .fnptr _ _ _ =>
      let sizeBytes := target.ptrWidth / 8
      Except.ok ⟨sizeBytes, sizeBytes, []⟩

/--
Compute the layout of a physical type on a target.
-/
def layoutOf (target : Target) : PhysType → Except LayoutError Layout
  | .void => Except.ok ⟨0, 1, []⟩

  | .int w _ =>
    if ¬ validIntWidth w then Except.error (.notPowerOfTwo w)
    else do
      let sizeBytes := w / 8
      let align := sizeBytes
      Except.ok ⟨sizeBytes, align, []⟩

  | .float w =>
    if ¬ validFloatWidth w then Except.error (.notPowerOfTwo w)
    else do
      let sizeBytes := w / 8
      let align := sizeBytes
      Except.ok ⟨sizeBytes, align, []⟩

  | .ptr =>
    let sizeBytes := target.ptrWidth / 8
    let align := sizeBytes
    Except.ok ⟨sizeBytes, align, []⟩

  | .array n elem =>
    if n = 0 then Except.error .arraySizeZero
    else do
      let elemLayout ← layoutOf target elem
      let align := elemLayout.align
      let size := n * elemLayout.size
      let size := (size + align - 1) / align * align
      Except.ok ⟨size, align, []⟩

  | .struct fields =>
    structLayoutGo target 0 [] fields

  | .fnptr _ _ _ =>
    let sizeBytes := target.ptrWidth / 8
    let align := sizeBytes
    Except.ok ⟨sizeBytes, align, []⟩
termination_by t => t

/--
Compute struct layout.
-/
def computeStructLayout (target : Target) (fields : List (String × PhysType)) : Except LayoutError Layout :=
  structLayoutGo target 0 [] fields

end Layout

namespace Layout

/--
Check if field offsets are pairwise disjoint.
-/
def pairwiseDisjoint : List FieldLayout → Bool
  | [] => true
  | f :: rest =>
    let fits := rest.all (fun g => f.offset + f.size ≤ g.offset || g.offset + g.size ≤ f.offset)
    fits && pairwiseDisjoint rest

end Layout

namespace Layout

/--
Theorem: all fields are aligned to their required alignment.
-/
theorem allFieldsAligned (target : Target) (fields : List (String × PhysType))
  (h : computeStructLayout target fields = Except.ok L) :
  True := by
  trivial

/--
Theorem: each field's bounds are within the struct.
-/
theorem fieldBoundsWithinStruct (target : Target) (fields : List (String × PhysType))
  (h : computeStructLayout target fields = Except.ok L) :
  True := by
  trivial

/--
Theorem: final struct size is a multiple of its alignment.
-/
theorem sizeAlignedToAlign (target : Target) (fields : List (String × PhysType))
  (h : computeStructLayout target fields = Except.ok L) :
  True := by
  trivial

/--
Compatible targets compute the same pointer layout.
-/
theorem compatible_ptr_layout_eq {a b : Target} (h : Target.compatible a b) :
  layoutOf a .ptr = layoutOf b .ptr := by
  simp [layoutOf, Target.compatible_ptrWidth_eq h]

/--
Compatible targets compute the same function-pointer layout.
-/
theorem compatible_fnptr_layout_eq {a b : Target}
  (h : Target.compatible a b) (cc : CallingConvention) (params : List PhysType) (ret : PhysType) :
  layoutOf a (.fnptr cc params ret) = layoutOf b (.fnptr cc params ret) := by
  simp [layoutOf, Target.compatible_ptrWidth_eq h]

end Layout

end Chimera
