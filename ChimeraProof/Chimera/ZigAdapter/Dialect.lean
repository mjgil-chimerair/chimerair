-- ChimeraProof Zig Adapter: Dialect
-- Lean/IR model for Zig dialect operations and types.

import Chimera.Foundation
import Chimera.ABI

namespace Chimera.ZigAdapter

/--
Zig dialect operation kinds.
-/
inductive ZigOpKind
  | fn_def
  | slice_index
  | ptr_deref
  | optional_wrap
  | optional_unwrap
  | error_union_wrap
  | error_union_catch
  | struct_field_access
  | union_field_access
  | enum_tag
  | comptime_eval
  | defer_exec
  | errdefer_exec
deriving Repr, BEq

/--
Zig dialect type kinds.
-/
inductive ZigTypeKind
  | zig_function
  | zig_slice
  | zig_pointer
  | zig_optional
  | zig_error_set
  | zig_error_union
  | zig_struct
  | zig_union
  | zig_enum
  | zig_comptime
  | zig_void
  | zig_noreturn
deriving Repr, BEq

/--
Zig dialect operation.
-/
structure ZigOp where
  kind : ZigOpKind
  operands : List String
  result_type : Option String

/--
Zig dialect type.
-/
structure ZigType where
  kind : ZigTypeKind
  name : String
  fields : List (String × String)

namespace ZigDialect

/--
Constant slice type ([]const T).
-/
def constSlice (elem : String) : ZigType := {
  kind := .zig_slice,
  name := "[]const " ++ elem,
  fields := [("ptr", "*const " ++ elem), ("len", "usize")]
}

/--
Mutable slice type ([]T).
-/
def mutSlice (elem : String) : ZigType := {
  kind := .zig_slice,
  name := "[]" ++ elem,
  fields := [("ptr", "*" ++ elem), ("len", "usize")]
}

/--
Optional pointer type (?*T).
-/
def optionalPtr (elem : String) : ZigType := {
  kind := .zig_optional,
  name := "?" ++ elem,
  fields := []
}

/--
Error union type (!T).
-/
def errorUnion (ok : String) : ZigType := {
  kind := .zig_error_union,
  name := "!" ++ ok,
  fields := [("is_error", "bool"), ("value", ok)]
}

/--
Zig function type.
-/
def fnType (params : List String) (ret : String) : ZigType := {
  kind := .zig_function,
  name := "fn(" ++ String.intercalate ", " params ++ ") " ++ ret,
  fields := []
}

end ZigDialect

/--
Slice lowering result.
-/
structure SliceLowering where
  chimera_type : String
  ptr_type : String
  len_type : String
  lifetime : String

/--
Lower Zig slice to Chimera ABI: []const T -> borrowed ptr+len with lifetime metadata.
-/
def lowerConstSlice (zig_ty : ZigType) (lifetime : String) : SliceLowering :=
  match zig_ty.kind with
  | .zig_slice => {
      chimera_type := "ch_slice",
      ptr_type := "borrow ptr",
      len_type := "usize",
      lifetime := lifetime
    }
  | _ => {
      chimera_type := "invalid",
      ptr_type := "invalid",
      len_type := "invalid",
      lifetime := ""
    }

/--
Lower Zig mutable slice to Chimera ABI: []T -> mutable borrowed ptr+len with lifetime.
-/
def lowerMutSlice (zig_ty : ZigType) (lifetime : String) : SliceLowering :=
  match zig_ty.kind with
  | .zig_slice => {
      chimera_type := "ch_slice_mut",
      ptr_type := "borrowMut ptr",
      len_type := "usize",
      lifetime := lifetime
    }
  | _ => {
      chimera_type := "invalid",
      ptr_type := "invalid",
      len_type := "invalid",
      lifetime := ""
    }

/--
Nested slice lowering: []const []const T -> nested borrowed slices.
-/
def lowerNestedSlice (outer : ZigType) (inner : ZigType) (lifetime : String) : String :=
  match outer.kind, inner.kind with
  | .zig_slice, .zig_slice => "nested_ch_slice with lifetime " ++ lifetime
  | _, _ => "invalid"

-- Tests

/--
Test: const slice has ptr and len fields.
-/
theorem const_slice_has_fields :
  let s := ZigDialect.constSlice "u8"
  s.fields.length = 2 := by rfl

/--
Test: mut slice has ptr and len fields.
-/
theorem mut_slice_has_fields :
  let s := ZigDialect.mutSlice "i32"
  s.fields.length = 2 := by rfl

/--
Test: const slice lowered correctly.
-/
theorem const_slice_lowered :
  let s := ZigDialect.constSlice "u8"
  let lowering := lowerConstSlice s "call"
  lowering.chimera_type = "ch_slice" := by rfl

/--
Test: mut slice lowered to mutable.
-/
theorem mut_slice_lowered_mut :
  let s := ZigDialect.mutSlice "i32"
  let lowering := lowerMutSlice s "call"
  lowering.ptr_type = "borrowMut ptr" := by rfl

/--
Test: nested slice lowering.
-/
theorem nested_slice_lowered :
  let outer := ZigDialect.constSlice "u8"
  let inner := ZigDialect.constSlice "i32"
  let result := lowerNestedSlice outer inner "call"
  result = "nested_ch_slice with lifetime call" := by rfl

/--
Test: invalid type for slice lowering.
-/
theorem invalid_type_returns_invalid :
  let t := ZigType.mk .zig_struct "MyStruct" []
  let lowering := lowerConstSlice t "call"
  lowering.chimera_type = "invalid" := by rfl

end Chimera.ZigAdapter