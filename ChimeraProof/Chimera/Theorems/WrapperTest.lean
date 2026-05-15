-- ChimeraProof Tests: Wrapper Tests
-- Concrete wrapper-generator properties over the current wrapper AST.

import Chimera.Foundation
import Chimera.ABI
import Chimera.Wrapper
import Chimera.Wrapper.AST
import Chimera.Wrapper.Generator

namespace Chimera.Test

private def sampleContract : FunctionContract :=
  {
    symbol := ⟨"", "sample_wrapper"⟩
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
    safety := .generatedWrapper
    allocator := none
    requiresDrop := false
    trust := .proofObligation
    errorDomain := none
  }

private def secondContract : FunctionContract :=
  { sampleContract with
    symbol := ⟨"", "second_wrapper"⟩
  }

private def fallibleDropContract : FunctionContract :=
  { sampleContract with
    symbol := ⟨"", "fallible_wrapper"⟩
    language := .zig
    form := .fallible
    effects := [.mayError, .mayPanic, .mayDealloc]
    panicPolicy := .catchUnwind
    requiresDrop := true
    errorDomain := some { domainName := "demo.error", codes := [1, 2] }
  }

namespace WrapperASTTest

theorem comment_stmt_is_comment :
  Chimera.Wrapper.WrapperStmt.isComment (.c (.comment "hello")) = true := by
  rfl

theorem comment_stmt_text_round_trips :
  Chimera.Wrapper.WrapperStmt.getComment? (.zig (.comment "cleanup would go here")) = some "cleanup would go here" := by
  rfl

end WrapperASTTest

namespace WrapperGeneratorTest

theorem c_wrapper_has_five_stmts :
  (Chimera.Wrapper.generateCWrapper sampleContract).stmts.length = 5 := by
  native_decide

theorem rust_wrapper_preserves_symbol :
  (Chimera.Wrapper.generateRustWrapper sampleContract).name = sampleContract.symbol := by
  rfl

theorem zig_wrapper_is_not_empty :
  (Chimera.Wrapper.generateZigWrapper sampleContract).isEmpty = false := by
  native_decide

theorem c_wrapper_starts_with_doc_comment :
  (Chimera.Wrapper.generateCWrapper sampleContract).stmts.head?.bind Chimera.Wrapper.WrapperStmt.getComment? =
    some "C wrapper for sample_wrapper" := by
  rfl

end WrapperGeneratorTest

namespace WrapperCorrectnessTest

theorem c_wrapper_module_tracks_contract_count :
  (Chimera.Wrapper.generateWrapperModule [sampleContract, sampleContract] .c).functions.length = 2 := by
  native_decide

theorem rust_wrapper_module_disables_runtime :
  (Chimera.Wrapper.generateWrapperModule [sampleContract] .rust).includesRuntime = false := by
  rfl

theorem zig_wrapper_module_targets_zig :
  (Chimera.Wrapper.generateWrapperModule [sampleContract] .zig).targetLanguage = .zig := by
  rfl

theorem rendered_rust_wrapper_mentions_symbol :
  let rendered :=
    (Chimera.Wrapper.generateRustWrapper sampleContract).stmts.map Wrapper.renderWrapperStmt |>.foldl (· ++ ·) ""
  rendered.contains "sample_wrapper" = true := by
  native_decide

theorem c_wrapper_module_preserves_symbol_order :
  (Chimera.Wrapper.generateWrapperModule [sampleContract, secondContract] .c).functions.map (·.name.fqn) =
    [sampleContract.symbol.fqn, secondContract.symbol.fqn] := by
  native_decide

theorem rust_wrapper_module_preserves_symbol_order :
  (Chimera.Wrapper.generateWrapperModule [sampleContract, secondContract] .rust).functions.map (·.name.fqn) =
    [sampleContract.symbol.fqn, secondContract.symbol.fqn] := by
  native_decide

theorem zig_wrapper_module_preserves_symbol_order :
  (Chimera.Wrapper.generateWrapperModule [sampleContract, secondContract] .zig).functions.map (·.name.fqn) =
    [sampleContract.symbol.fqn, secondContract.symbol.fqn] := by
  native_decide

theorem generated_wrapper_module_functions_are_non_empty :
  let generated := Chimera.Wrapper.generateWrapperModule [sampleContract, secondContract] .c
  generated.functions.all (fun f => f.isEmpty = false) = true := by
  native_decide

theorem c_wrapper_correctness_surface_holds :
  Chimera.Wrapper.WrapperCorrect sampleContract .c := by
  exact Chimera.Wrapper.generated_wrapper_correct sampleContract .c

theorem rust_wrapper_correctness_surface_holds :
  Chimera.Wrapper.WrapperCorrect fallibleDropContract .rust := by
  exact Chimera.Wrapper.generated_wrapper_correct fallibleDropContract .rust

theorem zig_wrapper_correctness_surface_holds :
  Chimera.Wrapper.WrapperCorrect fallibleDropContract .zig := by
  exact Chimera.Wrapper.generated_wrapper_correct fallibleDropContract .zig

theorem wrapper_correctness_preserves_result_panic_and_drop_contract :
  let wrapper := Chimera.Wrapper.generateWrapper fallibleDropContract .zig
  wrapper.contract.form = .fallible ∧
    wrapper.contract.panicPolicy = .catchUnwind ∧
    wrapper.contract.requiresDrop = true ∧
    wrapper.contract.errorDomain = fallibleDropContract.errorDomain := by
  rfl

theorem wrapper_module_correctness_holds_for_each_backend :
  (Chimera.Wrapper.generateWrapperModule [sampleContract, fallibleDropContract] .c).functions.all
      (fun wrapper => decide (Chimera.Wrapper.WrapperCorrect wrapper.contract .c)) = true ∧
    (Chimera.Wrapper.generateWrapperModule [sampleContract, fallibleDropContract] .rust).functions.all
      (fun wrapper => decide (Chimera.Wrapper.WrapperCorrect wrapper.contract .rust)) = true ∧
    (Chimera.Wrapper.generateWrapperModule [sampleContract, fallibleDropContract] .zig).functions.all
      (fun wrapper => decide (Chimera.Wrapper.WrapperCorrect wrapper.contract .zig)) = true := by
  native_decide

end WrapperCorrectnessTest

end Chimera.Test
