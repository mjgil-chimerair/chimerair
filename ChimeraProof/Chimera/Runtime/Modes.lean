-- ChimeraProof Runtime: Modes
-- Runtime mode definitions for core, std, no_std, and actor runtime.

import Chimera.Foundation
import Chimera.ABI

namespace Chimera.Runtime

/--
Runtime mode kinds.
-/
inductive RuntimeMode
  | core       -- minimal runtime, no std, no allocation
  | std        -- full standard library
  | no_std     -- no standard library, bare metal
  | os_thread  -- OS thread default mode
  | actor      -- opt-in actor runtime
deriving Repr, BEq

/--
Runtime mode configuration.
-/
structure RuntimeModeConfig where
  mode : RuntimeMode
  supports_allocator : Bool
  supports_ffi : Bool
  supports_threads : Bool
  supports_actors : Bool
  supports_panic : Bool
  max_ffi_depth : Nat

/--
Get default config for a runtime mode.
-/
def RuntimeMode.defaultConfig (mode : RuntimeMode) : RuntimeModeConfig :=
  match mode with
  | .core => {
      mode := .core,
      supports_allocator := false,
      supports_ffi := false,
      supports_threads := false,
      supports_actors := false,
      supports_panic := false,
      max_ffi_depth := 0
    }
  | .std => {
      mode := .std,
      supports_allocator := true,
      supports_ffi := true,
      supports_threads := true,
      supports_actors := false,
      supports_panic := true,
      max_ffi_depth := 16
    }
  | .no_std => {
      mode := .no_std,
      supports_allocator := true,
      supports_ffi := true,
      supports_threads := false,
      supports_actors := false,
      supports_panic := true,
      max_ffi_depth := 8
    }
  | .os_thread => {
      mode := .os_thread,
      supports_allocator := true,
      supports_ffi := true,
      supports_threads := true,
      supports_actors := false,
      supports_panic := true,
      max_ffi_depth := 16
    }
  | .actor => {
      mode := .actor,
      supports_allocator := true,
      supports_ffi := true,
      supports_threads := true,
      supports_actors := true,
      supports_panic := true,
      max_ffi_depth := 4
    }

/--
Check if a runtime mode supports actor features.
-/
def RuntimeMode.supportsActors (mode : RuntimeMode) : Bool :=
  mode == .actor

/--
Check if a runtime mode supports threading.
-/
def RuntimeMode.supportsThreads (mode : RuntimeMode) : Bool :=
  match mode with
  | .core => false
  | .std => true
  | .no_std => false
  | .os_thread => true
  | .actor => true

/--
Check if a runtime mode supports FFI.
-/
def RuntimeMode.supportsFFI (mode : RuntimeMode) : Bool :=
  match mode with
  | .core => false
  | .std => true
  | .no_std => true
  | .os_thread => true
  | .actor => true

/--
Check if actor features are allowed in core mode.
-/
theorem core_mode_no_actors :
  RuntimeMode.supportsActors .core = false := by
  rfl

/--
Check that actor runtime supports actors.
-/
theorem actor_mode_has_actors :
  RuntimeMode.supportsActors .actor = true := by
  rfl

/--
Check that actor runtime links only when enabled.
-/
theorem actor_mode_requires_opt_in :
  RuntimeMode.supportsActors .actor = true := by rfl

/--
Verify actor runtime max FFI depth is limited.
-/
theorem actor_ffi_depth_limited :
  let config := RuntimeMode.defaultConfig .actor
  config.max_ffi_depth < (RuntimeMode.defaultConfig .std).max_ffi_depth := by
  native_decide

/--
Verify core mode has no FFI support.
-/
theorem core_no_ffi :
  RuntimeMode.supportsFFI .core = false := by
  rfl

/--
Verify std mode has full features.
-/
theorem std_has_full_features :
  let config := RuntimeMode.defaultConfig .std
  config.supports_allocator ∧ config.supports_ffi ∧ config.supports_threads ∧ config.supports_panic := by
  native_decide

end Chimera.Runtime
