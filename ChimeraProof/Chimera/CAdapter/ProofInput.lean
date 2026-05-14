-- CAdapter Proof Input Schema
-- Task 137: Define C proof input schema in Lean

import Lean
import Chimera.CAdapter.Snapshot
import Chimera.CAdapter.DependencyGraph

namespace Chimera.CAdapter

/--
C Layout Proof Fact
-/
structure CLayoutProofFact where
  struct_name : String
  size_bytes : Nat
  alignment_bytes : Nat
  field_offsets : List (String × Nat)
  schema_version : Nat
deriving Repr, BEq, DecidableEq

/--
C Signature Proof Fact
-/
structure CSignatureProofFact where
  function_name : String
  parameter_types : List String
  return_type : String
  calling_convention : String
  schema_version : Nat
deriving Repr, BEq, DecidableEq

/--
C Pointer Proof Fact
-/
structure CPointerProofFact where
  pointer_name : String
  nullability : String
  ownership : String
  schema_version : Nat
deriving Repr, BEq, DecidableEq

/--
C Errno Proof Fact
-/
structure CErrnoProofFact where
  function_name : String
  error_convention : String
  schema_version : Nat
deriving Repr, BEq, DecidableEq

/--
C Cache Proof Fact
-/
structure CCacheProofFact where
  cache_key : String
  semantic_fingerprint_hex : String
  dependency_fingerprints_hex : List String
  schema_version : Nat
  target : String
  compiler_identity : String
  reusable : Bool
deriving Repr, BEq, DecidableEq

/--
C Invalidation Proof Fact
-/
structure CInvalidationProofFact where
  changed_node : Nat
  reason : String
  affected_exports : List Nat
  sound : Bool
deriving Repr, BEq, DecidableEq

/--
C Lean Proof Input Artifact
-/
structure CLeanProofInputArtifact where
  version : Nat
  target : String
  layout_facts : List CLayoutProofFact
  signature_facts : List CSignatureProofFact
  pointer_facts : List CPointerProofFact
  errno_facts : List CErrnoProofFact
  cache_facts : List CCacheProofFact
  invalidation_facts : List CInvalidationProofFact
deriving Repr, BEq, DecidableEq

namespace CLeanProofInputArtifact

/--
Current schema version
-/
def currentVersion : Nat := 1

/--
Serialize artifact to string
-/
def serialize (artifact : CLeanProofInputArtifact) : String :=
  let header := String.intercalate "|" ["c-proof-input", toString artifact.version, artifact.target]
  let layoutRows := artifact.layout_facts.map (fun fact =>
    String.intercalate "|" [
      "layout",
      fact.struct_name,
      toString fact.size_bytes,
      toString fact.alignment_bytes,
      String.intercalate "," (fact.field_offsets.map (fun (n, o) => n ++ ":" ++ toString o)),
      toString fact.schema_version
    ])
  let sigRows := artifact.signature_facts.map (fun fact =>
    String.intercalate "|" [
      "signature",
      fact.function_name,
      String.intercalate "," fact.parameter_types,
      fact.return_type,
      fact.calling_convention,
      toString fact.schema_version
    ])
  let ptrRows := artifact.pointer_facts.map (fun fact =>
    String.intercalate "|" [
      "pointer",
      fact.pointer_name,
      fact.nullability,
      fact.ownership,
      toString fact.schema_version
    ])
  let errnoRows := artifact.errno_facts.map (fun fact =>
    String.intercalate "|" [
      "errno",
      fact.function_name,
      fact.error_convention,
      toString fact.schema_version
    ])
  let cacheRows := artifact.cache_facts.map (fun fact =>
    String.intercalate "|" [
      "cache",
      fact.cache_key,
      fact.semantic_fingerprint_hex,
      String.intercalate "," fact.dependency_fingerprints_hex,
      toString fact.schema_version,
      fact.target,
      fact.compiler_identity,
      if fact.reusable then "true" else "false"
    ])
  let invRows := artifact.invalidation_facts.map (fun fact =>
    String.intercalate "|" [
      "invalidate",
      toString fact.changed_node,
      fact.reason,
      String.intercalate "," (fact.affected_exports.map toString),
      if fact.sound then "true" else "false"
    ])
  String.intercalate "\n" (header :: (layoutRows ++ sigRows ++ ptrRows ++ errnoRows ++ cacheRows ++ invRows))

/--
Deserialize string to artifact
-/
def deserialize? (wire : String) : Option CLeanProofInputArtifact := do
  let rows := wire.splitOn "\n"
  let header :: rest := rows | none
  let ["c-proof-input", version, target] := header.splitOn "|" | none
  let version ← version.toNat?
  let rec parseRows
      (remaining : List String)
      (layout_facts : List CLayoutProofFact)
      (signature_facts : List CSignatureProofFact)
      (pointer_facts : List CPointerProofFact)
      (errno_facts : List CErrnoProofFact)
      (cache_facts : List CCacheProofFact)
      (invalidation_facts : List CInvalidationProofFact)
      : Option (List CLayoutProofFact × List CSignatureProofFact × List CPointerProofFact × List CErrnoProofFact × List CCacheProofFact × List CInvalidationProofFact) := do
    match remaining with
    | [] => pure (layout_facts, signature_facts, pointer_facts, errno_facts, cache_facts, invalidation_facts)
    | row :: tail =>
      match row.splitOn "|" with
      | ["layout", name, size, align, fields, ver] =>
        let size ← size.toNat?
        let align ← align.toNat?
        let ver ← ver.toNat?
        let field_pairs := if fields.isEmpty then [] else fields.splitOn "," |>.filterMap (fun s =>
          match s.splitOn ":" with
          | [n, o] =>
            match o.toNat? with
            | some offset => some (n, offset)
            | none => none
          | _ => none)
        parseRows tail
          (layout_facts ++ [{ struct_name := name, size_bytes := size, alignment_bytes := align, field_offsets := field_pairs, schema_version := ver }])
          signature_facts pointer_facts errno_facts cache_facts invalidation_facts
      | ["signature", name, params, ret, cc, ver] =>
        let ver ← ver.toNat?
        parseRows tail layout_facts
          (signature_facts ++ [{ function_name := name, parameter_types := params.splitOn ",", return_type := ret, calling_convention := cc, schema_version := ver }])
          pointer_facts errno_facts cache_facts invalidation_facts
      | ["pointer", name, null, own, ver] =>
        let ver ← ver.toNat?
        parseRows tail layout_facts signature_facts
          (pointer_facts ++ [{ pointer_name := name, nullability := null, ownership := own, schema_version := ver }])
          errno_facts cache_facts invalidation_facts
      | ["errno", name, conv, ver] =>
        let ver ← ver.toNat?
        parseRows tail layout_facts signature_facts pointer_facts
          (errno_facts ++ [{ function_name := name, error_convention := conv, schema_version := ver }])
          cache_facts invalidation_facts
      | ["cache", key, sem, deps, ver, tgt, comp, reuse] =>
        let ver ← ver.toNat?
        let reuse ← if reuse == "true" then pure true else if reuse == "false" then pure false else none
        let dep_list := if deps.isEmpty then [] else deps.splitOn ","
        parseRows tail layout_facts signature_facts pointer_facts errno_facts
          (cache_facts ++ [{ cache_key := key, semantic_fingerprint_hex := sem, dependency_fingerprints_hex := dep_list, schema_version := ver, target := tgt, compiler_identity := comp, reusable := reuse }])
          invalidation_facts
      | ["invalidate", node, reason, exports, sound] =>
        let node ← node.toNat?
        let sound ← if sound == "true" then pure true else if sound == "false" then pure false else none
        let export_list ← if exports.isEmpty then pure [] else
          match exports.splitOn "," |>.mapM String.toNat? with
          | some l => pure l
          | none => none
        parseRows tail layout_facts signature_facts pointer_facts errno_facts cache_facts
          (invalidation_facts ++ [{ changed_node := node, reason := reason, affected_exports := export_list, sound := sound }])
      | _ => none
  let (lfs, sfs, pfs, efs, cfs, ifs) ← parseRows rest [] [] [] [] [] []
  pure {
    version := version
    target := target
    layout_facts := lfs
    signature_facts := sfs
    pointer_facts := pfs
    errno_facts := efs
    cache_facts := cfs
    invalidation_facts := ifs
  }

end CLeanProofInputArtifact

end Chimera.CAdapter