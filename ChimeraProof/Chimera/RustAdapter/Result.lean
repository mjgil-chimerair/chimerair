--! Chimera.RustAdapter.Result
--!
--! Lean model for Result lowering proofs.

import Chimera.RustAdapter
import Chimera.ABI

namespace Chimera.RustAdapter.Result

/--
  Result lowering configuration.
  
  Describes how `Result<T, E>` is lowered to Chimera types.
-/
structure ResultLowering where
  okVariant : LoweredVariant
  errVariant : LoweredVariant
  discriminantField : String
  payloadField : String

/--
  Variant lowering.
-/
structure LoweredVariant where
  discriminantValue : Nat
  statusField : String
  statusValue : Nat

/--
  Result lowering proof.
  
  Proves that Rust `Result<T, E>` wrapper convention preserves
  Ok/Err distinction through `ch_status` and out params.
-/
structure ResultLoweringProof where
  inputType : String
  outputType : String
  okCase : ResultCase
  errCase : ResultCase

/--
  Result case (Ok or Err).
-/
structure ResultCase where
  isOk : Bool
  discriminant : Nat
  status : Nat
  payloadPreserved : Bool
  proof : String

/--
  Validate result lowering for primitive type.
-/
structure PrimitiveResultValidation where
  type : String
  lowering : ResultLowering
  okValue : Nat
  errValue : Nat

/--
  Validate result lowering for owned type.
-/
structure OwnedResultValidation where
  type : String
  lowering : ResultLowering
  allocatorCorrect : Bool
  dropCorrect : Bool

/--
  Validate result lowering for error payload.
-/
structure ErrorPayloadValidation where
  errorType : String
  lowering : ResultLowering
  errorDomainCorrect : Bool
  errorCodePreserved : Bool

end Chimera.RustAdapter.Result
