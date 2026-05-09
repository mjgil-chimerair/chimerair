//! Comprehensive lowering of panic policy, generics, and comptime.
//!
//! Task 94: Panic policy - enforce no-unwind across boundaries
//! Task 95: Generics - type parameters and instantiations
//! Task 96: Comptime - value metadata and proof facts
//!
//! This module provides a unified interface for all three concerns.

use super::effects::PanicPolicy;
use super::generics::{ComptimeModel, ComptimeValue, GenericInstantiation, GenericModel};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Panic boundary configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicBoundary {
    /// Boundary name
    pub name: String,
    /// Is panic allowed across this boundary
    pub allows_panic: bool,
    /// Panic policy at this boundary
    pub policy: PanicPolicy,
}

/// Panic boundary registry
#[derive(Debug, Clone)]
pub struct PanicBoundaryRegistry {
    /// Registered boundaries
    boundaries: HashMap<String, PanicBoundary>,
    /// Default policy
    default_policy: PanicPolicy,
}

impl PanicBoundaryRegistry {
    /// Create a new registry
    pub fn new(default_policy: PanicPolicy) -> Self {
        Self {
            boundaries: HashMap::new(),
            default_policy,
        }
    }

    /// Register a boundary
    pub fn register(&mut self, boundary: PanicBoundary) {
        self.boundaries.insert(boundary.name.clone(), boundary);
    }

    /// Check if panic is allowed at a boundary
    pub fn allows_panic(&self, boundary: &str) -> bool {
        self.boundaries
            .get(boundary)
            .map(|b| b.allows_panic)
            .unwrap_or(false)
    }

    /// Get the policy at a boundary
    pub fn policy_at(&self, boundary: &str) -> PanicPolicy {
        self.boundaries
            .get(boundary)
            .map(|b| b.policy)
            .unwrap_or(self.default_policy)
    }

    /// Add a trusted computing base boundary
    pub fn add_tcb_boundary(&mut self, name: &str) {
        self.register(PanicBoundary {
            name: name.to_string(),
            allows_panic: false,
            policy: PanicPolicy::NoUnwind,
        });
    }

    /// Add an FFI boundary
    pub fn add_ffi_boundary(&mut self, name: &str) {
        self.register(PanicBoundary {
            name: name.to_string(),
            allows_panic: false,
            policy: PanicPolicy::NoUnwind,
        });
    }
}

/// Generic instantiation tracking
#[derive(Debug, Clone)]
pub struct InstantiationTracker {
    /// Instantiations by generic ID
    instantiations: HashMap<u64, Vec<GenericInstantiation>>,
    /// Generic function IDs
    generic_functions: HashSet<u64>,
}

impl InstantiationTracker {
    /// Create a new tracker
    pub fn new() -> Self {
        Self {
            instantiations: HashMap::new(),
            generic_functions: HashSet::new(),
        }
    }

    /// Register a generic function
    pub fn register_generic(&mut self, func_id: u64) {
        self.generic_functions.insert(func_id);
        self.instantiations.entry(func_id).or_default();
    }

    /// Add an instantiation
    pub fn add_instantiation(&mut self, inst: GenericInstantiation) {
        self.instantiations
            .entry(inst.generic_id)
            .or_default()
            .push(inst);
    }

    /// Get instantiations for a generic
    pub fn get_instantiations(&self, generic_id: u64) -> &[GenericInstantiation] {
        self.instantiations
            .get(&generic_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Check if a function is generic
    pub fn is_generic(&self, func_id: u64) -> bool {
        self.generic_functions.contains(&func_id)
    }

    /// Number of instantiations for a generic
    pub fn num_instantiations(&self, generic_id: u64) -> usize {
        self.instantiations
            .get(&generic_id)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}

impl Default for InstantiationTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Comptime value tracker
#[derive(Debug, Clone)]
pub struct ComptimeTracker {
    /// Known values
    values: HashMap<u64, ComptimeValue>,
    /// Functions that are comptime-only
    comptime_functions: HashSet<u64>,
    /// Dependencies between comptime values
    dependencies: HashMap<u64, Vec<u64>>,
}

impl ComptimeTracker {
    /// Create a new tracker
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            comptime_functions: HashSet::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Record a comptime value
    pub fn record(&mut self, id: u64, value: ComptimeValue) {
        self.values.insert(id, value);
    }

    /// Get a comptime value
    pub fn get(&self, id: u64) -> Option<&ComptimeValue> {
        self.values.get(&id)
    }

    /// Register a comptime-only function
    pub fn register_comptime_function(&mut self, func_id: u64) {
        self.comptime_functions.insert(func_id);
    }

    /// Check if a function is comptime-only
    pub fn is_comptime_function(&self, func_id: u64) -> bool {
        self.comptime_functions.contains(&func_id)
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, from: u64, to: u64) {
        self.dependencies.entry(from).or_default().push(to);
    }

    /// Get dependencies for a value
    pub fn get_dependencies(&self, id: u64) -> &[u64] {
        self.dependencies
            .get(&id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Check if a value is known at comptime
    pub fn is_comptime_known(&self, id: u64) -> bool {
        self.values.contains_key(&id)
    }
}

impl Default for ComptimeTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified lowering context for panic, generics, and comptime
#[derive(Debug, Clone)]
pub struct UnifiedLoweringContext {
    /// Panic boundary registry
    pub panic_boundaries: PanicBoundaryRegistry,
    /// Generic instantiation tracker
    pub instantiations: InstantiationTracker,
    /// Comptime value tracker
    pub comptime: ComptimeTracker,
    /// Generic model (for compatibility)
    pub generic_model: GenericModel,
    /// Comptime model (for compatibility)
    pub comptime_model: ComptimeModel,
}

impl UnifiedLoweringContext {
    /// Create a new context
    pub fn new() -> Self {
        let mut ctx = Self {
            panic_boundaries: PanicBoundaryRegistry::new(PanicPolicy::NoUnwind),
            instantiations: InstantiationTracker::new(),
            comptime: ComptimeTracker::new(),
            generic_model: GenericModel::new(),
            comptime_model: ComptimeModel::new(),
        };
        // Add default TCB boundary
        ctx.panic_boundaries.add_tcb_boundary("default_tcb");
        ctx
    }

    /// Validate a function's panic boundaries
    pub fn validate_panic_boundaries(&self, boundaries: &[String]) -> Vec<String> {
        let mut violations = Vec::new();
        for boundary in boundaries {
            if !self.panic_boundaries.allows_panic(boundary) {
                let policy = self.panic_boundaries.policy_at(boundary);
                if policy == PanicPolicy::NoUnwind {
                    violations.push(format!(
                        "panic not allowed at boundary '{}' (no_unwind policy)",
                        boundary
                    ));
                }
            }
        }
        violations
    }
}

impl Default for UnifiedLoweringContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panic_boundary_registry_creation() {
        let registry = PanicBoundaryRegistry::new(PanicPolicy::NoUnwind);
        assert!(!registry.allows_panic("unknown"));
    }

    #[test]
    fn test_register_panic_boundary() {
        let mut registry = PanicBoundaryRegistry::new(PanicPolicy::NoUnwind);
        registry.register(PanicBoundary {
            name: "test".to_string(),
            allows_panic: true,
            policy: PanicPolicy::AllowUnwind,
        });
        assert!(registry.allows_panic("test"));
        assert_eq!(registry.policy_at("test"), PanicPolicy::AllowUnwind);
    }

    #[test]
    fn test_add_tcb_boundary() {
        let mut registry = PanicBoundaryRegistry::new(PanicPolicy::NoUnwind);
        registry.add_tcb_boundary("ffi_boundary");
        assert!(!registry.allows_panic("ffi_boundary"));
    }

    #[test]
    fn test_instantiation_tracker_creation() {
        let tracker = InstantiationTracker::new();
        assert!(!tracker.is_generic(1));
    }

    #[test]
    fn test_register_generic() {
        let mut tracker = InstantiationTracker::new();
        tracker.register_generic(100);
        assert!(tracker.is_generic(100));
        assert_eq!(tracker.num_instantiations(100), 0);
    }

    #[test]
    fn test_add_instantiation() {
        let mut tracker = InstantiationTracker::new();
        tracker.register_generic(100);
        let inst = GenericInstantiation::new(100)
            .with_type_arg(1)
            .with_type_arg(2);
        tracker.add_instantiation(inst);
        assert_eq!(tracker.num_instantiations(100), 1);
    }

    #[test]
    fn test_comptime_tracker_creation() {
        let tracker = ComptimeTracker::new();
        assert!(!tracker.is_comptime_known(1));
    }

    #[test]
    fn test_record_comptime_value() {
        let mut tracker = ComptimeTracker::new();
        tracker.record(1, ComptimeValue::Int(42));
        assert!(tracker.is_comptime_known(1));
        assert_eq!(tracker.get(1), Some(&ComptimeValue::Int(42)));
    }

    #[test]
    fn test_register_comptime_function() {
        let mut tracker = ComptimeTracker::new();
        tracker.register_comptime_function(100);
        assert!(tracker.is_comptime_function(100));
        assert!(!tracker.is_comptime_function(200));
    }

    #[test]
    fn test_comptime_dependencies() {
        let mut tracker = ComptimeTracker::new();
        tracker.record(1, ComptimeValue::Int(10));
        tracker.record(2, ComptimeValue::Int(20));
        tracker.add_dependency(2, 1); // 2 depends on 1
        let deps = tracker.get_dependencies(2);
        assert!(deps.contains(&1));
    }

    #[test]
    fn test_unified_lowering_context_creation() {
        let ctx = UnifiedLoweringContext::new();
        assert!(!ctx.panic_boundaries.allows_panic("default_tcb"));
    }

    #[test]
    fn test_validate_panic_boundaries_no_violation() {
        let ctx = UnifiedLoweringContext::new();
        // Empty boundaries means no validation needed
        let violations = ctx.validate_panic_boundaries(&[]);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_validate_panic_boundaries_with_violation() {
        let mut ctx = UnifiedLoweringContext::new();
        ctx.panic_boundaries.add_ffi_boundary("export_boundary");
        let violations = ctx.validate_panic_boundaries(&["export_boundary".to_string()]);
        assert!(!violations.is_empty());
    }
}
