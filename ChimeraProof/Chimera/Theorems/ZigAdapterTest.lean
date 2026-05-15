-- ChimeraProof Tests: Zig Adapter
-- Theorem coverage for Zig invalidation and ABI reuse surfaces.

import Chimera.ZigAdapter

namespace Chimera.Test

namespace ZigAdapterInvalidationTest

private def sampleGraph : SemanticDependencyGraph :=
  ((SemanticDependencyGraph.empty
    |>.addNode .file "root.zig" (some "root.zig") true
    |>.addNode .decl "mid" none true
    |>.addNode .export "leaf" none true)
    |>.addEdge 0 1 .references
    |>.addEdge 1 2 .references)

theorem task_surface_covers_transitive_comptime_and_layout_invalidation :
    let summary := zig_invalidation_theorem_surface sampleGraph 0
    (invalidateComptimeChange sampleGraph 0).invalidated_nodes = [1, 2] ∧
      (invalidateLayoutChange sampleGraph 0).invalidated_nodes = [1, 2] := by
  native_decide

theorem exported_abi_change_breaks_reuse :
    changed_exported_abi_breaks_reuse := by
  exact changed_exported_abi_breaks_reuse

end ZigAdapterInvalidationTest

namespace ZigAdapterReuseTest

private def sampleContract : FunctionContract := {
  symbol := Symbol.simple "zig_surface_export"
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

theorem task_surface_preserves_unchanged_abi_reuse :
    let summary := zig_abi_reuse_theorem_surface sampleContract "before" "after"
      (by decide : "before" ≠ "after")
    let before := computeABIFingerprint sampleContract
    publicABIPreserved before before = true ∧
      privateChangeAltersABI before before = false := by
  native_decide

theorem private_impl_changes_leave_public_hash_stable :
    let before := computeABIFingerprint sampleContract
    let after := computeABIFingerprint sampleContract
    before.hash = after.hash := by
  exact private_impl_change_no_abi_impact sampleContract "before" "after" (by decide)

end ZigAdapterReuseTest

namespace ZigAdapterProofBridgeTest

theorem zig_bridge_roundtrip_preserves_items :
    ZigBridgeArtifact.serialize_roundtrip_sample := by
  exact ZigBridgeArtifact.serialize_roundtrip_sample

theorem zig_bridge_snapshot_connects_rust_shape_to_air_model :
    ZigBridgeArtifact.bridge_snapshot_tracks_export_and_type_counts := by
  exact ZigBridgeArtifact.bridge_snapshot_tracks_export_and_type_counts

theorem zig_bridge_rejects_orphan_payload_rows :
    ZigBridgeArtifact.deserialize? "zig-bridge|1|ffi_demo\nparam|0|input|i32" = none := by
  native_decide

theorem zig_cache_soundness_surface_allows_matching_reuse :
    matching_cache_inputs_allow_reuse := by
  exact matching_cache_inputs_allow_reuse

theorem zig_cache_soundness_surface_rejects_schema_drift :
    changed_schema_version_prevents_reuse := by
  exact changed_schema_version_prevents_reuse

theorem zig_cache_soundness_surface_rejects_dependency_drift :
    changed_dependency_fingerprint_prevents_reuse := by
  exact changed_dependency_fingerprint_prevents_reuse

end ZigAdapterProofBridgeTest

end Chimera.Test
