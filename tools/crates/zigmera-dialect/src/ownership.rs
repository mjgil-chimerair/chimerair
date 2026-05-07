//! Lowering of ownership and allocator semantics.
//!
//! Tracks allocator parameters, returned owned memory, drops, defer/errdefer
//! cleanup, noalias, and borrowed vs owned values.
//!
//! Task 92: Lower ownership and allocator semantics

use super::operations::{ZigInstruction, ZigOp};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Ownership classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ownership {
    /// Owned value (caller owns, callee must free)
    Owned,
    /// Borrowed value (caller retains ownership)
    Borrowed,
    /// Mutable borrow (exclusive access)
    Mutable,
    /// Shared borrow (read-only access)
    Shared,
    /// Moved value (ownership transferred)
    Moved,
}

/// Memory lifetime classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lifetime {
    /// Value lives for entire program
    Static,
    /// Value lives for function duration
    Function,
    /// Value lives for block duration
    Block,
    /// Value lives for expression evaluation
    Expression,
}

/// Allocator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatorInfo {
    /// Allocator type ID
    pub type_id: u64,
    /// Is this an allocator parameter?
    pub is_param: bool,
    /// Allocator name (for diagnostics)
    pub name: String,
}

/// Ownership context for a function
#[derive(Debug, Clone)]
pub struct OwnershipContext {
    /// Allocator parameters
    allocators: HashMap<u64, AllocatorInfo>,
    /// Owned values (value_id -> ownership)
    owned_values: HashMap<u64, Ownership>,
    /// Borrowed values
    borrowed_values: HashMap<u64, Lifetime>,
    /// Moved values
    moved_values: HashSet<u64>,
    /// Deferred cleanup operations
    deferred_ops: Vec<DeferredCleanup>,
}

/// A deferred cleanup operation
#[derive(Debug, Clone)]
pub struct DeferredCleanup {
    /// Operation type
    pub op_type: CleanupOpType,
    /// Target value ID
    pub target: u64,
    /// Block where cleanup should occur
    pub block_id: u64,
}

/// Type of cleanup operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CleanupOpType {
    /// Drop (deallocate memory)
    Drop,
    /// Defer (run on scope exit)
    Defer,
    /// Errdefer (run on error exit)
    Errdefer,
    /// Free (deallocate and nullify)
    Free,
}

impl OwnershipContext {
    /// Create a new ownership context
    pub fn new() -> Self {
        Self {
            allocators: HashMap::new(),
            owned_values: HashMap::new(),
            borrowed_values: HashMap::new(),
            moved_values: HashSet::new(),
            deferred_ops: Vec::new(),
        }
    }

    /// Register an allocator parameter
    pub fn register_allocator(&mut self, id: u64, name: &str) {
        self.allocators.insert(
            id,
            AllocatorInfo {
                type_id: id,
                is_param: true,
                name: name.to_string(),
            },
        );
    }

    /// Mark a value as owned
    pub fn mark_owned(&mut self, value_id: u64) {
        self.owned_values.insert(value_id, Ownership::Owned);
    }

    /// Mark a value as borrowed
    pub fn mark_borrowed(&mut self, value_id: u64, lifetime: Lifetime) {
        self.borrowed_values.insert(value_id, lifetime);
    }

    /// Mark a value as moved
    pub fn mark_moved(&mut self, value_id: u64) {
        self.moved_values.insert(value_id);
        self.owned_values.remove(&value_id);
        self.borrowed_values.remove(&value_id);
    }

    /// Mark a value as mutable borrowed
    pub fn mark_mutable(&mut self, value_id: u64) {
        self.owned_values.insert(value_id, Ownership::Mutable);
    }

    /// Add a deferred cleanup operation
    pub fn add_deferred_cleanup(&mut self, op_type: CleanupOpType, target: u64, block_id: u64) {
        self.deferred_ops.push(DeferredCleanup {
            op_type,
            target,
            block_id,
        });
    }

    /// Check if a value is owned
    pub fn is_owned(&self, value_id: u64) -> bool {
        self.owned_values.get(&value_id) == Some(&Ownership::Owned)
    }

    /// Check if a value is borrowed
    pub fn is_borrowed(&self, value_id: u64) -> bool {
        self.borrowed_values.contains_key(&value_id)
    }

    /// Check if a value is moved
    pub fn is_moved(&self, value_id: u64) -> bool {
        self.moved_values.contains(&value_id)
    }

    /// Check if a value is mutable
    pub fn is_mutable(&self, value_id: u64) -> bool {
        self.owned_values.get(&value_id) == Some(&Ownership::Mutable)
    }

    /// Get the lifetime of a borrowed value
    pub fn get_lifetime(&self, value_id: u64) -> Option<Lifetime> {
        self.borrowed_values.get(&value_id).cloned()
    }

    /// Check if a value needs cleanup
    pub fn needs_cleanup(&self, value_id: u64) -> bool {
        // Check if this value is in any deferred cleanup
        self.deferred_ops.iter().any(|op| op.target == value_id)
    }
}

impl Default for OwnershipContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Ownership lowering helper
#[derive(Debug, Clone)]
pub struct OwnershipLowering {
    /// Default lifetime for borrowed values
    default_lifetime: Lifetime,
    /// Track allocator usage
    track_allocators: bool,
}

impl OwnershipLowering {
    /// Create a new ownership lowering helper
    pub fn new(default_lifetime: Lifetime, track_allocators: bool) -> Self {
        Self {
            default_lifetime,
            track_allocators,
        }
    }

    /// Lower a load instruction (creates a borrow)
    pub fn lower_load(&self, inst: &ZigInstruction) -> Ownership {
        // Load creates a borrowed reference to existing memory
        Ownership::Borrowed
    }

    /// Lower a store instruction (consumes ownership or creates borrow)
    pub fn lower_store(&self, _inst: &ZigInstruction) -> Ownership {
        // Store doesn't consume the stored value, just writes to memory
        Ownership::Borrowed
    }

    /// Lower a call instruction
    pub fn lower_call(&self, result_type: Option<u64>) -> Ownership {
        // Function call returning owned memory -> caller owns result
        if result_type.is_some() {
            Ownership::Owned
        } else {
            Ownership::Borrowed
        }
    }

    /// Lower an alloca instruction (creates owned memory)
    pub fn lower_alloca(&self) -> Ownership {
        Ownership::Owned
    }

    /// Check if an operation has side effects on ownership
    pub fn has_ownership_effect(&self, op: &ZigOp) -> bool {
        matches!(
            op,
            ZigOp::Alloca
                | ZigOp::AllocGlobal
                | ZigOp::Store
                | ZigOp::AtomicStore
                | ZigOp::Call
                | ZigOp::CallIndirect
                | ZigOp::Invoke
        )
    }

    /// Emit MLIR annotation for ownership
    pub fn emit_ownership_attr(&self, ownership: &Ownership) -> String {
        match ownership {
            Ownership::Owned => "!chir.ownership(owned)".to_string(),
            Ownership::Borrowed => "!chir.ownership(borrowed)".to_string(),
            Ownership::Mutable => "!chir.ownership(mutable)".to_string(),
            Ownership::Shared => "!chir.ownership(shared)".to_string(),
            Ownership::Moved => "!chir.ownership(moved)".to_string(),
        }
    }

    /// Emit MLIR annotation for lifetime
    pub fn emit_lifetime_attr(&self, lifetime: &Lifetime) -> String {
        match lifetime {
            Lifetime::Static => "!chir.lifetime(static)".to_string(),
            Lifetime::Function => "!chir.lifetime(function)".to_string(),
            Lifetime::Block => "!chir.lifetime(block)".to_string(),
            Lifetime::Expression => "!chir.lifetime(expression)".to_string(),
        }
    }
}

/// Check if a type is owned by default (non-pointer scalar types)
pub fn is_owned_type(type_id: u64) -> bool {
    matches!(type_id, 1 | 2 | 3 | 4 | 5) // i8, i16, i32, i64, i128
}

/// Check if a type is borrowed by default (pointer types)
pub fn is_borrowed_type(type_id: u64) -> bool {
    type_id > 100 // Simplified: types with ID > 100 are pointers/references
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_context_creation() {
        let ctx = OwnershipContext::new();
        assert!(ctx.owned_values.is_empty());
        assert!(ctx.borrowed_values.is_empty());
    }

    #[test]
    fn test_register_allocator() {
        let mut ctx = OwnershipContext::new();
        ctx.register_allocator(1, "allocator");
        assert!(ctx.allocators.contains_key(&1));
    }

    #[test]
    fn test_mark_owned() {
        let mut ctx = OwnershipContext::new();
        ctx.mark_owned(100);
        assert!(ctx.is_owned(100));
    }

    #[test]
    fn test_mark_borrowed() {
        let mut ctx = OwnershipContext::new();
        ctx.mark_borrowed(200, Lifetime::Block);
        assert!(ctx.is_borrowed(200));
        assert_eq!(ctx.get_lifetime(200), Some(Lifetime::Block));
    }

    #[test]
    fn test_mark_moved() {
        let mut ctx = OwnershipContext::new();
        ctx.mark_owned(100);
        ctx.mark_moved(100);
        assert!(ctx.is_moved(100));
        assert!(!ctx.is_owned(100));
    }

    #[test]
    fn test_deferred_cleanup() {
        let mut ctx = OwnershipContext::new();
        ctx.add_deferred_cleanup(CleanupOpType::Drop, 100, 1);
        ctx.add_deferred_cleanup(CleanupOpType::Errdefer, 200, 1);
        assert!(ctx.needs_cleanup(100));
        assert!(ctx.needs_cleanup(200));
    }

    #[test]
    fn test_ownership_lowering_alloca() {
        let lowering = OwnershipLowering::new(Lifetime::Block, true);
        let ownership = lowering.lower_alloca();
        assert_eq!(ownership, Ownership::Owned);
    }

    #[test]
    fn test_ownership_lowering_load() {
        let lowering = OwnershipLowering::new(Lifetime::Block, true);
        let inst = ZigInstruction::new(1, ZigOp::Load);
        let ownership = lowering.lower_load(&inst);
        assert_eq!(ownership, Ownership::Borrowed);
    }

    #[test]
    fn test_emit_ownership_attr() {
        let lowering = OwnershipLowering::new(Lifetime::Block, true);
        assert!(lowering
            .emit_ownership_attr(&Ownership::Owned)
            .contains("owned"));
        assert!(lowering
            .emit_ownership_attr(&Ownership::Borrowed)
            .contains("borrowed"));
    }

    #[test]
    fn test_emit_lifetime_attr() {
        let lowering = OwnershipLowering::new(Lifetime::Block, true);
        assert!(lowering
            .emit_lifetime_attr(&Lifetime::Function)
            .contains("function"));
        assert!(lowering
            .emit_lifetime_attr(&Lifetime::Block)
            .contains("block"));
    }

    #[test]
    fn test_is_owned_type() {
        assert!(is_owned_type(1));
        assert!(is_owned_type(3));
        assert!(is_owned_type(5));
        assert!(!is_owned_type(100));
    }

    #[test]
    fn test_is_borrowed_type() {
        assert!(is_borrowed_type(101));
        assert!(is_borrowed_type(200));
        assert!(!is_borrowed_type(1));
    }
}
