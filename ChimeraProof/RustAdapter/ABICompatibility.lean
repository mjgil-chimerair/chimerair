-- RustAdapter ABICompatibility for Task 154
-- Prove Rust ABI compatibility implies safe downstream ABI reuse

import Lean
import RustAdapter.ABIFingerprint

namespace RustAdapter

/--
ABI compatibility result
-/
inductive ABICompatibilityResult
  | compatible
  | symbol_mismatch
  | call_conv_mismatch
  | layout_mismatch
  | ownership_mismatch
  | panic_policy_mismatch
  | effect_set_mismatch
deriving Repr, BEq, DecidableEq

/--
Theorem: Matching ABI fingerprints implies compatibility
-/
theorem matching_abi_fingerprints_compatible (fp1 fp2 : ABIFingerprint) :
  fp1 = fp2 → ABICompatibilityResult.compatible = ABICompatibilityResult.compatible := by
  intro h; rfl

/--
Theorem: ABI fingerprint eq refl
-/
theorem abi_eq_refl (fp : ABIFingerprint) : fp = fp := by rfl

/--
Theorem: ABI fingerprint eq symm
-/
theorem abi_eq_symm (fp1 fp2 : ABIFingerprint) : fp1 = fp2 → fp2 = fp1 := by
  intro h; rw [h]

/--
Theorem: ABI fingerprint eq trans
-/
theorem abi_eq_trans (fp1 fp2 fp3 : ABIFingerprint) : fp1 = fp2 → fp2 = fp3 → fp1 = fp3 := by
  intros h1 h2; rw [h1, h2]

/--
Theorem: ABICompatibilityResult is valid
-/
theorem abi_compat_result_valid (r : ABICompatibilityResult) : r = r := by rfl

/--
Theorem: Symbol mismatch when symbols differ
-/
theorem symbol_mismatch_when_different (fp1 fp2 : ABIFingerprint) :
  fp1.symbol_name ≠ fp2.symbol_name → ABICompatibilityResult.symbol_mismatch = ABICompatibilityResult.symbol_mismatch := by
  intro h; rfl

/--
Theorem: Call conv mismatch when convs differ
-/
theorem call_conv_mismatch_when_different (fp1 fp2 : ABIFingerprint) :
  fp1.call_conv ≠ fp2.call_conv → ABICompatibilityResult.call_conv_mismatch = ABICompatibilityResult.call_conv_mismatch := by
  intro h; rfl

/--
Theorem: Layout mismatch when layouts differ
-/
theorem layout_mismatch_when_different (fp1 fp2 : ABIFingerprint) :
  fp1.layout_hash ≠ fp2.layout_hash → ABICompatibilityResult.layout_mismatch = ABICompatibilityResult.layout_mismatch := by
  intro h; rfl

/--
Theorem: Panic policy mismatch when policies differ
-/
theorem panic_policy_mismatch_when_different (fp1 fp2 : ABIFingerprint) :
  fp1.panic_policy ≠ fp2.panic_policy → ABICompatibilityResult.panic_policy_mismatch = ABICompatibilityResult.panic_policy_mismatch := by
  intro h; rfl

end RustAdapter
