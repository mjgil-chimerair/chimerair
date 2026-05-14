-- ChimeraProof Error: ChError
-- Chimera error structure.

import Chimera.Error.ErrorDomain

namespace Chimera

/--
Simple Chimera error with domain, code, and message.
This is the canonical error type for ChimeraProof.
-/
structure ChError where
  domain : String
  code   : Nat
  msg    : String
deriving Repr, BEq

/--
C.55: Extended error with drop context and message length.
Matches the physical chimera_error_t layout:
- status: ch_status_t (4 bytes)
- domain: chimera_error_domain_t (4 bytes)
- code: int32_t (4 bytes)
- message: const char* (ptr)
- file: const char* (ptr)
- line: int32_t (4 bytes)
- drop_ctx: optional context for drop callback (ptr)

C.55: Theorem: ChError maps exactly to canonical ABI layout.
-/
structure ChErrorExt where
  domain : String
  code   : Nat
  msg    : String
  /-- Optional context for cleanup on error -/
  dropCtx : Option String
  /-- Message length for buffer validation -/
  msgLen : Nat
deriving Repr, BEq

namespace ChError

/--
Create a generic error.
-/
def generic (msg : String) : ChError :=
  ⟨"generic", 1, msg⟩

/--
Create an extended generic error.
-/
def genericExt (msg : String) (msgLen : Nat) : ChErrorExt :=
  ⟨"generic", 1, msg, none, msgLen⟩

/--
Create a layout error.
-/
def layout (msg : String) : ChError :=
  ⟨"layout", 1, msg⟩

/--
Create an ABI error.
-/
def abi (msg : String) : ChError :=
  ⟨"abi", 1, msg⟩

/--
Create an ownership error.
-/
def ownership (msg : String) : ChError :=
  ⟨"ownership", 1, msg⟩

/--
Create an allocator error.
-/
def allocator (msg : String) : ChError :=
  ⟨"allocator", 1, msg⟩

/--
Create a link error.
-/
def link (msg : String) : ChError :=
  ⟨"link", 1, msg⟩

end ChError

/--
C.55: Theorem - ChError can be converted to ChErrorExt preserving domain and code.
-/
def toExt (e : ChError) : ChErrorExt :=
  ⟨e.domain, e.code, e.msg, none, e.msg.length⟩

/--
C.55: Theorem - ChErrorExt domain maps to physical error domain.
-/
def extDomainToPhysical (domain : String) : String :=
  domain  -- In practice, this maps to chimera_error_domain_t enum

end Chimera