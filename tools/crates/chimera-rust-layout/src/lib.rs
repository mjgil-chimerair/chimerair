//! Chimera Rust Layout Extraction
//!
//! Extracts memory layout information from Rust types:
//! - Size and alignment
//! - Field offsets
//! - Enum discriminant placement
//! - Niche optimization encoding
//! - Transparent wrappers
//! - Target-specific layout

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ABI kind for a type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbiKind {
    Scalar,
    Vector,
    Aggregate,
    Uninhabited,
    NoAlias,
}

impl Default for AbiKind {
    fn default() -> Self {
        AbiKind::Scalar
    }
}

/// Field layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLayout {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub alignment: u64,
}

/// Struct/tuple fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fields(pub Vec<FieldLayout>);

/// Variant layout for enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantLayout {
    pub name: String,
    pub discriminant: u64,
    pub offset: u64,
    pub size_bytes: u64,
    pub alignment: u64,
}

/// Niche layout for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheLayout {
    pub offset: u64,
    pub size: u64,
    pub valid_range_start: u64,
    pub valid_range_end: u64,
}

/// Transparent wrapper info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparentWrapper {
    pub inner_type: String,
}

/// A complete layout fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutFact {
    pub stable_id: String,
    pub size_bytes: u64,
    pub alignment_bytes: u64,
    pub abi_kind: AbiKind,
    pub fields: Option<Fields>,
    pub variants: Option<Vec<VariantLayout>>,
    pub niche: Option<NicheLayout>,
    pub transparent_wrapper: Option<TransparentWrapper>,
    pub target_dependent: bool,
}

impl LayoutFact {
    /// Compute layout fingerprint (Task 145)
    /// Hashes: type kind, repr, fields, offsets, size, align, enum variants,
    ///         niches, target, rustc version
    pub fn compute_full_fingerprint(
        &self,
        type_kind: &str,
        repr: &str,
        target: &str,
        rustc_version: &str,
    ) -> String {
        let mut hasher = Hasher::new();

        // Type kind
        hasher.update(type_kind.as_bytes());

        // Repr
        hasher.update(repr.as_bytes());

        // Stable ID
        hasher.update(self.stable_id.as_bytes());

        // Size and alignment
        hasher.update(&self.size_bytes.to_le_bytes());
        hasher.update(&self.alignment_bytes.to_le_bytes());

        // ABI kind
        hasher.update(format!("{:?}", self.abi_kind).as_bytes());

        // Target dependent flag
        hasher.update(if self.target_dependent {
            b"target_dep"
        } else {
            b"stable"
        });

        // Hash fields if present
        if let Some(Fields(ref fields)) = self.fields {
            hasher.update(b"fields");
            hasher.update(&(fields.len() as u64).to_le_bytes());
            for f in fields {
                hasher.update(f.name.as_bytes());
                hasher.update(&f.offset.to_le_bytes());
                hasher.update(&f.size.to_le_bytes());
                hasher.update(&f.alignment.to_le_bytes());
            }
        }

        // Hash variants if present
        if let Some(ref variants) = self.variants {
            hasher.update(b"variants");
            hasher.update(&(variants.len() as u64).to_le_bytes());
            for v in variants {
                hasher.update(v.name.as_bytes());
                hasher.update(&v.discriminant.to_le_bytes());
                hasher.update(&v.offset.to_le_bytes());
                hasher.update(&v.size_bytes.to_le_bytes());
                hasher.update(&v.alignment.to_le_bytes());
            }
        }

        // Hash niche if present
        if let Some(ref niche) = self.niche {
            hasher.update(b"niche");
            hasher.update(&niche.offset.to_le_bytes());
            hasher.update(&niche.size.to_le_bytes());
            hasher.update(&niche.valid_range_start.to_le_bytes());
            hasher.update(&niche.valid_range_end.to_le_bytes());
        }

        // Target
        hasher.update(target.as_bytes());

        // Rustc version
        hasher.update(rustc_version.as_bytes());

        hasher.finalize().to_hex().to_string()
    }

    /// Legacy fingerprint for backward compatibility
    pub fn fingerprint(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.stable_id.hash(&mut hasher);
        self.size_bytes.hash(&mut hasher);
        self.alignment_bytes.hash(&mut hasher);
        format!("{:?}", self.abi_kind).hash(&mut hasher);
        self.target_dependent.hash(&mut hasher);

        if let Some(Fields(ref fields)) = self.fields {
            fields.len().hash(&mut hasher);
            for f in fields {
                f.name.hash(&mut hasher);
                f.offset.hash(&mut hasher);
                f.size.hash(&mut hasher);
                f.alignment.hash(&mut hasher);
            }
        }

        if let Some(ref variants) = self.variants {
            variants.len().hash(&mut hasher);
            for v in variants {
                v.name.hash(&mut hasher);
                v.discriminant.hash(&mut hasher);
                v.size_bytes.hash(&mut hasher);
                v.alignment.hash(&mut hasher);
            }
        }

        format!("{:x}", hasher.finish())
    }
}

/// Layout extractor for Rust types
#[derive(Debug, Clone, Default)]
pub struct LayoutExtractor {
    layouts: Vec<LayoutFact>,
    primitives: HashMap<String, LayoutFact>,
}

impl LayoutExtractor {
    /// Create a new layout extractor
    pub fn new() -> Self {
        Self {
            layouts: Vec::new(),
            primitives: HashMap::new(),
        }
    }

    /// Record a primitive type layout
    pub fn primitive(&mut self, name: &str, size: u64, align: u64, abi: AbiKind) {
        let layout = LayoutFact {
            stable_id: format!("lay_{}", name),
            size_bytes: size,
            alignment_bytes: align,
            abi_kind: abi,
            fields: None,
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };
        self.primitives.insert(name.to_string(), layout.clone());
        self.layouts.push(layout);
    }

    /// Record a struct layout
    pub fn record_struct(
        &mut self,
        stable_id: String,
        size: u64,
        align: u64,
        fields: Vec<FieldLayout>,
    ) {
        self.layouts.push(LayoutFact {
            stable_id,
            size_bytes: size,
            alignment_bytes: align,
            abi_kind: AbiKind::Scalar,
            fields: Some(Fields(fields)),
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        });
    }

    /// Record a tuple layout
    pub fn record_tuple(&mut self, stable_id: String, fields: Vec<FieldLayout>) {
        let size = fields
            .iter()
            .map(|f| f.offset.saturating_add(f.size))
            .max()
            .unwrap_or(0);
        let align = fields.iter().map(|f| f.alignment).max().unwrap_or(1);
        self.layouts.push(LayoutFact {
            stable_id,
            size_bytes: size,
            alignment_bytes: align,
            abi_kind: AbiKind::Aggregate,
            fields: Some(Fields(fields)),
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        });
    }

    /// Record an enum layout
    pub fn record_enum(
        &mut self,
        stable_id: String,
        variants: Vec<VariantLayout>,
        niche: Option<NicheLayout>,
    ) {
        let size = variants.iter().map(|v| v.size_bytes).max().unwrap_or(1);
        let align = variants.iter().map(|v| v.alignment).max().unwrap_or(1);
        self.layouts.push(LayoutFact {
            stable_id,
            size_bytes: size,
            alignment_bytes: align,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: Some(variants),
            niche,
            transparent_wrapper: None,
            target_dependent: false,
        });
    }

    /// Record a transparent wrapper
    pub fn transparent(&mut self, stable_id: String, inner: String) {
        self.layouts.push(LayoutFact {
            stable_id,
            size_bytes: 0,
            alignment_bytes: 1,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: None,
            niche: None,
            transparent_wrapper: Some(TransparentWrapper { inner_type: inner }),
            target_dependent: false,
        });
    }

    /// Get all layouts
    pub fn layouts(&self) -> &[LayoutFact] {
        &self.layouts
    }

    /// Find layout by stable ID
    pub fn find(&self, stable_id: &str) -> Option<&LayoutFact> {
        self.layouts.iter().find(|l| l.stable_id == stable_id)
    }

    /// Verify layout consistency
    pub fn verify_padding(&self, stable_id: &str) -> Result<(), LayoutError> {
        let lay = self.layouts.iter().find(|l| l.stable_id == stable_id);
        if let Some(lay) = lay {
            if let Some(Fields(fields)) = &lay.fields {
                for (i, field) in fields.iter().enumerate() {
                    // Check alignment
                    if field.offset % field.alignment != 0 {
                        return Err(LayoutError::MisalignedField {
                            struct_id: stable_id.to_string(),
                            field_index: i,
                            offset: field.offset,
                            required_align: field.alignment,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Count layouts
    pub fn len(&self) -> usize {
        self.layouts.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.layouts.is_empty()
    }

    /// Record a union layout
    pub fn record_union(&mut self, stable_id: String, variants: Vec<VariantLayout>) {
        let size = variants.iter().map(|v| v.size_bytes).max().unwrap_or(1);
        let align = variants.iter().map(|v| v.alignment).max().unwrap_or(1);
        self.layouts.push(LayoutFact {
            stable_id,
            size_bytes: size,
            alignment_bytes: align,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: Some(variants),
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        });
    }

    /// Record layout with niche (C-like enum optimization)
    pub fn record_with_niche(
        &mut self,
        stable_id: String,
        variants: Vec<VariantLayout>,
        niche: NicheLayout,
    ) {
        let size = variants.iter().map(|v| v.size_bytes).max().unwrap_or(1);
        let align = variants.iter().map(|v| v.alignment).max().unwrap_or(1);
        self.layouts.push(LayoutFact {
            stable_id,
            size_bytes: size,
            alignment_bytes: align,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: Some(variants),
            niche: Some(niche),
            transparent_wrapper: None,
            target_dependent: false,
        });
    }

    /// Build from schema LayoutDef
    pub fn from_schema_layouts(&mut self, layouts: Vec<(String, u64, u32, bool)>) {
        for (name, size, align, target_dep) in layouts {
            self.layouts.push(LayoutFact {
                stable_id: name,
                size_bytes: size,
                alignment_bytes: align as u64,
                abi_kind: AbiKind::Scalar,
                fields: None,
                variants: None,
                niche: None,
                transparent_wrapper: None,
                target_dependent: target_dep,
            });
        }
    }

    /// Get all primitive layouts for a target
    pub fn get_primitives_for_target(&self, target: &str) -> Vec<&LayoutFact> {
        self.layouts
            .iter()
            .filter(|l| l.stable_id.starts_with("lay_") && !l.target_dependent)
            .collect()
    }

    /// Verify layout consistency with struct field ordering
    pub fn verify_field_ordering(&self, stable_id: &str) -> Result<(), LayoutError> {
        let lay = self.layouts.iter().find(|l| l.stable_id == stable_id);
        if let Some(lay) = lay {
            if let Some(Fields(fields)) = &lay.fields {
                for i in 1..fields.len() {
                    if fields[i].offset < fields[i - 1].offset {
                        return Err(LayoutError::OverlappingFields(stable_id.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

/// Layout extraction errors
#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("misaligned field {field_index} in {struct_id}: offset {offset} not aligned to {required_align}")]
    MisalignedField {
        struct_id: String,
        field_index: usize,
        offset: u64,
        required_align: u64,
    },
    #[error("invalid variant discriminant: {0}")]
    InvalidDiscriminant(String),
    #[error("overlapping fields in {0}")]
    OverlappingFields(String),
}

/// Compare two layouts for equality
pub fn layouts_equal(a: &LayoutFact, b: &LayoutFact) -> bool {
    a.size_bytes == b.size_bytes && a.alignment_bytes == b.alignment_bytes
}

/// Emit layouts to JSON
pub fn emit_layouts_json(extractor: &LayoutExtractor) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&extractor.layouts)
}

/// Parse layouts from JSON
pub fn parse_layouts_json(json: &str) -> Result<Vec<LayoutFact>, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_layout() {
        let mut extractor = LayoutExtractor::new();
        extractor.primitive("i32", 4, 4, AbiKind::Scalar);

        let layouts = extractor.layouts();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].size_bytes, 4);
    }

    #[test]
    fn test_struct_layout() {
        let mut extractor = LayoutExtractor::new();
        extractor.record_struct(
            "MyStruct".to_string(),
            16,
            8,
            vec![
                FieldLayout {
                    name: "a".to_string(),
                    offset: 0,
                    size: 8,
                    alignment: 8,
                },
                FieldLayout {
                    name: "b".to_string(),
                    offset: 8,
                    size: 8,
                    alignment: 8,
                },
            ],
        );

        assert!(extractor.verify_padding("MyStruct").is_ok());
    }

    #[test]
    fn test_enum_layout() {
        let mut extractor = LayoutExtractor::new();
        extractor.record_enum(
            "MyEnum".to_string(),
            vec![
                VariantLayout {
                    name: "A".to_string(),
                    discriminant: 0,
                    offset: 0,
                    size_bytes: 8,
                    alignment: 8,
                },
                VariantLayout {
                    name: "B".to_string(),
                    discriminant: 1,
                    offset: 0,
                    size_bytes: 8,
                    alignment: 8,
                },
            ],
            None,
        );

        let layouts = extractor.layouts();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].variants.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_transparent_wrapper() {
        let mut extractor = LayoutExtractor::new();
        extractor.transparent("MyWrapper".to_string(), "InnerType".to_string());

        let layouts = extractor.layouts();
        assert!(layouts[0].transparent_wrapper.is_some());
    }

    #[test]
    fn test_tuple_layout() {
        let mut extractor = LayoutExtractor::new();
        extractor.record_tuple(
            "MyTuple".to_string(),
            vec![
                FieldLayout {
                    name: "0".to_string(),
                    offset: 0,
                    size: 4,
                    alignment: 4,
                },
                FieldLayout {
                    name: "1".to_string(),
                    offset: 4,
                    size: 8,
                    alignment: 8,
                },
            ],
        );

        let layouts = extractor.layouts();
        assert_eq!(layouts[0].size_bytes, 12); // 4 + 8 = 12
        assert_eq!(layouts[0].alignment_bytes, 8); // max(4, 8) = 8
    }

    #[test]
    fn test_find_layout() {
        let mut extractor = LayoutExtractor::new();
        extractor.primitive("i64", 8, 8, AbiKind::Scalar);

        assert!(extractor.find("lay_i64").is_some());
        assert!(extractor.find("not_exist").is_none());
    }

    #[test]
    fn test_misaligned_field() {
        let mut extractor = LayoutExtractor::new();
        // Field at offset 1, alignment 2 should fail verification
        extractor.record_struct(
            "BadStruct".to_string(),
            8,
            8,
            vec![FieldLayout {
                name: "a".to_string(),
                offset: 1,
                size: 4,
                alignment: 2,
            }],
        );

        assert!(extractor.verify_padding("BadStruct").is_err());
    }

    #[test]
    fn test_abi_kind_default() {
        assert!(matches!(AbiKind::default(), AbiKind::Scalar));
    }

    #[test]
    fn test_roundtrip_json() {
        let mut extractor = LayoutExtractor::new();
        extractor.primitive("u8", 1, 1, AbiKind::Scalar);

        let json = emit_layouts_json(&extractor).unwrap();
        let parsed = parse_layouts_json(&json).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].size_bytes, 1);
    }

    // Task 145: Layout fingerprint tests

    #[test]
    fn test_layout_fingerprint_struct() {
        let fact = LayoutFact {
            stable_id: "MyStruct".to_string(),
            size_bytes: 8,
            alignment_bytes: 4,
            abi_kind: AbiKind::Aggregate,
            fields: Some(Fields(vec![
                FieldLayout {
                    name: "a".to_string(),
                    offset: 0,
                    size: 4,
                    alignment: 4,
                },
                FieldLayout {
                    name: "b".to_string(),
                    offset: 4,
                    size: 4,
                    alignment: 4,
                },
            ])),
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fp = fact.fingerprint();
        assert!(!fp.is_empty());
        assert_eq!(fp.len(), 16); // 64-bit hex
    }

    #[test]
    fn test_layout_fingerprint_different_for_different_layouts() {
        let fact1 = LayoutFact {
            stable_id: "StructA".to_string(),
            size_bytes: 8,
            alignment_bytes: 4,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fact2 = LayoutFact {
            stable_id: "StructB".to_string(),
            size_bytes: 16,
            alignment_bytes: 8,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        assert_ne!(fact1.fingerprint(), fact2.fingerprint());
    }

    #[test]
    fn test_layout_fingerprint_enum_with_variants() {
        let fact = LayoutFact {
            stable_id: "MyEnum".to_string(),
            size_bytes: 4,
            alignment_bytes: 4,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: Some(vec![
                VariantLayout {
                    name: "A".to_string(),
                    discriminant: 0,
                    offset: 0,
                    size_bytes: 4,
                    alignment: 4,
                },
                VariantLayout {
                    name: "B".to_string(),
                    discriminant: 1,
                    offset: 0,
                    size_bytes: 4,
                    alignment: 4,
                },
            ]),
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fp = fact.fingerprint();
        assert!(!fp.is_empty());
    }

    // Task 145: Layout fingerprint tests

    #[test]
    fn test_layout_full_fingerprint_struct() {
        let fact = LayoutFact {
            stable_id: "MyStruct".to_string(),
            size_bytes: 8,
            alignment_bytes: 4,
            abi_kind: AbiKind::Aggregate,
            fields: Some(Fields(vec![
                FieldLayout {
                    name: "a".to_string(),
                    offset: 0,
                    size: 4,
                    alignment: 4,
                },
                FieldLayout {
                    name: "b".to_string(),
                    offset: 4,
                    size: 4,
                    alignment: 4,
                },
            ])),
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fp = fact.compute_full_fingerprint("struct", "C", "x86_64-unknown-linux-gnu", "1.0.0");
        assert!(!fp.is_empty());
        assert_eq!(fp.len(), 64); // blake3 hex length
    }

    #[test]
    fn test_layout_full_fingerprint_deterministic() {
        let fact1 = LayoutFact {
            stable_id: "MyStruct".to_string(),
            size_bytes: 8,
            alignment_bytes: 4,
            abi_kind: AbiKind::Aggregate,
            fields: Some(Fields(vec![FieldLayout {
                name: "a".to_string(),
                offset: 0,
                size: 4,
                alignment: 4,
            }])),
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fact2 = LayoutFact {
            stable_id: "MyStruct".to_string(),
            size_bytes: 8,
            alignment_bytes: 4,
            abi_kind: AbiKind::Aggregate,
            fields: Some(Fields(vec![FieldLayout {
                name: "a".to_string(),
                offset: 0,
                size: 4,
                alignment: 4,
            }])),
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fp1 =
            fact1.compute_full_fingerprint("struct", "C", "x86_64-unknown-linux-gnu", "1.0.0");
        let fp2 =
            fact2.compute_full_fingerprint("struct", "C", "x86_64-unknown-linux-gnu", "1.0.0");
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_layout_full_fingerprint_changes_with_size() {
        let fact1 = LayoutFact {
            stable_id: "MyStruct".to_string(),
            size_bytes: 8,
            alignment_bytes: 4,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fact2 = LayoutFact {
            stable_id: "MyStruct".to_string(),
            size_bytes: 16,
            alignment_bytes: 4,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fp1 =
            fact1.compute_full_fingerprint("struct", "C", "x86_64-unknown-linux-gnu", "1.0.0");
        let fp2 =
            fact2.compute_full_fingerprint("struct", "C", "x86_64-unknown-linux-gnu", "1.0.0");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_layout_full_fingerprint_changes_with_repr() {
        let fact = LayoutFact {
            stable_id: "MyStruct".to_string(),
            size_bytes: 8,
            alignment_bytes: 4,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fp1 = fact.compute_full_fingerprint("struct", "C", "x86_64-unknown-linux-gnu", "1.0.0");
        let fp2 = fact.compute_full_fingerprint(
            "struct",
            "transparent",
            "x86_64-unknown-linux-gnu",
            "1.0.0",
        );
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_layout_full_fingerprint_changes_with_target() {
        let fact = LayoutFact {
            stable_id: "MyStruct".to_string(),
            size_bytes: 8,
            alignment_bytes: 4,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fp1 = fact.compute_full_fingerprint("struct", "C", "x86_64-unknown-linux-gnu", "1.0.0");
        let fp2 = fact.compute_full_fingerprint("struct", "C", "aarch64-apple-darwin", "1.0.0");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_layout_full_fingerprint_changes_with_rustc_version() {
        let fact = LayoutFact {
            stable_id: "MyStruct".to_string(),
            size_bytes: 8,
            alignment_bytes: 4,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fp1 = fact.compute_full_fingerprint("struct", "C", "x86_64-unknown-linux-gnu", "1.0.0");
        let fp2 = fact.compute_full_fingerprint("struct", "C", "x86_64-unknown-linux-gnu", "1.1.0");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_layout_full_fingerprint_with_niche() {
        let fact = LayoutFact {
            stable_id: "MyEnum".to_string(),
            size_bytes: 4,
            alignment_bytes: 4,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: Some(vec![
                VariantLayout {
                    name: "A".to_string(),
                    discriminant: 0,
                    offset: 0,
                    size_bytes: 4,
                    alignment: 4,
                },
                VariantLayout {
                    name: "B".to_string(),
                    discriminant: 1,
                    offset: 0,
                    size_bytes: 4,
                    alignment: 4,
                },
            ]),
            niche: Some(NicheLayout {
                offset: 2,
                size: 2,
                valid_range_start: 0x80000000,
                valid_range_end: 0xFFFFFFFF,
            }),
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fp = fact.compute_full_fingerprint("enum", "C", "x86_64-unknown-linux-gnu", "1.0.0");
        assert!(!fp.is_empty());
    }

    #[test]
    fn test_layout_full_fingerprint_with_fields() {
        let fact = LayoutFact {
            stable_id: "MyStruct".to_string(),
            size_bytes: 16,
            alignment_bytes: 8,
            abi_kind: AbiKind::Aggregate,
            fields: Some(Fields(vec![
                FieldLayout {
                    name: "x".to_string(),
                    offset: 0,
                    size: 4,
                    alignment: 4,
                },
                FieldLayout {
                    name: "y".to_string(),
                    offset: 4,
                    size: 8,
                    alignment: 8,
                },
                FieldLayout {
                    name: "z".to_string(),
                    offset: 12,
                    size: 4,
                    alignment: 4,
                },
            ])),
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fp = fact.compute_full_fingerprint("struct", "C", "x86_64-unknown-linux-gnu", "1.0.0");
        assert!(!fp.is_empty());
    }

    #[test]
    fn test_layout_full_fingerprint_target_dependent_flag() {
        let fact1 = LayoutFact {
            stable_id: "MyStruct".to_string(),
            size_bytes: 8,
            alignment_bytes: 4,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: false,
        };

        let fact2 = LayoutFact {
            stable_id: "MyStruct".to_string(),
            size_bytes: 8,
            alignment_bytes: 4,
            abi_kind: AbiKind::Scalar,
            fields: None,
            variants: None,
            niche: None,
            transparent_wrapper: None,
            target_dependent: true,
        };

        let fp1 =
            fact1.compute_full_fingerprint("struct", "C", "x86_64-unknown-linux-gnu", "1.0.0");
        let fp2 =
            fact2.compute_full_fingerprint("struct", "C", "x86_64-unknown-linux-gnu", "1.0.0");
        assert_ne!(fp1, fp2);
    }
}
