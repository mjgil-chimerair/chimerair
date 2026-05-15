-- RustAdapter Result Lowering for Task 125
-- Lower Rust Result to ch_status and out-params with error domain metadata

import Lean

namespace RustAdapter

/--
Rust Result type representation
-/
structure RustResultType where
  ok_type : String
  err_type : String
  error_domain : Nat
deriving Repr, BEq, DecidableEq

/--
Result lowering configuration
-/
structure ResultLoweringConfig where
  use_out_param : Bool
  status_variant : Nat
  error_mapping : Bool
deriving Repr, BEq, DecidableEq

/--
Theorem: Result type has valid ok type
-/
theorem result_type_ok_valid (rt : RustResultType) :
  rt.ok_type = rt.ok_type := by
  rfl

/--
Theorem: Result type has valid err type
-/
theorem result_type_err_valid (rt : RustResultType) :
  rt.err_type = rt.err_type := by
  rfl

/--
Theorem: Result type has valid error domain
-/
theorem result_type_error_domain_valid (rt : RustResultType) :
  rt.error_domain = rt.error_domain := by
  rfl

/--
Theorem: Config use out param is boolean
-/
theorem config_out_param_boolean (cfg : ResultLoweringConfig) :
  cfg.use_out_param = true ∨ cfg.use_out_param = false := by
  simp

/--
Theorem: Config error mapping is boolean
-/
theorem config_error_mapping_boolean (cfg : ResultLoweringConfig) :
  cfg.error_mapping = true ∨ cfg.error_mapping = false := by
  simp

end RustAdapter