-- ChimeraProof Theorems: Metadata Models Test
-- Tests for .cho and .chproof models.

import Chimera.Metadata.CHO
import Chimera.Metadata.CHProof
import Chimera.Wrapper.AST

namespace Chimera.Metadata

-- ========== CHO tests ==========

namespace CHO_payload_kind_test

theorem display_name_object :
  CHO_payload_kind.display_name .object = "ChimeraIR Object" := by rfl

theorem display_name_bitcode :
  CHO_payload_kind.display_name .bitcode = "LLVM Bitcode" := by rfl

theorem display_name_archive :
  CHO_payload_kind.display_name .archive = "Static Archive" := by rfl

theorem display_name_wrapper :
  CHO_payload_kind.display_name .generated_wrapper = "Generated Wrapper Source" := by rfl

theorem display_name_meta :
  CHO_payload_kind.display_name .metadata_only = "Metadata Only" := by rfl

theorem display_name_proof :
  CHO_payload_kind.display_name .proof_carrying = "Proof-Carrying Object" := by rfl

theorem has_metadata_proof :
  CHO_payload_kind.has_metadata .proof_carrying = true := by rfl

theorem no_metadata_object :
  CHO_payload_kind.has_metadata .object = false := by rfl

end CHO_payload_kind_test

namespace CHO_header_test

theorem well_formed_valid :
  CHO_header.WellFormed ⟨"CHIMERA_OBJ", 1, .object, 64, .little, 0⟩ := by
  simp [CHO_header.WellFormed]

theorem well_formed_wasm32 :
  CHO_header.WellFormed ⟨"CHIMERA_OBJ", 1, .object, 32, .little, 0⟩ := by
    simp [CHO_header.WellFormed]

theorem well_formed_big_endian :
  CHO_header.WellFormed ⟨"CHIMERA_OBJ", 1, .object, 64, .big, 0⟩ := by
    simp [CHO_header.WellFormed]

theorem not_well_formed_bad_magic :
  ¬ CHO_header.WellFormed ⟨"INVALID", 1, .object, 64, .little, 0⟩ := by
    simp [CHO_header.WellFormed]

theorem not_well_formed_zero_version :
  ¬ CHO_header.WellFormed ⟨"CHIMERA_OBJ", 0, .object, 64, .little, 0⟩ := by
    simp [CHO_header.WellFormed]

theorem not_well_formed_bad_ptr_width :
  ¬ CHO_header.WellFormed ⟨"CHIMERA_OBJ", 1, .object, 128, .little, 0⟩ := by
    simp [CHO_header.WellFormed]

end CHO_header_test

namespace ChimeraObjectFile_test

theorem empty_object_64 :
  let f := ChimeraObjectFile.empty .object 64 .little
  f.header.payload_kind = .object := by rfl

theorem empty_object_64_well_formed :
  let f := ChimeraObjectFile.empty .object 64 .little
  CHO_header.WellFormed f.header := by
    simp [ChimeraObjectFile.empty, CHO_header.WellFormed]

theorem has_proofs_empty :
  ¬ (ChimeraObjectFile.empty .object 64 .little).has_proofs := by
    simp [ChimeraObjectFile.empty, ChimeraObjectFile.has_proofs]

theorem count_by_status_empty :
  (ChimeraObjectFile.empty .object 64 .little).count_by_status .proved = 0 := by
    simp [ChimeraObjectFile.empty, ChimeraObjectFile.count_by_status]

end ChimeraObjectFile_test

-- ========== CHProof tests ==========

namespace ProofObligationKind_test

theorem display_layout :
  ProofObligationKind.display_name .layout = "Layout" := by rfl

theorem display_signature :
  ProofObligationKind.display_name .signature = "Signature" := by rfl

theorem display_ownership :
  ProofObligationKind.display_name .ownership = "Ownership" := by rfl

theorem display_allocator :
  ProofObligationKind.display_name .allocator = "Allocator" := by rfl

theorem display_result :
  ProofObligationKind.display_name .result = "Result Bridge" := by rfl

theorem display_panic :
  ProofObligationKind.display_name .panic = "Panic Boundary" := by rfl

theorem display_effects :
  ProofObligationKind.display_name .effects = "Effects" := by rfl

theorem display_wrappers :
  ProofObligationKind.display_name .wrappers = "Wrapper Generation" := by rfl

theorem display_link :
  ProofObligationKind.display_name .link = "Link" := by rfl

end ProofObligationKind_test

namespace ProofReport_test

theorem empty_report_all_proved :
  let r := ProofReport.empty "build1" 64 .little
  r.summary.all_proved = true := by rfl

theorem empty_report_zero_totals :
  let r := ProofReport.empty "build1" 64 .little
  r.summary.total_obligations = 0 := by rfl

theorem empty_report_no_trusted :
  let r := ProofReport.empty "build1" 64 .little
  r.summary.has_trusted = false := by rfl

end ProofReport_test

namespace ProofCertificate_test

theorem proved_is_proved :
  let cert := ProofCertificate.mk .layout ⟨"", "test"⟩ "desc" .proved [] [] false
  cert.is_proved = true := by rfl

theorem assumed_not_proved :
  let cert := ProofCertificate.mk .layout ⟨"", "test"⟩ "desc" .assumed [] [] false
  cert.is_proved = false := by rfl

theorem trusted_relies_on_trust :
  let cert := ProofCertificate.mk .layout ⟨"", "test"⟩ "desc" .trusted [] [] true
  cert.relies_on_trust = true := by rfl

theorem assumed_relies_on_trust :
  let cert := ProofCertificate.mk .layout ⟨"", "test"⟩ "desc" .assumed ["asm1"] [] false
  cert.relies_on_trust = true := by rfl

theorem proved_no_trust :
  let cert := ProofCertificate.mk .layout ⟨"", "test"⟩ "desc" .proved [] [] false
  cert.relies_on_trust = false := by rfl

end ProofCertificate_test

end Metadata
