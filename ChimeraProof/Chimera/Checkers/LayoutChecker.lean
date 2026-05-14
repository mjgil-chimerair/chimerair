-- ChimeraProof Checkers: Layout Checker
-- Executable layout validation.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Layout
import Chimera.ABI.PhysicalType
import Chimera.IR.Module

namespace Chimera

inductive LayoutCheckError where
  | layoutMismatch (name : Symbol) (expected : Nat) (got : Nat)
  | alignmentMismatch (name : Symbol) (expected : Nat) (got : Nat)
  | fieldMismatch (name : Symbol) (field : String)
  | hashMismatch (name : Symbol)
deriving Repr, BEq

structure CheckedLayout where
  name : Symbol
  layout : Layout
  validated : Bool := true

private def layoutErrorToNat : LayoutError → Nat
  | .notPowerOfTwo w => w
  | .misalignment off _ => off
  | .structTooLarge s => s
  | .arraySizeZero => 0

private def checkDeclaredFields
  (decl : DeclaredLayout)
  (formal : Layout) : Except LayoutCheckError Unit := do
  let declFields := DeclaredLayout.fields decl
  if declFields.length != formal.fields.length then
    .error (.fieldMismatch decl.name "")
  List.zip declFields formal.fields |>.forM (fun (pair : DeclaredField × FieldLayout) => do
    let declF := pair.1
    let formalF := pair.2
    if declF.name != formalF.fieldName then
      .error (.fieldMismatch decl.name declF.name)
    if declF.offset != formalF.offset then
      .error (.fieldMismatch decl.name declF.name)
    if declF.size != formalF.size then
      .error (.layoutMismatch decl.name formalF.size declF.size)
    if declF.align != formalF.align then
      .error (.alignmentMismatch decl.name formalF.align declF.align)
  )

def checkDeclaredLayout
  (target : Target)
  (decl : DeclaredLayout)
  (physTy : PhysType) :
  Except LayoutCheckError CheckedLayout := do
  let formal ← Layout.layoutOf target physTy |>.mapError (fun e => .layoutMismatch decl.name (layoutErrorToNat e) decl.size)
  if formal.size != decl.size then
    .error (.layoutMismatch decl.name formal.size decl.size)
  if formal.align != decl.align then
    .error (.alignmentMismatch decl.name formal.align decl.align)
  let declFields : List DeclaredField := DeclaredLayout.fields decl
  if declFields = [] then
    pure ()
  else
    checkDeclaredFields decl formal
  .ok { name := decl.name, layout := formal, validated := true }

def checkStructLayout
  (target : Target)
  (fields : List (String × PhysType)) :
  Except LayoutCheckError Layout :=
  Layout.layoutOf target (.struct fields) |>.mapError (fun e => .layoutMismatch { ns := "", name := "" } (layoutErrorToNat e) 0)

def checkNoOverlap (layout : Layout) : Bool :=
  Layout.pairwiseDisjoint layout.fields

end Chimera
