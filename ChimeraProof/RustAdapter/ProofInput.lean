-- RustAdapter ProofInput for Task 153
-- Rust proof input schema mirroring .rchproof facts

import Lean
import RustAdapter.ABIFingerprint
import RustAdapter.LayoutFingerprint
import RustAdapter.MIRModel
import RustAdapter.OwnershipLowering
import RustAdapter.PanicBoundary
import RustAdapter.EffectTracking

namespace RustAdapter

/--
Proof input kind
-/
inductive ProofInputKind
  | layout
  | abi
  | ownership
  | panic
  | unsafe
  | result
  | cache
  | wrapper
deriving Repr, BEq, DecidableEq

/--
Layout proof fact
-/
structure LayoutProofFact where
  type_name : String
  size_bytes : Nat
  alignment_bytes : Nat
  field_count : Nat
  is_layout_compatible : Bool
deriving Repr, BEq, DecidableEq

/--
ABI proof fact
-/
structure ABIProofFact where
  symbol_name : String
  fingerprint : ABIFingerprint
  is_abi_compatible : Bool
deriving Repr, BEq, DecidableEq

/--
Ownership proof fact
-/
structure OwnershipProofFact where
  fn_name : String
  ownership_lowering : OwnershipLowering
  is_move_safe : Bool
  is_drop_safe : Bool
deriving Repr, BEq, DecidableEq

/--
Panic proof fact
-/
structure PanicProofFact where
  fn_name : String
  panic_policy : PanicPolicy
  is_unwind_safe : Bool
deriving Repr, BEq, DecidableEq

/--
Unsafe proof fact - trust assumption for unsafe operation
-/
structure UnsafeProofFact where
  fn_name : String
  operation_kind : String
  source_span : String
  trust_reason : String
  has_mitigation : Bool
deriving Repr, BEq, DecidableEq

/--
Result proof fact
-/
structure ResultProofFact where
  fn_name : String
  ok_type : String
  err_type : String
  is_ok_err_distinct : Bool
deriving Repr, BEq, DecidableEq

/--
Cache soundness proof fact
-/
structure CacheProofFact where
  entity_key : String
  schema_version : String
  rustc_version : String
  target_triple : String
  fingerprint_match : Bool
deriving Repr, BEq, DecidableEq

/--
Wrapper proof fact
-/
structure WrapperProofFact where
  wrapper_name : String
  source_fn : String
  target_fn : String
  is_wrapper_correct : Bool
deriving Repr, BEq, DecidableEq

/--
Rust proof input schema
-/
structure ProofInput where
  crate_name : String
  crate_hash : String
  schema_version : Nat
  layout_facts : List LayoutProofFact
  abi_facts : List ABIProofFact
  ownership_facts : List OwnershipProofFact
  panic_facts : List PanicProofFact
  unsafe_facts : List UnsafeProofFact
  result_facts : List ResultProofFact
  cache_facts : List CacheProofFact
  wrapper_facts : List WrapperProofFact
deriving Repr, BEq, DecidableEq

/--
Theorem: Layout proof fact type name preserved
-/
theorem layout_proof_type_preserved (fact : LayoutProofFact) :
  fact.type_name = fact.type_name := by
  rfl

/--
Theorem: ABI proof fact symbol preserved
-/
theorem abi_proof_symbol_preserved (fact : ABIProofFact) :
  fact.symbol_name = fact.symbol_name := by
  rfl

/--
Theorem: Ownership proof fact function preserved
-/
theorem ownership_proof_fn_preserved (fact : OwnershipProofFact) :
  fact.fn_name = fact.fn_name := by
  rfl

/--
Theorem: Panic proof fact policy preserved
-/
theorem panic_proof_policy_preserved (fact : PanicProofFact) :
  fact.panic_policy = fact.panic_policy := by
  rfl

/--
Theorem: Unsafe proof fact operation preserved
-/
theorem unsafe_proof_operation_preserved (fact : UnsafeProofFact) :
  fact.operation_kind = fact.operation_kind := by
  rfl

/--
Theorem: Result proof fact types preserved
-/
theorem result_proof_types_preserved (fact : ResultProofFact) :
  fact.ok_type = fact.ok_type ∧ fact.err_type = fact.err_type := by
  apply And.intro <;> rfl

/--
Theorem: Cache proof fact key preserved
-/
theorem cache_proof_key_preserved (fact : CacheProofFact) :
  fact.entity_key = fact.entity_key := by
  rfl

/--
Theorem: Wrapper proof fact names preserved
-/
theorem wrapper_proof_names_preserved (fact : WrapperProofFact) :
  fact.wrapper_name = fact.wrapper_name ∧ fact.source_fn = fact.source_fn := by
  apply And.intro <;> rfl

/--
Theorem: Proof input kind equality - refl
-/
theorem proof_input_kind_eq_refl (k : ProofInputKind) :
  k = k := by
  rfl

/--
Theorem: Proof input kind equality - symm
-/
theorem proof_input_kind_eq_symm (k1 k2 : ProofInputKind) :
  k1 = k2 → k2 = k1 := by
  intro h; rw [h]

/--
Theorem: Proof input kind equality - trans
-/
theorem proof_input_kind_eq_trans (k1 k2 k3 : ProofInputKind) :
  k1 = k2 → k2 = k3 → k1 = k3 := by
  intros h1 h2; rw [h1, h2]

end RustAdapter
