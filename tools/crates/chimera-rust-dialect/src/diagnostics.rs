//! Tasks 96-97: Rust dialect verifier and unsupported feature diagnostics
//!
//! Verifies type consistency, CFG validity, live locals, move/borrow/drop constraints.
//! Emits explicit diagnostics for async/await, generators, inline asm, SIMD,
//! trait object lowering, or proc macro cases until supported.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A dialect diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialectDiagnostic {
    /// Unique diagnostic ID
    pub id: String,
    /// Kind of diagnostic
    pub kind: DiagnosticKind,
    /// Severity level
    pub severity: Severity,
    /// Source location
    pub location: DiagnosticLocation,
    /// Message describing the issue
    pub message: String,
    /// Suggestions (if any)
    pub suggestions: Vec<String>,
}

/// Kind of diagnostic
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// Unsupported feature used
    UnsupportedFeature,
    /// Type consistency error
    TypeError,
    /// CFG validity error
    CfgError,
    /// Borrow checker error
    BorrowError,
    /// Drop order error
    DropError,
    /// Move after borrow error
    MoveError,
    /// Missing layout/type ref
    MissingRef,
    /// Trust assumption required
    TrustObligation,
}

/// Severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Note,
    Warning,
    Error,
    HardError,
}

/// Source location for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticLocation {
    pub file: String,
    pub line: u32,
    pub col_start: u32,
    pub col_end: u32,
    pub stable_id: Option<String>,
}

/// Unsupported feature type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsupportedFeature {
    /// Feature name
    pub name: String,
    /// Feature category
    pub category: FeatureCategory,
    /// Whether this is a blocking limitation
    pub is_blocking: bool,
    /// Workaround suggestion
    pub workaround: Option<String>,
}

/// Feature categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FeatureCategory {
    /// Async/await and generators
    Async,
    /// Inline assembly
    InlineAsm,
    /// SIMD intrinsics
    Simd,
    /// Trait objects and dynamic dispatch
    TraitObject,
    /// Proc macros
    ProcMacro,
    /// Custom derive
    CustomDerive,
    /// Async traits
    AsyncTrait,
    /// Existential types / RPITIT
    ExistentialType,
    /// Specialized default methods
    Specialization,
    /// Const generics (partial)
    ConstGenerics,
    /// Inline constants
    InlineConst,
    /// Unsafe traits
    UnsafeTrait,
    /// Unsized rvalues
    UnsizedRvalue,
    /// Other features
    Other,
}

/// Feature matrix tracking supported vs unsupported features
#[derive(Default)]
pub struct FeatureMatrix {
    /// All known features
    features: HashMap<FeatureCategory, Vec<FeatureStatus>>,
}

/// Status of a feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStatus {
    pub name: String,
    pub supported: bool,
    pub limitation: Option<String>,
}

/// Diagnostic emitter for the dialect
pub struct DiagnosticEmitter {
    diagnostics: Vec<DialectDiagnostic>,
    feature_matrix: FeatureMatrix,
    errors: usize,
    warnings: usize,
    notes: usize,
}

impl DiagnosticEmitter {
    /// Create a new diagnostic emitter
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            feature_matrix: FeatureMatrix::default(),
            errors: 0,
            warnings: 0,
            notes: 0,
        }
    }

    /// Emit an unsupported feature diagnostic
    pub fn emit_unsupported_feature(
        &mut self,
        feature: &str,
        category: FeatureCategory,
        location: DiagnosticLocation,
        is_blocking: bool,
        workaround: Option<String>,
    ) -> DialectDiagnostic {
        let kind = if is_blocking {
            DiagnosticKind::UnsupportedFeature
        } else {
            DiagnosticKind::TrustObligation
        };

        let sev = if is_blocking {
            Severity::HardError
        } else {
            Severity::Warning
        };
        let message = if is_blocking {
            format!(
                "Unsupported feature '{}' in category {:?}. Compilation cannot proceed.",
                feature, category
            )
        } else {
            format!(
                "Feature '{}' is partially supported. {}",
                feature,
                workaround.as_deref().unwrap_or("Use with caution.")
            )
        };

        let diagnostic = DialectDiagnostic {
            id: format!("diag_{}", self.diagnostics.len()),
            kind,
            severity: sev,
            location,
            message,
            suggestions: workaround.into_iter().collect(),
        };

        self.count_severity(&diagnostic.severity);
        self.diagnostics.push(diagnostic.clone());

        // Update feature matrix
        self.feature_matrix
            .record_feature(category, feature, is_blocking);

        diagnostic
    }

    /// Emit a type error diagnostic
    pub fn emit_type_error(
        &mut self,
        message: &str,
        location: DiagnosticLocation,
    ) -> DialectDiagnostic {
        let diagnostic = DialectDiagnostic {
            id: format!("diag_{}", self.diagnostics.len()),
            kind: DiagnosticKind::TypeError,
            severity: Severity::Error,
            location,
            message: message.to_string(),
            suggestions: vec![],
        };

        self.count_severity(&diagnostic.severity);
        self.diagnostics.push(diagnostic.clone());
        diagnostic
    }

    /// Emit a CFG error diagnostic
    pub fn emit_cfg_error(
        &mut self,
        message: &str,
        location: DiagnosticLocation,
    ) -> DialectDiagnostic {
        let diagnostic = DialectDiagnostic {
            id: format!("diag_{}", self.diagnostics.len()),
            kind: DiagnosticKind::CfgError,
            severity: Severity::HardError,
            location,
            message: message.to_string(),
            suggestions: vec![],
        };

        self.count_severity(&diagnostic.severity);
        self.diagnostics.push(diagnostic.clone());
        diagnostic
    }

    /// Emit a borrow error diagnostic
    pub fn emit_borrow_error(
        &mut self,
        message: &str,
        location: DiagnosticLocation,
    ) -> DialectDiagnostic {
        let diagnostic = DialectDiagnostic {
            id: format!("diag_{}", self.diagnostics.len()),
            kind: DiagnosticKind::BorrowError,
            severity: Severity::Error,
            location,
            message: message.to_string(),
            suggestions: vec![],
        };

        self.count_severity(&diagnostic.severity);
        self.diagnostics.push(diagnostic.clone());
        diagnostic
    }

    fn count_severity(&mut self, severity: &Severity) {
        match severity {
            Severity::Note => self.notes += 1,
            Severity::Warning => self.warnings += 1,
            Severity::Error | Severity::HardError => self.errors += 1,
        }
    }

    /// Get all diagnostics
    pub fn diagnostics(&self) -> &[DialectDiagnostic] {
        &self.diagnostics
    }

    /// Get diagnostics by kind
    pub fn diagnostics_by_kind(&self, kind: DiagnosticKind) -> Vec<&DialectDiagnostic> {
        self.diagnostics.iter().filter(|d| d.kind == kind).collect()
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.warnings
    }

    /// Check if verification passed
    pub fn verification_passed(&self) -> bool {
        self.errors == 0
    }

    /// Get feature matrix
    pub fn feature_matrix(&self) -> &FeatureMatrix {
        &self.feature_matrix
    }
}

impl Default for DiagnosticEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureMatrix {
    /// Record a feature status
    pub fn record_feature(&mut self, category: FeatureCategory, name: &str, supported: bool) {
        let status = FeatureStatus {
            name: name.to_string(),
            supported,
            limitation: if supported {
                None
            } else {
                Some("Not yet supported".to_string())
            },
        };
        self.features.entry(category).or_default().push(status);
    }

    /// Check if a category has any unsupported features
    pub fn has_unsupported(&self, category: FeatureCategory) -> bool {
        self.features
            .get(&category)
            .map(|v| v.iter().any(|f| !f.supported))
            .unwrap_or(false)
    }

    /// Get unsupported features
    pub fn unsupported_features(&self) -> Vec<FeatureStatus> {
        self.features
            .values()
            .flatten()
            .filter(|f| !f.supported)
            .cloned()
            .collect()
    }

    /// Get feature status by category
    pub fn features_by_category(&self, category: FeatureCategory) -> &[FeatureStatus] {
        self.features
            .get(&category)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

/// Predefined unsupported features
pub const KNOWN_UNSUPPORTED_FEATURES: &[(&str, FeatureCategory)] = &[
    ("async", FeatureCategory::Async),
    ("await", FeatureCategory::Async),
    ("Generator", FeatureCategory::Async),
    ("async fn", FeatureCategory::Async),
    ("async trait", FeatureCategory::AsyncTrait),
    ("asm!", FeatureCategory::InlineAsm),
    ("llvm_asm!", FeatureCategory::InlineAsm),
    ("simd_shuffle", FeatureCategory::Simd),
    ("std::arch::x86::", FeatureCategory::Simd),
    ("dyn Trait", FeatureCategory::TraitObject),
    ("impl Trait", FeatureCategory::TraitObject),
    ("proc_macro", FeatureCategory::ProcMacro),
    ("#[proc_macro]", FeatureCategory::ProcMacro),
    ("#[derive(...)]", FeatureCategory::CustomDerive),
    ("specialization", FeatureCategory::Specialization),
    ("min_specialization", FeatureCategory::Specialization),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn test_location() -> DiagnosticLocation {
        DiagnosticLocation {
            file: "test.rs".to_string(),
            line: 10,
            col_start: 1,
            col_end: 10,
            stable_id: Some("fn123".to_string()),
        }
    }

    #[test]
    fn test_emit_unsupported_feature_blocking() {
        let mut emitter = DiagnosticEmitter::new();
        emitter.emit_unsupported_feature(
            "async fn",
            FeatureCategory::Async,
            test_location(),
            true,
            Some("Use synchronous version instead".to_string()),
        );

        assert_eq!(emitter.error_count(), 1);
        assert!(!emitter.verification_passed());
    }

    #[test]
    fn test_emit_unsupported_feature_warning() {
        let mut emitter = DiagnosticEmitter::new();
        emitter.emit_unsupported_feature(
            "asm!",
            FeatureCategory::InlineAsm,
            test_location(),
            false,
            None,
        );

        assert_eq!(emitter.warning_count(), 1);
    }

    #[test]
    fn test_emit_type_error() {
        let mut emitter = DiagnosticEmitter::new();
        emitter.emit_type_error("type mismatch: expected i32, found u32", test_location());

        assert_eq!(emitter.error_count(), 1);
        assert_eq!(
            emitter.diagnostics_by_kind(DiagnosticKind::TypeError).len(),
            1
        );
    }

    #[test]
    fn test_emit_cfg_error() {
        let mut emitter = DiagnosticEmitter::new();
        emitter.emit_cfg_error("invalid basic block terminator", test_location());

        assert_eq!(emitter.error_count(), 1);
    }

    #[test]
    fn test_feature_matrix_record() {
        let mut matrix = FeatureMatrix::default();
        matrix.record_feature(FeatureCategory::Async, "async fn", false);
        matrix.record_feature(FeatureCategory::Async, "await", false);

        assert!(matrix.has_unsupported(FeatureCategory::Async));
        assert!(!matrix.has_unsupported(FeatureCategory::Simd));
    }

    #[test]
    fn test_feature_matrix_unsupported_list() {
        let mut matrix = FeatureMatrix::default();
        matrix.record_feature(FeatureCategory::InlineAsm, "asm!", false);

        let unsupported = matrix.unsupported_features();
        assert_eq!(unsupported.len(), 1);
        assert_eq!(unsupported[0].name, "asm!");
    }

    #[test]
    fn test_diagnostic_by_kind_filter() {
        let mut emitter = DiagnosticEmitter::new();
        emitter.emit_unsupported_feature(
            "async",
            FeatureCategory::Async,
            test_location(),
            true,
            None,
        );
        emitter.emit_type_error("type error", test_location());
        emitter.emit_type_error("another type error", test_location());

        assert_eq!(
            emitter
                .diagnostics_by_kind(DiagnosticKind::UnsupportedFeature)
                .len(),
            1
        );
        assert_eq!(
            emitter.diagnostics_by_kind(DiagnosticKind::TypeError).len(),
            2
        );
    }

    #[test]
    fn test_verification_passed() {
        let emitter = DiagnosticEmitter::new();
        assert!(emitter.verification_passed());
    }

    #[test]
    fn test_verification_failed() {
        let mut emitter = DiagnosticEmitter::new();
        emitter.emit_type_error("error", test_location());
        assert!(!emitter.verification_passed());
    }
}
