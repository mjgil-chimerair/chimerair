--! Chimera.RustAdapter.Snapshot
--!
--! Lean model for Rust semantic snapshots (`.rsnap`).
--! Represents the extracted Rust source information.

import Chimera.RustAdapter

namespace Chimera.RustAdapter.Snapshot

/--
  Item identifier in a Rust crate.
  
  Corresponds to `ItemId` in the schema.
-/
structure ItemId where
  index : Nat
  generation : Nat

/--
  Visibility of a Rust item.
  
  - `Private`: Only visible in defining module
  - `Module`: Visible in module and descendants
  - `Crate`: Visible in entire crate
  - `Public`: Visible everywhere
  - `ReExported`: Visible and re-exported
-/
inductive Visibility where
  | private
  | module
  | crate
  | public
  | reExported

/--
  Kind of a Rust item.
  
  - `Function`: fn item
  - `Static`: static variable
  - `Constant`: const item
  - `Type`: type alias or struct/enum/union
  - `Trait`: trait definition
  - `Impl`: impl block
-/
inductive ItemKind where
  | function
  | static
  | constant
  | type
  | trait
  | implBlock

/--
  A Rust item in the snapshot.
-/
structure Item where
  id : ItemId
  defPath : String
  kind : ItemKind
  visibility : Visibility
  attributes : List String
  generics : Option Generics
  whereClauses : List WhereClause

/--
  Generic parameters of a function or type.
-/
structure Generics where
  typeParams : List TypeParam
  constParams : List ConstParam
  lifetimes : List LifetimeParam

structure TypeParam where
  name : String
  bounds : List TypeBound

structure ConstParam where
  name : String
  ty : String

structure LifetimeParam where
  name : String

/--
  A type bound (e.g., `T: Clone`).
-/
inductive TypeBound where
  | sized
  | clone
  | copy
  | debug
  | display
  | error
  | from
  | into
  | default
  | unsafeBound (trait : String)

/--
  A where clause (e.g., `T: Clone` as a where clause).
-/
structure WhereClause where
  bounded : String
  bound : TypeBound

/--
  Rust crate node in the dependency graph.
-/
structure CrateNode where
  name : String
  version : String
  edition : String
  crateType : CrateType

/--
  Crate type (library, binary, proc-macro, etc.).
-/
inductive CrateType where
  | library
  | binary
  | procMacro
  | customBuild
  | test
  | benchmark
  | procMacroLib

/--
  Complete Rust semantic snapshot.
-/
structure Snapshot where
  rustcVersion : String
  targetTriple : String
  crateGraph : CrateGraph
  items : List Item
  exports : List ExportedItem
  sourceFiles : List String

structure CrateGraph where
  root : ItemId
  nodes : List CrateNode

structure ExportedItem where
  id : ItemId
  name : String
  reexportedFrom : Option String

end Chimera.RustAdapter.Snapshot
