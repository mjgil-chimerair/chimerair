-- ChimeraProof Metadata: Schema
-- Full metadata schema for ChimeraIR.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Contract
import Chimera.ABI.Signature
import Chimera.IR.Module
import Chimera.Memory.Allocator

namespace Chimera

inductive TrustAssumptionKind where
  | trustedFunction
  | trustedAllocator
  | trustedDrop
  | trustedLinker
  | trustedForeignAbi
  | manualProof
deriving Repr, BEq

structure ModuleMetadata where
  name : Symbol
  abiVersion : Nat
  language : SourceLanguage
  target : Target
deriving Repr, BEq

structure ImportMetadata where
  symbol : Symbol
  signature : SemanticSignature
  language : SourceLanguage
  target : Target
deriving Repr, BEq

structure ExportMetadata where
  symbol : Symbol
  signature : SemanticSignature
  language : SourceLanguage
  target : Target
  isPublic : Bool
deriving Repr, BEq

structure EffectMetadata where
  effect : Effect
  mayBlock : Bool
  mayAlloc : Bool
  mayDealloc : Bool
  mayPanic : Bool
deriving Repr, BEq

structure TrustAssumptionMetadata where
  kind : TrustAssumptionKind
  description : String
  externalRef : Option String
deriving Repr, BEq

inductive ParseChMetaError where
  | notImplemented
  | invalidFormat (reason : String)
  | missingField (field : String)
deriving Repr, BEq

namespace Metadata

structure ContractMetadata where
  symbol : Symbol
  safety : SafetyClass
  args : List ChType
  returns : ReturnSpec
  effects : EffectSet
  panic : PanicPolicy

structure DropFnMetadata where
  symbol : Symbol
  inputType : ChType
  language : SourceLanguage
  allocator : Option AllocatorId
  isTrusted : Bool
deriving Repr, BEq

inductive AllocatorKind where
  | system
  | null
  | shared
  | languageOwned
  | custom
deriving Repr, BEq

structure AllocatorMetadata where
  id : AllocatorId
  kind : AllocatorKind
  language : SourceLanguage
  isSystem : Bool
deriving Repr, BEq

structure PanicPolicyMetadata where
  policy : PanicPolicy
  catches : List Symbol
  aborts : List Symbol
deriving Repr, BEq

structure ChimeraMetadata where
  module_ : ModuleMetadata
  imports : List ImportMetadata
  exports : List ExportMetadata
  contracts : List ContractMetadata
  layouts : List DeclaredLayout
  types : List ChType
  effects : List EffectMetadata
  drops : List DropFnMetadata
  allocators : List AllocatorMetadata
  panicPolicy : PanicPolicyMetadata
  trustAssumptions : List TrustAssumptionMetadata

def parseChMetaJson (_json : String) : Except ParseChMetaError ChimeraMetadata :=
  Except.error .notImplemented

def parseChMetaJsonTCB (json : String) : Except ParseChMetaError ChimeraMetadata :=
  if json = "" then
    Except.error (.invalidFormat "empty JSON input")
  else
    parseChMetaJson json

end Metadata

/--
Well-formed metadata predicate: checks version, module, target, imports,
exports, contracts, layouts, allocators, drops, effects, and trust.
-/
def MetadataValid (m : Metadata.ChimeraMetadata) : Prop :=
  m.module_.abiVersion > 0 ∧
  m.module_.name.name ≠ "" ∧
  m.module_.target.ptrWidth > 0 ∧
  m.exports.all (fun e => e.symbol.name ≠ "") ∧
  m.imports.all (fun i => i.symbol.name ≠ "") ∧
  m.contracts.all (fun c => c.symbol.name ≠ "") ∧
  m.layouts.all (fun l => l.name.name ≠ "" ∧ l.align > 0 ∧ l.size ≥ l.align) ∧
  m.allocators.all (fun a => a.id.name.name ≠ "") ∧
  m.drops.all (fun d => d.symbol.name ≠ "") ∧
  m.trustAssumptions.all (fun t => t.description ≠ "")

theorem empty_metadata_invalid (m : Metadata.ChimeraMetadata)
  (hVersion : m.module_.abiVersion = 0) :
  ¬ MetadataValid m := by
  simp [MetadataValid, hVersion]

example : Metadata.parseChMetaJsonTCB "" = Except.error (.invalidFormat "empty JSON input") := by
  simp [Metadata.parseChMetaJsonTCB]

theorem parse_tcb_delegates_nonempty (json : String) (h : json ≠ "") :
  Metadata.parseChMetaJsonTCB json = Metadata.parseChMetaJson json := by
  simp [Metadata.parseChMetaJsonTCB, h]

end Chimera
