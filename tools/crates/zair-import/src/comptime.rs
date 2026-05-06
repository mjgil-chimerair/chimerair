//! Comptime value import for AIR constant values.
//!
//! Task 46: Import comptime values (primitive, enum, struct, slice, pointer, optional).

use serde::{Deserialize, Serialize};

/// A comptime value kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComptimeValueKind {
    /// Undefined value
    Undefined,
    /// Integer value
    Int(i64),
    /// Unsigned integer value
    Uint(u64),
    /// Float value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// String/value (for enum variant names)
    String(String),
    /// Type value (type-level)
    Type,
    /// Function pointer
    FuncPtr(u64),
    /// Pointer address
    Ptr(u64),
}

/// A comptime value from AIR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComptimeValue {
    /// Value kind
    pub kind: ComptimeValueKind,
    /// Result type ID
    pub result_type: u64,
}

/// Import result for comptime values
#[derive(Debug, Clone)]
pub struct ComptimeImportResult {
    /// All imported comptime values
    pub values: Vec<ComptimeValue>,
    /// Value ID mapping
    pub id_map: std::collections::HashMap<u64, u64>,
}

impl ComptimeImportResult {
    /// Create new import result
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            id_map: std::collections::HashMap::new(),
        }
    }

    /// Add a comptime value
    pub fn add_value(&mut self, value_id: u64, kind: ComptimeValueKind, result_type: u64) {
        let dialect_id = self.values.len() as u64;
        self.id_map.insert(value_id, dialect_id);
        self.values.push(ComptimeValue { kind, result_type });
    }

    /// Get dialect value ID for a zairpack value ID
    pub fn get_dialect_id(&self, value_id: u64) -> Option<u64> {
        self.id_map.get(&value_id).copied()
    }
}

impl Default for ComptimeImportResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Comptime value importer
#[derive(Debug, Clone)]
pub struct ComptimeImporter {
    /// Imported comptime results
    pub result: ComptimeImportResult,
}

impl ComptimeImporter {
    /// Create new importer
    pub fn new() -> Self {
        Self {
            result: ComptimeImportResult::new(),
        }
    }

    /// Import an integer comptime value
    pub fn import_int(&mut self, value_id: u64, result_type: u64, val: i64) {
        self.result.add_value(value_id, ComptimeValueKind::Int(val), result_type);
    }

    /// Import an unsigned integer comptime value
    pub fn import_uint(&mut self, value_id: u64, result_type: u64, val: u64) {
        self.result.add_value(value_id, ComptimeValueKind::Uint(val), result_type);
    }

    /// Import a float comptime value
    pub fn import_float(&mut self, value_id: u64, result_type: u64, val: f64) {
        self.result.add_value(value_id, ComptimeValueKind::Float(val), result_type);
    }

    /// Import a boolean comptime value
    pub fn import_bool(&mut self, value_id: u64, result_type: u64, val: bool) {
        self.result.add_value(value_id, ComptimeValueKind::Bool(val), result_type);
    }

    /// Import a string comptime value
    pub fn import_string(&mut self, value_id: u64, result_type: u64, val: String) {
        self.result.add_value(value_id, ComptimeValueKind::String(val), result_type);
    }

    /// Import a function pointer
    pub fn import_func_ptr(&mut self, value_id: u64, result_type: u64, func_id: u64) {
        self.result.add_value(value_id, ComptimeValueKind::FuncPtr(func_id), result_type);
    }

    /// Import a pointer
    pub fn import_ptr(&mut self, value_id: u64, result_type: u64, addr: u64) {
        self.result.add_value(value_id, ComptimeValueKind::Ptr(addr), result_type);
    }

    /// Import undefined value
    pub fn import_undefined(&mut self, value_id: u64, result_type: u64) {
        self.result.add_value(value_id, ComptimeValueKind::Undefined, result_type);
    }

    /// Import type-level value
    pub fn import_type(&mut self, value_id: u64, result_type: u64) {
        self.result.add_value(value_id, ComptimeValueKind::Type, result_type);
    }

    /// Get the import result
    pub fn finish(self) -> ComptimeImportResult {
        self.result
    }
}

impl Default for ComptimeImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comptime_importer_creation() {
        let importer = ComptimeImporter::new();
        assert_eq!(importer.result.values.len(), 0);
    }

    #[test]
    fn test_import_int() {
        let mut importer = ComptimeImporter::new();
        importer.import_int(1, 100, 42);
        assert_eq!(importer.result.values.len(), 1);
        assert!(matches!(importer.result.values[0].kind, ComptimeValueKind::Int(42)));
    }

    #[test]
    fn test_import_uint() {
        let mut importer = ComptimeImporter::new();
        importer.import_uint(1, 100, 100);
        assert_eq!(importer.result.values.len(), 1);
        assert!(matches!(importer.result.values[0].kind, ComptimeValueKind::Uint(100)));
    }

    #[test]
    fn test_import_float() {
        let mut importer = ComptimeImporter::new();
        importer.import_float(1, 100, 3.14);
        assert_eq!(importer.result.values.len(), 1);
        assert!(matches!(importer.result.values[0].kind, ComptimeValueKind::Float(v) if (v - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_import_bool() {
        let mut importer = ComptimeImporter::new();
        importer.import_bool(1, 100, true);
        assert_eq!(importer.result.values.len(), 1);
        assert!(matches!(importer.result.values[0].kind, ComptimeValueKind::Bool(true)));
    }

    #[test]
    fn test_import_string() {
        let mut importer = ComptimeImporter::new();
        importer.import_string(1, 100, "hello".to_string());
        assert_eq!(importer.result.values.len(), 1);
        assert!(matches!(&importer.result.values[0].kind, ComptimeValueKind::String(s) if s == "hello"));
    }

    #[test]
    fn test_import_func_ptr() {
        let mut importer = ComptimeImporter::new();
        importer.import_func_ptr(1, 100, 200);
        assert_eq!(importer.result.values.len(), 1);
        assert!(matches!(importer.result.values[0].kind, ComptimeValueKind::FuncPtr(200)));
    }

    #[test]
    fn test_import_ptr() {
        let mut importer = ComptimeImporter::new();
        importer.import_ptr(1, 100, 0x7fff0000);
        assert_eq!(importer.result.values.len(), 1);
        assert!(matches!(importer.result.values[0].kind, ComptimeValueKind::Ptr(0x7fff0000)));
    }

    #[test]
    fn test_import_undefined() {
        let mut importer = ComptimeImporter::new();
        importer.import_undefined(1, 100);
        assert_eq!(importer.result.values.len(), 1);
        assert!(matches!(importer.result.values[0].kind, ComptimeValueKind::Undefined));
    }

    #[test]
    fn test_import_type() {
        let mut importer = ComptimeImporter::new();
        importer.import_type(1, 100);
        assert_eq!(importer.result.values.len(), 1);
        assert!(matches!(importer.result.values[0].kind, ComptimeValueKind::Type));
    }

    #[test]
    fn test_id_mapping() {
        let mut importer = ComptimeImporter::new();
        importer.import_int(100, 1, 10);
        importer.import_bool(200, 2, true);
        assert_eq!(importer.result.get_dialect_id(100), Some(0));
        assert_eq!(importer.result.get_dialect_id(200), Some(1));
        assert_eq!(importer.result.get_dialect_id(999), None);
    }
}