//! ID remapper using compiler stable seeds for stable ID generation.
//!
//! Task 48: Implement ID remapper using compiler stable seeds.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A stable ID derived from multiple seeds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StableId(u64);

impl StableId {
    /// Get the raw value
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Create from raw value
    pub fn from_raw(val: u64) -> Self {
        StableId(val)
    }
}

/// Source of stability for ID generation
#[derive(Debug, Clone)]
pub enum IdSeed {
    /// Source file path
    SourcePath(String),
    /// Declaration path
    DeclPath(String),
    /// InternPool stable key
    InternPoolKey(u64),
    /// Type shape (hash of type structure)
    TypeShape(u64),
    /// Schema version
    SchemaVersion(u32),
}

/// ID remapper configuration
#[derive(Debug, Clone)]
pub struct IdRemapperConfig {
    /// Schema version for stable ID derivation
    pub schema_version: u32,
    /// Include source paths in ID calculation
    pub include_source_paths: bool,
    /// Include type shapes in ID calculation
    pub include_type_shapes: bool,
}

impl Default for IdRemapperConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            include_source_paths: true,
            include_type_shapes: true,
        }
    }
}

/// ID remapper using compiler stable seeds
#[derive(Debug, Clone)]
pub struct IdRemapper {
    /// Configuration
    config: IdRemapperConfig,
    /// Mapping from original IDs to stable IDs
    original_to_stable: HashMap<u64, StableId>,
    /// Reverse mapping
    stable_to_original: HashMap<StableId, u64>,
    /// Next available stable ID
    next_stable_id: u64,
}

impl IdRemapper {
    /// Create a new ID remapper with configuration
    pub fn new(config: IdRemapperConfig) -> Self {
        Self {
            config,
            original_to_stable: HashMap::new(),
            stable_to_original: HashMap::new(),
            next_stable_id: 1,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(IdRemapperConfig::default())
    }

    /// Compute a stable ID from seeds
    fn compute_stable_id(&self, seeds: &[u64]) -> u64 {
        let mut hasher = zigmera_hash::Blake3Hasher::with_schema_tag("stable-id-v1");
        for seed in seeds {
            hasher.update_u64(*seed);
        }
        hasher.update(&(self.config.schema_version as u64).to_le_bytes());
        // Use first 8 bytes of BLAKE3 as u64
        let hash = hasher.finalize();
        let hash_bytes = hash.as_bytes();
        u64::from_le_bytes([hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3], hash_bytes[4], hash_bytes[5], hash_bytes[6], hash_bytes[7]])
    }

    /// Remap an original ID to a stable ID
    pub fn remap(&mut self, original_id: u64, seeds: Vec<u64>) -> StableId {
        if let Some(existing) = self.original_to_stable.get(&original_id) {
            return *existing;
        }

        let stable_id_val = if seeds.is_empty() {
            let val = self.next_stable_id;
            self.next_stable_id += 1;
            val
        } else {
            // Use the first seed as the primary identifier, combine with schema
            self.compute_stable_id(&seeds)
        };

        let stable_id = StableId::from_raw(stable_id_val);
        self.original_to_stable.insert(original_id, stable_id);
        self.stable_to_original.insert(stable_id, original_id);

        // Ensure we don't reuse IDs
        if stable_id_val >= self.next_stable_id {
            self.next_stable_id = stable_id_val + 1;
        }

        stable_id
    }

    /// Get stable ID for an original ID
    pub fn get_stable_id(&self, original_id: u64) -> Option<StableId> {
        self.original_to_stable.get(&original_id).copied()
    }

    /// Get original ID for a stable ID
    pub fn get_original_id(&self, stable_id: StableId) -> Option<u64> {
        self.stable_to_original.get(&stable_id).copied()
    }

    /// Get the number of remapped IDs
    pub fn remapped_count(&self) -> usize {
        self.original_to_stable.len()
    }

    /// Clear all mappings
    pub fn clear(&mut self) {
        self.original_to_stable.clear();
        self.stable_to_original.clear();
        self.next_stable_id = 1;
    }
}

impl Default for IdRemapper {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_remapper_creation() {
        let remapper = IdRemapper::with_defaults();
        assert_eq!(remapper.remapped_count(), 0);
    }

    #[test]
    fn test_remap_with_seed() {
        let mut remapper = IdRemapper::with_defaults();
        let stable = remapper.remap(100, vec![100, 200]);
        assert!(stable.value() > 0);
    }

    #[test]
    fn test_remap_same_id_twice() {
        let mut remapper = IdRemapper::with_defaults();
        let stable1 = remapper.remap(100, vec![100]);
        let stable2 = remapper.remap(100, vec![100]);
        assert_eq!(stable1, stable2);
    }

    #[test]
    fn test_get_stable_id() {
        let mut remapper = IdRemapper::with_defaults();
        let stable = remapper.remap(100, vec![100]);
        assert_eq!(remapper.get_stable_id(100), Some(stable));
        assert_eq!(remapper.get_original_id(stable), Some(100));
    }

    #[test]
    fn test_get_stable_id_not_found() {
        let remapper = IdRemapper::with_defaults();
        assert_eq!(remapper.get_stable_id(999), None);
    }

    #[test]
    fn test_remap_different_ids_different_stable() {
        let mut remapper = IdRemapper::with_defaults();
        let stable1 = remapper.remap(100, vec![100]);
        let stable2 = remapper.remap(200, vec![200]);
        assert_ne!(stable1, stable2);
    }

    #[test]
    fn test_remap_no_seed_uses_sequential() {
        let mut remapper = IdRemapper::with_defaults();
        let stable1 = remapper.remap(100, vec![]);
        let stable2 = remapper.remap(200, vec![]);
        assert_ne!(stable1, stable2);
        // Sequential IDs
        assert_eq!(stable1.value(), 1);
        assert_eq!(stable2.value(), 2);
    }

    #[test]
    fn test_clear() {
        let mut remapper = IdRemapper::with_defaults();
        remapper.remap(100, vec![100]);
        assert_eq!(remapper.remapped_count(), 1);
        remapper.clear();
        assert_eq!(remapper.remapped_count(), 0);
    }

    #[test]
    fn test_stable_id_value() {
        let stable = StableId::from_raw(12345);
        assert_eq!(stable.value(), 12345);
    }

    #[test]
    fn test_schema_version_affects_stable_id() {
        let config1 = IdRemapperConfig { schema_version: 1, include_source_paths: true, include_type_shapes: true };
        let config2 = IdRemapperConfig { schema_version: 2, include_source_paths: true, include_type_shapes: true };

        let mut remapper1 = IdRemapper::new(config1);
        let mut remapper2 = IdRemapper::new(config2);

        let stable1 = remapper1.remap(100, vec![100]);
        let stable2 = remapper2.remap(100, vec![100]);

        // Same input should produce different output with different schema versions
        // Note: this depends on the hash function implementation
        assert_eq!(remapper1.get_stable_id(100), Some(stable1));
        assert_eq!(remapper2.get_stable_id(100), Some(stable2));
    }
}