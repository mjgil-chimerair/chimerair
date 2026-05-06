//! Chimera Rust Proof Obligation Generation
//!
//! Generates proof obligations for Rust code that requires verification:
//! - Memory safety
//! - Ownership and borrowing
//! - Undefined behavior prevention
//! - Panic safety
//! - FFI safety

use serde::{Deserialize, Serialize};

/// Proof obligation kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObligationKind {
    /// Memory allocation bounds
    AllocationBounds,
    /// Pointer dereference validity
    PointerDeref,
    /// Array index bounds
    IndexBounds,
    /// Division by zero
    DivisionByZero,
    /// Null pointer check
    NullCheck,
    /// Alignment requirement
    Alignment,
    /// Drop ordering
    DropOrder,
    /// Mutex safety
    MutexSafety,
    /// FFI safety
    FfiSafety,
    /// Unwind safety
    UnwindSafety,
    /// Undefined behavior prevention
    UndefinedBehavior,
}

/// Trust assumption for external code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Fully trusted - no verification needed
    Trusted,
    /// Partially trusted - some obligations must hold
    Partial,
    /// Untrusted - must be verified
    Untrusted,
}

/// A proof obligation that must be satisfied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofObligation {
    pub id: String,
    pub kind: ObligationKind,
    pub description: String,
    pub location: ObligationLocation,
    pub trust_level: TrustLevel,
    pub dependencies: Vec<String>,
    pub status: ObligationStatus,
}

/// Location of an obligation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObligationLocation {
    pub stable_id: String,
    pub span_start: u32,
    pub span_end: u32,
    pub source_file: String,
}

/// Status of a proof obligation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObligationStatus {
    /// Obligation must be proven
    Unproven,
    /// Obligation is provable
    Provable,
    /// Obligation requires manual proof
    ManualProofRequired,
    /// Obligation is a trust assumption
    TrustedAssumption,
}

/// Proof obligation generator
pub struct ObligationGenerator {
    obligations: Vec<ProofObligation>,
    next_id: u64,
}

impl ObligationGenerator {
    /// Create a new generator
    pub fn new() -> Self {
        Self {
            obligations: Vec::new(),
            next_id: 1,
        }
    }

    /// Generate next obligation ID
    fn next_id(&mut self) -> String {
        let id = format!("obl_{}", self.next_id);
        self.next_id += 1;
        id
    }

    /// Add an obligation
    pub fn add(&mut self, obligation: ProofObligation) {
        self.obligations.push(obligation);
    }

    /// Generate pointer deref obligation
    pub fn pointer_deref(&mut self, stable_id: &str, span: (u32, u32), file: &str) {
        let id = self.next_id();
        self.add(ProofObligation {
            id,
            kind: ObligationKind::PointerDeref,
            description: format!(
                "Pointer must be non-null and valid for dereferencing in {}",
                stable_id
            ),
            location: ObligationLocation {
                stable_id: stable_id.to_string(),
                span_start: span.0,
                span_end: span.1,
                source_file: file.to_string(),
            },
            trust_level: TrustLevel::Untrusted,
            dependencies: Vec::new(),
            status: ObligationStatus::Unproven,
        });
    }

    /// Generate bounds check obligation
    pub fn bounds_check(&mut self, stable_id: &str, span: (u32, u32), file: &str) {
        let id = self.next_id();
        self.add(ProofObligation {
            id,
            kind: ObligationKind::IndexBounds,
            description: format!("Index must be within bounds in {}", stable_id),
            location: ObligationLocation {
                stable_id: stable_id.to_string(),
                span_start: span.0,
                span_end: span.1,
                source_file: file.to_string(),
            },
            trust_level: TrustLevel::Untrusted,
            dependencies: Vec::new(),
            status: ObligationStatus::Unproven,
        });
    }

    /// Generate allocation bounds
    pub fn allocation_bounds(&mut self, stable_id: &str, span: (u32, u32), file: &str) {
        let id = self.next_id();
        self.add(ProofObligation {
            id,
            kind: ObligationKind::AllocationBounds,
            description: format!("Allocation size must not exceed maximum in {}", stable_id),
            location: ObligationLocation {
                stable_id: stable_id.to_string(),
                span_start: span.0,
                span_end: span.1,
                source_file: file.to_string(),
            },
            trust_level: TrustLevel::Untrusted,
            dependencies: Vec::new(),
            status: ObligationStatus::Unproven,
        });
    }

    /// Generate FFI safety obligation
    pub fn ffi_safety(&mut self, stable_id: &str, span: (u32, u32), file: &str) {
        let id = self.next_id();
        self.add(ProofObligation {
            id,
            kind: ObligationKind::FfiSafety,
            description: format!(
                "FFI call requires valid memory and ABI conformance in {}",
                stable_id
            ),
            location: ObligationLocation {
                stable_id: stable_id.to_string(),
                span_start: span.0,
                span_end: span.1,
                source_file: file.to_string(),
            },
            trust_level: TrustLevel::Partial,
            dependencies: Vec::new(),
            status: ObligationStatus::ManualProofRequired,
        });
    }

    /// Get all obligations
    pub fn obligations(&self) -> &[ProofObligation] {
        &self.obligations
    }

    /// Count by kind
    pub fn count_by_kind(&self, kind: ObligationKind) -> usize {
        self.obligations.iter().filter(|o| o.kind == kind).count()
    }

    /// Count by status
    pub fn count_by_status(&self, status: ObligationStatus) -> usize {
        self.obligations
            .iter()
            .filter(|o| o.status == status)
            .count()
    }
}

impl Default for ObligationGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Emit obligations to JSON
pub fn emit_obligations_json(gen: &ObligationGenerator) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&gen.obligations())
}

/// Parse obligations from JSON
pub fn parse_obligations_json(json: &str) -> Result<Vec<ProofObligation>, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pointer_deref_obligation() {
        let mut gen = ObligationGenerator::new();
        gen.pointer_deref("test_fn", (10, 25), "test.rs");

        assert_eq!(gen.obligations().len(), 1);
        let obl = &gen.obligations()[0];
        assert_eq!(obl.kind, ObligationKind::PointerDeref);
        assert_eq!(obl.trust_level, TrustLevel::Untrusted);
    }

    #[test]
    fn test_bounds_check_obligation() {
        let mut gen = ObligationGenerator::new();
        gen.bounds_check("index_fn", (30, 45), "test.rs");

        assert_eq!(gen.count_by_kind(ObligationKind::IndexBounds), 1);
    }

    #[test]
    fn test_ffi_safety_obligation() {
        let mut gen = ObligationGenerator::new();
        gen.ffi_safety("extern_fn", (50, 70), "test.rs");

        let obl = &gen.obligations()[0];
        assert_eq!(obl.kind, ObligationKind::FfiSafety);
        assert_eq!(obl.status, ObligationStatus::ManualProofRequired);
    }

    #[test]
    fn test_roundtrip_json() {
        let mut gen = ObligationGenerator::new();
        gen.pointer_deref("test", (0, 10), "test.rs");
        gen.bounds_check("test", (20, 30), "test.rs");

        let json = emit_obligations_json(&gen).unwrap();
        let parsed = parse_obligations_json(&json).unwrap();

        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_id_uniqueness() {
        let mut gen = ObligationGenerator::new();
        for _ in 0..100 {
            gen.pointer_deref("test", (0, 10), "test.rs");
        }

        let ids: Vec<_> = gen.obligations().iter().map(|o| o.id.clone()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn test_all_obligations_kind_count() {
        let mut gen = ObligationGenerator::new();
        gen.pointer_deref("fn1", (0, 10), "a.rs");
        gen.pointer_deref("fn1", (20, 30), "a.rs");
        gen.bounds_check("fn1", (40, 50), "a.rs");
        gen.allocation_bounds("fn2", (60, 80), "b.rs");

        assert_eq!(gen.count_by_kind(ObligationKind::PointerDeref), 2);
        assert_eq!(gen.count_by_kind(ObligationKind::IndexBounds), 1);
        assert_eq!(gen.count_by_kind(ObligationKind::AllocationBounds), 1);
    }

    #[test]
    fn test_obligations_status_count() {
        let mut gen = ObligationGenerator::new();
        gen.pointer_deref("fn1", (0, 10), "a.rs");
        gen.ffi_safety("fn2", (20, 30), "b.rs");

        assert_eq!(gen.count_by_status(ObligationStatus::Unproven), 1);
        assert_eq!(
            gen.count_by_status(ObligationStatus::ManualProofRequired),
            1
        );
    }

    #[test]
    fn test_obligation_kind_serialization() {
        let kinds = vec![
            ObligationKind::AllocationBounds,
            ObligationKind::PointerDeref,
            ObligationKind::IndexBounds,
            ObligationKind::DivisionByZero,
            ObligationKind::NullCheck,
            ObligationKind::Alignment,
            ObligationKind::DropOrder,
            ObligationKind::MutexSafety,
            ObligationKind::FfiSafety,
            ObligationKind::UnwindSafety,
            ObligationKind::UndefinedBehavior,
        ];
        for kind in kinds {
            let json = serde_json::to_string(&kind).unwrap();
            let parsed: ObligationKind = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, kind);
        }
    }

    #[test]
    fn test_trust_level_serialization() {
        let levels = vec![
            TrustLevel::Trusted,
            TrustLevel::Partial,
            TrustLevel::Untrusted,
        ];
        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let parsed: TrustLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, level);
        }
    }

    #[test]
    fn test_obligation_status_serialization() {
        let statuses = vec![
            ObligationStatus::Unproven,
            ObligationStatus::Provable,
            ObligationStatus::ManualProofRequired,
            ObligationStatus::TrustedAssumption,
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let parsed: ObligationStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn test_obligation_location_serialization() {
        let loc = ObligationLocation {
            stable_id: "test_fn".to_string(),
            span_start: 100,
            span_end: 200,
            source_file: "test.rs".to_string(),
        };
        let json = serde_json::to_string(&loc).unwrap();
        let parsed: ObligationLocation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.stable_id, "test_fn");
        assert_eq!(parsed.span_start, 100);
    }

    #[test]
    fn test_proof_obligation_serialization() {
        let obl = ProofObligation {
            id: "test_obl".to_string(),
            kind: ObligationKind::PointerDeref,
            description: "Test obligation".to_string(),
            location: ObligationLocation {
                stable_id: "fn1".to_string(),
                span_start: 10,
                span_end: 20,
                source_file: "test.rs".to_string(),
            },
            trust_level: TrustLevel::Untrusted,
            dependencies: vec!["dep1".to_string()],
            status: ObligationStatus::Unproven,
        };
        let json = serde_json::to_string(&obl).unwrap();
        let parsed: ProofObligation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "test_obl");
        assert_eq!(parsed.kind, ObligationKind::PointerDeref);
    }

    #[test]
    fn test_obligation_generator_default() {
        let gen = ObligationGenerator::default();
        assert!(gen.obligations().is_empty());
    }

    #[test]
    fn test_obligation_generator_len() {
        let mut gen = ObligationGenerator::new();
        assert_eq!(gen.obligations().len(), 0);
        gen.pointer_deref("fn1", (0, 10), "a.rs");
        assert_eq!(gen.obligations().len(), 1);
        gen.bounds_check("fn2", (20, 30), "b.rs");
        assert_eq!(gen.obligations().len(), 2);
    }

    #[test]
    fn test_division_by_zero_obligation() {
        let mut gen = ObligationGenerator::new();
        gen.add(ProofObligation {
            id: "obl_1".to_string(),
            kind: ObligationKind::DivisionByZero,
            description: "Division by zero check".to_string(),
            location: ObligationLocation {
                stable_id: "divide_fn".to_string(),
                span_start: 50,
                span_end: 60,
                source_file: "math.rs".to_string(),
            },
            trust_level: TrustLevel::Untrusted,
            dependencies: vec![],
            status: ObligationStatus::Unproven,
        });

        assert_eq!(gen.count_by_kind(ObligationKind::DivisionByZero), 1);
    }

    #[test]
    fn test_null_check_obligation() {
        let mut gen = ObligationGenerator::new();
        gen.add(ProofObligation {
            id: "obl_2".to_string(),
            kind: ObligationKind::NullCheck,
            description: "Null pointer check".to_string(),
            location: ObligationLocation {
                stable_id: "ptr_fn".to_string(),
                span_start: 70,
                span_end: 80,
                source_file: "ptr.rs".to_string(),
            },
            trust_level: TrustLevel::Partial,
            dependencies: vec![],
            status: ObligationStatus::Provable,
        });

        assert_eq!(gen.count_by_kind(ObligationKind::NullCheck), 1);
    }

    #[test]
    fn test_alignment_obligation() {
        let mut gen = ObligationGenerator::new();
        gen.add(ProofObligation {
            id: "obl_3".to_string(),
            kind: ObligationKind::Alignment,
            description: "Alignment requirement".to_string(),
            location: ObligationLocation {
                stable_id: "align_fn".to_string(),
                span_start: 90,
                span_end: 100,
                source_file: "align.rs".to_string(),
            },
            trust_level: TrustLevel::Trusted,
            dependencies: vec![],
            status: ObligationStatus::TrustedAssumption,
        });

        assert_eq!(gen.count_by_kind(ObligationKind::Alignment), 1);
        assert_eq!(gen.count_by_status(ObligationStatus::TrustedAssumption), 1);
    }
}
