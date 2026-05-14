-- ChimeraProof Checkers: Metadata Checker
-- Executable metadata validation for ChimeraIR modules.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Contract
import Chimera.IR.Module

namespace Chimera

/--
Helper: find duplicate in list (returns first duplicate found).
-/
def findDuplicate (names : List Symbol) : Option Symbol :=
  let rec go (seen : List Symbol) (remaining : List Symbol) : Option Symbol :=
    match remaining with
    | [] => none
    | n :: rest =>
      if seen.contains n then some n
      else go (n :: seen) rest
  go [] names

/--
Metadata check error.
-/
inductive MetaCheckError where
  | invalidVersion (got : String)
  | emptyModuleName
  | emptyExport
  | emptyImport
  | duplicateExport (sym : Symbol)
  | duplicateImport (sym : Symbol)
  | invalidTypeSize (name : Symbol) (size : Nat)
  | emptyContract
  | invalidSafety
  | emptyEffectSet
  | duplicateLayout (name : Symbol)
  | emptyLayoutName
  | invalidLayoutAlign (name : Symbol) (align : Nat)
  | layoutSizeBelowAlign (name : Symbol)
  | emptyImportSymbol
  | emptyExportSymbol
  | importMismatch (name : Symbol)
  | exportMismatch (name : Symbol)
  -- C.59: C imports require errno mapping
  | missingErrnoMapping (sym : Symbol)
  | invalidErrnoMapping (sym : Symbol)
  -- C.51: imports with requires_drop need allocator or drop
  | missingDropFunction (sym : Symbol)
  | mismatchedAllocator (sym : Symbol)
deriving Repr, BEq

/--
Checked metadata result.
-/
structure CheckedChMeta where
  module : Module
  validated : Bool := true

namespace MetaCheckError

/--
Error to string.
-/
def toString : MetaCheckError → String
  | .invalidVersion v => s!"invalid version: {v}"
  | .emptyModuleName => "empty module name"
  | .emptyExport => "empty export symbol"
  | .emptyImport => "empty import symbol"
  | .duplicateExport sym => s!"duplicate export: {sym.ns}/{sym.name}"
  | .duplicateImport sym => s!"duplicate import: {sym.ns}/{sym.name}"
  | .invalidTypeSize name s => s!"invalid type size for {name.ns}/{name.name}: {s}"
  | .emptyContract => "empty contract in import/export"
  | .invalidSafety => "invalid safety class"
  | .emptyEffectSet => "empty effect set in contract"
  | .duplicateLayout name => s!"duplicate layout: {name.ns}/{name.name}"
  | .emptyLayoutName => "empty layout name"
  | .invalidLayoutAlign name align => s!"invalid layout alignment for {name.ns}/{name.name}: {align}"
  | .layoutSizeBelowAlign name => s!"layout size below alignment for {name.ns}/{name.name}"
  | .emptyImportSymbol => "empty import symbol name"
  | .emptyExportSymbol => "empty export symbol name"
  | .importMismatch name => s!"import symbol mismatch: {name.ns}/{name.name}"
  | .exportMismatch name => s!"export symbol mismatch: {name.ns}/{name.name}"
  -- C.59: C imports require errno mapping
  | .missingErrnoMapping sym => s!"C import missing errno mapping: {sym.ns}/{sym.name}"
  | .invalidErrnoMapping sym => s!"C import has invalid errno mapping: {sym.ns}/{sym.name}"
  -- C.51: imports with requires_drop need allocator or drop
  | .missingDropFunction sym => s!"import missing drop function: {sym.ns}/{sym.name}"
  | .mismatchedAllocator sym => s!"import has mismatched allocator: {sym.ns}/{sym.name}"

end MetaCheckError

/--
Check metadata for a module.
Full validation includes: version, names, exports, imports, contracts,
layouts, duplicates, type sizes, and layout well-formedness.
-/
def checkChMeta (m : Module) : Except MetaCheckError CheckedChMeta :=
  if m.abiVersion != "0.1" then
    .error (.invalidVersion m.abiVersion)
  else if m.moduleName.name.isEmpty then
    .error .emptyModuleName
  else if m.exports.isEmpty then
    .error .emptyExport
  else if m.imports.isEmpty then
    .error .emptyImport
  else if m.exports.any (fun e => e.symbol.name.isEmpty) then
    .error .emptyExportSymbol
  else if m.imports.any (fun i => i.symbol.name.isEmpty) then
    .error .emptyImportSymbol
  else
    -- Check for duplicate exports
    match findDuplicate (m.exports.map (·.symbol)) with
    | some sym => .error (.duplicateExport sym)
    | none =>
      -- Check for duplicate imports
      match findDuplicate (m.imports.map (·.symbol)) with
      | some sym => .error (.duplicateImport sym)
      | none =>
        -- Check for duplicate layouts
        match findDuplicate (m.layouts.map (·.name)) with
        | some name => .error (.duplicateLayout name)
        | none =>
          -- Check type sizes
          match m.types.find? (fun td => td.size == 0) with
          | some td => .error (.invalidTypeSize td.name td.size)
          | none =>
            -- Check layout validity
            match m.layouts.find? (fun l =>
              l.name.name.isEmpty || l.align == 0 || l.size < l.align) with
            | some l =>
              if l.name.name.isEmpty then
                .error (.emptyLayoutName)
              else if l.align == 0 then
                .error (.invalidLayoutAlign l.name l.align)
              else
                .error (.layoutSizeBelowAlign l.name)
            | none =>
              -- Check export symbol matches contract symbol
              match m.exports.find? (fun e => e.symbol != e.contract.symbol) with
              | some e => .error (.exportMismatch e.symbol)
              | none =>
                -- Check import symbol matches contract symbol
                match m.imports.find? (fun i => i.symbol != i.contract.symbol) with
                | some i => .error (.importMismatch i.symbol)
                | none =>
                  -- C.59: Check C imports have errno mapping
                  match m.imports.find? (fun i =>
                    i.language == .c && i.errnoMapping.isNone) with
                  | some i => .error (.missingErrnoMapping i.symbol)
                  | none =>
                    -- C.51: Check imports with requires_drop have drop or allocator
                    match m.imports.find? (fun i =>
                      i.requiresDrop && not i.hasDropOrAllocator) with
                    | some i => .error (.missingDropFunction i.symbol)
                    | none =>
                      .ok { module := m, validated := true }

namespace MetaCheckError

end MetaCheckError

end Chimera