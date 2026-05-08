//! Type table import for AIR types.
//!
//! Task 44: Import type table into stable dialect types.

use zigmera_dialect::types::ZigTypeKind;
use serde::{Deserialize, Serialize};

/// A type mapping entry from zairpack type table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMapping {
    /// Type ID in the zairpack
    pub type_id: u64,
    /// Mapped ZigTypeKind
    pub kind: ZigTypeKind,
    /// Size in bytes (0 for unsized)
    pub size: u64,
    /// Alignment in bytes
    pub alignment: u32,
}

/// Import result for type table
#[derive(Debug, Clone)]
pub struct TypeImportResult {
    /// All type mappings
    pub mappings: Vec<TypeMapping>,
    /// Type ID remapping from zairpack to dialect
    pub id_map: std::collections::HashMap<u64, u64>,
}

impl TypeImportResult {
    /// Create new import result
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
            id_map: std::collections::HashMap::new(),
        }
    }

    /// Add a type mapping
    pub fn add_mapping(&mut self, type_id: u64, kind: ZigTypeKind, size: u64, alignment: u32) {
        let dialect_id = self.mappings.len() as u64;
        self.id_map.insert(type_id, dialect_id);
        self.mappings.push(TypeMapping {
            type_id,
            kind,
            size,
            alignment,
        });
    }

    /// Get dialect type ID for a zairpack type ID
    pub fn get_dialect_id(&self, type_id: u64) -> Option<u64> {
        self.id_map.get(&type_id).copied()
    }
}

impl Default for TypeImportResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Type table importer
#[derive(Debug, Clone)]
pub struct TypeTableImporter {
    /// Imported type results
    pub result: TypeImportResult,
}

impl TypeTableImporter {
    /// Create new importer
    pub fn new() -> Self {
        Self {
            result: TypeImportResult::new(),
        }
    }

    /// Import a boolean type
    pub fn import_bool(&mut self, type_id: u64) {
        self.result.add_mapping(type_id, ZigTypeKind::Bool, 1, 1);
    }

    /// Import an integer type
    pub fn import_int(&mut self, type_id: u64, width: u32, signed: bool) {
        let size = (width / 8) as u64;
        let alignment: u32 = size as u32;
        self.result.add_mapping(type_id, ZigTypeKind::Int { width, signed }, size, alignment);
    }

    /// Import a float type
    pub fn import_float(&mut self, type_id: u64, width: u32) {
        let size = (width / 8) as u64;
        let alignment: u32 = size as u32;
        self.result.add_mapping(type_id, ZigTypeKind::Float { width }, size, alignment);
    }

    /// Import a void type
    pub fn import_void(&mut self, type_id: u64) {
        self.result.add_mapping(type_id, ZigTypeKind::Void, 0, 0);
    }

    /// Import a pointer type
    pub fn import_pointer(&mut self, type_id: u64) {
        self.result.add_mapping(type_id, ZigTypeKind::Pointer, 8, 8);
    }

    /// Import a slice type
    pub fn import_slice(&mut self, type_id: u64, elem_type: u64) {
        self.result.add_mapping(type_id, ZigTypeKind::Slice { elem_type }, 16, 8);
    }

    /// Import an array type
    pub fn import_array(&mut self, type_id: u64, elem_type: u64, len: u64) {
        let elem_size = self.result.mappings.iter()
            .find(|m| m.type_id == elem_type)
            .map(|m| m.size)
            .unwrap_or(0);
        let size = elem_size * len;
        self.result.add_mapping(type_id, ZigTypeKind::Array { elem_type, len }, size, 8);
    }

    /// Import a struct type
    pub fn import_struct(
        &mut self,
        type_id: u64,
        field_types: Vec<u64>,
        field_offsets: Vec<u64>,
        field_names: Vec<String>,
        packed: bool,
        is_extern: bool,
    ) {
        let size = field_offsets.last().map(|&o| o).unwrap_or(0);
        let alignment = field_offsets.first().copied().unwrap_or(1) as u32;
        self.result.add_mapping(
            type_id,
            ZigTypeKind::Struct { field_types, field_offsets, field_names, packed, is_extern },
            size,
            alignment,
        );
    }

    /// Import an optional type
    pub fn import_optional(&mut self, type_id: u64, inner: u64) {
        self.result.add_mapping(type_id, ZigTypeKind::Optional { inner }, 16, 8);
    }

    /// Import an error set type
    pub fn import_error_set(&mut self, type_id: u64, errors: Vec<String>) {
        self.result.add_mapping(type_id, ZigTypeKind::ErrorSet { errors }, 16, 8);
    }

    /// Import an error union type
    pub fn import_error_union(&mut self, type_id: u64, error_set: u64, payload: u64) {
        self.result.add_mapping(type_id, ZigTypeKind::ErrorUnion { error_set, payload }, 24, 8);
    }

    /// Import a function type
    pub fn import_fn(&mut self, type_id: u64, params: Vec<u64>, return_type: Option<u64>, callconv: String) {
        self.result.add_mapping(type_id, ZigTypeKind::Fn { params, return_type, callconv }, 0, 1);
    }

    /// Get the import result
    pub fn finish(self) -> TypeImportResult {
        self.result
    }
}

impl Default for TypeTableImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_importer_creation() {
        let importer = TypeTableImporter::new();
        assert_eq!(importer.result.mappings.len(), 0);
    }

    #[test]
    fn test_import_bool() {
        let mut importer = TypeTableImporter::new();
        importer.import_bool(1);
        assert_eq!(importer.result.mappings.len(), 1);
        assert_eq!(importer.result.get_dialect_id(1), Some(0));
    }

    #[test]
    fn test_import_int() {
        let mut importer = TypeTableImporter::new();
        importer.import_int(1, 32, true);
        assert_eq!(importer.result.mappings.len(), 1);
        let mapping = &importer.result.mappings[0];
        assert!(matches!(mapping.kind, ZigTypeKind::Int { width: 32, signed: true }));
        assert_eq!(mapping.size, 4);
    }

    #[test]
    fn test_import_float() {
        let mut importer = TypeTableImporter::new();
        importer.import_float(1, 64);
        assert_eq!(importer.result.mappings.len(), 1);
        let mapping = &importer.result.mappings[0];
        assert!(matches!(mapping.kind, ZigTypeKind::Float { width: 64 }));
        assert_eq!(mapping.size, 8);
    }

    #[test]
    fn test_import_pointer() {
        let mut importer = TypeTableImporter::new();
        importer.import_pointer(1);
        assert_eq!(importer.result.mappings.len(), 1);
        let mapping = &importer.result.mappings[0];
        assert!(matches!(mapping.kind, ZigTypeKind::Pointer));
        assert_eq!(mapping.size, 8);
    }

    #[test]
    fn test_import_slice() {
        let mut importer = TypeTableImporter::new();
        importer.import_slice(1, 10);
        assert_eq!(importer.result.mappings.len(), 1);
        let mapping = &importer.result.mappings[0];
        assert!(matches!(mapping.kind, ZigTypeKind::Slice { elem_type: 10 }));
    }

    #[test]
    fn test_import_optional() {
        let mut importer = TypeTableImporter::new();
        importer.import_optional(1, 5);
        assert_eq!(importer.result.mappings.len(), 1);
        let mapping = &importer.result.mappings[0];
        assert!(matches!(mapping.kind, ZigTypeKind::Optional { inner: 5 }));
    }

    #[test]
    fn test_import_error_set() {
        let mut importer = TypeTableImporter::new();
        importer.import_error_set(1, vec!["OutOfMemory".to_string()]);
        assert_eq!(importer.result.mappings.len(), 1);
        let mapping = &importer.result.mappings[0];
        assert!(matches!(mapping.kind, ZigTypeKind::ErrorSet { .. }));
    }

    #[test]
    fn test_import_error_union() {
        let mut importer = TypeTableImporter::new();
        importer.import_error_union(1, 2, 3);
        assert_eq!(importer.result.mappings.len(), 1);
        let mapping = &importer.result.mappings[0];
        assert!(matches!(mapping.kind, ZigTypeKind::ErrorUnion { error_set: 2, payload: 3 }));
    }

    #[test]
    fn test_import_struct() {
        let mut importer = TypeTableImporter::new();
        // field_offsets [0, 8, 16] means:
        // - field 0 at offset 0
        // - field 1 at offset 8
        // - struct is 16 bytes (offset of end)
        importer.import_struct(
            1,
            vec![2, 3, 4],
            vec![0, 8, 16],
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            false,
            false,
        );
        assert_eq!(importer.result.mappings.len(), 1);
        let mapping = &importer.result.mappings[0];
        assert!(matches!(mapping.kind, ZigTypeKind::Struct { .. }));
        assert_eq!(mapping.size, 16);
    }

    #[test]
    fn test_id_mapping() {
        let mut importer = TypeTableImporter::new();
        importer.import_int(100, 32, true);
        importer.import_bool(200);
        assert_eq!(importer.result.get_dialect_id(100), Some(0));
        assert_eq!(importer.result.get_dialect_id(200), Some(1));
        assert_eq!(importer.result.get_dialect_id(999), None);
    }
}