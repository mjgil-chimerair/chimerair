//! Chimera Rust Effects Analysis
//!
//! Extracts and analyzes effects from Rust code:
//! - Memory effects (reads, writes, allocations)
//! - Panic safety
//! - Unwind safety
//! - Divergence
//! - I/O operations

use serde::{Deserialize, Serialize};

/// Effect kinds in Rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectKind {
    /// Memory read
    Read,
    /// Memory write
    Write,
    /// Allocation
    Allocate,
    /// Deallocation
    Deallocate,
    /// Panic
    Panic,
    /// Divergence (infinite loop, panic, etc.)
    Diverges,
    /// System call
    SystemCall,
    /// File I/O
    FileIO,
    /// Network I/O
    NetworkIO,
    /// Thread spawn
    ThreadSpawn,
    /// Mutex lock
    Lock,
    /// Mutex unlock
    Unlock,
}

/// Effect location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectLocation {
    pub span: Span,
    pub effect_kind: EffectKind,
}

/// Effects summary for a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectsSummary {
    pub stable_id: String,
    pub effects: Vec<EffectLocation>,
    pub may_diverge: bool,
    pub may_panic: bool,
    pub may_allocate: bool,
    pub may_read: bool,
    pub may_write: bool,
    pub is_unsafe: bool,
}

/// Effects analyzer
pub struct EffectsAnalyzer {
    summaries: Vec<EffectsSummary>,
}

impl EffectsAnalyzer {
    /// Create a new effects analyzer
    pub fn new() -> Self {
        Self {
            summaries: Vec::new(),
        }
    }

    /// Record effects for a function
    pub fn record(&mut self, summary: EffectsSummary) {
        self.summaries.push(summary);
    }

    /// Record a memory read
    pub fn record_read(&mut self, stable_id: &str, span: Span) {
        if let Some(summary) = self.summaries.iter_mut().find(|s| s.stable_id == stable_id) {
            summary.may_read = true;
            summary.effects.push(EffectLocation {
                span,
                effect_kind: EffectKind::Read,
            });
        }
    }

    /// Record a memory write
    pub fn record_write(&mut self, stable_id: &str, span: Span) {
        if let Some(summary) = self.summaries.iter_mut().find(|s| s.stable_id == stable_id) {
            summary.may_write = true;
            summary.effects.push(EffectLocation {
                span,
                effect_kind: EffectKind::Write,
            });
        }
    }

    /// Record an allocation
    pub fn record_alloc(&mut self, stable_id: &str, span: Span) {
        if let Some(summary) = self.summaries.iter_mut().find(|s| s.stable_id == stable_id) {
            summary.may_allocate = true;
            summary.effects.push(EffectLocation {
                span,
                effect_kind: EffectKind::Allocate,
            });
        }
    }

    /// Record a panic point
    pub fn record_panic(&mut self, stable_id: &str, span: Span) {
        if let Some(summary) = self.summaries.iter_mut().find(|s| s.stable_id == stable_id) {
            summary.may_panic = true;
            summary.effects.push(EffectLocation {
                span,
                effect_kind: EffectKind::Panic,
            });
        }
    }

    /// Mark function as diverging
    pub fn mark_diverges(&mut self, stable_id: &str) {
        if let Some(summary) = self.summaries.iter_mut().find(|s| s.stable_id == stable_id) {
            summary.may_diverge = true;
        }
    }

    /// Get all summaries
    pub fn summaries(&self) -> &[EffectsSummary] {
        &self.summaries
    }

    /// Find function by stable ID
    pub fn find(&self, stable_id: &str) -> Option<&EffectsSummary> {
        self.summaries.iter().find(|s| s.stable_id == stable_id)
    }

    /// Check if function is pure (no side effects)
    pub fn is_pure(&self, stable_id: &str) -> bool {
        self.find(stable_id).map_or(false, |s| {
            !s.may_diverge && !s.may_panic && !s.may_allocate && !s.may_read && !s.may_write
        })
    }
}

impl Default for EffectsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Source span
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
    pub file_id: u32,
}

impl Default for Span {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            file_id: 0,
        }
    }
}

/// Emit effects to JSON
pub fn emit_effects_json(analyzer: &EffectsAnalyzer) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&analyzer.summaries)
}

/// Parse effects from JSON
pub fn parse_effects_json(json: &str) -> Result<Vec<EffectsSummary>, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_effects() {
        let mut analyzer = EffectsAnalyzer::new();

        analyzer.record(EffectsSummary {
            stable_id: "func1".to_string(),
            effects: Vec::new(),
            may_diverge: false,
            may_panic: false,
            may_allocate: false,
            may_read: true,
            may_write: false,
            is_unsafe: false,
        });

        analyzer.record_read("func1", Span::default());

        let summary = analyzer.find("func1").unwrap();
        assert!(summary.may_read);
        assert!(!summary.may_write);
    }

    #[test]
    fn test_purity_check() {
        let mut analyzer = EffectsAnalyzer::new();

        // Pure function
        analyzer.record(EffectsSummary {
            stable_id: "pure".to_string(),
            effects: Vec::new(),
            may_diverge: false,
            may_panic: false,
            may_allocate: false,
            may_read: false,
            may_write: false,
            is_unsafe: false,
        });

        // Impure function
        analyzer.record(EffectsSummary {
            stable_id: "impure".to_string(),
            effects: Vec::new(),
            may_diverge: false,
            may_panic: false,
            may_allocate: true,
            may_read: false,
            may_write: false,
            is_unsafe: false,
        });

        assert!(analyzer.is_pure("pure"));
        assert!(!analyzer.is_pure("impure"));
    }

    #[test]
    fn test_divergence() {
        let mut analyzer = EffectsAnalyzer::new();

        analyzer.record(EffectsSummary {
            stable_id: "loop".to_string(),
            effects: Vec::new(),
            may_diverge: false,
            may_panic: false,
            may_allocate: false,
            may_read: false,
            may_write: false,
            is_unsafe: false,
        });

        analyzer.mark_diverges("loop");

        let summary = analyzer.find("loop").unwrap();
        assert!(summary.may_diverge);
    }

    #[test]
    fn test_record_write() {
        let mut analyzer = EffectsAnalyzer::new();

        analyzer.record(EffectsSummary {
            stable_id: "writer".to_string(),
            effects: Vec::new(),
            may_diverge: false,
            may_panic: false,
            may_allocate: false,
            may_read: false,
            may_write: false,
            is_unsafe: false,
        });

        analyzer.record_write(
            "writer",
            Span {
                start: 10,
                end: 20,
                file_id: 1,
            },
        );

        let summary = analyzer.find("writer").unwrap();
        assert!(summary.may_write);
        assert_eq!(summary.effects.len(), 1);
    }

    #[test]
    fn test_record_alloc() {
        let mut analyzer = EffectsAnalyzer::new();

        analyzer.record(EffectsSummary {
            stable_id: "allocator".to_string(),
            effects: Vec::new(),
            may_diverge: false,
            may_panic: false,
            may_allocate: false,
            may_read: false,
            may_write: false,
            is_unsafe: false,
        });

        analyzer.record_alloc(
            "allocator",
            Span {
                start: 5,
                end: 15,
                file_id: 2,
            },
        );

        let summary = analyzer.find("allocator").unwrap();
        assert!(summary.may_allocate);
    }

    #[test]
    fn test_record_panic() {
        let mut analyzer = EffectsAnalyzer::new();

        analyzer.record(EffectsSummary {
            stable_id: "panicker".to_string(),
            effects: Vec::new(),
            may_diverge: false,
            may_panic: false,
            may_allocate: false,
            may_read: false,
            may_write: false,
            is_unsafe: false,
        });

        analyzer.record_panic(
            "panicker",
            Span {
                start: 100,
                end: 150,
                file_id: 3,
            },
        );

        let summary = analyzer.find("panicker").unwrap();
        assert!(summary.may_panic);
    }

    #[test]
    fn test_effect_kind_serialization() {
        let kind = EffectKind::Allocate;
        let json = serde_json::to_string(&kind).unwrap();
        let parsed: EffectKind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, EffectKind::Allocate);
    }

    #[test]
    fn test_span_default() {
        let span = Span::default();
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 0);
        assert_eq!(span.file_id, 0);
    }

    #[test]
    fn test_span_custom() {
        let span = Span {
            start: 10,
            end: 20,
            file_id: 5,
        };
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.file_id, 5);
    }

    #[test]
    fn test_effects_summary_serialization() {
        let summary = EffectsSummary {
            stable_id: "test_fn".to_string(),
            effects: vec![EffectLocation {
                span: Span {
                    start: 0,
                    end: 10,
                    file_id: 1,
                },
                effect_kind: EffectKind::Read,
            }],
            may_diverge: false,
            may_panic: true,
            may_allocate: false,
            may_read: true,
            may_write: false,
            is_unsafe: true,
        };
        let json = serde_json::to_string(&summary).unwrap();
        let parsed: EffectsSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.stable_id, "test_fn");
        assert!(parsed.may_panic);
        assert!(parsed.is_unsafe);
    }

    #[test]
    fn test_emit_and_parse_effects_json() {
        let mut analyzer = EffectsAnalyzer::new();
        analyzer.record(EffectsSummary {
            stable_id: "test".to_string(),
            effects: vec![],
            may_diverge: false,
            may_panic: false,
            may_allocate: false,
            may_read: true,
            may_write: false,
            is_unsafe: false,
        });

        let json = emit_effects_json(&analyzer).unwrap();
        let parsed = parse_effects_json(&json).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].stable_id, "test");
    }

    #[test]
    fn test_is_pure_with_all_effects() {
        let mut analyzer = EffectsAnalyzer::new();
        analyzer.record(EffectsSummary {
            stable_id: "impure_fn".to_string(),
            effects: vec![],
            may_diverge: false,
            may_panic: false,
            may_allocate: true,
            may_read: true,
            may_write: true,
            is_unsafe: false,
        });
        assert!(!analyzer.is_pure("impure_fn"));
    }

    #[test]
    fn test_find_nonexistent() {
        let analyzer = EffectsAnalyzer::new();
        assert!(analyzer.find("nonexistent").is_none());
    }

    #[test]
    fn test_effect_kind_all_variants() {
        assert!(matches!(EffectKind::Read, EffectKind::Read));
        assert!(matches!(EffectKind::Write, EffectKind::Write));
        assert!(matches!(EffectKind::Allocate, EffectKind::Allocate));
        assert!(matches!(EffectKind::Deallocate, EffectKind::Deallocate));
        assert!(matches!(EffectKind::Panic, EffectKind::Panic));
        assert!(matches!(EffectKind::Diverges, EffectKind::Diverges));
        assert!(matches!(EffectKind::SystemCall, EffectKind::SystemCall));
        assert!(matches!(EffectKind::FileIO, EffectKind::FileIO));
        assert!(matches!(EffectKind::NetworkIO, EffectKind::NetworkIO));
        assert!(matches!(EffectKind::ThreadSpawn, EffectKind::ThreadSpawn));
        assert!(matches!(EffectKind::Lock, EffectKind::Lock));
        assert!(matches!(EffectKind::Unlock, EffectKind::Unlock));
    }

    #[test]
    fn test_effects_analyzer_len() {
        let mut analyzer = EffectsAnalyzer::new();
        assert_eq!(analyzer.summaries().len(), 0);

        analyzer.record(EffectsSummary {
            stable_id: "fn1".to_string(),
            effects: vec![],
            may_diverge: false,
            may_panic: false,
            may_allocate: false,
            may_read: false,
            may_write: false,
            is_unsafe: false,
        });
        assert_eq!(analyzer.summaries().len(), 1);
    }

    #[test]
    fn test_effects_analyzer_default() {
        let analyzer = EffectsAnalyzer::default();
        assert!(analyzer.summaries().is_empty());
    }
}
