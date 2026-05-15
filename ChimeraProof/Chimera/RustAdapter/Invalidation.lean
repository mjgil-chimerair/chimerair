--! Chimera.RustAdapter.Invalidation
--!
--! Lean model for Rust invalidation engine.

import Chimera.RustAdapter
import Chimera.RustAdapter.Cache

namespace Chimera.RustAdapter.Invalidation

/--
  Change classification for invalidation.
-/
inductive ChangeKind where
  | added
  | removed
  | changed
  | unchanged
  | renumbered
  | abiChanged
  | layoutChanged
  | bodyOnlyChanged

/--
  Node type in the dependency graph.
-/
inductive NodeKind where
  | source
  | type
  | layout
  | mirBody
  | constEval
  | genericInstantiation
  | export
  | wrapper
  | metadata
  | proof
  | object
  | link

/--
  Node in the dependency graph.
-/
structure GraphNode where
  id : Nat
  kind : NodeKind
  name : String
  changeKind : ChangeKind

/--
  Invalidation action to take.
-/
inductive InvalidationAction where
  | rebuild
  | keep
  | delete

/--
  Invalidation rule.
  
  Describes how changes propagate through the dependency graph.
-/
structure InvalidationRule where
  changedNode : NodeKind
  affectedNodes : List NodeKind
  action : InvalidationAction
  reason : String

/--
  Invalidation result for a cache entry.
-/
structure InvalidationResult where
  entry : CacheEntry
  isValid : Bool
  invalidationPath : List GraphNode
  action : InvalidationAction

/--
  Public API change detection.
-/
structure PublicAPICheck where
  before : List String
  after : List String
  breakingChanges : List String
  isBreaking : Bool

/--
  Layout change detection.
-/
structure LayoutChangeCheck where
  typeName : String
  beforeLayout : LayoutInfo
  afterLayout : LayoutInfo
  isBreaking : Bool

/--
  Generic instantiation cache key change.
-/
structure GenericChangeCheck where
  defPath : String
  substitutions : List String
  constArgs : List String
  traitObligations : List String
  target : String
  rustcVersion : String
  beforeFingerprint : String
  afterFingerprint : String
  isBreaking : Bool

end Chimera.RustAdapter.Invalidation
