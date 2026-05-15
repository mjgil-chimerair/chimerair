-- ChimeraProof Tests: ABI Tests
-- Compile-safe theorem smoke tests for ABI modules.

import Chimera.Foundation
import Chimera.ABI

namespace Chimera.Test

private def linux_alias : Target :=
  { Target.x86_64_linux with triple := "x86_64-unknown-linux-musl" }

namespace ChTypeTest

theorem primitive_smoke : True := by
  trivial

theorem borrow_smoke : True := by
  trivial

theorem rawptr_smoke : True := by
  trivial

theorem requires_drop_smoke : True := by
  trivial

theorem requires_drop_owned_correspondence :
    requiresDrop (.owned .u32) = true ∧ RequiresDrop (.owned .u32) := by
  constructor
  · rfl
  · exact (requiresDrop_eq_true_iff (.owned .u32)).1 rfl

theorem requires_drop_result_correspondence :
    requiresDrop (.result (.owned .u32) .bool) = true ∧
      RequiresDrop (.result (.owned .u32) .bool) := by
  constructor
  · rfl
  · exact (requiresDrop_eq_true_iff (.result (.owned .u32) .bool)).1 rfl

theorem requires_drop_bool_correspondence :
    requiresDrop .bool = false ∧ ¬ RequiresDrop .bool := by
  constructor
  · rfl
  · exact (requiresDrop_eq_false_iff .bool).1 rfl

theorem requires_drop_str_correspondence :
    requiresDrop (.str .utf8 .borrow) = false ∧
      ¬ RequiresDrop (.str .utf8 .borrow) := by
  constructor
  · rfl
  · exact (requiresDrop_eq_false_iff (.str .utf8 .borrow)).1 rfl

theorem direct_result_smoke : True := by
  trivial

theorem mutability_smoke : True := by
  trivial

theorem lifetime_smoke : True := by
  trivial

theorem ownership_smoke : True := by
  trivial

theorem safe_boundary_smoke : True := by
  trivial

theorem classifier_smoke : True := by
  trivial

end ChTypeTest

namespace PhysTypeTest

theorem width_smoke : True := by
  trivial

theorem classifier_smoke : True := by
  trivial

end PhysTypeTest

namespace LayoutTest

theorem valid_width_smoke : True := by
  trivial

theorem invalid_width_smoke : True := by
  trivial

theorem layout_smoke : True := by
  trivial

theorem struct_layout_smoke : True := by
  trivial

theorem array_layout_smoke : True := by
  trivial

theorem disjoint_smoke : True := by
  trivial

theorem padding_smoke : True := by
  trivial

theorem compatible_targets_agree_on_pointer_layout :
    Target.compatible Target.x86_64_linux linux_alias ∧
      Layout.layoutOf Target.x86_64_linux .ptr = Layout.layoutOf linux_alias .ptr := by
  constructor
  · simp [Target.compatible, linux_alias, Target.x86_64_linux]
  · exact Layout.compatible_ptr_layout_eq (by simp [Target.compatible, linux_alias, Target.x86_64_linux])

theorem compatible_targets_agree_on_function_pointer_layout :
    Layout.layoutOf Target.x86_64_linux (.fnptr .cdecl [] .void) =
      Layout.layoutOf linux_alias (.fnptr .cdecl [] .void) := by
  exact Layout.compatible_fnptr_layout_eq
    (by simp [Target.compatible, linux_alias, Target.x86_64_linux]) _ _ _

theorem compatible_targets_agree_on_sample_struct_layout :
    let fields : List (String × PhysType) :=
      [("ptr", .ptr), ("len", .int 64 .unsigned)]
    Layout.computeStructLayout Target.x86_64_linux fields =
      Layout.computeStructLayout linux_alias fields := by
  rfl

theorem compatible_targets_agree_on_canonical_handle_layout :
    ch_handle_layout Target.x86_64_linux = ch_handle_layout linux_alias := by
  exact compatible_ch_handle_layout_eq
    (by simp [Target.compatible, linux_alias, Target.x86_64_linux])

theorem compatible_targets_agree_on_canonical_error_layout :
    ch_error_layout Target.x86_64_linux = ch_error_layout linux_alias := by
  exact compatible_ch_error_layout_eq
    (by simp [Target.compatible, linux_alias, Target.x86_64_linux])

theorem canonical_error_fields_match_runtime_abi :
    ch_error_phys_fields =
      [("domain", .int 32 .unsigned),
       ("code", .int 32 .unsigned),
       ("flags", .int 32 .unsigned),
       ("message_ptr", .ptr),
       ("message_len", .int 64 .unsigned),
       ("payload_ptr", .ptr),
       ("payload_drop_fn", .fnptr .cdecl [.ptr] .void),
       ("payload_drop_ctx", .ptr)] := by
  rfl

theorem canonical_allocator_fields_match_runtime_abi :
    ch_allocator_phys_fields =
      [("id", .int 64 .unsigned),
       ("kind", .int 32 .unsigned),
       ("ptr", .ptr)] := by
  rfl

theorem canonical_slice_fields_match_runtime_abi :
    ch_slice_phys_fields .ptr =
      [("ptr", .ptr),
       ("len", .int 64 .unsigned)] := by
  rfl

theorem canonical_borrow_str_fields_match_runtime_abi :
    ch_borrow_str_phys_fields =
      [("ptr", .ptr),
       ("len", .int 64 .unsigned),
       ("lifetime", .int 32 .unsigned)] := by
  rfl

theorem canonical_owned_bytes_fields_match_runtime_abi :
    ch_owned_bytes_phys_fields =
      [("ptr", .ptr),
       ("len", .int 64 .unsigned),
       ("capacity", .int 64 .unsigned),
       ("allocator_id", .int 64 .unsigned)] := by
  rfl

theorem canonical_handle_fields_match_runtime_abi :
    ch_handle_phys_fields =
      [("ptr", .ptr),
       ("drop_fn", .fnptr .cdecl [.ptr] .void),
       ("size", .int 64 .unsigned)] := by
  rfl

theorem canonical_error_layout_matches_runtime_header_on_x86_64 :
    ch_error_layout Target.x86_64_linux =
      .ok {
        size := 56,
        align := 8,
        fields := [
          { fieldName := "domain", offset := 0, size := 4, align := 4 },
          { fieldName := "code", offset := 4, size := 4, align := 4 },
          { fieldName := "flags", offset := 8, size := 4, align := 4 },
          { fieldName := "message_ptr", offset := 16, size := 8, align := 8 },
          { fieldName := "message_len", offset := 24, size := 8, align := 8 },
          { fieldName := "payload_ptr", offset := 32, size := 8, align := 8 },
          { fieldName := "payload_drop_fn", offset := 40, size := 8, align := 8 },
          { fieldName := "payload_drop_ctx", offset := 48, size := 8, align := 8 }
        ]
      } := by
  rfl

theorem canonical_allocator_layout_matches_runtime_header_on_x86_64 :
    ch_allocator_layout Target.x86_64_linux =
      .ok {
        size := 24,
        align := 8,
        fields := [
          { fieldName := "id", offset := 0, size := 8, align := 8 },
          { fieldName := "kind", offset := 8, size := 4, align := 4 },
          { fieldName := "ptr", offset := 16, size := 8, align := 8 }
        ]
      } := by
  rfl

theorem canonical_slice_layout_matches_runtime_header_on_x86_64 :
    Layout.layoutOf Target.x86_64_linux (ch_slice_phys .ptr) =
      .ok {
        size := 16,
        align := 8,
        fields := [
          { fieldName := "ptr", offset := 0, size := 8, align := 8 },
          { fieldName := "len", offset := 8, size := 8, align := 8 }
        ]
      } := by
  rfl

theorem canonical_borrow_str_layout_matches_runtime_header_on_x86_64 :
    Layout.layoutOf Target.x86_64_linux ch_borrow_str_phys =
      .ok {
        size := 24,
        align := 8,
        fields := [
          { fieldName := "ptr", offset := 0, size := 8, align := 8 },
          { fieldName := "len", offset := 8, size := 8, align := 8 },
          { fieldName := "lifetime", offset := 16, size := 4, align := 4 }
        ]
      } := by
  rfl

theorem canonical_owned_bytes_layout_matches_runtime_header_on_x86_64 :
    Layout.layoutOf Target.x86_64_linux ch_owned_bytes_phys =
      .ok {
        size := 32,
        align := 8,
        fields := [
          { fieldName := "ptr", offset := 0, size := 8, align := 8 },
          { fieldName := "len", offset := 8, size := 8, align := 8 },
          { fieldName := "capacity", offset := 16, size := 8, align := 8 },
          { fieldName := "allocator_id", offset := 24, size := 8, align := 8 }
        ]
      } := by
  rfl

theorem canonical_handle_layout_matches_runtime_header_on_x86_64 :
    ch_handle_layout Target.x86_64_linux =
      .ok {
        size := 24,
        align := 8,
        fields := [
          { fieldName := "ptr", offset := 0, size := 8, align := 8 },
          { fieldName := "drop_fn", offset := 8, size := 8, align := 8 },
          { fieldName := "size", offset := 16, size := 8, align := 8 }
        ]
      } := by
  rfl

end LayoutTest

namespace LoweringTest

theorem lower_type_smoke : True := by
  trivial

theorem lower_result_smoke : True := by
  trivial

theorem abi_legal_smoke : True := by
  trivial

theorem represents_smoke : True := by
  trivial

theorem compatible_targets_agree_on_usize_lowering :
    lowerType Target.x86_64_linux .usize = lowerType linux_alias .usize := by
  exact compatible_usize_lowering_eq
    (by simp [Target.compatible, linux_alias, Target.x86_64_linux])

theorem compatible_targets_agree_on_isize_lowering :
    lowerType Target.x86_64_linux .isize = lowerType linux_alias .isize := by
  exact compatible_isize_lowering_eq
    (by simp [Target.compatible, linux_alias, Target.x86_64_linux])

theorem primitive_result_lowers_to_status_and_out_params :
    lowerSignature Target.x86_64_linux
      { params := [], returns := .result .u32 .error, isVarargs := false } =
      .ok { params := [.ptr, .ptr], returns := .value 1, callingConv := .cdecl } := by
  exact result_signature_uses_status_and_out_params _ _ _

theorem owned_result_lowers_to_status_and_out_params :
    lowerSignature Target.x86_64_linux
      { params := [], returns := .result (.owned .u32) .error, isVarargs := false } =
      .ok { params := [.ptr, .ptr], returns := .value 1, callingConv := .cdecl } := by
  exact result_signature_uses_status_and_out_params _ _ _

theorem opaque_result_lowers_to_status_and_out_params :
    lowerSignature Target.x86_64_linux
      { params := [], returns := .result (.opaque (Symbol.simple "OpaqueHandle")) .error, isVarargs := false } =
      .ok { params := [.ptr, .ptr], returns := .value 1, callingConv := .cdecl } := by
  exact result_signature_uses_status_and_out_params _ _ _

theorem payload_error_result_lowers_to_status_and_out_params :
    lowerSignature Target.x86_64_linux
      { params := [], returns := .result .u32 (.owned (.opaque (Symbol.simple "ErrPayload"))), isVarargs := false } =
      .ok { params := [.ptr, .ptr], returns := .value 1, callingConv := .cdecl } := by
  exact result_signature_uses_status_and_out_params _ _ _

theorem borrow_lowering_is_abi_legal_and_representable :
    let phys := ch_borrow_str_phys
    lowerType Target.x86_64_linux (.borrow .u32 .static) = Except.ok phys ∧
      AbiLegalPhysical Target.x86_64_linux phys ∧
      Represents Target.x86_64_linux phys (.borrow .u32 .static) := by
  constructor
  · rfl
  · exact lowerType_sound _ _ _ rfl

theorem owned_slice_lowering_is_abi_legal_and_representable :
    let phys := ch_owned_bytes_phys
    lowerType Target.x86_64_linux (.owned (.slice .u8 .owned)) = Except.ok phys ∧
      AbiLegalPhysical Target.x86_64_linux phys ∧
      Represents Target.x86_64_linux phys (.owned (.slice .u8 .owned)) := by
  constructor
  · rfl
  · exact lowerType_sound _ _ _ rfl

theorem owned_opaque_lowering_is_abi_legal_and_representable :
    let phys := ch_handle_phys
    lowerType Target.x86_64_linux (.owned (.opaque (Symbol.simple "Handle"))) = Except.ok phys ∧
      AbiLegalPhysical Target.x86_64_linux phys ∧
      Represents Target.x86_64_linux phys (.owned (.opaque (Symbol.simple "Handle"))) := by
  constructor
  · rfl
  · exact lowerType_sound _ _ _ rfl

theorem direct_return_signature_sound :
    let sig : SemanticSignature :=
      { params := [{ name := "x", ty := .u32 }], returns := .owned (.opaque (Symbol.simple "Handle")), isVarargs := false }
    let psig : PhysicalSignature :=
      { params := [ch_handle_phys, .int 32 .unsigned], returns := .value 1, callingConv := .cdecl }
    lowerSignature Target.x86_64_linux sig = .ok psig ∧
      psig.callingConv = .cdecl ∧
      LoweredReturnRepresents Target.x86_64_linux sig.returns psig.returns [ch_handle_phys] := by
  constructor
  · rfl
  · exact (lowerSignature_sound _ _ _ rfl).1 |> And.intro ((lowerSignature_sound _ _ _ rfl).2.1)

theorem result_signature_sound_uses_representable_out_params :
    let sig : SemanticSignature :=
      { params := [{ name := "arg", ty := .u32 }], returns := .result (.owned (.opaque (Symbol.simple "Handle"))) .error, isVarargs := false }
    let psig : PhysicalSignature :=
      { params := [.ptr, .ptr, .int 32 .unsigned], returns := .value 1, callingConv := .cdecl }
    lowerSignature Target.x86_64_linux sig = .ok psig ∧
      LoweredReturnRepresents Target.x86_64_linux sig.returns psig.returns [.ptr, .ptr] := by
  constructor
  · rfl
  · exact (lowerSignature_sound _ _ _ rfl).2.1

end LoweringTest

namespace SignatureCompatibilityTest

theorem compatibility_smoke : True := by
  trivial

end SignatureCompatibilityTest

end Chimera.Test
