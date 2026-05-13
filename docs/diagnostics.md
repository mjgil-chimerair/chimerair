# Checker Diagnostics

This document describes the diagnostic codes and messages produced by ChimeraIR checkers.

## Diagnostic Categories

Diagnostics are grouped by checker phase:
- Metadata diagnostics (module-level validation)
- Layout diagnostics (type layout verification)
- Ownership diagnostics (resource ownership validation)
- Allocator diagnostics (allocator registration)
- Result diagnostics (Result/error bridge)
- Panic diagnostics (panic policy)
- Contract diagnostics (function contracts)
- Link diagnostics (symbol resolution, module composition)

## Diagnostic Code Format

Each diagnostic has:
- **Code**: Unique identifier (e.g., `META_001`)
- **Severity**: `error` | `warning` | `info`
- **Message**: Human-readable description
- **Details**: Additional context (optional)

## Metadata Diagnostics (META_*)

| Code | Severity | Message | Details |
|------|----------|---------|---------|
| META_001 | error | Invalid ABI version | Expected "0.1", got `{version}` |
| META_002 | error | Empty module name | Module name cannot be empty |
| META_003 | warning | Empty exports | Module exports nothing |
| META_004 | error | Duplicate import | `{symbol}` imported multiple times |
| META_005 | error | Duplicate export | `{symbol}` exported multiple times |
| META_006 | error | Invalid type size | Type `{name}` has zero size |

## Layout Diagnostics (LAYOUT_*)

| Code | Severity | Message | Details |
|------|----------|---------|---------|
| LAYOUT_001 | error | Missing field | Field `{field}` not in layout |
| LAYOUT_002 | error | Wrong field | Expected field `{expected}`, got `{got}` |
| LAYOUT_003 | error | Wrong offset | Field `{field}` at offset `{got}`, expected `{expected}` |
| LAYOUT_004 | error | Wrong size | Field `{field}` has size `{got}`, expected `{expected}` |
| LAYOUT_005 | error | Wrong alignment | Field `{field}` has alignment `{got}`, expected `{expected}` |
| LAYOUT_006 | error | Invalid width | Integer width `{w}` not supported |
| LAYOUT_007 | error | Zero alignment | Alignment cannot be zero |
| LAYOUT_008 | error | Zero array length | Array length cannot be zero |

## Ownership Diagnostics (OWN_*)

| Code | Severity | Message | Details |
|------|----------|---------|---------|
| OWN_001 | error | Double ownership | Block `{id}` has multiple owners |
| OWN_002 | error | Write borrow alias | Write borrow conflicts with existing borrow |
| OWN_003 | error | Invalid ownership transition | Cannot transition from `{from}` to `{to}` |
| OWN_004 | error | Use after drop | Block `{id}` already dropped |
| OWN_005 | error | Use after move | Block `{id}` already moved |
| OWN_006 | error | Borrow escape | Borrow escapes its lifetime |

## Allocator Diagnostics (ALLOC_*)

| Code | Severity | Message | Details |
|------|----------|---------|---------|
| ALLOC_001 | error | Duplicate allocator | Allocator `{id}` already registered |
| ALLOC_002 | error | Allocator mismatch | Expected `{expected}`, got `{got}` |
| ALLOC_003 | warning | Missing drop | No drop function for type `{name}` |
| ALLOC_004 | error | Invalid allocator | Allocator `{id}` not found |

## Result Diagnostics (RES_*)

| Code | Severity | Message | Details |
|------|----------|---------|---------|
| RES_001 | error | Non-zero error status | Status must be 0 on success, got `{status}` |
| RES_002 | error | Missing out param | Output parameter required for `{type}` |
| RES_003 | error | Unexpected payload | Payload on success path |
| RES_004 | error | Invalid error domain | Domain `{domain}` not valid for error bridge |

## Panic Diagnostics (PANIC_*)

| Code | Severity | Message | Details |
|------|----------|---------|---------|
| PANIC_001 | error | Unwind not allowed | Unwinding prohibited by panic policy |
| PANIC_002 | error | Wrong panic policy | Policy `{policy}` does not allow panic |
| PANIC_003 | error | Abort without abortable | Abort policy requires abortable function |
| PANIC_004 | warning | Catch all | Catching all panics may hide bugs |

## Contract Diagnostics (CONTRACT_*)

| Code | Severity | Message | Details |
|------|----------|---------|---------|
| CONTRACT_001 | error | Missing allocator | No allocator specified in contract |
| CONTRACT_002 | error | Invalid effect set | Effects `{effects}` invalid for `{symbol}` |
| CONTRACT_003 | error | Unsafe call | `{caller}` (safety `{caller_safety}`) cannot call `{callee}` (safety `{callee_safety}`) |
| CONTRACT_004 | error | Invalid signature | Semantic/physical signature mismatch |
| CONTRACT_005 | error | Missing layout | Layout not declared for `{type}` |
| CONTRACT_006 | error | Raw pointer in safe contract | Safe contract cannot contain raw pointers |
| CONTRACT_007 | error | Missing panic policy | Fallible function requires panic policy |
| CONTRACT_008 | warning | Missing drop flag | Owned resources may leak without explicit drop flag |

## Link Diagnostics (LINK_*)

| Code | Severity | Message | Details |
|------|----------|---------|---------|
| LINK_001 | error | Unresolved import | Symbol `{symbol}` not found in module `{module}` |
| LINK_002 | error | Duplicate symbol | Symbol `{symbol}` defined multiple times |
| LINK_003 | error | Target mismatch | Module `{module}` targets `{target1}`, expected `{target2}` |
| LINK_004 | error | Incompatible signature | Import `{symbol}` signature incompatible with export |
| LINK_005 | error | Weak symbol override | Weak symbol `{symbol}` cannot override strong |
| LINK_006 | error | Visibility conflict | Private symbol `{symbol}` cannot be exported |

## Checker Error Types

Each checker produces a specific error type:

```lean
-- Metadata checker
inductive MetadataCheckError where
  | invalidVersion (got : String)
  | emptyModuleName
  | duplicateImport (sym : Symbol)
  | duplicateExport (sym : Symbol)
  | invalidTypeSize (name : String)

-- Layout checker
inductive LayoutCheckError where
  | missingField (field : String)
  | wrongField (expected got : String)
  | wrongOffset (field : String) (expected got : Nat)
  | wrongSize (field : String) (expected got : Nat)
  | wrongAlignment (field : String) (expected got : Nat)
  | invalidWidth (w : Nat)
  | zeroAlignment
  | zeroArrayLength

-- Ownership checker
inductive OwnershipCheckError where
  | doubleOwn (id : BlockId)
  | writeBorrowAlias (id : BlockId)
  | invalidTransition (from to : OwnershipState)
  | useAfterDrop (id : BlockId)
  | useAfterMove (id : BlockId)
  | borrowEscape (id : BlockId)

-- Contract checker
inductive ContractCheckError where
  | missingAllocator (sym : Symbol)
  | invalidEffectSet (symbol : Symbol)
  | unsafeCall (caller sym : Symbol)
  | invalidSignature (sym : Symbol)
  | missingLayout (ty : String)
  | rawPointerInSafe (sym : Symbol)
  | missingPanicPolicy (sym : Symbol)
  | missingDropFlag (sym : Symbol)

-- Result checker
inductive ResultCheckError where
  | nonZeroErrorStatus (status : Int32)
  | missingOutParam (ty : ChType)
  | unexpectedPayloadOnSuccess
  | invalidErrorDomain (domain : ErrorDomain)

-- Panic checker
inductive PanicCheckError where
  | unwindNotAllowed
  | panicWithWrongPolicy (policy : PanicPolicy)
  | abortWithoutAbortable
```

## Integration Notes

All checker errors implement:
- `Repr` for debugging output
- `BEq` for equality comparison
- `Except` error handling via `Except.error` constructors

The `FullChecker` accumulates errors from all phases and returns a `CertifiedBuild` on success or the first error encountered.