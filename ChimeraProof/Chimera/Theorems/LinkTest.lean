-- ChimeraProof Tests: Link Tests
-- Compile-safe theorem smoke tests for link modules.

import Chimera.Foundation
import Chimera.ABI
import Chimera.Link.SymbolTable
import Chimera.Link.Resolve
import Chimera.IR.Module

namespace Chimera.Test

private def sampleSemanticSig : SemanticSignature := {
  params := []
  returns := .unit
  isVarargs := false
}

private def samplePhysicalSig : PhysicalSignature := {
  params := []
  returns := .void
  callingConv := .cdecl
}

private def alternatePhysicalSig : PhysicalSignature := {
  params := [.ptr]
  returns := .void
  callingConv := .cdecl
}

private def alternateCallingConvSig : PhysicalSignature := {
  params := []
  returns := .void
  callingConv := .stdcall
}

private def alternateSemanticSig : SemanticSignature := {
  params := [{ name := "arg0", ty := .u32 }]
  returns := .unit
  isVarargs := false
}

private def sampleContract (sym : Symbol) (physicalSig := samplePhysicalSig) (semanticSig := sampleSemanticSig) : FunctionContract := {
  symbol := sym
  language := .c
  form := .infallible
  semanticSig := semanticSig
  physicalSig := physicalSig
  effects := [.pure]
  panicPolicy := .forbidden
  safety := .verified
  allocator := none
  requiresDrop := false
  trust := .proofObligation
  errorDomain := none
}

private def sampleDef (sym : Symbol) (strength : SymbolStrength) (physicalSig := samplePhysicalSig) (semanticSig := sampleSemanticSig) : SymbolDef := {
  symbol := sym
  contract := sampleContract sym physicalSig semanticSig
  sourceModule := Symbol.simple "export_mod"
  target := Target.x86_64_linux
  strength := strength
  visibility := .vis_public
}

private def sampleImport (sym : Symbol) (physicalSig := samplePhysicalSig) (semanticSig := sampleSemanticSig) : ImportRef :=
  {
    symbol := sym
    contract := sampleContract sym physicalSig semanticSig
    sourceModule := Symbol.simple "import_mod"
    target := Target.x86_64_linux
  }

namespace LinkPlanTest

theorem link_plan_smoke : True := by
  trivial

end LinkPlanTest

namespace SymbolTableTest

theorem symbol_table_smoke : True := by
  trivial

end SymbolTableTest

namespace ResolveTest

theorem resolve_smoke : True := by
  trivial

end ResolveTest

namespace ComposeTest

theorem compose_smoke : True := by
  trivial

end ComposeTest

namespace ResolveSymbolsTest

theorem resolve_symbols_smoke : True := by
  trivial

theorem duplicate_strong_symbols_are_rejected :
    let sym := Symbol.simple "dup_strong"
    let tbl : SymbolTable := {
      defs := [sampleDef sym .strong, sampleDef sym .strong]
      imports := [sampleImport sym]
    }
    resolveSymbols tbl = .error (.duplicateStrongSymbol sym) := by
  native_decide

theorem strong_definition_beats_weak_and_linkonce :
    let sym := Symbol.simple "prefer_strong"
    let strongContract := sampleContract sym
    let weakContract := sampleContract sym alternatePhysicalSig
    let linkonceContract := sampleContract sym alternatePhysicalSig
    let tbl : SymbolTable := {
      defs := [
        { symbol := sym, contract := linkonceContract, strength := .linkonce, visibility := .vis_public },
        { symbol := sym, contract := weakContract, strength := .weak, visibility := .vis_public },
        { symbol := sym, contract := strongContract, strength := .strong, visibility := .vis_public }
      ]
      imports := [sampleImport sym]
    }
    (resolveSymbols tbl).map (fun resolved => resolved.head?.map (·.contract.physicalSig)) =
      .ok (some samplePhysicalSig) := by
  native_decide

theorem weak_definition_beats_linkonce :
    let sym := Symbol.simple "prefer_weak"
    let tbl : SymbolTable := {
      defs := [
        sampleDef sym .linkonce alternatePhysicalSig,
        sampleDef sym .weak
      ]
      imports := [sampleImport sym]
    }
    (resolveSymbols tbl).map (fun resolved => resolved.head?.map (·.contract.physicalSig)) =
      .ok (some samplePhysicalSig) := by
  native_decide

theorem linkonce_definition_resolves_when_it_is_the_only_choice :
    let sym := Symbol.simple "linkonce_only"
    let tbl : SymbolTable := {
      defs := [sampleDef sym .linkonce]
      imports := [sampleImport sym]
    }
    (resolveSymbols tbl).map (fun resolved => resolved.head?.map (·.contract.physicalSig)) =
      .ok (some samplePhysicalSig) := by
  native_decide

theorem selected_definition_must_still_match_import_signature :
    let sym := Symbol.simple "mismatch"
    let tbl : SymbolTable := {
      defs := [sampleDef sym .strong alternatePhysicalSig, sampleDef sym .weak]
      imports := [sampleImport sym]
    }
    resolveSymbols tbl = .error (.incompatibleSignature sym) := by
  native_decide

theorem semantic_signature_mismatch_is_rejected :
    let sym := Symbol.simple "semantic_mismatch"
    let tbl : SymbolTable := {
      defs := [sampleDef sym .strong samplePhysicalSig alternateSemanticSig]
      imports := [sampleImport sym]
    }
    resolveSymbols tbl = .error (.incompatibleSignature sym) := by
  native_decide

theorem calling_convention_mismatch_is_rejected :
    let sym := Symbol.simple "cc_mismatch"
    let tbl : SymbolTable := {
      defs := [sampleDef sym .strong alternateCallingConvSig]
      imports := [sampleImport sym]
    }
    resolveSymbols tbl = .error (.incompatibleSignature sym) := by
  native_decide

theorem target_mismatch_is_rejected :
    let sym := Symbol.simple "target_mismatch"
    let tbl : SymbolTable := {
      defs := [{ (sampleDef sym .strong) with target := Target.x86_64_windows }]
      imports := [sampleImport sym]
    }
    resolveSymbols tbl = .error (.incompatibleTarget sym) := by
  native_decide

theorem safety_mismatch_is_rejected :
    let sym := Symbol.simple "safety_mismatch"
    let exportContract := { sampleContract sym with safety := .unsafeContract }
    let tbl : SymbolTable := {
      defs := [{ symbol := sym, contract := exportContract, sourceModule := Symbol.simple "export_mod",
        target := Target.x86_64_linux, strength := .strong, visibility := .vis_public }]
      imports := [sampleImport sym]
    }
    resolveSymbols tbl = .error (.incompatibleSignature sym) := by
  native_decide

theorem trust_mismatch_is_rejected :
    let sym := Symbol.simple "trust_mismatch"
    let exportContract := { sampleContract sym with trust := .trusted }
    let tbl : SymbolTable := {
      defs := [{ symbol := sym, contract := exportContract, sourceModule := Symbol.simple "export_mod",
        target := Target.x86_64_linux, strength := .strong, visibility := .vis_public }]
      imports := [sampleImport sym]
    }
    resolveSymbols tbl = .error (.incompatibleSignature sym) := by
  native_decide

end ResolveSymbolsTest

namespace BuildSymbolTableTest

private def importedContract : FunctionContract :=
  sampleContract (Symbol.simple "imported_fn")

private def exportingContract : FunctionContract :=
  sampleContract (Symbol.simple "exported_fn")

private def sampleModule : Module := {
  abiVersion := "0.1"
  moduleName := Symbol.simple "sample_mod"
  language := .c
  target := Target.x86_64_linux
  exports := [{ symbol := exportingContract.symbol, contract := exportingContract }]
  imports := [{ symbol := importedContract.symbol, contract := importedContract }]
  types := []
  layouts := []
}

theorem build_symbol_table_smoke :
    let tbl := buildSymbolTable [sampleModule]
    tbl.defs.length = 1 ∧ tbl.imports.length = 1 := by
  native_decide

theorem build_symbol_table_records_export_provenance :
    let tbl := buildSymbolTable [sampleModule]
    tbl.defs.head?.map (fun d => d.sourceModule == Symbol.simple "sample_mod" && d.target == Target.x86_64_linux) = some true := by
  native_decide

theorem build_symbol_table_records_import_provenance :
    let tbl := buildSymbolTable [sampleModule]
    tbl.imports.head?.map (fun i => i.sourceModule == Symbol.simple "sample_mod" && i.target == Target.x86_64_linux) = some true := by
  native_decide

theorem build_symbol_table_includes_unresolved_imports :
    let tbl := buildSymbolTable [sampleModule]
    tbl.imports.head?.map (·.symbol == Symbol.simple "imported_fn") = some true := by
  native_decide

end BuildSymbolTableTest

end Chimera.Test
