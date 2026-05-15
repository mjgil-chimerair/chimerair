-- ChimeraProof Zig Adapter: Proof Bridge
-- Versioned Rust↔Lean bridge artifact for Zig adapter surfaces.

import Chimera.ZigAdapter.AIRSnapshot

namespace Chimera.ZigAdapter

inductive ZigBridgeItemKind
  | exportFn
  | externStruct
  | typeAlias
  | errorSet
deriving Repr, BEq, DecidableEq

structure ZigBridgeParam where
  name : String
  typ : String
deriving Repr, BEq, DecidableEq

structure ZigBridgeField where
  name : String
  typ : String
deriving Repr, BEq, DecidableEq

structure ZigBridgeItem where
  kind : ZigBridgeItemKind
  name : String
  returnType : Option String := none
  hasErrorUnion : Bool := false
  aliasType : Option String := none
  params : List ZigBridgeParam := []
  fields : List ZigBridgeField := []
deriving Repr, BEq, DecidableEq

structure ZigBridgeArtifact where
  version : Nat
  moduleName : String
  items : List ZigBridgeItem
deriving Repr, BEq, DecidableEq

namespace ZigBridgeArtifact

def currentVersion : Nat := 1

private def boolToken (b : Bool) : String :=
  if b then "true" else "false"

private def parseBool? (s : String) : Option Bool :=
  match s with
  | "true" => some true
  | "false" => some false
  | _ => none

private def kindToken : ZigBridgeItemKind → String
  | .exportFn => "export_fn"
  | .externStruct => "extern_struct"
  | .typeAlias => "type_alias"
  | .errorSet => "error_set"

private def parseKind? (s : String) : Option ZigBridgeItemKind :=
  match s with
  | "export_fn" => some .exportFn
  | "extern_struct" => some .externStruct
  | "type_alias" => some .typeAlias
  | "error_set" => some .errorSet
  | _ => none

private def optionToken (s : Option String) : String :=
  s.getD "_"

private def parseOptionToken (s : String) : Option String :=
  if s = "_" then none else some s

private def itemWireLines (idx : Nat) (item : ZigBridgeItem) : List String :=
  let header :=
    String.intercalate "|" [
      "item",
      toString idx,
      kindToken item.kind,
      item.name,
      optionToken item.returnType,
      boolToken item.hasErrorUnion,
      optionToken item.aliasType
    ]
  let params := item.params.map (fun param =>
    String.intercalate "|" ["param", toString idx, param.name, param.typ])
  let fields := item.fields.map (fun field =>
    String.intercalate "|" ["field", toString idx, field.name, field.typ])
  header :: (params ++ fields)

def serialize (artifact : ZigBridgeArtifact) : String :=
  let header := String.intercalate "|" ["zig-bridge", toString artifact.version, artifact.moduleName]
  let itemLines := artifact.items.enum.bind (fun ⟨idx, item⟩ => itemWireLines idx item)
  String.intercalate "\n" (header :: itemLines)

private structure IndexedItem where
  idx : Nat
  item : ZigBridgeItem
deriving Repr, BEq, DecidableEq

private def attachParam (items : List IndexedItem) (idx : Nat) (param : ZigBridgeParam) : Option (List IndexedItem)
  match items with
  | [] => none
  | head :: tail =>
      if head.idx = idx then
        some ({ head with item := { head.item with params := head.item.params ++ [param] } } :: tail)
      else
        (attachParam tail idx param).map (head :: ·)

private def attachField (items : List IndexedItem) (idx : Nat) (field : ZigBridgeField) : Option (List IndexedItem)
  match items with
  | [] => none
  | head :: tail =>
      if head.idx = idx then
        some ({ head with item := { head.item with fields := head.item.fields ++ [field] } } :: tail)
      else
        (attachField tail idx field).map (head :: ·)

private def parseItemLine? (parts : List String) : Option IndexedItem := do
  let [_, idx, kind, name, returnType, hasErrorUnion, aliasType] := parts | none
  let idx ← idx.toNat?
  let kind ← parseKind? kind
  let hasErrorUnion ← parseBool? hasErrorUnion
  pure {
    idx := idx
    item := {
      kind := kind
      name := name
      returnType := parseOptionToken returnType
      hasErrorUnion := hasErrorUnion
      aliasType := parseOptionToken aliasType
    }
  }

private def parsePayloadLine?
    (parts : List String)
    (items : List IndexedItem) : Option (List IndexedItem) := do
  match parts with
  | ["param", idx, name, typ] =>
      let idx ← idx.toNat?
      attachParam items idx { name := name, typ := typ }
  | ["field", idx, name, typ] =>
      let idx ← idx.toNat?
      attachField items idx { name := name, typ := typ }
  | _ => none

private def parseRows (rows : List String) (items : List IndexedItem) : Option (List IndexedItem) := do
  match rows with
  | [] => pure items
  | row :: rest =>
      let parts := row.splitOn "|"
      match parts with
      | "item" :: _ =>
          let indexed ← parseItemLine? parts
          if indexed.idx = items.length then
            parseRows rest (items ++ [indexed])
          else
            none
      | "param" :: _ =>
          let items ← parsePayloadLine? parts items
          parseRows rest items
      | "field" :: _ =>
          let items ← parsePayloadLine? parts items
          parseRows rest items
      | _ => none

def deserialize? (wire : String) : Option ZigBridgeArtifact := do
  let rows := wire.splitOn "\n"
  let header :: itemRows := rows | none
  let ["zig-bridge", version, moduleName] := header.splitOn "|" | none
  let version ← version.toNat?
  let items ← parseRows itemRows []
  pure {
    version := version
    moduleName := moduleName
    items := items.map (·.item)
  }

private def functionSignature (item : ZigBridgeItem) : String :=
  let params := item.params.map (fun param => s!"{param.name}:{param.typ}")
  let ret := item.returnType.getD "void"
  s!"fn({String.intercalate "," params})->{ret}"

private def structLayoutHash (item : ZigBridgeItem) : String :=
  String.intercalate "," (item.fields.map (fun field => s!"{field.name}:{field.typ}"))

def toAIRSnapshot (artifact : ZigBridgeArtifact) : AIRSnapshot :=
  let functions := artifact.items.foldl (fun acc item =>
    match item.kind with
    | .exportFn =>
        acc ++ [{ name := item.name
                , params := item.params.map (·.typ)
                , return_type := item.returnType.getD "void"
                , body_ir := if item.hasErrorUnion then "bridge:error_union" else "bridge:direct" }]
    | _ => acc) []
  let types := artifact.items.foldl (fun acc item =>
    match item.kind with
    | .externStruct =>
        acc ++ [{ type_name := item.name, kind := "extern_struct", layout_hash := structLayoutHash item }]
    | .typeAlias =>
        acc ++ [{ type_name := item.name, kind := "type_alias", layout_hash := item.aliasType.getD "" }]
    | .errorSet =>
        acc ++ [{ type_name := item.name, kind := "error_set", layout_hash := "error_set" }]
    | _ => acc) []
  let exports := artifact.items.foldl (fun acc item =>
    match item.kind with
    | .exportFn =>
        acc ++ [{ name := item.name, signature := functionSignature item, visibility := "public" }]
    | _ => acc) []
  { version := artifact.version
  , functions := functions
  , type_table := types
  , layout_table := []
  , comptime_values := []
  , exported_symbols := exports }

theorem serialize_roundtrip_sample :
    let artifact : ZigBridgeArtifact := {
      version := currentVersion
      moduleName := "ffi_demo"
      items := [
        { kind := .exportFn
        , name := "demo_export"
        , returnType := some "!i32"
        , hasErrorUnion := true
        , params := [{ name := "input", typ := "*const u8" }] },
        { kind := .externStruct
        , name := "DemoStruct"
        , fields := [{ name := "len", typ := "usize" }, { name := "ptr", typ := "*const u8" }] },
        { kind := .typeAlias
        , name := "ByteSlice"
        , aliasType := some "[]const u8" },
        { kind := .errorSet
        , name := "ParseError" }
      ]
    }
    deserialize? artifact.serialize = some artifact := by
  native_decide

theorem bridge_snapshot_tracks_export_and_type_counts :
    let artifact : ZigBridgeArtifact := {
      version := currentVersion
      moduleName := "ffi_demo"
      items := [
        { kind := .exportFn
        , name := "demo_export"
        , returnType := some "i32"
        , params := [{ name := "lhs", typ := "i32" }, { name := "rhs", typ := "i32" }] },
        { kind := .externStruct
        , name := "DemoStruct"
        , fields := [{ name := "ptr", typ := "*const u8" }] },
        { kind := .errorSet
        , name := "ParseError" }
      ]
    }
    let snap := artifact.toAIRSnapshot
    snap.functions.length = 1 ∧ snap.type_table.length = 2 ∧ snap.exported_symbols.length = 1 := by
  native_decide

end ZigBridgeArtifact

end Chimera.ZigAdapter
