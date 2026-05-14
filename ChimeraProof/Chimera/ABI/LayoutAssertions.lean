-- ChimeraProof ABI: Layout Assertions
-- External layout assertion models for C, Rust, and Zig.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Layout
import Chimera.ABI.CanonicalStructs

namespace Chimera.ABI

/--
C static assertion template.
Generates C `_Static_assert` expression for layout verification.
-/
structure CStaticAssert where
  condition : String
  message : String

/--
Render C static assert as C source line.
-/
def CStaticAssert.render (a : CStaticAssert) : String :=
  "_Static_assert(" ++ a.condition ++ ", \"" ++ a.message ++ "\");"

/--
C layout assertion bundle for a struct.
-/
structure CLayoutAssertions where
  struct_name : String
  asserts : List CStaticAssert

/--
Render C layout assertions as C source.
-/
def CLayoutAssertions.render (a : CLayoutAssertions) : String :=
  let header := "// Layout assertions for " ++ a.struct_name ++ "\n"
  let body := a.asserts.foldl (fun acc a => acc ++ a.render ++ "\n") ""
  header ++ body

/--
Rust const assertion template.
Generates Rust `const` assertions for compile-time layout verification.
-/
structure RustConstAssert where
  condition : String
  message : String

/--
Render Rust const assert as Rust source line.
-/
def RustConstAssert.render (a : RustConstAssert) : String :=
  "const _: () = assert!(" ++ a.condition ++ ", \"" ++ a.message ++ "\");"

/--
Rust layout assertion bundle.
-/
structure RustLayoutAssertions where
  struct_name : String
  asserts : List RustConstAssert

/--
Render Rust layout assertions as Rust source.
-/
def RustLayoutAssertions.render (a : RustLayoutAssertions) : String :=
  let header := "// Layout assertions for " ++ a.struct_name ++ "\n"
  let body := a.asserts.foldl (fun acc a => acc ++ a.render ++ "\n") ""
  header ++ body

/--
Zig comptime assertion template.
Generates Zig `@compileTimeAssert` for comptime layout verification.
-/
structure ZigComptimeAssert where
  condition : String
  message : String

/--
Render Zig comptime assert as Zig source line.
-/
def ZigComptimeAssert.render (a : ZigComptimeAssert) : String :=
  "@compileTimeAssert(" ++ a.condition ++ "); // \"" ++ a.message ++ "\""

/--
Zig layout assertion bundle.
-/
structure ZigLayoutAssertions where
  struct_name : String
  asserts : List ZigComptimeAssert

/--
Render Zig layout assertions as Zig source.
-/
def ZigLayoutAssertions.render (a : ZigLayoutAssertions) : String :=
  let header := "// Layout assertions for " ++ a.struct_name ++ "\n"
  let body := a.asserts.foldl (fun acc a => acc ++ a.render ++ "\n") ""
  header ++ body

namespace LayoutAssertions

/--
Generate C static assertions for ch_status.
-/
def chStatusCAsserts : CLayoutAssertions := {
  struct_name := "ch_status",
  asserts := [
    CStaticAssert.mk "sizeof(ch_status) == 4" "ch_status must be 4 bytes",
    CStaticAssert.mk "alignof(ch_status) == 4" "ch_status must be 4-byte aligned"
  ]
}

/--
Generate Rust const assertions for ch_status.
-/
def chStatusRustAsserts : RustLayoutAssertions := {
  struct_name := "ch_status",
  asserts := [
    RustConstAssert.mk "std::mem::size_of::<i32>() == 4" "ch_status must be 4 bytes",
    RustConstAssert.mk "std::mem::align_of::<i32>() == 4" "ch_status must be 4-byte aligned"
  ]
}

/--
Generate Zig comptime assertions for ch_status.
-/
def chStatusZigAsserts : ZigLayoutAssertions := {
  struct_name := "ch_status",
  asserts := [
    ZigComptimeAssert.mk "@sizeOf(i32) == 4" "ch_status must be 4 bytes",
    ZigComptimeAssert.mk "@alignOf(i32) == 4" "ch_status must be 4-byte aligned"
  ]
}

/--
Generate C static assertions for ch_error.
-/
def chErrorCAsserts (target : Target) : CLayoutAssertions := {
  struct_name := "ch_error",
  asserts := [
    CStaticAssert.mk "sizeof(struct ch_error) == 48" "ch_error must be 48 bytes on 64-bit",
    CStaticAssert.mk "_Alignof(struct ch_error) == 8" "ch_error must be 8-byte aligned"
  ]
}

/--
Generate Rust const assertions for ch_error.
-/
def chErrorRustAsserts (target : Target) : RustLayoutAssertions := {
  struct_name := "ch_error",
  asserts := [
    RustConstAssert.mk "std::mem::size_of::<ChError>() == 48" "ch_error must be 48 bytes on 64-bit",
    RustConstAssert.mk "std::mem::align_of::<ChError>() == 8" "ch_error must be 8-byte aligned"
  ]
}

/--
Generate Zig comptime assertions for ch_error.
-/
def chErrorZigAsserts (target : Target) : ZigLayoutAssertions := {
  struct_name := "ch_error",
  asserts := [
    ZigComptimeAssert.mk "@sizeOf(Error) == 48" "ch_error must be 48 bytes on 64-bit",
    ZigComptimeAssert.mk "@alignOf(Error) == 8" "ch_error must be 8-byte aligned"
  ]
}

/--
Generate layout assertions for a declared layout.
-/
def forDeclaredLayout (layout : DeclaredLayout) (target : Target) : (CLayoutAssertions × RustLayoutAssertions × ZigLayoutAssertions) :=
  let c_asserts := layout.fields.map (fun f =>
    CStaticAssert.mk
      ("offsetof(struct " ++ layout.name ++ ", " ++ f.name ++ ") == " ++ Nat.toString f.offset)
      (layout.name ++ "." ++ f.name ++ " offset must be " ++ Nat.toString f.offset)
  )
  let rust_asserts := layout.fields.map (fun f =>
    RustConstAssert.mk
      ("offset_of!(Struct, \"" ++ f.name ++ "\") == " ++ Nat.toString f.offset)
      (layout.name ++ "." ++ f.name ++ " offset must be " ++ Nat.toString f.offset)
  )
  let zig_asserts := layout.fields.map (fun f =>
    ZigComptimeAssert.mk
      ("@offsetOf(" ++ layout.name ++ ", \"" ++ f.name ++ "\") == " ++ Nat.toString f.offset)
      (layout.name ++ "." ++ f.name ++ " offset must be " ++ Nat.toString f.offset)
  )
  (
    CLayoutAssertions.mk layout.name c_asserts,
    RustLayoutAssertions.mk layout.name rust_asserts,
    ZigLayoutAssertions.mk layout.name zig_asserts
  )

end LayoutAssertions

end Chimera.ABI

