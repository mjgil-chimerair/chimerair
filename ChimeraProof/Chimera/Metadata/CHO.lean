-- ChimeraProof Metadata: CHO
-- Chimera Object file format (.cho) model.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Contract
import Chimera.Metadata.Schema
import Chimera.IR.Module

namespace Chimera.Metadata

inductive CHO_payload_kind
  | object
  | bitcode
  | archive
  | generated_wrapper
  | metadata_only
  | proof_carrying
deriving Repr, BEq

structure CHO_header where
  magic : String := "CHIMERA_OBJ"
  version : Nat := 1
  payload_kind : CHO_payload_kind
  target_ptr_width : Nat
  target_endian : Endianness
  timestamp : Nat
deriving Repr, BEq

structure CHO_import_entry where
  symbol : Symbol
  signature : SemanticSignature
  language : SourceLanguage
deriving Repr, BEq

structure CHO_export_entry where
  symbol : Symbol
  signature : SemanticSignature
  language : SourceLanguage
  is_public : Bool
deriving Repr, BEq

structure CHO_contract_entry where
  symbol : Symbol
  safety : SafetyClass
  semantic_sig : SemanticSignature
  physical_sig : PhysicalSignature
  effects : EffectSet
  panic : PanicPolicy

structure CHO_layout_entry where
  name : String
  size : Nat
  align : Nat
  fields : List DeclaredField

structure CHO_drop_entry where
  symbol : Symbol
  input_type : ChType
  language : SourceLanguage
  allocator : Option AllocatorId
  is_trusted : Bool
deriving Repr, BEq

structure CHO_allocator_entry where
  id : AllocatorId
  kind : AllocatorKind
  language : SourceLanguage
  is_system : Bool
deriving Repr, BEq

structure CHO_trust_entry where
  kind : TrustAssumptionKind
  description : String
  external_ref : Option String
deriving Repr, BEq

inductive ProofStatus
  | implemented
  | tested
  | proved
  | assumed
  | trusted
  | unsupported
deriving Repr, BEq

structure CHO_proof_obligation where
  kind : String
  description : String
  status : ProofStatus
  assumptions : List String
deriving Repr, BEq

structure ChimeraObjectFile where
  header : CHO_header
  imports : List CHO_import_entry
  exports : List CHO_export_entry
  contracts : List CHO_contract_entry
  layouts : List CHO_layout_entry
  drops : List CHO_drop_entry
  allocators : List CHO_allocator_entry
  trust_entries : List CHO_trust_entry
  proof_obligations : List CHO_proof_obligation
  payload : Option String

namespace ChimeraObjectFile

def empty (pk : CHO_payload_kind) (ptr_width : Nat) (endian : Endianness) : ChimeraObjectFile :=
  {
    header := { payload_kind := pk, target_ptr_width := ptr_width, target_endian := endian, timestamp := 0 }
    imports := []
    exports := []
    contracts := []
    layouts := []
    drops := []
    allocators := []
    trust_entries := []
    proof_obligations := []
    payload := none
  }

def has_proofs (c : ChimeraObjectFile) : Bool :=
  !c.proof_obligations.isEmpty

def count_by_status (c : ChimeraObjectFile) (s : ProofStatus) : Nat :=
  c.proof_obligations.filter (fun o => o.status == s) |>.length

end ChimeraObjectFile

namespace CHO_payload_kind

def display_name : CHO_payload_kind → String
  | .object => "ChimeraIR Object"
  | .bitcode => "LLVM Bitcode"
  | .archive => "Static Archive"
  | .generated_wrapper => "Generated Wrapper Source"
  | .metadata_only => "Metadata Only"
  | .proof_carrying => "Proof-Carrying Object"

def has_metadata : CHO_payload_kind → Bool
  | .metadata_only => true
  | .proof_carrying => true
  | _ => false

end CHO_payload_kind

namespace CHO_header

def WellFormed (h : CHO_header) : Prop :=
  h.magic = "CHIMERA_OBJ" ∧
  h.version ≥ 1 ∧
  h.target_ptr_width ∈ [32, 64] ∧
  (h.target_endian = .little ∨ h.target_endian = .big)

theorem empty_header_well_formed : WellFormed { payload_kind := .object, target_ptr_width := 64, target_endian := .little, timestamp := 0 } := by
  simp [WellFormed]

end CHO_header

end Chimera.Metadata
