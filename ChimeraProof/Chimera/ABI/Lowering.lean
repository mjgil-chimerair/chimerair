-- ChimeraProof ABI: Type Lowering
-- Lowering from semantic ChType to physical PhysType.

import Chimera.Foundation
import Chimera.ABI.Type
import Chimera.ABI.PhysicalType
import Chimera.ABI.Signature
import Chimera.ABI.Layout
import Chimera.ABI.CanonicalStructs

namespace Chimera

/--
ABI lowering error.
-/
inductive LoweringError where
  | illegalType (ty : ChType)
  | resultNotDirect (ty : ChType)
  | borrowInReturn (ty : ChType)
  | unsupportedFeature (msg : String)
deriving Repr, BEq

/--
Lower a ChType to a PhysType for FFI boundary.
-/
def lowerType (target : Target) : ChType → Except LoweringError PhysType
  | .unit => .ok .void
  | .bool => .ok (.int 8 .unsigned)
  | .i8 => .ok (.int 8 .signed)
  | .i16 => .ok (.int 16 .signed)
  | .i32 => .ok (.int 32 .signed)
  | .i64 => .ok (.int 64 .signed)
  | .u8 => .ok (.int 8 .unsigned)
  | .u16 => .ok (.int 16 .unsigned)
  | .u32 => .ok (.int 32 .unsigned)
  | .u64 => .ok (.int 64 .unsigned)
  | .usize => .ok (.int target.usizeWidth .unsigned)
  | .isize => .ok (.int target.usizeWidth .signed)
  | .f32 => .ok (.float 32)
  | .f64 => .ok (.float 64)
  | .status => .ok (.int 32 .signed)
  | .error => .ok (.int 32 .unsigned)
  | .allocator => .ok .ptr
  | .ptr _ _ => .ok .ptr
  | .rawptr _ => .ok .ptr
  | .borrow _ _ => lowerBorrow target
  | .borrowMut _ _ => lowerBorrow target
  | .owned innerTy => lowerOwned target innerTy
  | .out ty => lowerOutParam target ty
  | .inout ty => .ok .ptr
  | .slice elemTy _ => lowerSlice target elemTy
  | .str enc _ => lowerString target enc
  | .opaque _ => .ok .ptr
  | .result _ _ => .error (.unsupportedFeature "result type - use lowerResultSignature")
where
  lowerBorrow (_ : Target) : Except LoweringError PhysType := do
    .ok ch_borrow_str_phys

  lowerOwned (target : Target) (innerTy : ChType) : Except LoweringError PhysType := do
    match innerTy with
    | .opaque _ => .ok ch_handle_phys
    | .slice _ _ => .ok ch_owned_bytes_phys
    | _ => .ok ch_handle_phys

  lowerOutParam (target : Target) (ty : ChType) : Except LoweringError PhysType := do
    -- out parameters are always passed as pointer to the value
    .ok .ptr

  lowerSlice (target : Target) (elemTy : ChType) : Except LoweringError PhysType := do
    let elemPhys ← lowerType target elemTy
    .ok (ch_slice_phys elemPhys)

  lowerString (target : Target) (enc : Encoding) : Except LoweringError PhysType := do
    match enc with
    | .utf8 => .ok ch_borrow_str_phys
    | .ascii => .ok ch_borrow_str_phys
    | .platform => .ok ch_borrow_str_phys

namespace LoweringError

/--
Error message.
-/
def toString : LoweringError → String
  | .illegalType _ty => "illegal type"
  | .resultNotDirect _ty => "result cannot be direct"
  | .borrowInReturn _ty => "borrow in return type"
  | .unsupportedFeature msg => s!"unsupported: {msg}"

end LoweringError

/--
Check if a physical type is ABI-legal on a target.
-/
def AbiLegalPhysical (target : Target) : PhysType → Prop
  | .void => True
  | .int w s => w > 0 ∧ (w = 8 ∨ w = 16 ∨ w = 32 ∨ w = 64)
  | .float w => w = 32 ∨ w = 64
  | .ptr => True
  | .array n elem => n > 0 ∧ AbiLegalPhysical target elem
  | .struct fields => allStructFieldsLegal target fields
  | .fnptr _ params ret => allParamsLegal target params ∧ AbiLegalPhysical target ret
where
  allStructFieldsLegal (target : Target) : List (String × PhysType) → Prop
    | [] => True
    | (_, ty) :: rest => AbiLegalPhysical target ty ∧ allStructFieldsLegal target rest

  allParamsLegal (target : Target) : List PhysType → Prop
    | [] => True
    | ty :: rest => AbiLegalPhysical target ty ∧ allParamsLegal target rest

/--
Check if a physical type represents a semantic type.
-/
def Represents (target : Target) : PhysType → ChType → Prop
  | .void, .unit => True
  | .int w Signedness.signed, .i8 => w = 8
  | .int w Signedness.signed, .i16 => w = 16
  | .int w Signedness.signed, .i32 => w = 32
  | .int w Signedness.signed, .i64 => w = 64
  | .int w Signedness.unsigned, .u8 => w = 8
  | .int w Signedness.unsigned, .u16 => w = 16
  | .int w Signedness.unsigned, .u32 => w = 32
  | .int w Signedness.unsigned, .u64 => w = 64
  | .int w Signedness.unsigned, .usize => w = target.usizeWidth
  | .int w Signedness.signed, .isize => w = target.usizeWidth
  | .int w Signedness.signed, .status => w = 32
  | .int w Signedness.unsigned, .error => w = 32
  | .float w, .f32 => w = 32
  | .float w, .f64 => w = 64
  | .ptr, .ptr _ _ => True
  | .ptr, .rawptr _ => True
  | .ptr, .allocator => True
  | .ptr, .out _ => True
  | .ptr, .inout _ => True
  | .ptr, .opaque _ => True
  | .struct [⟨"ptr", .ptr⟩, ⟨"len", .int 64 _⟩], .slice _ _ => True
  | .struct [⟨"ptr", .ptr⟩, ⟨"len", .int 64 _⟩, ⟨"lifetime", .int 32 _⟩], .borrow _ _ => True
  | .struct [⟨"ptr", .ptr⟩, ⟨"len", .int 64 _⟩, ⟨"lifetime", .int 32 _⟩], .borrowMut _ _ => True
  | .struct [⟨"ptr", .ptr⟩, ⟨"len", .int 64 _⟩, ⟨"lifetime", _⟩], .str _ _ => True
  | .struct [⟨"ptr", .ptr⟩, ⟨"len", _⟩, ⟨"capacity", _⟩, ⟨"allocator_id", _⟩], .owned (.slice _ _) => True
  | .struct [⟨"ptr", .ptr⟩, ⟨"drop_fn", .fnptr .cdecl [.ptr] .void⟩, ⟨"size", .int 64 _⟩], .owned _ => True
  | _, _ => False

/--
All physical parameters in a lowered list are ABI-legal.
-/
def PhysParamsAbiLegal (target : Target) : List PhysType → Prop
  | [] => True
  | ty :: rest => AbiLegalPhysical target ty ∧ PhysParamsAbiLegal target rest

/--
Physical parameter list represents the semantic parameter list pointwise.
-/
def PhysParamsRepresent (target : Target) : List PhysType → List Param → Prop
  | [], [] => True
  | phys :: physRest, param :: paramRest =>
      Represents target phys param.ty ∧ PhysParamsRepresent target physRest paramRest
  | _, _ => False

/--
Number of synthetic physical parameters introduced by return lowering.
-/
def loweredReturnParamCount : ChType → Nat
  | .unit => 0
  | .result _ _ => 2
  | _ => 1

/--
Return lowering relation for the current ABI implementation.
-/
def LoweredReturnRepresents (target : Target) (retTy : ChType)
  (retSpec : ReturnSpec) (prefixParams : List PhysType) : Prop :=
  match retTy, retSpec, prefixParams with
  | .unit, .void, [] => True
  | .result okTy errTy, .value 1, [okPtrTy, errPtrTy] =>
      Represents target (.int 32 .signed) .status ∧
      AbiLegalPhysical target okPtrTy ∧
      AbiLegalPhysical target errPtrTy ∧
      Represents target okPtrTy (.out okTy) ∧
      Represents target errPtrTy (.out errTy)
  | directTy, .value 1, [phys] =>
      AbiLegalPhysical target phys ∧ Represents target phys directTy
  | _, _, _ => False

/--
Theorem: lowerType is sound - if it succeeds, the result is ABI-legal and represents the input.

Soundness requires proving:
1. AbiLegalPhysical target pty - the physical type is ABI-legal on the target
2. Represents target pty ty - the physical type represents the semantic type

This theorem is non-trivial because lowerType is defined recursively with helper functions
in a where clause, making structural induction complex. The theorem is stated here
and the proof obligations are documented for future work.
-/
theorem lowerType_sound (target : Target) (ty : ChType) (pty : PhysType) :
  lowerType target ty = Except.ok pty →
  AbiLegalPhysical target pty ∧ Represents target pty ty := by
  intro h
  cases ty <;> simp [lowerType, AbiLegalPhysical, Represents, ch_borrow_str_phys,
    ch_borrow_str_phys_fields, ch_handle_phys, ch_handle_phys_fields,
    ch_owned_bytes_phys, ch_owned_bytes_phys_fields, ch_slice_phys, ch_slice_phys_fields] at h ⊢
  all_goals try aesop
  · cases h
  · cases h
  · cases h
  · cases h
  · cases h
  · cases h
  · cases h
  · cases h

/--
Lower a semantic signature to a physical signature.
Handles result types by converting them to ch_status + out parameters.
-/
def lowerSignature (target : Target) (sig : SemanticSignature) : Except LoweringError PhysicalSignature := do
  let params ← sig.params.mapM (fun p => lowerType target p.ty)
  let (returns, newParams) ← lowerReturn target sig.returns
  .ok { params := newParams ++ params, returns := returns, callingConv := .cdecl }
where
  lowerReturn (target : Target) (ret : ChType) : Except LoweringError (ReturnSpec × List PhysType) := do
    match ret with
    | .result okTy errTy => do
      -- Result<T,E> lowers to ch_status return + out_ok ptr + out_err ptr
      -- Lower the ok/err types to verify they are valid, then return the out-params
      let _okPhys ← lowerType target okTy
      let _errPhys ← lowerType target errTy
      .ok (.value 1, [.ptr, .ptr])
    | .unit => .ok (.void, [])
    | _ => do
      let phys ← lowerType target ret
      .ok (.value 1, [phys])

/--
Theorem: safe_boundary_no_native_result - semantic Result cannot cross ABI directly.
-/
theorem safe_boundary_no_native_result (ty : ChType) :
  SafeBoundaryType ty →
  ty.isDirectResult = false := by
  intro h
  -- Result<T,E> is NOT in SafeBoundaryType (see Type.lean SafeBoundaryType definition)
  -- Therefore if SafeBoundaryType ty holds, ty cannot be a result type
  -- and isDirectResult must be false
  cases ty <;> simp [SafeBoundaryType] at h
  <;> rfl

/--
Lowered semantic parameter list is ABI-legal and represented pointwise.
-/
theorem lowerParams_sound (target : Target) (params : List Param) (physParams : List PhysType) :
  params.mapM (fun p => lowerType target p.ty) = Except.ok physParams →
    PhysParamsAbiLegal target physParams ∧ PhysParamsRepresent target physParams params := by
  induction params generalizing physParams with
  | nil =>
      intro h
      simp at h
      cases h
      simp [PhysParamsAbiLegal, PhysParamsRepresent]
  | cons param rest ih =>
      intro h
      simp [List.mapM] at h
      rcases h with ⟨paramPhys, restPhys, hLower, hRest, hEq⟩
      subst hEq
      have hParam := lowerType_sound target param.ty paramPhys hLower
      have hTail := ih hRest
      exact ⟨hParam.1, hTail.1, hParam.2, hTail.2⟩

/--
Theorem: lowerSignature is sound - lowered physical signature represents semantic signature.

Soundness requires proving:
1. Each param's physical type is ABI-legal and represents the semantic type
2. The return spec correctly encodes the semantic return type

For result types, the lowering produces:
- Return spec: .value 1 (one integer return for ch_status)
- Additional params: [ch_status_phys, ptr (out_ok), ptr (out_err)]

This theorem is complex due to the list concatenation in signature lowering.
The proof is pending completion.
-/
theorem lowerSignature_sound (target : Target) (sig : SemanticSignature) (psig : PhysicalSignature) :
  lowerSignature target sig = Except.ok psig →
  psig.callingConv = .cdecl ∧
    LoweredReturnRepresents target sig.returns psig.returns
      (psig.params.take (loweredReturnParamCount sig.returns)) ∧
    PhysParamsAbiLegal target (psig.params.drop (loweredReturnParamCount sig.returns)) ∧
    PhysParamsRepresent target (psig.params.drop (loweredReturnParamCount sig.returns)) sig.params := by
  intro h
  simp [lowerSignature] at h
  rcases h with ⟨loweredInputs, returnSpec, returnPrefix, hInputs, hReturn, hEq⟩
  subst hEq
  have hInputsSound := lowerParams_sound target sig.params loweredInputs hInputs
  constructor
  · rfl
  constructor
  · cases hRet : sig.returns <;> simp [loweredReturnParamCount, LoweredReturnRepresents] at hReturn ⊢
    · cases hReturn
      trivial
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.bool phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.i8 phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.i16 phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.i32 phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.i64 phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.u8 phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.u16 phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.u32 phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.u64 phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.usize phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.isize phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.f32 phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.f64 phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.status phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.error phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target ChType.allocator phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target (.ptr ty mut) phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target (.rawptr ty) phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target (.borrow ty lifetime) phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target (.borrowMut ty lifetime) phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target (.owned ty) phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target (.out ty) phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target (.inout ty) phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target (.slice ty ownership) phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target (.str encoding ownership) phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨phys, hLower, hSpec⟩
      subst hSpec
      have hPhys := lowerType_sound target (.opaque sym) phys hLower
      simpa using hPhys
    · rcases hReturn with ⟨okPhys, errPhys, hOk, hErr, hSpec⟩
      subst hSpec
      have hOkSound := lowerType_sound target okTy okPhys hOk
      have hErrSound := lowerType_sound target errTy errPhys hErr
      simp [LoweredReturnRepresents, Represents, AbiLegalPhysical, hOkSound.1, hErrSound.1]
  constructor
  · simpa [loweredReturnParamCount] using hInputsSound.1
  · simpa [loweredReturnParamCount] using hInputsSound.2

/--
Compatible targets lower `usize` identically.
-/
theorem compatible_usize_lowering_eq {a b : Target} (h : Target.compatible a b) :
  lowerType a .usize = lowerType b .usize := by
  simp [lowerType, Target.compatible_usizeWidth_eq h]

/--
Compatible targets lower `isize` identically.
-/
theorem compatible_isize_lowering_eq {a b : Target} (h : Target.compatible a b) :
  lowerType a .isize = lowerType b .isize := by
  simp [lowerType, Target.compatible_usizeWidth_eq h]

/--
Result signatures lower to a status return plus `out_ok`/`out_error` parameters.
-/
theorem result_signature_uses_status_and_out_params
  (target : Target) (okTy errTy : ChType) :
  lowerSignature target { params := [], returns := .result okTy errTy, isVarargs := false } =
    .ok { params := [.ptr, .ptr], returns := .value 1, callingConv := .cdecl } := by
  simp [lowerSignature]

end Chimera
