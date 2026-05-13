//! Ownership validation for BEAM values.
//!
//! Validates that ownership transfers and references are safe.

use super::categories::{HeapOwnership, OwnershipCategory};
use super::tracking::OwnershipTracker;
use chimera_beam_process::BeamPid;
use serde::{Deserialize, Serialize};

/// Validation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    /// Validation passed.
    Ok,
    /// Warning (non-fatal).
    Warning(String),
    /// Error (fatal).
    Error(String),
}

impl ValidationResult {
    /// Check if result is OK.
    pub fn is_ok(&self) -> bool {
        matches!(self, ValidationResult::Ok)
    }

    /// Check if result is error.
    pub fn is_error(&self) -> bool {
        matches!(self, ValidationResult::Error(_))
    }

    /// Get error message if error.
    pub fn error_message(&self) -> Option<&str> {
        match self {
            ValidationResult::Error(msg) => Some(msg),
            _ => None,
        }
    }
}

impl std::fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationResult::Ok => write!(f, "ok"),
            ValidationResult::Warning(msg) => write!(f, "warning: {}", msg),
            ValidationResult::Error(msg) => write!(f, "error: {}", msg),
        }
    }
}

/// Ownership validator.
#[derive(Debug, Clone)]
pub struct OwnershipValidator {
    /// Enable strict mode.
    strict: bool,
    /// Track violations.
    violations: Vec<Violation>,
}

impl OwnershipValidator {
    /// Create a new validator.
    pub fn new() -> Self {
        OwnershipValidator {
            strict: false,
            violations: vec![],
        }
    }

    /// Create with strict mode.
    pub fn with_strict(strict: bool) -> Self {
        OwnershipValidator {
            strict,
            violations: vec![],
        }
    }

    /// Validate process ownership.
    pub fn validate_process_ownership(
        &self,
        pid: BeamPid,
        tracker: &OwnershipTracker,
    ) -> ValidationResult {
        // Check if process is registered
        if let Some(ownership) = tracker.get_process_ownership(pid.to_u64()) {
            // Process is registered, check heap category
            if ownership.heap_category == OwnershipCategory::Owned {
                ValidationResult::Ok
            } else if ownership.heap_category == OwnershipCategory::GcRoot {
                ValidationResult::Warning(format!("process {} heap is GC-managed", pid))
            } else {
                ValidationResult::Error(format!("process {} has invalid heap category", pid))
            }
        } else {
            ValidationResult::Error(format!("process {} not registered", pid))
        }
    }

    /// Validate heap ownership.
    pub fn validate_heap_ownership(
        &self,
        address: u64,
        tracker: &OwnershipTracker,
    ) -> ValidationResult {
        if let Some(ownership) = tracker.get_heap_ownership(address) {
            // Check if movable value is being shared
            if ownership.movable && !ownership.can_share() {
                ValidationResult::Error(format!(
                    "movable value at 0x{:x} cannot be shared",
                    address
                ))
            } else if ownership.category == OwnershipCategory::Weak {
                ValidationResult::Warning(format!(
                    "weak reference at 0x{:x} may be invalid",
                    address
                ))
            } else {
                ValidationResult::Ok
            }
        } else {
            ValidationResult::Error(format!("heap value at 0x{:x} not found", address))
        }
    }

    /// Validate ownership transfer.
    pub fn validate_transfer(
        &self,
        from_pid: BeamPid,
        to_pid: BeamPid,
        tracker: &OwnershipTracker,
    ) -> ValidationResult {
        // Source must exist
        if tracker.get_process_ownership(from_pid.to_u64()).is_none() {
            return ValidationResult::Error(format!("source process {} not found", from_pid));
        }

        // Destination must exist
        if tracker.get_process_ownership(to_pid.to_u64()).is_none() {
            return ValidationResult::Error(format!("destination process {} not found", to_pid));
        }

        // Check if transfer is valid (different processes)
        if from_pid == to_pid {
            return ValidationResult::Warning("transfer to same process".to_string());
        }

        ValidationResult::Ok
    }

    /// Validate message send.
    pub fn validate_message_send(
        &self,
        src_pid: BeamPid,
        dst_pid: BeamPid,
        tracker: &OwnershipTracker,
    ) -> ValidationResult {
        // Source must exist
        if tracker.get_process_ownership(src_pid.to_u64()).is_none() {
            return ValidationResult::Error(format!("source process {} not found", src_pid));
        }

        // Destination must exist
        if tracker.get_process_ownership(dst_pid.to_u64()).is_none() {
            return ValidationResult::Error(format!("destination process {} not found", dst_pid));
        }

        ValidationResult::Ok
    }

    /// Validate spawn with ownership.
    pub fn validate_spawn(
        &self,
        parent_pid: BeamPid,
        child_pid: BeamPid,
        tracker: &OwnershipTracker,
    ) -> ValidationResult {
        // Parent must exist
        if tracker.get_process_ownership(parent_pid.to_u64()).is_none() {
            return ValidationResult::Error(format!("parent process {} not found", parent_pid));
        }

        // In strict mode, require child to be registered
        if self.strict && tracker.get_process_ownership(child_pid.to_u64()).is_none() {
            return ValidationResult::Error(format!("child process {} not registered", child_pid));
        }

        ValidationResult::Ok
    }

    /// Record a violation.
    pub fn record_violation(&mut self, violation: Violation) {
        self.violations.push(violation);
    }

    /// Get all violations.
    pub fn violations(&self) -> &[Violation] {
        &self.violations
    }

    /// Clear violations.
    pub fn clear_violations(&mut self) {
        self.violations.clear();
    }

    /// Check if any violations occurred.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }
}

impl Default for OwnershipValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// An ownership violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// Violation type.
    pub violation_type: ViolationType,
    /// Description.
    pub description: String,
    /// Location (if known).
    pub location: Option<String>,
    /// Timestamp.
    pub timestamp: u64,
}

impl Violation {
    /// Create a new violation.
    pub fn new(violation_type: ViolationType, description: impl Into<String>) -> Self {
        Violation {
            violation_type,
            description: description.into(),
            location: None,
            timestamp: current_time_ms(),
        }
    }

    /// Set location.
    pub fn at(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

/// Violation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    /// Use-after-free.
    UseAfterFree,
    /// Double-free.
    DoubleFree,
    /// Invalid transfer.
    InvalidTransfer,
    /// Sharing violation.
    SharingViolation,
    /// Not found.
    NotFound,
    /// Category mismatch.
    CategoryMismatch,
}

impl ViolationType {
    /// Get type name.
    pub fn as_str(&self) -> &'static str {
        match self {
            ViolationType::UseAfterFree => "use_after_free",
            ViolationType::DoubleFree => "double_free",
            ViolationType::InvalidTransfer => "invalid_transfer",
            ViolationType::SharingViolation => "sharing_violation",
            ViolationType::NotFound => "not_found",
            ViolationType::CategoryMismatch => "category_mismatch",
        }
    }
}

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_ok() {
        let result = ValidationResult::Ok;
        assert!(result.is_ok());
        assert!(!result.is_error());
    }

    #[test]
    fn test_validation_result_error() {
        let result = ValidationResult::Error("test error".to_string());
        assert!(!result.is_ok());
        assert!(result.is_error());
        assert_eq!(result.error_message(), Some("test error"));
    }

    #[test]
    fn test_validation_result_warning() {
        let result = ValidationResult::Warning("test warning".to_string());
        // Warning is not error, but not strictly "ok" either
        assert!(!result.is_error());
    }

    #[test]
    fn test_ownership_validator_new() {
        let validator = OwnershipValidator::new();
        assert!(!validator.has_violations());
    }

    #[test]
    fn test_ownership_validator_strict() {
        let validator = OwnershipValidator::with_strict(true);
        assert!(validator.violations.is_empty());
    }

    #[test]
    fn test_ownership_validator_record_violation() {
        let mut validator = OwnershipValidator::new();
        validator.record_violation(Violation::new(
            ViolationType::UseAfterFree,
            "test violation",
        ));
        assert!(validator.has_violations());
        assert_eq!(validator.violations().len(), 1);
    }

    #[test]
    fn test_ownership_validator_clear() {
        let mut validator = OwnershipValidator::new();
        validator.record_violation(Violation::new(ViolationType::DoubleFree, "test"));
        validator.clear_violations();
        assert!(!validator.has_violations());
    }

    #[test]
    fn test_violation_new() {
        let violation = Violation::new(ViolationType::UseAfterFree, "test");
        assert_eq!(violation.violation_type, ViolationType::UseAfterFree);
        assert_eq!(violation.description, "test");
        assert!(violation.location.is_none());
    }

    #[test]
    fn test_violation_at() {
        let violation = Violation::new(ViolationType::InvalidTransfer, "test").at("mod:fun/1");
        assert_eq!(violation.location, Some("mod:fun/1".to_string()));
    }

    #[test]
    fn test_violation_type_as_str() {
        assert_eq!(ViolationType::UseAfterFree.as_str(), "use_after_free");
        assert_eq!(ViolationType::DoubleFree.as_str(), "double_free");
    }
}
