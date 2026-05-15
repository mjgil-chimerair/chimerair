-- ChimeraProof Theorems: ProofReport Test
-- Tests for proof report generation.

import Chimera.Metadata.CHO
import Chimera.Metadata.CHProof
import Chimera.Metadata.ProofReport
import Chimera.Checkers.FullChecker
import Chimera.IR.Module

namespace Chimera.Metadata

namespace proofStatusToString_test

theorem proved :
  proofStatusToString .proved = "Proved" := by rfl

theorem assumed :
  proofStatusToString .assumed = "Assumed" := by rfl

theorem trusted :
  proofStatusToString .trusted = "Trusted" := by rfl

theorem implemented :
  proofStatusToString .implemented = "Implemented" := by rfl

theorem tested :
  proofStatusToString .tested = "Tested" := by rfl

theorem unsupported :
  proofStatusToString .unsupported = "Unsupported" := by rfl

end proofStatusToString_test

namespace obligationKindToString_test

theorem layout :
  obligationKindToString .layout = "Layout" := by rfl

theorem signature :
  obligationKindToString .signature = "Signature" := by rfl

theorem ownership :
  obligationKindToString .ownership = "Ownership" := by rfl

theorem allocator :
  obligationKindToString .allocator = "Allocator" := by rfl

theorem result :
  obligationKindToString .result = "Result Bridge" := by rfl

theorem panic :
  obligationKindToString .panic = "Panic Boundary" := by rfl

theorem effects :
  obligationKindToString .effects = "Effects" := by rfl

theorem wrappers :
  obligationKindToString .wrappers = "Wrapper Generation" := by rfl

theorem link :
  obligationKindToString .link = "Link" := by rfl

end obligationKindToString_test

namespace ProofReport_test

private def sampleContract
  (symName : String)
  (safety : SafetyClass)
  (trust : TrustAssumption) : FunctionContract :=
  {
    symbol := ⟨"", symName⟩
    language := .c
    form := .infallible
    semanticSig := {
      params := []
      returns := .unit
      isVarargs := false
    }
    physicalSig := {
      params := []
      returns := .void
      callingConv := .cdecl
    }
    effects := [.pure]
    panicPolicy := .forbidden
    safety := safety
    allocator := none
    requiresDrop := false
    trust := trust
    errorDomain := none
  }

private def sampleModule (contract : FunctionContract) : Module :=
  {
    abiVersion := "0.1"
    moduleName := ⟨"", "sample_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := contract.symbol, contract := contract }]
    imports := []
    types := []
    layouts := []
  }

private def sampleValidModule
  (exportContract : FunctionContract)
  (importContract : FunctionContract) : Module :=
  {
    abiVersion := "0.1"
    moduleName := ⟨"", "sample_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := exportContract.symbol, contract := exportContract }]
    imports := [{ symbol := importContract.symbol, contract := importContract }]
    types := []
    layouts := []
  }

private def sampleModuleWithLayoutAndImport
  (exportContract : FunctionContract)
  (importContract : FunctionContract) : Module :=
  {
    abiVersion := "0.1"
    moduleName := ⟨"", "sample_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := exportContract.symbol, contract := exportContract }]
    imports := [{ symbol := importContract.symbol, contract := importContract }]
    types := []
    layouts := [{
      name := ⟨"", "sample_layout"⟩
      size := 16
      align := 8
      hash := 0
      fields := []
    }]
  }

theorem empty_report_ptr_width :
  let r := ProofReport.empty "build1" 64 .little
  r.target_ptr_width = 64 := by rfl

theorem empty_report_endian :
  let r := ProofReport.empty "build1" 64 .little
  r.target_endian = .little := by rfl

theorem empty_report_build_id :
  let r := ProofReport.empty "build1" 64 .little
  r.build_id = "build1" := by rfl

theorem empty_report_modules_empty :
  let r := ProofReport.empty "build1" 64 .little
  r.modules = [] := by rfl

theorem generated_report_counts_proved_export :
  let exportContract := sampleContract "proved_fn" .verified .proofObligation
  let importContract := sampleContract "import_fn" .verified .proofObligation
  let build : CertifiedBuild := {
    modules := [sampleValidModule exportContract importContract]
    validated := true
  }
  let report := generateProofReport build "proof-build"
  report.summary.obligations_proved = 2 := by
  native_decide

theorem generated_report_emits_distinct_obligations :
  let contract : FunctionContract :=
    { sampleContract "rich_fn" .generatedWrapper .proofObligation with
      form := .constructor
      effects := [.mayAlloc, .mayDealloc]
      panicPolicy := .catchUnwind
      allocator := some ⟨"", "global_alloc"⟩
      requiresDrop := true
    }
  let build : CertifiedBuild := {
    modules := [sampleModule contract]
    validated := true
  }
  let report := generateProofReport build "rich-build"
  report.summary.total_obligations = 6 := by
  native_decide

theorem generated_report_emits_phase_specific_evidence :
  let contract : FunctionContract :=
    { sampleContract "evidence_fn" .generatedWrapper .proofObligation with
      form := .constructor
      effects := [.mayAlloc]
      panicPolicy := .catchUnwind
    }
  let report := generateProofReport { modules := [sampleModule contract], validated := true } "evidence-build"
  let evidence := (report.modules.head?.bind (fun m => m.obligations.head?.map (fun o => o.evidence))).getD []
  evidence.contains "fullCheck" && evidence.contains "contract-check" = true := by
  native_decide

theorem generated_report_records_trusted_assumption :
  let build : CertifiedBuild := {
    modules := [sampleModule (sampleContract "trusted_fn" .trustedContract .trusted)]
    validated := true
  }
  let report := generateProofReport build "trust-build"
  report.summary.has_trusted = true := by
  native_decide

theorem emitted_report_mentions_build_and_symbol :
  let build : CertifiedBuild := {
    modules := [sampleModule (sampleContract "emit_fn" .verified .proofObligation)]
    validated := true
  }
  let text := emitProofReportText (generateProofReport build "emit-build")
  text.contains "emit-build" && text.contains "emit_fn" = true := by
  native_decide

theorem emitted_report_mentions_effects_obligation :
  let contract : FunctionContract :=
    { sampleContract "effects_fn" .verified .proofObligation with
      effects := [.mayAlloc]
    }
  let text := emitProofReportText (generateProofReport { modules := [sampleModule contract], validated := true } "effects-build")
  text.contains "Effects" = true := by
  native_decide

theorem emitted_report_mentions_wrapper_obligation :
  let contract : FunctionContract :=
    { sampleContract "wrapper_fn" .generatedWrapper .proofObligation with
      effects := [.mayAlloc]
    }
  let text := emitProofReportText (generateProofReport { modules := [sampleModule contract], validated := true } "wrapper-build")
  text.contains "Wrapper Generation" = true := by
  native_decide

theorem generated_report_emits_layout_and_link_obligations :
  let exportContract := sampleContract "export_fn" .verified .proofObligation
  let importContract := sampleContract "import_fn" .verified .proofObligation
  let report := generateProofReport
    { modules := [sampleModuleWithLayoutAndImport exportContract importContract], validated := true }
    "layout-link-build"
  report.summary.total_obligations = 3 := by
  native_decide

theorem generated_report_emits_link_phase_evidence :
  let exportContract := sampleContract "export_fn" .verified .proofObligation
  let importContract := sampleContract "import_fn" .verified .proofObligation
  let report := generateProofReport
    { modules := [sampleModuleWithLayoutAndImport exportContract importContract], validated := true }
    "link-evidence-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .link))).map (·.evidence)).getD []
  evidence.contains "fullCheck" && evidence.contains "link-check" = true := by
  native_decide

theorem emitted_report_mentions_layout_and_link :
  let exportContract := sampleContract "export_fn" .verified .proofObligation
  let importContract := sampleContract "import_fn" .verified .proofObligation
  let text := emitProofReportText (generateProofReport
    { modules := [sampleModuleWithLayoutAndImport exportContract importContract], validated := true }
    "layout-link-build")
  text.contains "Layout" && text.contains "Link" && text.contains "sample_layout" && text.contains "import_fn" = true := by
  native_decide

theorem generated_report_records_layout_failure_evidence :
  let contract := sampleContract "bad_layout_fn" .verified .proofObligation
  let invalidModule : Module := {
    abiVersion := "0.1"
    moduleName := ⟨"", "bad_layout_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := contract.symbol, contract := contract }]
    imports := []
    types := []
    layouts := [{
      name := ⟨"", "bad_layout"⟩
      size := 4
      align := 8
      hash := 0
      fields := []
    }]
  }
  let report := generateProofReportForModules [invalidModule] "invalid-layout-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .layout))).map (·.evidence)).getD []
  evidence.contains "layout-failed" && evidence.contains "layout-size-below-align" = true := by
  native_decide

theorem generated_report_records_layout_alignment_failure_evidence :
  let contract := sampleContract "bad_align_fn" .verified .proofObligation
  let invalidModule : Module := {
    abiVersion := "0.1"
    moduleName := ⟨"", "bad_align_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := contract.symbol, contract := contract }]
    imports := []
    types := []
    layouts := [{
      name := ⟨"", "bad_align"⟩
      size := 16
      align := 0
      hash := 0
      fields := []
    }]
  }
  let report := generateProofReportForModules [invalidModule] "invalid-align-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .layout))).map (·.evidence)).getD []
  evidence.contains "layout-failed" && evidence.contains "invalid-layout-align" = true := by
  native_decide

theorem generated_report_for_modules_matches_valid_full_check :
  let exportContract := sampleContract "valid_export" .verified .proofObligation
  let importContract := sampleContract "valid_import" .verified .proofObligation
  let report := generateProofReportForModules
    [sampleModuleWithLayoutAndImport exportContract importContract]
    "valid-full-check-build"
  report.summary.total_obligations = 3 := by
  native_decide

theorem generated_report_for_modules_marks_link_failure_unsupported :
  let exportContract := sampleContract "export_fn" .verified .proofObligation
  let importContract := sampleContract "import_fn" .verified .proofObligation
  let primary := sampleModuleWithLayoutAndImport exportContract importContract
  let secondary : Module := {
    abiVersion := "0.1"
    moduleName := ⟨"", "wasm_mod"⟩
    language := .c
    target := Target.wasm32
    exports := []
    imports := []
    types := []
    layouts := []
  }
  let report := generateProofReportForModules [primary, secondary] "invalid-link-build"
  let status :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .link))).map (·.status)).getD .proved
  status == .unsupported = true := by
  native_decide

theorem generated_report_for_modules_marks_failed_checker_evidence :
  let exportContract := sampleContract "export_fn" .verified .proofObligation
  let importContract := sampleContract "import_fn" .verified .proofObligation
  let primary := sampleModuleWithLayoutAndImport exportContract importContract
  let secondary : Module := {
    abiVersion := "0.1"
    moduleName := ⟨"", "wasm_mod"⟩
    language := .c
    target := Target.wasm32
    exports := []
    imports := []
    types := []
    layouts := []
  }
  let report := generateProofReportForModules [primary, secondary] "invalid-link-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .link))).map (·.evidence)).getD []
  evidence.contains "checker-failed" && evidence.contains "link-check" && evidence.contains "target-mismatch" = true := by
  native_decide

theorem generated_report_records_metadata_failure_evidence :
  let invalidContract := sampleContract "invalid_meta_fn" .verified .proofObligation
  let invalidModule : Module := {
    abiVersion := "0.2"
    moduleName := ⟨"", "invalid_meta_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := invalidContract.symbol, contract := invalidContract }]
    imports := []
    types := []
    layouts := []
  }
  let report := generateProofReportForModules [invalidModule] "invalid-metadata-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .signature))).map (·.evidence)).getD []
  evidence.contains "metadata-failed" && evidence.contains "invalid-version" = true := by
  native_decide

theorem generated_report_records_empty_import_symbol_metadata_evidence :
  let exportContract := sampleContract "export_fn" .verified .proofObligation
  let importContract := sampleContract "import_fn" .verified .proofObligation
  let invalidModule : Module := {
    abiVersion := "0.1"
    moduleName := ⟨"", "bad_import_symbol_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := exportContract.symbol, contract := exportContract }]
    imports := [{ symbol := ⟨"", ""⟩, contract := importContract }]
    types := []
    layouts := []
  }
  let report := generateProofReportForModules [invalidModule] "invalid-import-symbol-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .signature))).map (·.evidence)).getD []
  evidence.contains "metadata-failed" && evidence.contains "empty-import-symbol" = true := by
  native_decide

theorem generated_report_records_duplicate_layout_metadata_evidence :
  let contract := sampleContract "layout_dup_fn" .verified .proofObligation
  let sharedLayout : DeclaredLayout := {
    name := ⟨"", "dup_layout"⟩
    size := 16
    align := 8
    hash := 0
    fields := []
  }
  let invalidModule : Module := {
    abiVersion := "0.1"
    moduleName := ⟨"", "dup_layout_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := contract.symbol, contract := contract }]
    imports := [{ symbol := ⟨"", "import_fn"⟩, contract := sampleContract "import_fn" .verified .proofObligation }]
    types := []
    layouts := [sharedLayout, sharedLayout]
  }
  let report := generateProofReportForModules [invalidModule] "duplicate-layout-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .signature))).map (·.evidence)).getD []
  evidence.contains "metadata-failed" && evidence.contains "duplicate-layout" = true := by
  native_decide

theorem generated_report_records_empty_layout_name_metadata_evidence :
  let contract := sampleContract "layout_name_fn" .verified .proofObligation
  let invalidModule : Module := {
    abiVersion := "0.1"
    moduleName := ⟨"", "empty_layout_name_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := contract.symbol, contract := contract }]
    imports := [{ symbol := ⟨"", "import_fn"⟩, contract := sampleContract "import_fn" .verified .proofObligation }]
    types := []
    layouts := [{
      name := ⟨"", ""⟩
      size := 16
      align := 8
      hash := 0
      fields := []
    }]
  }
  let report := generateProofReportForModules [invalidModule] "empty-layout-name-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .signature))).map (·.evidence)).getD []
  evidence.contains "metadata-failed" && evidence.contains "empty-layout-name" = true := by
  native_decide

theorem generated_report_records_export_mismatch_metadata_evidence :
  let exportContract := sampleContract "contract_name" .verified .proofObligation
  let importContract := sampleContract "import_fn" .verified .proofObligation
  let invalidModule : Module := {
    abiVersion := "0.1"
    moduleName := ⟨"", "export_mismatch_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := ⟨"", "export_name"⟩, contract := exportContract }]
    imports := [{ symbol := importContract.symbol, contract := importContract }]
    types := []
    layouts := []
  }
  let report := generateProofReportForModules [invalidModule] "export-mismatch-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .signature))).map (·.evidence)).getD []
  evidence.contains "metadata-failed" && evidence.contains "export-mismatch" = true := by
  native_decide

theorem generated_report_records_import_mismatch_metadata_evidence :
  let exportContract := sampleContract "export_fn" .verified .proofObligation
  let importContract := sampleContract "contract_name" .verified .proofObligation
  let invalidModule : Module := {
    abiVersion := "0.1"
    moduleName := ⟨"", "import_mismatch_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := exportContract.symbol, contract := exportContract }]
    imports := [{ symbol := ⟨"", "import_name"⟩, contract := importContract }]
    types := []
    layouts := []
  }
  let report := generateProofReportForModules [invalidModule] "import-mismatch-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .signature))).map (·.evidence)).getD []
  evidence.contains "metadata-failed" && evidence.contains "import-mismatch" = true := by
  native_decide

theorem generated_report_records_contract_failure_evidence :
  let invalidContract : FunctionContract :=
    { sampleContract "alloc_fn" .verified .proofObligation with
      effects := [.mayAlloc]
    }
  let report := generateProofReportForModules [sampleModule invalidContract] "invalid-contract-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .signature))).map (·.evidence)).getD []
  evidence.contains "contract-failed" && evidence.contains "missing-allocator" = true := by
  native_decide

theorem generated_report_records_result_failure_evidence :
  let invalidContract : FunctionContract :=
    { sampleContract "result_fn" .verified .proofObligation with
      errorDomain := some .rustPanic
    }
  let report := generateProofReportForModules [sampleModule invalidContract] "invalid-result-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .result))).map (·.evidence)).getD []
  evidence.contains "result-failed" && evidence.contains "invalid-error-domain" = true := by
  native_decide

theorem generated_report_records_ownership_failure_evidence :
  let invalidContract : FunctionContract :=
    { sampleContract "ownership_fn" .verified .proofObligation with
      form := .constructor
      semanticSig := {
        params := []
        returns := .borrow .i32 .call
        isVarargs := false
      }
    }
  let report := generateProofReportForModules [sampleModule invalidContract] "invalid-ownership-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .ownership))).map (·.evidence)).getD []
  evidence.contains "ownership-failed" && evidence.contains "call-lifetime-return" = true := by
  native_decide

theorem generated_report_records_effects_failure_evidence :
  let invalidContract : FunctionContract :=
    { sampleContract "effects_fn" .verified .proofObligation with
      semanticSig := {
        params := []
        returns := .owned .i32
        isVarargs := false
      }
      effects := [.mayAlloc]
      allocator := some ⟨"", "global_alloc"⟩
    }
  let report := generateProofReportForModules [sampleModule invalidContract] "invalid-effects-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .effects))).map (·.evidence)).getD []
  evidence.contains "effects-failed" && evidence.contains "underdeclared-effects" = true := by
  native_decide

theorem generated_report_records_allocator_failure_evidence :
  let invalidContract : FunctionContract :=
    { sampleContract "allocator_fn" .verified .proofObligation with
      semanticSig := {
        params := []
        returns := .owned (.opaque ⟨"", "OpaqueHandle"⟩)
        isVarargs := false
      }
      allocator := some ⟨"", "global_alloc"⟩
      requiresDrop := true
    }
  let report := generateProofReportForModules [sampleModule invalidContract] "invalid-allocator-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .allocator))).map (·.evidence)).getD []
  evidence.contains "allocator-failed" && evidence.contains "no-drop-function" = true := by
  native_decide

theorem generated_report_records_wrapper_generation_evidence :
  let wrapperContract : FunctionContract :=
    { sampleContract "wrapper_fn" .generatedWrapper .proofObligation with
      effects := [.mayAlloc]
      allocator := some ⟨"", "global_alloc"⟩
    }
  let report := generateProofReportForModules [sampleModule wrapperContract] "wrapper-evidence-build"
  let evidence :=
    ((report.modules.head?.bind (fun m =>
      m.obligations.find? (fun obligation => obligation.kind == .wrappers))).map (·.evidence)).getD []
  evidence.contains "wrapper-generated" && evidence.contains "wrapper-non-empty" = true := by
  native_decide

theorem generated_report_records_trusted_import_assumption :
  let exportContract := sampleContract "export_fn" .verified .proofObligation
  let importContract := sampleContract "trusted_import" .trustedContract .trusted
  let report := generateProofReportForModules
    [sampleModuleWithLayoutAndImport exportContract importContract]
    "trusted-import-build"
  let assumptions := (report.modules.head?.map (·.trust_assumptions)).getD []
  assumptions.any (fun assumption =>
    assumption.kind == .trustedLinker &&
      assumption.description.contains "trusted import contract trusted_import") = true := by
  native_decide

theorem emitted_report_mentions_trusted_import_assumption :
  let exportContract := sampleContract "export_fn" .verified .proofObligation
  let importContract := sampleContract "trusted_import" .trustedContract .trusted
  let text := emitProofReportText (generateProofReportForModules
    [sampleModuleWithLayoutAndImport exportContract importContract]
    "trusted-import-build")
  text.contains "Trusted Linker" && text.contains "trusted import contract trusted_import" = true := by
  native_decide

theorem emitted_report_mentions_obligation_evidence :
  let invalidContract : FunctionContract :=
    { sampleContract "alloc_fn" .verified .proofObligation with
      effects := [.mayAlloc]
    }
  let text := emitProofReportText (generateProofReportForModules
    [sampleModule invalidContract]
    "evidence-text-build")
  text.contains "evidence: checker-failed, fullCheck, metadata-check, contract-check, contract-failed, missing-allocator" = true := by
  native_decide

theorem emitted_report_mentions_duplicate_layout_metadata_evidence :
  let contract := sampleContract "layout_dup_fn" .verified .proofObligation
  let sharedLayout : DeclaredLayout := {
    name := ⟨"", "dup_layout"⟩
    size := 16
    align := 8
    hash := 0
    fields := []
  }
  let invalidModule : Module := {
    abiVersion := "0.1"
    moduleName := ⟨"", "dup_layout_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := contract.symbol, contract := contract }]
    imports := [{ symbol := ⟨"", "import_fn"⟩, contract := sampleContract "import_fn" .verified .proofObligation }]
    types := []
    layouts := [sharedLayout, sharedLayout]
  }
  let text := emitProofReportText (generateProofReportForModules [invalidModule] "duplicate-layout-build")
  text.contains "duplicate-layout" && text.contains "dup_layout" = true := by
  native_decide

theorem emitted_report_mentions_invalid_layout_align_metadata_evidence :
  let contract := sampleContract "bad_align_fn" .verified .proofObligation
  let invalidModule : Module := {
    abiVersion := "0.1"
    moduleName := ⟨"", "bad_align_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := contract.symbol, contract := contract }]
    imports := [{ symbol := ⟨"", "import_fn"⟩, contract := sampleContract "import_fn" .verified .proofObligation }]
    types := []
    layouts := [{
      name := ⟨"", "bad_align"⟩
      size := 16
      align := 0
      hash := 0
      fields := []
    }]
  }
  let text := emitProofReportText (generateProofReportForModules [invalidModule] "invalid-align-build")
  text.contains "invalid-layout-align" && text.contains "bad_align" = true := by
  native_decide

theorem emitted_report_mentions_layout_size_below_align_metadata_evidence :
  let contract := sampleContract "bad_layout_fn" .verified .proofObligation
  let invalidModule : Module := {
    abiVersion := "0.1"
    moduleName := ⟨"", "bad_layout_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := contract.symbol, contract := contract }]
    imports := [{ symbol := ⟨"", "import_fn"⟩, contract := sampleContract "import_fn" .verified .proofObligation }]
    types := []
    layouts := [{
      name := ⟨"", "bad_layout"⟩
      size := 4
      align := 8
      hash := 0
      fields := []
    }]
  }
  let text := emitProofReportText (generateProofReportForModules [invalidModule] "invalid-layout-build")
  text.contains "layout-size-below-align" && text.contains "bad_layout" = true := by
  native_decide

theorem emitted_report_mentions_export_mismatch_metadata_evidence :
  let exportContract := sampleContract "contract_name" .verified .proofObligation
  let importContract := sampleContract "import_fn" .verified .proofObligation
  let invalidModule : Module := {
    abiVersion := "0.1"
    moduleName := ⟨"", "export_mismatch_mod"⟩
    language := .c
    target := Target.x86_64_linux
    exports := [{ symbol := ⟨"", "export_name"⟩, contract := exportContract }]
    imports := [{ symbol := importContract.symbol, contract := importContract }]
    types := []
    layouts := []
  }
  let text := emitProofReportText (generateProofReportForModules [invalidModule] "export-mismatch-build")
  text.contains "export-mismatch" && text.contains "export_name" = true := by
  native_decide

theorem emitted_report_mentions_obligation_assumptions :
  let trustedContract := sampleContract "trusted_export" .trustedContract .trusted
  let text := emitProofReportText (generateProofReportForModules
    [sampleModule trustedContract]
    "assumption-text-build")
  text.contains "assumptions: trusted contract accepted without full proof" = true := by
  native_decide

theorem emitted_report_summary_mentions_non_proved_counts :
  let trustedContract := sampleContract "trusted_export" .trustedContract .trusted
  let importContract := sampleContract "import_fn" .verified .proofObligation
  let text := emitProofReportText (generateProofReportForModules
    [sampleValidModule trustedContract importContract]
    "summary-text-build")
  text.contains "Summary: 1/2 proved, 1 trusted, 0 assumed, 0 unsupported, trust assumptions: 1" = true := by
  native_decide

theorem emitted_report_summary_mentions_unsupported_counts :
  let invalidContract : FunctionContract :=
    { sampleContract "alloc_fn" .verified .proofObligation with
      effects := [.mayAlloc]
    }
  let text := emitProofReportText (generateProofReportForModules
    [sampleModule invalidContract]
    "unsupported-summary-build")
  text.contains "Summary: 0/2 proved, 0 trusted, 0 assumed, 2 unsupported, trust assumptions: 0" = true := by
  native_decide

theorem emitted_report_summary_mentions_trust_assumption_count :
  let exportContract := sampleContract "export_fn" .verified .proofObligation
  let importContract := sampleContract "trusted_import" .trustedContract .trusted
  let text := emitProofReportText (generateProofReportForModules
    [sampleModuleWithLayoutAndImport exportContract importContract]
    "trust-count-build")
  text.contains "trust assumptions: 1" = true := by
  native_decide

end ProofReport_test

end Metadata
