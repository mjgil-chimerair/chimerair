//! Restart strategies for BEAM supervisors.
//!
//! Defines how a supervisor responds when a child process fails.

use serde::{Deserialize, Serialize};

/// Restart strategy for a supervisor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartStrategy {
    /// Only the failed child restarts.
    OneForOne,
    /// All children terminate and restart.
    OneForAll,
    /// Failed child and subsequent siblings restart.
    RestForOne,
    /// All children are identical (parameterized spawn).
    SimpleOneForOne,
}

impl Default for RestartStrategy {
    fn default() -> Self {
        RestartStrategy::OneForOne
    }
}

impl RestartStrategy {
    /// Get the strategy name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            RestartStrategy::OneForOne => "one_for_one",
            RestartStrategy::OneForAll => "one_for_all",
            RestartStrategy::RestForOne => "rest_for_one",
            RestartStrategy::SimpleOneForOne => "simple_one_for_one",
        }
    }

    /// Parse from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "one_for_one" => Some(RestartStrategy::OneForOne),
            "one_for_all" => Some(RestartStrategy::OneForAll),
            "rest_for_one" => Some(RestartStrategy::RestForOne),
            "simple_one_for_one" => Some(RestartStrategy::SimpleOneForOne),
            _ => None,
        }
    }
}

/// Restart intensity configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartIntensity {
    /// Maximum number of restarts within the period.
    pub max: u32,
    /// Time period in seconds.
    pub period: u32,
}

impl RestartIntensity {
    /// Create a new restart intensity.
    pub fn new(max: u32, period: u32) -> Self {
        RestartIntensity { max, period }
    }

    /// Default intensity: 3 restarts per 5 seconds.
    pub fn default_intensity() -> Self {
        RestartIntensity { max: 3, period: 5 }
    }

    /// Check if restart is within intensity bounds.
    pub fn is_within_bounds(&self, restart_count: u32) -> bool {
        restart_count < self.max
    }
}

impl Default for RestartIntensity {
    fn default() -> Self {
        Self::default_intensity()
    }
}

/// Restart kind for a child.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartKind {
    /// Always restart the child.
    Permanent,
    /// Never restart the child (no crash logging).
    Temporary,
    /// Only restart on abnormal exit.
    Transient,
}

impl Default for RestartKind {
    fn default() -> Self {
        RestartKind::Permanent
    }
}

impl RestartKind {
    /// Get the kind as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            RestartKind::Permanent => "permanent",
            RestartKind::Temporary => "temporary",
            RestartKind::Transient => "transient",
        }
    }

    /// Parse from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "permanent" => Some(RestartKind::Permanent),
            "temporary" => Some(RestartKind::Temporary),
            "transient" => Some(RestartKind::Transient),
            _ => None,
        }
    }

    /// Check if a child with this restart kind should restart for the given reason.
    pub fn should_restart(&self, exited_normal: bool) -> bool {
        match self {
            RestartKind::Permanent => true,
            RestartKind::Temporary => false,
            RestartKind::Transient => !exited_normal,
        }
    }
}

/// Helper to calculate which children to restart based on strategy.
pub fn get_children_to_restart(
    strategy: RestartStrategy,
    failed_child_id: &str,
    child_ids: &[String],
) -> Vec<String> {
    match strategy {
        RestartStrategy::OneForOne => vec![failed_child_id.to_string()],
        RestartStrategy::OneForAll => child_ids.to_vec(),
        RestartStrategy::RestForOne => {
            let mut to_restart = vec![];
            let mut past_failed = false;
            for id in child_ids {
                if id == failed_child_id {
                    past_failed = true;
                    to_restart.push(id.clone());
                } else if past_failed {
                    to_restart.push(id.clone());
                }
            }
            to_restart
        }
        RestartStrategy::SimpleOneForOne => vec![failed_child_id.to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_as_str() {
        assert_eq!(RestartStrategy::OneForOne.as_str(), "one_for_one");
        assert_eq!(RestartStrategy::OneForAll.as_str(), "one_for_all");
        assert_eq!(RestartStrategy::RestForOne.as_str(), "rest_for_one");
        assert_eq!(
            RestartStrategy::SimpleOneForOne.as_str(),
            "simple_one_for_one"
        );
    }

    #[test]
    fn test_strategy_from_str() {
        assert_eq!(
            RestartStrategy::from_str("one_for_one"),
            Some(RestartStrategy::OneForOne)
        );
        assert_eq!(
            RestartStrategy::from_str("one_for_all"),
            Some(RestartStrategy::OneForAll)
        );
        assert_eq!(
            RestartStrategy::from_str("rest_for_one"),
            Some(RestartStrategy::RestForOne)
        );
        assert_eq!(
            RestartStrategy::from_str("simple_one_for_one"),
            Some(RestartStrategy::SimpleOneForOne)
        );
        assert_eq!(RestartStrategy::from_str("invalid"), None);
    }

    #[test]
    fn test_restart_kind_as_str() {
        assert_eq!(RestartKind::Permanent.as_str(), "permanent");
        assert_eq!(RestartKind::Temporary.as_str(), "temporary");
        assert_eq!(RestartKind::Transient.as_str(), "transient");
    }

    #[test]
    fn test_restart_kind_should_restart() {
        assert!(RestartKind::Permanent.should_restart(true));
        assert!(RestartKind::Permanent.should_restart(false));

        assert!(!RestartKind::Temporary.should_restart(true));
        assert!(!RestartKind::Temporary.should_restart(false));

        assert!(RestartKind::Transient.should_restart(false));
        assert!(!RestartKind::Transient.should_restart(true));
    }

    #[test]
    fn test_restart_intensity_default() {
        let intensity = RestartIntensity::default();
        assert_eq!(intensity.max, 3);
        assert_eq!(intensity.period, 5);
    }

    #[test]
    fn test_restart_intensity_within_bounds() {
        let intensity = RestartIntensity::new(3, 5);
        assert!(intensity.is_within_bounds(0));
        assert!(intensity.is_within_bounds(2));
        assert!(!intensity.is_within_bounds(3));
    }

    #[test]
    fn test_get_children_one_for_one() {
        let child_ids = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = get_children_to_restart(RestartStrategy::OneForOne, "b", &child_ids);
        assert_eq!(result, vec!["b"]);
    }

    #[test]
    fn test_get_children_one_for_all() {
        let child_ids = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = get_children_to_restart(RestartStrategy::OneForAll, "b", &child_ids);
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_get_children_rest_for_one() {
        let child_ids = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let result = get_children_to_restart(RestartStrategy::RestForOne, "b", &child_ids);
        assert_eq!(result, vec!["b", "c", "d"]);
    }

    #[test]
    fn test_get_children_simple_one_for_one() {
        let child_ids = vec!["a".to_string(), "b".to_string()];
        let result = get_children_to_restart(RestartStrategy::SimpleOneForOne, "a", &child_ids);
        assert_eq!(result, vec!["a"]);
    }
}
