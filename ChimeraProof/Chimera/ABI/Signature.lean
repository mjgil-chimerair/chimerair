-- ChimeraProof ABI: Function Signature
-- Function signature representation and validation.

import Chimera.Foundation
import Chimera.ABI.Type
import Chimera.ABI.PhysicalType

namespace Chimera

inductive ReturnSpec : Type where
  | void : ReturnSpec
  | value (n : Nat) : ReturnSpec
  | values (ns : List Nat) : ReturnSpec
deriving Repr, BEq

structure Param : Type where
  name : String
  ty : ChType
deriving Repr, BEq

structure SemanticSignature : Type where
  params : List Param
  returns : ChType
  isVarargs : Bool := false
deriving Repr, BEq

structure PhysicalSignature : Type where
  params : List PhysType
  returns : ReturnSpec
  callingConv : CallingConvention
deriving Repr, BEq

/--
Check if two ChTypes are ABI-compatible (can be passed across ABI boundary).
-/
def ChType.compatibleWith (a b : ChType) : Bool :=
  match a, b with
  | .i8, .i8 => true
  | .i16, .i16 => true
  | .i32, .i32 => true
  | .i64, .i64 => true
  | .u8, .u8 => true
  | .u16, .u16 => true
  | .u32, .u32 => true
  | .u64, .u64 => true
  | .usize, .usize => true
  | .isize, .isize => true
  | .f32, .f32 => true
  | .f64, .f64 => true
  | .bool, .bool => true
  | .unit, .unit => true
  | .status, .status => true
  | .error, .error => true
  | .allocator, .allocator => true
  | .ptr aTy aMut, .ptr bTy bMut => aTy.compatibleWith bTy && aMut == bMut
  | .rawptr _, .rawptr _ => true
  | .borrow aTy aLt, .borrow bTy bLt => aTy.compatibleWith bTy && aLt == bLt
  | .borrowMut aTy aLt, .borrowMut bTy bLt => aTy.compatibleWith bTy && aLt == bLt
  | .owned aTy, .owned bTy => aTy.compatibleWith bTy
  | .out aTy, .out bTy => aTy.compatibleWith bTy
  | .inout aTy, .inout bTy => aTy.compatibleWith bTy
  | .slice aTy aOwn, .slice bTy bOwn => aOwn == bOwn
  | .str aEnc aOwn, .str bEnc bOwn => aEnc == bEnc && aOwn == bOwn
  | .result aOk aErr, .result bOk bErr => aOk.compatibleWith bOk && aErr.compatibleWith bErr
  | .opaque a, .opaque b => a == b
  | _, _ => false

/--
Check if two PhysTypes are physically compatible.
-/
def PhysType.compatibleWith (a b : PhysType) : Bool :=
  match a, b with
  | .void, .void => true
  | .int wa sa, .int wb sb => wa == wb && sa == sb
  | .float wa, .float wb => wa == wb
  | .ptr, .ptr => true
  | .array na ea, .array nb eb => na == nb && ea.compatibleWith eb
  | .struct fa, .struct fb => fa.length == fb.length && structFieldsCompatible fa fb
  | .fnptr ca pa ra, .fnptr cb pb rb => ca == cb && pa.length == pb.length && paramsCompatible pa pb && ra.compatibleWith rb
  | _, _ => false
where
  structFieldsCompatible : List (String × PhysType) → List (String × PhysType) → Bool
    | [], [] => true
    | (fn, ft) :: fr, (gn, gt) :: gr => fn == gn && ft.compatibleWith gt && structFieldsCompatible fr gr
    | _, _ => false
  paramsCompatible : List PhysType → List PhysType → Bool
    | [], [] => true
    | a :: ar, b :: br => a.compatibleWith b && paramsCompatible ar br
    | _, _ => false

/--
Check if two signatures are compatible (used for import/export matching).
-/
def SemanticSignature.compatibleWith (a b : SemanticSignature) : Bool :=
  (a.isVarargs == b.isVarargs) &&
  (a.params.length == b.params.length) &&
  List.all (a.params.zip b.params) (fun (pa, pb) => pa.ty.compatibleWith pb.ty) &&
  a.returns.compatibleWith b.returns

/--
Check if two physical signatures are compatible.
-/
def PhysicalSignature.compatibleWith (a b : PhysicalSignature) : Bool :=
  (a.callingConv == b.callingConv) &&
  (a.params.length == b.params.length) &&
  List.all (a.params.zip b.params) (fun (pa, pb) => pa.compatibleWith pb) &&
  match a.returns, b.returns with
  | .void, .void => true
  | .value na, .value nb => na == nb
  | .values na, .values nb => na == nb
  | _, _ => false

end Chimera