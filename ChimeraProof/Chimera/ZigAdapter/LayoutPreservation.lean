-- ChimeraProof Zig Adapter: Layout Preservation
-- Proof surface for preserving Zig-emitted layout facts through Chimera lowering.

import Chimera.Foundation

namespace Chimera.ZigAdapter

/--
High-level layout shape tracked by the current proof surface.
-/
inductive LayoutShape
  | plainStruct
  | packedStruct
  | externStruct
  | optionalValue
  | sliceValue
  | errorUnion
deriving Repr, BEq, DecidableEq

/--
Layout facts emitted from Zig and preserved by Chimera lowering.
-/
structure LayoutFact where
  shape : LayoutShape
  typeName : String
  size : Nat
  align : Nat
  fieldOffsets : List Nat
deriving Repr, BEq, DecidableEq

/--
Current layout lowering model. The proof surface says Chimera must preserve the
layout facts it is given for size, alignment, and field offsets.
-/
def lowerLayoutFact (fact : LayoutFact) : LayoutFact := fact

theorem lower_preserves_size (fact : LayoutFact) :
  (lowerLayoutFact fact).size = fact.size := by
  rfl

theorem lower_preserves_align (fact : LayoutFact) :
  (lowerLayoutFact fact).align = fact.align := by
  rfl

theorem lower_preserves_field_offsets (fact : LayoutFact) :
  (lowerLayoutFact fact).fieldOffsets = fact.fieldOffsets := by
  rfl

private def sampleStructLayout : LayoutFact := {
  shape := .plainStruct
  typeName := "Point"
  size := 8
  align := 4
  fieldOffsets := [0, 4]
}

private def samplePackedStructLayout : LayoutFact := {
  shape := .packedStruct
  typeName := "PackedPair"
  size := 3
  align := 1
  fieldOffsets := [0, 1]
}

private def sampleExternStructLayout : LayoutFact := {
  shape := .externStruct
  typeName := "ExternPair"
  size := 16
  align := 8
  fieldOffsets := [0, 8]
}

private def sampleOptionalLayout : LayoutFact := {
  shape := .optionalValue
  typeName := "?u64"
  size := 16
  align := 8
  fieldOffsets := [0, 8]
}

private def sampleSliceLayout : LayoutFact := {
  shape := .sliceValue
  typeName := "[]const u8"
  size := 16
  align := 8
  fieldOffsets := [0, 8]
}

private def sampleErrorUnionLayout : LayoutFact := {
  shape := .errorUnion
  typeName := "!Payload"
  size := 24
  align := 8
  fieldOffsets := [0, 8]
}

theorem struct_layout_preserved :
  lowerLayoutFact sampleStructLayout = sampleStructLayout := by
  native_decide

theorem packed_struct_layout_preserved :
  lowerLayoutFact samplePackedStructLayout = samplePackedStructLayout := by
  native_decide

theorem extern_struct_layout_preserved :
  lowerLayoutFact sampleExternStructLayout = sampleExternStructLayout := by
  native_decide

theorem optional_layout_preserved :
  lowerLayoutFact sampleOptionalLayout = sampleOptionalLayout := by
  native_decide

theorem slice_layout_preserved :
  lowerLayoutFact sampleSliceLayout = sampleSliceLayout := by
  native_decide

theorem error_union_layout_preserved :
  lowerLayoutFact sampleErrorUnionLayout = sampleErrorUnionLayout := by
  native_decide

/--
Task 116 summary theorem: the current proof surface preserves size, alignment, and
field offsets for struct, packed/extern, optional, slice, and error-union layouts.
-/
theorem zig_layout_preservation_surface :
  lowerLayoutFact sampleStructLayout = sampleStructLayout ∧
    lowerLayoutFact samplePackedStructLayout = samplePackedStructLayout ∧
    lowerLayoutFact sampleExternStructLayout = sampleExternStructLayout ∧
    lowerLayoutFact sampleOptionalLayout = sampleOptionalLayout ∧
    lowerLayoutFact sampleSliceLayout = sampleSliceLayout ∧
    lowerLayoutFact sampleErrorUnionLayout = sampleErrorUnionLayout := by
  exact And.intro rfl <|
    And.intro rfl <|
      And.intro rfl <|
        And.intro rfl <|
          And.intro rfl rfl

end Chimera.ZigAdapter
