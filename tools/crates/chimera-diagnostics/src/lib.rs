//! Chimera diagnostics crate
//!
//! Centralizes diagnostics rendering, spans, codes, and machine-readable output.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Note,
    Warning,
    Error,
    Fatal,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Note => write!(f, "note"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
            Severity::Fatal => write!(f, "fatal"),
        }
    }
}

/// Diagnostic code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Code {
    // Parser (1000-1999)
    ParseUnknownType,
    ParseMalformedFunctionType,
    ParseInvalidLifetime,

    // Type (2000-2999)
    TypeMismatch,
    TypeNotBorrowable,
    TypeNotOwned,
    TypeInvalidResult,
    TypeInvalidSlice,

    // Ownership (3000-3999)
    OwnershipDoubleBorrow,
    OwnershipUseAfterMove,
    OwnershipIllegalEscape,
    OwnershipDanglingReference,
    OwnershipBorrowExclusivity,

    // Memory (4000-4999)
    MemoryInvalidAlloc,
    MemoryInvalidFree,
    MemoryLeak,
    MemoryDoubleFree,

    // Result (5000-5999)
    ResultInvalidOk,
    ResultInvalidErr,
    ResultUnwrapWithoutCheck,

    // Panic (6000-6999)
    PanicInvalidMessage,
    PanicUnwindMismatch,

    // Link (7000-7999)
    LinkDuplicateSymbol,
    LinkUnresolvedImport,
    LinkTargetMismatch,
    LinkAbiMismatch,
    LinkMissingRpath,
    LinkWrongCrateType,
    LinkInvalidAbiMode,

    // Merge (8000-8999)
    MergeUnresolvedImport,
    MergeAbiMismatch,
    MergeDuplicateExport,
    MergeEffectViolation,
    MergeOwnershipMismatch,
    MergeTypeConflict,
    MergeInvalidGraph,

    // Internal (9000-9999)
    InternalError,
    VerifierFailed,
}

impl Code {
    pub fn code(&self) -> u32 {
        match self {
            Code::ParseUnknownType => 1000,
            Code::ParseMalformedFunctionType => 1001,
            Code::ParseInvalidLifetime => 1002,
            Code::TypeMismatch => 2000,
            Code::TypeNotBorrowable => 2001,
            Code::TypeNotOwned => 2002,
            Code::TypeInvalidResult => 2003,
            Code::TypeInvalidSlice => 2004,
            Code::OwnershipDoubleBorrow => 3000,
            Code::OwnershipUseAfterMove => 3001,
            Code::OwnershipIllegalEscape => 3002,
            Code::OwnershipDanglingReference => 3003,
            Code::OwnershipBorrowExclusivity => 3004,
            Code::MemoryInvalidAlloc => 4000,
            Code::MemoryInvalidFree => 4001,
            Code::MemoryLeak => 4002,
            Code::MemoryDoubleFree => 4003,
            Code::ResultInvalidOk => 5000,
            Code::ResultInvalidErr => 5001,
            Code::ResultUnwrapWithoutCheck => 5002,
            Code::PanicInvalidMessage => 6000,
            Code::PanicUnwindMismatch => 6001,
            Code::LinkDuplicateSymbol => 7000,
            Code::LinkUnresolvedImport => 7001,
            Code::LinkTargetMismatch => 7002,
            Code::LinkAbiMismatch => 7003,
            Code::LinkMissingRpath => 7004,
            Code::LinkWrongCrateType => 7005,
            Code::LinkInvalidAbiMode => 7006,
            Code::MergeUnresolvedImport => 8000,
            Code::MergeAbiMismatch => 8001,
            Code::MergeDuplicateExport => 8002,
            Code::MergeEffectViolation => 8003,
            Code::MergeOwnershipMismatch => 8004,
            Code::MergeTypeConflict => 8005,
            Code::MergeInvalidGraph => 8006,
            Code::InternalError => 9000,
            Code::VerifierFailed => 9001,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Code::ParseUnknownType => "E1000",
            Code::ParseMalformedFunctionType => "E1001",
            Code::ParseInvalidLifetime => "E1002",
            Code::TypeMismatch => "E2000",
            Code::TypeNotBorrowable => "E2001",
            Code::TypeNotOwned => "E2002",
            Code::TypeInvalidResult => "E2003",
            Code::TypeInvalidSlice => "E2004",
            Code::OwnershipDoubleBorrow => "E3000",
            Code::OwnershipUseAfterMove => "E3001",
            Code::OwnershipIllegalEscape => "E3002",
            Code::OwnershipDanglingReference => "E3003",
            Code::OwnershipBorrowExclusivity => "E3004",
            Code::MemoryInvalidAlloc => "E4000",
            Code::MemoryInvalidFree => "E4001",
            Code::MemoryLeak => "E4002",
            Code::MemoryDoubleFree => "E4003",
            Code::ResultInvalidOk => "E5000",
            Code::ResultInvalidErr => "E5001",
            Code::ResultUnwrapWithoutCheck => "E5002",
            Code::PanicInvalidMessage => "E6000",
            Code::PanicUnwindMismatch => "E6001",
            Code::LinkDuplicateSymbol => "E7000",
            Code::LinkUnresolvedImport => "E7001",
            Code::LinkTargetMismatch => "E7002",
            Code::LinkAbiMismatch => "E7003",
            Code::LinkMissingRpath => "E7004",
            Code::LinkWrongCrateType => "E7005",
            Code::LinkInvalidAbiMode => "E7006",
            Code::MergeUnresolvedImport => "E8000",
            Code::MergeAbiMismatch => "E8001",
            Code::MergeDuplicateExport => "E8002",
            Code::MergeEffectViolation => "E8003",
            Code::MergeOwnershipMismatch => "E8004",
            Code::MergeTypeConflict => "E8005",
            Code::MergeInvalidGraph => "E8006",
            Code::InternalError => "E9000",
            Code::VerifierFailed => "E9001",
        }
    }
}

/// Source code span
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

impl Span {
    pub fn new(file: &str, line: u32, column: u32, end_line: u32, end_column: u32) -> Self {
        Self {
            file: file.to_string(),
            line,
            column,
            end_line,
            end_column,
        }
    }

    pub fn single(file: &str, line: u32, column: u32) -> Self {
        Self {
            file: file.to_string(),
            line,
            column,
            end_line: line,
            end_column: column + 1,
        }
    }
}

/// A single diagnostic message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: Code,
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
    #[serde(default)]
    pub hint: Option<String>,
    #[serde(default)]
    pub context: Vec<String>,
}

impl Diagnostic {
    pub fn error(code: Code, message: &str) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.to_string(),
            span: None,
            hint: None,
            context: vec![],
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_hint(mut self, hint: &str) -> Self {
        self.hint = Some(hint.to_string());
        self
    }
}

/// Diagnostic output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Plain,
    Color,
    Json,
}

/// Diagnostic renderer
pub struct Renderer {
    format: OutputFormat,
    #[allow(dead_code)]
    color_enabled: bool,
}

impl Renderer {
    pub fn new(format: OutputFormat) -> Self {
        Self {
            format,
            color_enabled: format == OutputFormat::Color,
        }
    }

    /// Render a single diagnostic
    pub fn render(&self, diag: &Diagnostic) -> String {
        match self.format {
            OutputFormat::Json => self.render_json(diag),
            _ => self.render_plain(diag),
        }
    }

    fn render_plain(&self, diag: &Diagnostic) -> String {
        let mut result = String::new();

        // Severity and code
        let severity_str = match diag.severity {
            Severity::Note => "note",
            Severity::Warning => "warning",
            Severity::Error => "error",
            Severity::Fatal => "fatal error",
        };

        if let Some(ref span) = diag.span {
            result.push_str(&format!("{}:{}:{}: ", span.file, span.line, span.column));
        }

        result.push_str(&format!("[{}] {}\n", diag.code.name(), severity_str));
        result.push_str(&format!("    {}\n", diag.message));

        if let Some(ref hint) = diag.hint {
            result.push_str(&format!("    help: {}\n", hint));
        }

        for ctx in &diag.context {
            result.push_str(&format!("      {}\n", ctx));
        }

        result
    }

    fn render_json(&self, diag: &Diagnostic) -> String {
        serde_json::to_string(diag).unwrap_or_default()
    }

    /// Render multiple diagnostics
    pub fn render_all(&self, diags: &[Diagnostic]) -> String {
        diags
            .iter()
            .map(|d| self.render(d))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Collect all diagnostics
#[derive(Debug, Default)]
pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    pub fn new() -> Self {
        Self {
            diagnostics: vec![],
        }
    }

    pub fn push(&mut self, diag: Diagnostic) {
        self.diagnostics.push(diag);
    }

    pub fn error(&mut self, code: Code, message: &str) -> &mut Diagnostic {
        self.diagnostics.push(Diagnostic::error(code, message));
        self.diagnostics.last_mut().unwrap()
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error || d.severity == Severity::Fatal)
    }

    pub fn drain(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    pub fn render(&self, format: OutputFormat) -> String {
        Renderer::new(format).render_all(&self.diagnostics)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_values() {
        assert_eq!(Code::ParseUnknownType.code(), 1000);
        assert_eq!(Code::TypeMismatch.code(), 2000);
        assert_eq!(Code::OwnershipDoubleBorrow.code(), 3000);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::Error.to_string(), "error");
        assert_eq!(Severity::Warning.to_string(), "warning");
    }

    #[test]
    fn test_diagnostic_error() {
        let diag = Diagnostic::error(Code::ParseUnknownType, "unknown type");
        assert_eq!(diag.code.code(), 1000);
    }

    #[test]
    fn test_diagnostic_with_span() {
        let diag = Diagnostic::error(Code::TypeMismatch, "type mismatch")
            .with_span(Span::single("test.ch", 10, 5))
            .with_hint("did you mean to use a reference?");
        assert!(diag.span.is_some());
        assert!(diag.hint.is_some());
    }

    #[test]
    fn test_renderer_plain() {
        let renderer = Renderer::new(OutputFormat::Plain);
        let diag = Diagnostic::error(Code::TypeMismatch, "type mismatch");
        let output = renderer.render(&diag);
        assert!(output.contains("type mismatch"));
    }

    #[test]
    fn test_renderer_json() {
        let renderer = Renderer::new(OutputFormat::Json);
        let diag = Diagnostic::error(Code::TypeMismatch, "type mismatch");
        let output = renderer.render(&diag);
        assert!(output.contains("TypeMismatch"));
    }

    #[test]
    fn test_diagnostic_bag_errors() {
        let mut bag = DiagnosticBag::new();
        assert!(!bag.has_errors());

        bag.error(Code::TypeMismatch, "test");
        assert!(bag.has_errors());
    }

    #[test]
    fn test_span_single() {
        let span = Span::single("test.ch", 10, 5);
        assert_eq!(span.line, 10);
        assert_eq!(span.column, 5);
        assert_eq!(span.end_line, 10);
    }

    #[test]
    fn test_all_codes_have_unique_ids() {
        let all_codes = [
            // Parser (1000-1999)
            (Code::ParseUnknownType, 1000, "E1000"),
            (Code::ParseMalformedFunctionType, 1001, "E1001"),
            (Code::ParseInvalidLifetime, 1002, "E1002"),
            // Type (2000-2999)
            (Code::TypeMismatch, 2000, "E2000"),
            (Code::TypeNotBorrowable, 2001, "E2001"),
            (Code::TypeNotOwned, 2002, "E2002"),
            (Code::TypeInvalidResult, 2003, "E2003"),
            (Code::TypeInvalidSlice, 2004, "E2004"),
            // Ownership (3000-3999)
            (Code::OwnershipDoubleBorrow, 3000, "E3000"),
            (Code::OwnershipUseAfterMove, 3001, "E3001"),
            (Code::OwnershipIllegalEscape, 3002, "E3002"),
            (Code::OwnershipDanglingReference, 3003, "E3003"),
            (Code::OwnershipBorrowExclusivity, 3004, "E3004"),
            // Memory (4000-4999)
            (Code::MemoryInvalidAlloc, 4000, "E4000"),
            (Code::MemoryInvalidFree, 4001, "E4001"),
            (Code::MemoryLeak, 4002, "E4002"),
            (Code::MemoryDoubleFree, 4003, "E4003"),
            // Result (5000-5999)
            (Code::ResultInvalidOk, 5000, "E5000"),
            (Code::ResultInvalidErr, 5001, "E5001"),
            (Code::ResultUnwrapWithoutCheck, 5002, "E5002"),
            // Panic (6000-6999)
            (Code::PanicInvalidMessage, 6000, "E6000"),
            (Code::PanicUnwindMismatch, 6001, "E6001"),
            // Link (7000-7999)
            (Code::LinkDuplicateSymbol, 7000, "E7000"),
            (Code::LinkUnresolvedImport, 7001, "E7001"),
            (Code::LinkTargetMismatch, 7002, "E7002"),
            (Code::LinkAbiMismatch, 7003, "E7003"),
            // Internal (9000-9999)
            (Code::InternalError, 9000, "E9000"),
            (Code::VerifierFailed, 9001, "E9001"),
        ];

        // Verify all codes have unique IDs and names
        let mut seen_codes: std::collections::HashSet<u32> = std::collections::HashSet::new();
        let mut seen_names: std::collections::HashSet<&'static str> =
            std::collections::HashSet::new();

        for (code, expected_id, expected_name) in all_codes.iter() {
            let id = code.code();
            let name = code.name();
            assert_eq!(
                id,
                *expected_id,
                "Code {:?} should have id {}",
                code.name(),
                expected_id
            );
            assert_eq!(
                name, *expected_name,
                "Code {:?} should have name {}",
                id, expected_name
            );
            assert!(seen_codes.insert(id), "Duplicate code id: {}", id);
            assert!(seen_names.insert(name), "Duplicate code name: {}", name);
        }
    }

    #[test]
    fn test_all_codes_render_in_plain_format() {
        let renderer = Renderer::new(OutputFormat::Plain);
        let codes = [
            Code::ParseUnknownType,
            Code::TypeMismatch,
            Code::OwnershipDoubleBorrow,
            Code::MemoryInvalidAlloc,
            Code::ResultInvalidOk,
            Code::PanicInvalidMessage,
            Code::LinkDuplicateSymbol,
            Code::InternalError,
        ];

        for code in codes {
            let diag = Diagnostic::error(code, "test message");
            let output = renderer.render(&diag);
            assert!(
                output.contains(&code.name()),
                "Plain render should contain code name for {:?}",
                code
            );
            assert!(
                output.contains("test message"),
                "Plain render should contain message for {:?}",
                code
            );
        }
    }

    #[test]
    fn test_all_codes_render_in_json_format() {
        let renderer = Renderer::new(OutputFormat::Json);
        let codes = [
            Code::ParseUnknownType,
            Code::TypeMismatch,
            Code::OwnershipDoubleBorrow,
            Code::MemoryInvalidAlloc,
            Code::ResultInvalidOk,
            Code::PanicInvalidMessage,
            Code::LinkDuplicateSymbol,
            Code::InternalError,
        ];

        for code in codes {
            let diag = Diagnostic::error(code, "test message");
            let output = renderer.render(&diag);
            assert!(
                !output.is_empty(),
                "JSON render should not be empty for {:?}",
                code
            );
            // Verify it's valid JSON
            let parsed: serde_json::Value = serde_json::from_str(&output)
                .expect(&format!("Should be valid JSON for {:?}: {}", code, output));
            assert!(
                parsed.get("code").is_some(),
                "JSON should have 'code' field for {:?}",
                code
            );
        }
    }

    #[test]
    fn test_diagnostic_bag_collects_all_severities() {
        let mut bag = DiagnosticBag::new();

        bag.push(Diagnostic {
            code: Code::ParseUnknownType,
            severity: Severity::Note,
            message: "note message".to_string(),
            span: None,
            hint: None,
            context: vec![],
        });
        bag.push(Diagnostic {
            code: Code::TypeMismatch,
            severity: Severity::Warning,
            message: "warning message".to_string(),
            span: None,
            hint: None,
            context: vec![],
        });
        bag.push(Diagnostic {
            code: Code::OwnershipDoubleBorrow,
            severity: Severity::Error,
            message: "error message".to_string(),
            span: None,
            hint: None,
            context: vec![],
        });
        bag.push(Diagnostic {
            code: Code::InternalError,
            severity: Severity::Fatal,
            message: "fatal message".to_string(),
            span: None,
            hint: None,
            context: vec![],
        });

        assert_eq!(bag.len(), 4);
        assert!(bag.has_errors()); // Fatal and Error both count as errors
    }

    #[test]
    fn test_diagnostic_serdes_roundtrip() {
        let diag = Diagnostic::error(Code::TypeMismatch, "type mismatch")
            .with_span(Span::new("test.ch", 10, 5, 10, 15))
            .with_hint("check the types");

        let json = serde_json::to_string(&diag).unwrap();
        let parsed: Diagnostic = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.code, diag.code);
        assert_eq!(parsed.severity, diag.severity);
        assert_eq!(parsed.message, diag.message);
        assert!(parsed.span.is_some());
        assert!(parsed.hint.is_some());
    }

    #[test]
    fn test_span_serdes_roundtrip() {
        let span = Span::new("test.ch", 10, 5, 12, 20);
        let json = serde_json::to_string(&span).unwrap();
        let parsed: Span = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, span);
    }

    #[test]
    fn test_code_from_u32() {
        // Helper to check code exists by id
        fn check_code(id: u32) -> Option<&'static str> {
            match id {
                1000 => Some("E1000"),
                2000 => Some("E2000"),
                3000 => Some("E3000"),
                4000 => Some("E4000"),
                5000 => Some("E5000"),
                6000 => Some("E6000"),
                7000 => Some("E7000"),
                8000 => Some("E8000"),
                8001 => Some("E8001"),
                8002 => Some("E8002"),
                8003 => Some("E8003"),
                8004 => Some("E8004"),
                8005 => Some("E8005"),
                8006 => Some("E8006"),
                9000 => Some("E9000"),
                _ => None,
            }
        }

        assert_eq!(check_code(1000), Some("E1000"));
        assert_eq!(check_code(2000), Some("E2000"));
        assert_eq!(check_code(3000), Some("E3000"));
        assert_eq!(check_code(8000), Some("E8000"));
        assert_eq!(check_code(8001), Some("E8001"));
        assert!(check_code(1234).is_none());
    }
}
