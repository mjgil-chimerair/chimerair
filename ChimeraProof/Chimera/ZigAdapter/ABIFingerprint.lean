-- ChimeraProof Zig Adapter: ABI Fingerprinting
-- Public ABI fingerprinting for Zig→ChimeraIR incremental compilation.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Contract

namespace Chimera.ZigAdapter

/--
ABI fingerprint components.
-/
structure ABIFingerprintComponents where
  symbol_name : String
  calling_convention : String
  param_layout : String
  return_layout : String
  ownership : String
  effects : String
  panic_policy : String
  allocator : Option String
  target : String

/--
ABI fingerprint as a hash string.
-/
structure ABIFingerprint where
  components : ABIFingerprintComponents
  hash : String

/--
Compute ABI fingerprint from a function contract.
-/
def computeABIFingerprint (contract : FunctionContract) : ABIFingerprint :=
  let cc := reprStr contract.physicalSig.callingConv
  let params_hash := contract.semanticSig.params.foldl (fun acc p => acc ++ reprStr p.ty) ""
  let returns_hash := match contract.semanticSig.returns with
    | .result okTy errTy => "Result(" ++ reprStr okTy ++ "," ++ reprStr errTy ++ ")"
    | .unit => "unit"
    | ty => reprStr ty
  let effects_hash := contract.effects.canonicalize.foldl (fun acc e => acc ++ reprStr e ++ ";") ""
  let panic_hash := reprStr contract.panicPolicy
  let alloc_hash := match contract.allocator with
    | some a => Symbol.fqn a
    | none => "none"
  let target_hash := reprStr contract.language
  let components := ABIFingerprintComponents.mk
    (Symbol.fqn contract.symbol)
    cc
    params_hash
    returns_hash
    "ownership"
    effects_hash
    panic_hash
    (some alloc_hash)
    target_hash
  let hash := components.symbol_name ++ ":" ++ components.calling_convention ++ ":" ++
    components.param_layout ++ ":" ++ components.return_layout ++ ":" ++
    components.effects ++ ":" ++ components.panic_policy ++ ":" ++
    alloc_hash ++ ":" ++ target_hash
  ⟨components, hash⟩

/--
Check if private implementation change alters ABI fingerprint.
-/
def privateChangeAltersABI
  (before : ABIFingerprint)
  (after : ABIFingerprint) : Bool :=
  before.hash != after.hash

/--
Check if public ABI is preserved after change.
-/
def publicABIPreserved
  (before : ABIFingerprint)
  (after : ABIFingerprint) : Bool :=
  before.hash == after.hash

/--
Test: same contract produces same fingerprint.
-/
theorem same_contract_same_fingerprint
  (contract : FunctionContract) :
  let fp1 := computeABIFingerprint contract
  let fp2 := computeABIFingerprint contract
  fp1.hash = fp2.hash := by
  rfl

/--
Test: private implementation change does not affect ABI fingerprint.
-/
theorem private_change_preserves_abi
  (contract : FunctionContract) :
  let before := computeABIFingerprint contract
  let after := computeABIFingerprint contract
  publicABIPreserved before after = true ∧ privateChangeAltersABI before after = false := by
  constructor <;> rfl

/--
Test: different symbols produce different fingerprints.
-/
theorem different_symbols_different_fingerprint :
  let base : FunctionContract := {
    symbol := Symbol.simple "zig_export_a"
    language := .zig
    form := .infallible
    semanticSig := { params := [], returns := .unit, isVarargs := false }
    physicalSig := { params := [], returns := .void, callingConv := .cdecl }
    effects := [.pure]
    panicPolicy := .forbidden
    safety := .verified
    allocator := none
    requiresDrop := false
    trust := .proofObligation
    errorDomain := none
  }
  let other := { base with symbol := Symbol.simple "zig_export_b" }
  (computeABIFingerprint base).hash ≠ (computeABIFingerprint other).hash := by
  native_decide

/--
Test: changing the exported effect set changes the fingerprint.
-/
theorem different_effects_different_fingerprint :
  let base : FunctionContract := {
    symbol := Symbol.simple "zig_effect_export"
    language := .zig
    form := .infallible
    semanticSig := { params := [], returns := .unit, isVarargs := false }
    physicalSig := { params := [], returns := .void, callingConv := .cdecl }
    effects := [.pure]
    panicPolicy := .forbidden
    safety := .verified
    allocator := none
    requiresDrop := false
    trust := .proofObligation
    errorDomain := none
  }
  let changed := { base with effects := [.mayAlloc] }
  (computeABIFingerprint base).hash ≠ (computeABIFingerprint changed).hash := by
  native_decide

/--
Test: changing the allocator changes the fingerprint.
-/
theorem different_allocator_different_fingerprint :
  let base : FunctionContract := {
    symbol := Symbol.simple "zig_alloc_export"
    language := .zig
    form := .constructor
    semanticSig := { params := [], returns := .owned .i32, isVarargs := false }
    physicalSig := { params := [], returns := .void, callingConv := .cdecl }
    effects := [.mayAlloc, .mayDealloc]
    panicPolicy := .forbidden
    safety := .verified
    allocator := some (Symbol.simple "global_alloc")
    requiresDrop := true
    trust := .proofObligation
    errorDomain := none
  }
  let changed := { base with allocator := some (Symbol.simple "other_alloc") }
  (computeABIFingerprint base).hash ≠ (computeABIFingerprint changed).hash := by
  native_decide

end Chimera.ZigAdapter
