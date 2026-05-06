//! Chimera Rust Ownership Facts
//!
//! Extracts ownership and drop-related facts from Rust code:
//! - Drop flags and elaboration
//! - Drop order
//! - Cleanup paths
//! - Panic cleanup edges
//! - Storage live/dead
//! - Borrow lifetime approximations

use serde::{Deserialize, Serialize};

/// Drop fact kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DropFactKind {
    /// Drop flag (runtime check)
    DropFlag,
    /// Drop order index
    DropOrder,
    /// Cleanup path
    CleanupPath,
    /// Panic cleanup edge
    PanicCleanup,
    /// Storage live marker
    StorageLive,
    /// Storage dead marker
    StorageDead,
    /// Drop impl reference
    DropImpl,
}

/// A drop-related fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropFact {
    pub id: String,
    pub kind: DropFactKind,
    pub stable_id: String,
    pub location: DropLocation,
    pub drop_order: u32,
    pub target: Option<String>,
    pub dependencies: Vec<String>,
}

/// Location of a drop fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropLocation {
    pub span_start: u32,
    pub span_end: u32,
    pub source_file: String,
}

/// Ownership fact kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OwnershipFactKind {
    /// Move operation
    Move,
    /// Borrow (shared)
    BorrowShared,
    /// Borrow (mutable)
    BorrowMutable,
    /// Reborrow
    Reborrow,
    /// Lifetime approximation
    Lifetime,
    /// Region approximation
    Region,
}

/// An ownership fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipFact {
    pub id: String,
    pub kind: OwnershipFactKind,
    pub stable_id: String,
    pub place: String,
    pub location: OwnershipLocation,
    pub lifetime: Option<String>,
    pub region: Option<String>,
}

/// Location of an ownership fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipLocation {
    pub span_start: u32,
    pub span_end: u32,
    pub source_file: String,
}

/// Ownership analyzer
#[derive(Default)]
pub struct OwnershipAnalyzer {
    drop_facts: Vec<DropFact>,
    ownership_facts: Vec<OwnershipFact>,
    next_drop_id: u64,
    next_own_id: u64,
}

impl OwnershipAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {
            drop_facts: Vec::new(),
            ownership_facts: Vec::new(),
            next_drop_id: 1,
            next_own_id: 1,
        }
    }

    /// Generate next drop ID
    fn next_drop_id(&mut self) -> String {
        let id = format!("drop_{}", self.next_drop_id);
        self.next_drop_id += 1;
        id
    }

    /// Generate next ownership ID
    fn next_own_id(&mut self) -> String {
        let id = format!("own_{}", self.next_own_id);
        self.next_own_id += 1;
        id
    }

    // === Drop facts ===

    /// Record a drop flag
    pub fn drop_flag(&mut self, stable_id: &str, span: (u32, u32), file: &str, order: u32) {
        let id = self.next_drop_id();
        let loc = DropLocation {
            span_start: span.0,
            span_end: span.1,
            source_file: file.to_string(),
        };
        self.drop_facts.push(DropFact {
            id,
            kind: DropFactKind::DropFlag,
            stable_id: stable_id.to_string(),
            location: loc,
            drop_order: order,
            target: None,
            dependencies: Vec::new(),
        });
    }

    /// Record drop order
    pub fn drop_order(&mut self, stable_id: &str, span: (u32, u32), file: &str, order: u32) {
        let id = self.next_drop_id();
        let loc = DropLocation {
            span_start: span.0,
            span_end: span.1,
            source_file: file.to_string(),
        };
        self.drop_facts.push(DropFact {
            id,
            kind: DropFactKind::DropOrder,
            stable_id: stable_id.to_string(),
            location: loc,
            drop_order: order,
            target: None,
            dependencies: Vec::new(),
        });
    }

    /// Record cleanup path
    pub fn cleanup_path(&mut self, stable_id: &str, span: (u32, u32), file: &str, target: &str) {
        let id = self.next_drop_id();
        let loc = DropLocation {
            span_start: span.0,
            span_end: span.1,
            source_file: file.to_string(),
        };
        self.drop_facts.push(DropFact {
            id,
            kind: DropFactKind::CleanupPath,
            stable_id: stable_id.to_string(),
            location: loc,
            drop_order: 0,
            target: Some(target.to_string()),
            dependencies: Vec::new(),
        });
    }

    /// Record panic cleanup edge
    pub fn panic_cleanup(&mut self, stable_id: &str, span: (u32, u32), file: &str, target: &str) {
        let id = self.next_drop_id();
        let loc = DropLocation {
            span_start: span.0,
            span_end: span.1,
            source_file: file.to_string(),
        };
        self.drop_facts.push(DropFact {
            id,
            kind: DropFactKind::PanicCleanup,
            stable_id: stable_id.to_string(),
            location: loc,
            drop_order: 0,
            target: Some(target.to_string()),
            dependencies: Vec::new(),
        });
    }

    /// Record storage live
    pub fn storage_live(&mut self, stable_id: &str, span: (u32, u32), file: &str, place: &str) {
        let id = self.next_drop_id();
        let loc = DropLocation {
            span_start: span.0,
            span_end: span.1,
            source_file: file.to_string(),
        };
        self.drop_facts.push(DropFact {
            id,
            kind: DropFactKind::StorageLive,
            stable_id: stable_id.to_string(),
            location: loc,
            drop_order: 0,
            target: Some(place.to_string()),
            dependencies: Vec::new(),
        });
    }

    /// Record storage dead
    pub fn storage_dead(&mut self, stable_id: &str, span: (u32, u32), file: &str, place: &str) {
        let id = self.next_drop_id();
        let loc = DropLocation {
            span_start: span.0,
            span_end: span.1,
            source_file: file.to_string(),
        };
        self.drop_facts.push(DropFact {
            id,
            kind: DropFactKind::StorageDead,
            stable_id: stable_id.to_string(),
            location: loc,
            drop_order: 0,
            target: Some(place.to_string()),
            dependencies: Vec::new(),
        });
    }

    // === Ownership facts ===

    /// Record a move
    pub fn move_place(&mut self, stable_id: &str, span: (u32, u32), file: &str, place: &str) {
        let id = self.next_own_id();
        let loc = OwnershipLocation {
            span_start: span.0,
            span_end: span.1,
            source_file: file.to_string(),
        };
        self.ownership_facts.push(OwnershipFact {
            id,
            kind: OwnershipFactKind::Move,
            stable_id: stable_id.to_string(),
            place: place.to_string(),
            location: loc,
            lifetime: None,
            region: None,
        });
    }

    /// Record a shared borrow
    pub fn borrow_shared(
        &mut self,
        stable_id: &str,
        span: (u32, u32),
        file: &str,
        place: &str,
        lifetime: &str,
    ) {
        let id = self.next_own_id();
        let loc = OwnershipLocation {
            span_start: span.0,
            span_end: span.1,
            source_file: file.to_string(),
        };
        self.ownership_facts.push(OwnershipFact {
            id,
            kind: OwnershipFactKind::BorrowShared,
            stable_id: stable_id.to_string(),
            place: place.to_string(),
            location: loc,
            lifetime: Some(lifetime.to_string()),
            region: None,
        });
    }

    /// Record a mutable borrow
    pub fn borrow_mutable(
        &mut self,
        stable_id: &str,
        span: (u32, u32),
        file: &str,
        place: &str,
        lifetime: &str,
    ) {
        let id = self.next_own_id();
        let loc = OwnershipLocation {
            span_start: span.0,
            span_end: span.1,
            source_file: file.to_string(),
        };
        self.ownership_facts.push(OwnershipFact {
            id,
            kind: OwnershipFactKind::BorrowMutable,
            stable_id: stable_id.to_string(),
            place: place.to_string(),
            location: loc,
            lifetime: Some(lifetime.to_string()),
            region: None,
        });
    }

    // === Getters ===

    /// Get all drop facts
    pub fn drop_facts(&self) -> &[DropFact] {
        &self.drop_facts
    }

    /// Get all ownership facts
    pub fn ownership_facts(&self) -> &[OwnershipFact] {
        &self.ownership_facts
    }

    /// Count drops for a function
    pub fn drop_count(&self, stable_id: &str) -> usize {
        self.drop_facts
            .iter()
            .filter(|f| f.stable_id == stable_id)
            .count()
    }
}

/// Emit facts to JSON
pub fn emit_ownership_json(analyzer: &OwnershipAnalyzer) -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct Output<'a> {
        drop_facts: &'a [DropFact],
        ownership_facts: &'a [OwnershipFact],
    }
    serde_json::to_string_pretty(&Output {
        drop_facts: analyzer.drop_facts(),
        ownership_facts: analyzer.ownership_facts(),
    })
}

/// Parse facts from JSON
pub fn parse_ownership_json(
    json: &str,
) -> Result<(Vec<DropFact>, Vec<OwnershipFact>), serde_json::Error> {
    #[derive(Deserialize)]
    struct Input {
        drop_facts: Vec<DropFact>,
        ownership_facts: Vec<OwnershipFact>,
    }
    let input: Input = serde_json::from_str(json)?;
    Ok((input.drop_facts, input.ownership_facts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drop_flag() {
        let mut analyzer = OwnershipAnalyzer::new();
        analyzer.drop_flag("test_fn", (10, 20), "test.rs", 1);

        assert_eq!(analyzer.drop_facts().len(), 1);
        assert_eq!(analyzer.drop_facts()[0].kind, DropFactKind::DropFlag);
    }

    #[test]
    fn test_panic_cleanup() {
        let mut analyzer = OwnershipAnalyzer::new();
        analyzer.panic_cleanup("test_fn", (30, 40), "test.rs", "cleanup_target");

        let fact = &analyzer.drop_facts()[0];
        assert_eq!(fact.kind, DropFactKind::PanicCleanup);
        assert_eq!(fact.target, Some("cleanup_target".to_string()));
    }

    #[test]
    fn test_move_operation() {
        let mut analyzer = OwnershipAnalyzer::new();
        analyzer.move_place("test_fn", (50, 60), "test.rs", "x");

        assert_eq!(analyzer.ownership_facts().len(), 1);
        assert_eq!(analyzer.ownership_facts()[0].kind, OwnershipFactKind::Move);
    }

    #[test]
    fn test_borrow_mutable() {
        let mut analyzer = OwnershipAnalyzer::new();
        analyzer.borrow_mutable("test_fn", (70, 80), "test.rs", "x", "'a");

        let fact = &analyzer.ownership_facts()[0];
        assert_eq!(fact.kind, OwnershipFactKind::BorrowMutable);
        assert_eq!(fact.lifetime, Some("'a".to_string()));
    }

    #[test]
    fn test_roundtrip_json() {
        let mut analyzer = OwnershipAnalyzer::new();
        analyzer.drop_flag("test_fn", (10, 20), "test.rs", 1);
        analyzer.move_place("test_fn", (30, 40), "test.rs", "x");

        let json = emit_ownership_json(&analyzer).unwrap();
        let (drops, owns) = parse_ownership_json(&json).unwrap();

        assert_eq!(drops.len(), 1);
        assert_eq!(owns.len(), 1);
    }

    #[test]
    fn test_drop_count() {
        let mut analyzer = OwnershipAnalyzer::new();
        analyzer.drop_flag("fn1", (10, 20), "test.rs", 1);
        analyzer.drop_flag("fn1", (30, 40), "test.rs", 2);
        analyzer.drop_flag("fn2", (50, 60), "test.rs", 1);

        assert_eq!(analyzer.drop_count("fn1"), 2);
        assert_eq!(analyzer.drop_count("fn2"), 1);
    }

    #[test]
    fn test_reborrow() {
        let mut analyzer = OwnershipAnalyzer::new();
        analyzer.borrow_shared("test_fn", (10, 20), "test.rs", "x", "'a");
        analyzer.borrow_mutable("test_fn", (30, 40), "test.rs", "y", "'b");

        assert_eq!(analyzer.ownership_facts().len(), 2);
        assert_eq!(
            analyzer.ownership_facts()[0].kind,
            OwnershipFactKind::BorrowShared
        );
        assert_eq!(
            analyzer.ownership_facts()[1].kind,
            OwnershipFactKind::BorrowMutable
        );
    }

    #[test]
    fn test_storage_live_dead() {
        let mut analyzer = OwnershipAnalyzer::new();
        analyzer.storage_live("test_fn", (10, 20), "test.rs", "x");
        analyzer.storage_dead("test_fn", (30, 40), "test.rs", "x");

        assert_eq!(analyzer.drop_facts().len(), 2);
        assert_eq!(analyzer.drop_facts()[0].kind, DropFactKind::StorageLive);
        assert_eq!(analyzer.drop_facts()[1].kind, DropFactKind::StorageDead);
    }

    #[test]
    fn test_cleanup_path() {
        let mut analyzer = OwnershipAnalyzer::new();
        analyzer.cleanup_path("test_fn", (100, 110), "test.rs", "drop_target");

        let fact = &analyzer.drop_facts()[0];
        assert_eq!(fact.kind, DropFactKind::CleanupPath);
        assert_eq!(fact.target, Some("drop_target".to_string()));
    }

    #[test]
    fn test_drop_order() {
        let mut analyzer = OwnershipAnalyzer::new();
        analyzer.drop_order("fn1", (10, 20), "mod.rs", 1);
        analyzer.drop_order("fn1", (30, 40), "mod.rs", 2);
        analyzer.drop_order("fn2", (50, 60), "mod.rs", 1);

        assert_eq!(analyzer.drop_count("fn1"), 2);
        assert_eq!(analyzer.drop_count("fn2"), 1);
    }

    #[test]
    fn test_ownership_fact_kind_serialization() {
        let kinds = vec![
            OwnershipFactKind::Move,
            OwnershipFactKind::BorrowShared,
            OwnershipFactKind::BorrowMutable,
            OwnershipFactKind::Reborrow,
            OwnershipFactKind::Lifetime,
            OwnershipFactKind::Region,
        ];
        for kind in kinds {
            let json = serde_json::to_string(&kind).unwrap();
            let parsed: OwnershipFactKind = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, kind);
        }
    }

    #[test]
    fn test_drop_fact_kind_serialization() {
        let kinds = vec![
            DropFactKind::DropFlag,
            DropFactKind::DropOrder,
            DropFactKind::CleanupPath,
            DropFactKind::PanicCleanup,
            DropFactKind::StorageLive,
            DropFactKind::StorageDead,
            DropFactKind::DropImpl,
        ];
        for kind in kinds {
            let json = serde_json::to_string(&kind).unwrap();
            let parsed: DropFactKind = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, kind);
        }
    }

    #[test]
    fn test_ownership_location() {
        let loc = OwnershipLocation {
            span_start: 100,
            span_end: 200,
            source_file: "main.rs".to_string(),
        };
        let json = serde_json::to_string(&loc).unwrap();
        let parsed: OwnershipLocation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.span_start, 100);
        assert_eq!(parsed.source_file, "main.rs");
    }

    #[test]
    fn test_drop_location() {
        let loc = DropLocation {
            span_start: 50,
            span_end: 75,
            source_file: "lib.rs".to_string(),
        };
        let json = serde_json::to_string(&loc).unwrap();
        let parsed: DropLocation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.span_end, 75);
    }

    #[test]
    fn test_ownership_analyzer_default() {
        let analyzer = OwnershipAnalyzer::default();
        assert!(analyzer.drop_facts().is_empty());
        assert!(analyzer.ownership_facts().is_empty());
    }

    #[test]
    fn test_borrow_shared() {
        let mut analyzer = OwnershipAnalyzer::new();
        analyzer.borrow_shared("test_fn", (70, 80), "test.rs", "y", "'b");

        let fact = &analyzer.ownership_facts()[0];
        assert_eq!(fact.kind, OwnershipFactKind::BorrowShared);
        assert_eq!(fact.place, "y");
        assert_eq!(fact.lifetime, Some("'b".to_string()));
    }

    #[test]
    fn test_emit_ownership_json_empty() {
        let analyzer = OwnershipAnalyzer::new();
        let json = emit_ownership_json(&analyzer).unwrap();
        let (drops, owns) = parse_ownership_json(&json).unwrap();
        assert!(drops.is_empty());
        assert!(owns.is_empty());
    }

    #[test]
    fn test_drop_fact_serialization() {
        let fact = DropFact {
            id: "test_drop".to_string(),
            kind: DropFactKind::DropFlag,
            stable_id: "fn1".to_string(),
            location: DropLocation {
                span_start: 10,
                span_end: 20,
                source_file: "test.rs".to_string(),
            },
            drop_order: 5,
            target: None,
            dependencies: vec![],
        };
        let json = serde_json::to_string(&fact).unwrap();
        let parsed: DropFact = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "test_drop");
        assert_eq!(parsed.drop_order, 5);
    }
}
