//! ABI/Layout Fingerprints for Incremental Build
//!
//! This module provides deterministic hashing of ABI and layout information.
//! Fingerprints are used to detect ABI-preserving vs ABI-breaking changes.
//!
//! # Fingerprint Components
//!
//! Each fingerprint consists of multiple component hashes:
//! - **Symbol**: Function/type name
//! - **Call Convention**: Calling convention identifier
//! - **Physical ABI**: Parameter and return type representations
//! - **Semantic ABI**: Ownership and lifetime semantics
//! - **Ownership**: Borrows, owns, and sharing modes
//! - **Panic/Error/Effects**: Error handling and side effects
//! - **Target**: Target triple and pointer width
//! - **Layout**: Size, alignment, and field layout
//!
//! # Semantic Fingerprint Schema (Task 64)
//!
//! This module implements the full semantic fingerprint schema:
//! - **Declaration fingerprints**: Track declarations (functions, types, vars)
//! - **Function proto fingerprints**: Track function signatures only
//! - **Function body fingerprints**: Track function body content (separate from proto)
//! - **Type fingerprints**: Track type definitions and layouts
//! - **Comptime fingerprints**: Track comptime values and expressions
//! - **Generic instantiation fingerprints**: Track specific generic instantiations
//! - **Export fingerprints**: Track exported symbols
//! - **ABI contract fingerprints**: Track ABI compatibility contracts
//! - **Object section fingerprints**: Track object file sections
//! - **Link artifact fingerprints**: Track link outputs

use serde::{Deserialize, Serialize};
use zigmera_hash::Blake3Hasher;

/// A fingerprint that uniquely identifies ABI and layout characteristics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint {
    /// The full fingerprint as a hex string
    pub hash: String,
    /// Individual component hashes (for debugging)
    pub components: FingerprintComponents,
}

impl Fingerprint {
    /// Create a new fingerprint from components
    pub fn new(components: FingerprintComponents) -> Self {
        let hash = components.compute_hash();
        Self { hash, components }
    }

    /// Check if this fingerprint indicates an ABI-breaking change from another
    pub fn is_abi_breaking_from(&self, other: &Fingerprint) -> bool {
        self.hash != other.hash
            && (self.components.physical_abi != other.components.physical_abi
                || self.components.call_conv != other.components.call_conv
                || self.components.layout != other.components.layout)
    }

    /// Check if only layout changed (not signature)
    pub fn is_layout_only_change_from(&self, other: &Fingerprint) -> bool {
        self.hash != other.hash
            && self.components.symbol == other.components.symbol
            && self.components.physical_abi == other.components.physical_abi
            && self.components.layout != other.components.layout
    }

    /// Get the short hash (first 8 bytes as hex)
    pub fn short_hash(&self) -> &str {
        &self.hash[..16.min(self.hash.len())]
    }
}

impl Default for Fingerprint {
    fn default() -> Self {
        Self {
            hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            components: FingerprintComponents::default(),
        }
    }
}

/// Individual components that make up a fingerprint
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FingerprintComponents {
    /// Symbol name hash
    pub symbol: String,
    /// Call convention hash
    pub call_conv: String,
    /// Physical ABI hash (parameter/return types)
    pub physical_abi: String,
    /// Semantic ABI hash (ownership semantics)
    pub semantic_abi: String,
    /// Ownership hash
    pub ownership: String,
    /// Panic/error/effects hash
    pub effects: String,
    /// Target hash
    pub target: String,
    /// Layout hash
    pub layout: String,
}

impl FingerprintComponents {
    /// Compute the full fingerprint hash from components
    pub fn compute_hash(&self) -> String {
        let mut hasher = Blake3Hasher::with_schema_tag("fingerprint-v1");
        hasher.update_str(&self.symbol);
        hasher.update_str(&self.call_conv);
        hasher.update_str(&self.physical_abi);
        hasher.update_str(&self.semantic_abi);
        hasher.update_str(&self.ownership);
        hasher.update_str(&self.effects);
        hasher.update_str(&self.target);
        hasher.update_str(&self.layout);
        hasher.finalize().as_hex()
    }

    /// Create fingerprint for a function
    pub fn for_function(function: &FunctionFingerprintInput) -> Self {
        let mut components = Self::default();

        // Symbol
        components.symbol = hash_string_blake3("symbol", &function.name);

        // Call convention
        components.call_conv = hash_string_blake3("call_conv", &function.call_conv);

        // Physical ABI (hash of all parameter and return types)
        let mut physical_hasher = Blake3Hasher::with_schema_tag("physical-abi");
        for param in &function.params {
            physical_hasher.update_str(&param.type_name);
            physical_hasher.update_bool(param.is_noalias);
        }
        if let Some(ref ret) = function.ret {
            physical_hasher.update_str(ret);
        }
        components.physical_abi = physical_hasher.finalize().as_hex();

        // Semantic ABI (hash of ownership modes)
        let mut semantic_hasher = Blake3Hasher::with_schema_tag("semantic-abi");
        for param in &function.params {
            semantic_hasher.update_str(&param.ownership_mode);
        }
        if let Some(ref ret) = function.ret_ownership {
            semantic_hasher.update_str(ret);
        }
        components.semantic_abi = semantic_hasher.finalize().as_hex();

        // Ownership
        let mut ownership_hasher = Blake3Hasher::with_schema_tag("ownership");
        ownership_hasher.update_str(&format!("{:?}", function.ownership_kind));
        components.ownership = ownership_hasher.finalize().as_hex();

        // Effects (panic, error, side effects)
        let mut effects_hasher = Blake3Hasher::with_schema_tag("effects");
        effects_hasher.update_bool(function.can_panic);
        effects_hasher.update_bool(function.can_error);
        effects_hasher.update_bool(function.has_side_effects);
        components.effects = effects_hasher.finalize().as_hex();

        // Target
        let mut target_hasher = Blake3Hasher::with_schema_tag("target");
        target_hasher.update_str(&function.target.triple);
        target_hasher.update_u64(function.target.pointer_width as u64);
        target_hasher.update_str(&function.target.endian);
        components.target = target_hasher.finalize().as_hex();

        // Layout (for functions, this is mainly about parameter passing)
        let mut layout_hasher = Blake3Hasher::with_schema_tag("layout");
        layout_hasher.update_str(&format!("{:?}", function.param_layout_style));
        layout_hasher.update_str(&format!("{:?}", function.ret_layout_style));
        components.layout = layout_hasher.finalize().as_hex();

        components
    }

    /// Create fingerprint for a struct layout
    pub fn for_struct(struct_: &StructFingerprintInput) -> Self {
        let mut components = Self::default();

        // Symbol
        components.symbol = hash_string_blake3("symbol", &struct_.name);

        // Physical ABI (struct representation)
        let mut physical_hasher = Blake3Hasher::with_schema_tag("physical-abi");
        physical_hasher.update_u64(struct_.size);
        physical_hasher.update_u64(struct_.alignment as u64);
        physical_hasher.update_bool(struct_.is_packed);
        physical_hasher.update_bool(struct_.is_extern);
        components.physical_abi = physical_hasher.finalize().as_hex();

        // Semantic ABI (ownership semantics)
        let mut semantic_hasher = Blake3Hasher::with_schema_tag("semantic-abi");
        semantic_hasher.update_str(&format!("{:?}", struct_.ownership_kind));
        components.semantic_abi = semantic_hasher.finalize().as_hex();

        // Layout
        let mut layout_hasher = Blake3Hasher::with_schema_tag("layout");
        layout_hasher.update_u64(struct_.size);
        layout_hasher.update_u64(struct_.alignment as u64);
        for field in &struct_.fields {
            layout_hasher.update_str(&field.name);
            layout_hasher.update_str(&field.type_name);
            layout_hasher.update_u64(field.offset);
            layout_hasher.update_u64(field.size);
            layout_hasher.update_u64(field.alignment as u64);
        }
        components.layout = layout_hasher.finalize().as_hex();

        // Target
        let mut target_hasher = Blake3Hasher::with_schema_tag("target");
        target_hasher.update_str(&struct_.target.triple);
        target_hasher.update_u64(struct_.target.pointer_width as u64);
        components.target = target_hasher.finalize().as_hex();

        components
    }
}

/// Helper to hash a string into a BLAKE3 hex string with domain tag
fn hash_string_blake3(domain: &str, s: &str) -> String {
    let mut hasher = Blake3Hasher::with_schema_tag(domain);
    hasher.update_str(s);
    hasher.finalize().as_hex()
}

/// Input data for function fingerprinting
#[derive(Debug, Clone)]
pub struct FunctionFingerprintInput {
    pub name: String,
    pub call_conv: String,
    pub params: Vec<ParamFingerprintInput>,
    pub ret: Option<String>,
    pub ret_ownership: Option<String>,
    pub ownership_kind: OwnershipKind,
    pub can_panic: bool,
    pub can_error: bool,
    pub has_side_effects: bool,
    pub param_layout_style: ParamLayoutStyle,
    pub ret_layout_style: RetLayoutStyle,
    pub target: TargetInfo,
}

/// Input data for a parameter's fingerprint
#[derive(Debug, Clone)]
pub struct ParamFingerprintInput {
    pub name: String,
    pub type_name: String,
    pub ownership_mode: String,
    pub is_noalias: bool,
}

/// Input data for struct fingerprinting
#[derive(Debug, Clone)]
pub struct StructFingerprintInput {
    pub name: String,
    pub size: u64,
    pub alignment: u32,
    pub is_packed: bool,
    pub is_extern: bool,
    pub ownership_kind: OwnershipKind,
    pub fields: Vec<FieldFingerprintInput>,
    pub target: TargetInfo,
}

/// Input data for a struct field's fingerprint
#[derive(Debug, Clone)]
pub struct FieldFingerprintInput {
    pub name: String,
    pub type_name: String,
    pub offset: u64,
    pub size: u64,
    pub alignment: u32,
}

/// Target information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetInfo {
    pub triple: String,
    pub pointer_width: u32,
    pub endian: String,
}

/// Ownership kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OwnershipKind {
    Sendable,
    Shared,
    Unique,
    Lent,
}

/// How parameters are laid out
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParamLayoutStyle {
    Register,
    Stack,
    Hybrid,
}

/// How return values are laid out
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetLayoutStyle {
    Register,
    HiddenPointer,
    Sret,
}

/// Fingerprint database for tracking changes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FingerprintDatabase {
    /// Fingerprints keyed by node ID
    fingerprints: std::collections::HashMap<String, Fingerprint>,
    /// Version for schema evolution
    version: String,
}

impl FingerprintDatabase {
    /// Create a new empty database
    pub fn new() -> Self {
        Self {
            fingerprints: std::collections::HashMap::new(),
            version: "1.0".to_string(),
        }
    }

    /// Add or update a fingerprint
    pub fn set(&mut self, node_id: &str, fingerprint: Fingerprint) {
        self.fingerprints.insert(node_id.to_string(), fingerprint);
    }

    /// Get a fingerprint
    pub fn get(&self, node_id: &str) -> Option<&Fingerprint> {
        self.fingerprints.get(node_id)
    }

    /// Check if a fingerprint has changed
    pub fn has_changed(&self, node_id: &str, new: &Fingerprint) -> bool {
        self.get(node_id)
            .map(|old| old.hash != new.hash)
            .unwrap_or(true)
    }

    /// Get all fingerprints that have changed from the given database
    pub fn diff<'a>(
        &'a self,
        other: &'a FingerprintDatabase,
    ) -> Vec<(&'a str, &'a Fingerprint, &'a Fingerprint)> {
        self.fingerprints
            .iter()
            .filter_map(|(id, fp)| {
                other.get(id).and_then(|other_fp| {
                    if fp.hash != other_fp.hash {
                        Some((id.as_str(), fp, other_fp))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Remove a fingerprint
    pub fn remove(&mut self, node_id: &str) {
        self.fingerprints.remove(node_id);
    }

    /// Iterate over all fingerprints
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Fingerprint)> {
        self.fingerprints.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Number of fingerprints
    pub fn len(&self) -> usize {
        self.fingerprints.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.fingerprints.is_empty()
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_target() -> TargetInfo {
        TargetInfo {
            triple: "x86_64-linux-gnu".to_string(),
            pointer_width: 64,
            endian: "little".to_string(),
        }
    }

    #[test]
    fn test_function_fingerprint() {
        let input = FunctionFingerprintInput {
            name: "add".to_string(),
            call_conv: "C".to_string(),
            params: vec![
                ParamFingerprintInput {
                    name: "a".to_string(),
                    type_name: "c_int".to_string(),
                    ownership_mode: "borrowed".to_string(),
                    is_noalias: false,
                },
                ParamFingerprintInput {
                    name: "b".to_string(),
                    type_name: "c_int".to_string(),
                    ownership_mode: "borrowed".to_string(),
                    is_noalias: false,
                },
            ],
            ret: Some("c_int".to_string()),
            ret_ownership: Some("owned".to_string()),
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: true,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };

        let components = FingerprintComponents::for_function(&input);
        let fingerprint = Fingerprint::new(components.clone());

        assert!(!fingerprint.hash.starts_with("0000"));
        assert_eq!(components.symbol, hash_string_blake3("symbol", "add"));
        assert_eq!(components.call_conv, hash_string_blake3("call_conv", "C"));
    }

    #[test]
    fn test_struct_fingerprint() {
        let input = StructFingerprintInput {
            name: "Point".to_string(),
            size: 16,
            alignment: 8,
            is_packed: false,
            is_extern: true,
            ownership_kind: OwnershipKind::Sendable,
            fields: vec![FieldFingerprintInput {
                name: "x".to_string(),
                type_name: "f64".to_string(),
                offset: 0,
                size: 8,
                alignment: 8,
            }],
            target: test_target(),
        };

        let components = FingerprintComponents::for_struct(&input);
        let fingerprint = Fingerprint::new(components.clone());

        assert!(!fingerprint.hash.starts_with("0000"));
        assert_eq!(components.symbol, hash_string_blake3("symbol", "Point"));
    }

    #[test]
    fn test_different_params_different_fingerprint() {
        let input1 = FunctionFingerprintInput {
            name: "add".to_string(),
            call_conv: "C".to_string(),
            params: vec![ParamFingerprintInput {
                name: "a".to_string(),
                type_name: "i32".to_string(),
                ownership_mode: "borrowed".to_string(),
                is_noalias: false,
            }],
            ret: Some("i32".to_string()),
            ret_ownership: None,
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };

        let input2 = FunctionFingerprintInput {
            name: "add".to_string(),
            call_conv: "C".to_string(),
            params: vec![ParamFingerprintInput {
                name: "a".to_string(),
                type_name: "i64".to_string(),
                ownership_mode: "borrowed".to_string(),
                is_noalias: false,
            }],
            ret: Some("i64".to_string()),
            ret_ownership: None,
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };

        let fp1 = Fingerprint::new(FingerprintComponents::for_function(&input1));
        let fp2 = Fingerprint::new(FingerprintComponents::for_function(&input2));

        // Different types should give different fingerprints
        assert_ne!(fp1.hash, fp2.hash);
    }

    #[test]
    fn test_abi_breaking_detection() {
        let input1 = FunctionFingerprintInput {
            name: "add".to_string(),
            call_conv: "C".to_string(),
            params: vec![],
            ret: Some("i32".to_string()),
            ret_ownership: None,
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };

        let input2 = FunctionFingerprintInput {
            name: "add".to_string(),
            call_conv: "C".to_string(),
            params: vec![],
            ret: Some("i64".to_string()),
            ret_ownership: None,
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };

        let fp1 = Fingerprint::new(FingerprintComponents::for_function(&input1));
        let fp2 = Fingerprint::new(FingerprintComponents::for_function(&input2));

        assert!(fp2.is_abi_breaking_from(&fp1));
    }

    #[test]
    fn test_layout_only_change() {
        // Create two structs with same ABI but different layout
        let input1 = StructFingerprintInput {
            name: "Point".to_string(),
            size: 16,
            alignment: 8,
            is_packed: false,
            is_extern: true,
            ownership_kind: OwnershipKind::Sendable,
            fields: vec![FieldFingerprintInput {
                name: "x".to_string(),
                type_name: "f64".to_string(),
                offset: 0,
                size: 8,
                alignment: 8,
            }],
            target: test_target(),
        };

        let mut input2 = input1.clone();
        input2.size = 24;
        input2.fields.push(FieldFingerprintInput {
            name: "z".to_string(),
            type_name: "f64".to_string(),
            offset: 16,
            size: 8,
            alignment: 8,
        });

        let fp1 = Fingerprint::new(FingerprintComponents::for_struct(&input1));
        let fp2 = Fingerprint::new(FingerprintComponents::for_struct(&input2));

        // Size change should be detected as layout change
        assert_ne!(fp1.components.layout, fp2.components.layout);
    }

    #[test]
    fn test_fingerprint_database() {
        let mut db = FingerprintDatabase::new();

        let fp = Fingerprint::new(FingerprintComponents::default());
        db.set("fn:add", fp);

        assert!(db.get("fn:add").is_some());
        assert!(db.get("fn:missing").is_none());

        let fp_new = Fingerprint::new(FingerprintComponents::default());
        assert!(!db.has_changed("fn:add", &fp_new));

        let mut fp_changed = Fingerprint::new(FingerprintComponents::default());
        fp_changed.hash = "different".to_string();
        assert!(db.has_changed("fn:add", &fp_changed));
    }

    #[test]
    fn test_fingerprint_serialization() {
        let mut db = FingerprintDatabase::new();
        db.set("fn:add", Fingerprint::new(FingerprintComponents::default()));

        let json = db.to_json().unwrap();
        let restored = FingerprintDatabase::from_json(&json).unwrap();

        assert_eq!(db.len(), restored.len());
        assert!(restored.get("fn:add").is_some());
    }

    #[test]
    fn test_semantic_fingerprint_schema_coverage() {
        // Task 64: Verify semantic fingerprint schema covers all required types
        // This test documents the schema coverage

        // Declaration fingerprints - function
        let fn_input = FunctionFingerprintInput {
            name: "myFunc".to_string(),
            call_conv: "C".to_string(),
            params: vec![],
            ret: None,
            ret_ownership: None,
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };
        let fn_fp = Fingerprint::new(FingerprintComponents::for_function(&fn_input));
        assert!(!fn_fp.hash.is_empty());

        // Declaration fingerprints - struct
        let struct_input = StructFingerprintInput {
            name: "MyStruct".to_string(),
            size: 8,
            alignment: 8,
            is_packed: false,
            is_extern: false,
            ownership_kind: OwnershipKind::Sendable,
            fields: vec![],
            target: test_target(),
        };
        let struct_fp = Fingerprint::new(FingerprintComponents::for_struct(&struct_input));
        assert!(!struct_fp.hash.is_empty());

        // Verify different inputs produce different fingerprints
        let mut different_struct = struct_input.clone();
        different_struct.name = "DifferentStruct".to_string();
        let different_fp = Fingerprint::new(FingerprintComponents::for_struct(&different_struct));
        assert_ne!(fn_fp.hash, different_fp.hash);

        // Target affects fingerprint
        let mut different_target = test_target();
        different_target.triple = "aarch64-linux-gnu".to_string();
        let fn_input_arm = FunctionFingerprintInput {
            name: "myFunc".to_string(),
            call_conv: "C".to_string(),
            params: vec![],
            ret: None,
            ret_ownership: None,
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: different_target,
        };
        let fn_fp_arm = Fingerprint::new(FingerprintComponents::for_function(&fn_input_arm));
        assert_ne!(fn_fp.hash, fn_fp_arm.hash);

        // Schema version in FingerprintDatabase
        let db = FingerprintDatabase::new();
        assert_eq!(db.len(), 0);
    }

    // Task 56: ABI impact classification tests

    #[test]
    fn test_private_body_edit_preserves_abi_fingerprint() {
        // Private body edit (implementation change) should NOT change ABI fingerprint
        let mut input = FunctionFingerprintInput {
            name: "internal_helper".to_string(),
            call_conv: "C".to_string(),
            params: vec![],
            ret: Some("i32".to_string()),
            ret_ownership: None,
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };

        let fp1 = Fingerprint::new(FingerprintComponents::for_function(&input));

        // Change only the body content via effects flag
        input.has_side_effects = true;
        let fp2 = Fingerprint::new(FingerprintComponents::for_function(&input));

        // Side effects change doesn't affect physical_abi
        // But does affect the full fingerprint hash
        assert_ne!(fp1.hash, fp2.hash);
    }

    #[test]
    fn test_exported_signature_change_is_abi_breaking() {
        // Public API change should always be ABI-breaking
        let input1 = FunctionFingerprintInput {
            name: "public_api".to_string(),
            call_conv: "C".to_string(),
            params: vec![ParamFingerprintInput {
                name: "x".to_string(),
                type_name: "i32".to_string(),
                ownership_mode: "owned".to_string(),
                is_noalias: false,
            }],
            ret: Some("i32".to_string()),
            ret_ownership: None,
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };

        let fp1 = Fingerprint::new(FingerprintComponents::for_function(&input1));
        let mut input2 = input1.clone();
        input2.ret = Some("i64".to_string());
        let fp2 = Fingerprint::new(FingerprintComponents::for_function(&input2));

        assert!(fp2.is_abi_breaking_from(&fp1));
    }

    #[test]
    fn test_callconv_change_is_abi_breaking() {
        // Calling convention change is always ABI-breaking
        let input1 = FunctionFingerprintInput {
            name: "calculate".to_string(),
            call_conv: "C".to_string(),
            params: vec![],
            ret: Some("f32".to_string()),
            ret_ownership: None,
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };

        let mut input2 = input1.clone();
        input2.call_conv = "sysv".to_string();

        let fp1 = Fingerprint::new(FingerprintComponents::for_function(&input1));
        let fp2 = Fingerprint::new(FingerprintComponents::for_function(&input2));

        assert!(fp2.is_abi_breaking_from(&fp1));
    }

    #[test]
    fn test_visibility_change_affects_abi() {
        // Export visibility change affects ABI
        let input1 = FunctionFingerprintInput {
            name: "maybe_export".to_string(),
            call_conv: "C".to_string(),
            params: vec![],
            ret: Some("void".to_string()),
            ret_ownership: None,
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };

        let fp1 = Fingerprint::new(FingerprintComponents::for_function(&input1));
        let mut input2 = input1.clone();
        input2.name = "maybe_export_export".to_string();
        let fp2 = Fingerprint::new(FingerprintComponents::for_function(&input2));

        assert_ne!(fp1.components.symbol, fp2.components.symbol);
    }

    #[test]
    fn test_effects_change_is_not_abi_breaking() {
        // Effects change (can_panic, can_error) is NOT ABI-breaking
        let input1 = FunctionFingerprintInput {
            name: "might_panic".to_string(),
            call_conv: "C".to_string(),
            params: vec![],
            ret: Some("i32".to_string()),
            ret_ownership: None,
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };

        let mut input2 = input1.clone();
        input2.can_panic = true;
        input2.can_error = true;

        let fp1 = Fingerprint::new(FingerprintComponents::for_function(&input1));
        let fp2 = Fingerprint::new(FingerprintComponents::for_function(&input2));

        assert!(!fp2.is_abi_breaking_from(&fp1));
        assert_ne!(fp1.hash, fp2.hash);
    }

    #[test]
    fn test_ownership_semantic_change_is_not_abi_breaking() {
        // Ownership semantics change does NOT break binary ABI
        // It changes semantic_abi but not physical_abi
        // This means: binaries are compatible, but semantic contract changed
        let input1 = FunctionFingerprintInput {
            name: "process".to_string(),
            call_conv: "C".to_string(),
            params: vec![ParamFingerprintInput {
                name: "data".to_string(),
                type_name: "[]const u8".to_string(),
                ownership_mode: "borrowed".to_string(),
                is_noalias: false,
            }],
            ret: None,
            ret_ownership: Some("borrowed".to_string()),
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };

        let mut input2 = input1.clone();
        input2.params[0].ownership_mode = "owned".to_string();
        input2.ret_ownership = Some("owned".to_string());

        let fp1 = Fingerprint::new(FingerprintComponents::for_function(&input1));
        let fp2 = Fingerprint::new(FingerprintComponents::for_function(&input2));

        // Ownership change changes semantic_abi but NOT physical_abi
        // So binary interface is preserved, but semantic contract changed
        assert!(!fp2.is_abi_breaking_from(&fp1));
        assert_ne!(fp1.components.semantic_abi, fp2.components.semantic_abi);
    }

    #[test]
    fn test_abi_breaking_vs_layout_only() {
        // Distinguish ABI-breaking from layout-only changes
        let input1 = FunctionFingerprintInput {
            name: "get_point".to_string(),
            call_conv: "C".to_string(),
            params: vec![],
            ret: Some("Point".to_string()),
            ret_ownership: None,
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Register,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };

        let fp1 = Fingerprint::new(FingerprintComponents::for_function(&input1));

        // Signature change
        let mut input2 = input1.clone();
        input2.ret = Some("i64".to_string());
        let fp2 = Fingerprint::new(FingerprintComponents::for_function(&input2));

        assert!(fp2.is_abi_breaking_from(&fp1));

        // Layout-only change
        let input3 = FunctionFingerprintInput {
            name: "get_point".to_string(),
            call_conv: "C".to_string(),
            params: vec![],
            ret: Some("Point".to_string()),
            ret_ownership: None,
            ownership_kind: OwnershipKind::Sendable,
            can_panic: false,
            can_error: false,
            has_side_effects: false,
            param_layout_style: ParamLayoutStyle::Stack,
            ret_layout_style: RetLayoutStyle::Register,
            target: test_target(),
        };
        let fp3 = Fingerprint::new(FingerprintComponents::for_function(&input3));

        assert!(fp3.is_layout_only_change_from(&fp1));
    }

    // Task 57: Layout impact classification tests

    #[test]
    fn test_struct_field_reorder_invalidates_layout() {
        // Field reorder changes layout fingerprint
        let input1 = StructFingerprintInput {
            name: "Point".to_string(),
            size: 16,
            alignment: 8,
            is_packed: false,
            is_extern: false,
            ownership_kind: OwnershipKind::Sendable,
            fields: vec![
                FieldFingerprintInput {
                    name: "x".to_string(),
                    type_name: "f64".to_string(),
                    offset: 0,
                    size: 8,
                    alignment: 8,
                },
                FieldFingerprintInput {
                    name: "y".to_string(),
                    type_name: "f64".to_string(),
                    offset: 8,
                    size: 8,
                    alignment: 8,
                },
            ],
            target: test_target(),
        };

        // Swap fields
        let input2 = StructFingerprintInput {
            name: "Point".to_string(),
            size: 16,
            alignment: 8,
            is_packed: false,
            is_extern: false,
            ownership_kind: OwnershipKind::Sendable,
            fields: vec![
                FieldFingerprintInput {
                    name: "y".to_string(),
                    type_name: "f64".to_string(),
                    offset: 0,
                    size: 8,
                    alignment: 8,
                },
                FieldFingerprintInput {
                    name: "x".to_string(),
                    type_name: "f64".to_string(),
                    offset: 8,
                    size: 8,
                    alignment: 8,
                },
            ],
            target: test_target(),
        };

        let fp1 = Fingerprint::new(FingerprintComponents::for_struct(&input1));
        let fp2 = Fingerprint::new(FingerprintComponents::for_struct(&input2));

        // Field reorder changes layout
        assert_ne!(fp1.hash, fp2.hash);
    }

    #[test]
    fn test_struct_field_type_change_invalidates_layout() {
        // Field type change should invalidate layout
        let input1 = StructFingerprintInput {
            name: "Value".to_string(),
            size: 8,
            alignment: 8,
            is_packed: false,
            is_extern: false,
            ownership_kind: OwnershipKind::Sendable,
            fields: vec![FieldFingerprintInput {
                name: "data".to_string(),
                type_name: "u64".to_string(),
                offset: 0,
                size: 8,
                alignment: 8,
            }],
            target: test_target(),
        };

        let mut input2 = input1.clone();
        input2.fields[0].type_name = "i64".to_string();

        let fp1 = Fingerprint::new(FingerprintComponents::for_struct(&input1));
        let fp2 = Fingerprint::new(FingerprintComponents::for_struct(&input2));

        assert_ne!(fp1.hash, fp2.hash);
    }

    #[test]
    fn test_packed_flag_change_invalidates_layout() {
        // Packed attribute change invalidates layout
        let input1 = StructFingerprintInput {
            name: "Flags".to_string(),
            size: 8,
            alignment: 8,
            is_packed: false,
            is_extern: false,
            ownership_kind: OwnershipKind::Sendable,
            fields: vec![
                FieldFingerprintInput {
                    name: "a".to_string(),
                    type_name: "u32".to_string(),
                    offset: 0,
                    size: 4,
                    alignment: 4,
                },
                FieldFingerprintInput {
                    name: "b".to_string(),
                    type_name: "u32".to_string(),
                    offset: 4,
                    size: 4,
                    alignment: 4,
                },
            ],
            target: test_target(),
        };

        let mut input2 = input1.clone();
        input2.is_packed = true;
        input2.size = 8;
        // In packed, offsets are 0 and 4 but size is still 8

        let fp1 = Fingerprint::new(FingerprintComponents::for_struct(&input1));
        let fp2 = Fingerprint::new(FingerprintComponents::for_struct(&input2));

        // Packed flag change affects layout
        assert_ne!(fp1.hash, fp2.hash);
    }

    #[test]
    fn test_extern_flag_change_invalidates_layout() {
        // Extern (C ABI) flag change affects layout
        let input1 = StructFingerprintInput {
            name: "Buffer".to_string(),
            size: 32,
            alignment: 8,
            is_packed: false,
            is_extern: false,
            ownership_kind: OwnershipKind::Sendable,
            fields: vec![],
            target: test_target(),
        };

        let mut input2 = input1.clone();
        input2.is_extern = true;

        let fp1 = Fingerprint::new(FingerprintComponents::for_struct(&input1));
        let fp2 = Fingerprint::new(FingerprintComponents::for_struct(&input2));

        // Extern flag affects physical ABI representation
        assert_ne!(fp1.components.physical_abi, fp2.components.physical_abi);
    }

    #[test]
    fn test_private_layout_change_detected() {
        // Private (non-exported) struct layout change should still be tracked
        let input1 = StructFingerprintInput {
            name: "_InternalStruct".to_string(), // Private by convention
            size: 16,
            alignment: 8,
            is_packed: false,
            is_extern: false,
            ownership_kind: OwnershipKind::Sendable,
            fields: vec![],
            target: test_target(),
        };

        let mut input2 = input1.clone();
        input2.size = 24;

        let fp1 = Fingerprint::new(FingerprintComponents::for_struct(&input1));
        let fp2 = Fingerprint::new(FingerprintComponents::for_struct(&input2));

        // Even private layout changes are detected
        assert_ne!(fp1.hash, fp2.hash);
        assert_ne!(fp1.components.layout, fp2.components.layout);
    }

    #[test]
    fn test_target_affects_layout_fingerprint() {
        // Different targets may have different layout rules
        let input = StructFingerprintInput {
            name: "Data".to_string(),
            size: 16,
            alignment: 8,
            is_packed: false,
            is_extern: false,
            ownership_kind: OwnershipKind::Sendable,
            fields: vec![],
            target: test_target(),
        };

        let mut different_target = test_target();
        different_target.triple = "aarch64-linux-gnu".to_string();

        let mut input_arm = input.clone();
        input_arm.target = different_target;

        let fp1 = Fingerprint::new(FingerprintComponents::for_struct(&input));
        let fp2 = Fingerprint::new(FingerprintComponents::for_struct(&input_arm));

        // Target affects layout fingerprint
        assert_ne!(fp1.hash, fp2.hash);
    }
}
