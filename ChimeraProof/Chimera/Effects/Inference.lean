-- ChimeraProof Effects: Inference
-- Effect inference for function contracts.

import Chimera.ABI
import Chimera.ABI.Contract
import Chimera.ABI.Type

namespace Chimera

/--
Inferred effect summary for a function.
-/
structure InferredEffects where
  mayAlloc : Bool := false
  mayDealloc : Bool := false
  mayError : Bool := false
  mayPanic : Bool := false
  mayBlock : Bool := false
  mayCallback : Bool := false
  mayTouchRaw : Bool := false
  mayReadGlobal : Bool := false
  mayWriteGlobal : Bool := false
  threadSafe : Bool := false

namespace InferredEffects

/--
Merge two inferred effect sets.
-/
def merge (a b : InferredEffects) : InferredEffects :=
  { mayAlloc := a.mayAlloc || b.mayAlloc,
    mayDealloc := a.mayDealloc || b.mayDealloc,
    mayError := a.mayError || b.mayError,
    mayPanic := a.mayPanic || b.mayPanic,
    mayBlock := a.mayBlock || b.mayBlock,
    mayCallback := a.mayCallback || b.mayCallback,
    mayTouchRaw := a.mayTouchRaw || b.mayTouchRaw,
    mayReadGlobal := a.mayReadGlobal || b.mayReadGlobal,
    mayWriteGlobal := a.mayWriteGlobal || b.mayWriteGlobal,
    threadSafe := a.threadSafe && b.threadSafe }

/--
Convert inferred effects to effect set.
C.63: Effect sets are normalized to be duplicate-free.
-/
def toEffectSet (ie : InferredEffects) : EffectSet :=
  let es := []
  let es := if ie.mayAlloc then .mayAlloc :: es else es
  let es := if ie.mayDealloc then .mayDealloc :: es else es
  let es := if ie.mayError then .mayError :: es else es
  let es := if ie.mayPanic then .mayPanic :: es else es
  let es := if ie.mayBlock then .mayBlock :: es else es
  let es := if ie.mayCallback then .mayCallback :: es else es
  let es := if ie.mayTouchRaw then .mayTouchRaw :: es else es
  let es := if ie.mayReadGlobal then .mayReadGlobal :: es else es
  let es := if ie.mayWriteGlobal then .mayWriteGlobal :: es else es
  let es := if ie.threadSafe then .threadSafe :: es else .notThreadSafe :: es
  -- C.63: Deduplicate the effect list
  dedup es

end InferredEffects

/--
Infer effects from a semantic type.
-/
def inferFromType : ChType → InferredEffects
  | .owned _ => { mayAlloc := true, mayDealloc := true }
  | .result okTy errTy => InferredEffects.merge (inferFromType okTy) (inferFromType errTy)
  | .slice _ _ => { mayAlloc := true, mayDealloc := true }
  | .str _ _ => { mayAlloc := true, mayDealloc := true }
  | .rawptr _ => { mayTouchRaw := true }
  | _ => {}

/--
Infer effects from a semantic signature.
-/
def inferFromSignature (sig : SemanticSignature) : InferredEffects :=
  let ie := sig.params.foldl (fun acc p => InferredEffects.merge acc (inferFromType p.ty)) {}
  match sig.returns with
  | .result okTy errTy => InferredEffects.merge ie (InferredEffects.merge (inferFromType okTy) (inferFromType errTy))
  | .unit => ie
  | ty => InferredEffects.merge ie (inferFromType ty)

end Chimera
