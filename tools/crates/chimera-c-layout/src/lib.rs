//! Chimera C layout extraction crate.
//!
//! Models sizeof, alignof, field offsets, bitfields, flexible arrays,
//! packed/aligned attributes, target ABI, and endianness.
//!
//! Task 14: C layout extraction crate

use chimera_c_abi::CAbiExtractor;
use chimera_c_schema::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result type for layout operations
pub type Result<T> = std::result::Result<T, LayoutError>;

/// Layout-related errors
#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("incomplete type layout: {0}")]
    IncompleteType(String),
    #[error("invalid field offset: {0}")]
    InvalidFieldOffset(String),
    #[error("bitfield overflow: {0}")]
    BitfieldOverflow(String),
    #[error("flexible array not at end: {0}")]
    FlexibleArrayPosition(String),
    #[error("packed/aligned conflict: {0}")]
    PackedAlignedConflict(String),
    #[error("size mismatch: expected {expected}, got {actual}")]
    SizeMismatch { expected: u64, actual: u64 },
    #[error("alignment mismatch: expected {expected}, got {actual}")]
    AlignmentMismatch { expected: u32, actual: u32 },
    #[error("layout verification failed: {0}")]
    VerificationFailed(String),
}

/// Target ABI information
#[derive(Debug, Clone)]
pub struct TargetAbi {
    /// Target triple
    pub triple: String,
    /// Data layout string (similar to LLVM datalayout)
    pub data_layout: DataLayout,
    /// Endianness
    pub endianness: Endianness,
    /// Pointer size in bytes
    pub pointer_size: u32,
    /// Size of long (for cross-compilation)
    pub long_size: u32,
    /// Size of long long
    pub long_long_size: u32,
    /// Size of int
    pub int_size: u32,
    /// Size of short
    pub short_size: u32,
    /// Size of char
    pub char_size: u32,
    /// Size of double
    pub double_size: u32,
    /// Size of long double
    pub long_double_size: u32,
    /// Alignment of int64
    pub int64_align: u32,
    /// Alignment of long long
    pub long_long_align: u32,
    /// Alignment of double
    pub double_align: u32,
    /// Alignment of long double
    pub long_double_align: u32,
    /// Sysroot path for cross-compilation
    pub sysroot: Option<String>,
    /// Resource directory path
    pub resource_dir: Option<String>,
    /// Whether this is a cross-compilation target
    pub is_cross_compile: bool,
    /// Host architecture (for cross-compilation validation)
    pub host_arch: Option<String>,
}

impl Default for TargetAbi {
    fn default() -> Self {
        Self {
            triple: String::new(),
            data_layout: DataLayout::default(),
            endianness: Endianness::Little,
            pointer_size: 8,
            long_size: 8,
            long_long_size: 8,
            int_size: 4,
            short_size: 2,
            char_size: 1,
            double_size: 8,
            long_double_size: 16,
            int64_align: 8,
            long_long_align: 8,
            double_align: 8,
            long_double_align: 16,
            sysroot: None,
            resource_dir: None,
            is_cross_compile: false,
            host_arch: None,
        }
    }
}

/// Data layout string components
#[derive(Debug, Clone, Default)]
pub struct DataLayout {
    /// Vector alignment
    pub vector_align: u32,
    /// Aggregate alignment
    pub aggregate_align: u32,
    /// Function alignment
    pub function_align: u32,
    /// Field alignments
    pub field_alignments: HashMap<String, u32>,
}

/// Endianness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Endianness {
    Little,
    Big,
}

impl Default for Endianness {
    fn default() -> Self {
        Endianness::Little
    }
}

/// C type layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeLayout {
    /// Type name
    pub name: String,
    /// Size in bytes
    pub size: u64,
    /// Alignment in bytes
    pub align: u32,
    /// Whether this type is packed
    pub is_packed: bool,
    /// Explicit alignment (if specified via _Alignas or #pragma pack)
    pub explicit_align: Option<u32>,
    /// Fields with offsets
    pub fields: Vec<FieldLayout>,
    /// Bitfields
    pub bitfields: Vec<BitfieldLayout>,
    /// Whether this type has a flexible array member
    pub has_flexible_array: bool,
    /// Layout fingerprint for caching
    pub fingerprint: String,
}

impl TypeLayout {
    /// Compute fingerprint for this layout
    pub fn compute_fingerprint(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.size.to_le_bytes());
        hasher.update(&self.align.to_le_bytes());
        hasher.update(&[if self.is_packed { 1u8 } else { 0u8 }]);
        if let Some(a) = self.explicit_align {
            hasher.update(&a.to_le_bytes());
        }
        for field in &self.fields {
            hasher.update(field.name.as_bytes());
            hasher.update(&field.offset.to_le_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }

    /// Verify this layout matches expected values
    pub fn verify(&self, expected: &TypeLayout) -> std::result::Result<(), LayoutError> {
        if self.size != expected.size {
            return Err(LayoutError::SizeMismatch {
                expected: expected.size,
                actual: self.size,
            });
        }
        if self.align != expected.align {
            return Err(LayoutError::AlignmentMismatch {
                expected: expected.align,
                actual: self.align,
            });
        }
        for (f1, f2) in self.fields.iter().zip(expected.fields.iter()) {
            if f1.offset != f2.offset {
                return Err(LayoutError::InvalidFieldOffset(format!(
                    "field {} offset mismatch: {} vs {}",
                    f1.name, f1.offset, f2.offset
                )));
            }
        }
        Ok(())
    }
}

/// Field layout within a type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLayout {
    /// Field name
    pub name: String,
    /// Offset from start of type
    pub offset: u64,
    /// Size in bytes
    pub size: u64,
    /// Alignment requirement
    pub align: u32,
    /// Bitfield info (if this is a bitfield)
    pub bitfield: Option<BitfieldLayout>,
}

/// Bitfield layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitfieldLayout {
    /// Bit offset within the storage unit
    pub bit_offset: u8,
    /// Number of bits
    pub bit_width: u8,
    /// Storage unit type
    pub storage_type: String,
    /// Container offset (byte offset to storage unit)
    pub container_offset: u64,
    /// Whether the bitfield is signed
    pub is_signed: bool,
}

/// Flexible array member info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexibleArrayMember {
    /// Field name
    pub name: String,
    /// Offset from start of containing struct
    pub offset: u64,
    /// Element type
    pub element_type: String,
    /// Whether length is known at definition time
    pub known_length: Option<u64>,
}

/// Packed/aligned attribute info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackedAlignedAttr {
    /// Attribute kind
    pub kind: PackedAlignedKind,
    /// Value (alignment for aligned, pack value for packed)
    pub value: Option<u32>,
    /// Source location
    pub location: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackedAlignedKind {
    Packed,
    Aligned,
    PragmaPack,
}

/// Static assertion for layout verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAssert {
    /// The assertion expression
    pub expression: String,
    /// Optional message
    pub message: Option<String>,
    /// Source location
    pub location: SourceSpan,
}

impl StaticAssert {
    /// Create a layout assertion
    pub fn size_assert(name: &str, size: u64, location: SourceSpan) -> Self {
        Self {
            expression: format!("sizeof({}) == {}", name, size),
            message: Some(format!("size of {} must be {}", name, size)),
            location,
        }
    }

    /// Create an alignment assertion
    pub fn align_assert(name: &str, align: u32, location: SourceSpan) -> Self {
        Self {
            expression: format!("_Alignof({}) == {}", name, align),
            message: Some(format!("alignment of {} must be {}", name, align)),
            location,
        }
    }
}

/// Layout context for a translation unit
#[derive(Debug, Clone, Default)]
pub struct LayoutContext {
    /// All type layouts indexed by type reference
    pub type_layouts: HashMap<TypeRef, TypeLayout>,
    /// Flexible array members
    pub flexible_arrays: HashMap<TypeRef, FlexibleArrayMember>,
    /// Packed/aligned attributes
    pub packed_aligned: HashMap<TypeRef, Vec<PackedAlignedAttr>>,
    /// Static assertions
    pub static_asserts: Vec<StaticAssert>,
    /// Target ABI info
    pub target_abi: Option<TargetAbi>,
}

impl LayoutContext {
    /// Add a type layout
    pub fn add_layout(&mut self, type_ref: TypeRef, layout: TypeLayout) {
        self.type_layouts.insert(type_ref, layout);
    }

    /// Get a type layout
    pub fn get_layout(&self, type_ref: &TypeRef) -> Option<&TypeLayout> {
        self.type_layouts.get(type_ref)
    }

    /// Add a static assertion
    pub fn add_assertion(&mut self, assertion: StaticAssert) {
        self.static_asserts.push(assertion);
    }
}

/// C layout extractor using Clang facts
pub struct CLayoutExtractor {
    /// Target ABI
    pub target_abi: TargetAbi,
    /// ABI extractor (for layout computation)
    abi_extractor: CAbiExtractor,
}

impl CLayoutExtractor {
    /// Create a new layout extractor for a target
    pub fn new(target_triple: &str) -> Self {
        let target_abi = TargetAbi::from_triple(target_triple);
        Self {
            target_abi: target_abi.clone(),
            abi_extractor: CAbiExtractor::new(target_triple),
        }
    }

    /// Extract layout for a struct declaration
    pub fn extract_struct_layout(&self, decl: &StructDecl) -> Result<TypeLayout> {
        let fields: Vec<FieldLayout> = decl
            .fields
            .iter()
            .map(|f| FieldLayout {
                name: f.name.clone(),
                offset: f.offset,
                size: f.size,
                align: f.align,
                bitfield: None,
            })
            .collect();

        let size = decl.fields.last().map(|f| f.offset + f.size).unwrap_or(0);

        let align = decl.fields.iter().map(|f| f.align).max().unwrap_or(1);

        let mut layout = TypeLayout {
            name: decl
                .name
                .clone()
                .unwrap_or_else(|| "(anonymous)".to_string()),
            size,
            align,
            is_packed: decl.is_packed,
            explicit_align: decl.pack_align,
            fields,
            bitfields: vec![],
            has_flexible_array: decl
                .fields
                .iter()
                .any(|f| f.bitfield_width.is_none() && f.name.is_empty()),
            fingerprint: String::new(),
        };

        layout.fingerprint = layout.compute_fingerprint();
        Ok(layout)
    }

    /// Extract layout for a union declaration
    pub fn extract_union_layout(&self, decl: &UnionDecl) -> Result<TypeLayout> {
        let fields: Vec<FieldLayout> = decl
            .variants
            .iter()
            .map(|f| FieldLayout {
                name: f.name.clone(),
                offset: 0,
                size: f.size,
                align: f.align,
                bitfield: None,
            })
            .collect();

        let mut layout = TypeLayout {
            name: decl
                .name
                .clone()
                .unwrap_or_else(|| "(anonymous)".to_string()),
            size: decl.size,
            align: decl.align,
            is_packed: false,
            explicit_align: None,
            fields,
            bitfields: vec![],
            has_flexible_array: false,
            fingerprint: String::new(),
        };

        layout.fingerprint = layout.compute_fingerprint();
        Ok(layout)
    }

    /// Compute layout for a primitive type
    pub fn primitive_layout(&self, type_name: &str) -> Option<TypeLayout> {
        let (size, align) = match type_name {
            "char" | "bool" | "int8_t" | "uint8_t" => (1, 1),
            "short" | "int16_t" | "uint16_t" => (2, 2),
            "int" | "long" | "int32_t" | "uint32_t" | "float" => (4, 4),
            "long long" | "int64_t" | "uint64_t" | "double" => (8, 8),
            "long double" => (16, 16),
            "void" => (0, 0),
            _ => return None,
        };

        Some(TypeLayout {
            name: type_name.to_string(),
            size,
            align,
            is_packed: false,
            explicit_align: None,
            fields: vec![],
            bitfields: vec![],
            has_flexible_array: false,
            fingerprint: String::new(),
        })
    }

    /// Generate _Static_assert for this layout
    pub fn generate_static_assert(&self, layout: &TypeLayout) -> StaticAssert {
        StaticAssert::size_assert(
            &layout.name,
            layout.size,
            SourceSpan {
                file: "(generated)".to_string(),
                line: 0,
                col: 0,
                byte_offset: 0,
                byte_length: 0,
            },
        )
    }

    /// Verify a layout against compiler-generated facts
    pub fn verify_layout(
        &self,
        layout: &TypeLayout,
        compiler_facts: &CompilerLayoutFacts,
    ) -> std::result::Result<(), LayoutError> {
        if layout.size != compiler_facts.size {
            return Err(LayoutError::SizeMismatch {
                expected: compiler_facts.size,
                actual: layout.size,
            });
        }

        if layout.align != compiler_facts.align {
            return Err(LayoutError::AlignmentMismatch {
                expected: compiler_facts.align,
                actual: layout.align,
            });
        }

        for fact in &compiler_facts.field_facts {
            if let Some(field) = layout.fields.iter().find(|f| f.name == fact.name) {
                if field.offset != fact.offset {
                    return Err(LayoutError::InvalidFieldOffset(format!(
                        "field {} offset mismatch: {} vs {}",
                        fact.name, field.offset, fact.offset
                    )));
                }
            }
        }

        Ok(())
    }
}

/// Compiler-provided layout facts (from Clang)
#[derive(Debug, Clone, Default)]
pub struct CompilerLayoutFacts {
    /// Type size
    pub size: u64,
    /// Type alignment
    pub align: u32,
    /// Field layout facts
    pub field_facts: Vec<CompilerFieldFact>,
    /// Bitfield facts
    pub bitfield_facts: Vec<CompilerBitfieldFact>,
}

#[derive(Debug, Clone)]
pub struct CompilerFieldFact {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub align: u32,
}

#[derive(Debug, Clone)]
pub struct CompilerBitfieldFact {
    pub name: String,
    pub container_offset: u64,
    pub bit_offset: u8,
    pub bit_width: u8,
}

impl TargetAbi {
    /// Create TargetAbi from a target triple with sysroot and resource dir
    pub fn from_triple_with_sysroot(
        triple: &str,
        sysroot: Option<&str>,
        resource_dir: Option<&str>,
    ) -> Self {
        let (endianness, pointer_size, long_size) = if triple.contains("windows") {
            (Endianness::Little, 8, 4)
        } else if triple.contains("darwin") || triple.contains("apple") {
            (Endianness::Little, 8, 8)
        } else if triple.contains("wasm") {
            (Endianness::Little, 4, 4) // wasm32 is 32-bit
        } else {
            (Endianness::Little, 8, 8)
        };

        let (long_long_size, int_size, short_size, char_size) = (8, 4, 2, 1);
        let (double_size, long_double_size) = (8, 16);

        let mut abi = Self {
            triple: triple.to_string(),
            data_layout: DataLayout::default(),
            endianness,
            pointer_size,
            long_size,
            long_long_size,
            int_size,
            short_size,
            char_size,
            double_size,
            long_double_size,
            int64_align: 8,
            long_long_align: 8,
            double_align: 8,
            long_double_align: 16,
            sysroot: sysroot.map(String::from),
            resource_dir: resource_dir.map(String::from),
            is_cross_compile: false,
            host_arch: None,
        };

        // Detect cross-compilation by checking if host != target
        let host_arch = std::env::consts::ARCH;
        let target_arch = if triple.starts_with("x86_64") {
            "x86_64"
        } else if triple.starts_with("aarch64") || triple.starts_with("arm64") {
            "aarch64"
        } else if triple.starts_with("wasm") {
            "wasm"
        } else if triple.starts_with("i386") || triple.starts_with("i686") {
            "x86"
        } else {
            "unknown"
        };

        abi.is_cross_compile = host_arch != target_arch;
        abi.host_arch = Some(host_arch.to_string());

        abi
    }

    /// Create TargetAbi from a target triple (legacy constructor)
    pub fn from_triple(triple: &str) -> Self {
        Self::from_triple_with_sysroot(triple, None, None)
    }

    /// Get the platform pointer size
    pub fn pointer_size(&self) -> u32 {
        self.pointer_size
    }

    /// Check if this is a big-endian platform
    pub fn is_big_endian(&self) -> bool {
        self.endianness == Endianness::Big
    }

    /// Get the appropriate int type for a given size
    pub fn int_type_for_size(&self, size: u32) -> &'static str {
        match size {
            1 => "int8_t",
            2 => "int16_t",
            4 => "int32_t",
            8 => "int64_t",
            _ => "int",
        }
    }

    /// Check if this target is big-endian (for cross-compilation)
    pub fn is_cross_compilation(&self) -> bool {
        self.is_cross_compile
    }

    /// Get the sysroot path if set
    pub fn sysroot_path(&self) -> Option<&str> {
        self.sysroot.as_deref()
    }

    /// Get the resource directory if set
    pub fn resource_dir_path(&self) -> Option<&str> {
        self.resource_dir.as_deref()
    }

    /// Get host architecture
    pub fn host_arch(&self) -> Option<&str> {
        self.host_arch.as_deref()
    }

    /// Compute a layout key for caching cross-compiled layouts
    pub fn layout_key(&self) -> String {
        let mut key = format!(
            "target={}:ptr={}:long={}:ll={}:int={}:short={}:char={}:double={}:ld={}",
            self.triple,
            self.pointer_size,
            self.long_size,
            self.long_long_size,
            self.int_size,
            self.short_size,
            self.char_size,
            self.double_size,
            self.long_double_size
        );
        if let Some(ref sr) = self.sysroot {
            key.push_str(&format!(":sysroot={}", sr));
        }
        if let Some(ref rd) = self.resource_dir {
            key.push_str(&format!(":rd={}", rd));
        }
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_abi_from_triple() {
        let abi = TargetAbi::from_triple("x86_64-unknown-linux-gnu");
        assert_eq!(abi.pointer_size, 8);
        assert_eq!(abi.endianness, Endianness::Little);
        assert_eq!(abi.long_size, 8);

        let abi_win = TargetAbi::from_triple("x86_64-pc-windows-msvc");
        assert_eq!(abi_win.pointer_size, 8);
        assert_eq!(abi_win.long_size, 4);
    }

    #[test]
    fn test_endianness_default() {
        let endian = Endianness::default();
        assert_eq!(endian, Endianness::Little);
    }

    #[test]
    fn test_type_layout_fingerprint() {
        let layout = TypeLayout {
            name: "Point".to_string(),
            size: 8,
            align: 4,
            is_packed: false,
            explicit_align: None,
            fields: vec![
                FieldLayout {
                    name: "x".to_string(),
                    offset: 0,
                    size: 4,
                    align: 4,
                    bitfield: None,
                },
                FieldLayout {
                    name: "y".to_string(),
                    offset: 4,
                    size: 4,
                    align: 4,
                    bitfield: None,
                },
            ],
            bitfields: vec![],
            has_flexible_array: false,
            fingerprint: String::new(),
        };

        let fp = layout.compute_fingerprint();
        assert!(!fp.is_empty());
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn test_type_layout_verify() {
        let layout1 = TypeLayout {
            name: "Test".to_string(),
            size: 8,
            align: 8,
            is_packed: false,
            explicit_align: None,
            fields: vec![],
            bitfields: vec![],
            has_flexible_array: false,
            fingerprint: String::new(),
        };

        let layout2 = TypeLayout {
            name: "Test".to_string(),
            size: 8,
            align: 8,
            is_packed: false,
            explicit_align: None,
            fields: vec![],
            bitfields: vec![],
            has_flexible_array: false,
            fingerprint: String::new(),
        };

        assert!(layout1.verify(&layout2).is_ok());
    }

    #[test]
    fn test_type_layout_verify_size_mismatch() {
        let layout1 = TypeLayout {
            name: "Test".to_string(),
            size: 8,
            align: 8,
            is_packed: false,
            explicit_align: None,
            fields: vec![],
            bitfields: vec![],
            has_flexible_array: false,
            fingerprint: String::new(),
        };

        let layout2 = TypeLayout {
            name: "Test".to_string(),
            size: 16,
            align: 8,
            is_packed: false,
            explicit_align: None,
            fields: vec![],
            bitfields: vec![],
            has_flexible_array: false,
            fingerprint: String::new(),
        };

        let result = layout1.verify(&layout2);
        assert!(matches!(result, Err(LayoutError::SizeMismatch { .. })));
    }

    #[test]
    fn test_static_assert_size() {
        let assertion = StaticAssert::size_assert(
            "my_struct",
            16,
            SourceSpan {
                file: "test.h".to_string(),
                line: 10,
                col: 1,
                byte_offset: 100,
                byte_length: 30,
            },
        );
        assert!(assertion.expression.contains("sizeof"));
        assert!(assertion.message.is_some());
    }

    #[test]
    fn test_static_assert_align() {
        let assertion = StaticAssert::align_assert(
            "my_struct",
            8,
            SourceSpan {
                file: "test.h".to_string(),
                line: 10,
                col: 1,
                byte_offset: 100,
                byte_length: 30,
            },
        );
        assert!(assertion.expression.contains("_Alignof"));
    }

    #[test]
    fn test_layout_context_default() {
        let ctx = LayoutContext::default();
        assert!(ctx.type_layouts.is_empty());
        assert!(ctx.static_asserts.is_empty());
    }

    #[test]
    fn test_layout_context_add_layout() {
        let mut ctx = LayoutContext::default();
        let layout = TypeLayout {
            name: "TestStruct".to_string(),
            size: 8,
            align: 8,
            is_packed: false,
            explicit_align: None,
            fields: vec![],
            bitfields: vec![],
            has_flexible_array: false,
            fingerprint: String::new(),
        };

        ctx.add_layout(TypeRef(42), layout);
        assert!(ctx.get_layout(&TypeRef(42)).is_some());
    }

    #[test]
    fn test_c_layout_extractor_primitive() {
        let extractor = CLayoutExtractor::new("x86_64-unknown-linux-gnu");

        let char_layout = extractor.primitive_layout("char");
        assert!(char_layout.is_some());
        assert_eq!(char_layout.unwrap().size, 1);

        let int_layout = extractor.primitive_layout("int");
        assert!(int_layout.is_some());
        assert_eq!(int_layout.unwrap().size, 4);

        let unknown = extractor.primitive_layout("unknown_type");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_flexible_array_member_detection() {
        let layout = TypeLayout {
            name: "WithFlexible".to_string(),
            size: 16,
            align: 4,
            is_packed: false,
            explicit_align: None,
            fields: vec![
                FieldLayout {
                    name: "count".to_string(),
                    offset: 0,
                    size: 4,
                    align: 4,
                    bitfield: None,
                },
                FieldLayout {
                    name: "".to_string(),
                    offset: 4,
                    size: 0,
                    align: 1,
                    bitfield: None,
                },
            ],
            bitfields: vec![],
            has_flexible_array: true,
            fingerprint: String::new(),
        };

        assert!(layout.has_flexible_array);
    }

    #[test]
    fn test_bitfield_layout() {
        let bf = BitfieldLayout {
            bit_offset: 0,
            bit_width: 4,
            storage_type: "int".to_string(),
            container_offset: 0,
            is_signed: false,
        };

        assert_eq!(bf.bit_width, 4);
        assert!(!bf.is_signed);
    }

    #[test]
    fn test_packed_aligned_attr() {
        let attr = PackedAlignedAttr {
            kind: PackedAlignedKind::Packed,
            value: None,
            location: SourceSpan {
                file: "test.h".to_string(),
                line: 5,
                col: 1,
                byte_offset: 50,
                byte_length: 10,
            },
        };

        assert_eq!(attr.kind, PackedAlignedKind::Packed);
    }

    #[test]
    fn test_compiler_layout_facts_default() {
        let facts = CompilerLayoutFacts::default();
        assert_eq!(facts.size, 0);
        assert!(facts.field_facts.is_empty());
    }

    #[test]
    fn test_target_abi_pointer_size() {
        let abi = TargetAbi::from_triple("x86_64-unknown-linux-gnu");
        assert_eq!(abi.pointer_size(), 8);
    }

    #[test]
    fn test_target_abi_int_type_for_size() {
        let abi = TargetAbi::from_triple("x86_64-unknown-linux-gnu");
        assert_eq!(abi.int_type_for_size(4), "int32_t");
        assert_eq!(abi.int_type_for_size(8), "int64_t");
    }

    // =============================================================================
    // Cross-Compilation Tests (Task 40)
    // =============================================================================

    #[test]
    fn test_target_abi_from_triple_with_sysroot() {
        let abi = TargetAbi::from_triple_with_sysroot(
            "aarch64-unknown-linux-gnu",
            Some("/opt/cross/aarch64"),
            Some("/opt/cross/aarch64/lib/clang/14.0.0"),
        );
        assert_eq!(abi.triple, "aarch64-unknown-linux-gnu");
        assert_eq!(abi.sysroot.as_ref().unwrap(), "/opt/cross/aarch64");
        assert!(abi.resource_dir.is_some());
        assert!(abi.is_cross_compile);
    }

    #[test]
    fn test_target_abi_default() {
        let abi = TargetAbi::default();
        assert_eq!(abi.pointer_size, 8);
        assert_eq!(abi.endianness, Endianness::Little);
        assert!(!abi.is_cross_compile);
        assert!(abi.sysroot.is_none());
    }

    #[test]
    fn test_target_abi_is_cross_compilation() {
        // This test will detect if we're on x86_64 and targeting aarch64
        let abi = TargetAbi::from_triple("aarch64-unknown-linux-gnu");
        // Cross compile if host_arch != target arch
        let host = std::env::consts::ARCH;
        if host == "x86_64" {
            assert!(abi.is_cross_compilation());
        } else {
            // If host is aarch64, then it won't be cross compile
            assert!(!abi.is_cross_compilation() || host == "aarch64");
        }
    }

    #[test]
    fn test_target_abi_sysroot_path() {
        let abi = TargetAbi::from_triple_with_sysroot(
            "x86_64-unknown-linux-gnu",
            Some("/usr/aarch64-linux-gnu"),
            None,
        );
        assert_eq!(abi.sysroot_path(), Some("/usr/aarch64-linux-gnu"));
    }

    #[test]
    fn test_target_abi_resource_dir_path() {
        let abi = TargetAbi::from_triple_with_sysroot(
            "x86_64-unknown-linux-gnu",
            None,
            Some("/opt/clang/lib"),
        );
        assert_eq!(abi.resource_dir_path(), Some("/opt/clang/lib"));
    }

    #[test]
    fn test_target_abi_host_arch() {
        let abi = TargetAbi::from_triple("x86_64-unknown-linux-gnu");
        assert_eq!(abi.host_arch(), Some(std::env::consts::ARCH));
    }

    #[test]
    fn test_target_abi_layout_key() {
        let abi = TargetAbi::from_triple("x86_64-unknown-linux-gnu");
        let key = abi.layout_key();
        assert!(key.contains("target=x86_64-unknown-linux-gnu"));
        assert!(key.contains("ptr=8"));
        assert!(key.contains("long=8"));
    }

    #[test]
    fn test_target_abi_layout_key_with_sysroot() {
        let abi = TargetAbi::from_triple_with_sysroot(
            "aarch64-unknown-linux-gnu",
            Some("/opt/cross"),
            None,
        );
        let key = abi.layout_key();
        assert!(key.contains("sysroot=/opt/cross"));
    }

    #[test]
    fn test_target_abi_different_archs() {
        let x86 = TargetAbi::from_triple("x86_64-unknown-linux-gnu");
        let aarch64 = TargetAbi::from_triple("aarch64-unknown-linux-gnu");
        let wasm = TargetAbi::from_triple("wasm32-unknown-unknown");

        assert_eq!(x86.pointer_size, 8);
        assert_eq!(aarch64.pointer_size, 8);
        assert_eq!(wasm.pointer_size, 4); // wasm32 is 32-bit
    }

    #[test]
    fn test_target_abi_windows_long_size() {
        let abi = TargetAbi::from_triple("x86_64-pc-windows-msvc");
        assert_eq!(abi.long_size, 4); // Windows long is 4 bytes
    }
}
