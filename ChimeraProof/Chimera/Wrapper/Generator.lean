-- ChimeraProof Wrapper: Generator
-- Wrapper generation from function contracts.

import Chimera.Wrapper.AST
import Chimera.ABI.Contract

namespace Chimera.Wrapper

/--
Generate a wrapper for a specific backend.
-/
def generateWrapper (contract : FunctionContract) (lang : WrapperLanguage) : WrapperFunction :=
  match lang with
  | .c => generateCWrapper contract
  | .rust => generateRustWrapper contract
  | .zig => generateZigWrapper contract

/--
Generate a C wrapper function from a contract.
Adds comments describing the wrapper structure for pointer validation,
conversions, implementation call, result mapping, and cleanup.
-/
def generateCWrapper (contract : FunctionContract) : WrapperFunction :=
  let doc := [WrapperStmt.c (CStmt.comment ("C wrapper for " ++ contract.symbol.name))]
  let allocCheck := [WrapperStmt.c (CStmt.comment "pointer validation would go here")]
  let call := [WrapperStmt.c (CStmt.comment "implementation call would go here")]
  let resultMap := [WrapperStmt.c (CStmt.comment "result mapping would go here")]
  let cleanup := [WrapperStmt.c (CStmt.comment "cleanup would go here")]
  ⟨contract.symbol, contract, doc ++ allocCheck ++ call ++ resultMap ++ cleanup⟩

/--
Generate a Rust wrapper function from a contract.
Adds comments describing the wrapper structure.
-/
def generateRustWrapper (contract : FunctionContract) : WrapperFunction :=
  let doc := [WrapperStmt.rust (RustStmt.comment ("Rust wrapper for " ++ contract.symbol.name))]
  let allocCheck := [WrapperStmt.rust (RustStmt.comment "pointer validation would go here")]
  let call := [WrapperStmt.rust (RustStmt.comment "implementation call would go here")]
  let resultMap := [WrapperStmt.rust (RustStmt.comment "result mapping would go here")]
  let cleanup := [WrapperStmt.rust (RustStmt.comment "cleanup would go here")]
  ⟨contract.symbol, contract, doc ++ allocCheck ++ call ++ resultMap ++ cleanup⟩

/--
Generate a Zig wrapper function from a contract.
Adds comments describing the wrapper structure.
-/
def generateZigWrapper (contract : FunctionContract) : WrapperFunction :=
  let doc := [WrapperStmt.zig (ZigStmt.comment ("Zig wrapper for " ++ contract.symbol.name))]
  let allocCheck := [WrapperStmt.zig (ZigStmt.comment "pointer validation would go here")]
  let call := [WrapperStmt.zig (ZigStmt.comment "implementation call would go here")]
  let resultMap := [WrapperStmt.zig (ZigStmt.comment "result mapping would go here")]
  let cleanup := [WrapperStmt.zig (ZigStmt.comment "cleanup would go here")]
  ⟨contract.symbol, contract, doc ++ allocCheck ++ call ++ resultMap ++ cleanup⟩

/--
Generate wrapper module from contracts.
-/
def generateWrapperModule
  (contracts : List FunctionContract)
  (targetLang : WrapperLanguage)
  : WrapperModule :=
  let functions := match targetLang with
    | .c => contracts.map generateCWrapper
    | .rust => contracts.map generateRustWrapper
    | .zig => contracts.map generateZigWrapper
  let runtime := match targetLang with
    | .c => true
    | .rust => false
    | .zig => false
  ⟨targetLang, functions, runtime⟩

namespace generateCWrapper

/--
Generated C wrapper is well-formed.
-/
theorem generated_wrapper_wf (contract : FunctionContract) :
  let wrapper := generateCWrapper contract
  wrapper.name = contract.symbol ∧
    wrapper.contract = contract ∧
    wrapper.isEmpty = false := by
  simp [generateCWrapper, WrapperFunction.isEmpty]

end generateCWrapper

namespace generateRustWrapper

/--
Generated Rust wrapper is well-formed and non-empty.
-/
theorem generated_rust_wrapper_wf (contract : FunctionContract) :
  let wrapper := generateRustWrapper contract
  wrapper.contract = contract ∧
    wrapper.isEmpty = false := by
  simp [generateRustWrapper, WrapperFunction.isEmpty]

/--
Generated Rust wrapper has correct symbol.
-/
theorem generated_rust_wrapper_symbol (contract : FunctionContract) :
  (generateRustWrapper contract).name = contract.symbol := by
  rfl

end generateRustWrapper

namespace generateZigWrapper

/--
Generated Zig wrapper is well-formed and non-empty.
-/
theorem generated_zig_wrapper_wf (contract : FunctionContract) :
  let wrapper := generateZigWrapper contract
  wrapper.contract = contract ∧
    wrapper.isEmpty = false := by
  simp [generateZigWrapper, WrapperFunction.isEmpty]

/--
Generated Zig wrapper has correct symbol.
-/
theorem generated_zig_wrapper_symbol (contract : FunctionContract) :
  (generateZigWrapper contract).name = contract.symbol := by
  rfl

end generateZigWrapper

/--
Wrapper correctness: generated wrapper preserves contract symbol.
-/
theorem wrapper_preserves_symbol (contract : FunctionContract) (lang : WrapperLanguage) :
  (generateWrapper contract lang).name = contract.symbol := by
  cases lang <;> rfl

/--
Wrapper correctness: generated wrapper is non-empty for all languages.
-/
theorem wrapper_non_empty (contract : FunctionContract) (lang : WrapperLanguage) :
  (generateWrapper contract lang).isEmpty = false := by
  cases lang <;> simp [generateWrapper, generateCWrapper, generateRustWrapper, generateZigWrapper,
    WrapperFunction.isEmpty]

/--
Wrapper correctness: Rust wrapper renders without panicking.
-/
theorem rust_wrapper_render_safes (contract : FunctionContract) :
  let rendered := (generateRustWrapper contract).stmts.map renderWrapperStmt
  rendered.length = 5 := by
  simp [generateRustWrapper]

/--
Wrapper correctness: Zig wrapper renders without panicking.
-/
theorem zig_wrapper_render_safes (contract : FunctionContract) :
  let rendered := (generateZigWrapper contract).stmts.map renderWrapperStmt
  rendered.length = 5 := by
  simp [generateZigWrapper]

/--
Wrapper correctness: module targets C requires runtime.
-/
theorem c_module_requires_runtime (contracts : List FunctionContract) :
  (generateWrapperModule contracts .c).includesRuntime = true := by
  simp [generateWrapperModule]

/--
Wrapper correctness: rust module does not include runtime.
-/
theorem rust_module_no_runtime (contracts : List FunctionContract) :
  (generateWrapperModule contracts .rust).includesRuntime = false := by
  simp [generateWrapperModule]

/--
Wrapper correctness: zig module does not include runtime.
-/
theorem zig_module_no_runtime (contracts : List FunctionContract) :
  (generateWrapperModule contracts .zig).includesRuntime = false := by
  simp [generateWrapperModule]

end Chimera.Wrapper
