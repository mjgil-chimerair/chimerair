-- ChimeraProof IR: Operations
-- IR operations for ChimeraIR.

import Chimera.Foundation
import Chimera.ABI
import Chimera.Memory
import Chimera.IR.Module

namespace Chimera

/--
IR operation kinds.
-/
inductive OperationKind where
  | import
  | export
  | call
  | ownershipTransfer
  | borrow
  | drop
  | errorBridge
  | panicBridge
  | rawUnsafeCall
  | link
deriving Repr, BEq

/--
Source location for diagnostics.
-/
structure SourceLocation where
  file : String
  line : Nat
  column : Nat
deriving Repr, BEq

/--
Operand value representation.
-/
inductive OperandValue where
  | register (name : String)
  | immediate (val : Nat)
  | memory (addr : Pointer)
deriving Repr, BEq

/--
Operand (input or output of an operation).
-/
structure Operand where
  ty : ChType
  value : OperandValue
deriving Repr, BEq

/--
Operation with source location.
-/
structure Operation where
  kind : OperationKind
  inputs : List Operand
  outputs : List Operand
  location : Option SourceLocation
deriving Repr, BEq

namespace Operation

/--
Executable mirror of `SafeBoundaryType`.
-/
def safeBoundaryType : ChType → Bool
  | .rawptr _ => false
  | .ptr _ _ => false
  | .borrow t _ => safeBoundaryType t
  | .borrowMut t _ => safeBoundaryType t
  | .owned t => safeBoundaryType t
  | .out t => safeBoundaryType t
  | .inout t => safeBoundaryType t
  | .slice t _ => safeBoundaryType t
  | .str _ _ => true
  | .opaque _ => true
  | .result _ _ => false
  | .status => true
  | .error => true
  | .allocator => true
  | .unit | .bool | .i8 | .i16 | .i32 | .i64
    | .u8 | .u16 | .u32 | .u64 | .usize | .isize | .f32 | .f64 => true

/--
Check whether a type is an explicit borrow result.
-/
def isBorrowResultType : ChType → Bool
  | .borrow _ _ => true
  | .borrowMut _ _ => true
  | _ => false

/--
Check whether a type is a direct result type that must go through an error bridge.
-/
def isResultType : ChType → Bool
  | .result _ _ => true
  | _ => false

/--
Check whether a type is an explicit raw-pointer surface.
This treats both `rawptr` and `ptr` as unsafe surfaces that require the
`rawUnsafeCall` operation kind.
-/
def isRawSurfaceType : ChType → Bool
  | .ptr _ _ => true
  | ty => ty.containsRawPtr

/--
Check whether every output avoids escaping borrows.
-/
def outputsHaveNoEscapingBorrows (op : Operation) : Bool :=
  op.outputs.all (fun out => ! out.ty.containsEscapingBorrow)

/--
Check whether every output avoids direct `Result` values.
-/
def outputsAvoidDirectResult (op : Operation) : Bool :=
  op.outputs.all (fun out => ! isResultType out.ty)

/--
Check whether any operand uses an explicit raw surface.
-/
def hasRawSurface (op : Operation) : Bool :=
  (op.inputs ++ op.outputs).any (fun operand => isRawSurfaceType operand.ty)

/--
Check whether all operands are boundary-safe.
-/
def allOperandsBoundarySafe (op : Operation) : Bool :=
  (op.inputs ++ op.outputs).all (fun operand => safeBoundaryType operand.ty)

/--
Executable operation well-formedness checks.
-/
def isWellFormed (op : Operation) : Bool :=
  let defaultChecks := op.outputsHaveNoEscapingBorrows
  match op.kind with
  | .import =>
      defaultChecks
  | .export =>
      defaultChecks
  | .call =>
      defaultChecks &&
      ! op.inputs.isEmpty &&
      op.outputs.length ≤ 1 &&
      op.outputsAvoidDirectResult
  | .ownershipTransfer =>
      match op.inputs, op.outputs with
      | [input], [output] =>
          requiresDrop input.ty &&
          requiresDrop output.ty &&
          defaultChecks
      | _, _ => false
  | .borrow =>
      match op.inputs, op.outputs with
      | [_], [output] =>
          isBorrowResultType output.ty &&
          defaultChecks
      | _, _ => false
  | .drop =>
      match op.inputs, op.outputs with
      | [input], [] => requiresDrop input.ty
      | _, _ => false
  | .errorBridge =>
      match op.inputs, op.outputs with
      | [input], [output] =>
          ((isResultType input.ty && output.ty == .status) ||
            (input.ty == .status && isResultType output.ty)) &&
            defaultChecks
      | _, _ => false
  | .panicBridge =>
      defaultChecks &&
      op.outputs.length ≤ 1 &&
      allOperandsBoundarySafe op &&
      op.outputs.all (fun out => ! requiresDrop out.ty)
  | .rawUnsafeCall =>
      ! op.inputs.isEmpty &&
      op.outputs.length ≤ 1 &&
      hasRawSurface op
  | .link =>
      defaultChecks

/--
Get all types used in operation inputs.
-/
def inputTypes (op : Operation) : List ChType :=
  op.inputs.map (·.ty)

/--
Get all types produced by operation outputs.
-/
def outputTypes (op : Operation) : List ChType :=
  op.outputs.map (·.ty)

/--
Check if operation has location info.
-/
def hasLocation (op : Operation) : Bool :=
  op.location.isSome

end Operation

/--
IR module with operations.
-/
structure IRModule where
  module_ : Module
  operations : List Operation

namespace IRModule

/--
Empty IR module.
-/
def empty (m : Module) : IRModule := ⟨m, []⟩

/--
Add an operation to the IR module.
-/
def addOperation (irm : IRModule) (op : Operation) : IRModule :=
  { irm with operations := op :: irm.operations }

end IRModule

/--
Check if an operation is well-formed.
Validates that:
- All inputs and outputs have valid types
- Call operations have compatible arity
- Ownership operations reference valid blocks
- Unsafe boundaries are marked
-/
def WellFormedOperation (op : Operation) : Prop :=
  op.isWellFormed = true

/--
Theorem: empty inputs/outputs are well-formed.
-/
theorem empty_op_well_formed : WellFormedOperation ⟨.import, [], [], none⟩ := by
  rfl

end Chimera
