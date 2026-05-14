-- ChimeraProof Error: Error Domain
-- Error domain classification.

namespace Chimera

/--
Error domain - categorizes where an error originates.
-/
inductive ErrorDomain where
  | none          -- No error
  | chimeraCore   -- Chimera runtime itself
  | cErrno        -- C errno
  | rustResult    -- Rust Result<T, E>
  | rustPanic     -- Rust panic
  | zigErrorSet   -- Zig error union
  | zigPanic      -- Zig panic
  | user : Nat → ErrorDomain  -- User-defined domain
deriving Repr, BEq

namespace ErrorDomain

/--
Check if error domain represents a panic.
-/
def isPanic : ErrorDomain → Bool
  | .rustPanic => true
  | .zigPanic => true
  | _ => false

/--
Check if error domain represents a recoverable error.
-/
def isRecoverable : ErrorDomain → Bool
  | .cErrno => true
  | .rustResult => true
  | .zigErrorSet => true
  | .user _ => true
  | _ => false

/--
Get the domain name.
-/
def toString : ErrorDomain → String
  | .none => "none"
  | .chimeraCore => "chimera.core"
  | .cErrno => "c.errno"
  | .rustResult => "rust.result"
  | .rustPanic => "rust.panic"
  | .zigErrorSet => "zig.error"
  | .zigPanic => "zig.panic"
  | .user n => s!"user.{n}"

end ErrorDomain

end Chimera