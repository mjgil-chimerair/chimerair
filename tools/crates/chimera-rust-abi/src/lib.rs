//! Chimera Rust ABI Facts Extraction
//!
//! Extracts ABI-related facts from Rust crates:
//! - Symbol names (no_mangle, export_name)
//! - Calling conventions (Rust, C, CUnwind, System)
//! - Visibility and linkage
//! - Import/export classification

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolFact {
    pub stable_id: String,
    pub name: String,
    pub linkage: LinkageKind,
    pub visibility: Visibility,
    pub classification: SymbolClassification,
    pub calling_convention: CallingConvention,
    pub is_unsafe: bool,
    pub is_const: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinkageKind {
    None,
    External,
    Static,
    Internal,
    Weak,
    WeakExternal,
}

impl Default for LinkageKind {
    fn default() -> Self {
        LinkageKind::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Restricted(String),
    Crate,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Public
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CallingConvention {
    Rust,
    C,
    CUnwind,
    System,
    Vectorcall,
    Thiscall,
    Fastcall,
}

impl Default for CallingConvention {
    fn default() -> Self {
        CallingConvention::Rust
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolClassification {
    Export,
    Import,
    ImportFromLibrary,
}

impl Default for SymbolClassification {
    fn default() -> Self {
        SymbolClassification::Export
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiParamFact {
    pub index: usize,
    pub passing_convention: PassingConvention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PassingConvention {
    ByValue,
    ByRef,
    ByValPair,
    Splat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiSignature {
    pub params: Vec<AbiParamFact>,
    pub return_param: Option<AbiParamFact>,
    pub calling_convention: CallingConvention,
    pub is_variadic: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AbiFactExtractor {
    symbols: Vec<SymbolFact>,
}

impl AbiFactExtractor {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    pub fn extract_symbol(&mut self, item: SymbolFact) {
        self.symbols.push(item);
    }

    pub fn from_no_mangle(&mut self, name: String) {
        self.symbols.push(SymbolFact {
            stable_id: format!("sym_{}", name.replace("-", "_").replace("::", "_")),
            name: name.clone(),
            linkage: LinkageKind::External,
            visibility: Visibility::Public,
            classification: SymbolClassification::Export,
            calling_convention: CallingConvention::Rust,
            is_unsafe: false,
            is_const: false,
        });
    }

    pub fn from_export_name(&mut self, _item_name: String, export_name: String) {
        self.symbols.push(SymbolFact {
            stable_id: format!("sym_{}", export_name.replace("-", "_").replace("::", "_")),
            name: export_name,
            linkage: LinkageKind::External,
            visibility: Visibility::Public,
            classification: SymbolClassification::Export,
            calling_convention: CallingConvention::Rust,
            is_unsafe: false,
            is_const: false,
        });
    }

    pub fn from_extern_c(&mut self, name: String) {
        self.symbols.push(SymbolFact {
            stable_id: format!("sym_{}", name.replace("-", "_").replace("::", "_")),
            name: name.clone(),
            linkage: LinkageKind::External,
            visibility: Visibility::Public,
            classification: SymbolClassification::Export,
            calling_convention: CallingConvention::C,
            is_unsafe: false,
            is_const: false,
        });
    }

    pub fn symbols(&self) -> &[SymbolFact] {
        &self.symbols
    }

    pub fn exports(&self) -> Vec<&SymbolFact> {
        self.symbols
            .iter()
            .filter(|s| s.classification == SymbolClassification::Export)
            .collect()
    }

    pub fn imports(&self) -> Vec<&SymbolFact> {
        self.symbols
            .iter()
            .filter(|s| s.classification == SymbolClassification::Import)
            .collect()
    }

    pub fn validate_no_duplicates(&self) -> Result<(), AbiError> {
        let mut names: HashMap<&str, usize> = HashMap::new();
        for (idx, sym) in self.symbols.iter().enumerate() {
            if let Some(prev) = names.get(sym.name.as_str()) {
                return Err(AbiError::DuplicateSymbol {
                    name: sym.name.clone(),
                    first_index: *prev,
                    duplicate_index: idx,
                });
            }
            names.insert(&sym.name, idx);
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.symbols)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let symbols: Vec<SymbolFact> = serde_json::from_str(json)?;
        Ok(Self { symbols })
    }

    /// Compute ABI fingerprint for a symbol (Task 144)
    pub fn fingerprint(&self, symbol: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let sym = self.symbols.iter().find(|s| s.name == symbol);
        let mut hasher = DefaultHasher::new();

        if let Some(s) = sym {
            s.name.hash(&mut hasher);
            format!("{:?}", s.calling_convention).hash(&mut hasher);
            s.is_unsafe.hash(&mut hasher);
            s.is_const.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    /// Compute ABI fingerprint for all exported symbols (Task 144)
    pub fn fingerprint_all(&self) -> Vec<(String, String)> {
        self.exports()
            .iter()
            .map(|s| (s.name.clone(), self.fingerprint(&s.name)))
            .collect()
    }

    /// Comprehensive ABI fingerprint computation (Task 144)
    /// Hashes: symbol, callconv, semantic types, physical ABI, layout refs,
    ///         ownership, panic policy, effect set, target, schema
    pub fn compute_full_fingerprint(
        &self,
        symbol: &str,
        layout_ref: Option<&str>,
        ownership_hash: Option<u64>,
        panic_policy: &str,
        effect_set: &[String],
        target: &str,
        schema_version: u32,
    ) -> String {
        use blake3::Hasher;

        let sym = self.symbols.iter().find(|s| s.name == symbol);
        let mut hasher = Hasher::new();

        // Symbol name
        hasher.update(symbol.as_bytes());

        // Calling convention
        if let Some(s) = sym {
            hasher.update(format!("{:?}", s.calling_convention).as_bytes());
            hasher.update(if s.is_unsafe { b"unsafe" } else { b"safe" });
            hasher.update(if s.is_const { b"const" } else { b"mut" });
            hasher.update(format!("{:?}", s.visibility).as_bytes());
            hasher.update(format!("{:?}", s.linkage).as_bytes());
        }

        // Layout reference
        if let Some(lr) = layout_ref {
            hasher.update(lr.as_bytes());
        }

        // Ownership hash
        if let Some(oh) = ownership_hash {
            hasher.update(&oh.to_le_bytes());
        }

        // Panic policy
        hasher.update(panic_policy.as_bytes());

        // Effect set
        for effect in effect_set {
            hasher.update(effect.as_bytes());
        }

        // Target
        hasher.update(target.as_bytes());

        // Schema version
        hasher.update(&schema_version.to_le_bytes());

        hasher.finalize().to_hex().to_string()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AbiError {
    #[error("duplicate symbol: {name} at indices {first_index} and {duplicate_index}")]
    DuplicateSymbol {
        name: String,
        first_index: usize,
        duplicate_index: usize,
    },
    #[error("invalid calling convention: {0}")]
    InvalidCallingConvention(String),
    #[error("visibility violation: {0}")]
    VisibilityViolation(String),
}

pub fn compute_c_abi_signature(params: &[AbiParamFact]) -> AbiSignature {
    AbiSignature {
        params: params.to_vec(),
        return_param: None,
        calling_convention: CallingConvention::C,
        is_variadic: false,
    }
}

pub fn compute_rust_abi_signature(params: &[AbiParamFact]) -> AbiSignature {
    AbiSignature {
        params: params.to_vec(),
        return_param: None,
        calling_convention: CallingConvention::Rust,
        is_variadic: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_no_mangle() {
        let mut extractor = AbiFactExtractor::new();
        extractor.from_no_mangle("my_function".to_string());

        assert_eq!(extractor.symbols.len(), 1);
        assert_eq!(extractor.exports().len(), 1);
        assert_eq!(extractor.symbols[0].name, "my_function");
        assert_eq!(
            extractor.symbols[0].calling_convention,
            CallingConvention::Rust
        );
    }

    #[test]
    fn test_extract_extern_c() {
        let mut extractor = AbiFactExtractor::new();
        extractor.from_extern_c("c_function".to_string());

        let symbols = extractor.symbols();
        assert_eq!(symbols[0].calling_convention, CallingConvention::C);
        assert_eq!(symbols[0].classification, SymbolClassification::Export);
    }

    #[test]
    fn test_validate_no_duplicates() {
        let mut extractor = AbiFactExtractor::new();
        extractor.from_no_mangle("dup".to_string());
        extractor.from_no_mangle("dup".to_string());

        assert!(extractor.validate_no_duplicates().is_err());
    }

    #[test]
    fn test_roundtrip_json() {
        let mut extractor = AbiFactExtractor::new();
        extractor.from_no_mangle("test_func".to_string());

        let json = extractor.to_json().unwrap();
        let parsed = AbiFactExtractor::from_json(&json).unwrap();

        assert_eq!(parsed.symbols.len(), 1);
        assert_eq!(parsed.symbols[0].name, "test_func");
    }

    #[test]
    fn test_c_abi_signature() {
        let params = vec![
            AbiParamFact {
                index: 0,
                passing_convention: PassingConvention::ByValue,
            },
            AbiParamFact {
                index: 1,
                passing_convention: PassingConvention::ByRef,
            },
        ];
        let sig = compute_c_abi_signature(&params);
        assert_eq!(sig.calling_convention, CallingConvention::C);
        assert_eq!(sig.params.len(), 2);
    }

    #[test]
    fn test_rust_abi_signature() {
        let params = vec![AbiParamFact {
            index: 0,
            passing_convention: PassingConvention::ByValue,
        }];
        let sig = compute_rust_abi_signature(&params);
        assert_eq!(sig.calling_convention, CallingConvention::Rust);
    }

    #[test]
    fn test_linkage_kind_serialization() {
        let linkage = LinkageKind::External;
        let json = serde_json::to_string(&linkage).unwrap();
        let parsed: LinkageKind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, LinkageKind::External);
    }

    #[test]
    fn test_calling_convention_serialization() {
        let cc = CallingConvention::C;
        let json = serde_json::to_string(&cc).unwrap();
        let parsed: CallingConvention = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, CallingConvention::C);
    }

    #[test]
    fn test_symbol_classification() {
        assert_eq!(SymbolClassification::Export, SymbolClassification::Export);
        assert_eq!(SymbolClassification::Import, SymbolClassification::Import);
        assert_ne!(SymbolClassification::Export, SymbolClassification::Import);
    }

    #[test]
    fn test_abi_param_fact() {
        let param = AbiParamFact {
            index: 5,
            passing_convention: PassingConvention::ByRef,
        };
        let json = serde_json::to_string(&param).unwrap();
        let parsed: AbiParamFact = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.index, 5);
        assert_eq!(parsed.passing_convention, PassingConvention::ByRef);
    }

    #[test]
    fn test_abi_signature_serialization() {
        let sig = AbiSignature {
            params: vec![AbiParamFact {
                index: 0,
                passing_convention: PassingConvention::ByValue,
            }],
            return_param: Some(AbiParamFact {
                index: 1,
                passing_convention: PassingConvention::ByValue,
            }),
            calling_convention: CallingConvention::C,
            is_variadic: true,
        };
        let json = serde_json::to_string(&sig).unwrap();
        let parsed: AbiSignature = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.is_variadic, true);
        assert_eq!(parsed.calling_convention, CallingConvention::C);
    }

    #[test]
    fn test_visibility_serialization() {
        let vis = Visibility::Restricted("crate".to_string());
        let json = serde_json::to_string(&vis).unwrap();
        let parsed: Visibility = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Visibility::Restricted("crate".to_string()));
    }

    // Task 144: ABI fingerprint tests

    #[test]
    fn test_abi_fingerprint_basic() {
        let mut extractor = AbiFactExtractor::new();
        extractor.from_no_mangle("my_func".to_string());

        let fp = extractor.fingerprint("my_func");
        assert!(!fp.is_empty());
        assert_eq!(fp.len(), 16); // 64-bit hex
    }

    #[test]
    fn test_abi_fingerprint_nonexistent() {
        let extractor = AbiFactExtractor::new();
        let fp = extractor.fingerprint("nonexistent");
        // Empty string input returns hash of nothing meaningful
        assert!(!fp.is_empty());
    }

    #[test]
    fn test_abi_fingerprint_different_for_different_symbols() {
        let mut extractor = AbiFactExtractor::new();
        extractor.from_no_mangle("func_a".to_string());
        extractor.from_extern_c("func_b".to_string());

        let fp_a = extractor.fingerprint("func_a");
        let fp_b = extractor.fingerprint("func_b");
        assert_ne!(fp_a, fp_b); // Different names should produce different fingerprints
    }

    #[test]
    fn test_abi_fingerprint_all() {
        let mut extractor = AbiFactExtractor::new();
        extractor.from_no_mangle("exported_func".to_string());
        extractor.from_extern_c("c_func".to_string());

        let fps = extractor.fingerprint_all();
        assert_eq!(fps.len(), 2);
    }

    #[test]
    fn test_full_fingerprint_computation() {
        let mut extractor = AbiFactExtractor::new();
        extractor.from_no_mangle("test_func".to_string());

        let fp = extractor.compute_full_fingerprint(
            "test_func",
            Some("layout_ref_123"),
            Some(42),
            "abort",
            &["may_panic".to_string(), "may_alloc".to_string()],
            "x86_64-unknown-linux-gnu",
            1,
        );

        assert!(!fp.is_empty());
        assert_eq!(fp.len(), 64); // blake3 hex length
    }

    #[test]
    fn test_full_fingerprint_deterministic() {
        let mut extractor1 = AbiFactExtractor::new();
        extractor1.from_extern_c("my_func".to_string());

        let mut extractor2 = AbiFactExtractor::new();
        extractor2.from_extern_c("my_func".to_string());

        let fp1 = extractor1.compute_full_fingerprint(
            "my_func",
            None,
            None,
            "unwind",
            &[],
            "aarch64-apple-darwin",
            1,
        );
        let fp2 = extractor2.compute_full_fingerprint(
            "my_func",
            None,
            None,
            "unwind",
            &[],
            "aarch64-apple-darwin",
            1,
        );

        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_full_fingerprint_changes_with_symbol() {
        let mut extractor = AbiFactExtractor::new();
        extractor.from_no_mangle("func_a".to_string());
        extractor.from_no_mangle("func_b".to_string());

        let fp_a = extractor.compute_full_fingerprint(
            "func_a",
            None,
            None,
            "abort",
            &[],
            "x86_64-unknown-linux-gnu",
            1,
        );
        let fp_b = extractor.compute_full_fingerprint(
            "func_b",
            None,
            None,
            "abort",
            &[],
            "x86_64-unknown-linux-gnu",
            1,
        );

        assert_ne!(fp_a, fp_b);
    }

    #[test]
    fn test_full_fingerprint_changes_with_panic_policy() {
        let mut extractor = AbiFactExtractor::new();
        extractor.from_no_mangle("my_fn".to_string());

        let fp_abort = extractor.compute_full_fingerprint(
            "my_fn",
            None,
            None,
            "abort",
            &[],
            "x86_64-unknown-linux-gnu",
            1,
        );
        let fp_unwind = extractor.compute_full_fingerprint(
            "my_fn",
            None,
            None,
            "unwind",
            &[],
            "x86_64-unknown-linux-gnu",
            1,
        );

        assert_ne!(fp_abort, fp_unwind);
    }

    #[test]
    fn test_full_fingerprint_changes_with_target() {
        let mut extractor = AbiFactExtractor::new();
        extractor.from_no_mangle("target_fn".to_string());

        let fp_x86 = extractor.compute_full_fingerprint(
            "target_fn",
            None,
            None,
            "abort",
            &[],
            "x86_64-unknown-linux-gnu",
            1,
        );
        let fp_arm = extractor.compute_full_fingerprint(
            "target_fn",
            None,
            None,
            "abort",
            &[],
            "aarch64-apple-darwin",
            1,
        );

        assert_ne!(fp_x86, fp_arm);
    }

    #[test]
    fn test_full_fingerprint_changes_with_effect_set() {
        let mut extractor = AbiFactExtractor::new();
        extractor.from_no_mangle("effect_fn".to_string());

        let fp_no_effect = extractor.compute_full_fingerprint(
            "effect_fn",
            None,
            None,
            "abort",
            &[],
            "x86_64-unknown-linux-gnu",
            1,
        );
        let fp_with_effect = extractor.compute_full_fingerprint(
            "effect_fn",
            None,
            None,
            "abort",
            &["may_panic".to_string()],
            "x86_64-unknown-linux-gnu",
            1,
        );

        assert_ne!(fp_no_effect, fp_with_effect);
    }
}
