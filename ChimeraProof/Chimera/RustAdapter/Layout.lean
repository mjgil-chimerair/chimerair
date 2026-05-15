--! Chimera.RustAdapter.Layout
--!
--! Lean model for Rust layout information and fingerprints.

import Chimera.RustAdapter

namespace Chimera.RustAdapter.Layout

/--
  Layout information for a Rust type.
-/
structure LayoutInfo where
  size : Nat
  alignment : Nat
  fields : List FieldLayout
  isCyclic : Bool

/--
  Layout of a struct/enum field.
-/
structure FieldLayout where
  name : Option String
  offset : Nat
  ty : String

/--
  Enum layout variant.
-/
structure VariantLayout where
  name : String
  index : Nat
  offset : Nat
  fields : List FieldLayout

/--
  Niche encoding information for optimizations.
-/
structure NicheInfo where
  offset : Nat
  size : Nat
  validRange : Nat × Nat

/--
  Discriminant kind for enums.
-/
inductive DiscriminantKind where
  | explicit (repr : String)
  | niche (niche : NicheInfo)
  | pointerlike

/--
  ABI representation of a type.
  
  - `C`: C ABI compatible
  - `Rust`: Rust ABI
  - `Transparent`: Transparent wrapper
  - `Vector`: SIMD vector
  - `Scalar`: Scalar value
  - `ScalarPair`: Pair of scalars
-/
inductive AbiKind where
  | c
  | rust
  | transparent
  | vector (size : Nat)
  | scalar (value : ScalarValue)
  | scalarPair (first : ScalarValue, second : ScalarValue)

/--
  Scalar value representation.
-/
structure ScalarValue where
  value : Nat
  validRange : Nat × Nat

/--
  Layout fingerprint for caching/invalidation.
  
  Deterministic hash of layout information.
-/
structure LayoutFingerprint where
  typeKind : TypeKind
  representation : String
  fields : List (String × Nat)  -- field name/offset pairs
  size : Nat
  alignment : Nat
  variants : List VariantLayout
  target : String
  rustcVersion : String

/--
  Kinds of types.
-/
inductive TypeKind where
  | primitive
  | array
  | slice
  | str
  | tuple
  | structKind
  | enumKind
  | unionKind
  | foreign
  | closure
  | generator
  | pointer
  | function
  | never
  | tupleStruct
  | tupleVariant
  | fnPointer

end Chimera.RustAdapter.Layout
