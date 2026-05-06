//! Effect inference engine.
//!
//! Tracks and infers effects from BEAM operations.

use super::effect::{EffectInfo, EffectLocation, EffectSeverity, EffectType};
use super::EffectCategory;
use std::collections::{HashMap, HashSet};

/// Context for effect inference.
#[derive(Debug, Clone)]
pub struct EffectContext {
    /// Current module.
    pub module: String,
    /// Current function.
    pub function: String,
    /// Call stack depth.
    pub depth: usize,
    /// Whether we're in a try/catch block.
    pub in_try_catch: bool,
    /// Whether we're in a receive block.
    pub in_receive: bool,
    /// Captured variables (for closure effects).
    pub captured_vars: HashSet<String>,
}

impl EffectContext {
    /// Create a new context.
    pub fn new(module: impl Into<String>, function: impl Into<String>) -> Self {
        EffectContext {
            module: module.into(),
            function: function.into(),
            depth: 0,
            in_try_catch: false,
            in_receive: false,
            captured_vars: HashSet::new(),
        }
    }

    /// Push a new scope.
    pub fn push_scope(&self) -> Self {
        let mut ctx = self.clone();
        ctx.depth += 1;
        ctx
    }

    /// Set try/catch flag.
    pub fn with_try_catch(&self, in_try: bool) -> Self {
        let mut ctx = self.clone();
        ctx.in_try_catch = in_try;
        ctx
    }

    /// Set receive flag.
    pub fn with_receive(&self, in_recv: bool) -> Self {
        let mut ctx = self.clone();
        ctx.in_receive = in_recv;
        ctx
    }

    /// Add a captured variable.
    pub fn add_captured(&self, var: impl Into<String>) -> Self {
        let mut ctx = self.clone();
        ctx.captured_vars.insert(var.into());
        ctx
    }

    /// Get current location.
    pub fn location(&self, line: u32, column: u32) -> EffectLocation {
        EffectLocation::new(&self.module, &self.function, line, column)
    }
}

impl Default for EffectContext {
    fn default() -> Self {
        Self::new("unknown", "unknown")
    }
}

/// Result of effect analysis.
#[derive(Debug, Clone)]
pub struct EffectResult {
    /// All effects found.
    pub effects: Vec<EffectInfo>,
    /// Effects grouped by category.
    pub by_category: HashMap<String, Vec<usize>>,
    /// Whether function is pure.
    pub is_pure: bool,
    /// Whether function may diverge (throw/loop).
    pub may_diverge: bool,
    /// Summary string.
    pub summary: String,
}

impl EffectResult {
    /// Create a new result.
    pub fn new() -> Self {
        EffectResult {
            effects: vec![],
            by_category: HashMap::new(),
            is_pure: true,
            may_diverge: false,
            summary: String::new(),
        }
    }

    /// Add an effect.
    pub fn add_effect(&mut self, effect: EffectInfo) {
        let effect_type = effect.effect_type;
        let severity = effect.severity;
        let category = effect_type_category(&effect_type);
        let cat_str = category.as_str();

        self.by_category
            .entry(cat_str.to_string())
            .or_default()
            .push(self.effects.len());

        self.effects.push(effect);

        // Update purity
        if severity != EffectSeverity::Pure {
            self.is_pure = false;
        }

        // Update divergence
        if matches!(effect_type, EffectType::ProcessExit) {
            self.may_diverge = true;
        }
    }

    /// Check if has effect of type.
    pub fn has_effect_type(&self, effect_type: EffectType) -> bool {
        self.effects.iter().any(|e| e.effect_type == effect_type)
    }

    /// Check if has any spawn effect.
    pub fn may_spawn(&self) -> bool {
        self.has_effect_type(EffectType::ProcessSpawn)
    }

    /// Check if has any message effect.
    pub fn may_message(&self) -> bool {
        self.has_effect_type(EffectType::MessageSend)
    }

    /// Check if has any receive effect.
    pub fn may_receive(&self) -> bool {
        self.has_effect_type(EffectType::MessageReceive)
    }

    /// Get effects by category.
    pub fn get_by_category(&self, category: &str) -> Vec<&EffectInfo> {
        match self.by_category.get(category) {
            Some(indices) => indices
                .iter()
                .filter_map(|i| self.effects.get(*i))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Build summary string.
    pub fn build_summary(&mut self) {
        let mut parts = vec![];
        if self.is_pure {
            parts.push("pure".to_string());
        }
        if self.may_diverge {
            parts.push("may_diverge".to_string());
        }
        for (cat, indices) in &self.by_category {
            if !indices.is_empty() {
                parts.push(format!("{}({})", cat, indices.len()));
            }
        }
        self.summary = parts.join(", ");
    }
}

impl Default for EffectResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Map effect type to category.
fn effect_type_category(effect_type: &EffectType) -> EffectCategory {
    match effect_type {
        EffectType::ProcessSpawn => EffectCategory::Spawn,
        EffectType::MessageSend => EffectCategory::Message,
        EffectType::MessageReceive => EffectCategory::Receive,
        EffectType::TimerSchedule => EffectCategory::Timing,
        EffectType::ProcessLink | EffectType::ProcessMonitor | EffectType::ProcessExit => {
            EffectCategory::Lifecycle
        }
        EffectType::CodeLoad => EffectCategory::CodeLoad,
        EffectType::Registry => EffectCategory::Registry,
        EffectType::Distribution => EffectCategory::Distribution,
        EffectType::NifCall => EffectCategory::External,
        EffectType::ProcessInfo => EffectCategory::Lifecycle,
        EffectType::MemoryAlloc => EffectCategory::External,
    }
}

/// Effect tracker for collecting and analyzing effects.
#[derive(Debug, Clone)]
pub struct EffectTracker {
    /// Current context.
    context: EffectContext,
    /// Results by function.
    results: HashMap<String, EffectResult>,
    /// Call graph (caller -> callees).
    call_graph: HashMap<String, Vec<String>>,
}

impl EffectTracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        EffectTracker {
            context: EffectContext::default(),
            results: HashMap::new(),
            call_graph: HashMap::new(),
        }
    }

    /// Set context.
    pub fn set_context(&mut self, context: EffectContext) {
        self.context = context;
    }

    /// Get current context.
    pub fn context(&self) -> &EffectContext {
        &self.context
    }

    /// Record an effect.
    pub fn record_effect(&mut self, effect: EffectInfo) {
        let key = format!("{}:{}", self.context.module, self.context.function);
        let result = self.results.entry(key).or_insert_with(EffectResult::new);
        result.add_effect(effect);
    }

    /// Record effect from operation.
    pub fn record(&mut self, effect_type: EffectType, line: u32, column: u32) {
        let location = self.context.location(line, column);
        let effect = EffectInfo::at(effect_type, location);
        self.record_effect(effect);
    }

    /// Record spawn effect.
    pub fn record_spawn(&mut self, line: u32, column: u32) {
        self.record(EffectType::ProcessSpawn, line, column);
    }

    /// Record message send effect.
    pub fn record_send(&mut self, line: u32, column: u32) {
        self.record(EffectType::MessageSend, line, column);
    }

    /// Record receive effect.
    pub fn record_receive(&mut self, line: u32, column: u32) {
        self.record(EffectType::MessageReceive, line, column);
    }

    /// Record link effect.
    pub fn record_link(&mut self, line: u32, column: u32) {
        self.record(EffectType::ProcessLink, line, column);
    }

    /// Record exit effect.
    pub fn record_exit(&mut self, line: u32, column: u32) {
        self.record(EffectType::ProcessExit, line, column);
    }

    /// Record a function call.
    pub fn record_call(&mut self, callee: &str) {
        let caller = format!("{}:{}", self.context.module, self.context.function);
        self.call_graph
            .entry(caller)
            .or_insert_with(Vec::new)
            .push(callee.to_string());
    }

    /// Get result for a function.
    pub fn get_result(&self, module: &str, function: &str) -> Option<&EffectResult> {
        let key = format!("{}:{}", module, function);
        self.results.get(&key)
    }

    /// Get all results.
    pub fn results(&self) -> &HashMap<String, EffectResult> {
        &self.results
    }

    /// Get call graph.
    pub fn call_graph(&self) -> &HashMap<String, Vec<String>> {
        &self.call_graph
    }

    /// Build summaries for all results.
    pub fn build_all_summaries(&mut self) {
        for result in self.results.values_mut() {
            result.build_summary();
        }
    }
}

impl Default for EffectTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_context_new() {
        let ctx = EffectContext::new("mod", "fun");
        assert_eq!(ctx.module, "mod");
        assert_eq!(ctx.function, "fun");
        assert_eq!(ctx.depth, 0);
    }

    #[test]
    fn test_effect_context_push_scope() {
        let ctx = EffectContext::new("mod", "fun");
        let pushed = ctx.push_scope();
        assert_eq!(pushed.depth, 1);
    }

    #[test]
    fn test_effect_context_try_catch() {
        let ctx = EffectContext::new("mod", "fun");
        let in_try = ctx.with_try_catch(true);
        assert!(in_try.in_try_catch);
    }

    #[test]
    fn test_effect_context_receive() {
        let ctx = EffectContext::new("mod", "fun");
        let in_recv = ctx.with_receive(true);
        assert!(in_recv.in_receive);
    }

    #[test]
    fn test_effect_result_new() {
        let result = EffectResult::new();
        assert!(result.effects.is_empty());
        assert!(result.is_pure);
        assert!(!result.may_diverge);
    }

    #[test]
    fn test_effect_result_add_effect() {
        let mut result = EffectResult::new();
        let loc = EffectLocation::new("mod", "fun", 1, 1);
        result.add_effect(EffectInfo::at(EffectType::MessageSend, loc));
        assert!(!result.is_pure);
        assert!(result.may_message());
    }

    #[test]
    fn test_effect_result_may_spawn() {
        let mut result = EffectResult::new();
        let loc = EffectLocation::new("mod", "fun", 1, 1);
        result.add_effect(EffectInfo::at(EffectType::ProcessSpawn, loc));
        assert!(result.may_spawn());
    }

    #[test]
    fn test_effect_result_may_diverge() {
        let mut result = EffectResult::new();
        let loc = EffectLocation::new("mod", "fun", 1, 1);
        result.add_effect(EffectInfo::at(EffectType::ProcessExit, loc));
        assert!(result.may_diverge);
    }

    #[test]
    fn test_effect_tracker_record() {
        let mut tracker = EffectTracker::new();
        tracker.set_context(EffectContext::new("mod", "fun"));
        tracker.record_spawn(10, 5);
        assert!(tracker.results.len() >= 1);
    }

    #[test]
    fn test_effect_tracker_call() {
        let mut tracker = EffectTracker::new();
        tracker.set_context(EffectContext::new("mod", "fun"));
        tracker.record_call("mod:callee");
        let calls = tracker.call_graph().get("mod:fun");
        assert!(calls.is_some());
        assert_eq!(calls.unwrap()[0], "mod:callee");
    }
}
