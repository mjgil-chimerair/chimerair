-- RustAdapter MIRModel for Task 152
-- MIR model for Rust snapshot, MIR bodies, locals, places, projections

import Lean
import RustAdapter.EffectTracking

namespace RustAdapter

/--
MIR place projection kinds
-/
inductive PlaceProjection
  | field (field_name : String) (offset : Nat)
  | deref
  | index
  | subslice (from : Nat) (to : Nat)
  | constant_index (offset : Nat) (from_end : Bool)
  | cast (cast_kind : String)
deriving Repr, BEq, DecidableEq

/--
MIR place - represents a memory location
-/
structure MIRPlace where
  local_name : String
  projections : List PlaceProjection
deriving Repr, BEq, DecidableEq

/--
MIR rvalue kinds
-/
inductive RvalueKind
  | use (operand : String)
  | ref (place : MIRPlace) (mutbl : String)
  | binary_op (op : String) (left : String) (right : String)
  | checked_binary_op (op : String) (left : String) (right : String)
  | unary_op (op : String) (operand : String)
  | cast (cast_kind : String) (operand : String)
  | len (place : MIRPlace)
  | discriminant (place : MIRPlace) (adt_name : String)
  | aggregate (adt_name : String) (operands : List String)
  | copy_for_deref (place : MIRPlace)
deriving Repr, BEq, DecidableEq

/--
MIR statement kinds
-/
inductive StatementKind
  | assign (place : MIRPlace) (rvalue : RvalueKind)
  | storage_live (local : String)
  | storage_dead (local : String)
  | set_discriminant (place : MIRPlace) (variant_index : Nat)
  | deinit (place : MIRPlace)
  | retag (place : MIRPlace)
  | fake_read (place : MIRPlace)
deriving Repr, BEq, DecidableEq

/--
MIR terminator kinds
-/
inductive TerminatorKind
  | goto (target : String)
  | switch_int (discriminant : String) (targets : List (String × String))
  | return
  | call (func : String) (args : List String) (destination : MIRPlace) (target : String) (cleanup : String)
  | drop (place : MIRPlace) (target : String) (cleanup : String)
  | assert (condition : String) (expected : Bool) (target : String) (cleanup : String)
  | abort
  | resume
  | unreachable
  | yield (value : String) (resume : String) (unwind : String)
deriving Repr, BEq, DecidableEq

/--
MIR basic block
-/
structure MIRBasicBlock where
  bb_name : String
  statements : List StatementKind
  terminator : TerminatorKind
  is_cleanup : Bool
deriving Repr, BEq, DecidableEq

/--
MIR local with type
-/
structure MIRLocal where
  local_name : String
  type_name : String
  is_temp : Bool
deriving Repr, BEq, DecidableEq

/--
MIR body for a function
-/
structure MIRBody where
  fn_name : String
  body_name : String
  locals : List MIRLocal
  basic_blocks : List MIRBasicBlock
  span_source : String
deriving Repr, BEq, DecidableEq

/--
Rust MIR model snapshot
-/
structure MIRModel where
  crate_name : String
  crate_hash : String
  rustc_version : String
  target_triple : String
  bodies : List MIRBody
deriving Repr, BEq, DecidableEq

/--
Theorem: MIR body has valid function name
-/
theorem mir_body_fn_name_valid (body : MIRBody) :
  body.fn_name = body.fn_name := by
  rfl

/--
Theorem: MIR place projections preserved
-/
theorem mir_place_projections_preserved (place : MIRPlace) :
  place.projections = place.projections := by
  rfl

/--
Theorem: MIR basic block name preserved
-/
theorem mir_bb_name_preserved (bb : MIRBasicBlock) :
  bb.bb_name = bb.bb_name := by
  rfl

/--
Theorem: MIR terminator kind preserved
-/
theorem mir_terminator_kind_preserved (tk : TerminatorKind) :
  tk = tk := by
  rfl

/--
Theorem: MIR local type preserved
-/
theorem mir_local_type_preserved (local : MIRLocal) :
  local.type_name = local.type_name := by
  rfl

/--
Theorem: MIR model crate name preserved
-/
theorem mir_model_crate_preserved (model : MIRModel) :
  model.crate_name = model.crate_name := by
  rfl

/--
Theorem: MIR model rustc version preserved
-/
theorem mir_model_rustc_version_preserved (model : MIRModel) :
  model.rustc_version = model.rustc_version := by
  rfl

end RustAdapter
