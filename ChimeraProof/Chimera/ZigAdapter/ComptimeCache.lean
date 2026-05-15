-- ChimeraProof Zig Adapter: Comptime Cache
-- Comptime artifact cache model for Zig→ChimeraIR incremental compilation.

import Chimera.Foundation
import Chimera.ZigAdapter.DependencyGraph

namespace Chimera.ZigAdapter

/--
Comptime cache key components.
-/
structure ComptimeCacheKeyComponents where
  schemaVersion : Nat
  function_body : String
  semanticFingerprint : String
  args : List String
  target : String
  build_options : String
  dependencyFingerprints : List String
  referenced_decls : List String
  builtins : List String
  embed_file_hashes : List String

/--
Comptime cache key.
-/
structure ComptimeCacheKey where
  components : ComptimeCacheKeyComponents
  hash : String

/--
Comptime cache value.
-/
structure ComptimeCacheValue where
  result : String
  dependencies : List Nat
  cached_at : Nat

/--
Comptime cache entry.
-/
structure ComptimeCacheEntry where
  key : ComptimeCacheKey
  value : ComptimeCacheValue

/--
Comptime cache.
-/
structure ComptimeCache where
  entries : List ComptimeCacheEntry
  hits : Nat
  misses : Nat

namespace ComptimeCache

/--
Empty comptime cache.
-/
def empty : ComptimeCache := ⟨[], 0, 0⟩

/--
Compute cache key from components.
-/
def computeKey (components : ComptimeCacheKeyComponents) : ComptimeCacheKey :=
  let hash := toString components.schemaVersion ++ ":" ++
    components.function_body ++ ":" ++
    components.semanticFingerprint ++ ":" ++
    components.args.foldl (fun acc a => acc ++ a) "" ++ ":" ++
    components.target ++ ":" ++
    components.build_options ++ ":" ++
    components.dependencyFingerprints.foldl (fun acc dep => acc ++ dep) ""
  ⟨components, hash⟩

/--
Check if cache entry is reusable for given key.
-/
def isReusable (cache : ComptimeCache) (key : ComptimeCacheKey) : Bool :=
  cache.entries.any (·.key.hash = key.hash)

/--
Check if cache entry is invalidated.
-/
def isInvalidated (cache : ComptimeCache) (key : ComptimeCacheKey) : Bool :=
  not (cache.entries.any (·.key.hash = key.hash))

/--
Add entry to cache.
-/
def addEntry (cache : ComptimeCache) (entry : ComptimeCacheEntry) : ComptimeCache :=
  { cache with entries := entry :: cache.entries }

end ComptimeCache

/--
Test: empty cache has no entries.
-/
theorem empty_cache_no_entries :
  ComptimeCache.empty.entries = [] := by rfl

/--
Test: empty cache has zero hits and misses.
-/
theorem empty_cache_zero_stats :
  ComptimeCache.empty.hits = 0 ∧ ComptimeCache.empty.misses = 0 := by
  simp [ComptimeCache.empty]

/--
Test: same key components produce same hash.
-/
theorem same_components_same_hash :
  let components := ComptimeCacheKeyComponents.mk 1 "fn body" "semfp" ["arg1"] "x86_64" "debug" ["depfp"] [] [] []
  let key1 := ComptimeCache.computeKey components
  let key2 := ComptimeCache.computeKey components
  key1.hash = key2.hash := by rfl

/--
Matching schema/target/build/dependency inputs allow reuse.
-/
theorem matching_cache_inputs_allow_reuse :
  let components := ComptimeCacheKeyComponents.mk 1 "fn body" "semfp" ["arg1"] "x86_64" "debug" ["depfp"] [] [] []
  let key := ComptimeCache.computeKey components
  let entry : ComptimeCacheEntry := {
    key := key
    value := {
      result := "42"
      dependencies := []
      cached_at := 0
    }
  }
  let cache := ComptimeCache.addEntry ComptimeCache.empty entry
  ComptimeCache.isReusable cache key = true := by
  native_decide

/--
Changing the schema version changes the cache key and prevents reuse.
-/
theorem changed_schema_version_prevents_reuse :
  let oldComponents := ComptimeCacheKeyComponents.mk 1 "fn body" "semfp" ["arg1"] "x86_64" "debug" ["depfp"] [] [] []
  let newComponents := { oldComponents with schemaVersion := 2 }
  let key1 := ComptimeCache.computeKey oldComponents
  let key2 := ComptimeCache.computeKey newComponents
  key1.hash ≠ key2.hash := by
  native_decide

/--
Changing dependency fingerprints changes the cache key and prevents reuse.
-/
theorem changed_dependency_fingerprint_prevents_reuse :
  let oldComponents := ComptimeCacheKeyComponents.mk 1 "fn body" "semfp" ["arg1"] "x86_64" "debug" ["depfp"] [] [] []
  let newComponents := { oldComponents with dependencyFingerprints := ["depfp2"] }
  let key1 := ComptimeCache.computeKey oldComponents
  let key2 := ComptimeCache.computeKey newComponents
  key1.hash ≠ key2.hash := by
  native_decide

/--
Changing target changes the cache key and prevents reuse.
-/
theorem changed_target_prevents_reuse :
  let oldComponents := ComptimeCacheKeyComponents.mk 1 "fn body" "semfp" ["arg1"] "x86_64" "debug" ["depfp"] [] [] []
  let newComponents := { oldComponents with target := "aarch64" }
  let key1 := ComptimeCache.computeKey oldComponents
  let key2 := ComptimeCache.computeKey newComponents
  key1.hash ≠ key2.hash := by
  native_decide

/--
Changing build options changes the cache key and prevents reuse.
-/
theorem changed_build_options_prevents_reuse :
  let oldComponents := ComptimeCacheKeyComponents.mk 1 "fn body" "semfp" ["arg1"] "x86_64" "debug" ["depfp"] [] [] []
  let newComponents := { oldComponents with build_options := "release-fast" }
  let key1 := ComptimeCache.computeKey oldComponents
  let key2 := ComptimeCache.computeKey newComponents
  key1.hash ≠ key2.hash := by
  native_decide

/--
Changing the semantic fingerprint changes the cache key and prevents reuse.
-/
theorem changed_semantic_fingerprint_prevents_reuse :
  let oldComponents := ComptimeCacheKeyComponents.mk 1 "fn body" "semfp" ["arg1"] "x86_64" "debug" ["depfp"] [] [] []
  let newComponents := { oldComponents with semanticFingerprint := "semfp2" }
  let key1 := ComptimeCache.computeKey oldComponents
  let key2 := ComptimeCache.computeKey newComponents
  key1.hash ≠ key2.hash := by
  native_decide

/--
Task 115 summary theorem: cache reuse is accepted for matching inputs and rejected for
schema, target, build-option, semantic-fingerprint, and dependency-fingerprint drift.
-/
theorem zig_cache_soundness_surface :
  (let components := ComptimeCacheKeyComponents.mk 1 "fn body" "semfp" ["arg1"] "x86_64" "debug" ["depfp"] [] [] []
   let key := ComptimeCache.computeKey components
   let entry : ComptimeCacheEntry := {
     key := key
     value := { result := "42", dependencies := [], cached_at := 0 }
   }
   let cache := ComptimeCache.addEntry ComptimeCache.empty entry
   ComptimeCache.isReusable cache key = true) ∧
    (let oldComponents := ComptimeCacheKeyComponents.mk 1 "fn body" "semfp" ["arg1"] "x86_64" "debug" ["depfp"] [] [] []
     let newComponents := { oldComponents with schemaVersion := 2 }
     let key1 := ComptimeCache.computeKey oldComponents
     let key2 := ComptimeCache.computeKey newComponents
     key1.hash ≠ key2.hash) ∧
    (let oldComponents := ComptimeCacheKeyComponents.mk 1 "fn body" "semfp" ["arg1"] "x86_64" "debug" ["depfp"] [] [] []
     let newComponents := { oldComponents with target := "aarch64" }
     let key1 := ComptimeCache.computeKey oldComponents
     let key2 := ComptimeCache.computeKey newComponents
     key1.hash ≠ key2.hash) ∧
    (let oldComponents := ComptimeCacheKeyComponents.mk 1 "fn body" "semfp" ["arg1"] "x86_64" "debug" ["depfp"] [] [] []
     let newComponents := { oldComponents with build_options := "release-fast" }
     let key1 := ComptimeCache.computeKey oldComponents
     let key2 := ComptimeCache.computeKey newComponents
     key1.hash ≠ key2.hash) ∧
    (let oldComponents := ComptimeCacheKeyComponents.mk 1 "fn body" "semfp" ["arg1"] "x86_64" "debug" ["depfp"] [] [] []
     let newComponents := { oldComponents with semanticFingerprint := "semfp2" }
     let key1 := ComptimeCache.computeKey oldComponents
     let key2 := ComptimeCache.computeKey newComponents
     key1.hash ≠ key2.hash) ∧
    (let oldComponents := ComptimeCacheKeyComponents.mk 1 "fn body" "semfp" ["arg1"] "x86_64" "debug" ["depfp"] [] [] []
     let newComponents := { oldComponents with dependencyFingerprints := ["depfp2"] }
     let key1 := ComptimeCache.computeKey oldComponents
     let key2 := ComptimeCache.computeKey newComponents
     key1.hash ≠ key2.hash) := by
  repeat' constructor <;> native_decide

end Chimera.ZigAdapter
