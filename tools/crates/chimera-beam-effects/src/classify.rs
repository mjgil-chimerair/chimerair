//! Effect classification for BEAM operations.
//!
//! Maps BEAM operations to their effect types.

use super::effect::{EffectInfo, EffectLocation, EffectSeverity, EffectType};
use super::EffectCategory;

/// Classifier for BEAM operations.
#[derive(Debug, Clone)]
pub struct EffectClassifier {
    /// Classification rules.
    rules: Vec<ClassificationRule>,
}

impl EffectClassifier {
    /// Create a new classifier.
    pub fn new() -> Self {
        let mut classifier = EffectClassifier { rules: vec![] };
        classifier.add_default_rules();
        classifier
    }

    /// Add default classification rules.
    fn add_default_rules(&mut self) {
        // Process lifecycle
        self.add_rule(
            "spawn",
            EffectType::ProcessSpawn,
            EffectSeverity::Global,
            EffectCategory::Spawn,
        );
        self.add_rule(
            "spawn_link",
            EffectType::ProcessSpawn,
            EffectSeverity::Global,
            EffectCategory::Spawn,
        );
        self.add_rule(
            "spawn_monitor",
            EffectType::ProcessSpawn,
            EffectSeverity::Global,
            EffectCategory::Spawn,
        );
        self.add_rule(
            "exit",
            EffectType::ProcessExit,
            EffectSeverity::Critical,
            EffectCategory::Lifecycle,
        );
        self.add_rule(
            "exit2",
            EffectType::ProcessExit,
            EffectSeverity::Critical,
            EffectCategory::Lifecycle,
        );
        self.add_rule(
            "kill",
            EffectType::ProcessExit,
            EffectSeverity::Critical,
            EffectCategory::Lifecycle,
        );

        // Links and monitors
        self.add_rule(
            "link",
            EffectType::ProcessLink,
            EffectSeverity::Global,
            EffectCategory::Lifecycle,
        );
        self.add_rule(
            "unlink",
            EffectType::ProcessLink,
            EffectSeverity::Global,
            EffectCategory::Lifecycle,
        );
        self.add_rule(
            "monitor",
            EffectType::ProcessMonitor,
            EffectSeverity::Local,
            EffectCategory::Lifecycle,
        );
        self.add_rule(
            "demonitor",
            EffectType::ProcessMonitor,
            EffectSeverity::Local,
            EffectCategory::Lifecycle,
        );

        // Message passing
        self.add_rule(
            "send",
            EffectType::MessageSend,
            EffectSeverity::Local,
            EffectCategory::Message,
        );
        self.add_rule(
            "send_after",
            EffectType::MessageSend,
            EffectSeverity::Local,
            EffectCategory::Message,
        );
        self.add_rule(
            "receive",
            EffectType::MessageReceive,
            EffectSeverity::Local,
            EffectCategory::Receive,
        );

        // Timing
        self.add_rule(
            "start_timer",
            EffectType::TimerSchedule,
            EffectSeverity::Local,
            EffectCategory::Timing,
        );
        self.add_rule(
            "cancel_timer",
            EffectType::TimerSchedule,
            EffectSeverity::Local,
            EffectCategory::Timing,
        );
        self.add_rule(
            "after",
            EffectType::TimerSchedule,
            EffectSeverity::Local,
            EffectCategory::Timing,
        );

        // Code loading
        self.add_rule(
            "code_load",
            EffectType::CodeLoad,
            EffectSeverity::Global,
            EffectCategory::CodeLoad,
        );
        self.add_rule(
            "code_replace",
            EffectType::CodeLoad,
            EffectSeverity::Global,
            EffectCategory::CodeLoad,
        );
        self.add_rule(
            "code_change",
            EffectType::CodeLoad,
            EffectSeverity::Global,
            EffectCategory::CodeLoad,
        );

        // Registry
        self.add_rule(
            "register",
            EffectType::Registry,
            EffectSeverity::Global,
            EffectCategory::Registry,
        );
        self.add_rule(
            "unregister",
            EffectType::Registry,
            EffectSeverity::Global,
            EffectCategory::Registry,
        );
        self.add_rule(
            "whereis",
            EffectType::Registry,
            EffectSeverity::Local,
            EffectCategory::Registry,
        );

        // Process info
        self.add_rule(
            "self",
            EffectType::ProcessInfo,
            EffectSeverity::Local,
            EffectCategory::Lifecycle,
        );
        self.add_rule(
            "process_info",
            EffectType::ProcessInfo,
            EffectSeverity::Local,
            EffectCategory::Lifecycle,
        );

        // NIF calls
        self.add_rule(
            "nif",
            EffectType::NifCall,
            EffectSeverity::Critical,
            EffectCategory::External,
        );

        // Distribution
        self.add_rule(
            "spawn_node",
            EffectType::Distribution,
            EffectSeverity::Global,
            EffectCategory::Distribution,
        );
    }

    /// Add a classification rule.
    pub fn add_rule(
        &mut self,
        op: impl Into<String>,
        effect_type: EffectType,
        severity: EffectSeverity,
        category: EffectCategory,
    ) {
        self.rules.push(ClassificationRule {
            op: op.into(),
            effect_type,
            severity,
            category,
        });
    }

    /// Classify an operation.
    pub fn classify(&self, op: &str) -> Option<EffectInfo> {
        self.rules.iter().find(|r| r.op == op).map(|r| EffectInfo {
            effect_type: r.effect_type,
            severity: r.severity,
            location: EffectLocation::unknown(),
            target: None,
            description: op.to_string(),
            tags: vec![r.category.as_str().to_string()],
        })
    }

    /// Classify with location.
    pub fn classify_at(&self, op: &str, location: EffectLocation) -> Option<EffectInfo> {
        self.classify(op).map(|mut e| {
            e.location = location;
            e
        })
    }

    /// Get all known operations.
    pub fn known_operations(&self) -> Vec<&str> {
        self.rules.iter().map(|r| r.op.as_str()).collect()
    }

    /// Get operations by category.
    pub fn operations_by_category(&self, category: EffectCategory) -> Vec<&str> {
        self.rules
            .iter()
            .filter(|r| r.category == category)
            .map(|r| r.op.as_str())
            .collect()
    }
}

impl Default for EffectClassifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Classification rule.
#[derive(Debug, Clone)]
struct ClassificationRule {
    op: String,
    effect_type: EffectType,
    severity: EffectSeverity,
    category: EffectCategory,
}

/// Classify a BEAM operation string directly.
pub fn classify_operation(op: &str) -> Option<EffectType> {
    let classifier = EffectClassifier::new();
    classifier.classify(op).map(|e| e.effect_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classifier_new() {
        let classifier = EffectClassifier::new();
        assert!(!classifier.known_operations().is_empty());
    }

    #[test]
    fn test_classifier_spawn() {
        let classifier = EffectClassifier::new();
        let effect = classifier.classify("spawn");
        assert!(effect.is_some());
        assert_eq!(effect.unwrap().effect_type, EffectType::ProcessSpawn);
    }

    #[test]
    fn test_classifier_send() {
        let classifier = EffectClassifier::new();
        let effect = classifier.classify("send");
        assert!(effect.is_some());
        assert_eq!(effect.unwrap().effect_type, EffectType::MessageSend);
    }

    #[test]
    fn test_classifier_receive() {
        let classifier = EffectClassifier::new();
        let effect = classifier.classify("receive");
        assert!(effect.is_some());
        assert_eq!(effect.unwrap().effect_type, EffectType::MessageReceive);
    }

    #[test]
    fn test_classifier_link() {
        let classifier = EffectClassifier::new();
        let effect = classifier.classify("link");
        assert!(effect.is_some());
        assert_eq!(effect.unwrap().effect_type, EffectType::ProcessLink);
    }

    #[test]
    fn test_classifier_exit() {
        let classifier = EffectClassifier::new();
        let effect = classifier.classify("exit");
        assert!(effect.is_some());
        assert_eq!(effect.unwrap().effect_type, EffectType::ProcessExit);
    }

    #[test]
    fn test_classifier_unknown() {
        let classifier = EffectClassifier::new();
        let effect = classifier.classify("unknown_op");
        assert!(effect.is_none());
    }

    #[test]
    fn test_classifier_operations_by_category() {
        let classifier = EffectClassifier::new();
        let spawn_ops = classifier.operations_by_category(EffectCategory::Spawn);
        assert!(!spawn_ops.is_empty());
        assert!(spawn_ops.contains(&"spawn"));
    }

    #[test]
    fn test_classify_operation() {
        assert_eq!(classify_operation("spawn"), Some(EffectType::ProcessSpawn));
        assert_eq!(classify_operation("send"), Some(EffectType::MessageSend));
        assert_eq!(classify_operation("unknown"), None);
    }

    #[test]
    fn test_classifier_with_location() {
        let classifier = EffectClassifier::new();
        let loc = EffectLocation::new("mod", "fun", 10, 5);
        let effect = classifier.classify_at("spawn", loc);
        assert!(effect.is_some());
        let e = effect.unwrap();
        assert_eq!(e.location.module, "mod");
        assert_eq!(e.location.line, 10);
    }
}
