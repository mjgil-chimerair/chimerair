//! Zig error handling modeling.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Error set model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSetModel {
    /// Error set name
    pub name: String,
    /// Error variants
    pub errors: Vec<String>,
    /// Is this a set of all errors?
    pub is_anyerror: bool,
}

impl ErrorSetModel {
    /// Create a new error set
    pub fn new(name: String) -> Self {
        Self {
            name,
            errors: Vec::new(),
            is_anyerror: false,
        }
    }

    /// Create the special "anyerror" set
    pub fn anyerror() -> Self {
        Self {
            name: "anyerror".to_string(),
            errors: Vec::new(),
            is_anyerror: true,
        }
    }

    /// Add an error variant
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Check if error set contains an error
    pub fn contains(&self, error: &str) -> bool {
        if self.is_anyerror {
            true
        } else {
            self.errors.contains(&error.to_string())
        }
    }
}

/// Error union model (T || E)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorUnionModel {
    /// Payload type ID
    pub payload_type: u64,
    /// Error set type ID
    pub error_set: u64,
    /// Is this error-only (no payload)?
    pub is_error_only: bool,
}

impl ErrorUnionModel {
    /// Create a new error union
    pub fn new(payload_type: u64, error_set: u64) -> Self {
        Self {
            payload_type,
            error_set,
            is_error_only: false,
        }
    }

    /// Create an error-only union (no payload)
    pub fn error_only(error_set: u64) -> Self {
        Self {
            payload_type: 0,
            error_set,
            is_error_only: true,
        }
    }
}

/// Error handling tracker for a function
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorTracking {
    /// Error sets used in this context
    error_sets: HashSet<u64>,
    /// Error unions used in this context
    error_unions: HashSet<u64>,
    /// Functions that can return errors
    error_returning: HashSet<u64>,
}

impl ErrorTracking {
    /// Create new error tracking
    pub fn new() -> Self {
        Self {
            error_sets: HashSet::new(),
            error_unions: HashSet::new(),
            error_returning: HashSet::new(),
        }
    }

    /// Register an error set
    pub fn register_error_set(&mut self, id: u64) {
        self.error_sets.insert(id);
    }

    /// Register an error union
    pub fn register_error_union(&mut self, id: u64) {
        self.error_unions.insert(id);
    }

    /// Register an error-returning function
    pub fn register_error_returning(&mut self, id: u64) {
        self.error_returning.insert(id);
    }

    /// Check if an error set is used
    pub fn uses_error_set(&self, id: u64) -> bool {
        self.error_sets.contains(&id)
    }

    /// Check if an error union is used
    pub fn uses_error_union(&self, id: u64) -> bool {
        self.error_unions.contains(&id)
    }

    /// Check if a function returns errors
    pub fn is_error_returning(&self, id: u64) -> bool {
        self.error_returning.contains(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_set_creation() {
        let mut set = ErrorSetModel::new("FileError".to_string());
        set.add_error("NotFound".to_string());
        set.add_error("PermissionDenied".to_string());

        assert_eq!(set.name, "FileError");
        assert_eq!(set.errors.len(), 2);
        assert!(!set.is_anyerror);
    }

    #[test]
    fn test_error_set_anyerror() {
        let set = ErrorSetModel::anyerror();
        assert!(set.is_anyerror);
        assert!(set.contains("AnyError"));
        assert!(set.contains("SomeOtherError"));
    }

    #[test]
    fn test_error_union_creation() {
        let eu = ErrorUnionModel::new(100, 200);
        assert_eq!(eu.payload_type, 100);
        assert_eq!(eu.error_set, 200);
        assert!(!eu.is_error_only);
    }

    #[test]
    fn test_error_union_error_only() {
        let eu = ErrorUnionModel::error_only(200);
        assert!(eu.is_error_only);
        assert_eq!(eu.payload_type, 0);
    }

    #[test]
    fn test_error_tracking() {
        let mut tracking = ErrorTracking::new();
        tracking.register_error_set(1);
        tracking.register_error_union(2);
        tracking.register_error_returning(3);

        assert!(tracking.uses_error_set(1));
        assert!(!tracking.uses_error_set(99));
        assert!(tracking.uses_error_union(2));
        assert!(tracking.is_error_returning(3));
    }
}
