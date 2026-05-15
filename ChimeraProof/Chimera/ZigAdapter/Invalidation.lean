-- ChimeraProof Zig Adapter: Invalidation Rules
-- Invalidation rules for Zig→ChimeraIR incremental compilation.

import Chimera.Foundation
import Chimera.ZigAdapter.DependencyGraph

namespace Chimera.ZigAdapter

/--
Invalidation trigger kinds.
-/
inductive InvalidationTrigger
  | source_edit
  | type_layout_change
  | exported_abi_change
  | comptime_value_change
  | embed_file_change
  | build_option_change
deriving Repr, BEq

/--
Invalidation result indicating what was invalidated.
-/
structure InvalidationResult where
  invalidated_nodes : List Nat
  downstream_modules : List String
  recompute_required : Bool

/--
Source edit invalidation: edit to a source file.
-/
def invalidateSourceEdit
  (graph : SemanticDependencyGraph)
  (file_id : Nat)
  (content_hash : String) : InvalidationResult :=
  let dep_ids := graph.reachableDependents file_id
  ⟨dep_ids, [], true⟩

/--
Type layout change invalidation: struct field changed offset/size.
-/
def invalidateLayoutChange
  (graph : SemanticDependencyGraph)
  (type_node_id : Nat) : InvalidationResult :=
  let dep_ids := graph.reachableDependents type_node_id
  ⟨dep_ids, [], true⟩

/--
Exported ABI change invalidation: function signature changed.
-/
def invalidateABIChange
  (graph : SemanticDependencyGraph)
  (export_node_id : Nat) : InvalidationResult :=
  let dep_ids := graph.reachableDependents export_node_id
  ⟨dep_ids, [], true⟩

/--
Comptime value change invalidation: compile-time constant changed.
-/
def invalidateComptimeChange
  (graph : SemanticDependencyGraph)
  (comptime_node_id : Nat) : InvalidationResult :=
  let dep_ids := graph.reachableDependents comptime_node_id
  ⟨dep_ids, [], true⟩

/--
Embed file change invalidation: @embedFile content changed.
-/
def invalidateEmbedFileChange
  (graph : SemanticDependencyGraph)
  (embed_node_id : Nat) : InvalidationResult :=
  let dep_ids := graph.reachableDependents embed_node_id
  ⟨dep_ids, [], true⟩

/--
Build option change invalidation: target/feature flags changed.
-/
def invalidateBuildOptionChange
  (graph : SemanticDependencyGraph)
  (option_name : String) : InvalidationResult :=
  -- All dependent nodes must be recomputed on build option change
  let all_ids := graph.nodes.map (·.id)
  ⟨all_ids, [], true⟩

/--
Check if private body change affects public ABI.
-/
def privateBodyChange
  (graph : SemanticDependencyGraph)
  (func_node_id : Nat) : Bool :=
  let node := graph.nodes.find? (fun n => n.id == func_node_id)
  match node with
  | some n => n.isPublic
  | none => false

/--
Check if public ABI change affects downstream.
-/
def publicABIChange
  (graph : SemanticDependencyGraph)
  (export_node_id : Nat) : Bool :=
  let outgoing := graph.getOutgoing export_node_id
  !outgoing.isEmpty

/--
Test: source edit invalidates dependent nodes.
-/
theorem source_edit_invalidates :
  let g := ((SemanticDependencyGraph.empty
    |>.addNode .file "a.zig" (some "a.zig") true
    |>.addNode .decl "A" none true
    |>.addNode .export "exportA" none true)
    |>.addEdge 0 1 .references
    |>.addEdge 1 2 .references)
  let result := invalidateSourceEdit g 0 "hash"
  result.invalidated_nodes = [1, 2] := by
  native_decide

/--
Test: layout change invalidates dependents.
-/
theorem layout_change_invalidates :
  let g := ((SemanticDependencyGraph.empty
    |>.addNode .type_node "TypeA" none true
    |>.addNode .layout_node "LayoutA" none true
    |>.addNode .export "exportA" none true)
    |>.addEdge 0 1 .specializes
    |>.addEdge 1 2 .references)
  let result := invalidateLayoutChange g 0
  result.invalidated_nodes = [1, 2] := by
  native_decide

/--
Test: build option change invalidates all nodes.
-/
theorem build_option_invalidates_all :
  let g := (SemanticDependencyGraph.empty
    |>.addNode .file "a.zig" (some "a.zig") true
    |>.addNode .export "exportA" none true)
  let result := invalidateBuildOptionChange g "target"
  result.invalidated_nodes = [1, 0] := by
  native_decide

/--
Test: private body change reports whether it affects public ABI.
-/
theorem private_body_not_public :
  let g := SemanticDependencyGraph.empty.addNode .function_body "body" none false
  privateBodyChange g 0 = false := by
  native_decide

/--
Test: public ABI change affects downstream.
-/
theorem public_abi_has_downstream :
  let g := ((SemanticDependencyGraph.empty
    |>.addNode .export "exportA" none true
    |>.addNode .link_artifact "libA" none true)
    |>.addEdge 0 1 .link_requires)
  publicABIChange g 0 = true := by
  native_decide

/--
Task 114 scenario: a private body change stays private and does not become a public ABI change.
-/
theorem private_body_change_preserves_public_reuse :
  let g := SemanticDependencyGraph.empty.addNode .function_body "helper_body" none false
  privateBodyChange g 0 = false := by
  native_decide

/--
Task 114 scenario: a public ABI change invalidates all required downstream nodes.
-/
theorem public_abi_change_invalidates_required_downstream :
  let g := ((SemanticDependencyGraph.empty
    |>.addNode .export "zig_export" none true
    |>.addNode .decl "wrapper_contract" none true
    |>.addNode .link_artifact "ffi_binary" none true)
    |>.addEdge 0 1 .references
    |>.addEdge 1 2 .link_requires)
  (invalidateABIChange g 0).invalidated_nodes = [1, 2] := by
  native_decide

/--
Task 114 scenario: a layout change invalidates dependent declarations and exports.
-/
theorem layout_change_invalidates_required_downstream :
  let g := ((SemanticDependencyGraph.empty
    |>.addNode .layout_node "PointLayout" none true
    |>.addNode .decl "PointMetadata" none true
    |>.addNode .export "point_export" none true)
    |>.addEdge 0 1 .references
    |>.addEdge 1 2 .references)
  (invalidateLayoutChange g 0).invalidated_nodes = [1, 2] := by
  native_decide

/--
Task 114 scenario: a comptime change invalidates the full reachable slice.
-/
theorem comptime_change_invalidates_required_downstream :
  let g := ((SemanticDependencyGraph.empty
    |>.addNode .comptime_call "compute_layout" none true
    |>.addNode .decl "PointDescriptor" none true
    |>.addNode .export "descriptor_export" none true)
    |>.addEdge 0 1 .call_depends
    |>.addEdge 1 2 .references)
  (invalidateComptimeChange g 0).invalidated_nodes = [1, 2] := by
  native_decide

/--
Task 114 scenario: an exported surface change invalidates downstream link consumers.
-/
theorem export_change_invalidates_required_downstream :
  let g := ((SemanticDependencyGraph.empty
    |>.addNode .export "zig_export" none true
    |>.addNode .link_artifact "wrapper_object" none true
    |>.addNode .link_artifact "final_binary" none true)
    |>.addEdge 0 1 .link_requires
    |>.addEdge 1 2 .link_requires)
  (invalidateABIChange g 0).invalidated_nodes = [1, 2] := by
  native_decide

/--
Task 114 summary theorem: the invalidation surface covers private-body reuse and
required downstream invalidation for ABI, layout, comptime, and export changes.
-/
theorem zig_invalidation_soundness_surface :
  (let g := SemanticDependencyGraph.empty.addNode .function_body "helper_body" none false
   privateBodyChange g 0 = false) ∧
    (let g := ((SemanticDependencyGraph.empty
      |>.addNode .export "zig_export" none true
      |>.addNode .decl "wrapper_contract" none true
      |>.addNode .link_artifact "ffi_binary" none true)
      |>.addEdge 0 1 .references
      |>.addEdge 1 2 .link_requires)
    (invalidateABIChange g 0).invalidated_nodes = [1, 2]) ∧
    (let g := ((SemanticDependencyGraph.empty
      |>.addNode .layout_node "PointLayout" none true
      |>.addNode .decl "PointMetadata" none true
      |>.addNode .export "point_export" none true)
      |>.addEdge 0 1 .references
      |>.addEdge 1 2 .references)
    (invalidateLayoutChange g 0).invalidated_nodes = [1, 2]) ∧
    (let g := ((SemanticDependencyGraph.empty
      |>.addNode .comptime_call "compute_layout" none true
      |>.addNode .decl "PointDescriptor" none true
      |>.addNode .export "descriptor_export" none true)
      |>.addEdge 0 1 .call_depends
      |>.addEdge 1 2 .references)
    (invalidateComptimeChange g 0).invalidated_nodes = [1, 2]) ∧
    (let g := ((SemanticDependencyGraph.empty
      |>.addNode .export "zig_export" none true
      |>.addNode .link_artifact "wrapper_object" none true
      |>.addNode .link_artifact "final_binary" none true)
      |>.addEdge 0 1 .link_requires
      |>.addEdge 1 2 .link_requires)
    (invalidateABIChange g 0).invalidated_nodes = [1, 2]) := by
  repeat' constructor <;> native_decide

end Chimera.ZigAdapter
