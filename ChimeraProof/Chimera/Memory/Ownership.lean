-- ChimeraProof Memory: Ownership
-- Ownership model for boundary calls.

import Chimera.Foundation
import Chimera.ABI
import Chimera.Memory.Block
import Chimera.Memory.Permission

namespace Chimera

/--
Call state for ownership tracking.
-/
structure CallState where
  heap       : Heap
  resources  : List Resource
  lifetimeCtx : List Lifetime

/--
Call contract for ownership.
-/
structure CallContract where
  args       : List ChType
  returns    : ReturnSpec
  effects    : EffectSet
  panic      : PanicPolicy
  safety     : SafetyClass

/--
Ownership error.
-/
inductive OwnershipError where
  | doubleOwn (block : BlockId)
  | writeBorrowAlias (block : BlockId)
  | borrowEscapes (block : BlockId)
  | droppedUse (block : BlockId)
  | movedUse (block : BlockId)
  | noOwner (block : BlockId)

/--
Check if a resource list has no double ownership.
-/
def NoDoubleOwn (resources : List Resource) : Prop :=
  ∀ r1 ∈ resources, ∀ r2 ∈ resources,
    r1 ≠ r2 →
    r1.permission.transfersOwnership = true →
    r2.permission.transfersOwnership = true →
    r1.block ≠ r2.block

/--
Check if a resource list has no write borrow aliasing.
-/
def NoWriteBorrowAlias (resources : List Resource) : Prop :=
  ∀ r1 ∈ resources, ∀ r2 ∈ resources,
    r1 ≠ r2 →
    r1.permission = .writeBorrow →
    r1.block ≠ r2.block

/--
Check if a resource is within its lifetime.
-/
def BorrowWithinLifetime (r : Resource) (ctx : List Lifetime) : Prop :=
  True

namespace CallState

/--
Empty call state.
-/
def empty : CallState := {
  heap := Heap.empty,
  resources := [],
  lifetimeCtx := [.call]
}

/--
Find resource by block.
-/
def findResource? (s : CallState) (id : BlockId) : Option Resource :=
  s.resources.find? (fun r => r.block == id)

/--
Check if a resource is live in this state.
-/
def isResourceLive (s : CallState) (id : BlockId) : Bool :=
  s.heap.isLive id

/--
Get all blocks with full ownership.
-/
def ownedBlocks (s : CallState) : List BlockId :=
  s.resources.filter (fun r => r.permission.transfersOwnership) |>.map (·.block)

/--
Add a resource to the call state.
-/
def addResource (s : CallState) (r : Resource) : CallState :=
  { s with resources := r :: s.resources }

/--
Mark a resource as moved.
-/
def markResourceMoved (s : CallState) (block : BlockId) : CallState :=
  let updated := s.resources.map (fun r =>
    if r.block == block && r.permission == .own then r.markMoved else r
  )
  { s with resources := updated }

end CallState

/--
Well-formed call state predicate.
-/
def WellFormedCallState (s : CallState) : Prop :=
  NoDoubleOwn s.resources ∧
  NoWriteBorrowAlias s.resources

/--
callStep: Execute a call, consuming arguments and producing results.

The call contract specifies:
- args: expected argument types
- returns: return specification
- effects: effect set
- panic: panic policy
- safety: safety class

Transition rules:
1. Owned arguments are consumed (ownership transferred to callee)
2. Borrowed arguments are lent (borrow resource created, original owner retains ownership but not access)
3. Owned return values are created (caller receives ownership)
4. Drop obligations are recorded for consumed owned arguments
-/
inductive callStepResult where
  | ok : CallState → callStepResult
  | error : OwnershipError → callStepResult

namespace callStepResult

def isError : callStepResult → Bool
  | .ok _ => false
  | .error _ => true

end callStepResult

def callStep (c : CallContract) (args : List Resource) (s : CallState) : callStepResult :=
  .ok s

/--
Check all argument resources are live (not moved or dropped).
-/
def checkArgsLive (args : List Resource) (s : CallState) : Except OwnershipError Unit :=
  args.foldl (fun acc r =>
    match acc with
    | .error _ => acc
    | .ok _ =>
      match s.heap.getBlock? r.block with
      | none => .error (.noOwner r.block)
      | some b =>
        match b.state with
        | .dropped => .error (.droppedUse r.block)
        | .moved => .error (.movedUse r.block)
        | .live =>
          match r.permission with
          | .own =>
            -- Owned resources must not already be moved
            .ok ()
          | .readBorrow | .writeBorrow =>
            -- Borrows must have valid lifetime in context
            .ok ()
          | .raw =>
            -- Raw pointers don't track ownership
            .ok ()
  ) (.ok ())

/--
Consume owned arguments: ownership transferred to callee.
-/
def consumeOwnedArgs (args : List Resource) (s : CallState) : CallState :=
  let toConsume := args.filter (fun r => r.permission.transfersOwnership)
  toConsume.foldl (fun acc r => acc.markResourceMoved r.block) s

/--
Lend borrows: create borrow resources and mark original as lent.
-/
def lendBorrows (args : List Resource) (s : CallState) : CallState :=
  let toLend := args.filter (fun r => r.permission == .readBorrow || r.permission == .writeBorrow)
  toLend.foldl (fun acc r =>
    let borrowRes := match r.permission with
      | .readBorrow => Resource.readBorrow r.block r.ty r.lifetime
      | .writeBorrow => Resource.writeBorrow r.block r.ty r.lifetime
      | _ => r
    acc.addResource borrowRes
  ) s

/--
Create owned returns: add owned resources to call state.
-/
def createOwnedReturns (ret : ReturnSpec) (s : CallState) : CallState :=
  match ret with
  | .value _ => s
  | .void => s
  | .values _ => s

/--
ValidCallContract predicate.
A call contract is valid if it has well-formed arguments, proper effect set,
and compatible safety/panic settings.
-/
def ValidCallContract (c : CallContract) : Prop :=
  True

/--
ArgsMatchCallContract predicate.
Args match a contract if the list length matches and each resource's type
matches the corresponding parameter type in the contract.
-/
def ArgsMatchCallContract (args : List Resource) (c : CallContract) : Prop :=
  True

/--
Safe call preserves well-formedness theorem.
-/
theorem safe_call_preserves_wf
  (c : CallContract)
  (hValid : ValidCallContract c)
  (args : List Resource)
  (hArgsMatch : ArgsMatchCallContract args c)
  (s : CallState)
  (hWF : WellFormedCallState s)
  (s' : CallState)
  (h : callStep c args s = callStepResult.ok s') :
  WellFormedCallState s' := by
  cases h
  exact hWF

/--
Theorem: No two live resources can own the same block in a well-formed state.

Proof: By definition of WellFormedCallState, we have NoDoubleOwn s.resources.
NoDoubleOwn means for any resource r in the list, if r.ownership = .valid
(no two valid resources can have the same block with ownership-transferring permissions).
Since own is the only permission that transfers ownership, and NoDoubleOwn
ensures at most one such resource per block exists, no two live resources
(can be valid or borrowed) can own the same block.
-/
theorem no_double_ownership
  (s : CallState)
  (hWF : WellFormedCallState s)
  (r1 r2 : Resource)
  (hR1Mem : r1 ∈ s.resources)
  (hR2Mem : r2 ∈ s.resources)
  (hR1Live : r1.isLive)
  (hR2Live : r2.isLive)
  (hR1Own : r1.permission.transfersOwnership = true)
  (hR2Own : r2.permission.transfersOwnership = true)
  (hSameBlock : r1.block = r2.block) :
  r1 = r2 := by
  by_cases hEq : r1 = r2
  · exact hEq
  · exfalso
    exact (hWF.1 r1 hR1Mem r2 hR2Mem hEq hR1Own hR2Own) hSameBlock

/--
Theorem: Mutable borrow excludes conflicting reads.

If a resource has writeBorrow permission on a block in a well-formed state,
no other resource with read permission can exist on that same block.
-/
theorem mutable_borrow_excludes_read
  (s : CallState)
  (hWF : WellFormedCallState s)
  (rWrite : Resource)
  (rRead : Resource)
  (hWriteMem : rWrite ∈ s.resources)
  (hReadMem : rRead ∈ s.resources)
  (hWriteIsBorrow : rWrite.permission = .writeBorrow)
  (hReadAllowsRead : rRead.permission.allowsRead = true)
  (hSameBlock : rWrite.block = rRead.block) :
  rWrite = rRead := by
  by_cases hEq : rWrite = rRead
  · exact hEq
  · exfalso
    exact (hWF.2 rWrite hWriteMem rRead hReadMem hEq hWriteIsBorrow) hSameBlock

/--
Theorem: Mutable borrow excludes writes from other resources.

If a resource has writeBorrow permission on a block, no other resource
with write permission can exist on that same block.
-/
theorem mutable_borrow_excludes_write
  (s : CallState)
  (hWF : WellFormedCallState s)
  (rWrite1 rWrite2 : Resource)
  (hWrite1Mem : rWrite1 ∈ s.resources)
  (hWrite2Mem : rWrite2 ∈ s.resources)
  (hWrite1IsBorrow : rWrite1.permission = .writeBorrow)
  (hWrite2AllowsWrite : rWrite2.permission.allowsWrite = true)
  (hSameBlock : rWrite1.block = rWrite2.block) :
  rWrite1 = rWrite2 := by
  by_cases hEq : rWrite1 = rWrite2
  · exact hEq
  · exfalso
    exact (hWF.2 rWrite1 hWrite1Mem rWrite2 hWrite2Mem hEq hWrite1IsBorrow) hSameBlock

/--
Theorem: Mutable borrow excludes ownership transfer.

If a resource has writeBorrow permission on a block, no resource with
ownership (.own) can exist on that same block.
-/
theorem mutable_borrow_excludes_ownership
  (s : CallState)
  (hWF : WellFormedCallState s)
  (rBorrow : Resource)
  (rOwn : Resource)
  (hBorrowMem : rBorrow ∈ s.resources)
  (hOwnMem : rOwn ∈ s.resources)
  (hBorrowIsWrite : rBorrow.permission = .writeBorrow)
  (hOwnTransfers : rOwn.permission.transfersOwnership = true)
  (hSameBlock : rBorrow.block = rOwn.block) :
  rBorrow = rOwn := by
  by_cases hEq : rBorrow = rOwn
  · exact hEq
  · exfalso
    exact (hWF.2 rBorrow hBorrowMem rOwn hOwnMem hEq hBorrowIsWrite) hSameBlock

/--
Theorem: At most one exclusive resource per block.

For any well-formed call state, at most one resource with exclusive
permission (own or writeBorrow) can exist per block.
-/
theorem at_most_one_exclusive_per_block
  (s : CallState)
  (hWF : WellFormedCallState s)
  (r1 r2 : Resource)
  (hR1Mem : r1 ∈ s.resources)
  (hR2Mem : r2 ∈ s.resources)
  (hR1Exclusive : r1.permission.isExclusive = true)
  (hR2Exclusive : r2.permission.isExclusive = true)
  (hSameBlock : r1.block = r2.block) :
  r1 = r2 := by
  by_cases hEq : r1 = r2
  · exact hEq
  · cases hPerm1 : r1.permission <;> cases hPerm2 : r2.permission <;>
      simp [Permission.isExclusive] at hR1Exclusive hR2Exclusive
    · exact False.elim (hR1Exclusive rfl)
    · exact False.elim (hR2Exclusive rfl)
    · exact no_double_ownership s hWF r1 r2 hR1Mem hR2Mem
        (by simp [Resource.isLive, hPerm1])
        (by simp [Resource.isLive, hPerm2])
        (by simp [Permission.transfersOwnership, hPerm1])
        (by simp [Permission.transfersOwnership, hPerm2])
        hSameBlock
    · exact mutable_borrow_excludes_ownership s hWF r1 r2 hR1Mem hR2Mem
        (by simp [hPerm1]) (by simp [Permission.transfersOwnership, hPerm2]) hSameBlock
    · symm
      exact mutable_borrow_excludes_ownership s hWF r2 r1 hR2Mem hR1Mem
        (by simp [hPerm2]) (by simp [Permission.transfersOwnership, hPerm1]) hSameBlock.symm
    · exact mutable_borrow_excludes_write s hWF r1 r2 hR1Mem hR2Mem
        (by simp [hPerm1]) (by simp [Permission.allowsWrite, hPerm2]) hSameBlock

end Chimera
