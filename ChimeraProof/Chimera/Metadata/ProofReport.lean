-- ChimeraProof Metadata: ProofReport
-- Proof report generation over the current certified-build surface.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Contract
import Chimera.Metadata.Schema
import Chimera.Metadata.CHProof
import Chimera.IR.Module
import Chimera.Effects.Inference
import Chimera.Wrapper.Generator
import Chimera.Checkers.MetadataChecker
import Chimera.Checkers.ContractChecker
import Chimera.Checkers.OwnershipChecker
import Chimera.Checkers.AllocatorChecker
import Chimera.Checkers.ResultChecker
import Chimera.Checkers.PanicChecker
import Chimera.Checkers.FullChecker

namespace Chimera.Metadata

private def exceptSucceeded {ε : Type u} {α : Type v} : Except ε α → Bool
  | .ok _ => true
  | .error _ => false

private def moduleContracts (m : Module) : List FunctionContract :=
  m.exports.map (·.contract)

private def moduleTargetsCompatible (modules : List Module) (m : Module) : Bool :=
  modules.all (fun other =>
    other.target.ptrWidth == m.target.ptrWidth &&
      other.target.endian == m.target.endian)

private def moduleLayoutCheckPassed (m : Module) : Bool :=
  m.layouts.all (fun layout =>
    layout.align != 0 &&
      (layout.size == 0 || layout.size >= layout.align))

private def layoutPhaseEvidence (m : Module) : List String :=
  if m.layouts.all (fun layout => layout.align != 0) then
    if m.layouts.all (fun layout => layout.size == 0 || layout.size >= layout.align) then
      ["layout-ok"]
    else
      ["layout-failed", "layout-size-below-align"]
  else
    ["layout-failed", "invalid-layout-align"]

private def ownershipContractOf (contract : FunctionContract) : CallContract :=
  {
    args := contract.semanticSig.params.map (·.ty)
    returns := .void
    effects := contract.effects
    panic := contract.panicPolicy
    safety := contract.safety
  }

private def moduleOwnershipCheckPassed (m : Module) : Bool :=
  (moduleContracts m).all (fun contract =>
    exceptSucceeded (checkOwnership CallState.empty (ownershipContractOf contract)) &&
      exceptSucceeded (checkNoCallLifetimeEscape contract.semanticSig.returns))

private def allocatorReturnTypes : ChType → List ChType
  | .result okTy errTy => [okTy, errTy]
  | ty => [ty]

private def moduleAllocatorCheckPassed (m : Module) : Bool :=
  (moduleContracts m).all (fun contract =>
    (allocatorReturnTypes contract.semanticSig.returns).all (fun ty =>
      exceptSucceeded (checkOwnedOpaqueHasDrop DropRegistry.empty ty)))

private def moduleResultCheckPassed (m : Module) : Bool :=
  (moduleContracts m).all (fun contract =>
    exceptSucceeded (checkFallibleSignature contract.semanticSig) &&
      match contract.errorDomain with
      | some domain => exceptSucceeded (checkErrorDomain domain)
      | none => true)

private def modulePanicCheckPassed (m : Module) : Bool :=
  (moduleContracts m).all (fun contract =>
    exceptSucceeded (checkPanicPolicy contract.panicPolicy))

private def effectsCovered (inferred declared : EffectSet) : Bool :=
  inferred.all (memberEffect declared)

private def moduleEffectsCheckPassed (m : Module) : Bool :=
  (moduleContracts m).all (fun contract =>
    let inferredSet := (inferFromSignature contract.semanticSig).toEffectSet
    effectsCovered inferredSet contract.effects)

private def wrapperGenerated (contract : FunctionContract) : Bool :=
  let cWrapper := Chimera.Wrapper.generateCWrapper contract
  let rustWrapper := Chimera.Wrapper.generateRustWrapper contract
  let zigWrapper := Chimera.Wrapper.generateZigWrapper contract
  !cWrapper.stmts.isEmpty && !rustWrapper.stmts.isEmpty && !zigWrapper.stmts.isEmpty

private def moduleWrapperCheckPassed (m : Module) : Bool :=
  (moduleContracts m).all (fun contract =>
    match contract.safety with
    | .generatedWrapper => wrapperGenerated contract
    | _ => true)

private structure ModulePhaseReport where
  metadataOk : Bool
  layoutOk : Bool
  contractOk : Bool
  ownershipOk : Bool
  allocatorOk : Bool
  resultOk : Bool
  panicOk : Bool
  effectsOk : Bool
  wrappersOk : Bool
  linkOk : Bool

private def evaluateModulePhases (modules : List Module) (m : Module) : ModulePhaseReport :=
  let contractOk := (moduleContracts m).all (fun contract => exceptSucceeded (checkAllContract contract))
  {
    metadataOk := exceptSucceeded (checkChMeta m)
    layoutOk := moduleLayoutCheckPassed m
    contractOk := contractOk
    ownershipOk := moduleOwnershipCheckPassed m
    allocatorOk := moduleAllocatorCheckPassed m
    resultOk := moduleResultCheckPassed m
    panicOk := modulePanicCheckPassed m
    effectsOk := moduleEffectsCheckPassed m
    wrappersOk := moduleWrapperCheckPassed m
    linkOk := moduleTargetsCompatible modules m
  }

private def obligationPhasePassed (phases : ModulePhaseReport) (kind : ProofObligationKind) : Bool :=
  match kind with
  | .layout => phases.layoutOk
  | .signature => phases.metadataOk && phases.contractOk
  | .ownership => phases.ownershipOk
  | .allocator => phases.allocatorOk
  | .result => phases.resultOk
  | .panic => phases.panicOk
  | .effects => phases.effectsOk
  | .wrappers => phases.wrappersOk
  | .link => phases.linkOk

private def metadataPhaseEvidence (m : Module) : List String :=
  match checkChMeta m with
  | .ok _ => ["metadata-ok"]
  | .error (.invalidVersion _) => ["metadata-failed", "invalid-version"]
  | .error .emptyModuleName => ["metadata-failed", "empty-module-name"]
  | .error .emptyExport => ["metadata-failed", "empty-export"]
  | .error .emptyImport => ["metadata-failed", "empty-import"]
  | .error .emptyContract => ["metadata-failed", "empty-contract"]
  | .error .invalidSafety => ["metadata-failed", "invalid-safety"]
  | .error .emptyEffectSet => ["metadata-failed", "empty-effect-set"]
  | .error (.duplicateExport _) => ["metadata-failed", "duplicate-export"]
  | .error (.duplicateImport _) => ["metadata-failed", "duplicate-import"]
  | .error (.duplicateLayout _) => ["metadata-failed", "duplicate-layout"]
  | .error .emptyLayoutName => ["metadata-failed", "empty-layout-name"]
  | .error (.invalidLayoutAlign _ _) => ["metadata-failed", "invalid-layout-align"]
  | .error (.layoutSizeBelowAlign _) => ["metadata-failed", "layout-size-below-align"]
  | .error .emptyImportSymbol => ["metadata-failed", "empty-import-symbol"]
  | .error .emptyExportSymbol => ["metadata-failed", "empty-export-symbol"]
  | .error (.importMismatch _) => ["metadata-failed", "import-mismatch"]
  | .error (.exportMismatch _) => ["metadata-failed", "export-mismatch"]
  | .error (.invalidTypeSize _ _) => ["metadata-failed", "invalid-type-size"]

private def contractPhaseEvidence (contract : FunctionContract) : List String :=
  match checkAllContract contract with
  | .ok _ => ["contract-ok"]
  | .error (.unsafeRawPtr _) => ["contract-failed", "unsafe-raw-ptr"]
  | .error (.missingAllocator _) => ["contract-failed", "missing-allocator"]
  | .error (.invalidEffectSet _) => ["contract-failed", "invalid-effect-set"]
  | .error (.incompatibleSignatures _) => ["contract-failed", "incompatible-signatures"]
  | .error (.untrustedExternal _) => ["contract-failed", "untrusted-external"]

private def resultPhaseEvidence (contract : FunctionContract) : List String :=
  match contract.errorDomain with
  | none => ["result-ok"]
  | some domain =>
      match checkErrorDomain domain with
      | .ok _ => ["result-ok"]
      | .error (.invalidErrorDomain _) => ["result-failed", "invalid-error-domain"]
      | .error (.missingOutParam _) => ["result-failed", "missing-out-param"]
      | .error (.nonZeroErrorStatus _) => ["result-failed", "non-zero-error-status"]
      | .error .unexpectedPayloadOnSuccess => ["result-failed", "unexpected-payload-on-success"]

private def ownershipPhaseEvidence (contract : FunctionContract) : List String :=
  match checkNoCallLifetimeEscape contract.semanticSig.returns with
  | .ok _ => ["ownership-ok"]
  | .error (.callLifetimeReturn _) => ["ownership-failed", "call-lifetime-return"]
  | .error (.doubleOwn _) => ["ownership-failed", "double-own"]
  | .error (.writeBorrowAlias _) => ["ownership-failed", "write-borrow-alias"]
  | .error (.borrowEscapes _) => ["ownership-failed", "borrow-escapes"]
  | .error (.droppedUse _) => ["ownership-failed", "dropped-use"]
  | .error (.movedUse _) => ["ownership-failed", "moved-use"]
  | .error (.noOwner _) => ["ownership-failed", "no-owner"]

private def allocatorPhaseEvidence (contract : FunctionContract) : List String :=
  let outcomes := (allocatorReturnTypes contract.semanticSig.returns).map (checkOwnedOpaqueHasDrop DropRegistry.empty)
  if outcomes.all exceptSucceeded then
    ["allocator-ok"]
  else if outcomes.any (fun outcome =>
      match outcome with
      | .error (.noDropFunction _) => true
      | _ => false) then
    ["allocator-failed", "no-drop-function"]
  else if outcomes.any (fun outcome =>
      match outcome with
      | .error (.mismatchedAllocator _) => true
      | _ => false) then
    ["allocator-failed", "mismatched-allocator"]
  else
    ["allocator-failed", "invalid-drop"]

private def effectsPhaseEvidence (contract : FunctionContract) : List String :=
  let inferredSet := (inferFromSignature contract.semanticSig).toEffectSet
  if effectsCovered inferredSet contract.effects then
    ["effects-ok"]
  else
    ["effects-failed", "underdeclared-effects"]

private def wrapperPhaseEvidence (contract : FunctionContract) : List String :=
  match contract.safety with
  | .generatedWrapper =>
      if wrapperGenerated contract then
        ["wrapper-generated", "wrapper-non-empty"]
      else
        ["wrapper-failed", "wrapper-empty"]
  | _ => ["wrapper-not-required"]

private def linkPhaseEvidence (modules : List Module) (m : Module) : List String :=
  if moduleTargetsCompatible modules m then
    ["link-ok"]
  else
    ["link-failed", "target-mismatch"]

private def moduleObligationPhaseEvidence
  (modules : List Module)
  (m : Module)
  (kind : ProofObligationKind) : List String :=
  match kind with
  | .signature => metadataPhaseEvidence m
  | .link => linkPhaseEvidence modules m
  | _ => []

private def checkedStatus (status : ProofStatus) (phaseOk : Bool) : ProofStatus :=
  if phaseOk then status else .unsupported

private def checkedEvidence (baseEvidence : List String) (phaseOk : Bool) : List String :=
  (if phaseOk then ["checker-pass"] else ["checker-failed"]) ++ baseEvidence

private def obligationStatus (contract : FunctionContract) : ProofStatus :=
  match contract.safety, contract.trust with
  | .unsafeContract, _ => .unsupported
  | .trustedContract, _ => .trusted
  | _, .unchecked => .assumed
  | _, .trusted => .trusted
  | _, .proofObligation => .proved

private def obligationEvidencePrefix (contract : FunctionContract) : List String :=
  match contract.safety, contract.trust with
  | .unsafeContract, _ => ["unsafe-boundary"]
  | .trustedContract, _ => ["trusted-contract"]
  | _, .unchecked => ["unchecked-boundary"]
  | _, .trusted => ["trusted-boundary"]
  | _, .proofObligation => ["fullCheck"]

private def phaseEvidence (kind : ProofObligationKind) : List String :=
  match kind with
  | .layout => ["layout-check"]
  | .signature => ["metadata-check", "contract-check"]
  | .ownership => ["ownership-check"]
  | .allocator => ["allocator-check"]
  | .result => ["result-check"]
  | .panic => ["panic-check"]
  | .effects => ["effects-check"]
  | .wrappers => ["wrapper-check"]
  | .link => ["link-check"]

private def obligationAssumptions (contract : FunctionContract) : List String :=
  match contract.safety, contract.trust with
  | .unsafeContract, _ => ["unsafe export requires manual review"]
  | .trustedContract, _ => ["trusted contract accepted without full proof"]
  | _, .unchecked => ["unchecked boundary accepted"]
  | _, .trusted => ["trusted foreign ABI boundary"]
  | _, .proofObligation => []

private def contractObligationKinds (contract : FunctionContract) : List ProofObligationKind :=
  let baseKinds := [ProofObligationKind.signature]
  let onlyPureEffects :=
    contract.effects.length = 1 && memberEffect contract.effects Effect.pure
  let baseKinds :=
    if contract.effects.isEmpty || onlyPureEffects then baseKinds
    else ProofObligationKind.effects :: baseKinds
  let baseKinds :=
    if contract.panicPolicy == PanicPolicy.forbidden then baseKinds
    else ProofObligationKind.panic :: baseKinds
  let baseKinds :=
    if contract.isFallible || contract.errorDomain.isSome then
      ProofObligationKind.result :: baseKinds
    else baseKinds
  let baseKinds :=
    if contract.allocator.isSome || contract.requiresDrop then
      ProofObligationKind.allocator :: baseKinds
    else baseKinds
  let baseKinds := match contract.form with
    | AbiForm.constructor | AbiForm.destructor => ProofObligationKind.ownership :: baseKinds
    | _ => baseKinds
  let baseKinds := match contract.safety with
    | SafetyClass.generatedWrapper => ProofObligationKind.wrappers :: baseKinds
    | _ => baseKinds
  baseKinds.reverse

private def layoutObligations (phases : ModulePhaseReport) (m : Module) : List ProofCertificate :=
  m.layouts.map (fun layout =>
    {
      kind := .layout
      target := layout.name
      description := "declared layout checked by fullCheck"
      status := checkedStatus .proved (obligationPhasePassed phases .layout)
      assumptions := []
      evidence := checkedEvidence
        (["fullCheck"] ++ phaseEvidence .layout ++ layoutPhaseEvidence m)
        (obligationPhasePassed phases .layout)
      trusted := false
    })

private def exportKindEvidence (m : Module) (contract : FunctionContract) (kind : ProofObligationKind) : List String :=
  contractPhaseEvidence contract ++
    match kind with
    | .signature => metadataPhaseEvidence m
    | .ownership => ownershipPhaseEvidence contract
    | .allocator => allocatorPhaseEvidence contract
    | .result => resultPhaseEvidence contract
    | .effects => effectsPhaseEvidence contract
    | .wrappers => wrapperPhaseEvidence contract
    | _ => []

private def exportObligations (phases : ModulePhaseReport) (m : Module) : List ProofCertificate :=
  m.exports.flatMap (fun exp =>
    let baseStatus := obligationStatus exp.contract
    let assumptions := obligationAssumptions exp.contract
    let prefixEvidence := obligationEvidencePrefix exp.contract
    (contractObligationKinds exp.contract).map (fun kind =>
      let phaseOk := obligationPhasePassed phases kind
      {
        kind := kind
        target := exp.symbol
        description := "export contract checked by fullCheck"
        status := checkedStatus baseStatus phaseOk
        assumptions := assumptions
        evidence := checkedEvidence
          (prefixEvidence ++ phaseEvidence kind ++ exportKindEvidence m exp.contract kind)
          phaseOk
        trusted := checkedStatus baseStatus phaseOk == ProofStatus.trusted
      }))

private def importObligations (modules : List Module) (phases : ModulePhaseReport) (m : Module) : List ProofCertificate :=
  m.imports.map (fun imp =>
    let baseStatus := obligationStatus imp.contract
    let assumptions := obligationAssumptions imp.contract
    let prefixEvidence := obligationEvidencePrefix imp.contract
    {
      kind := .link
      target := imp.symbol
      description := "import boundary checked by link validation"
      status := checkedStatus baseStatus (obligationPhasePassed phases .link)
      assumptions := assumptions
      evidence := checkedEvidence
        (prefixEvidence ++ phaseEvidence .link ++ moduleObligationPhaseEvidence modules m .link)
        (obligationPhasePassed phases .link)
      trusted := checkedStatus baseStatus (obligationPhasePassed phases .link) == ProofStatus.trusted
    })

private def obligationsForModule (modules : List Module) (m : Module) : List ProofCertificate :=
  let phases := evaluateModulePhases modules m
  layoutObligations phases m ++ exportObligations phases m ++ importObligations modules phases m

private def trustAssumptionForBoundary
  (boundary : String)
  (symbol : Symbol)
  (contract : FunctionContract) : Option ProofTrustAssumption :=
  match contract.safety, contract.trust with
  | .unsafeContract, _ =>
      some {
        kind := TrustAssumptionKind.manualProof
        description := s!"unsafe {boundary} {symbol.fqn} requires manual review"
        external_ref := none
        trusted := true
      }
  | .trustedContract, _ =>
      some {
        kind := if boundary == "import" then TrustAssumptionKind.trustedLinker else TrustAssumptionKind.trustedFunction
        description := s!"trusted {boundary} contract {symbol.fqn}"
        external_ref := none
        trusted := true
      }
  | _, .unchecked =>
      some {
        kind := if boundary == "import" then TrustAssumptionKind.trustedLinker else TrustAssumptionKind.trustedForeignAbi
        description := s!"unchecked {boundary} boundary {symbol.fqn}"
        external_ref := none
        trusted := true
      }
  | _, .trusted =>
      some {
        kind := if boundary == "import" then TrustAssumptionKind.trustedLinker else TrustAssumptionKind.trustedForeignAbi
        description := s!"trusted {boundary} boundary {symbol.fqn}"
        external_ref := none
        trusted := true
      }
  | _, .proofObligation => none

private def trustAssumptionsForModule (m : Module) : List ProofTrustAssumption :=
  let exportAssumptions := m.exports.filterMap (fun exp =>
    trustAssumptionForBoundary "export" exp.symbol exp.contract)
  let importAssumptions := m.imports.filterMap (fun imp =>
    trustAssumptionForBoundary "import" imp.symbol imp.contract)
  exportAssumptions ++ importAssumptions

def generateProofReport (build : CertifiedBuild) (build_id : String) : ProofReport :=
  let ptr_width := match build.modules with
    | [] => 64
    | m :: _ => m.target.ptrWidth
  let endian := match build.modules with
    | [] => .little
    | m :: _ => m.target.endian
  let entries := build.modules.map (fun m =>
    {
      module_name := m.moduleName
      abi_version := 1
      language := m.language
      obligations := obligationsForModule build.modules m
      trust_assumptions := trustAssumptionsForModule m
    })
  let report : ProofReport := {
    build_id := build_id
    timestamp := 0
    target_ptr_width := ptr_width
    target_endian := endian
    modules := entries
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
  { report with summary := ProofReport.compute_summary report }

def generateProofReportForModules (modules : List Module) (build_id : String) : ProofReport :=
  match fullCheck modules with
  | .ok build => generateProofReport build build_id
  | .error _ =>
      let ptr_width := match modules with
        | [] => 64
        | m :: _ => m.target.ptrWidth
      let endian := match modules with
        | [] => .little
        | m :: _ => m.target.endian
      let entries := modules.map (fun m =>
        {
          module_name := m.moduleName
          abi_version := 1
          language := m.language
          obligations := obligationsForModule modules m
          trust_assumptions := trustAssumptionsForModule m
        })
      let report : ProofReport := {
        build_id := build_id
        timestamp := 0
        target_ptr_width := ptr_width
        target_endian := endian
        modules := entries
        summary := {
          total_obligations := 0
          obligations_proved := 0
          obligations_assumed := 0
          obligations_trusted := 0
          obligations_unsupported := 0
          all_proved := false
          has_trusted := false
        }
      }
      { report with summary := ProofReport.compute_summary report }

def proofStatusToString (s : ProofStatus) : String :=
  match s with
  | .implemented => "Implemented"
  | .tested => "Tested"
  | .proved => "Proved"
  | .assumed => "Assumed"
  | .trusted => "Trusted"
  | .unsupported => "Unsupported"

def obligationKindToString (k : ProofObligationKind) : String :=
  ProofObligationKind.display_name k

def trustAssumptionKindToString (k : TrustAssumptionKind) : String :=
  match k with
  | .trustedFunction => "Trusted Function"
  | .trustedAllocator => "Trusted Allocator"
  | .trustedDrop => "Trusted Drop"
  | .trustedLinker => "Trusted Linker"
  | .trustedForeignAbi => "Trusted Foreign ABI"
  | .manualProof => "Manual Review"

def emitProofReportText (r : ProofReport) : String :=
  let header := "=== Chimera Proof Report ===\nBuild: " ++ r.build_id ++
    "\nTarget: " ++ s!"{r.target_ptr_width}" ++ "-bit " ++
    (match r.target_endian with | .little => "little" | .big => "big") ++ " endian\n"
  let trustCount := r.modules.foldl (fun acc m => acc + m.trust_assumptions.length) 0
  let modulesText := r.modules.foldl (fun acc m =>
    acc ++ "\nModule: " ++ m.module_name.fqn ++ "\n" ++
      m.obligations.foldl (fun inner o =>
        inner ++ "  [" ++ obligationKindToString o.kind ++ "] " ++
          o.target.fqn ++ ": " ++ o.description ++
          " [" ++ proofStatusToString o.status ++ "]\n" ++
          (if o.evidence.isEmpty then ""
           else "    evidence: " ++ String.intercalate ", " o.evidence ++ "\n") ++
          (if o.assumptions.isEmpty then ""
           else "    assumptions: " ++ String.intercalate "; " o.assumptions ++ "\n")) "" ++
      m.trust_assumptions.foldl (fun inner assumption =>
        inner ++ "  (trust) [" ++ trustAssumptionKindToString assumption.kind ++ "] " ++
          assumption.description ++ "\n") "") ""
  let summaryText := "\nSummary: " ++ s!"{r.summary.obligations_proved}" ++
    "/" ++ s!"{r.summary.total_obligations}" ++ " proved" ++
    ", " ++ s!"{r.summary.obligations_trusted}" ++ " trusted" ++
    ", " ++ s!"{r.summary.obligations_assumed}" ++ " assumed" ++
    ", " ++ s!"{r.summary.obligations_unsupported}" ++ " unsupported" ++
    ", trust assumptions: " ++ s!"{trustCount}"
  header ++ modulesText ++ summaryText

namespace ProofReport

def fromCertifiedBuild (build : CertifiedBuild) (build_id : String) : ProofReport :=
  generateProofReport build build_id

def fromModules (modules : List Module) (build_id : String) : ProofReport :=
  generateProofReportForModules modules build_id

end ProofReport

end Chimera.Metadata
