//! Ownership categories for BEAM values.
//!
//! Maps BEAM constructs to ownership semantics.

use chimera_beam_process::BeamPid;
use serde::{Deserialize, Serialize};

/// Ownership category for a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnershipCategory {
    /// Owned by current context (exclusive).
    Owned,
    /// Borrowed (shared, read-only).
    Borrowed,
    /// Mutable borrowed (exclusive access).
    Exclusive,
    /// Garbage collected (managed by BEAM runtime).
    GcRoot,
    /// Static (global, never freed).
    Static,
    /// Weak reference (doesn't prevent collection).
    Weak,
    /// Unrooted (no ownership tracking).
    Unrooted,
}

impl Default for OwnershipCategory {
    fn default() -> Self {
        OwnershipCategory::Owned
    }
}

impl OwnershipCategory {
    /// Get category name.
    pub fn as_str(&self) -> &'static str {
        match self {
            OwnershipCategory::Owned => "owned",
            OwnershipCategory::Borrowed => "borrowed",
            OwnershipCategory::Exclusive => "exclusive",
            OwnershipCategory::GcRoot => "gc_root",
            OwnershipCategory::Static => "static",
            OwnershipCategory::Weak => "weak",
            OwnershipCategory::Unrooted => "unrooted",
        }
    }

    /// Check if this category can be safely shared.
    pub fn is_shareable(&self) -> bool {
        matches!(
            self,
            OwnershipCategory::Static | OwnershipCategory::Borrowed
        )
    }

    /// Check if this category requires cleanup.
    pub fn requires_cleanup(&self) -> bool {
        matches!(
            self,
            OwnershipCategory::Owned | OwnershipCategory::Exclusive
        )
    }
}

/// Process ownership information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessOwnership {
    /// Process PID.
    pub pid: BeamPid,
    /// Process's heap ownership category.
    pub heap_category: OwnershipCategory,
    /// Stack ownership category.
    pub stack_category: OwnershipCategory,
    /// Registered names (static).
    pub registered_static: bool,
    /// Links (weak references).
    pub links_weak: bool,
    /// Monitors (unrooted).
    pub monitors_unrooted: bool,
}

impl ProcessOwnership {
    /// Create ownership for a new process.
    pub fn new(pid: BeamPid) -> Self {
        ProcessOwnership {
            pid,
            heap_category: OwnershipCategory::Owned,
            stack_category: OwnershipCategory::Owned,
            registered_static: false,
            links_weak: false,
            monitors_unrooted: true,
        }
    }

    /// Set heap category.
    pub fn with_heap_category(mut self, category: OwnershipCategory) -> Self {
        self.heap_category = category;
        self
    }

    /// Mark as having static registered names.
    pub fn with_static_registration(mut self) -> Self {
        self.registered_static = true;
        self
    }

    /// Mark as having weak links.
    pub fn with_weak_links(mut self) -> Self {
        self.links_weak = true;
        self
    }
}

/// Heap value ownership information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapOwnership {
    /// Value address (approximate).
    pub address: u64,
    /// Size in bytes.
    pub size: usize,
    /// Ownership category.
    pub category: OwnershipCategory,
    /// Whether value is movable.
    pub movable: bool,
    /// Whether value is on process heap.
    pub on_process_heap: bool,
}

impl HeapOwnership {
    /// Create ownership for a heap value.
    pub fn new(address: u64, size: usize) -> Self {
        HeapOwnership {
            address,
            size,
            category: OwnershipCategory::Owned,
            movable: true,
            on_process_heap: true,
        }
    }

    /// Create for a static value.
    pub fn static_value(address: u64, size: usize) -> Self {
        HeapOwnership {
            address,
            size,
            category: OwnershipCategory::Static,
            movable: false,
            on_process_heap: false,
        }
    }

    /// Create for a gc-managed value.
    pub fn gc_root(address: u64, size: usize) -> Self {
        HeapOwnership {
            address,
            size,
            category: OwnershipCategory::GcRoot,
            movable: true,
            on_process_heap: true,
        }
    }

    /// Check if value can be shared.
    pub fn can_share(&self) -> bool {
        self.category.is_shareable()
    }
}

/// Map BEAM term to ownership category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermOwnership {
    /// Atom (static).
    Atom,
    /// PID (owned).
    Pid,
    /// Reference (unrooted).
    Reference,
    /// Tuple (owned, movable).
    Tuple,
    /// List (owned, movable).
    List,
    /// Binary (owned or static).
    Binary,
    /// Integer (owned/immediate).
    Integer,
    /// Float (owned).
    Float,
}

impl TermOwnership {
    /// Get ownership category for this term type.
    pub fn to_category(self) -> OwnershipCategory {
        match self {
            TermOwnership::Atom => OwnershipCategory::Static,
            TermOwnership::Pid => OwnershipCategory::Owned,
            TermOwnership::Reference => OwnershipCategory::Unrooted,
            TermOwnership::Tuple => OwnershipCategory::Owned,
            TermOwnership::List => OwnershipCategory::Owned,
            TermOwnership::Binary => OwnershipCategory::Owned,
            TermOwnership::Integer => OwnershipCategory::Owned,
            TermOwnership::Float => OwnershipCategory::Owned,
        }
    }

    /// Check if this term type is movable.
    pub fn is_movable(self) -> bool {
        matches!(
            self,
            TermOwnership::Tuple
                | TermOwnership::List
                | TermOwnership::Binary
                | TermOwnership::Integer
                | TermOwnership::Float
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_category_as_str() {
        assert_eq!(OwnershipCategory::Owned.as_str(), "owned");
        assert_eq!(OwnershipCategory::Borrowed.as_str(), "borrowed");
        assert_eq!(OwnershipCategory::Static.as_str(), "static");
    }

    #[test]
    fn test_ownership_category_shareable() {
        assert!(OwnershipCategory::Static.is_shareable());
        assert!(OwnershipCategory::Borrowed.is_shareable());
        assert!(!OwnershipCategory::Owned.is_shareable());
    }

    #[test]
    fn test_process_ownership_new() {
        let pid = BeamPid::new(1, 1, 0);
        let ownership = ProcessOwnership::new(pid);
        assert_eq!(ownership.pid, pid);
        assert_eq!(ownership.heap_category, OwnershipCategory::Owned);
    }

    #[test]
    fn test_process_ownership_builder() {
        let pid = BeamPid::new(1, 1, 0);
        let ownership = ProcessOwnership::new(pid)
            .with_heap_category(OwnershipCategory::GcRoot)
            .with_static_registration()
            .with_weak_links();

        assert_eq!(ownership.heap_category, OwnershipCategory::GcRoot);
        assert!(ownership.registered_static);
        assert!(ownership.links_weak);
    }

    #[test]
    fn test_heap_ownership_new() {
        let ownership = HeapOwnership::new(0x1000, 32);
        assert_eq!(ownership.address, 0x1000);
        assert_eq!(ownership.size, 32);
        assert_eq!(ownership.category, OwnershipCategory::Owned);
        assert!(ownership.movable);
    }

    #[test]
    fn test_heap_ownership_static() {
        let ownership = HeapOwnership::static_value(0x2000, 64);
        assert_eq!(ownership.category, OwnershipCategory::Static);
        assert!(!ownership.movable);
    }

    #[test]
    fn test_term_ownership_to_category() {
        assert_eq!(TermOwnership::Atom.to_category(), OwnershipCategory::Static);
        assert_eq!(TermOwnership::Pid.to_category(), OwnershipCategory::Owned);
        assert_eq!(
            TermOwnership::Reference.to_category(),
            OwnershipCategory::Unrooted
        );
    }

    #[test]
    fn test_term_ownership_movable() {
        assert!(TermOwnership::Tuple.is_movable());
        assert!(TermOwnership::List.is_movable());
        // Atoms are interned and not movable in the traditional sense
        assert!(!TermOwnership::Atom.is_movable());
        assert!(!TermOwnership::Reference.is_movable());
    }
}
