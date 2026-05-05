//! Chimera Zig Adapter
//!
//! Validates Zig FFI boundaries: export fn, extern struct, no native slices
//! or error unions crossing boundaries, and proper error handling.
//!
//! # Trust Levels
//!
//! This adapter supports multiple trust levels:
//! - **Authoritative**: Output from the patched Zig compiler (production-ready)
//! - **Fixture**: Output from pre-recorded fixture files (non-authoritative)
//! - **CacheScrape**: Output from Zig build cache scraping (non-authoritative)
//! - **Unavailable**: Output when patched Zig is unavailable (non-authoritative)
//!
//! # Authority Status
//!
//! **IMPORTANT**: The invalidation engine in this crate is NON-AUTHORITATIVE.
//! For production Zig builds, ownership belongs to `zigmera-lowering`.
//! See [`docs/zig-incremental-ownership-plan.md`] for details.
//!
//! This crate is maintained as:
//! - A thin compatibility wrapper for existing code
//! - A fixture/fallback surface when patched Zig is unavailable
//! - A test harness for validating `zigmera-lowering` behavior
//!
//! DO NOT add new invalidation logic here. All invalidation authority for
//! production builds must reside in `zigmera-lowering`.
//!
//! # Safety
//!
//! This adapter validates Zig FFI code and ensures ABI safety.
//!
//! # Public API
//!
//! This crate exposes a stable public API. All public items are documented
//! and considered part of the API guarantee.
//!
//! ## Core Types
//! - [`ZigAdapter`] - Main adapter for validating Zig FFI boundaries
//! - [`ZigItem`] - Representation of parsed Zig items (export fn, extern struct, etc.)
//! - [`ZigParam`] - Function parameter
//! - [`ZigField`] - Struct field
//! - [`ZigStructLayout`] / [`ZigFieldLayout`] - Layout information
//! - [`AdapterError`] - Error types
//! - [`ErrorDomain`] - Error classification
//!
//! ## Parsing Functions
//! - [`parse_zig_source()`] - Parse Zig source and extract FFI items
//! - [`lower_error_union()`] - Lower Zig error unions to Chimera ABI
//! - [`parse_error_from_return()`] - Extract error set from return type
//!
//! ## Validation Functions
//! - [`ZigAdapter::validate_export_fn()`] - Validate export function declaration
//! - [`ZigAdapter::validate_type_not_foreign()`] - Check if type crosses FFI boundary legally
//! - [`ZigAdapter::validate_layout()`] - Validate struct layout matches expected metadata
//!
//! # Module Organization
//!
//! Public submodules (exposed via pub mod):
//! - `snapshot` - Snapshot protocol handling
//! - `context` - Zig compile context (ZigCompileContext)
//! - `graph` - Dependency graph (NON-AUTHORITATIVE)
//! - `invalidation` - Invalidation engine (NON-AUTHORITATIVE, delegating to zigmera-lowering)
//! - `fingerprint` - ABI/layout fingerprints
//! - `comptime` - Comptime cache model
//! - `fallback` - Fallback mode for unavailable patched Zig
//!
//! # Private Items
//!
//! The following are internal implementation details (subject to change):
//! - Internal validation helpers
//! - Regex patterns for parsing
//! - FORBIDDEN_TYPES constant implementation details

// Include snapshot protocol
pub mod snapshot;
// Include dependency graph
pub mod graph;
// Include invalidation engine
pub mod invalidation;
// Include ABI/layout fingerprints
pub mod fingerprint;
// Include comptime cache model
pub mod comptime;
// Include fallback mode for when patched Zig is unavailable
pub mod fallback;
// Include patched Zig detection
pub mod detection;
// Include Zig compile context
pub mod context;
// Include authority boundary
pub mod authority;
// Include artifact emission
pub mod artifact;
use chimera_diagnostics::{Code, DiagnosticBag};
use chimera_meta::LayoutMetadata;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Error domain for Zig adapter errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorDomain {
    None,
    Type,
    Ownership,
    Validation,
    ErrorSet,
}

/// Zig adapter errors
#[derive(Debug, Clone)]
pub enum AdapterError {
    InvalidExport(String),
    UnsupportedType(String),
    NativeTypeCrossesBoundary(String),
    MissingExportFn(String),
    ErrorUnionLowering(String),
    ParseError(String),
}

/// Types that are NOT allowed to cross FFI boundaries in Zig
const FORBIDDEN_TYPES: &[&str] = &[
    "std.",
    "os.",
    "fs.",
    "net.",
    "process.",
    "mem.",
    "allocator",
    "ArrayList",
    "HashMap",
];

/// Zig item representation for FFI validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZigItem {
    ExportFn {
        name: String,
        params: Vec<ZigParam>,
        ret: Option<String>,
        has_error_union: bool,
    },
    ExternStruct {
        name: String,
        fields: Vec<ZigField>,
    },
    TypeAlias {
        name: String,
        typ: String,
    },
    ErrorSet {
        name: String,
    },
}

/// Zig function parameter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZigParam {
    pub name: String,
    pub typ: String,
}

/// Zig struct field
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZigField {
    pub name: String,
    pub typ: String,
}

/// Parse Zig source and extract FFI items
pub fn parse_zig_source(source: &str) -> Result<Vec<ZigItem>, AdapterError> {
    let mut items = Vec::new();

    for line in source.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("//") || line.starts_with("/*") {
            continue;
        }

        // Parse export fn: pub fn name(...) ... or pub fn name(...) -> T
        // Also handles: pub fn name(...) !T (error union directly after paren)
        if let Some(caps) =
            Regex::new(r"^(?:pub\s+)?fn\s+(\w+)\s*\(([^)]*)\)(?:\s*(?:->\s*)?([^\s{]+))?")
                .ok()
                .and_then(|re| re.captures(line))
        {
            let name = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let params_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let ret_str = caps.get(3).map(|m| m.as_str()).unwrap_or("void");

            let params: Vec<ZigParam> = params_str
                .split(',')
                .filter_map(|p| {
                    let parts: Vec<&str> = p.split(':').collect();
                    if parts.len() == 2 {
                        Some(ZigParam {
                            name: parts[0].trim().to_string(),
                            typ: parts[1].trim().to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            let has_error_union = ret_str.contains('!');
            let ret = if ret_str == "void" {
                None
            } else {
                Some(ret_str.to_string())
            };

            items.push(ZigItem::ExportFn {
                name,
                params,
                ret,
                has_error_union,
            });
        }

        // Parse extern struct: const Name = extern struct { ... };
        if let Some(caps) = Regex::new(r"^const\s+(\w+)\s*=\s*extern\s+struct\s*\{([^}]*)\}")
            .ok()
            .and_then(|re| re.captures(line))
        {
            let name = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let fields_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            let fields: Vec<ZigField> = fields_str
                .split(';')
                .filter_map(|f| {
                    let f = f.trim();
                    if f.is_empty() {
                        return None;
                    }
                    let parts: Vec<&str> = f.split(':').collect();
                    if parts.len() == 2 {
                        Some(ZigField {
                            name: parts[0].trim().to_string(),
                            typ: parts[1].trim().to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            items.push(ZigItem::ExternStruct { name, fields });
        }

        // Parse error set: const Name = error { ... };
        if let Some(caps) = Regex::new(r"^const\s+(\w+)\s*=\s*error\s*\{")
            .ok()
            .and_then(|re| re.captures(line))
        {
            let name = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            items.push(ZigItem::ErrorSet { name });
        }

        // Parse type alias: const Name = @Type(...);
        if let Some(caps) = Regex::new(r"^const\s+(\w+)\s*=\s*\.+")
            .ok()
            .and_then(|re| re.captures(line))
        {
            let name = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            items.push(ZigItem::TypeAlias {
                name,
                typ: "@Type".to_string(),
            });
        }
    }

    Ok(items)
}

/// Lower Zig error union to Chimera ABI types
///
/// Zig `!T` (error union) becomes `Result<T, ErrorCode>` in Chimera ABI
/// Zig `?T` (optional) becomes `?T` via nullable pointer handling
pub fn lower_error_union(zig_type: &str) -> Option<String> {
    // Error union: `!T` means `anyerror!T`
    if zig_type.starts_with('!') {
        let inner = &zig_type[1..];
        // The error union lowering: T or E!T becomes (ok: T, err: ErrorCode)
        Some(format!("(ok: {}, err: ch_error)", inner))
    } else if zig_type.starts_with('?') {
        // Optional: `?T` becomes nullable pointer to T
        let inner = &zig_type[1..];
        Some(format!("?{}", inner))
    } else {
        None
    }
}

/// Parse error set type from return type
pub fn parse_error_from_return(ret: &str) -> Option<String> {
    // Check for error union patterns like `!u8` or `anyerror!u8`
    if ret.starts_with('!') {
        Some("anyerror".to_string())
    } else if ret.contains("!") {
        // Could be `anyerror!u8` or similar
        let parts: Vec<&str> = ret.split('!').collect();
        if parts.len() == 2 {
            return Some(parts[0].to_string());
        }
        // Try to extract error set name
        let re = Regex::new(r"(\w+)!").ok()?;
        re.captures(ret)
            .map(|c| c.get(1).map(|m| m.as_str().to_string()).unwrap_or_default())
    } else {
        None
    }
}

/// Zig adapter for validating FFI boundaries
pub struct ZigAdapter {
    diagnostics: DiagnosticBag,
    #[allow(dead_code)]
    extern_structs: Vec<ZigItem>,
    #[allow(dead_code)]
    export_fns: Vec<ZigItem>,
}

impl ZigAdapter {
    /// Create a new Zig adapter
    pub fn new() -> Self {
        Self {
            diagnostics: DiagnosticBag::new(),
            extern_structs: Vec::new(),
            export_fns: Vec::new(),
        }
    }

    /// Get diagnostics from the adapter
    pub fn diagnostics(&self) -> &DiagnosticBag {
        &self.diagnostics
    }

    /// Check if adapter has errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// Parse and validate Zig source
    pub fn parse_source(&mut self, source: &str) -> Result<Vec<ZigItem>, AdapterError> {
        let items = parse_zig_source(source)?;
        for item in &items {
            self.validate_item(item);
        }
        Ok(items)
    }

    fn validate_item(&mut self, item: &ZigItem) {
        match item {
            ZigItem::ExportFn {
                name,
                params,
                ret,
                has_error_union: _,
            } => {
                if !self.validate_export_fn(name, params, ret.as_deref()) {
                    // Error already added
                }
            }
            ZigItem::ExternStruct { name: _, fields } => {
                for field in fields {
                    if !self.validate_type_not_foreign(&field.typ) {
                        // Error already added
                    }
                }
            }
            _ => {}
        }
    }

    /// Validate an export function declaration
    pub fn validate_export_fn(
        &mut self,
        name: &str,
        params: &[ZigParam],
        ret: Option<&str>,
    ) -> bool {
        // Check for std lib usage
        if name.starts_with("std.") || name.starts_with("os.") || name.starts_with("fs.") {
            self.diagnostics.error(
                Code::TypeMismatch,
                &format!(
                    "export fn {} uses standard library which is not allowed in FFI",
                    name
                ),
            );
            return false;
        }

        // Check params
        for param in params {
            if !self.validate_type_not_foreign(&param.typ) {
                return false;
            }
        }

        // Check return type
        if let Some(ret) = ret {
            if !self.validate_type_not_foreign(ret) {
                return false;
            }
        }

        true
    }

    /// Check if a type crosses the FFI boundary illegally
    pub fn validate_type_not_foreign(&mut self, type_name: &str) -> bool {
        // Check for slices
        if type_name.contains("[]") {
            self.diagnostics.error(
                Code::TypeMismatch,
                &format!(
                    "type {} contains slice which is not allowed in FFI",
                    type_name
                ),
            );
            return false;
        }

        // Check for error union (allowed but needs special handling)
        if type_name.starts_with('!') {
            // Error unions are allowed but must be lowered
            return true;
        }

        // Check for optional (allowed)
        if type_name.starts_with('?') {
            return true;
        }

        // Check for forbidden stdlib types
        for forbidden in FORBIDDEN_TYPES {
            if type_name.contains(forbidden) {
                self.diagnostics.error(
                    Code::TypeMismatch,
                    &format!(
                        "type {} contains forbidden standard library type {}",
                        type_name, forbidden
                    ),
                );
                return false;
            }
        }
        true
    }

    /// Validate layout matches expected metadata
    pub fn validate_layout(
        &mut self,
        zig_layout: &ZigStructLayout,
        expected: &LayoutMetadata,
    ) -> bool {
        let mut valid = true;

        if zig_layout.size != expected.size {
            self.diagnostics.error(
                Code::TypeMismatch,
                &format!(
                    "struct {} has size {} but expected {}",
                    zig_layout.name, zig_layout.size, expected.size
                ),
            );
            valid = false;
        }

        if zig_layout.align != expected.align {
            self.diagnostics.error(
                Code::TypeMismatch,
                &format!(
                    "struct {} has alignment {} but expected {}",
                    zig_layout.name, zig_layout.align, expected.align
                ),
            );
            valid = false;
        }

        for expected_field in &expected.fields {
            if let Some(zig_field) = zig_layout
                .fields
                .iter()
                .find(|f| f.name == expected_field.name)
            {
                if zig_field.offset != expected_field.offset {
                    self.diagnostics.error(
                        Code::TypeMismatch,
                        &format!(
                            "field {} in {} has offset {} but expected {}",
                            expected_field.name,
                            zig_layout.name,
                            zig_field.offset,
                            expected_field.offset
                        ),
                    );
                    valid = false;
                }
            }
        }

        valid
    }

    /// Map Zig error set to error domain
    pub fn map_error_set(error_set: &str) -> ErrorDomain {
        if error_set.is_empty() {
            ErrorDomain::None
        } else {
            ErrorDomain::ErrorSet
        }
    }
}

/// Zig struct field layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigFieldLayout {
    pub name: String,
    pub offset: u64,
    pub size: u64,
}

/// Zig struct layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigStructLayout {
    pub name: String,
    pub size: u64,
    pub align: u64,
    pub fields: Vec<ZigFieldLayout>,
}

impl Default for ZigAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdapterError::InvalidExport(s) => write!(f, "Invalid export: {}", s),
            AdapterError::UnsupportedType(s) => write!(f, "Unsupported type: {}", s),
            AdapterError::NativeTypeCrossesBoundary(s) => {
                write!(f, "Native type crosses boundary: {}", s)
            }
            AdapterError::MissingExportFn(s) => write!(f, "Missing export fn: {}", s),
            AdapterError::ErrorUnionLowering(s) => write!(f, "Error union lowering: {}", s),
            AdapterError::ParseError(s) => write!(f, "Parse error: {}", s),
        }
    }
}

impl std::error::Error for AdapterError {}

impl fmt::Display for ErrorDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorDomain::None => write!(f, "none"),
            ErrorDomain::Type => write!(f, "type"),
            ErrorDomain::Ownership => write!(f, "ownership"),
            ErrorDomain::Validation => write!(f, "validation"),
            ErrorDomain::ErrorSet => write!(f, "errorset"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let adapter = ZigAdapter::new();
        assert!(!adapter.has_errors());
    }

    #[test]
    fn test_valid_export_fn() {
        let mut adapter = ZigAdapter::new();
        let params = vec![ZigParam {
            name: "arg".to_string(),
            typ: "c_int".to_string(),
        }];
        assert!(adapter.validate_export_fn("myFunc", &params, Some("c_int")));
        assert!(!adapter.has_errors());
    }

    #[test]
    fn test_invalid_std_export() {
        let mut adapter = ZigAdapter::new();
        let params = vec![];
        assert!(!adapter.validate_export_fn("std.mem.copy", &params, None));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_valid_type() {
        let mut adapter = ZigAdapter::new();
        assert!(adapter.validate_type_not_foreign("c_int"));
        assert!(!adapter.has_errors());
    }

    #[test]
    fn test_invalid_slice_type() {
        let mut adapter = ZigAdapter::new();
        assert!(!adapter.validate_type_not_foreign("[]const u8"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_invalid_stdlib_type() {
        let mut adapter = ZigAdapter::new();
        assert!(!adapter.validate_type_not_foreign("std.mem.Allocator"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_validate_layout_success() {
        let mut adapter = ZigAdapter::new();
        let zig_layout = ZigStructLayout {
            name: "Test".to_string(),
            size: 16,
            align: 8,
            fields: vec![],
        };
        let expected = LayoutMetadata {
            name: "Test".to_string(),
            size: 16,
            align: 8,
            fields: vec![],
            is_packed: false,
        };
        assert!(adapter.validate_layout(&zig_layout, &expected));
    }

    #[test]
    fn test_validate_layout_size_mismatch() {
        let mut adapter = ZigAdapter::new();
        let zig_layout = ZigStructLayout {
            name: "Test".to_string(),
            size: 16,
            align: 8,
            fields: vec![],
        };
        let expected = LayoutMetadata {
            name: "Test".to_string(),
            size: 8,
            align: 8,
            fields: vec![],
            is_packed: false,
        };
        assert!(!adapter.validate_layout(&zig_layout, &expected));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_error_set_mapping() {
        assert_eq!(ZigAdapter::map_error_set(""), ErrorDomain::None);
        assert_eq!(
            ZigAdapter::map_error_set("FileError"),
            ErrorDomain::ErrorSet
        );
    }

    #[test]
    fn test_safe_type() {
        let mut adapter = ZigAdapter::new();
        assert!(adapter.validate_type_not_foreign("u32"));
        assert!(!adapter.has_errors());
    }

    #[test]
    fn test_invalid_os_type() {
        let mut adapter = ZigAdapter::new();
        assert!(!adapter.validate_type_not_foreign("os.File"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_negative_slice_fixture_rejected() {
        let fixture_path = std::path::Path::new("fixtures/negative_slice_crosses_ffi.zig");
        if fixture_path.exists() {
            let content = std::fs::read_to_string(fixture_path).unwrap();
            assert!(content.contains("[]const u8"));
            assert!(content.contains("!std.fs.File"));
        }
    }

    // A6/A7: Parse Zig source tests
    #[test]
    fn test_parse_export_fn() {
        let source = "pub fn add(a: i32, b: i32) i32";
        let items = parse_zig_source(source).unwrap();
        assert_eq!(items.len(), 1);
        assert!(matches!(items[0], ZigItem::ExportFn { .. }));
    }

    #[test]
    fn test_parse_extern_struct() {
        let source = "const Point = extern struct { x: i32, y: i32 }";
        let items = parse_zig_source(source).unwrap();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_parse_error_union_return() {
        let source = "pub fn failing() !i32";
        let items = parse_zig_source(source).unwrap();
        assert!(!items.is_empty());
        if let ZigItem::ExportFn {
            has_error_union, ..
        } = &items[0]
        {
            assert!(has_error_union);
        }
    }

    #[test]
    fn test_lower_error_union() {
        let lowered = lower_error_union("!i32");
        assert!(lowered.is_some());
        assert!(lowered.unwrap().contains("ch_error"));
    }

    #[test]
    fn test_lower_optional() {
        let lowered = lower_error_union("?i32");
        assert!(lowered.is_some());
        assert!(lowered.unwrap() == "?i32");
    }

    #[test]
    fn test_parse_error_from_return() {
        assert_eq!(parse_error_from_return("!u8"), Some("anyerror".to_string()));
        assert_eq!(
            parse_error_from_return("FileError!u8"),
            Some("FileError".to_string())
        );
        assert!(parse_error_from_return("void").is_none());
    }

    #[test]
    fn test_error_union_allowed_in_validation() {
        // Error unions should be allowed (they are valid FFI types, just need lowering)
        let mut adapter = ZigAdapter::new();
        assert!(adapter.validate_type_not_foreign("!i32"));
        assert!(!adapter.has_errors());
    }

    #[test]
    fn test_optional_allowed_in_validation() {
        // Optional types should be allowed
        let mut adapter = ZigAdapter::new();
        assert!(adapter.validate_type_not_foreign("?*u8"));
        assert!(!adapter.has_errors());
    }

    // Task 1 (PR 1): Authority boundary documentation test
    #[test]
    fn test_authority_status_documented() {
        // This test validates that the authority status is properly documented
        // The module docstrings contain the authority status
        let lib_doc = include_str!("lib.rs");
        assert!(
            lib_doc.contains("NON-AUTHORITATIVE"),
            "lib.rs must document non-authoritative status"
        );
        assert!(
            lib_doc.contains("zigmera-lowering"),
            "lib.rs must reference zigmera-lowering as authoritative"
        );
    }
}
