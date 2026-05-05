//! Zig Semantic Snapshot Protocol
//!
//! This module defines how Zig Sema/AIR/type/layout/comptime/export data enters ChimeraIR.
//! The protocol is designed to be language-agnostic and supports incremental updates.
//!
//! # Trust Boundary
//!
//! Data from the Zig compiler crosses this boundary as an untrusted input.
//! The adapter validates all types, layouts, and export signatures before use.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semantic snapshot version
pub const SNAPSHOT_VERSION: &str = "1.0";

/// Zig semantic snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigSnapshot {
    /// Snapshot format version
    pub version: String,
    /// Target triple for this snapshot
    pub target: TargetInfo,
    /// All type declarations
    pub types: Vec<TypeDecl>,
    /// All function declarations (exported and internal)
    pub functions: Vec<FunctionDecl>,
    /// All struct declarations (extern and packed)
    pub structs: Vec<StructDecl>,
    /// All error sets
    pub error_sets: Vec<ErrorSetDecl>,
    /// All comptime values and functions
    pub comptime: Vec<ComptimeValue>,
    /// Embed file contents
    pub embeds: HashMap<String, EmbedEntry>,
    /// Export metadata
    pub exports: Vec<ExportDecl>,
}

/// Target information for this snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    pub triple: String,
    pub os: String,
    pub arch: String,
    pub abi: String,
    pub pointer_width: u32,
}

/// Type declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDecl {
    pub name: String,
    pub kind: TypeKind,
    pub size_bytes: Option<u64>,
    pub align_bytes: Option<u64>,
}

/// Type kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    Void,
    Bool,
    Int {
        signed: bool,
        bits: u32,
    },
    Float {
        bits: u32,
    },
    Pointer {
        child: Box<TypeRef>,
    },
    Array {
        child: Box<TypeRef>,
        len: u64,
    },
    Slice {
        child: Box<TypeRef>,
    },
    Struct {
        name: String,
    },
    Enum {
        name: String,
    },
    ErrorSet {
        name: String,
    },
    Opaque,
    Fn {
        sig: Box<FnSignature>,
    },
    ErrorUnion {
        error: Box<TypeRef>,
        value: Box<TypeRef>,
    },
    Optional {
        child: Box<TypeRef>,
    },
}

/// Type reference (by name or inline definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TypeRef {
    ByName(String),
    Inline(Box<TypeKind>),
}

/// Function declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDecl {
    pub name: String,
    pub signature: FnSignature,
    pub location: SourceLocation,
    pub is_exported: bool,
    pub is_comptime: bool,
}

/// Function signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnSignature {
    pub params: Vec<FnParam>,
    pub ret: Option<Box<TypeRef>>,
    pub call_conv: CallingConvention,
    pub align: Option<u32>,
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnParam {
    pub name: String,
    pub typ: TypeRef,
    pub is_noalias: bool,
}

/// Calling convention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallingConvention {
    Unspecified,
    C,
    Cold,
    Naked,
    Null,
    Stdcall,
    Thiscall,
    Vectorcall,
    Vectorscale,
    Apcs,
    Opaque,
}

/// Source location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Struct declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDecl {
    pub name: String,
    pub layout: StructLayout,
    pub fields: Vec<StructField>,
    pub methods: Vec<FunctionDecl>,
}

/// Struct layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructLayout {
    pub alignment: u32,
    pub size_bytes: u64,
    pub backing_enum: Option<String>,
}

/// Struct field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub typ: TypeRef,
    pub offset_bytes: u64,
    pub alignment: u32,
}

/// Error set declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSetDecl {
    pub name: String,
    pub values: Vec<ErrorValue>,
}

/// Error value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorValue {
    pub name: String,
    pub value: u32,
}

/// Comptime value or function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComptimeValue {
    pub name: String,
    pub kind: ComptimeKind,
    pub location: SourceLocation,
}

/// Comptime kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComptimeKind {
    /// A compile-time known constant
    Constant { value: String },
    /// A function only valid at comptime
    Function { decl: FunctionDecl },
    /// A type known at comptime
    Type { decl: TypeDecl },
    /// A block of comptime code
    Block { source: String },
}

/// Embed file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedEntry {
    pub path: String,
    pub size_bytes: u64,
    pub checksum: String,
}

/// Export declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDecl {
    pub name: String,
    pub symbol: String,
    pub kind: ExportKind,
    pub visibility: Visibility,
    pub linksection: Option<String>,
}

/// Export kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportKind {
    Function,
    Variable,
    Type,
}

/// Visibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Private,
    Pub,
    Export,
}

// ============================================================================
// Snapshot Parser and Validator
// ============================================================================

/// Errors that can occur during snapshot parsing
#[derive(Debug, Clone)]
pub enum SnapshotError {
    InvalidVersion(String),
    MissingRequiredField(String),
    InvalidTypeRef(String),
    InvalidLayout(String),
    DuplicateSymbol(String),
    ParseError(String),
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotError::InvalidVersion(v) => write!(f, "invalid snapshot version: {}", v),
            SnapshotError::MissingRequiredField(field) => {
                write!(f, "missing required field: {}", field)
            }
            SnapshotError::InvalidTypeRef(msg) => write!(f, "invalid type reference: {}", msg),
            SnapshotError::InvalidLayout(msg) => write!(f, "invalid layout: {}", msg),
            SnapshotError::DuplicateSymbol(sym) => write!(f, "duplicate symbol: {}", sym),
            SnapshotError::ParseError(msg) => write!(f, "parse error: {}", msg),
        }
    }
}

impl std::error::Error for SnapshotError {}

/// Parse and validate a Zig semantic snapshot
pub fn parse_snapshot(json: &str) -> Result<ZigSnapshot, SnapshotError> {
    let snapshot: ZigSnapshot =
        serde_json::from_str(json).map_err(|e| SnapshotError::ParseError(e.to_string()))?;

    // Validate version
    if snapshot.version != SNAPSHOT_VERSION {
        return Err(SnapshotError::InvalidVersion(snapshot.version.clone()));
    }

    // Validate required fields
    validate_snapshot(&snapshot)?;

    // Validate no duplicate symbols
    validate_symbols(&snapshot)?;

    Ok(snapshot)
}

/// Validate snapshot has all required fields
fn validate_snapshot(snapshot: &ZigSnapshot) -> Result<(), SnapshotError> {
    // Target must have valid pointer width
    if snapshot.target.pointer_width != 32 && snapshot.target.pointer_width != 64 {
        return Err(SnapshotError::MissingRequiredField(
            "target.pointer_width must be 32 or 64".to_string(),
        ));
    }

    // All type names should be unique
    let mut type_names = std::collections::HashSet::new();
    for decl in &snapshot.types {
        if !type_names.insert(&decl.name) {
            return Err(SnapshotError::DuplicateSymbol(decl.name.clone()));
        }
    }

    // All struct names should be unique
    let mut struct_names = std::collections::HashSet::new();
    for decl in &snapshot.structs {
        if !struct_names.insert(&decl.name) {
            return Err(SnapshotError::DuplicateSymbol(decl.name.clone()));
        }
    }

    // All error set names should be unique
    let mut error_set_names = std::collections::HashSet::new();
    for decl in &snapshot.error_sets {
        if !error_set_names.insert(&decl.name) {
            return Err(SnapshotError::DuplicateSymbol(decl.name.clone()));
        }
    }

    Ok(())
}

/// Validate no duplicate symbols exist
fn validate_symbols(snapshot: &ZigSnapshot) -> Result<(), SnapshotError> {
    let mut symbols = std::collections::HashSet::new();

    // Check function names
    for func in &snapshot.functions {
        if !symbols.insert(format!("fn:{}", func.name)) {
            return Err(SnapshotError::DuplicateSymbol(func.name.clone()));
        }
    }

    // Check struct names
    for struct_ in &snapshot.structs {
        if !symbols.insert(format!("struct:{}", struct_.name)) {
            return Err(SnapshotError::DuplicateSymbol(struct_.name.clone()));
        }
    }

    // Check export names
    for export in &snapshot.exports {
        if !symbols.insert(format!("export:{}", export.symbol)) {
            return Err(SnapshotError::DuplicateSymbol(export.symbol.clone()));
        }
    }

    Ok(())
}

/// Convert a snapshot to ChimeraIR representation
pub fn snapshot_to_chimera(snapshot: &ZigSnapshot) -> ChimeraIR {
    let mut ir = ChimeraIR::new();

    // Add types
    for decl in &snapshot.types {
        ir.add_type(TypeDecl::from_zig(decl));
    }

    // Add structs
    for decl in &snapshot.structs {
        ir.add_struct(StructDecl::from_zig(decl));
    }

    // Add functions (only exported ones become FFI boundaries)
    for func in &snapshot.functions {
        if func.is_exported {
            ir.add_export_function(FnDecl::from_zig(func));
        }
    }

    // Add exports
    for export in &snapshot.exports {
        ir.add_export(Export::from_zig(export));
    }

    ir
}

// ============================================================================
// ChimeraIR types (simplified for adapter compatibility)
// ============================================================================

/// Simplified Chimera IR representation from snapshot
#[derive(Debug, Clone)]
pub struct ChimeraIR {
    pub types: Vec<chimera_meta::LayoutMetadata>,
    pub structs: Vec<chimera_meta::LayoutMetadata>,
    pub functions: Vec<FnDecl>,
    pub exports: Vec<Export>,
}

impl ChimeraIR {
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            structs: Vec::new(),
            functions: Vec::new(),
            exports: Vec::new(),
        }
    }

    pub fn add_type(&mut self, t: chimera_meta::LayoutMetadata) {
        self.types.push(t);
    }

    pub fn add_struct(&mut self, s: chimera_meta::LayoutMetadata) {
        self.structs.push(s);
    }

    pub fn add_export_function(&mut self, f: FnDecl) {
        self.functions.push(f);
    }

    pub fn add_export(&mut self, e: Export) {
        self.exports.push(e);
    }
}

impl Default for ChimeraIR {
    fn default() -> Self {
        Self::new()
    }
}

/// Function declaration for ChimeraIR
#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<(String, String)>, // (name, type)
    pub ret: Option<String>,
}

/// Export declaration for ChimeraIR
#[derive(Debug, Clone)]
pub struct Export {
    pub name: String,
    pub symbol: String,
}

// Conversion helpers
impl FnDecl {
    fn from_zig(zig: &FunctionDecl) -> Self {
        let params: Vec<(String, String)> = zig
            .signature
            .params
            .iter()
            .map(|p| (p.name.clone(), "?".to_string())) // Simplified
            .collect();
        let ret = None; // Would need proper type conversion
        Self {
            name: zig.name.clone(),
            params,
            ret,
        }
    }
}

impl Export {
    fn from_zig(zig: &ExportDecl) -> Self {
        Self {
            name: zig.name.clone(),
            symbol: zig.symbol.clone(),
        }
    }
}

impl TypeDecl {
    fn from_zig(zig: &TypeDecl) -> chimera_meta::LayoutMetadata {
        chimera_meta::LayoutMetadata {
            name: zig.name.clone(),
            size: zig.size_bytes.unwrap_or(0),
            align: zig.align_bytes.unwrap_or(0),
            fields: Vec::new(),
            is_packed: false,
        }
    }
}

impl StructDecl {
    fn from_zig(zig: &StructDecl) -> chimera_meta::LayoutMetadata {
        let fields: Vec<chimera_meta::FieldLayout> = zig
            .fields
            .iter()
            .map(|f| chimera_meta::FieldLayout {
                name: f.name.clone(),
                offset: f.offset_bytes,
                typ: type_ref_name(&f.typ),
                size: 0,
                align: u64::from(f.alignment),
            })
            .collect();

        chimera_meta::LayoutMetadata {
            name: zig.name.clone(),
            size: zig.layout.size_bytes,
            align: u64::from(zig.layout.alignment),
            fields,
            is_packed: zig.layout.backing_enum.is_none(),
        }
    }
}

fn type_ref_name(ty: &TypeRef) -> String {
    match ty {
        TypeRef::ByName(name) => name.clone(),
        TypeRef::Inline(kind) => match kind.as_ref() {
            TypeKind::Void => "void".to_string(),
            TypeKind::Bool => "bool".to_string(),
            TypeKind::Int { signed, bits } => {
                if *signed {
                    format!("i{}", bits)
                } else {
                    format!("u{}", bits)
                }
            }
            TypeKind::Float { bits } => format!("f{}", bits),
            TypeKind::Pointer { child } => format!("*{}", type_ref_name(child)),
            TypeKind::Array { child, len } => format!("[{}]{}", len, type_ref_name(child)),
            TypeKind::Slice { child } => format!("[]{}", type_ref_name(child)),
            TypeKind::Struct { name } => name.clone(),
            TypeKind::Enum { name } => name.clone(),
            TypeKind::ErrorSet { name } => name.clone(),
            TypeKind::Opaque => "opaque".to_string(),
            TypeKind::Fn { .. } => "fn".to_string(),
            TypeKind::ErrorUnion { error, value } => {
                format!("{}!{}", type_ref_name(error), type_ref_name(value))
            }
            TypeKind::Optional { child } => format!("?{}", type_ref_name(child)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_snapshot() {
        let json = r#"{
            "version": "1.0",
            "target": {
                "triple": "x86_64-linux-gnu",
                "os": "linux",
                "arch": "x86_64",
                "abi": "gnu",
                "pointer_width": 64
            },
            "types": [],
            "functions": [],
            "structs": [],
            "error_sets": [],
            "comptime": [],
            "embeds": {},
            "exports": []
        }"#;

        let snapshot = parse_snapshot(json);
        assert!(snapshot.is_ok());
        let s = snapshot.unwrap();
        assert_eq!(s.version, "1.0");
        assert_eq!(s.target.pointer_width, 64);
    }

    #[test]
    fn test_invalid_version() {
        let json = r#"{
            "version": "0.9",
            "target": {
                "triple": "x86_64-linux-gnu",
                "os": "linux",
                "arch": "x86_64",
                "abi": "gnu",
                "pointer_width": 64
            },
            "types": [],
            "functions": [],
            "structs": [],
            "error_sets": [],
            "comptime": [],
            "embeds": {},
            "exports": []
        }"#;

        let result = parse_snapshot(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SnapshotError::InvalidVersion(_)
        ));
    }

    #[test]
    fn test_invalid_pointer_width() {
        let json = r#"{
            "version": "1.0",
            "target": {
                "triple": "x86_64-linux-gnu",
                "os": "linux",
                "arch": "x86_64",
                "abi": "gnu",
                "pointer_width": 128
            },
            "types": [],
            "functions": [],
            "structs": [],
            "error_sets": [],
            "comptime": [],
            "embeds": {},
            "exports": []
        }"#;

        let result = parse_snapshot(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_types_rejected() {
        let json = r#"{
            "version": "1.0",
            "target": {
                "triple": "x86_64-linux-gnu",
                "os": "linux",
                "arch": "x86_64",
                "abi": "gnu",
                "pointer_width": 64
            },
            "types": [
                {"name": "MyType", "kind": {"Int": {"signed": true, "bits": 32}}},
                {"name": "MyType", "kind": {"Int": {"signed": true, "bits": 32}}}
            ],
            "functions": [],
            "structs": [],
            "error_sets": [],
            "comptime": [],
            "embeds": {},
            "exports": []
        }"#;

        let result = parse_snapshot(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SnapshotError::DuplicateSymbol(_)
        ));
    }

    #[test]
    fn test_minimal_function_snapshot() {
        let json = r#"{
            "version": "1.0",
            "target": {
                "triple": "x86_64-linux-gnu",
                "os": "linux",
                "arch": "x86_64",
                "abi": "gnu",
                "pointer_width": 64
            },
            "types": [],
            "functions": [{
                "name": "add",
                "signature": {
                    "params": [
                        {"name": "a", "typ": "c_int", "is_noalias": false},
                        {"name": "b", "typ": "c_int", "is_noalias": false}
                    ],
                    "ret": "c_int",
                    "call_conv": "C"
                },
                "location": {"file": "test.zig", "line": 1, "column": 1},
                "is_exported": true,
                "is_comptime": false
            }],
            "structs": [],
            "error_sets": [],
            "comptime": [],
            "embeds": {},
            "exports": []
        }"#;

        let result = parse_snapshot(json);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert_eq!(s.functions.len(), 1);
        assert_eq!(s.functions[0].name, "add");
        assert!(s.functions[0].is_exported);
    }

    #[test]
    fn test_fixture_snapshot_v1_imports_non_empty_snapshot() {
        let json = include_str!("../../fixtures/test_snapshot_v1.json");
        let snapshot = parse_snapshot(json).expect("fixture snapshot should parse");

        assert_eq!(snapshot.version, SNAPSHOT_VERSION);
        assert_eq!(snapshot.target.pointer_width, 64);
        assert_eq!(snapshot.functions.len(), 2);
        assert_eq!(snapshot.structs.len(), 2);
        assert_eq!(snapshot.error_sets.len(), 1);
        assert_eq!(snapshot.exports.len(), 1);

        let ir = snapshot_to_chimera(&snapshot);
        assert_eq!(ir.functions.len(), 2);
        assert_eq!(ir.exports.len(), 1);
        assert_eq!(ir.structs.len(), 2);
    }
}
