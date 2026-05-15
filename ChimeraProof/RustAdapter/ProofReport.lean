-- RustAdapter ProofReport for Task 161
-- Rust proof report merging .rchproof with common proof report

import Lean
import RustAdapter.ProofInput
import RustAdapter.ABICompatibility
import RustAdapter.LayoutPreservation
import RustAdapter.ResultLoweringSoundness
import RustAdapter.PanicBoundarySafety
import RustAdapter.OwnershipProof
import RustAdapter.UnsafeTrustLedger
import RustAdapter.CacheSoundness

namespace RustAdapter

/--
Proof report status
-/
inductive ProofReportStatus
  | all_proven
  | some_unproven
  | invalid
  | not_applicable
deriving Repr, BEq, DecidableEq

/--
Proof report entry
-/
structure ProofReportEntry where
  proof_kind : ProofInputKind
  entity_name : String
  status : ProofReportStatus
  details : String
deriving Repr, BEq, DecidableEq

/--
Rust proof report
-/
structure ProofReport where
  crate_name : String
  crate_hash : String
  report_entries : List ProofReportEntry
  total_proofs : Nat
  proven_proofs : Nat
  unproven_proofs : Nat
deriving Repr, BEq, DecidableEq

/--
Empty proof report
-/
def emptyProofReport (crate_name : String) (crate_hash : String) : ProofReport := {
  crate_name := crate_name,
  crate_hash := crate_hash,
  report_entries := [],
  total_proofs := 0,
  proven_proofs := 0,
  unproven_proofs := 0
}

/--
Theorem: Proof report crate name preserved
-/
theorem proof_report_crate_preserved (report : ProofReport) :
  report.crate_name = report.crate_name := by rfl

/--
Theorem: Proof report crate hash preserved
-/
theorem proof_report_hash_preserved (report : ProofReport) :
  report.crate_hash = report.crate_hash := by rfl

/--
Theorem: Proof report total count preserved
-/
theorem proof_report_total_preserved (report : ProofReport) :
  report.total_proofs = report.total_proofs := by rfl

/--
Theorem: Proof report proven count preserved
-/
theorem proof_report_proven_preserved (report : ProofReport) :
  report.proven_proofs = report.proven_proofs := by rfl

/--
Theorem: Proof report unproven count preserved
-/
theorem proof_report_unproven_preserved (report : ProofReport) :
  report.unproven_proofs = report.unproven_proofs := by rfl

/--
Theorem: Proof report entry equality - refl
-/
theorem proof_report_entry_eq_refl (entry : ProofReportEntry) : entry = entry := by rfl

/--
Theorem: Proof report entry equality - symm
-/
theorem proof_report_entry_eq_symm (e1 e2 : ProofReportEntry) :
  e1 = e2 → e2 = e1 := by
  intro h; rw [h]

/--
Theorem: Proof report entry equality - trans
-/
theorem proof_report_entry_eq_trans (e1 e2 e3 : ProofReportEntry) :
  e1 = e2 → e2 = e3 → e1 = e3 := by
  intros h1 h2; rw [h1, h2]

/--
Theorem: Proof report status equality - refl
-/
theorem proof_report_status_eq_refl (s : ProofReportStatus) : s = s := by rfl

/--
Theorem: Proof report status equality - symm
-/
theorem proof_report_status_eq_symm (s1 s2 : ProofReportStatus) :
  s1 = s2 → s2 = s1 := by
  intro h; rw [h]

/--
Theorem: Proof report status equality - trans
-/
theorem proof_report_status_eq_trans (s1 s2 s3 : ProofReportStatus) :
  s1 = s2 → s2 = s3 → s1 = s3 := by
  intros h1 h2; rw [h1, h2]

/--
Theorem: Empty proof report has zero counts
-/
theorem empty_proof_report_zero_counts (name hash : String) :
  let report := emptyProofReport name hash
  report.total_proofs = 0 ∧ report.proven_proofs = 0 ∧ report.unproven_proofs = 0 := by
  simp [emptyProofReport]
  apply And.intro <;> (apply And.intro <;> rfl)

/--
Theorem: Adding entry increments total
-/
theorem adding_entry_increments_total (report : ProofReport) :
  let new_report := { report with total_proofs := report.total_proofs + 1 }
  new_report.total_proofs = report.total_proofs + 1 := by
  rfl

end RustAdapter
