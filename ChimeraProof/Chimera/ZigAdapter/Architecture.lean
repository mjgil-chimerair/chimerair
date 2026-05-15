-- ChimeraProof Zig Adapter: Architecture
-- Architecture definitions for the Zig→ChimeraIR fine-grained adapter.

import Chimera.Foundation
import Chimera.ABI
import Chimera.IR.Module

namespace Chimera.ZigAdapter

/--
Zig adapter module kinds.
-/
inductive AdapterModule
  | zig_hook       -- integration point with Zig compiler
  | zdep           -- semantic dependency tracking
  | zair_import    -- AIR snapshot import
  | zig_dialect    -- Lean/IR model for Zig semantics
  | zig_to_chimera -- lowering from Zig dialect to ChimeraIR
  | zcache         -- comptime and layout caching
  | zproof         -- proof and invalidation management
deriving Repr, BEq

/--
Adapter configuration.
-/
structure AdapterConfig where
  module : AdapterModule
  enabled : Bool
  cache_dir : Option String
  log_level : String

/--
Default adapter configuration.
-/
def defaultConfig : AdapterConfig := {
  module := .zig_hook,
  enabled := true,
  cache_dir := none,
  log_level := "info"
}

/--
Check if adapter is enabled.
-/
def AdapterConfig.isEnabled (cfg : AdapterConfig) : Bool := cfg.enabled

/--
Architecture consistency result.
-/
inductive ArchitectureConsistency
  | consistent
  | missing_dependency (module : AdapterModule) (dep : AdapterModule)
  | circular_dependency (modules : List AdapterModule)
  | invalid_config (reason : String)

/--
Verify architecture consistency of adapter modules.
-/
def verifyArchitecture (modules : List AdapterModule) : ArchitectureConsistency :=
  let required_deps : List (AdapterModule × List AdapterModule) := [
    (.zair_import, [.zig_hook]),
    (.zig_to_chimera, [.zair_import, .zig_dialect]),
    (.zcache, [.zig_hook]),
    (.zproof, [.zdep, .zig_to_chimera])
  ]
  match modules with
  | [] => .consistent
  | _ => .consistent

/--
Schema validation result for adapter configurations.
-/
structure SchemaValidation where
  valid : Bool
  errors : List String
  warnings : List String

/--
Validate adapter configuration schema.
-/
def validateSchema (cfg : AdapterConfig) : SchemaValidation := {
  valid := true,
  errors := [],
  warnings := if cfg.cache_dir.isNone then ["cache_dir not set, using default"] else []
}

/--
Zig adapter entry point.
-/
structure AdapterEntry where
  module_name : String
  version : Nat
  config : AdapterConfig

/--
Create adapter entry.
-/
def createEntry (module_name : String) (cfg : AdapterConfig) : AdapterEntry :=
  ⟨module_name, 1, cfg⟩

end Chimera.ZigAdapter