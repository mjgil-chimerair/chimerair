-- ChimeraProof ABI: Canonical Structs
-- Canonical ABI struct definitions for ch_status, ch_error, ch_allocator, etc.

import Chimera.Foundation
import Chimera.ABI.PhysicalType
import Chimera.ABI.Layout

namespace Chimera

/--
Canonical ch_status struct layout.
Signed 32-bit integer where 0 = success, non-zero = error.
-/
def ch_status_phys : PhysType := .int 32 Signedness.signed

/--
Canonical ch_error struct with all fields.
-/
def ch_error_phys_fields : List (String × PhysType) :=
  let ptrVoid := PhysType.ptr
  let int32u := PhysType.int 32 Signedness.unsigned
  let int64u := PhysType.int 64 Signedness.unsigned
  let dropFn := PhysType.fnptr .cdecl [PhysType.ptr] .void
  [
    ("domain", int32u),
    ("code", int32u),
    ("flags", int32u),
    ("message_ptr", ptrVoid),
    ("message_len", int64u),
    ("payload_ptr", ptrVoid),
    ("payload_drop_fn", dropFn),
    ("payload_drop_ctx", ptrVoid)
  ]

def ch_error_phys : PhysType := .struct ch_error_phys_fields

/--
Canonical ch_allocator struct.
-/
def ch_allocator_phys_fields : List (String × PhysType) :=
  let ptrVoid := PhysType.ptr
  [
    ("id", PhysType.int 64 Signedness.unsigned),
    ("kind", PhysType.int 32 Signedness.unsigned),
    ("ptr", ptrVoid)
  ]

def ch_allocator_phys : PhysType := .struct ch_allocator_phys_fields

/--
Canonical ch_slice struct (ptr + len).
-/
def ch_slice_phys_fields (elemTy : PhysType) : List (String × PhysType) := [
  ("ptr", PhysType.ptr),
  ("len", PhysType.int 64 Signedness.unsigned)
]

def ch_slice_phys (elemTy : PhysType) : PhysType := .struct (ch_slice_phys_fields elemTy)

/--
Canonical ch_borrow_str struct.
-/
def ch_borrow_str_phys_fields : List (String × PhysType) := [
  ("ptr", PhysType.ptr),
  ("len", PhysType.int 64 Signedness.unsigned),
  ("lifetime", PhysType.int 32 Signedness.unsigned)
]

def ch_borrow_str_phys : PhysType := .struct ch_borrow_str_phys_fields

/--
Canonical ch_owned_bytes struct.
-/
def ch_owned_bytes_phys_fields : List (String × PhysType) := [
  ("ptr", PhysType.ptr),
  ("len", PhysType.int 64 Signedness.unsigned),
  ("capacity", PhysType.int 64 Signedness.unsigned),
  ("allocator_id", PhysType.int 64 Signedness.unsigned)
]

def ch_owned_bytes_phys : PhysType := .struct ch_owned_bytes_phys_fields

/--
Canonical ch_handle struct for owned opaque types.
-/
def ch_handle_phys_fields : List (String × PhysType) := [
  ("ptr", PhysType.ptr),
  ("drop_fn", PhysType.fnptr .cdecl [PhysType.ptr] .void),
  ("size", PhysType.int 64 Signedness.unsigned)
]

def ch_handle_phys : PhysType := .struct ch_handle_phys_fields

/--
Compute layout for ch_status on a target.
-/
def ch_status_layout (target : Target) : Except LayoutError Layout :=
  Layout.layoutOf target ch_status_phys

/--
Compute layout for ch_error on a target.
-/
def ch_error_layout (target : Target) : Except LayoutError Layout :=
  Layout.layoutOf target ch_error_phys

/--
Compute layout for ch_allocator on a target.
-/
def ch_allocator_layout (target : Target) : Except LayoutError Layout :=
  Layout.layoutOf target ch_allocator_phys

/--
Compute layout for ch_handle on a target.
-/
def ch_handle_layout (target : Target) : Except LayoutError Layout :=
  Layout.layoutOf target ch_handle_phys

/--
Compatible targets compute the same canonical `ch_status` layout.
-/
theorem compatible_ch_status_layout_eq {a b : Target} (h : Target.compatible a b) :
  ch_status_layout a = ch_status_layout b := by
  simp [ch_status_layout]

/--
Compatible targets compute the same canonical `ch_error` layout.
-/
theorem compatible_ch_error_layout_eq {a b : Target} (h : Target.compatible a b) :
  ch_error_layout a = ch_error_layout b := by
  simp [ch_error_layout, Layout.layoutOf, Target.compatible_ptrWidth_eq h]

/--
Compatible targets compute the same canonical `ch_allocator` layout.
-/
theorem compatible_ch_allocator_layout_eq {a b : Target} (h : Target.compatible a b) :
  ch_allocator_layout a = ch_allocator_layout b := by
  simp [ch_allocator_layout, Layout.layoutOf, Target.compatible_ptrWidth_eq h]

/--
Compatible targets compute the same canonical `ch_handle` layout.
-/
theorem compatible_ch_handle_layout_eq {a b : Target} (h : Target.compatible a b) :
  ch_handle_layout a = ch_handle_layout b := by
  simp [ch_handle_layout, Layout.layoutOf, Target.compatible_ptrWidth_eq h]

end Chimera
