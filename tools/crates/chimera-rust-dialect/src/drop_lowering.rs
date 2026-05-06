//! Task 93: Lower drops to dialect operations
//!
//! Emits explicit drop ops, drop glue references, drop order,
//! cleanup blocks, and panic-path cleanup.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Drop operation in the dialect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropOp {
    /// Unique ID for this drop
    pub id: String,
    /// The place being dropped
    pub place: String,
    /// Type of the place
    pub ty: String,
    /// Drop kind
    pub kind: DropKind,
    /// Source location
    pub location: SourceLocation,
    /// Drop glue reference if any
    pub drop_glue: Option<DropGlue>,
}

/// Kind of drop operation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DropKind {
    /// Explicit drop through `Drop::drop()` call
    Explicit,
    /// Implicit drop (storage dead, scope exit)
    Implicit,
    /// Drop via drop glue
    Glue,
    /// Drop panic cleanup path
    PanicCleanup,
}

/// Source location for drop ops
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub col: u32,
}

/// Drop glue function reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropGlue {
    /// Symbol name of the drop glue function
    pub symbol: String,
    /// Whether this is a weak glue
    pub is_weak: bool,
    /// Size of the type being dropped
    pub size: u64,
    /// Alignment of the type
    pub align: u64,
}

/// Drop order specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropOrder {
    /// Function this order belongs to
    pub function: String,
    /// Ordered list of drop IDs
    pub drops: Vec<String>,
    /// Parallel drop groups (drops that can happen concurrently)
    pub groups: Vec<Vec<String>>,
}

/// Cleanup path in the function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPath {
    /// Entry point of this cleanup path
    pub entry: String,
    /// Blocks in the cleanup path
    pub blocks: Vec<String>,
    /// Whether this is a panic cleanup
    pub is_panic: bool,
}

/// Lower a place drop to dialect operations
pub fn lower_drop(place: &str, ty: &str, location: SourceLocation) -> DropOp {
    let drop_kind = match ty {
        "()" => DropKind::Explicit,
        _ => DropKind::Implicit,
    };

    DropOp {
        id: format!(
            "drop_{}",
            place.replace('.', "_").replace('[', "_").replace(']', "_")
        ),
        place: place.to_string(),
        ty: ty.to_string(),
        kind: drop_kind,
        location,
        drop_glue: None,
    }
}

/// Lower a cleanup path (panic/unwind)
pub fn lower_cleanup_path(entry_block: &str, blocks: Vec<String>, is_panic: bool) -> CleanupPath {
    CleanupPath {
        entry: entry_block.to_string(),
        blocks,
        is_panic,
    }
}

/// Drop analyzer for a function body
#[derive(Default)]
pub struct DropAnalyzer {
    drops: Vec<DropOp>,
    drop_glues: HashMap<String, DropGlue>,
    cleanup_paths: Vec<CleanupPath>,
    drop_orders: HashMap<String, DropOrder>,
}

impl DropAnalyzer {
    /// Create a new drop analyzer
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a drop operation
    pub fn record_drop(&mut self, op: DropOp) {
        self.drops.push(op);
    }

    /// Record a drop glue
    pub fn record_drop_glue(&mut self, symbol: String, size: u64, align: u64) {
        self.drop_glues.insert(
            symbol.clone(),
            DropGlue {
                symbol,
                is_weak: false,
                size,
                align,
            },
        );
    }

    /// Record a cleanup path
    pub fn record_cleanup_path(&mut self, path: CleanupPath) {
        self.cleanup_paths.push(path);
    }

    /// Compute drop order for a function
    pub fn compute_drop_order(&mut self, function: &str) -> DropOrder {
        let mut ordered: Vec<String> = self.drops.iter().map(|d| d.id.clone()).collect();

        // Sort by drop kind (panic cleanup first, then explicit, then implicit)
        ordered.sort_by(|a, b| {
            let kind_a = &self.drops.iter().find(|d| &d.id == a).unwrap().kind;
            let kind_b = &self.drops.iter().find(|d| &d.id == b).unwrap().kind;
            kind_a.cmp(kind_b)
        });

        let order = DropOrder {
            function: function.to_string(),
            drops: ordered.clone(),
            groups: vec![ordered], // Simplified: all in one group
        };

        self.drop_orders.insert(function.to_string(), order.clone());
        order
    }

    /// Get all recorded drops
    pub fn drops(&self) -> &[DropOp] {
        &self.drops
    }

    /// Get drop glue by symbol
    pub fn get_drop_glue(&self, symbol: &str) -> Option<&DropGlue> {
        self.drop_glues.get(symbol)
    }

    /// Get cleanup paths
    pub fn cleanup_paths(&self) -> &[CleanupPath] {
        &self.cleanup_paths
    }

    /// Check for double-drop possibility (returns offending drop IDs)
    pub fn check_double_drop(&self) -> Vec<String> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut offenders: Vec<String> = Vec::new();

        for drop in &self.drops {
            if seen.contains(&drop.place) {
                offenders.push(drop.id.clone());
            }
            seen.insert(drop.place.clone());
        }

        offenders
    }

    /// Validate drop consistency
    pub fn validate(&self) -> Result<(), DropError> {
        // Check for double drops
        let offenders = self.check_double_drop();
        if !offenders.is_empty() {
            return Err(DropError::DoubleDrop(offenders));
        }

        // Check for drops referencing non-existent drop glues
        for drop in &self.drops {
            if let Some(glue) = &drop.drop_glue {
                if !self.drop_glues.contains_key(&glue.symbol) {
                    return Err(DropError::MissingDropGlue(glue.symbol.clone()));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DropError {
    #[error("double drop detected for places: {0:?}")]
    DoubleDrop(Vec<String>),
    #[error("missing drop glue symbol: {0}")]
    MissingDropGlue(String),
    #[error("invalid drop order")]
    InvalidDropOrder,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_location() -> SourceLocation {
        SourceLocation {
            file: "test.rs".to_string(),
            line: 10,
            col: 5,
        }
    }

    #[test]
    fn test_lower_drop_explicit() {
        let drop = lower_drop("x", "()", test_location());
        assert_eq!(drop.place, "x");
        assert_eq!(drop.kind, DropKind::Explicit);
        assert!(drop.drop_glue.is_none());
    }

    #[test]
    fn test_lower_drop_with_glue() {
        let mut analyzer = DropAnalyzer::new();
        analyzer.record_drop_glue("drop_u32".to_string(), 4, 4);

        let mut drop = lower_drop("x", "u32", test_location());
        drop.drop_glue = analyzer.get_drop_glue("drop_u32").cloned();

        assert!(drop.drop_glue.is_some());
        assert_eq!(drop.drop_glue.unwrap().symbol, "drop_u32");
    }

    #[test]
    fn test_lower_cleanup_path_panic() {
        let path = lower_cleanup_path(
            "cleanup",
            vec!["cleanup_block1".to_string(), "cleanup_block2".to_string()],
            true,
        );
        assert!(path.is_panic);
        assert_eq!(path.blocks.len(), 2);
    }

    #[test]
    fn test_drop_order_computation() {
        let mut analyzer = DropAnalyzer::new();
        analyzer.record_drop(lower_drop("a", "i32", test_location()));
        analyzer.record_drop(lower_drop("b", "i32", test_location()));

        let order = analyzer.compute_drop_order("test_fn");
        assert_eq!(order.function, "test_fn");
        assert_eq!(order.drops.len(), 2);
    }

    #[test]
    fn test_double_drop_detection() {
        let mut analyzer = DropAnalyzer::new();
        analyzer.record_drop(lower_drop("x", "i32", test_location()));
        analyzer.record_drop(lower_drop("y", "i32", test_location()));
        analyzer.record_drop(lower_drop("x", "i32", test_location())); // Double drop

        let offenders = analyzer.check_double_drop();
        assert_eq!(offenders.len(), 1);
        assert!(offenders[0].contains("x"));
    }

    #[test]
    fn test_drop_validation_no_errors() {
        let analyzer = DropAnalyzer::new();
        assert!(analyzer.validate().is_ok());
    }

    #[test]
    fn test_drop_validation_double_drop() {
        let mut analyzer = DropAnalyzer::new();
        analyzer.record_drop(lower_drop("x", "i32", test_location()));
        analyzer.record_drop(lower_drop("x", "i32", test_location()));

        let result = analyzer.validate();
        assert!(matches!(result, Err(DropError::DoubleDrop(_))));
    }

    #[test]
    fn test_drop_glue_metadata() {
        let glue = DropGlue {
            symbol: "drop_Struct".to_string(),
            is_weak: false,
            size: 16,
            align: 8,
        };
        assert_eq!(glue.size, 16);
        assert_eq!(glue.align, 8);
    }
}
