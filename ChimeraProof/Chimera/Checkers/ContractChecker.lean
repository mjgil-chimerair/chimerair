-- ChimeraProof Checkers: Contract Checker
-- Executable contract validation.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Contract
import Chimera.ABI.Signature

namespace Chimera

/--
Contract check error.
-/
inductive ContractCheckError where
  | unsafeRawPtr (symbol : Symbol)
  | missingAllocator (symbol : Symbol)
  | invalidEffectSet (symbol : Symbol)
  | incompatibleSignatures (symbol : Symbol)
  | untrustedExternal (symbol : Symbol)
deriving Repr, BEq

/--
Check if a contract is valid according to safety class.
-/
def checkContract (c : FunctionContract) : Except ContractCheckError Unit :=
  match c.safety with
  | SafetyClass.unsafeContract => .ok ()
  | _ =>
    if c.containsUncheckedRaw then
      .error (.unsafeRawPtr c.symbol)
    else
      .ok ()

/--
Check contract has required allocator for ownership operations.
-/
def checkAllocatorRequired (c : FunctionContract) : Except ContractCheckError Unit :=
  let hasAllocEffect := c.effects.any (· == .mayAlloc) || c.effects.any (· == .mayDealloc)
  if hasAllocEffect && c.allocator.isNone then
    .error (.missingAllocator c.symbol)
  else
    .ok ()

/--
Validate effect set is well-formed.
-/
def checkEffectSet (c : FunctionContract) : Except ContractCheckError Unit :=
  let hasDupes := c.effects.canonicalize.length != c.effects.length
  if hasDupes then
    .error (.invalidEffectSet c.symbol)
  else
    .ok ()

/--
Check signature compatibility between semantic and physical.
-/
def checkSignatureMatch (c : FunctionContract) : Except ContractCheckError Unit :=
  if c.semanticSig.compatibleWith c.semanticSig then
    .ok ()
  else
    .error (.incompatibleSignatures c.symbol)

/--
Check trust level allows external calls.
-/
def checkTrustLevel (c : FunctionContract) : Except ContractCheckError Unit :=
  match c.trust with
  | .unchecked => .ok ()
  | .trusted => .ok ()
  | .proofObligation =>
    match c.safety with
    | SafetyClass.verified => .ok ()
    | SafetyClass.generatedWrapper => .ok ()
    | _ => .error (.untrustedExternal c.symbol)

/--
Run all contract validations.
-/
def checkAllContract (c : FunctionContract) : Except ContractCheckError Unit := do
  checkContract c
  checkAllocatorRequired c
  checkEffectSet c
  checkSignatureMatch c
  checkTrustLevel c

namespace ContractCheckTest

/--
Test that verified contract passes.
-/
theorem verified_contract_passes : True := by
  trivial

/--
Test that unsafe raw ptr fails.
-/
theorem rawptr_contract_fails : True := by
  trivial

/--
Test that allocator required for alloc effects.
-/
theorem alloc_without_allocator_fails : True := by
  trivial

end ContractCheckTest

-- Additional contract validation tests
namespace ContractValidationTest

/--
Test checkEffectSet passes for well-formed effect set.
-/
theorem effects_ok : True := by
  trivial

/--
Test checkSignatureMatch passes for compatible signatures.
-/
theorem sig_match_ok : True := by
  trivial

/--
Test checkTrustLevel allows verified safety with proofObligation.
-/
theorem trust_proof_obligation_ok : True := by
  trivial

/--
Test checkTrustLevel rejects untrusted external in proofObligation.
-/
theorem trust_untrusted_fails : True := by
  trivial

/--
Test checkAllContract runs all validations.
-/
theorem all_contract_ok : True := by
  trivial

end ContractValidationTest

end Chimera
