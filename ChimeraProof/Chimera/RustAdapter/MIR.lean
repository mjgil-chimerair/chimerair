--! Chimera.RustAdapter.MIR
--!
--! Lean model for Rust MIR (Mid-level IR).
--! Represents normalized MIR bodies extracted from rustc.

import Chimera.RustAdapter.Snapshot

namespace Chimera.RustAdapter.MIR

/--
  A normalized MIR local (variable).
-/
structure Local where
  index : Nat
  ty : String
  isInit : Bool

/--
  Projection into a MIR place.
  
  A place is a location (local, static, field, deref, etc.).
-/
inductive Projection where
  | field (fieldIndex : Nat)
  | deref
  | index (offset : Nat)
  | constantIndex (index : Nat, offset : Nat, fromBuffer : Bool)
  | downcast (variantIndex : Nat)

/--
  A MIR place (memory location).
-/
structure Place where
  local : Nat
  projections : List Projection

/--
  MIR operand (value or constant).
-/
inductive Operand where
  | copy (place : Place)
  | move (place : Place)
  | constant (value : Const)

/--
  A constant value.
-/
structure Const where
  ty : String
  bytes : List UInt8

/--
  MIR statement kinds.
-/
inductive StatementKind where
  | assign (destination : Place, source : Rvalue)
  | dead (local : Local)
  | storageLive (local : Local)
  | storageDead (local : Local)
  | set_discriminant (place : Place, variantIndex : Nat)
  | validate (place : Place, variantIndex : Option Nat)

/--
  MIR terminator kinds.
-/
inductive TerminatorKind where
  | return
  | goto (target : Nat)
  | switchInt (discr : Operand, targets : List (Nat × Nat))
  | call (func : Operand, args : List Operand, destination : Option (Place × Nat))
  | yield
  | noreturn
  | unreachable
  | drop (place : Place, target : Nat, unwind : Nat)
  | assert (condition : AssertKind, target : Nat, unwind : Nat)
  | inlineAsm ( asm : String, constraints : String, destination : Option (Place × Nat))

/--
  Assert condition kinds.
-/
inductive AssertKind where
  | panic (msg : String)
  | overflow (op : String, left : Operand, right : Operand)
  | divisionByZero (left : Operand, right : Operand)
  | invalidValue (ty : String, place : Place)

/--
  MIR Rvalue (right-hand side expression).
-/
inductive Rvalue where
  | use (operand : Operand)
  | repeat (value : Operand, count : Const)
  | ref (place : Place, borrowKind : BorrowKind)
  | threadLocalRef
  | cast (kind : CastKind, operand : Operand, ty : String)
  | binaryOp (op : BinOp, left : Operand, right : Operand)
  | checkedBinaryOp (op : BinOp, left : Operand, right : Operand)
  | nullaryOp (kind : NullOpKind)
  | unaryOp (op : UnOp, operand : Operand)
  | discriminant (place : Place)

/--
  Borrow kinds.
-/
inductive BorrowKind where
  | shared
  | mutable
  | unique

/--
  Cast kinds.
-/
inductive CastKind where
  | transmute
  | pointerCoerce (reify : Bool)

/--
  Binary operators.
-/
inductive BinOp where
  | add | sub | mul | div | rem | bitXor | bitAnd | bitOr | shl | shr
  | eq | ne | lt | le | gt | ge

/--
  Unary operators.
-/
inductive UnOp where
  | not | neg

/--
  Nullary operators (for constants).
-/
inductive NullOpKind where
  | sizeOf | alignOf | Prelude

/--
  A MIR basic block.
-/
structure BasicBlock where
  statements : List StatementKind
  terminator : TerminatorKind

/--
  A normalized MIR body.
-/
structure Body where
  locals : List Local
  blocks : List BasicBlock
  entry : Nat

end Chimera.RustAdapter.MIR
