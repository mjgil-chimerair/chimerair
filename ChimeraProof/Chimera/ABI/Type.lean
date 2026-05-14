-- ChimeraProof ABI: Semantic Types
-- Chimera semantic type system for the proof system.

import Chimera.Foundation
import Chimera.ABI.PhysicalType

namespace Chimera

/--
Mutability of a reference.
-/
inductive Mutability where
  | const
  | mut
deriving Repr, BEq

/--
Lifetime of a borrowed value.
-/
inductive Lifetime where
  | call      -- Lifetime limited to the current call
  | static    -- Static lifetime
  | owner : Symbol → Lifetime  -- Lifetime tied to an owner
deriving Repr, BEq

/--
Ownership kind for Chimera types.
-/
inductive Ownership where
  | borrow
  | borrowMut
  | owned
  | raw
deriving Repr, BEq

/--
Encoding for strings.
-/
inductive Encoding where
  | utf8
  | ascii
  | platform
deriving Repr, BEq

/--
ChType is the semantic type system for ChimeraIR.
-/
inductive ChType where
  | unit
  | bool
  | i8 | i16 | i32 | i64
  | u8 | u16 | u32 | u64
  | usize | isize
  | f32 | f64
  | status
  | error
  | allocator
  | ptr : ChType → Mutability → ChType
  | rawptr : ChType → ChType
  | borrow : ChType → Lifetime → ChType
  | borrowMut : ChType → Lifetime → ChType  -- Now has explicit Lifetime
  | owned : ChType → ChType
  | out : ChType → ChType
  | inout : ChType → ChType
  | slice : ChType → Ownership → ChType
  | str : Encoding → Ownership → ChType
  | opaque : Symbol → ChType
  | result : ChType → ChType → ChType
deriving Repr, BEq

namespace ChType

/--
Check if a type is a primitive ABI type.
-/
def isPrimitive : ChType → Bool
  | .i8 | .i16 | .i32 | .i64 => true
  | .u8 | .u16 | .u32 | .u64 => true
  | .usize | .isize => true
  | .f32 | .f64 => true
  | _ => false

/--
Check if a type contains a borrow (not including raw pointers).
.ptr is a raw pointer type, not a borrow - use containsRawPtr for that.
-/
def containsBorrow : ChType → Bool
  | .ptr _ _ => false  -- ptr is raw pointer, not a borrow
  | .rawptr _ => false
  | .borrow _ _ => true
  | .borrowMut _ _ => true
  | .owned t => t.containsBorrow
  | .out t => t.containsBorrow
  | .inout t => t.containsBorrow
  | .slice t _ => t.containsBorrow
  | .str _ _ => false
  | .opaque _ => false
  | .result ok err => ok.containsBorrow || err.containsBorrow
  | _ => false

/--
Check if a type contains a raw pointer.
-/
def containsRawPtr : ChType → Bool
  | .rawptr _ => true
  | .ptr _ _ => false
  | .owned t => t.containsRawPtr
  | .out t => t.containsRawPtr
  | .inout t => t.containsRawPtr
  | .slice t _ => t.containsRawPtr
  | .result ok err => ok.containsRawPtr || err.containsRawPtr
  | _ => false

/--
Check if type is passed directly as a physical result.
-/
def isDirectResult : ChType → Bool
  | .ptr _ _ => true
  | .rawptr _ => true
  | .result _ _ => true
  | _ => false

end ChType

/--
Predicate: is this an ABI-legal type?
-/
def AbiLegalType (target : Target) (ty : ChType) : Prop :=
  match ty with
  | .ptr inner m =>
    AbiLegalType target inner
  | .rawptr inner =>
    AbiLegalType target inner
  | .borrow t _ =>
    AbiLegalType target t
  | .borrowMut t _ =>
    AbiLegalType target t
  | .owned t =>
    AbiLegalType target t
  | .out t =>
    AbiLegalType target t
  | .inout t =>
    AbiLegalType target t
  | .slice t _ =>
    AbiLegalType target t
  | .opaque _ => True
  | .result _ _ => True
  | _ => ty.isPrimitive

/--
Predicate: is this a safe boundary type?
Result<T,E> cannot cross physical ABI directly - must be lowered to ch_status + out params.
-/
def SafeBoundaryType : ChType → Prop
  | .rawptr _ => False
  | .borrow t _ => SafeBoundaryType t
  | .borrowMut t _ => SafeBoundaryType t
  | .owned t => SafeBoundaryType t
  | .out t => SafeBoundaryType t
  | .inout t => SafeBoundaryType t
  | .slice t _ => SafeBoundaryType t
  | .str _ _ => True
  | .opaque _ => True
  | .result _ _ => False  -- Result cannot cross ABI directly
  | .status => True
  | .error => True
  | .allocator => True
  | .unit | .bool | .i8 | .i16 | .i32 | .i64 | .u8 | .u16 | .u32 | .u64 | .usize | .isize | .f32 | .f64 => True
  | .ptr _ _ => False

/--
Predicate: does this type require drop?
-/
def RequiresDrop : ChType → Prop
  | .owned _ => True
  | .slice _ _ => True
  | .opaque _ => True
  | .result ok err => RequiresDrop ok ∨ RequiresDrop err
  | _ => False

/--
Executable version: does this type require drop?
-/
def requiresDrop : ChType → Bool
  | .owned _ => true
  | .slice _ _ => true
  | .opaque _ => true
  | .result ok err => requiresDrop ok || requiresDrop err
  | _ => false

/--
Executable and propositional drop requirements agree exactly.
-/
theorem requiresDrop_eq_true_iff (ty : ChType) :
  requiresDrop ty = true ↔ RequiresDrop ty := by
  induction ty with
  | unit | bool | i8 | i16 | i32 | i64 | u8 | u16 | u32 | u64
    | usize | isize | f32 | f64 | status | error | allocator =>
      simp [requiresDrop, RequiresDrop]
  | ptr inner mut =>
      simp [requiresDrop, RequiresDrop]
  | rawptr inner =>
      simp [requiresDrop, RequiresDrop]
  | borrow inner lt =>
      simp [requiresDrop, RequiresDrop]
  | borrowMut inner lt =>
      simp [requiresDrop, RequiresDrop]
  | owned inner =>
      simp [requiresDrop, RequiresDrop]
  | out inner =>
      simp [requiresDrop, RequiresDrop]
  | inout inner =>
      simp [requiresDrop, RequiresDrop]
  | slice inner ownership =>
      simp [requiresDrop, RequiresDrop]
  | str encoding ownership =>
      simp [requiresDrop, RequiresDrop]
  | opaque sym =>
      simp [requiresDrop, RequiresDrop]
  | result ok err ihOk ihErr =>
      simp [requiresDrop, RequiresDrop, ihOk, ihErr, Bool.or_eq_true]

/--
The complementary false case follows from the exact correspondence theorem.
-/
theorem requiresDrop_eq_false_iff (ty : ChType) :
  requiresDrop ty = false ↔ ¬ RequiresDrop ty := by
  constructor
  · intro hFalse hProp
    have hTrue : requiresDrop ty = true := (requiresDrop_eq_true_iff ty).2 hProp
    cases hFalse.trans hTrue.symm
  · intro hNotProp
    cases hBool : requiresDrop ty with
    | false => rfl
    | true =>
        have hProp : RequiresDrop ty := (requiresDrop_eq_true_iff ty).1 hBool
        exact False.elim (hNotProp hProp)

/--
Check if type contains raw pointer (escapes ownership tracking).
-/
def isRawPtrEscaping : ChType → Bool
  | .rawptr _ => true
  | _ => false

/--
Check if type is POD (Plain Old Data) - can be passed by value.
-/
def isPOD : ChType → Bool
  | .i8 | .i16 | .i32 | .i64 => true
  | .u8 | .u16 | .u32 | .u64 => true
  | .usize | .isize => true
  | .f32 | .f64 => true
  | .bool | .unit => true
  | .status | .error | .allocator => true
  | _ => false

/--
Check if type is an opaque handle (non-transparent resource).
-/
def isOpaqueHandle : ChType → Bool
  | .opaque _ => true
  | _ => false

/--
Check if type represents an owned resource (needs drop).
-/
def isOwnedResource : ChType → Bool
  | .owned _ => true
  | _ => false

/--
Check if type is C-compatible (no special ABI semantics).
-/
def isCCompatible : ChType → Bool
  | .i8 | .i16 | .i32 | .i64 => true
  | .u8 | .u16 | .u32 | .u64 => true
  | .usize | .isize => true
  | .f32 | .f64 => true
  | .bool | .unit => true
  | .status | .error | .allocator => true
  | .ptr _ _ => true
  | .rawptr _ => true
  | .opaque _ => true
  | _ => false

/--
Lifetime context for escape checking.
-/
inductive LifetimeContext where
  | returnValue   -- In a direct return value
  | argument      -- As a function argument
  | resultOk      -- Inside result's ok type
  | resultErr     -- Inside result's err type
  | ownedWrapper  -- Inside an owned wrapper
  | sliceElement  -- As slice element type
  | stringElement -- As string element type
deriving Repr, BEq

/--
Check if a lifetime is valid in a given context.
.call lifetime is valid only in:
- Function arguments

A .call borrow in a direct return, result, owned wrapper, slice, or string context
\"escapes\" the call and is invalid.
-/
def lifetimeIsValidInContext (lt : Lifetime) (ctx : LifetimeContext) : Bool :=
  match lt with
  | .call =>
    match ctx with
    | .returnValue => false
    | .argument => true
    | .resultOk => false
    | .resultErr => false
    | .ownedWrapper => false
    | .sliceElement => false
    | .stringElement => false
  | .static => true
  | .owner _ => true

/--
Check if a type contains a borrowing construct that escapes its context.
This rejects `borrow<T, call>` inside returns, nested results, owned wrappers, slices, and strings.
-/
def containsEscapingBorrow : ChType → Bool
  | .borrow _ .call => true
  | .borrowMut _ .call => true
  | .result ok err => containsEscapingBorrow ok || containsEscapingBorrow err
  | .owned inner => containsEscapingBorrow inner
  | .slice ty _ => containsEscapingBorrow ty
  | .str _ _ => false  -- strings don't contain borrow in their element type directly
  | _ => false

/--
Check if a type contains a call-lifetime borrow (escaping or not).
-/
def containsCallLifetimeBorrow : ChType → Bool
  | .borrow _ .call => true
  | .borrowMut _ .call => true
  | .result ok err => containsCallLifetimeBorrow ok || containsCallLifetimeBorrow err
  | .owned inner => containsCallLifetimeBorrow inner
  | .slice ty _ => containsCallLifetimeBorrow ty
  | _ => false

/--
Call-lifetime borrows are valid only as arguments.
-/
theorem call_lifetime_valid_only_for_arguments :
  lifetimeIsValidInContext .call .argument = true ∧
    lifetimeIsValidInContext .call .returnValue = false ∧
    lifetimeIsValidInContext .call .resultOk = false ∧
    lifetimeIsValidInContext .call .resultErr = false ∧
    lifetimeIsValidInContext .call .ownedWrapper = false ∧
    lifetimeIsValidInContext .call .sliceElement = false ∧
    lifetimeIsValidInContext .call .stringElement = false := by
  decide

/--
Every escaping borrow contains a call-lifetime borrow.
-/
theorem containsEscapingBorrow_implies_containsCallLifetimeBorrow (ty : ChType) :
  containsEscapingBorrow ty = true → containsCallLifetimeBorrow ty = true := by
  induction ty with
  | unit | bool | i8 | i16 | i32 | i64 | u8 | u16 | u32 | u64
    | usize | isize | f32 | f64 | status | error | allocator
    | ptr .. | rawptr .. | out .. | inout .. | str .. | opaque .. =>
      intro h
      simp [containsEscapingBorrow] at h
  | borrow inner lt =>
      intro h
      cases lt <;> simp [containsEscapingBorrow, containsCallLifetimeBorrow] at h ⊢
  | borrowMut inner lt =>
      intro h
      cases lt <;> simp [containsEscapingBorrow, containsCallLifetimeBorrow] at h ⊢
  | owned inner ih =>
      intro h
      simp [containsEscapingBorrow, containsCallLifetimeBorrow] at h ⊢
      exact ih h
  | slice inner ownership ih =>
      intro h
      simp [containsEscapingBorrow, containsCallLifetimeBorrow] at h ⊢
      exact ih h
  | result ok err ihOk ihErr =>
      intro h
      simp [containsEscapingBorrow, containsCallLifetimeBorrow] at h ⊢
      cases h with
      | inl hOk => exact ihOk hOk
      | inr hErr => exact ihErr hErr

end Chimera
