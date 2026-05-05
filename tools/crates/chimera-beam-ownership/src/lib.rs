//! BEAM ownership semantics for ChimeraIR.
//!
//! Maps BEAM process/heap values to ownership categories
//! for the Rust ownership system integration.

pub mod categories;
pub mod tracking;
pub mod validation;

pub use categories::{HeapOwnership, OwnershipCategory, ProcessOwnership};
pub use tracking::{OwnershipRef, OwnershipTracker};
pub use validation::{OwnershipValidator, ValidationResult};

/// Maximum ownership references per process.
pub const MAX_OWNERSHIP_REFS: usize = 65536;

/// Default ownership category for terms.
pub const DEFAULT_OWNERSHIP: OwnershipCategory = OwnershipCategory::Owned;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_ownership_refs() {
        assert!(MAX_OWNERSHIP_REFS > 0);
    }

    #[test]
    fn test_default_ownership() {
        assert_eq!(DEFAULT_OWNERSHIP, OwnershipCategory::Owned);
    }
}
