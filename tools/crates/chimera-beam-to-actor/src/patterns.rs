//! Lowering patterns for BEAM to Actor.
//!
//! Defines patterns for matching and transforming BEAM operations.

use chimera_beam_dialect::ops::BeamOpKind;
use chimera_beam_dialect::types::BeamType;
use chimera_beam_dialect::BeamOp;
use serde::{Deserialize, Serialize};

/// A lowering pattern for BEAM to Actor.
#[derive(Debug, Clone)]
pub struct LoweringPattern {
    /// Pattern name.
    pub name: String,
    /// Source operation kinds this pattern matches.
    pub source_ops: Vec<BeamOpKind>,
    /// Transformation function name.
    pub transform: String,
    /// Priority (higher = applied first).
    pub priority: u32,
}

impl LoweringPattern {
    /// Create a new pattern.
    pub fn new(
        name: impl Into<String>,
        source_ops: Vec<BeamOpKind>,
        transform: impl Into<String>,
    ) -> Self {
        LoweringPattern {
            name: name.into(),
            source_ops,
            transform: transform.into(),
            priority: 0,
        }
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Check if this pattern matches an operation.
    pub fn matches(&self, op: &BeamOp) -> bool {
        self.source_ops.contains(&op.kind)
    }
}

/// Pattern matcher for BEAM operations.
#[derive(Debug, Clone, Default)]
pub struct PatternMatcher {
    /// Registered patterns.
    patterns: Vec<LoweringPattern>,
}

impl PatternMatcher {
    /// Create a new matcher.
    pub fn new() -> Self {
        let mut matcher = PatternMatcher { patterns: vec![] };
        matcher.add_default_patterns();
        matcher
    }

    /// Add default patterns.
    fn add_default_patterns(&mut self) {
        // Spawn patterns
        self.add_pattern(
            LoweringPattern::new(
                "spawn_to_actor_spawn",
                vec![BeamOpKind::Spawn],
                "beam.spawn -> actor.spawn",
            )
            .with_priority(100),
        );

        self.add_pattern(
            LoweringPattern::new(
                "spawn_link_to_actor_spawn_link",
                vec![BeamOpKind::SpawnLink],
                "beam.spawn_link -> actor.spawn_link",
            )
            .with_priority(100),
        );

        self.add_pattern(
            LoweringPattern::new(
                "spawn_monitor_to_actor_spawn_monitor",
                vec![BeamOpKind::SpawnMonitor],
                "beam.spawn_monitor -> actor.spawn_monitor",
            )
            .with_priority(100),
        );

        // Message patterns
        self.add_pattern(
            LoweringPattern::new(
                "send_to_actor_send",
                vec![BeamOpKind::Send],
                "beam.send -> actor.send",
            )
            .with_priority(100),
        );

        self.add_pattern(
            LoweringPattern::new(
                "recv_to_actor_receive",
                vec![BeamOpKind::Recv],
                "beam.recv -> actor.receive",
            )
            .with_priority(100),
        );

        // Lifecycle patterns
        self.add_pattern(
            LoweringPattern::new(
                "link_to_actor_link",
                vec![BeamOpKind::Link],
                "beam.link -> actor.link",
            )
            .with_priority(100),
        );

        self.add_pattern(
            LoweringPattern::new(
                "unlink_to_actor_unlink",
                vec![BeamOpKind::Unlink],
                "beam.unlink -> actor.unlink",
            )
            .with_priority(100),
        );

        self.add_pattern(
            LoweringPattern::new(
                "monitor_to_actor_monitor",
                vec![BeamOpKind::Monitor],
                "beam.monitor -> actor.monitor",
            )
            .with_priority(100),
        );

        self.add_pattern(
            LoweringPattern::new(
                "exit_to_actor_exit",
                vec![BeamOpKind::Exit],
                "beam.exit -> actor.exit",
            )
            .with_priority(100),
        );

        // Registry patterns
        self.add_pattern(
            LoweringPattern::new(
                "register_to_actor_register",
                vec![BeamOpKind::Register],
                "beam.register -> actor.register",
            )
            .with_priority(100),
        );

        self.add_pattern(
            LoweringPattern::new(
                "whereis_to_actor_whereis",
                vec![BeamOpKind::Whereis],
                "beam.whereis -> actor.whereis",
            )
            .with_priority(100),
        );
    }

    /// Add a pattern.
    pub fn add_pattern(&mut self, pattern: LoweringPattern) {
        self.patterns.push(pattern);
        // Sort by priority (descending)
        self.patterns.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Find matching pattern for an operation.
    pub fn find_match(&self, op: &BeamOp) -> Option<&LoweringPattern> {
        self.patterns.iter().find(|p| p.matches(op))
    }

    /// Get all patterns.
    pub fn patterns(&self) -> &[LoweringPattern] {
        &self.patterns
    }

    /// Get patterns for a specific operation kind.
    pub fn patterns_for(&self, op_kind: BeamOpKind) -> Vec<&LoweringPattern> {
        self.patterns
            .iter()
            .filter(|p| p.source_ops.contains(&op_kind))
            .collect()
    }
}

/// Result of pattern matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatchResult {
    /// Whether a match was found.
    pub matched: bool,
    /// The matched pattern.
    pub pattern: Option<String>,
    /// Transformed output.
    pub output: Vec<String>,
}

impl PatternMatchResult {
    /// Create a matched result.
    pub fn matched(pattern: &str, output: Vec<String>) -> Self {
        PatternMatchResult {
            matched: true,
            pattern: Some(pattern.to_string()),
            output,
        }
    }

    /// Create a no-match result.
    pub fn no_match() -> Self {
        PatternMatchResult {
            matched: false,
            pattern: None,
            output: vec![],
        }
    }
}

/// Apply patterns to an operation.
pub fn apply_patterns(op: &BeamOp, matcher: &PatternMatcher) -> PatternMatchResult {
    match matcher.find_match(op) {
        Some(pattern) => {
            // Simple transform based on pattern name
            let output = vec![pattern.transform.clone()];
            PatternMatchResult::matched(&pattern.name, output)
        }
        None => PatternMatchResult::no_match(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowering_pattern_new() {
        let pattern = LoweringPattern::new(
            "test_pattern",
            vec![BeamOpKind::Spawn],
            "spawn -> actor.spawn",
        );
        assert_eq!(pattern.name, "test_pattern");
        assert_eq!(pattern.source_ops.len(), 1);
    }

    #[test]
    fn test_lowering_pattern_matches() {
        let pattern = LoweringPattern::new(
            "spawn_pattern",
            vec![BeamOpKind::Spawn, BeamOpKind::SpawnLink],
            "transform",
        );
        let spawn_op = BeamOp::spawn("mod".to_string(), "fun".to_string());
        let send_op = BeamOp::send(BeamType::pid(), BeamType::atom());

        assert!(pattern.matches(&spawn_op));
        assert!(!pattern.matches(&send_op));
    }

    #[test]
    fn test_pattern_matcher_new() {
        let matcher = PatternMatcher::new();
        assert!(!matcher.patterns().is_empty());
    }

    #[test]
    fn test_pattern_matcher_find_match() {
        let matcher = PatternMatcher::new();
        let op = BeamOp::spawn("mod".to_string(), "fun".to_string());
        let matched = matcher.find_match(&op);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().name, "spawn_to_actor_spawn");
    }

    #[test]
    fn test_pattern_matcher_no_match() {
        let matcher = PatternMatcher::new();
        let op = BeamOp {
            kind: BeamOpKind::Now,
            name: "beam.now".to_string(),
            inputs: vec![],
            outputs: vec![],
            attributes: vec![],
            regions: 0,
        };
        let matched = matcher.find_match(&op);
        assert!(matched.is_none());
    }

    #[test]
    fn test_pattern_matcher_patterns_for() {
        let matcher = PatternMatcher::new();
        let spawn_patterns = matcher.patterns_for(BeamOpKind::Spawn);
        assert!(!spawn_patterns.is_empty());
    }

    #[test]
    fn test_pattern_match_result_matched() {
        let result = PatternMatchResult::matched("test_pattern", vec!["output".to_string()]);
        assert!(result.matched);
        assert_eq!(result.pattern, Some("test_pattern".to_string()));
    }

    #[test]
    fn test_pattern_match_result_no_match() {
        let result = PatternMatchResult::no_match();
        assert!(!result.matched);
        assert!(result.pattern.is_none());
    }

    #[test]
    fn test_apply_patterns() {
        let matcher = PatternMatcher::new();
        let op = BeamOp::spawn("mod".to_string(), "fun".to_string());
        let result = apply_patterns(&op, &matcher);
        assert!(result.matched);
    }

    #[test]
    fn test_apply_patterns_no_match() {
        let matcher = PatternMatcher::new();
        let op = BeamOp {
            kind: BeamOpKind::Now,
            name: "beam.now".to_string(),
            inputs: vec![],
            outputs: vec![],
            attributes: vec![],
            regions: 0,
        };
        let result = apply_patterns(&op, &matcher);
        assert!(!result.matched);
    }
}
