//! BEAM effect inference for ChimeraIR.
//!
//! Models and infers observable effects from BEAM programs:
//! message send, receive, spawn, timer, link, exit, code load, registry.

pub mod classify;
pub mod effect;
pub mod inference;

pub use classify::{classify_operation, EffectClassifier};
pub use effect::{EffectInfo, EffectLocation, EffectSeverity, EffectType};
pub use inference::{EffectContext, EffectResult, EffectTracker};

use serde::{Deserialize, Serialize};

/// Maximum effects per function.
pub const MAX_EFFECTS_PER_FUNCTION: usize = 256;

/// Effect categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectCategory {
    /// Process spawn effect.
    Spawn,
    /// Message passing effect.
    Message,
    /// Message receive effect.
    Receive,
    /// Timing/scheduling effect.
    Timing,
    /// Process lifecycle effect.
    Lifecycle,
    /// Code loading effect.
    CodeLoad,
    /// Registry effect.
    Registry,
    /// Distribution effect.
    Distribution,
    /// NIF/external call effect.
    External,
}

impl EffectCategory {
    /// Get category name.
    pub fn as_str(&self) -> &'static str {
        match self {
            EffectCategory::Spawn => "spawn",
            EffectCategory::Message => "message",
            EffectCategory::Receive => "receive",
            EffectCategory::Timing => "timing",
            EffectCategory::Lifecycle => "lifecycle",
            EffectCategory::CodeLoad => "code_load",
            EffectCategory::Registry => "registry",
            EffectCategory::Distribution => "distribution",
            EffectCategory::External => "external",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_category_as_str() {
        assert_eq!(EffectCategory::Spawn.as_str(), "spawn");
        assert_eq!(EffectCategory::Message.as_str(), "message");
        assert_eq!(EffectCategory::Receive.as_str(), "receive");
    }

    #[test]
    fn test_max_effects() {
        assert!(MAX_EFFECTS_PER_FUNCTION > 0);
    }
}
