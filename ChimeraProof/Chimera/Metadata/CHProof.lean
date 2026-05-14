-- ChimeraProof Metadata: CHProof
-- Proof certificate schema (.chproof) model.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Contract
import Chimera.Metadata.Schema
import Chimera.Metadata.CHO
import Chimera.Wrapper.AST

namespace Chimera.Metadata

inductive ProofObligationKind
  | layout
  | signature
  | ownership
  | allocator
  | result
  | panic
  | effects
  | wrappers
  | link
deriving Repr, BEq

structure ProofCertificate where
  kind : ProofObligationKind
  target : Symbol
  description : String
  status : ProofStatus
  assumptions : List String
  evidence : List String
  trusted : Bool
deriving Repr, BEq

structure ProofTrustAssumption where
  kind : TrustAssumptionKind
  description : String
  external_ref : Option String
  trusted : Bool
deriving Repr, BEq

structure ProofModuleEntry where
  module_name : Symbol
  abi_version : Nat
  language : SourceLanguage
  obligations : List ProofCertificate
  trust_assumptions : List ProofTrustAssumption
deriving Repr, BEq

structure FieldProof where
  field_name : String
  offset : Nat
  offset_proved : Bool
  size : Nat
  size_proved : Bool
  align : Nat
  align_proved : Bool
deriving Repr, BEq

structure LayoutProof where
  type_name : String
  size : Nat
  align : Nat
  field_proofs : List FieldProof
deriving Repr, BEq

structure SignatureProof where
  semantic_sig : SemanticSignature
  physical_sig : PhysicalSignature
  compatible : Bool
  calling_convention_matched : Bool
deriving Repr, BEq

structure OwnershipProof where
  no_double_own : Bool
  no_write_alias : Bool
  borrow_exclusive : Bool
deriving Repr, BEq

structure AllocatorProof where
  registered : Bool
  unique : Bool
  matches_drop : Bool
deriving Repr, BEq

structure ResultProof where
  ok_status_nonzero : Bool
  err_has_payload : Bool
  no_false_ok : Bool
deriving Repr, BEq

structure PanicProof where
  no_unwind : Bool
  abort_correct : Bool
  catch_correct : Bool
deriving Repr, BEq

structure EffectProof where
  inferred_set : List Effect
  declared_set : List Effect
  subset_proved : Bool
deriving Repr, BEq

structure WrapperProof where
  wrapper_language : Chimera.Wrapper.WrapperLanguage
  contract_symbol : Symbol
  body_generated : Bool
  renderer_verified : Bool
deriving Repr, BEq

structure LinkProof where
  symbols_resolved : Bool
  no_duplicate_strong : Bool
  targets_compatible : Bool
  signatures_compatible : Bool
deriving Repr, BEq

structure ProofReportSummary where
  total_obligations : Nat
  obligations_proved : Nat
  obligations_assumed : Nat
  obligations_trusted : Nat
  obligations_unsupported : Nat
  all_proved : Bool
  has_trusted : Bool
deriving Repr, BEq

structure ProofReport where
  build_id : String
  timestamp : Nat
  target_ptr_width : Nat
  target_endian : Endianness
  modules : List ProofModuleEntry
  summary : ProofReportSummary
deriving Repr, BEq

namespace ProofObligationKind

def display_name : ProofObligationKind → String
  | .layout => "Layout"
  | .signature => "Signature"
  | .ownership => "Ownership"
  | .allocator => "Allocator"
  | .result => "Result Bridge"
  | .panic => "Panic Boundary"
  | .effects => "Effects"
  | .wrappers => "Wrapper Generation"
  | .link => "Link"

end ProofObligationKind

namespace ProofReport

def compute_summary (r : ProofReport) : ProofReportSummary :=
  let total := r.modules.foldl (fun acc m => acc + m.obligations.length) 0
  let proved := r.modules.foldl (fun acc m =>
    acc + (m.obligations.filter (fun o => o.status == ProofStatus.proved)).length) 0
  let assumed := r.modules.foldl (fun acc m =>
    acc + (m.obligations.filter (fun o => o.status == ProofStatus.assumed)).length) 0
  let trusted := r.modules.foldl (fun acc m =>
    acc + (m.obligations.filter (fun o => o.status == ProofStatus.trusted)).length) 0
  let unsupported := r.modules.foldl (fun acc m =>
    acc + (m.obligations.filter (fun o => o.status == ProofStatus.unsupported)).length) 0
  {
    total_obligations := total
    obligations_proved := proved
    obligations_assumed := assumed
    obligations_trusted := trusted
    obligations_unsupported := unsupported
    all_proved := r.modules.all (fun m => m.obligations.all (fun o => o.status == ProofStatus.proved))
    has_trusted := r.modules.any (fun m => m.trust_assumptions.any (fun a => a.trusted))
  }

def empty (build_id : String) (ptr_width : Nat) (endian : Endianness) : ProofReport :=
  {
    build_id := build_id
    timestamp := 0
    target_ptr_width := ptr_width
    target_endian := endian
    modules := []
    summary := {
      total_obligations := 0
      obligations_proved := 0
      obligations_assumed := 0
      obligations_trusted := 0
      obligations_unsupported := 0
      all_proved := true
      has_trusted := false
    }
  }

end ProofReport

namespace ProofCertificate

def is_proved (c : ProofCertificate) : Bool :=
  c.status == ProofStatus.proved

def relies_on_trust (c : ProofCertificate) : Bool :=
  c.trusted || !c.assumptions.isEmpty

end ProofCertificate

end Chimera.Metadata
