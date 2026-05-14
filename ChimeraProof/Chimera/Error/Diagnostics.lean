-- ChimeraProof Error: Diagnostics
-- Stable diagnostic codes and structured messages.

import Chimera.Foundation
import Chimera.Error

namespace Chimera

inductive DiagnosticSeverity
  | error
  | warning
  | info
  | hint
deriving Repr, BEq

inductive DiagnosticCode
  | MD_invalid_version
  | MD_missing_field
  | MD_duplicate_module
  | MD_invalid_language
  | MD_invalid_target
  | MD_invalid_import
  | MD_invalid_export
  | LY_zero_alignment
  | LY_invalid_width
  | LY_zero_array
  | LY_padding_overflow
  | LY_field_missing
  | LY_field_offset_mismatch
  | LY_field_size_mismatch
  | LY_struct_size_mismatch
  | OW_double_ownership
  | OW_write_alias
  | OW_borrow_exclusive
  | OW_lifetime_escape
  | AL_duplicate_allocator
  | AL_mismatched_drop
  | AL_missing_drop
  | AL_invalid_id
  | CT_invalid_signature
  | CT_layout_mismatch
  | CT_result_lowering
  | CT_allocator_mismatch
  | CT_drop_mismatch
  | CT_panic_violation
  | CT_effects_mismatch
  | CT_trust_violation
  | RS_zero_ok_status
  | RS_missing_error
  | RS_false_ok
  | RS_err_payload_missing
  | PN_boundary_violation
  | PN_unwind_detected
  | PN_abort_mismatch
  | LK_duplicate_strong_symbol
  | LK_unresolved_import
  | LK_target_mismatch
  | LK_signature_incompatible
  | LK_calling_convention_mismatch
  | EF_inferred_not_declared
  | EF_duplicate_effect
  | EF_invalid_effect_set
deriving Repr, BEq

structure Diagnostic where
  code : DiagnosticCode
  severity : DiagnosticSeverity
  message : String
  location : Option String
  notes : List String
deriving Repr, BEq

namespace DiagnosticCode

def code_number (c : DiagnosticCode) : Nat :=
  match c with
  | .MD_invalid_version => 1001
  | .MD_missing_field => 1002
  | .MD_duplicate_module => 1003
  | .MD_invalid_language => 1004
  | .MD_invalid_target => 1005
  | .MD_invalid_import => 1006
  | .MD_invalid_export => 1007
  | .LY_zero_alignment => 2001
  | .LY_invalid_width => 2002
  | .LY_zero_array => 2003
  | .LY_padding_overflow => 2004
  | .LY_field_missing => 2005
  | .LY_field_offset_mismatch => 2006
  | .LY_field_size_mismatch => 2007
  | .LY_struct_size_mismatch => 2008
  | .OW_double_ownership => 3001
  | .OW_write_alias => 3002
  | .OW_borrow_exclusive => 3003
  | .OW_lifetime_escape => 3004
  | .AL_duplicate_allocator => 4001
  | .AL_mismatched_drop => 4002
  | .AL_missing_drop => 4003
  | .AL_invalid_id => 4004
  | .CT_invalid_signature => 5001
  | .CT_layout_mismatch => 5002
  | .CT_result_lowering => 5003
  | .CT_allocator_mismatch => 5004
  | .CT_drop_mismatch => 5005
  | .CT_panic_violation => 5006
  | .CT_effects_mismatch => 5007
  | .CT_trust_violation => 5008
  | .RS_zero_ok_status => 6001
  | .RS_missing_error => 6002
  | .RS_false_ok => 6003
  | .RS_err_payload_missing => 6004
  | .PN_boundary_violation => 7001
  | .PN_unwind_detected => 7002
  | .PN_abort_mismatch => 7003
  | .LK_duplicate_strong_symbol => 8001
  | .LK_unresolved_import => 8002
  | .LK_target_mismatch => 8003
  | .LK_signature_incompatible => 8004
  | .LK_calling_convention_mismatch => 8005
  | .EF_inferred_not_declared => 9001
  | .EF_duplicate_effect => 9002
  | .EF_invalid_effect_set => 9003

def description (c : DiagnosticCode) : String :=
  match c with
  | .MD_invalid_version => "invalid ABI version"
  | .MD_missing_field => "required metadata field missing"
  | .MD_duplicate_module => "duplicate module name"
  | .MD_invalid_language => "unsupported language"
  | .MD_invalid_target => "incompatible target"
  | .MD_invalid_import => "invalid import entry"
  | .MD_invalid_export => "invalid export entry"
  | .LY_zero_alignment => "alignment must be positive power of two"
  | .LY_invalid_width => "invalid integer or float width"
  | .LY_zero_array => "array element count must be non-zero"
  | .LY_padding_overflow => "padding exceeds struct size"
  | .LY_field_missing => "required struct field missing"
  | .LY_field_offset_mismatch => "field offset does not match layout"
  | .LY_field_size_mismatch => "field size does not match declaration"
  | .LY_struct_size_mismatch => "struct size does not match declaration"
  | .OW_double_ownership => "block has multiple owning references"
  | .OW_write_alias => "mutable borrow aliases with another borrow"
  | .OW_borrow_exclusive => "mutable borrow exclusivity violated"
  | .OW_lifetime_escape => "lifetime extends beyond valid scope"
  | .AL_duplicate_allocator => "allocator id already registered"
  | .AL_mismatched_drop => "allocator and drop function mismatch"
  | .AL_missing_drop => "no drop function for owned type"
  | .AL_invalid_id => "invalid allocator identifier"
  | .CT_invalid_signature => "function signature validation failed"
  | .CT_layout_mismatch => "contract layout not matching ABI"
  | .CT_result_lowering => "Result type lowering invalid"
  | .CT_allocator_mismatch => "allocator mismatch in contract"
  | .CT_drop_mismatch => "drop function mismatch in contract"
  | .CT_panic_violation => "panic policy violation detected"
  | .CT_effects_mismatch => "declared effects do not match inferred"
  | .CT_trust_violation => "trust policy violation"
  | .RS_zero_ok_status => "success status must be non-zero"
  | .RS_missing_error => "error domain not specified"
  | .RS_false_ok => "false positive success status"
  | .RS_err_payload_missing => "error status missing payload"
  | .PN_boundary_violation => "panic boundary crossing detected"
  | .PN_unwind_detected => "unwind across safe boundary"
  | .PN_abort_mismatch => "abort policy mismatch"
  | .LK_duplicate_strong_symbol => "duplicate strong symbol definition"
  | .LK_unresolved_import => "import symbol not resolved"
  | .LK_target_mismatch => "target incompatible across modules"
  | .LK_signature_incompatible => "import/export signature mismatch"
  | .LK_calling_convention_mismatch => "calling convention mismatch"
  | .EF_inferred_not_declared => "inferred effect not declared"
  | .EF_duplicate_effect => "duplicate effect in set"
  | .EF_invalid_effect_set => "effect set contains invalid effect"

end DiagnosticCode

namespace Diagnostic

def error (code : DiagnosticCode) (message : String) : Diagnostic :=
  { code := code, severity := .error, message := message, location := none, notes := [] }

def warning (code : DiagnosticCode) (message : String) : Diagnostic :=
  { code := code, severity := .warning, message := message, location := none, notes := [] }

def withLocation (d : Diagnostic) (loc : String) : Diagnostic :=
  { d with location := some loc }

def withNote (d : Diagnostic) (note : String) : Diagnostic :=
  { d with notes := d.notes ++ [note] }

def format (d : Diagnostic) : String :=
  let locStr :=
    match d.location with
    | some l => l ++ ": "
    | none => ""
  let sevStr :=
    match d.severity with
    | .error => "error"
    | .warning => "warning"
    | .info => "info"
    | .hint => "hint"
  let codeStr := "[" ++ toString (DiagnosticCode.code_number d.code) ++ "]"
  let notesStr :=
    if d.notes.isEmpty then
      ""
    else
      "\n" ++ String.intercalate "\n" (d.notes.map (fun n => "note: " ++ n))
  locStr ++ sevStr ++ " " ++ codeStr ++ ": " ++ d.message ++ notesStr

end Diagnostic

end Chimera
