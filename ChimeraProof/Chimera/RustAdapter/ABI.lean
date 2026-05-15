--! Chimera.RustAdapter.ABI
--!
--! Lean model for Rust ABI fingerprinting and validation.

import Chimera.RustAdapter
import Chimera.ABI

namespace Chimera.RustAdapter.ABI

/--
  ABI fingerprint for Rust functions.
  
  Captures all information needed to validate ABI compatibility:
  - Symbol name
  - Calling convention
  - Semantic types
  - Physical ABI
  - Layout references
  - Ownership model
  - Panic policy
  - Effect set
-/
structure ABIFingerprint where
  symbol : String
  callConv : CallingConvention
  semanticTypes : List String
  physicalABI : PhysicalABI
  layoutRefs : List String
  ownership : OwnershipInfo
  panicPolicy : PanicPolicy
  effectSet : EffectSet
  target : String
  schemaVersion : Nat

/--
  Physical ABI description.
-/
structure PhysicalABI where
  params : List ParamLayout
  returnLayout : ReturnLayout
  registerSize : Nat

/--
  Parameter layout in physical ABI.
-/
structure ParamLayout where
  name : String
  ty : String
  offset : Nat
  size : Nat
  alignment : Nat
  passedIn : ParamPassing

/--
  How a parameter is passed.
-/
inductive ParamPassing where
  | register
  | stack
  | pointer
  | pointerPair

/--
  Return value layout.
-/
structure ReturnLayout where
  layout : Option String
  isDirect : Bool
  viaPointer : Bool
  pointerOffset : Option Nat

/--
  Ownership information for a function.
-/
structure OwnershipInfo where
  takesOwnership : List String
  returnsOwnership : List String
  borrows : List BorrowInfo

/--
  Borrow information.
-/
structure BorrowInfo where
  paramName : String
  lifetime : String
  isMutable : Bool

/--
  Effect set for a function.
-/
structure EffectSet where
  mayPanic : Bool
  mayAlloc : Bool
  mayCall : Bool
  mayAccessUnsafe : Bool
  effects : List EffectKind

/--
  Kinds of effects.
-/
inductive EffectKind where
  | io
  | memoryAllocation
  | externalCall
  | systemTime
  | randomness
  | unsanitary

/--
  Calling conventions.
-/
inductive CallingConvention where
  | cconvC : CConv
  | rust
  | stdcall
  | fastcall
  | vectorcall
  | thiscall
  | aapcs
  | win64
  | sysv64

/--
  C calling conventions.
-/
inductive CConv where
  | c
  | stdcall
  | fastcall
  | thiscall
  | vectorcall

/--
  Validate ABI fingerprint compatibility.
  
  Two functions have compatible ABI if their fingerprints match
  under the stated assumptions.
-/
structure ABICompatibility where
  sameCallConv : Bool
  sameParamLayout : Bool
  sameReturnLayout : Bool
  sameOwnership : Bool
  samePanicPolicy : Bool

end Chimera.RustAdapter.ABI
