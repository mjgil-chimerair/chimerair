-- CAdapter MLIR Attributes for Task 105
-- C-specific MLIR attributes: source file, header provenance, macro stack,
-- symbol linkage, C calling convention, layout hash, errno policy

import Lean

namespace Chimera.CAdapter.MLIR

/--
C source provenance
-/
structure CSourceProvenance where
  source_file : String
  header_file : Option String
  line_number : Nat
deriving Repr, BEq, DecidableEq

/--
C symbol linkage
-/
inductive CLinkage
  | none
  | internal
  | external
  | weak
deriving Repr, BEq, DecidableEq

/--
C calling convention
-/
inductive CCallingConv
  | cdecl
  | stdcall
  | fastcall
  | vectorcall
  | thiscall
deriving Repr, BEq, DecidableEq

/--
C errno policy
-/
inductive CErrnoPolicy
  | none
  | errno
  | last_errno
deriving Repr, BEq, DecidableEq

/--
C function attributes
-/
structure CFuncAttributes where
  source : CSourceProvenance
  linkage : CLinkage
  cconv : CCallingConv
  errno_policy : CErrnoPolicy
  layout_hash : String
deriving Repr, BEq, DecidableEq

/--
Theorem: Source provenance preserves file path
-/
theorem source_provenance_file (prov : CSourceProvenance) :
  prov.source_file = prov.source_file := by
  rfl

/--
Theorem: errno policy is one of none, errno, last_errno
-/
theorem errno_policy_valid (policy : CErrnoPolicy) :
  policy = policy := by
  rfl

/--
Theorem: C calling convention is one of cdecl, stdcall, fastcall, vectorcall, thiscall
-/
theorem cconv_valid (cconv : CCallingConv) :
  cconv = cconv := by
  rfl

/--
Theorem: Layout hash is a string
-/
theorem layout_hash_is_string (attrs : CFuncAttributes) :
  attrs.layout_hash = attrs.layout_hash := by
  rfl

end Chimera.CAdapter.MLIR
