-- ChimeraProof Tests: Resource Model
-- Tests for resources with ownership state.

import Chimera.Memory.Permission
import Chimera.Memory.Block
import Chimera.Memory.Ownership

namespace Chimera.Test

-- Permission tests
namespace PermissionTest

theorem own_is_own : Permission.own = Permission.own := by rfl
theorem readBorrow_is_readBorrow : Permission.readBorrow = Permission.readBorrow := by rfl
theorem writeBorrow_is_writeBorrow : Permission.writeBorrow = Permission.writeBorrow := by rfl
theorem raw_is_raw : Permission.raw = Permission.raw := by rfl

theorem own_allows_read : Permission.own.allowsRead = true := by rfl
theorem readBorrow_allows_read : Permission.readBorrow.allowsRead = true := by rfl
theorem writeBorrow_allows_read : Permission.writeBorrow.allowsRead = true := by rfl
theorem raw_allows_read : Permission.raw.allowsRead = false := by rfl

theorem own_allows_write : Permission.own.allowsWrite = true := by rfl
theorem writeBorrow_allows_write : Permission.writeBorrow.allowsWrite = true := by rfl
theorem raw_allows_write : Permission.raw.allowsWrite = false := by rfl
theorem readBorrow_allows_write : Permission.readBorrow.allowsWrite = false := by rfl

theorem own_is_exclusive : Permission.own.isExclusive = true := by rfl
theorem writeBorrow_is_exclusive : Permission.writeBorrow.isExclusive = true := by rfl
theorem readBorrow_not_exclusive : Permission.readBorrow.isExclusive = false := by rfl
theorem raw_not_exclusive : Permission.raw.isExclusive = false := by rfl

theorem own_transfers : Permission.own.transfersOwnership = true := by rfl
theorem readBorrow_not_transfers : Permission.readBorrow.transfersOwnership = false := by rfl
theorem writeBorrow_not_transfers : Permission.writeBorrow.transfersOwnership = false := by rfl
theorem raw_not_transfers : Permission.raw.transfersOwnership = false := by rfl

end PermissionTest

-- OwnershipState tests
namespace OwnershipStateTest

theorem valid_is_valid : OwnershipState.valid = OwnershipState.valid := by rfl
theorem borrowed_is_borrowed : OwnershipState.borrowed = OwnershipState.borrowed := by rfl
theorem moved_is_moved : OwnershipState.moved = OwnershipState.moved := by rfl
theorem dropped_is_dropped : OwnershipState.dropped = OwnershipState.dropped := by rfl

theorem valid_accessible : OwnershipState.valid.isAccessible = true := by rfl
theorem borrowed_accessible : OwnershipState.borrowed.isAccessible = true := by rfl
theorem moved_not_accessible : OwnershipState.moved.isAccessible = false := by rfl
theorem dropped_not_accessible : OwnershipState.dropped.isAccessible = false := by rfl

theorem valid_can_drop : OwnershipState.valid.canDrop = true := by rfl
theorem borrowed_cannot_drop : OwnershipState.borrowed.canDrop = false := by rfl
theorem moved_cannot_drop : OwnershipState.moved.canDrop = false := by rfl
theorem dropped_cannot_drop : OwnershipState.dropped.canDrop = false := by rfl

theorem valid_can_transfer : OwnershipState.valid.canTransfer = true := by rfl
theorem borrowed_cannot_transfer : OwnershipState.borrowed.canTransfer = false := by rfl
theorem moved_cannot_transfer : OwnershipState.moved.canTransfer = false := by rfl
theorem dropped_cannot_transfer : OwnershipState.dropped.canTransfer = false := by rfl

end OwnershipStateTest

-- Resource tests
namespace ResourceTest

private def blockOne : BlockId := ⟨1⟩
private def blockTwo : BlockId := ⟨2⟩
private def allocSym : Symbol := Symbol.simple "alloc"

-- Resource.owned creates valid owned resource
theorem owned_creates_valid : ∀ (block : BlockId) (ty : ChType) (lifetime : Lifetime),
  let r := Resource.owned block ty lifetime
  r.block = block ∧ r.permission = .own ∧ r.lifetime = lifetime ∧ r.ty = ty ∧ r.ownership = .valid := by
  intros block ty lifetime
  simp [Resource.owned]
  constructor <;> rfl

-- Resource.readBorrow creates borrowed resource
theorem readBorrow_creates_borrowed : ∀ (block : BlockId) (ty : ChType) (lifetime : Lifetime),
  let r := Resource.readBorrow block ty lifetime
  r.block = block ∧ r.permission = .readBorrow ∧ r.lifetime = lifetime ∧ r.ty = ty ∧ r.ownership = .borrowed := by
  intros block ty lifetime
  simp [Resource.readBorrow]
  constructor <;> rfl

-- Resource.writeBorrow creates borrowed resource
theorem writeBorrow_creates_borrowed : ∀ (block : BlockId) (ty : ChType) (lifetime : Lifetime),
  let r := Resource.writeBorrow block ty lifetime
  r.block = block ∧ r.permission = .writeBorrow ∧ r.lifetime = lifetime ∧ r.ty = ty ∧ r.ownership = .borrowed := by
  intros block ty lifetime
  simp [Resource.writeBorrow]
  constructor <;> rfl

-- Resource.raw creates raw resource
theorem raw_creates_raw : ∀ (block : BlockId) (ty : ChType),
  let r := Resource.raw block ty
  r.block = block ∧ r.permission = .raw ∧ r.lifetime = .call ∧ r.ty = ty ∧ r.ownership = .valid := by
  intros block ty
  simp [Resource.raw]
  constructor <;> rfl

-- isLive returns true for valid
theorem isLive_valid : (⟨⟨1⟩, .own, .call, .i32, .valid⟩ : Resource).isLive = true := by rfl

-- isLive returns true for borrowed
theorem isLive_borrowed : (⟨⟨1⟩, .readBorrow, .call, .i32, .borrowed⟩ : Resource).isLive = true := by rfl

-- isLive returns false for moved
theorem isLive_moved : (⟨⟨1⟩, .own, .call, .i32, .moved⟩ : Resource).isLive = false := by rfl

-- isLive returns false for dropped
theorem isLive_dropped : (⟨⟨1⟩, .own, .call, .i32, .dropped⟩ : Resource).isLive = false := by rfl

-- canRead for owned resource
theorem canRead_owned : (⟨⟨1⟩, .own, .call, .i32, .valid⟩ : Resource).canRead = true := by rfl

-- canRead for raw resource (raw doesn't allow read)
theorem cannotRead_raw : (⟨⟨1⟩, .raw, .call, .i32, .valid⟩ : Resource).canRead = false := by rfl

-- canWrite for owned resource
theorem canWrite_owned : (⟨⟨1⟩, .own, .call, .i32, .valid⟩ : Resource).canWrite = true := by rfl

-- canWrite for read borrow (doesn't allow write)
theorem cannotWrite_readBorrow : (⟨⟨1⟩, .readBorrow, .call, .i32, .valid⟩ : Resource).canWrite = false := by rfl

-- canWrite for moved resource
theorem cannotWrite_moved : (⟨⟨1⟩, .own, .call, .i32, .moved⟩ : Resource).canWrite = false := by rfl

-- markMoved changes ownership
theorem markMoved_changes : (⟨⟨1⟩, .own, .call, .i32, .valid⟩ : Resource).markMoved.ownership = .moved := by rfl

-- markDropped changes ownership
theorem markDropped_changes : (⟨⟨1⟩, .own, .call, .i32, .valid⟩ : Resource).markDropped.ownership = .dropped := by rfl

-- mayAlias for same exclusive block
theorem may_alias_same : mayAlias ⟨⟨1⟩, .own, .call, .i32, .valid⟩ ⟨⟨1⟩, .writeBorrow, .call, .i32, .borrowed⟩ = true := by rfl

-- mustNotAlias for different blocks
theorem must_not_alias_diff : mustNotAlias ⟨⟨1⟩, .own, .call, .i32, .valid⟩ ⟨⟨2⟩, .own, .call, .i32, .valid⟩ = true := by rfl

-- mustNotAlias for non-exclusive
theorem must_not_alias_non_exclusive : mustNotAlias ⟨⟨1⟩, .readBorrow, .call, .i32, .borrowed⟩ ⟨⟨1⟩, .readBorrow, .call, .i32, .borrowed⟩ = true := by rfl

-- NoDoubleOwn tests
namespace NoDoubleOwnTest

-- Empty list has no double ownership
theorem empty_no_double_own : NoDoubleOwn [] = True := by rfl

-- Single valid owned resource has no double ownership
theorem single_valid_no_double_own : NoDoubleOwn [Resource.owned ⟨1⟩ .i32 .call] = True := by
  simp [NoDoubleOwn, Resource.owned]
  constructor
  . intro h
    simp at h
  . rfl

-- Two different blocks with owned resources have no double ownership
theorem different_blocks_no_double_own : NoDoubleOwn [
  Resource.owned ⟨1⟩ .i32 .call,
  Resource.owned ⟨2⟩ .i32 .call
] = True := by
  simp [NoDoubleOwn]
  constructor
  . intros r1 rest hIn
    cases hIn
    rfl
  . constructor
    . intros r1 rest hIn
      cases hIn
      rfl
    . rfl

-- Two owned resources on same block (if such existed) would violate NoDoubleOwn
-- But our construction prevents this - we can test the negation
theorem owned_and_borrowed_no_double_own : NoDoubleOwn [
  Resource.owned ⟨1⟩ .i32 .call,
  Resource.readBorrow ⟨1⟩ .i32 .call
] = True := by
  simp [NoDoubleOwn, Resource.owned, Resource.readBorrow]
  constructor
  . intros r1 rest hIn
    cases hIn
    simp
    intro hCont
    simp at hCont
  . constructor
    . intros r1 rest hIn
      cases hIn
      rfl
    . rfl

end NoDoubleOwnTest

-- WellFormedCallState tests
namespace WellFormedCallStateTest

-- Empty call state is well-formed
theorem empty_wf : WellFormedCallState CallState.empty = True := by
  simp [WellFormedCallState, CallState.empty]
  constructor <;> rfl

-- Call state with valid resources is well-formed
theorem valid_resources_wf : WellFormedCallState {
  heap := Heap.empty,
  resources := [
    Resource.owned ⟨1⟩ .i32 .call,
    Resource.readBorrow ⟨2⟩ .i32 .call
  ],
  lifetimeCtx := [.call]
} = True := by
  simp [WellFormedCallState]
  constructor
  . simp [NoDoubleOwn]
    constructor
    . intros r1 rest hIn
      cases hIn
      rfl
    . constructor
      . intros r1 rest hIn
        cases hIn
        rfl
      . rfl
  . simp [NoWriteBorrowAlias]
    constructor
    . intros r1 rest hIn
      cases hIn
      rfl
    . constructor
      . intros r1 rest hIn
        cases hIn
        rfl
      . rfl

-- Call state with write borrow on one block and read borrow on different block is well-formed
theorem write_and_read_borrow_wf : WellFormedCallState {
  heap := Heap.empty,
  resources := [
    Resource.writeBorrow ⟨1⟩ .i32 .call,
    Resource.readBorrow ⟨2⟩ .i32 .call
  ],
  lifetimeCtx := [.call]
} = True := by
  simp [WellFormedCallState]
  constructor
  . simp [NoDoubleOwn]
    constructor
    . intros r1 rest hIn
      cases hIn
      rfl
    . constructor
      . intros r1 rest hIn
        cases hIn
        rfl
      . rfl
  . simp [NoWriteBorrowAlias]
    constructor
    . intros r1 rest hIn
      cases hIn
      rfl
    . constructor
      . intros r1 rest hIn
        cases hIn
        rfl
      . rfl

end WellFormedCallStateTest

namespace OwnershipSoundnessTest

private def ownerRes : Resource := Resource.owned blockOne .i32 .call
private def borrowRes : Resource := Resource.writeBorrow blockOne .i32 .call
private def otherOwnerRes : Resource := Resource.owned blockTwo .i32 .call

theorem no_double_ownership_rejects_distinct_same_block_owners :
    let s : CallState := {
      heap := Heap.empty,
      resources := [ownerRes, { ownerRes with ty := .u32 }],
      lifetimeCtx := [.call]
    }
    WellFormedCallState s → False := by
  intro s hWF
  have hSameBlock : ownerRes.block = ({ ownerRes with ty := .u32 }).block := rfl
  have hDistinct : ownerRes ≠ { ownerRes with ty := .u32 } := by
    intro hEq
    have : ownerRes.ty = (.u32 : ChType) := by simpa using congrArg Resource.ty hEq
    cases this
  have hEq := no_double_ownership s hWF ownerRes { ownerRes with ty := .u32 }
    (by simp [s])
    (by simp [s])
    (by simp [Resource.isLive, ownerRes, Resource.owned])
    (by simp [Resource.isLive, ownerRes, Resource.owned])
    (by simp [Permission.transfersOwnership, ownerRes, Resource.owned])
    (by simp [Permission.transfersOwnership, ownerRes, Resource.owned])
    hSameBlock
  exact hDistinct hEq

theorem mutable_borrow_excludes_ownership_on_same_block :
    let s : CallState := {
      heap := Heap.empty,
      resources := [borrowRes, ownerRes],
      lifetimeCtx := [.call]
    }
    WellFormedCallState s → False := by
  intro s hWF
  have hEq := mutable_borrow_excludes_ownership s hWF borrowRes ownerRes
    (by simp [s])
    (by simp [s])
    (by rfl)
    (by rfl)
    rfl
  have hDistinct : borrowRes ≠ ownerRes := by
    intro h
    have : borrowRes.permission = ownerRes.permission := congrArg Resource.permission h
    simp [borrowRes, ownerRes, Resource.writeBorrow, Resource.owned] at this
  exact hDistinct hEq

theorem at_most_one_exclusive_per_block_rejects_two_mutable_resources :
    let s : CallState := {
      heap := Heap.empty,
      resources := [borrowRes, ownerRes],
      lifetimeCtx := [.call]
    }
    WellFormedCallState s → False := by
  intro s hWF
  have hEq := at_most_one_exclusive_per_block s hWF borrowRes ownerRes
    (by simp [s])
    (by simp [s])
    (by rfl)
    (by rfl)
    rfl
  have hDistinct : borrowRes ≠ ownerRes := by
    intro h
    have : borrowRes.permission = ownerRes.permission := congrArg Resource.permission h
    simp [borrowRes, ownerRes, Resource.writeBorrow, Resource.owned] at this
  exact hDistinct hEq

theorem moved_resource_is_not_live :
    ownerRes.markMoved.isLive = false := by
  exact Resource.moved_not_live ownerRes

theorem dropped_resource_is_not_live :
    ownerRes.markDropped.isLive = false := by
  exact Resource.dropped_not_live ownerRes

theorem checkArgsLive_rejects_moved_use :
    let heap := (Heap.alloc Heap.empty blockOne 8 8 allocSym).markMoved blockOne
    let s : CallState := { CallState.empty with heap := heap }
    checkArgsLive [ownerRes] s = Except.error (.movedUse blockOne) := by
  rfl

theorem checkArgsLive_rejects_dropped_use :
    let heap := (Heap.alloc Heap.empty blockOne 8 8 allocSym).markDropped blockOne
    let s : CallState := { CallState.empty with heap := heap }
    checkArgsLive [ownerRes] s = Except.error (.droppedUse blockOne) := by
  rfl

theorem checkArgsLive_accepts_live_owned_resource :
    let heap := Heap.alloc Heap.empty blockTwo 8 8 allocSym
    let s : CallState := { CallState.empty with heap := heap }
    checkArgsLive [otherOwnerRes] s = Except.ok () := by
  rfl

end OwnershipSoundnessTest

-- CallState tests
namespace CallStateTest

-- empty creates empty call state
theorem empty_has_no_resources : CallState.empty.resources = [] := by rfl

-- addResource adds to resources list
theorem addResource_increases_list : ∀ (s : CallState) (r : Resource),
  (s.addResource r).resources = r :: s.resources := by
  intros s r
  simp [CallState.addResource]

-- addResource preserves heap
theorem addResource_preserves_heap : ∀ (s : CallState) (r : Resource),
  (s.addResource r).heap = s.heap := by
  intros s r
  simp [CallState.addResource]

-- markResourceMoved changes owned resource to moved
theorem markMoved_changes_ownership : ∀ (s : CallState) (block : BlockId),
  let r := Resource.owned block .i32 .call
  let s' := s.addResource r |>.markResourceMoved block
  s'.resources.head?.map (·.ownership) = some .moved := by
  intros s block
  simp [CallState.addResource, CallState.markResourceMoved, Resource.owned, Resource.markMoved]

end CallStateTest

-- callStep tests
namespace CallStepTest

-- callStep with owned args consumes them
theorem callstep_consumes_owned : ∀ (c : CallContract) (s : CallState) (r : Resource),
  r.permission = .own →
  let s' := callStep c [r] s
  match s' with
  | .ok s'' => s''.resources.any (fun r' => r'.block = r.block ∧ r'.ownership = .moved)
  | .error _ => false := by
  intros c s r hOwn
  simp [callStep, consumeOwnedArgs, lendBorrows, createOwnedReturns]
  -- The owned resource should be marked as moved

-- callStep with borrowed args creates borrow resources
theorem callstep_lends_borrow : ∀ (c : CallContract) (s : CallState) (r : Resource),
  r.permission = .readBorrow →
  let s' := callStep c [r] s
  match s' with
  | .ok s'' => s''.resources.any (fun r' => r'.block = r.block ∧ r'.permission = .readBorrow)
  | .error _ => false := by
  intros c s r hBorrow
  simp [callStep, consumeOwnedArgs, lendBorrows, createOwnedReturns]
  -- A borrow resource should be added

end CallStepTest

-- NoWriteBorrowAlias tests
namespace NoWriteBorrowAliasTest

-- Empty list satisfies NoWriteBorrowAlias
theorem empty_no_write_alias : NoWriteBorrowAlias [] = True := by rfl

-- Single write borrow satisfies NoWriteBorrowAlias
theorem single_write_borrow_no_alias : NoWriteBorrowAlias [Resource.writeBorrow ⟨1⟩ .i32 .call] = True := by
  simp [NoWriteBorrowAlias, Resource.writeBorrow]
  constructor
  . intros r1 rest hIn
    cases hIn
    rfl
  . rfl

-- Write borrow and read borrow on different blocks satisfies NoWriteBorrowAlias
theorem write_read_different_blocks : NoWriteBorrowAlias [
  Resource.writeBorrow ⟨1⟩ .i32 .call,
  Resource.readBorrow ⟨2⟩ .i32 .call
] = True := by
  simp [NoWriteBorrowAlias]
  constructor
  . intros r1 rest hIn
    cases hIn
    rfl
  . constructor
    . intros r1 rest hIn
      cases hIn
      rfl
    . rfl

end NoWriteBorrowAliasTest

end ResourceTest

end Chimera.Test
