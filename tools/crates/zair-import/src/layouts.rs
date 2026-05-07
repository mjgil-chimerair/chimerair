//! Layout table import for AIR memory layouts.
//!
//! Task 45: Import layout table with size, alignment, field offsets.

use serde::{Deserialize, Serialize};

/// A layout record from zairpack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutRecord {
    /// Layout ID
    pub layout_id: u64,
    /// Size in bytes
    pub size: u64,
    /// Alignment in bytes
    pub alignment: u32,
    /// Field offsets (for structs)
    pub field_offsets: Vec<u64>,
    /// Field sizes (for structs)
    pub field_sizes: Vec<u64>,
    /// Packed struct flag
    pub packed: bool,
    /// Extern struct flag
    pub is_extern: bool,
}

/// Import result for layout table
#[derive(Debug, Clone)]
pub struct LayoutImportResult {
    /// All layout records
    pub layouts: Vec<LayoutRecord>,
    /// Layout ID remapping
    pub id_map: std::collections::HashMap<u64, u64>,
}

impl LayoutImportResult {
    /// Create new import result
    pub fn new() -> Self {
        Self {
            layouts: Vec::new(),
            id_map: std::collections::HashMap::new(),
        }
    }

    /// Add a layout record
    pub fn add_layout(&mut self, layout_id: u64, record: LayoutRecord) {
        let dialect_id = self.layouts.len() as u64;
        self.id_map.insert(layout_id, dialect_id);
        self.layouts.push(record);
    }

    /// Get dialect layout ID for a zairpack layout ID
    pub fn get_dialect_id(&self, layout_id: u64) -> Option<u64> {
        self.id_map.get(&layout_id).copied()
    }
}

impl Default for LayoutImportResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout table importer
#[derive(Debug, Clone)]
pub struct LayoutTableImporter {
    /// Imported layout results
    pub result: LayoutImportResult,
}

impl LayoutTableImporter {
    /// Create new importer
    pub fn new() -> Self {
        Self {
            result: LayoutImportResult::new(),
        }
    }

    /// Import a simple scalar layout
    pub fn import_scalar(&mut self, layout_id: u64, size: u64, alignment: u32) {
        self.result.add_layout(layout_id, LayoutRecord {
            layout_id,
            size,
            alignment,
            field_offsets: Vec::new(),
            field_sizes: Vec::new(),
            packed: false,
            is_extern: false,
        });
    }

    /// Import a struct layout
    pub fn import_struct(
        &mut self,
        layout_id: u64,
        size: u64,
        alignment: u32,
        field_offsets: Vec<u64>,
        field_sizes: Vec<u64>,
        packed: bool,
        is_extern: bool,
    ) {
        self.result.add_layout(layout_id, LayoutRecord {
            layout_id,
            size,
            alignment,
            field_offsets,
            field_sizes,
            packed,
            is_extern,
        });
    }

    /// Import a pointer layout
    pub fn import_pointer(&mut self, layout_id: u64, address_space: u32) {
        let size = 8u64;
        let alignment = if address_space == 0 { 8 } else { 4 };
        self.result.add_layout(layout_id, LayoutRecord {
            layout_id,
            size,
            alignment,
            field_offsets: Vec::new(),
            field_sizes: Vec::new(),
            packed: false,
            is_extern: false,
        });
    }

    /// Import a slice layout
    pub fn import_slice(&mut self, layout_id: u64, elem_size: u64) {
        self.result.add_layout(layout_id, LayoutRecord {
            layout_id,
            size: 16,
            alignment: 8,
            field_offsets: vec![0, 8],
            field_sizes: vec![8, elem_size],
            packed: false,
            is_extern: false,
        });
    }

    /// Get the import result
    pub fn finish(self) -> LayoutImportResult {
        self.result
    }
}

impl Default for LayoutTableImporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Check layout consistency against expected size
pub fn validate_layout_consistency(layout: &LayoutRecord) -> bool {
    if layout.packed {
        // Packed structs have no padding
        if let Some(last_offset) = layout.field_offsets.last() {
            let computed_size = last_offset + layout.field_sizes.last().unwrap_or(&0);
            if computed_size != layout.size {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_importer_creation() {
        let importer = LayoutTableImporter::new();
        assert_eq!(importer.result.layouts.len(), 0);
    }

    #[test]
    fn test_import_scalar() {
        let mut importer = LayoutTableImporter::new();
        importer.import_scalar(1, 4, 4);
        assert_eq!(importer.result.layouts.len(), 1);
        let layout = &importer.result.layouts[0];
        assert_eq!(layout.size, 4);
        assert_eq!(layout.alignment, 4);
        assert!(!layout.packed);
        assert!(!layout.is_extern);
    }

    #[test]
    fn test_import_struct() {
        let mut importer = LayoutTableImporter::new();
        importer.import_struct(1, 16, 8, vec![0, 8], vec![8, 8], false, false);
        assert_eq!(importer.result.layouts.len(), 1);
        let layout = &importer.result.layouts[0];
        assert_eq!(layout.size, 16);
        assert_eq!(layout.alignment, 8);
        assert_eq!(layout.field_offsets.len(), 2);
        assert_eq!(layout.field_sizes.len(), 2);
    }

    #[test]
    fn test_import_pointer() {
        let mut importer = LayoutTableImporter::new();
        importer.import_pointer(1, 0);
        assert_eq!(importer.result.layouts.len(), 1);
        let layout = &importer.result.layouts[0];
        assert_eq!(layout.size, 8);
        assert_eq!(layout.alignment, 8);
    }

    #[test]
    fn test_import_slice() {
        let mut importer = LayoutTableImporter::new();
        importer.import_slice(1, 4);
        assert_eq!(importer.result.layouts.len(), 1);
        let layout = &importer.result.layouts[0];
        assert_eq!(layout.size, 16);
        assert_eq!(layout.alignment, 8);
        assert_eq!(layout.field_offsets.len(), 2);
        assert_eq!(layout.field_sizes.len(), 2);
    }

    #[test]
    fn test_id_mapping() {
        let mut importer = LayoutTableImporter::new();
        importer.import_scalar(100, 4, 4);
        importer.import_scalar(200, 8, 8);
        assert_eq!(importer.result.get_dialect_id(100), Some(0));
        assert_eq!(importer.result.get_dialect_id(200), Some(1));
        assert_eq!(importer.result.get_dialect_id(999), None);
    }

    #[test]
    fn test_validate_layout_consistency() {
        let layout = LayoutRecord {
            layout_id: 1,
            size: 16,
            alignment: 8,
            field_offsets: vec![0, 8],
            field_sizes: vec![8, 8],
            packed: false,
            is_extern: false,
        };
        assert!(validate_layout_consistency(&layout));
    }

    #[test]
    fn test_validate_packed_layout_consistency() {
        let layout = LayoutRecord {
            layout_id: 1,
            size: 10,
            alignment: 1,
            field_offsets: vec![0, 3, 7],
            field_sizes: vec![3, 4, 3],
            packed: true,
            is_extern: false,
        };
        assert!(validate_layout_consistency(&layout));
    }
}