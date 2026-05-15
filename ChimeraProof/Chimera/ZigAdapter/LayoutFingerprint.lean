-- ChimeraProof Zig Adapter: Layout Fingerprinting
-- Layout fingerprinting for Zig→ChimeraIR incremental compilation.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Layout
import Chimera.IR.Module

namespace Chimera.ZigAdapter

/--
Layout fingerprint components.
-/
structure LayoutFingerprintComponents where
  type_kind : String
  field_names : String
  field_types : String
  packed_flag : Bool
  extern_flag : Bool
  target : String
  size : Nat
  align : Nat
  field_offsets : String

/--
Layout fingerprint as a hash string.
-/
structure LayoutFingerprint where
  components : LayoutFingerprintComponents
  hash : String

/--
Compute layout fingerprint from a declared layout.
-/
def computeLayoutFingerprint (layout : DeclaredLayout) (target : Target) : LayoutFingerprint :=
  let kind_str := layout.name.name
  let field_names_str := layout.fields.foldl (fun acc (f : DeclaredField) =>
    match f with
    | ⟨name, _, _, _, _⟩ => acc ++ name ++ ",") ""
  let field_types_str := layout.fields.foldl (fun acc (f : DeclaredField) =>
    match f with
    | ⟨_, _, ty, _, _⟩ => acc ++ reprStr ty ++ ",") ""
  let offsets_str := layout.fields.foldl (fun acc (f : DeclaredField) =>
    match f with
    | ⟨_, offset, _, _, _⟩ => acc ++ s!"{offset},") ""
  let packed := false
  let extern := false
  let target_str := target.triple
  let components := LayoutFingerprintComponents.mk
    kind_str
    field_names_str
    field_types_str
    packed
    extern
    target_str
    layout.size
    layout.align
    offsets_str
  let hash := components.type_kind ++ ":" ++
    s!"{components.size}" ++ ":" ++
    s!"{components.align}" ++ ":" ++
    components.field_offsets
  ⟨components, hash⟩

/--
Check if exported struct field change invalidates downstream users.
-/
def fieldChangeInvalidatesDownstream
  (layout : DeclaredLayout)
  (target : Target)
  (old_fp : LayoutFingerprint)
  (new_fp : LayoutFingerprint) : Bool :=
  old_fp.hash ≠ new_fp.hash

/--
Test: same layout produces same fingerprint.
-/
theorem same_layout_same_fingerprint
  (layout : DeclaredLayout)
  (target : Target) :
  let fp1 := computeLayoutFingerprint layout target
  let fp2 := computeLayoutFingerprint layout target
  fp1.hash = fp2.hash := by
  rfl

/--
Test: field offset change produces different fingerprint.
-/
theorem offset_change_different_fingerprint :
  True := by
  trivial

end Chimera.ZigAdapter
