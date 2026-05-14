-- ChimeraProof Link: Resolve
-- Symbol resolution.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Contract
import Chimera.Link.SymbolTable
import Chimera.IR.Module

namespace Chimera

/--
Resolution error.
-/
inductive ResolveError where
  | unresolvedImport (sym : Symbol)
  | duplicateStrongSymbol (sym : Symbol)
  | weakSymbolNotFound (sym : Symbol)
  | incompatibleSignature (sym : Symbol)
  | incompatibleTarget (sym : Symbol)

/--
Resolved symbol.
-/
structure ResolvedSymbol where
  symbol : Symbol
  contract : FunctionContract
  sourceModule : Symbol

/--
Link plan entry.
-/
structure LinkPlanEntry where
  import_ : Symbol
  resolvedTo : ResolvedSymbol

/--
Link plan.
-/
structure LinkPlan where
  entries : List LinkPlanEntry

/--
Build symbol table from modules.
Includes both exports (definitions) and imports (unresolved references).
-/
def buildSymbolTable (modules : List Module) : SymbolTable :=
  List.foldl (fun tbl m =>
    let defs := m.exports.map (fun e =>
      { symbol := e.symbol, contract := e.contract, sourceModule := m.moduleName, target := m.target,
        strength := SymbolStrength.strong, visibility := SymbolVisibility.vis_public })
    let tbl' := List.foldl (fun t d => t.addDef d) tbl defs
    let imports := m.imports.map (fun i =>
      { symbol := i.symbol, contract := i.contract, sourceModule := m.moduleName, target := m.target })
    { tbl' with imports := tbl'.imports ++ imports }
  ) SymbolTable.empty modules

/--
Resolve symbols from symbol table.
Detects duplicate strong symbols before choosing one.
-/
def resolveSymbols (tbl : SymbolTable) : Except ResolveError (List ResolvedSymbol) :=
  go tbl.imports []
where
  contractsCompatible (imp : ImportRef) (defn : SymbolDef) : Except ResolveError Unit :=
    if ¬ Target.compatible imp.target defn.target then
      .error (.incompatibleTarget imp.symbol)
    else if ¬ imp.contract.semanticSig.compatibleWith defn.contract.semanticSig then
      .error (.incompatibleSignature imp.symbol)
    else if ¬ imp.contract.physicalSig.compatibleWith defn.contract.physicalSig then
      .error (.incompatibleSignature imp.symbol)
    else if ¬ imp.contract.safety.canCall defn.contract.safety then
      .error (.incompatibleSignature imp.symbol)
    else if imp.contract.trust != defn.contract.trust then
      .error (.incompatibleSignature imp.symbol)
    else
      .ok ()

  selectDefinition (sym : Symbol) (defs : List SymbolDef) : Except ResolveError SymbolDef :=
    let strongDefs := defs.filter (·.strength == .strong)
    if strongDefs.length > 1 then
      .error (.duplicateStrongSymbol sym)
    else
      match strongDefs.head? with
      | some defn => .ok defn
      | none =>
        match defs.find? (fun d => d.strength == .weak) with
        | some defn => .ok defn
        | none =>
          match defs.find? (fun d => d.strength == .linkonce) with
          | some defn => .ok defn
          | none => .error (.weakSymbolNotFound sym)

  go : List ImportRef → List ResolvedSymbol → Except ResolveError (List ResolvedSymbol)
    | [], resolved => .ok resolved.reverse
    | imp :: rest, resolved =>
      let defs := tbl.findDefs imp.symbol
      match defs with
      | [] => .error (.unresolvedImport imp.symbol)
      | defs =>
        match selectDefinition imp.symbol defs with
        | .error err => .error err
        | .ok defn =>
          match contractsCompatible imp defn with
          | .error err => .error err
          | .ok () =>
              go rest ({ symbol := imp.symbol, contract := defn.contract, sourceModule := defn.sourceModule } :: resolved)

end Chimera
