-- ChimeraProof ABI: Physical Types
-- Physical ABI type representation.

import Chimera.Foundation

namespace Chimera

/--
Signedness of an integer.
-/
inductive Signedness where
  | signed
  | unsigned
deriving Repr, BEq

/--
Calling convention for function pointers.
-/
inductive CallingConvention where
  | cdecl
  | stdcall
  | fastcall
  | sysv
  | windows
  | wasm
deriving Repr, BEq

/--
Address space for pointer types.
B.36: Different address spaces have different semantics (device vs host memory, etc).
-/
inductive AddressSpace where
  | null      -- null/unknown address space
  | generic   -- generic memory
  | device    -- device/mapped memory
  | io        -- memory-mapped I/O
  | stack     -- stack memory (address taken)
  | code      -- code memory
deriving Repr, BEq

/--
Physical ABI types (what actually goes in memory/registers).
B.36: Ptr now includes pointee type and address space for full expressiveness.
-/
inductive PhysType where
  | void
  | int : Nat → Signedness → PhysType  -- width in bits, signedness
  | float : Nat → PhysType  -- width in bits (32 or 64)
  | ptr : Option PhysType → AddressSpace → PhysType  -- pointee type (none = opaque), address space
  | array : Nat → PhysType → PhysType  -- length, element type
  | struct : List (String × PhysType) → PhysType  -- (field name, field type) pairs
  | fnptr : CallingConvention → List PhysType → PhysType → PhysType  -- cc, params, return
deriving Repr, BEq

/--
Field layout with offset, size and alignment (computed after layout).
-/
structure FieldLayout where
  fieldName : String
  offset    : Nat
  size      : Nat
  align     : Nat
deriving Repr, BEq

namespace PhysType

/--
Get the natural number tag for a physical type.
-/
def toTag : PhysType → Nat
  | .void => 0
  | .int w s => match s with | .signed => w | .unsigned => 100 + w
  | .float w => 200 + w
  | .ptr _ _ => 300  -- B.36: ptr now carries pointee and address space
  | .array n t => 400 + n * 1000 + toTag t
  | .struct fields => 500 + fields.length
  | .fnptr .. => 600

/--
Check if a physical type is an integer type.
-/
def isInt : PhysType → Bool
  | .int _ _ => true
  | _ => false

/--
Check if a physical type is a floating point type.
-/
def isFloat : PhysType → Bool
  | .float _ => true
  | _ => false

/--
Get width of an integer or float type.
For pointers, width depends on the target.
-/
def getWidth (target : Target) : PhysType → Nat
  | .int w _ => w
  | .float w => w
  | .ptr _ _ => target.ptrWidth
  | .struct fields => structWidth target fields
  | .array n elem => n * getWidth target elem
  | .fnptr .. => target.ptrWidth
  | .void => 0
where
  structWidth (target : Target) : List (String × PhysType) → Nat
    | [] => 0
    | f :: rest => getWidth target f.2 + structWidth target rest

end PhysType

end Chimera
