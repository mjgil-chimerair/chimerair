-- ChimeraProof ABI: Contract
-- Function contract specification.

import Chimera.Foundation
import Chimera.ABI.Type
import Chimera.ABI.PhysicalType
import Chimera.ABI.Signature

namespace Chimera

/--
Panic policy for a function.
-/
inductive PanicPolicy : Type where
  | abort : PanicPolicy
  | catchUnwind : PanicPolicy
  | forbidden : PanicPolicy
deriving Repr, BEq

/--
Safety classification of a function.
-/
inductive SafetyClass : Type where
  | verified : SafetyClass  -- Proven safe, can be called from safe code
  | generatedWrapper : SafetyClass  -- Generated safe wrapper
  | trustedContract : SafetyClass  -- Trusted but unverified
  | unsafeContract : SafetyClass  -- Explicitly unsafe
deriving BEq, Repr

/--
ABI function forms for classification.
-/
inductive AbiForm : Type where
  | infallible : AbiForm  -- Cannot fail, returns value or unit
  | fallible : AbiForm  -- Can return error via ch_status
  | constructor : AbiForm  -- Creates an owned resource
  | destructor : AbiForm  -- Releases resources, no return value
  | callback : AbiForm  -- Callable from C/rust/zig, passed as function pointer
  | unsafeRaw : AbiForm  -- Raw FFI with minimal safety guarantees
deriving BEq, Repr

/--
Can this safety class call the given safety class?
Safe code may call verified/generated boundaries.
Trusted boundaries require explicit policy.
Unsafe requires unsafe context.
-/
def SafetyClass.canCall (caller : SafetyClass) (callee : SafetyClass) : Bool :=
  match caller with
  | .unsafeContract => true  -- Unsafe can call anything
  | .trustedContract => callee != .unsafeContract  -- Trusted can call anyone except unsafe
  | .generatedWrapper => callee == .verified || callee == .generatedWrapper  -- Generated only calls safe
  | .verified => callee == .verified || callee == .generatedWrapper  -- Verified only calls verified/generated

/--
Effect set for a function.
-/
inductive Effect : Type where
  | pure : Effect
  | mayAlloc : Effect
  | mayDealloc : Effect
  | mayError : Effect
  | mayPanic : Effect
  | mayAbort : Effect
  | mayBlock : Effect
  | mayCallback : Effect
  | mayTouchRaw : Effect
  | mayReadGlobal : Effect
  | mayWriteGlobal : Effect
  | threadSafe : Effect
  | notThreadSafe : Effect
deriving BEq, Repr, Hashable

/--
Effect set as a finite set stored as duplicate-free list.
-/
def EffectSet := List Effect

/--
Check if an effect is in a set.
-/
def memberEffect : EffectSet → Effect → Bool
  | [], _ => false
  | e :: rest, f => e == f || memberEffect rest f

namespace Effect

/--
Deterministic enumeration of the finite effect universe.
-/
def all : List Effect :=
  [.pure, .mayAlloc, .mayDealloc, .mayError, .mayPanic, .mayAbort, .mayBlock,
   .mayCallback, .mayTouchRaw, .mayReadGlobal, .mayWriteGlobal, .threadSafe,
   .notThreadSafe]

end Effect

/--
Canonicalize effect set by removing duplicates in deterministic order.
-/
def EffectSet.canonicalize (es : EffectSet) : EffectSet :=
  Effect.all.filter (fun e => memberEffect es e)

/--
Add an effect to a set.
-/
def insertEffect : EffectSet → Effect → EffectSet
  | [], f => [f]
  | e :: rest, f => if e == f then e :: rest else e :: insertEffect rest f

/--
Effect subset relation.
-/
def EffectSubset : EffectSet → EffectSet → Prop
  | [], _ => True
  | e :: rest, set => memberEffect set e ∧ EffectSubset rest set

/--
Compose effects from multiple functions.
Results in canonical (duplicate-free) set.
-/
def composeEffectSets : List EffectSet → EffectSet
  | [] => []
  | sets => EffectSet.canonicalize (List.foldr (fun es result => List.append es result) [] sets)

/--
Trust assumptions for a function contract.
-/
inductive TrustAssumption : Type where
  | trusted : TrustAssumption  -- Trusted C/external contract
  | proofObligation : TrustAssumption  -- Requires proof before use
  | unchecked : TrustAssumption  -- Unchecked unsafe boundary
deriving BEq, Repr

/--
Function contract specification.
-/
structure FunctionContract : Type where
  symbol : Symbol
  language : SourceLanguage
  form : AbiForm := .infallible
  semanticSig : SemanticSignature
  physicalSig : PhysicalSignature
  effects : EffectSet
  panicPolicy : PanicPolicy
  safety : SafetyClass
  allocator : Option Symbol
  /-- Whether the function requires drop for owned resources -/
  requiresDrop : Bool := false
  /-- Trust assumption for this contract -/
  trust : TrustAssumption := .proofObligation
  /-- Error domain if fallible -/
  errorDomain : Option ErrorDomain := none

/--
Check if this is a safe contract.
-/
def FunctionContract.isSafe (c : FunctionContract) : Bool :=
  match c.safety with
  | SafetyClass.verified => true
  | SafetyClass.generatedWrapper => true
  | SafetyClass.trustedContract => false
  | SafetyClass.unsafeContract => false

/--
Check if this contract contains unchecked raw pointers.
-/
def FunctionContract.containsUncheckedRaw (c : FunctionContract) : Bool :=
  c.semanticSig.returns.containsRawPtr

/--
Check if this is a fallible function form.
-/
def FunctionContract.isFallible (c : FunctionContract) : Bool :=
  match c.form with
  | .fallible => true
  | _ => false

/--
Valid contract predicate.
-/
def ValidContract (c : FunctionContract) : Prop :=
  match c.safety with
  | SafetyClass.unsafeContract => True
  | _ => ¬ c.containsUncheckedRaw

namespace EffectLatticeLaws

/--
Membership is preserved by canonicalization.
-/
theorem memberEffect_canonicalize (es : EffectSet) (e : Effect) :
  memberEffect (EffectSet.canonicalize es) e = memberEffect es e := by
  cases e <;> simp [EffectSet.canonicalize, Effect.all, memberEffect]

/--
Effects from the left side of an append remain present.
-/
theorem memberEffect_append_left {a b : EffectSet} {e : Effect} :
  memberEffect a e = true → memberEffect (a ++ b) e = true := by
  induction a with
  | nil =>
      intro h
      simp at h
  | cons head tail ih =>
      intro h
      by_cases hHead : head == e
      · simp [memberEffect, hHead]
      · simp [memberEffect, hHead] at h ⊢
        exact ih h

/--
Effects from the right side of an append remain present.
-/
theorem memberEffect_append_right {a b : EffectSet} {e : Effect} :
  memberEffect b e = true → memberEffect (a ++ b) e = true := by
  induction a with
  | nil =>
      intro h
      simpa [memberEffect] using h
  | cons head tail ih =>
      intro h
      by_cases hHead : head == e
      · simp [memberEffect, hHead]
      · simp [memberEffect, hHead]
        exact ih h

/--
Theorem: EffectSubset is reflexive.
-/
theorem EffectSubset_reflexive (es : EffectSet) : EffectSubset es es := by
  intro e h
  exact h

/--
Theorem: EffectSubset is transitive.
-/
theorem EffectSubset_transitive (a b c : EffectSet) :
  EffectSubset a b → EffectSubset b c → EffectSubset a c := by
  intro hAB hBC e hMem
  exact hBC e (hAB e hMem)

/--
Theorem: mutual subset implies equal canonical sets.
-/
theorem EffectSubset_antisymmetric (a b : EffectSet) :
  EffectSubset a b → EffectSubset b a →
    EffectSet.canonicalize a = EffectSet.canonicalize b := by
  intro hAB hBA
  unfold EffectSet.canonicalize
  have hPred : (fun e => memberEffect a e) = (fun e => memberEffect b e) := by
    funext e
    cases hA : memberEffect a e <;> cases hB : memberEffect b e <;> try rfl
    · have : memberEffect b e = true := hAB e hA
      cases hB.trans this.symm
    · have : memberEffect a e = true := hBA e hB
      cases hA.trans this.symm
  simp [hPred]

/--
Union law: composing two sets contains the left operand.
-/
theorem composeEffectSets_contains_left (a b : EffectSet) :
  EffectSubset a (composeEffectSets [a, b]) := by
  intro e hMem
  have hApp : memberEffect (a ++ b) e = true := memberEffect_append_left hMem
  unfold composeEffectSets
  rw [memberEffect_canonicalize (a ++ b) e]
  exact hApp

/--
Union law: composing two sets contains the right operand.
-/
theorem composeEffectSets_contains_right (a b : EffectSet) :
  EffectSubset b (composeEffectSets [a, b]) := by
  intro e hMem
  have hApp : memberEffect (a ++ b) e = true := memberEffect_append_right hMem
  unfold composeEffectSets
  rw [memberEffect_canonicalize (a ++ b) e]
  exact hApp

/--
Canonicalization is idempotent.
-/
theorem canonicalize_idempotent (es : EffectSet) :
  EffectSet.canonicalize (EffectSet.canonicalize es) = EffectSet.canonicalize es := by
  apply EffectSubset_antisymmetric
  · intro e hMem
    rw [memberEffect_canonicalize (EffectSet.canonicalize es) e] at hMem
    rw [memberEffect_canonicalize es e]
    exact hMem
  · intro e hMem
    rw [memberEffect_canonicalize es e] at hMem
    rw [memberEffect_canonicalize (EffectSet.canonicalize es) e]
    exact hMem

/--
Theorem: EffectSubset is reflexive (alternative).
-/
theorem EffectSubset_refl (es : EffectSet) : EffectSubset es es := EffectSubset_reflexive es

end EffectLatticeLaws

end Chimera
