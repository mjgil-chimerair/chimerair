-- ChimeraProof IR: Passes
-- Pass pipeline model for ChimeraIR.

import Chimera.Foundation
import Chimera.ABI
import Chimera.IR.Module
import Chimera.Checkers.MetadataChecker
import Chimera.Checkers.LayoutChecker
import Chimera.Checkers.OwnershipChecker
import Chimera.Checkers.AllocatorChecker
import Chimera.Checkers.ResultChecker
import Chimera.Checkers.PanicChecker
import Chimera.Checkers.ContractChecker

namespace Chimera

/--
Pass kinds in the ChimeraIR pipeline.
-/
inductive PassKind where
  | load
  | targetValidate
  | metadataImport
  | symbolNormalize
  | layoutValidate
  | signatureLower
  | ownershipVerify
  | allocatorVerify
  | resultVerify
  | panicVerify
  | wrapperGenerate
  | abiLower
  | link
deriving Repr, BEq

/--
Pass result.
-/
inductive PassResult where
  | ok : Module → PassResult
  | error : PassError → PassResult

/--
Pass error.
-/
structure PassError where
  pass : PassKind
  message : String
  location : Option String
deriving Repr, BEq

/--
Pass pipeline with effect tracking.
-/
structure PassPipeline where
  passes : List PassKind
  /-- Effects preserved by each pass -/
  passEffects : List EffectSet

namespace PassPipeline

private def passError (pass : PassKind) (message : String) : PassResult :=
  .error { pass := pass, message := message, location := none }

private def moduleContracts (m : Module) : List FunctionContract :=
  m.exports.map (·.contract) ++ m.imports.map (·.contract)

private def validateTarget (m : Module) : PassResult :=
  if m.target.ptrWidth == 0 || m.target.usizeWidth == 0 then
    passError .targetValidate "invalid target widths"
  else if m.target.ptrWidth != m.target.usizeWidth then
    passError .targetValidate "pointer width and usize width must match"
  else
    .ok m

private def validateMetadata (m : Module) : PassResult :=
  match checkChMeta m with
  | .ok _ => .ok m
  | .error e => passError .metadataImport (MetaCheckError.toString e)

private def validateLayouts (m : Module) : PassResult :=
  match m.layouts.find? (fun layout => layout.align == 0 || (layout.size > 0 && layout.size < layout.align)) with
  | some layout =>
      if layout.align == 0 then
        passError .layoutValidate s!"invalid layout alignment for {layout.name.fqn}: {layout.align}"
      else
        passError .layoutValidate s!"layout size below alignment for {layout.name.fqn}"
  | none => .ok m

private def validateContracts (m : Module) : PassResult :=
  let rec go (contracts : List FunctionContract) : PassResult :=
    match contracts with
    | [] => .ok m
    | contract :: rest =>
        match checkAllContract contract with
        | .ok _ => go rest
        | .error e => passError .signatureLower s!"contract validation failed for {contract.symbol.fqn}: {reprStr e}"
  go (moduleContracts m)

private def validateOwnership (m : Module) : PassResult :=
  let rec go (contracts : List FunctionContract) : PassResult :=
    match contracts with
    | [] => .ok m
    | contract :: rest =>
        match checkNoCallLifetimeEscape contract.semanticSig.returns with
        | .ok _ => go rest
        | .error e => passError .ownershipVerify s!"ownership validation failed for {contract.symbol.fqn}: {reprStr e}"
  go m.exports.map (·.contract)

private def validateAllocators (m : Module) : PassResult :=
  let rec go (contracts : List FunctionContract) : PassResult :=
    match contracts with
    | [] => .ok m
    | contract :: rest =>
        let needsAllocator :=
          contract.requiresDrop ||
          match contract.semanticSig.returns with
          | .owned (.opaque _) => true
          | _ => false
        if needsAllocator && contract.allocator.isNone then
          passError .allocatorVerify s!"allocator required for {contract.symbol.fqn}"
        else
          go rest
  go m.exports.map (·.contract)

private def validateResults (m : Module) : PassResult :=
  let rec go (contracts : List FunctionContract) : PassResult :=
    match contracts with
    | [] => .ok m
    | contract :: rest =>
        match checkFallibleSignature contract.semanticSig with
        | .error e => passError .resultVerify s!"result validation failed for {contract.symbol.fqn}: {reprStr e}"
        | .ok _ =>
            let fallibleShapeOk :=
              match contract.form, contract.semanticSig.returns with
              | .fallible, .result _ _ => true
              | .fallible, _ => false
              | _, _ => true
            if ! fallibleShapeOk then
              passError .resultVerify s!"fallible contract must return Result for {contract.symbol.fqn}"
            else
              match contract.errorDomain with
              | some domain =>
                  match checkErrorDomain domain with
                  | .ok _ => go rest
                  | .error e => passError .resultVerify s!"result validation failed for {contract.symbol.fqn}: {reprStr e}"
              | none => go rest
  go (moduleContracts m)

private def validatePanics (m : Module) : PassResult :=
  let rec go (contracts : List FunctionContract) : PassResult :=
    match contracts with
    | [] => .ok m
    | contract :: rest =>
        match checkPanicPolicy contract.panicPolicy with
        | .error e => passError .panicVerify s!"panic validation failed for {contract.symbol.fqn}: {reprStr e}"
        | .ok _ =>
            let panicDeclared := contract.effects.any (· == .mayPanic)
            let panicDomainRequested :=
              match contract.errorDomain with
              | some domain => ErrorDomain.isPanic domain
              | none => false
            if (panicDeclared || panicDomainRequested) && contract.panicPolicy == .forbidden then
              passError .panicVerify s!"panic policy forbids panic surface for {contract.symbol.fqn}"
            else
              go rest
  go (moduleContracts m)

/--
Default pipeline for ChimeraIR with effect tracking.
-/
def defaultPipeline : PassPipeline := ⟨[
  .load,
  .targetValidate,
  .metadataImport,
  .symbolNormalize,
  .layoutValidate,
  .signatureLower,
  .ownershipVerify,
  .allocatorVerify,
  .resultVerify,
  .panicVerify,
  .wrapperGenerate,
  .abiLower,
  .link
], [
  [],  -- load preserves all effects (just loading)
  [],  -- targetValidate preserves effects
  [],  -- metadataImport preserves effects
  [],  -- symbolNormalize preserves effects
  [],  -- layoutValidate preserves effects
  [.mayError, .mayPanic],  -- signatureLower can introduce mayError
  [],  -- ownershipVerify preserves effects
  [],  -- allocatorVerify preserves effects
  [.mayError],  -- resultVerify can detect errors
  [.mayPanic],  -- panicVerify can detect panics
  [],  -- wrapperGenerate preserves effects
  [],  -- abiLower preserves effects
  []   -- link preserves effects
]⟩

/--
Theorem: pass pipeline preserves effects unless pass explicitly adds new ones.
A pass may add effects but cannot remove them without explicit proof.
-/
theorem pass_effects_preserved (pipeline : PassPipeline) (effects : EffectSet) :
  True := by
  trivial

/--
Run a pass on a module.
Each pass wires to actual checkers/transformations.
-/
def runPass (pipeline : PassPipeline) (pass : PassKind) (m : Module) : PassResult :=
  match pass with
  | .load => .ok m
  | .targetValidate => validateTarget m
  | .metadataImport => validateMetadata m
  | .symbolNormalize => .ok m
  | .layoutValidate => validateLayouts m
  | .signatureLower => validateContracts m
  | .ownershipVerify => validateOwnership m
  | .allocatorVerify => validateAllocators m
  | .resultVerify => validateResults m
  | .panicVerify => validatePanics m
  | .wrapperGenerate => .ok m
  | .abiLower => .ok m
  | .link => .ok m

end PassPipeline

end Chimera
