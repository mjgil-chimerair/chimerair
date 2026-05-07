//! `.zchmeta` Chimera metadata schema.

use serde::{Deserialize, Serialize};

/// Magic bytes for `.zchmeta` format.
pub const ZCHMETA_MAGIC: &[u8; 8] = b"ZCHMET01";

/// Current schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// `.zchmeta` header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChmetaHeader {
    pub magic: [u8; 8],
    pub schema_version: u32,
    pub zig_commit: [u8; 20],
    pub target: String,
    pub timestamp_ns: u64,
    pub checksum: [u8; 32],
}

/// Semantic signature for a declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSignature {
    pub decl_id: u64,
    pub name: String,
    pub signature_hash: [u8; 32],
    pub type_id: u64,
    pub params: Vec<ParamSignature>,
    pub return_type: u64,
}

/// Parameter signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamSignature {
    pub name: Option<String>,
    pub type_id: u64,
    pub is_noalias: bool,
}

/// Physical ABI representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalAbi {
    pub size_bytes: u64,
    pub alignment: u32,
    pub memory_repr: MemoryRepresentation,
}

/// How a type is represented in memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryRepresentation {
    Integer { width_bits: u32 },
    Float { width_bits: u32 },
    Pointer,
    Slice,
    Struct { field_offsets: Vec<u64> },
    ErrorUnion,
    Optional,
    Opaque,
}

/// Effect annotation on a function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectAnnotation {
    pub may_error: bool,
    pub may_panic: bool,
    pub may_alloc: bool,
    pub may_dealloc: bool,
    pub may_ffi: bool,
    pub filesystem: bool,
    pub network: bool,
}

/// Ownership and allocator semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipInfo {
    pub allocator_param: Option<u32>,
    pub returns_owned: bool,
    pub borrows: Vec<u32>,
}

/// Layout compatibility check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutCompatibility {
    pub zig_layout_hash: [u8; 32],
    pub chimera_layout_hash: [u8; 32],
    pub compatible: bool,
    pub size_match: bool,
    pub alignment_match: bool,
    pub field_offsets_match: bool,
}

/// Complete `.zchmeta` metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChmetaSchema {
    pub header: ChmetaHeader,
    pub signatures: Vec<SemanticSignature>,
    pub abis: Vec<PhysicalAbi>,
    pub effects: Vec<EffectAnnotation>,
    pub ownership: Vec<OwnershipInfo>,
    pub layout_compat: Vec<LayoutCompatibility>,
}

impl ChmetaSchema {
    pub fn header_magic_valid(&self) -> bool {
        &self.header.magic == ZCHMETA_MAGIC
    }

    pub fn header_version_compatible(&self) -> bool {
        self.header.schema_version <= SCHEMA_VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_header() -> ChmetaHeader {
        ChmetaHeader {
            magic: *ZCHMETA_MAGIC,
            schema_version: SCHEMA_VERSION,
            zig_commit: [0u8; 20],
            target: "x86_64-unknown-linux-gnu".to_string(),
            timestamp_ns: 1234567890,
            checksum: [0u8; 32],
        }
    }

    fn make_test_signature() -> SemanticSignature {
        SemanticSignature {
            decl_id: 1,
            name: "test_func".to_string(),
            signature_hash: [1u8; 32],
            type_id: 2,
            params: vec![
                ParamSignature {
                    name: Some("a".to_string()),
                    type_id: 3,
                    is_noalias: true,
                },
                ParamSignature {
                    name: Some("b".to_string()),
                    type_id: 3,
                    is_noalias: false,
                },
            ],
            return_type: 4,
        }
    }

    fn make_test_abi() -> PhysicalAbi {
        PhysicalAbi {
            size_bytes: 8,
            alignment: 8,
            memory_repr: MemoryRepresentation::Integer { width_bits: 64 },
        }
    }

    fn make_test_effect() -> EffectAnnotation {
        EffectAnnotation {
            may_error: true,
            may_panic: false,
            may_alloc: false,
            may_dealloc: false,
            may_ffi: true,
            filesystem: false,
            network: false,
        }
    }

    fn make_test_ownership() -> OwnershipInfo {
        OwnershipInfo {
            allocator_param: Some(0),
            returns_owned: true,
            borrows: vec![1, 2],
        }
    }

    fn make_test_layout_compat() -> LayoutCompatibility {
        LayoutCompatibility {
            zig_layout_hash: [2u8; 32],
            chimera_layout_hash: [2u8; 32],
            compatible: true,
            size_match: true,
            alignment_match: true,
            field_offsets_match: true,
        }
    }

    #[test]
    fn test_zchmeta_header_magic_valid() {
        let header = make_test_header();
        assert!(header.magic == *ZCHMETA_MAGIC);
    }

    #[test]
    fn test_zchmeta_roundtrip_header() {
        let header = make_test_header();
        let json = serde_json::to_string(&header).unwrap();
        let parsed: ChmetaHeader = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.schema_version, header.schema_version);
        assert_eq!(parsed.target, header.target);
    }

    #[test]
    fn test_zchmeta_roundtrip_signature() {
        let sig = make_test_signature();
        let json = serde_json::to_string(&sig).unwrap();
        let parsed: SemanticSignature = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, sig.name);
        assert_eq!(parsed.params.len(), sig.params.len());
        assert_eq!(parsed.return_type, sig.return_type);
    }

    #[test]
    fn test_zchmeta_roundtrip_physical_abi() {
        let abi = make_test_abi();
        let json = serde_json::to_string(&abi).unwrap();
        let parsed: PhysicalAbi = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.size_bytes, abi.size_bytes);
        assert_eq!(parsed.alignment, abi.alignment);
    }

    #[test]
    fn test_zchmeta_roundtrip_effect_annotation() {
        let effect = make_test_effect();
        let json = serde_json::to_string(&effect).unwrap();
        let parsed: EffectAnnotation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.may_error, effect.may_error);
        assert_eq!(parsed.may_ffi, effect.may_ffi);
    }

    #[test]
    fn test_zchmeta_roundtrip_ownership_info() {
        let ownership = make_test_ownership();
        let json = serde_json::to_string(&ownership).unwrap();
        let parsed: OwnershipInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.allocator_param, ownership.allocator_param);
        assert_eq!(parsed.returns_owned, ownership.returns_owned);
    }

    #[test]
    fn test_zchmeta_roundtrip_layout_compatibility() {
        let compat = make_test_layout_compat();
        let json = serde_json::to_string(&compat).unwrap();
        let parsed: LayoutCompatibility = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.compatible, compat.compatible);
        assert_eq!(parsed.size_match, compat.size_match);
    }

    #[test]
    fn test_zchmeta_roundtrip_full_schema() {
        let schema = ChmetaSchema {
            header: make_test_header(),
            signatures: vec![make_test_signature()],
            abis: vec![make_test_abi()],
            effects: vec![make_test_effect()],
            ownership: vec![make_test_ownership()],
            layout_compat: vec![make_test_layout_compat()],
        };

        let json = serde_json::to_string(&schema).unwrap();
        let parsed: ChmetaSchema = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.header.schema_version, schema.header.schema_version);
        assert_eq!(parsed.signatures.len(), schema.signatures.len());
        assert_eq!(parsed.abis.len(), schema.abis.len());
        assert_eq!(parsed.effects.len(), schema.effects.len());
        assert_eq!(parsed.ownership.len(), schema.ownership.len());
        assert_eq!(parsed.layout_compat.len(), schema.layout_compat.len());
    }

    #[test]
    fn test_zchmeta_schema_magic_valid() {
        let schema = ChmetaSchema {
            header: make_test_header(),
            signatures: vec![],
            abis: vec![],
            effects: vec![],
            ownership: vec![],
            layout_compat: vec![],
        };
        assert!(schema.header_magic_valid());
    }

    #[test]
    fn test_zchmeta_schema_version_compatible() {
        let schema = ChmetaSchema {
            header: make_test_header(),
            signatures: vec![],
            abis: vec![],
            effects: vec![],
            ownership: vec![],
            layout_compat: vec![],
        };
        assert!(schema.header_version_compatible());
    }

    #[test]
    fn test_zchmeta_param_signature_noalias() {
        let param = ParamSignature {
            name: Some("ptr".to_string()),
            type_id: 5,
            is_noalias: true,
        };
        let json = serde_json::to_string(&param).unwrap();
        let parsed: ParamSignature = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_noalias);
    }

    #[test]
    fn test_zchmeta_memory_repr_variants() {
        let variants = vec![
            PhysicalAbi {
                size_bytes: 1,
                alignment: 1,
                memory_repr: MemoryRepresentation::Integer { width_bits: 8 },
            },
            PhysicalAbi {
                size_bytes: 4,
                alignment: 4,
                memory_repr: MemoryRepresentation::Float { width_bits: 32 },
            },
            PhysicalAbi {
                size_bytes: 8,
                alignment: 8,
                memory_repr: MemoryRepresentation::Pointer,
            },
            PhysicalAbi {
                size_bytes: 16,
                alignment: 8,
                memory_repr: MemoryRepresentation::Slice,
            },
            PhysicalAbi {
                size_bytes: 8,
                alignment: 8,
                memory_repr: MemoryRepresentation::Struct {
                    field_offsets: vec![0, 4],
                },
            },
            PhysicalAbi {
                size_bytes: 8,
                alignment: 8,
                memory_repr: MemoryRepresentation::ErrorUnion,
            },
            PhysicalAbi {
                size_bytes: 8,
                alignment: 8,
                memory_repr: MemoryRepresentation::Optional,
            },
            PhysicalAbi {
                size_bytes: 0,
                alignment: 1,
                memory_repr: MemoryRepresentation::Opaque,
            },
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let _parsed: PhysicalAbi = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_zchmeta_effect_annotation_all_flags() {
        let effect = EffectAnnotation {
            may_error: true,
            may_panic: true,
            may_alloc: true,
            may_dealloc: true,
            may_ffi: true,
            filesystem: true,
            network: true,
        };
        let json = serde_json::to_string(&effect).unwrap();
        let parsed: EffectAnnotation = serde_json::from_str(&json).unwrap();
        assert!(parsed.may_error);
        assert!(parsed.may_panic);
        assert!(parsed.may_alloc);
        assert!(parsed.may_dealloc);
        assert!(parsed.may_ffi);
        assert!(parsed.filesystem);
        assert!(parsed.network);
    }

    #[test]
    fn test_zchmeta_layout_compat_all_flags_false() {
        let compat = LayoutCompatibility {
            zig_layout_hash: [3u8; 32],
            chimera_layout_hash: [4u8; 32],
            compatible: false,
            size_match: false,
            alignment_match: false,
            field_offsets_match: false,
        };
        let json = serde_json::to_string(&compat).unwrap();
        let parsed: LayoutCompatibility = serde_json::from_str(&json).unwrap();
        assert!(!parsed.compatible);
        assert!(!parsed.size_match);
        assert!(!parsed.alignment_match);
        assert!(!parsed.field_offsets_match);
    }

    #[test]
    fn test_zchmeta_ownership_no_allocator() {
        let ownership = OwnershipInfo {
            allocator_param: None,
            returns_owned: false,
            borrows: vec![],
        };
        let json = serde_json::to_string(&ownership).unwrap();
        let parsed: OwnershipInfo = serde_json::from_str(&json).unwrap();
        assert!(parsed.allocator_param.is_none());
        assert!(!parsed.returns_owned);
        assert!(parsed.borrows.is_empty());
    }
}
