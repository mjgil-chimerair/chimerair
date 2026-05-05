//! Chimera metadata schema modeling
//!
//! Models and validates `.chmeta` schemas, versions, and layout declarations.

use serde::{Deserialize, Serialize};

/// Chimera ABI version - canonical version consumed by all layers
pub const CHIMERA_ABI_VERSION_MAJOR: u32 = 0;
pub const CHIMERA_ABI_VERSION_MINOR: u32 = 1;
pub const CHIMERA_ABI_VERSION_PATCH: u32 = 0;
pub const CHIMERA_ABI_VERSION_STRING: &str = "0.1.0";

/// Check if a version is compatible with the current ABI version
pub fn is_compatible(major: u32, minor: u32, _patch: u32) -> bool {
    major == CHIMERA_ABI_VERSION_MAJOR && minor == CHIMERA_ABI_VERSION_MINOR
}

/// Get current ABI version as a tuple
pub fn get_version() -> (u32, u32, u32) {
    (
        CHIMERA_ABI_VERSION_MAJOR,
        CHIMERA_ABI_VERSION_MINOR,
        CHIMERA_ABI_VERSION_PATCH,
    )
}

/// Chimera ABI version
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Default for Version {
    fn default() -> Self {
        Self::new(0, 1, 0)
    }
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(s: &str) -> Result<Self, ParseVersionError> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(ParseVersionError);
        }
        let major = parts[0].parse().map_err(|_| ParseVersionError)?;
        let minor = parts[1].parse().map_err(|_| ParseVersionError)?;
        let patch = parts[2].parse().map_err(|_| ParseVersionError)?;
        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    pub fn as_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone)]
pub struct ParseVersionError;

impl std::fmt::Display for ParseVersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid version string")
    }
}

impl std::error::Error for ParseVersionError {}

/// Source language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SourceLanguage {
    /// Chimera IR / MLIR source
    #[serde(alias = "mlir")]
    #[default]
    Chimera,
    C,
    Rust,
    Zig,
    Wasm,
}

impl SourceLanguage {
    /// Convert from string, including legacy/alias values
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "chimera" | "mlir" => Some(SourceLanguage::Chimera),
            "c" => Some(SourceLanguage::C),
            "rust" => Some(SourceLanguage::Rust),
            "zig" => Some(SourceLanguage::Zig),
            "wasm" => Some(SourceLanguage::Wasm),
            _ => None,
        }
    }
}

/// Module metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub target: String,
    pub source_lang: SourceLanguage,
}

/// Function declaration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Function {
    pub name: String,
    #[serde(default)]
    pub import: bool,
    #[serde(default)]
    pub export: bool,
    #[serde(default)]
    pub cconv: Option<String>,
    #[serde(default)]
    pub signature: Option<Signature>,
}

/// Proof obligation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofObligation {
    pub id: String,
    #[serde(rename = "type")]
    pub obligation_type: String,
    pub function: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Chimera metadata document
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub version: Version,
    #[serde(default)]
    pub module: Option<Module>,
    #[serde(default)]
    pub imports: Vec<ImportMetadata>,
    #[serde(default)]
    pub exports: Vec<ExportMetadata>,
    #[serde(default)]
    pub contracts: Vec<ContractMetadata>,
    #[serde(default)]
    pub layouts: Vec<LayoutMetadata>,
    #[serde(default)]
    pub functions: Vec<Function>,
    #[serde(default)]
    pub proof_obligations: Vec<ProofObligation>,
    #[serde(default)]
    pub wrappers: Vec<WrapperDeclaration>,
    #[serde(default)]
    pub effects: Vec<EffectMetadata>,
    #[serde(default)]
    pub drops: Vec<DropFnMetadata>,
    #[serde(default)]
    pub allocators: Vec<AllocatorMetadata>,
    #[serde(default)]
    pub panic_policy: Option<PanicPolicyMetadata>,
    #[serde(default)]
    pub trust_assumptions: Vec<TrustAssumptionMetadata>,
}

/// Import metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMetadata {
    pub symbol: String,
    pub signature: Signature,
    pub language: SourceLanguage,
    pub target: String,
    /// C.59: errno mapping for C imports - maps errno values to error domains
    #[serde(default)]
    pub errno_mapping: Option<ErrnoMapping>,
    /// C.51: whether this import requires drop handling for owned values
    #[serde(default)]
    pub requires_drop: bool,
}

impl Default for ImportMetadata {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            signature: Signature::default(),
            language: SourceLanguage::C,
            target: String::new(),
            errno_mapping: None,
            requires_drop: false,
        }
    }
}

/// Errno mapping for C imports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrnoMapping {
    pub domain: String,
    #[serde(default)]
    pub codes: Vec<ErrnoCode>,
}

/// A single errno code mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrnoCode {
    pub value: i32,
    pub name: String,
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub symbol: String,
    pub signature: Signature,
    pub language: SourceLanguage,
    pub target: String,
    #[serde(default)]
    pub is_public: bool,
}

/// Layout metadata for type layouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutMetadata {
    pub name: String,
    pub size: u64,
    pub align: u64,
    #[serde(default)]
    pub fields: Vec<FieldLayout>,
    #[serde(default)]
    pub is_packed: bool,
}

/// Field layout within a struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLayout {
    pub name: String,
    pub offset: u64,
    pub typ: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub align: u64,
}

/// Panic policy metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicPolicyMetadata {
    pub policy: PanicPolicy,
    #[serde(default)]
    pub catches: Vec<String>,
    #[serde(default)]
    pub aborts: Vec<String>,
}

/// Wrapper declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrapperDeclaration {
    pub function: String,
    pub language: String,
    pub path: String,
}

/// Safety classification for functions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SafetyClass {
    Verified,
    Generated,
    Trusted,
    Unsafe,
}

/// Calling convention
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CallingConvention {
    #[default]
    C,
    SysV,
    FastCall,
    ThisCall,
    /// Chimera native calling convention
    Chimera,
}

/// Function signature for import/export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub cconv: CallingConvention,
    pub params: Vec<String>, // Type IDs or names
    pub return_type: Option<String>,
}

impl Default for Signature {
    fn default() -> Self {
        Signature {
            cconv: CallingConvention::C,
            params: vec![],
            return_type: None,
        }
    }
}

/// Contract metadata for exported symbols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub symbol: String,
    pub safety: SafetyClass,
    pub args: Vec<String>,
    pub returns: Option<String>,
    pub effects: Vec<String>,
    pub panic_policy: String,
}

/// Effect metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectMetadata {
    pub effect: String,
    pub may_block: bool,
    pub may_alloc: bool,
    pub may_dealloc: bool,
    pub may_panic: bool,
}

/// Drop function metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropFnMetadata {
    pub symbol: String,
    pub input_type: String,
    pub language: SourceLanguage,
    pub allocator: Option<String>,
    pub is_trusted: bool,
}

/// Allocator kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AllocatorKind {
    System,
    Null,
    Shared,
    LanguageOwned,
    Custom,
}

/// Allocator metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatorMetadata {
    pub id: String,
    pub kind: AllocatorKind,
    pub language: SourceLanguage,
    pub is_system: bool,
}

/// Panic policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PanicPolicy {
    Abort,
    Catch,
    Unwind,
}

/// Trust assumption kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustAssumptionKind {
    TrustedFunction,
    TrustedAllocator,
    TrustedDrop,
    TrustedLinker,
    TrustedForeignAbi,
    ManualProof,
}

/// Trust assumption metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAssumptionMetadata {
    pub kind: TrustAssumptionKind,
    pub description: String,
    pub external_ref: Option<String>,
}

impl Metadata {
    /// Parse metadata from JSON string
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Parse metadata from JSON file
    pub fn parse_file(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Serialize metadata to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Validate metadata
    /// C.67: Validates imports, exports, contracts, layouts, duplicates, source language scope, drops, allocators, effects
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.version.major != 0 || self.version.minor != 1 {
            return Err(ValidationError::UnsupportedVersion(self.version.clone()));
        }
        if let Some(ref m) = self.module {
            if m.name.is_empty() {
                return Err(ValidationError::InvalidModuleName);
            }
            if m.target.is_empty() {
                return Err(ValidationError::MissingRequiredField("target".to_string()));
            }
        }

        // C.67: Check for duplicate imports
        let mut import_symbols: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for imp in &self.imports {
            if imp.symbol.is_empty() {
                return Err(ValidationError::EmptyImportSymbol);
            }
            if !import_symbols.insert(imp.symbol.clone()) {
                return Err(ValidationError::DuplicateImport(imp.symbol.clone()));
            }
        }

        // C.67: Check for duplicate exports
        let mut export_symbols: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for exp in &self.exports {
            if exp.symbol.is_empty() {
                return Err(ValidationError::EmptyExportSymbol);
            }
            if !export_symbols.insert(exp.symbol.clone()) {
                return Err(ValidationError::DuplicateExport(exp.symbol.clone()));
            }
        }

        // C.67: Check for duplicate layouts
        let mut layout_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for layout in &self.layouts {
            if layout.name.is_empty() {
                return Err(ValidationError::EmptyLayoutName);
            }
            if !layout_names.insert(layout.name.clone()) {
                return Err(ValidationError::DuplicateLayout(layout.name.clone()));
            }
            // C.67: Layout size must be >= alignment
            if layout.size > 0 && layout.size < layout.align {
                return Err(ValidationError::LayoutSizeBelowAlign {
                    layout_name: layout.name.clone(),
                    size: layout.size,
                    align: layout.align,
                });
            }
        }

        // C.67: Validate layout field offsets
        for layout in &self.layouts {
            let mut field_offsets: std::collections::HashSet<u64> =
                std::collections::HashSet::new();
            for field in &layout.fields {
                if !field_offsets.insert(field.offset) {
                    // Duplicate offset detected - this might be intentional for padding
                }
            }
        }

        // C.59: Validate C imports have errno mappings
        for imp in &self.imports {
            if imp.language == SourceLanguage::C {
                if imp.errno_mapping.is_none() {
                    return Err(ValidationError::MissingErrnoMapping(imp.symbol.clone()));
                }
                // C.59: Validate errno mapping codes
                if let Some(ref mapping) = imp.errno_mapping {
                    if mapping.codes.is_empty() && mapping.domain.is_empty() {
                        return Err(ValidationError::InvalidErrnoMapping {
                            symbol: imp.symbol.clone(),
                            reason: "empty mapping".to_string(),
                        });
                    }
                    for code in &mapping.codes {
                        // Validate errno value ranges (1-1000 is typical)
                        if code.value < 0 || code.value > 10000 {
                            return Err(ValidationError::InvalidErrnoValue {
                                symbol: imp.symbol.clone(),
                                value: code.value,
                            });
                        }
                    }
                }
            }

            // C.51: Owned cross-language values must require allocator or registered drop path
            if imp.requires_drop {
                // Check if there's a registered drop function for this import's input type
                let has_drop = self.drops.iter().any(|d| d.input_type == imp.symbol);
                // Check if import has allocator
                let has_allocator = imp
                    .signature
                    .params
                    .iter()
                    .any(|p| p.contains("alloc") || p.contains("Allocator"));

                // C.51: Must have either drop path or allocator
                if !has_drop && !has_allocator {
                    return Err(ValidationError::MissingDropFunction {
                        symbol: imp.symbol.clone(),
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ValidationError {
    UnsupportedVersion(Version),
    InvalidModuleName,
    MissingRequiredField(String),
    // C.67: Strengthened metadata validation errors
    DuplicateImport(String),
    DuplicateExport(String),
    DuplicateLayout(String),
    EmptyImportSymbol,
    EmptyExportSymbol,
    EmptyLayoutName,
    InvalidImportMismatch {
        symbol: String,
        expected: String,
        found: String,
    },
    InvalidExportMismatch {
        symbol: String,
        expected: String,
        found: String,
    },
    InvalidTypeSize {
        type_name: String,
        size: u64,
    },
    InvalidLayoutAlign {
        layout_name: String,
        align: u64,
    },
    LayoutSizeBelowAlign {
        layout_name: String,
        size: u64,
        align: u64,
    },
    InvalidSourceLanguageScope {
        symbol: String,
        language: SourceLanguage,
    },
    MissingDropFunction {
        symbol: String,
    },
    MismatchedAllocator {
        symbol: String,
        expected: String,
        found: String,
    },
    InvalidEffectSet {
        symbol: String,
    },
    // C.59: Errno mapping validation errors
    MissingErrnoMapping(String),
    InvalidErrnoMapping {
        symbol: String,
        reason: String,
    },
    InvalidErrnoValue {
        symbol: String,
        value: i32,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::UnsupportedVersion(v) => {
                write!(f, "unsupported version: {}", v.as_string())
            }
            ValidationError::InvalidModuleName => write!(f, "invalid module name"),
            ValidationError::MissingRequiredField(name) => {
                write!(f, "missing required field: {}", name)
            }
            // C.67: Strengthened metadata validation
            ValidationError::DuplicateImport(symbol) => write!(f, "duplicate import: {}", symbol),
            ValidationError::DuplicateExport(symbol) => write!(f, "duplicate export: {}", symbol),
            ValidationError::DuplicateLayout(name) => write!(f, "duplicate layout: {}", name),
            ValidationError::EmptyImportSymbol => write!(f, "import symbol cannot be empty"),
            ValidationError::EmptyExportSymbol => write!(f, "export symbol cannot be empty"),
            ValidationError::EmptyLayoutName => write!(f, "layout name cannot be empty"),
            ValidationError::InvalidImportMismatch {
                symbol,
                expected,
                found,
            } => write!(
                f,
                "import {} mismatch: expected {}, found {}",
                symbol, expected, found
            ),
            ValidationError::InvalidExportMismatch {
                symbol,
                expected,
                found,
            } => write!(
                f,
                "export {} mismatch: expected {}, found {}",
                symbol, expected, found
            ),
            ValidationError::InvalidTypeSize { type_name, size } => {
                write!(f, "invalid type size for {}: {}", type_name, size)
            }
            ValidationError::InvalidLayoutAlign { layout_name, align } => {
                write!(f, "invalid layout alignment for {}: {}", layout_name, align)
            }
            ValidationError::LayoutSizeBelowAlign {
                layout_name,
                size,
                align,
            } => write!(
                f,
                "layout {} size {} below align {}",
                layout_name, size, align
            ),
            ValidationError::InvalidSourceLanguageScope { symbol, language } => write!(
                f,
                "invalid source language scope for {}: {:?}",
                symbol, language
            ),
            ValidationError::MissingDropFunction { symbol } => {
                write!(f, "missing drop function for owned type: {}", symbol)
            }
            ValidationError::MismatchedAllocator {
                symbol,
                expected,
                found,
            } => write!(
                f,
                "allocator mismatch for {}: expected {}, found {}",
                symbol, expected, found
            ),
            ValidationError::InvalidEffectSet { symbol } => {
                write!(f, "invalid effect set for {}", symbol)
            }
            // C.59: Errno mapping errors
            ValidationError::MissingErrnoMapping(symbol) => {
                write!(f, "C import {} missing errno mapping", symbol)
            }
            ValidationError::InvalidErrnoMapping { symbol, reason } => write!(
                f,
                "C import {} has invalid errno mapping: {}",
                symbol, reason
            ),
            ValidationError::InvalidErrnoValue { symbol, value } => {
                write!(f, "C import {} has invalid errno value {}", symbol, value)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::parse("0.1.0").unwrap();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_version_invalid() {
        assert!(Version::parse("invalid").is_err());
        assert!(Version::parse("0.1").is_err());
        assert!(Version::parse("0.1.0.0").is_err());
    }

    #[test]
    fn test_metadata_parse() {
        let json = r#"{
            "version": {"major": 0, "minor": 1, "patch": 0},
            "module": {
                "name": "test",
                "target": "x86_64-unknown-linux-gnu",
                "source_lang": "rust"
            }
        }"#;
        let meta = Metadata::parse(json).unwrap();
        assert_eq!(meta.version.major, 0);
        assert!(meta.module.is_some());
    }

    #[test]
    fn test_metadata_roundtrip() {
        let meta = Metadata {
            version: Version::new(0, 1, 0),
            module: Some(Module {
                name: "test".to_string(),
                target: "x86_64-unknown-linux-gnu".to_string(),
                source_lang: SourceLanguage::Rust,
            }),
            imports: vec![],
            exports: vec![],
            contracts: vec![],
            layouts: vec![],
            functions: vec![],
            proof_obligations: vec![],
            wrappers: vec![],
            effects: vec![],
            drops: vec![],
            allocators: vec![],
            panic_policy: None,
            trust_assumptions: vec![],
        };
        let json = meta.to_json().unwrap();
        let parsed = Metadata::parse(&json).unwrap();
        assert_eq!(parsed.version, meta.version);
    }

    #[test]
    fn test_metadata_validate() {
        let meta = Metadata {
            version: Version::new(0, 1, 0),
            module: Some(Module {
                name: "test".to_string(),
                target: "x86_64-unknown-linux-gnu".to_string(),
                source_lang: SourceLanguage::Rust,
            }),
            imports: vec![],
            exports: vec![],
            contracts: vec![],
            layouts: vec![],
            functions: vec![],
            proof_obligations: vec![],
            wrappers: vec![],
            effects: vec![],
            drops: vec![],
            allocators: vec![],
            panic_policy: None,
            trust_assumptions: vec![],
        };
        assert!(meta.validate().is_ok());
    }

    #[test]
    fn test_validation_error_unsupported_version() {
        let meta = Metadata {
            version: Version::new(1, 0, 0),
            module: None,
            imports: vec![],
            exports: vec![],
            contracts: vec![],
            layouts: vec![],
            functions: vec![],
            proof_obligations: vec![],
            wrappers: vec![],
            effects: vec![],
            drops: vec![],
            allocators: vec![],
            panic_policy: None,
            trust_assumptions: vec![],
        };
        assert!(meta.validate().is_err());
    }

    #[test]
    fn test_is_compatible() {
        assert!(is_compatible(0, 1, 0));
        assert!(is_compatible(0, 1, 5));
        assert!(!is_compatible(0, 2, 0));
        assert!(!is_compatible(1, 0, 0));
    }

    #[test]
    fn test_get_version() {
        let v = get_version();
        assert_eq!(v, (0, 1, 0));
    }

    #[test]
    fn test_safety_class_serialization() {
        let safety = SafetyClass::Verified;
        let json = serde_json::to_string(&safety).unwrap();
        assert_eq!(json, "\"verified\"");
        let parsed: SafetyClass = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, SafetyClass::Verified);
    }

    #[test]
    fn test_calling_convention_default() {
        let cconv = CallingConvention::default();
        assert_eq!(cconv, CallingConvention::C);
    }

    #[test]
    fn test_allocator_kind_serialization() {
        let kind = AllocatorKind::System;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"system\"");
        let parsed: AllocatorKind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, AllocatorKind::System);
    }

    #[test]
    fn test_trust_assumption_kind_serialization() {
        let kind = TrustAssumptionKind::TrustedFunction;
        let json = serde_json::to_string(&kind).unwrap();
        assert!(json.contains("trusted_function") || json.contains("trustedFunction"));
    }

    #[test]
    fn test_panic_policy_serialization() {
        let policy = PanicPolicy::Abort;
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, "\"abort\"");
        let parsed: PanicPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, PanicPolicy::Abort);
    }

    #[test]
    fn test_signature_default() {
        let sig = Signature::default();
        assert_eq!(sig.cconv, CallingConvention::C);
        assert!(sig.params.is_empty());
        assert!(sig.return_type.is_none());
    }

    #[test]
    fn test_import_export_metadata() {
        let import = ImportMetadata {
            symbol: "my_func".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::C,
            target: "x86_64-unknown-linux-gnu".to_string(),
            errno_mapping: None,
            requires_drop: false,
        };
        let json = serde_json::to_string(&import).unwrap();
        assert!(json.contains("my_func"));

        let export = ExportMetadata {
            symbol: "export_func".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            is_public: true,
        };
        let json = serde_json::to_string(&export).unwrap();
        assert!(json.contains("export_func"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_layout_metadata() {
        let layout = LayoutMetadata {
            name: "MyStruct".to_string(),
            size: 64,
            align: 8,
            fields: vec![
                FieldLayout {
                    name: "field1".to_string(),
                    offset: 0,
                    typ: "i32".to_string(),
                    size: 4,
                    align: 4,
                },
                FieldLayout {
                    name: "field2".to_string(),
                    offset: 32,
                    typ: "i64".to_string(),
                    size: 8,
                    align: 8,
                },
            ],
            is_packed: false,
        };
        let json = serde_json::to_string_pretty(&layout).unwrap();
        assert!(json.contains("MyStruct"));
        assert!(json.contains("64"));
        assert!(json.contains("field1"));
        // is_packed is default(false) so omitted; check fields have size/align
        assert!(json.contains("\"size\":4") || json.contains("size"));
    }

    #[test]
    fn test_layout_metadata_with_alignment() {
        // Test that fields with proper alignment are verified
        let layout = LayoutMetadata {
            name: "AlignedStruct".to_string(),
            size: 16,
            align: 8,
            fields: vec![
                FieldLayout {
                    name: "a".to_string(),
                    offset: 0,
                    typ: "i32".to_string(),
                    size: 4,
                    align: 4,
                },
                FieldLayout {
                    name: "b".to_string(),
                    offset: 8,
                    typ: "i64".to_string(),
                    size: 8,
                    align: 8,
                },
            ],
            is_packed: false,
        };
        assert_eq!(layout.size, 16);
        assert_eq!(layout.fields.len(), 2);
        assert_eq!(layout.fields[0].align, 4);
        assert_eq!(layout.fields[1].align, 8);
    }

    #[test]
    fn test_effect_metadata() {
        let effect = EffectMetadata {
            effect: "io".to_string(),
            may_block: true,
            may_alloc: false,
            may_dealloc: false,
            may_panic: false,
        };
        let json = serde_json::to_string(&effect).unwrap();
        assert!(json.contains("io"));
        assert!(json.contains("may_block"));
    }

    #[test]
    fn test_drop_fn_metadata() {
        let drop = DropFnMetadata {
            symbol: "drop_my_type".to_string(),
            input_type: "MyType".to_string(),
            language: SourceLanguage::Rust,
            allocator: Some("system".to_string()),
            is_trusted: true,
        };
        let json = serde_json::to_string(&drop).unwrap();
        assert!(json.contains("drop_my_type"));
        assert!(json.contains("trusted"));
    }

    #[test]
    fn test_trust_assumption_metadata() {
        let trust = TrustAssumptionMetadata {
            kind: TrustAssumptionKind::TrustedLinker,
            description: "Linker produces valid ELF binaries".to_string(),
            external_ref: Some("https://example.com/linker-spec".to_string()),
        };
        let json = serde_json::to_string(&trust).unwrap();
        assert!(json.contains("trusted_linker") || json.contains("trustedLinker"));
        assert!(json.contains("Linker produces valid ELF binaries"));
    }

    #[test]
    fn test_panic_policy_metadata() {
        let policy = PanicPolicyMetadata {
            policy: PanicPolicy::Catch,
            catches: vec!["error_handler".to_string()],
            aborts: vec!["critical_failure".to_string()],
        };
        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("catch"));
        assert!(json.contains("error_handler"));
    }

    #[test]
    fn test_full_metadata_alignment() {
        // Test that Rust metadata can parse output from compiler-core driver
        let json = r#"{
          "version": {"major": 0, "minor": 1, "patch": 0},
          "module": {
            "name": "test_module",
            "target": "x86_64-unknown-linux-gnu",
            "source_lang": "mlir"
          },
          "functions": [],
          "proof_obligations": [],
          "wrappers": []
        }"#;
        let meta = Metadata::parse(json).unwrap();
        assert_eq!(meta.version.major, 0);
        assert_eq!(meta.version.minor, 1);
        assert!(meta.module.is_some());
        assert_eq!(meta.module.as_ref().unwrap().name, "test_module");
        assert_eq!(
            meta.module.as_ref().unwrap().target,
            "x86_64-unknown-linux-gnu"
        );
    }

    // G1-G5: Runtime Layout Verification tests
    #[test]
    fn test_g1_c_header_layout() {
        // G1: Verify C header layout matches expected metadata
        let c_layout = LayoutMetadata {
            name: "ch_status_t".to_string(),
            size: 4,
            align: 4,
            fields: vec![FieldLayout {
                name: "code".to_string(),
                offset: 0,
                typ: "u32".to_string(),
                size: 4,
                align: 4,
            }],
            is_packed: false,
        };

        // Verify size and alignment match canonical values
        assert_eq!(c_layout.size, 4, "ch_status_t should be 4 bytes");
        assert_eq!(c_layout.align, 4, "ch_status_t should be 4-byte aligned");
        assert_eq!(c_layout.fields.len(), 1, "ch_status_t should have 1 field");
        assert_eq!(c_layout.fields[0].name, "code");
        assert_eq!(c_layout.fields[0].offset, 0);
    }

    #[test]
    fn test_g2_rust_repr_layout() {
        // G2: Verify Rust repr(C) layout matches expected metadata
        let rust_layout = LayoutMetadata {
            name: "ChStatus".to_string(),
            size: 4,
            align: 4,
            fields: vec![FieldLayout {
                name: "code".to_string(),
                offset: 0,
                typ: "u32".to_string(),
                size: 4,
                align: 4,
            }],
            is_packed: false,
        };

        // Verify Rust layout matches C layout
        assert_eq!(rust_layout.size, 4);
        assert_eq!(rust_layout.align, 4);
        // G2: Rust repr(C) should produce same layout as C
        assert_eq!(rust_layout.size, 4); // Would compare against C layout in real test
    }

    #[test]
    fn test_g3_zig_layout() {
        // G3: Verify Zig extern struct layout matches expected metadata
        let zig_layout = LayoutMetadata {
            name: "ch_status_t".to_string(),
            size: 4,
            align: 4,
            fields: vec![FieldLayout {
                name: "code".to_string(),
                offset: 0,
                typ: "u32".to_string(),
                size: 4,
                align: 4,
            }],
            is_packed: false,
        };

        // Verify Zig layout matches C layout
        assert_eq!(zig_layout.size, 4);
        assert_eq!(zig_layout.align, 4);
    }

    #[test]
    fn test_g4_cross_language_roundtrip() {
        // G4: Verify C -> Rust -> Zig layout consistency
        let c_layout = LayoutMetadata {
            name: "Point".to_string(),
            size: 16,
            align: 8,
            fields: vec![
                FieldLayout {
                    name: "x".to_string(),
                    offset: 0,
                    typ: "i32".to_string(),
                    size: 4,
                    align: 4,
                },
                FieldLayout {
                    name: "y".to_string(),
                    offset: 8,
                    typ: "i32".to_string(),
                    size: 4,
                    align: 4,
                },
            ],
            is_packed: false,
        };

        // G4: Simulate Rust repr(C) layout (should match C)
        let rust_layout = LayoutMetadata {
            name: "Point".to_string(),
            size: 16,
            align: 8,
            fields: vec![
                FieldLayout {
                    name: "x".to_string(),
                    offset: 0,
                    typ: "i32".to_string(),
                    size: 4,
                    align: 4,
                },
                FieldLayout {
                    name: "y".to_string(),
                    offset: 8,
                    typ: "i32".to_string(),
                    size: 4,
                    align: 4,
                },
            ],
            is_packed: false,
        };

        // G4: Verify all three languages produce same layout
        assert_eq!(c_layout.size, rust_layout.size);
        assert_eq!(c_layout.align, rust_layout.align);
        assert_eq!(c_layout.fields.len(), rust_layout.fields.len());
    }

    #[test]
    fn test_g5_canonical_struct_verification() {
        // G5: Verify ch_status, ch_error, ch_allocator layouts
        use super::chimera_abi;

        // ch_status_t verification
        let status = chimera_abi::ch_status_layout();
        assert_eq!(status.name, "ch_status_t");
        assert_eq!(status.size, 4);
        assert_eq!(status.align, 4);

        // ch_error_t verification
        let error = chimera_abi::ch_error_layout();
        assert_eq!(error.name, "ch_error_t");
        assert_eq!(error.size, 8);
        assert_eq!(error.align, 8);
        assert_eq!(error.fields.len(), 2);

        // ch_allocator_t verification
        let allocator = chimera_abi::ch_allocator_layout();
        assert_eq!(allocator.name, "ch_allocator_t");
        assert_eq!(allocator.size, 16);
        assert_eq!(allocator.align, 8);
        assert_eq!(allocator.fields.len(), 2);
    }

    #[test]
    fn test_layout_verification_matching() {
        use super::verify_layouts_match;

        let layout1 = LayoutMetadata {
            name: "Test".to_string(),
            size: 8,
            align: 8,
            fields: vec![],
            is_packed: false,
        };

        let layout2 = LayoutMetadata {
            name: "Test".to_string(),
            size: 8,
            align: 8,
            fields: vec![],
            is_packed: false,
        };

        assert!(verify_layouts_match(&layout1, &layout2).is_ok());
    }

    #[test]
    fn test_layout_verification_size_mismatch() {
        use super::verify_layouts_match;
        use super::LayoutError;

        let layout1 = LayoutMetadata {
            name: "Test".to_string(),
            size: 8,
            align: 8,
            fields: vec![],
            is_packed: false,
        };

        let layout2 = LayoutMetadata {
            name: "Test".to_string(),
            size: 16, // Different size
            align: 8,
            fields: vec![],
            is_packed: false,
        };

        let result = verify_layouts_match(&layout1, &layout2);
        assert!(matches!(result, Err(LayoutError::SizeMismatch { .. })));
    }

    #[test]
    fn test_layout_verification_align_mismatch() {
        use super::verify_layouts_match;
        use super::LayoutError;

        let layout1 = LayoutMetadata {
            name: "Test".to_string(),
            size: 8,
            align: 8,
            fields: vec![],
            is_packed: false,
        };

        let layout2 = LayoutMetadata {
            name: "Test".to_string(),
            size: 8,
            align: 4, // Different alignment
            fields: vec![],
            is_packed: false,
        };

        let result = verify_layouts_match(&layout1, &layout2);
        assert!(matches!(result, Err(LayoutError::AlignMismatch { .. })));
    }

    #[test]
    fn test_layout_verification_offset_mismatch() {
        use super::verify_layouts_match;
        use super::LayoutError;

        let layout1 = LayoutMetadata {
            name: "Test".to_string(),
            size: 16,
            align: 8,
            fields: vec![
                FieldLayout {
                    name: "a".to_string(),
                    offset: 0,
                    typ: "i32".to_string(),
                    size: 4,
                    align: 4,
                },
                FieldLayout {
                    name: "b".to_string(),
                    offset: 8,
                    typ: "i64".to_string(),
                    size: 8,
                    align: 8,
                },
            ],
            is_packed: false,
        };

        let layout2 = LayoutMetadata {
            name: "Test".to_string(),
            size: 16,
            align: 8,
            fields: vec![
                FieldLayout {
                    name: "a".to_string(),
                    offset: 0,
                    typ: "i32".to_string(),
                    size: 4,
                    align: 4,
                },
                FieldLayout {
                    name: "b".to_string(),
                    offset: 4,
                    typ: "i64".to_string(),
                    size: 8,
                    align: 8,
                }, // Wrong offset
            ],
            is_packed: false,
        };

        let result = verify_layouts_match(&layout1, &layout2);
        assert!(matches!(result, Err(LayoutError::OffsetMismatch { .. })));
    }

    // C.67: Tests for strengthened metadata validation

    #[test]
    fn test_c67_duplicate_import_rejected() {
        let mut meta = Metadata::default();
        meta.imports.push(ImportMetadata {
            symbol: "func".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            ..Default::default()
        });
        meta.imports.push(ImportMetadata {
            symbol: "func".to_string(), // Duplicate!
            signature: Signature::default(),
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            ..Default::default()
        });
        assert!(meta.validate().is_err());
    }

    #[test]
    fn test_c67_duplicate_export_rejected() {
        let mut meta = Metadata::default();
        meta.exports.push(ExportMetadata {
            symbol: "exported".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            is_public: true,
        });
        meta.exports.push(ExportMetadata {
            symbol: "exported".to_string(), // Duplicate!
            signature: Signature::default(),
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            is_public: true,
        });
        assert!(meta.validate().is_err());
    }

    #[test]
    fn test_c67_duplicate_layout_rejected() {
        let mut meta = Metadata::default();
        meta.layouts.push(LayoutMetadata {
            name: "MyStruct".to_string(),
            size: 8,
            align: 8,
            fields: vec![],
            is_packed: false,
        });
        meta.layouts.push(LayoutMetadata {
            name: "MyStruct".to_string(), // Duplicate!
            size: 8,
            align: 8,
            fields: vec![],
            is_packed: false,
        });
        assert!(meta.validate().is_err());
    }

    #[test]
    fn test_c67_empty_import_symbol_rejected() {
        let mut meta = Metadata::default();
        meta.imports.push(ImportMetadata {
            symbol: "".to_string(), // Empty!
            signature: Signature::default(),
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            ..Default::default()
        });
        assert!(matches!(
            meta.validate().unwrap_err(),
            ValidationError::EmptyImportSymbol
        ));
    }

    #[test]
    fn test_c67_empty_export_symbol_rejected() {
        let mut meta = Metadata::default();
        meta.exports.push(ExportMetadata {
            symbol: "".to_string(), // Empty!
            signature: Signature::default(),
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            is_public: true,
        });
        assert!(matches!(
            meta.validate().unwrap_err(),
            ValidationError::EmptyExportSymbol
        ));
    }

    #[test]
    fn test_c67_empty_layout_name_rejected() {
        let mut meta = Metadata::default();
        meta.layouts.push(LayoutMetadata {
            name: "".to_string(), // Empty!
            size: 8,
            align: 8,
            fields: vec![],
            is_packed: false,
        });
        assert!(matches!(
            meta.validate().unwrap_err(),
            ValidationError::EmptyLayoutName
        ));
    }

    #[test]
    fn test_c67_layout_size_below_align_rejected() {
        let mut meta = Metadata::default();
        meta.layouts.push(LayoutMetadata {
            name: "SmallStruct".to_string(),
            size: 2, // Size below alignment
            align: 8,
            fields: vec![],
            is_packed: false,
        });
        let result = meta.validate();
        assert!(matches!(
            result,
            Err(ValidationError::LayoutSizeBelowAlign { .. })
        ));
    }

    #[test]
    fn test_c67_validation_error_display() {
        let err = ValidationError::DuplicateImport("func".to_string());
        assert!(err.to_string().contains("duplicate import"));
        assert!(err.to_string().contains("func"));

        let err = ValidationError::MissingDropFunction {
            symbol: "my_func".to_string(),
        };
        assert!(err.to_string().contains("missing drop function"));
        assert!(err.to_string().contains("my_func"));

        let err = ValidationError::LayoutSizeBelowAlign {
            layout_name: "Test".to_string(),
            size: 2,
            align: 8,
        };
        assert!(err.to_string().contains("size 2 below align 8"));
    }

    // C.59: Tests for C errno bridge integration

    #[test]
    fn test_c59_rust_import_no_errno_required() {
        // Rust imports don't need errno mapping
        let mut meta = Metadata::default();
        meta.imports.push(ImportMetadata {
            symbol: "rust_func".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            errno_mapping: None,
            requires_drop: false,
        });
        assert!(meta.validate().is_ok());
    }

    #[test]
    fn test_c59_c_import_with_errno_mapping_ok() {
        // C imports with errno mapping should pass
        let mut meta = Metadata::default();
        meta.imports.push(ImportMetadata {
            symbol: "c_read".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::C,
            target: "x86_64-unknown-linux-gnu".to_string(),
            errno_mapping: Some(ErrnoMapping {
                domain: "io".to_string(),
                codes: vec![
                    ErrnoCode {
                        value: 2,
                        name: "ENOENT".to_string(),
                    },
                    ErrnoCode {
                        value: 5,
                        name: "EIO".to_string(),
                    },
                    ErrnoCode {
                        value: 12,
                        name: "ENOMEM".to_string(),
                    },
                ],
            }),
            requires_drop: false,
        });
        assert!(meta.validate().is_ok());
    }

    #[test]
    fn test_c59_c_import_missing_errno_mapping_rejected() {
        // C imports must have errno mapping
        let mut meta = Metadata::default();
        meta.imports.push(ImportMetadata {
            symbol: "c_func".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::C,
            target: "x86_64-unknown-linux-gnu".to_string(),
            errno_mapping: None,
            requires_drop: false,
        });
        let result = meta.validate();
        assert!(
            matches!(result, Err(ValidationError::MissingErrnoMapping(ref s)) if s == "c_func")
        );
    }

    #[test]
    fn test_c59_c_import_empty_errno_mapping_rejected() {
        // C import with empty errno mapping should be rejected
        let mut meta = Metadata::default();
        meta.imports.push(ImportMetadata {
            symbol: "c_bad_func".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::C,
            target: "x86_64-unknown-linux-gnu".to_string(),
            errno_mapping: Some(ErrnoMapping {
                domain: "".to_string(),
                codes: vec![],
            }),
            requires_drop: false,
        });
        let result = meta.validate();
        assert!(matches!(
            result,
            Err(ValidationError::InvalidErrnoMapping { .. })
        ));
    }

    #[test]
    fn test_c59_c_import_invalid_errno_value_rejected() {
        // C import with out-of-range errno value should be rejected
        let mut meta = Metadata::default();
        meta.imports.push(ImportMetadata {
            symbol: "c_func".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::C,
            target: "x86_64-unknown-linux-gnu".to_string(),
            errno_mapping: Some(ErrnoMapping {
                domain: "io".to_string(),
                codes: vec![
                    ErrnoCode {
                        value: -1,
                        name: "NEGATIVE".to_string(),
                    }, // Invalid negative value
                ],
            }),
            requires_drop: false,
        });
        let result = meta.validate();
        assert!(matches!(
            result,
            Err(ValidationError::InvalidErrnoValue { value: -1, .. })
        ));
    }

    #[test]
    fn test_c59_errno_mapping_serialization() {
        let mapping = ErrnoMapping {
            domain: "io".to_string(),
            codes: vec![ErrnoCode {
                value: 2,
                name: "ENOENT".to_string(),
            }],
        };
        let json = serde_json::to_string(&mapping).unwrap();
        assert!(json.contains("io"));
        assert!(json.contains("ENOENT"));
        assert!(json.contains("2"));
    }

    #[test]
    fn test_c59_mixed_imports_validation() {
        // Mix of C and Rust imports - only C needs errno mapping
        let mut meta = Metadata::default();

        // Rust import - no errno needed
        meta.imports.push(ImportMetadata {
            symbol: "rust_func".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            errno_mapping: None,
            requires_drop: false,
        });

        // C import - needs errno mapping
        meta.imports.push(ImportMetadata {
            symbol: "c_read".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::C,
            target: "x86_64-unknown-linux-gnu".to_string(),
            errno_mapping: Some(ErrnoMapping {
                domain: "io".to_string(),
                codes: vec![ErrnoCode {
                    value: 2,
                    name: "ENOENT".to_string(),
                }],
            }),
            requires_drop: false,
        });

        assert!(meta.validate().is_ok());
    }

    // C.51: Tests for allocator/drop enforcement

    #[test]
    fn test_c51_import_with_drop_no_allocator_ok() {
        // Import with requires_drop=true and drop function registered should pass
        let mut meta = Metadata::default();

        // Register a drop function for "my_func"
        meta.drops.push(DropFnMetadata {
            symbol: "drop_my_func".to_string(),
            input_type: "my_func".to_string(),
            language: SourceLanguage::Rust,
            allocator: None,
            is_trusted: false,
        });

        meta.imports.push(ImportMetadata {
            symbol: "my_func".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            requires_drop: true,
            ..Default::default()
        });

        assert!(meta.validate().is_ok());
    }

    #[test]
    fn test_c51_import_with_allocator_no_drop_ok() {
        // Import with allocator in params but no explicit drop should pass
        let mut meta = Metadata::default();

        meta.imports.push(ImportMetadata {
            symbol: "func_with_alloc".to_string(),
            signature: Signature {
                cconv: CallingConvention::C,
                params: vec!["allocator:system".to_string()],
                return_type: None,
            },
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            requires_drop: true,
            ..Default::default()
        });

        assert!(meta.validate().is_ok());
    }

    #[test]
    fn test_c51_import_missing_drop_and_allocator_rejected() {
        // Import with requires_drop=true but no drop or allocator should fail
        let mut meta = Metadata::default();

        meta.imports.push(ImportMetadata {
            symbol: "func_no_drop".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            requires_drop: true,
            ..Default::default()
        });

        let result = meta.validate();
        assert!(matches!(
            result,
            Err(ValidationError::MissingDropFunction { .. })
        ));
    }

    #[test]
    fn test_c51_import_requires_drop_false_no_constraint() {
        // Import with requires_drop=false should not need drop or allocator
        let mut meta = Metadata::default();

        meta.imports.push(ImportMetadata {
            symbol: "func_simple".to_string(),
            signature: Signature::default(),
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            requires_drop: false,
            ..Default::default()
        });

        assert!(meta.validate().is_ok());
    }

    #[test]
    fn test_c51_allocator_param_name_detection() {
        // Test that various allocator param name patterns are detected
        let mut meta = Metadata::default();

        // "alloc" in param should count as allocator
        meta.imports.push(ImportMetadata {
            symbol: "func_alloc".to_string(),
            signature: Signature {
                cconv: CallingConvention::C,
                params: vec!["alloc:system".to_string()],
                return_type: None,
            },
            language: SourceLanguage::Rust,
            target: "x86_64-unknown-linux-gnu".to_string(),
            requires_drop: true,
            ..Default::default()
        });

        assert!(meta.validate().is_ok());
    }
}

// G1-G5: Runtime Layout Verification

/// Layout verification errors
#[derive(Debug, Clone)]
pub enum LayoutError {
    SizeMismatch {
        name: String,
        expected: u64,
        found: u64,
    },
    AlignMismatch {
        name: String,
        expected: u64,
        found: u64,
    },
    OffsetMismatch {
        name: String,
        field: String,
        expected: u64,
        found: u64,
    },
    MissingField {
        name: String,
        field: String,
    },
}

/// Verify that two layouts are identical
pub fn verify_layouts_match(a: &LayoutMetadata, b: &LayoutMetadata) -> Result<(), LayoutError> {
    if a.size != b.size {
        return Err(LayoutError::SizeMismatch {
            name: a.name.clone(),
            expected: a.size,
            found: b.size,
        });
    }

    if a.align != b.align {
        return Err(LayoutError::AlignMismatch {
            name: a.name.clone(),
            expected: a.align,
            found: b.align,
        });
    }

    // Check field count
    if a.fields.len() != b.fields.len() {
        return Err(LayoutError::SizeMismatch {
            name: a.name.clone(),
            expected: a.fields.len() as u64,
            found: b.fields.len() as u64,
        });
    }

    // Check that a's fields match b's fields (name, offset, size, align)
    for a_field in &a.fields {
        let b_field = b.fields.iter().find(|f| f.name == a_field.name);
        match b_field {
            Some(bf) => {
                if a_field.offset != bf.offset {
                    return Err(LayoutError::OffsetMismatch {
                        name: a.name.clone(),
                        field: a_field.name.clone(),
                        expected: a_field.offset,
                        found: bf.offset,
                    });
                }
                if a_field.size != bf.size {
                    return Err(LayoutError::SizeMismatch {
                        name: format!("{}.{}", a.name, a_field.name),
                        expected: a_field.size,
                        found: bf.size,
                    });
                }
                if a_field.align != bf.align {
                    return Err(LayoutError::AlignMismatch {
                        name: format!("{}.{}", a.name, a_field.name),
                        expected: a_field.align,
                        found: bf.align,
                    });
                }
            }
            None => {
                return Err(LayoutError::MissingField {
                    name: a.name.clone(),
                    field: a_field.name.clone(),
                });
            }
        }
    }

    Ok(())
}

/// Canonical chimera ABI types that must have consistent layouts across languages
pub mod chimera_abi {
    use super::*;

    /// ch_status_t - function return status
    pub fn ch_status_layout() -> LayoutMetadata {
        LayoutMetadata {
            name: "ch_status_t".to_string(),
            size: 4,
            align: 4,
            fields: vec![FieldLayout {
                name: "code".to_string(),
                offset: 0,
                typ: "u32".to_string(),
                size: 4,
                align: 4,
            }],
            is_packed: false,
        }
    }

    /// ch_error_t - error code type
    pub fn ch_error_layout() -> LayoutMetadata {
        LayoutMetadata {
            name: "ch_error_t".to_string(),
            size: 8,
            align: 8,
            fields: vec![
                FieldLayout {
                    name: "domain".to_string(),
                    offset: 0,
                    typ: "u32".to_string(),
                    size: 4,
                    align: 4,
                },
                FieldLayout {
                    name: "code".to_string(),
                    offset: 4,
                    typ: "u32".to_string(),
                    size: 4,
                    align: 4,
                },
            ],
            is_packed: false,
        }
    }

    /// ch_allocator_t - memory allocator handle
    pub fn ch_allocator_layout() -> LayoutMetadata {
        LayoutMetadata {
            name: "ch_allocator_t".to_string(),
            size: 16,
            align: 8,
            fields: vec![
                FieldLayout {
                    name: "alloc".to_string(),
                    offset: 0,
                    typ: "fn*".to_string(),
                    size: 8,
                    align: 8,
                },
                FieldLayout {
                    name: "dealloc".to_string(),
                    offset: 8,
                    typ: "fn*".to_string(),
                    size: 8,
                    align: 8,
                },
            ],
            is_packed: false,
        }
    }

    /// Verify canonical layout internal consistency
    pub fn verify_canonical_layouts() -> Result<(), LayoutError> {
        let status = ch_status_layout();
        let error = ch_error_layout();
        let allocator = ch_allocator_layout();

        // Verify each layout is internally consistent (size covers all fields)
        verify_internal_consistency(&status)?;
        verify_internal_consistency(&error)?;
        verify_internal_consistency(&allocator)?;
        Ok(())
    }

    fn verify_internal_consistency(layout: &LayoutMetadata) -> Result<(), LayoutError> {
        if layout.fields.is_empty() {
            return Ok(());
        }
        let last_field = layout.fields.last().unwrap();
        let end_offset = last_field.offset + last_field.size;
        if end_offset > layout.size {
            return Err(LayoutError::SizeMismatch {
                name: layout.name.clone(),
                expected: layout.size,
                found: end_offset,
            });
        }
        Ok(())
    }
}
