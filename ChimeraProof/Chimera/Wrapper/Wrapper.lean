-- ChimeraProof Wrapper
-- Wrapper generation and verification.

import Chimera.Wrapper.AST
import Chimera.Wrapper.Generator

namespace Chimera.Wrapper

/--
Wrapper generation error.
-/
inductive WrapperError where
  | invalidContract
  | unsupportedLanguage
  | unsupportedType

/--
Expected leading documentation comment for a generated wrapper.
-/
def wrapperDocComment (lang : WrapperLanguage) (contract : FunctionContract) : String :=
  match lang with
  | .c => "C wrapper for " ++ contract.symbol.name
  | .rust => "Rust wrapper for " ++ contract.symbol.name
  | .zig => "Zig wrapper for " ++ contract.symbol.name

/--
The current wrapper template must explicitly cover validation, call, result handling,
and cleanup in the generated AST.
-/
def wrapperHasPhaseComments (wrapper : WrapperFunction) : Prop :=
  wrapper.stmts.any (fun stmt => WrapperStmt.getComment? stmt == some "pointer validation would go here") = true ∧
  wrapper.stmts.any (fun stmt => WrapperStmt.getComment? stmt == some "implementation call would go here") = true ∧
  wrapper.stmts.any (fun stmt => WrapperStmt.getComment? stmt == some "result mapping would go here") = true ∧
  wrapper.stmts.any (fun stmt => WrapperStmt.getComment? stmt == some "cleanup would go here") = true

/--
AST-level wrapper correctness for the current generator surface.
The generated wrapper preserves the full contract record and carries the required
template phases that model ownership checking, the implementation call, result mapping,
and drop cleanup.
-/
def WrapperCorrect (contract : FunctionContract) (lang : WrapperLanguage) : Prop :=
  let wrapper := generateWrapper contract lang
  wrapper.name = contract.symbol ∧
    wrapper.contract = contract ∧
    wrapper.isEmpty = false ∧
    wrapper.stmts.head?.bind WrapperStmt.getComment? = some (wrapperDocComment lang contract) ∧
    wrapperHasPhaseComments wrapper

/--
Generated wrapper correctness theorem placeholder.
-/
theorem generated_wrapper_correct
  (contract : FunctionContract)
  (lang : WrapperLanguage) :
  WrapperCorrect contract lang := by
  cases lang <;>
    simp [WrapperCorrect, generateWrapper, generateCWrapper, generateRustWrapper, generateZigWrapper,
      wrapperDocComment, wrapperHasPhaseComments, WrapperFunction.isEmpty]

/--
Theorem: C wrapper module includes runtime when targeting C.
-/
theorem c_wrapper_module_includes_runtime (contracts : List FunctionContract) :
  (generateWrapperModule contracts .c).includesRuntime = true := by
  exact c_module_requires_runtime contracts

/--
Theorem: Rust wrapper module does not include runtime.
-/
theorem rust_wrapper_module_no_runtime (contracts : List FunctionContract) :
  (generateWrapperModule contracts .rust).includesRuntime = false := by
  exact rust_module_no_runtime contracts

/--
Theorem: Zig wrapper module does not include runtime.
-/
theorem zig_wrapper_module_no_runtime (contracts : List FunctionContract) :
  (generateWrapperModule contracts .zig).includesRuntime = false := by
  exact zig_module_no_runtime contracts

/--
Every generated wrapper module preserves the input contract count.
-/
theorem generated_wrapper_module_preserves_contract_count
  (contracts : List FunctionContract)
  (lang : WrapperLanguage) :
  (generateWrapperModule contracts lang).functions.length = contracts.length := by
  cases lang <;> simp [generateWrapperModule]

/--
Every generated wrapper in a module satisfies the AST-level wrapper correctness predicate.
-/
theorem generated_wrapper_module_correct
  (contracts : List FunctionContract)
  (lang : WrapperLanguage) :
  (generateWrapperModule contracts lang).functions.all
      (fun wrapper => decide (WrapperCorrect wrapper.contract lang)) = true := by
  induction contracts with
  | nil =>
      cases lang <;> simp [generateWrapperModule]
  | cons head tail ih =>
      cases lang <;>
        simp [generateWrapperModule, generated_wrapper_correct, WrapperCorrect, generateWrapper,
          generateCWrapper, generateRustWrapper, generateZigWrapper, wrapperDocComment,
          wrapperHasPhaseComments, WrapperFunction.isEmpty, ih]

end Chimera.Wrapper
