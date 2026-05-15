-- ChimeraProof Zig Adapter: Proof Input
-- Standalone Lean-readable proof input schema for `.zchproof` cache and invalidation facts.

import Chimera.Foundation

namespace Chimera.ZigAdapter

structure ZigLeanCacheProofFact where
  cacheKey : String
  semanticFingerprintHex : String
  dependencyFingerprintsHex : List String
  schemaVersion : Nat
  target : String
  buildOptionsHashHex : String
  reusable : Bool
deriving Repr, BEq, DecidableEq

structure ZigLeanInvalidationProofFact where
  changedNode : Nat
  reason : String
  affectedExports : List Nat
  sound : Bool
deriving Repr, BEq, DecidableEq

structure ZigLeanProofInputArtifact where
  version : Nat
  target : String
  cacheFacts : List ZigLeanCacheProofFact
  invalidationFacts : List ZigLeanInvalidationProofFact
deriving Repr, BEq, DecidableEq

namespace ZigLeanProofInputArtifact

def currentVersion : Nat := 1

private def boolToken (b : Bool) : String :=
  if b then "true" else "false"

private def parseBool? (s : String) : Option Bool :=
  match s with
  | "true" => some true
  | "false" => some false
  | _ => none

private def repeatToken (token : String) (count : Nat) : String :=
  String.intercalate "" (List.replicate count token)

def serialize (artifact : ZigLeanProofInputArtifact) : String :=
  let header := String.intercalate "|" ["zig-proof-input", toString artifact.version, artifact.target]
  let cacheRows := artifact.cacheFacts.map (fun fact =>
    String.intercalate "|" [
      "cache",
      fact.cacheKey,
      fact.semanticFingerprintHex,
      String.intercalate "," fact.dependencyFingerprintsHex,
      toString fact.schemaVersion,
      fact.target,
      fact.buildOptionsHashHex,
      boolToken fact.reusable
    ])
  let invalidationRows := artifact.invalidationFacts.map (fun fact =>
    String.intercalate "|" [
      "invalidate",
      toString fact.changedNode,
      fact.reason,
      String.intercalate "," (fact.affectedExports.map toString),
      boolToken fact.sound
    ])
  String.intercalate "\n" (header :: (cacheRows ++ invalidationRows))

def deserialize? (wire : String) : Option ZigLeanProofInputArtifact := do
  let rows := wire.splitOn "\n"
  let header :: rest := rows | none
  let ["zig-proof-input", version, target] := header.splitOn "|" | none
  let version ← version.toNat?
  let rec parseRows
      (remaining : List String)
      (cacheFacts : List ZigLeanCacheProofFact)
      (invalidationFacts : List ZigLeanInvalidationProofFact)
      : Option (List ZigLeanCacheProofFact × List ZigLeanInvalidationProofFact) := do
    match remaining with
    | [] => pure (cacheFacts, invalidationFacts)
    | row :: tail =>
        match row.splitOn "|" with
        | ["cache", cacheKey, semanticFingerprintHex, dependencyFingerprintsHex, schemaVersion, cacheTarget, buildOptionsHashHex, reusable] =>
            let schemaVersion ← schemaVersion.toNat?
            let reusable ← parseBool? reusable
            let deps :=
              if dependencyFingerprintsHex.isEmpty then [] else dependencyFingerprintsHex.splitOn ","
            parseRows tail
              (cacheFacts ++ [{
                cacheKey := cacheKey
                semanticFingerprintHex := semanticFingerprintHex
                dependencyFingerprintsHex := deps
                schemaVersion := schemaVersion
                target := cacheTarget
                buildOptionsHashHex := buildOptionsHashHex
                reusable := reusable
              }])
              invalidationFacts
        | ["invalidate", changedNode, reason, affectedExports, sound] =>
            let changedNode ← changedNode.toNat?
            let sound ← parseBool? sound
            let exports ←
              if affectedExports.isEmpty then
                pure []
              else
                affectedExports.splitOn "," |>.mapM String.toNat?
            parseRows tail cacheFacts (invalidationFacts ++ [{
              changedNode := changedNode
              reason := reason
              affectedExports := exports
              sound := sound
            }])
        | _ => none
  let (cacheFacts, invalidationFacts) ← parseRows rest [] []
  pure {
    version := version
    target := target
    cacheFacts := cacheFacts
    invalidationFacts := invalidationFacts
  }

theorem proof_input_roundtrip_sample :
    let artifact : ZigLeanProofInputArtifact := {
      version := currentVersion
      target := "x86_64-unknown-linux-gnu"
      cacheFacts := [{
        cacheKey := "cache-key-1"
        semanticFingerprintHex := repeatToken "01" 32
        dependencyFingerprintsHex := [repeatToken "02" 32, repeatToken "03" 32]
        schemaVersion := 1
        target := "x86_64-unknown-linux-gnu"
        buildOptionsHashHex := repeatToken "04" 32
        reusable := true
      }]
      invalidationFacts := [{
        changedNode := 9
        reason := "layout_changed"
        affectedExports := [11, 12]
        sound := true
      }]
    }
    deserialize? artifact.serialize = some artifact := by
  native_decide

theorem proof_input_preserves_fact_counts :
    let artifact : ZigLeanProofInputArtifact := {
      version := currentVersion
      target := "x86_64-unknown-linux-gnu"
      cacheFacts := [{
        cacheKey := "cache-key-1"
        semanticFingerprintHex := repeatToken "01" 32
        dependencyFingerprintsHex := [repeatToken "02" 32]
        schemaVersion := 1
        target := "x86_64-unknown-linux-gnu"
        buildOptionsHashHex := repeatToken "04" 32
        reusable := true
      }]
      invalidationFacts := [{
        changedNode := 9
        reason := "layout_changed"
        affectedExports := []
        sound := true
      }]
    }
    let restored := deserialize? artifact.serialize
    restored.map (fun value => (value.cacheFacts.length, value.invalidationFacts.length)) = some (1, 1) := by
  native_decide

end ZigLeanProofInputArtifact

end Chimera.ZigAdapter
