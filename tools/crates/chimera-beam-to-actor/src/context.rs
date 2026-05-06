//! Lowering context for BEAM to Actor conversion.
//!
//! Tracks state during the lowering process.

use chimera_beam_effects::EffectTracker;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context for lowering BEAM to Actor.
#[derive(Debug, Clone)]
pub struct LoweringContext {
    /// Module name being lowered.
    pub module: String,
    /// Function name being lowered.
    pub function: String,
    /// Registered actor operations.
    pub actor_ops: Vec<String>,
    /// Type mapping from BEAM types to Actor types.
    pub type_map: HashMap<String, String>,
    /// Effect tracker.
    effect_tracker: EffectTracker,
    /// Statistics.
    stats: LoweringStats,
}

impl LoweringContext {
    /// Create a new context.
    pub fn new(module: impl Into<String>, function: impl Into<String>) -> Self {
        LoweringContext {
            module: module.into(),
            function: function.into(),
            actor_ops: vec![],
            type_map: HashMap::new(),
            effect_tracker: EffectTracker::new(),
            stats: LoweringStats::default(),
        }
    }

    /// Add an actor operation.
    pub fn add_actor_op(&mut self, op: impl Into<String>) {
        self.actor_ops.push(op.into());
        self.stats.ops_converted += 1;
    }

    /// Set type mapping.
    pub fn set_type_mapping(&mut self, beam_type: &str, actor_type: &str) {
        self.type_map
            .insert(beam_type.to_string(), actor_type.to_string());
    }

    /// Get type mapping.
    pub fn get_type_mapping(&self, beam_type: &str) -> Option<&str> {
        self.type_map.get(beam_type).map(String::as_str)
    }

    /// Get effect tracker reference.
    pub fn effect_tracker(&self) -> &EffectTracker {
        &self.effect_tracker
    }

    /// Get mutable effect tracker.
    pub fn effect_tracker_mut(&mut self) -> &mut EffectTracker {
        &mut self.effect_tracker
    }

    /// Get statistics.
    pub fn stats(&self) -> &LoweringStats {
        &self.stats
    }

    /// Increment errors.
    pub fn add_error(&mut self) {
        self.stats.errors += 1;
    }

    /// Increment warnings.
    pub fn add_warning(&mut self) {
        self.stats.warnings += 1;
    }

    /// Get all collected actor operations.
    pub fn collected_ops(&self) -> &[String] {
        &self.actor_ops
    }
}

impl Default for LoweringContext {
    fn default() -> Self {
        Self::new("unknown", "unknown")
    }
}

/// Lowering statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoweringStats {
    /// Operations converted.
    pub ops_converted: u32,
    /// Errors encountered.
    pub errors: u32,
    /// Warnings issued.
    pub warnings: u32,
    /// Types mapped.
    pub types_mapped: u32,
}

impl LoweringStats {
    /// Create new stats.
    pub fn new() -> Self {
        LoweringStats {
            ops_converted: 0,
            errors: 0,
            warnings: 0,
            types_mapped: 0,
        }
    }

    /// Check if any errors.
    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }
}

/// Builder for lowering context.
#[derive(Debug, Clone)]
pub struct LoweringContextBuilder {
    module: String,
    function: String,
    type_mappings: Vec<(String, String)>,
}

impl LoweringContextBuilder {
    /// Create a new builder.
    pub fn new(module: impl Into<String>, function: impl Into<String>) -> Self {
        LoweringContextBuilder {
            module: module.into(),
            function: function.into(),
            type_mappings: vec![],
        }
    }

    /// Add a type mapping.
    pub fn add_type_mapping(mut self, beam_type: &str, actor_type: &str) -> Self {
        self.type_mappings
            .push((beam_type.to_string(), actor_type.to_string()));
        self
    }

    /// Build the context.
    pub fn build(self) -> LoweringContext {
        let mut ctx = LoweringContext::new(self.module, self.function);
        for (beam, actor) in self.type_mappings {
            ctx.set_type_mapping(&beam, &actor);
            ctx.stats.types_mapped += 1;
        }
        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowering_context_new() {
        let ctx = LoweringContext::new("mod", "fun");
        assert_eq!(ctx.module, "mod");
        assert_eq!(ctx.function, "fun");
        assert!(ctx.actor_ops.is_empty());
    }

    #[test]
    fn test_lowering_context_add_op() {
        let mut ctx = LoweringContext::new("mod", "fun");
        ctx.add_actor_op("actor.spawn");
        assert_eq!(ctx.actor_ops.len(), 1);
        assert_eq!(ctx.stats.ops_converted, 1);
    }

    #[test]
    fn test_lowering_context_type_mapping() {
        let mut ctx = LoweringContext::new("mod", "fun");
        ctx.set_type_mapping("beam.pid", "actor.pid");
        assert_eq!(ctx.get_type_mapping("beam.pid"), Some("actor.pid"));
    }

    #[test]
    fn test_lowering_context_add_error() {
        let mut ctx = LoweringContext::new("mod", "fun");
        ctx.add_error();
        assert!(ctx.stats.has_errors());
    }

    #[test]
    fn test_lowering_stats_new() {
        let stats = LoweringStats::new();
        assert!(!stats.has_errors());
    }

    #[test]
    fn test_lowering_context_builder() {
        let ctx = LoweringContextBuilder::new("mod", "fun")
            .add_type_mapping("beam.pid", "actor.pid")
            .add_type_mapping("beam.atom", "actor.atom")
            .build();
        assert_eq!(ctx.module, "mod");
        assert_eq!(ctx.type_map.len(), 2);
    }
}
