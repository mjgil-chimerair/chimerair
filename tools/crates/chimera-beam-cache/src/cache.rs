//! BEAM incremental cache implementation.
//!
//! Provides caching for BEAM module analysis results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::entry::{CacheEntry, CacheMetadata, CacheStats};
use super::key::{CacheKey, KeyType};

/// BEAM analysis cache.
#[derive(Debug, Clone)]
pub struct BeamCache {
    /// Cache entries by key.
    entries: HashMap<String, CacheEntry>,
    /// Cache statistics.
    stats: CacheStats,
    /// Default TTL in seconds.
    default_ttl_secs: u64,
    /// Maximum cache size in bytes.
    max_size_bytes: usize,
    /// Current size in bytes.
    current_size: usize,
}

impl BeamCache {
    /// Create a new cache.
    pub fn new() -> Self {
        BeamCache {
            entries: HashMap::new(),
            stats: CacheStats::new(),
            default_ttl_secs: 86400,           // 24 hours
            max_size_bytes: 100 * 1024 * 1024, // 100 MB
            current_size: 0,
        }
    }

    /// Create with configuration.
    pub fn with_config(max_size_bytes: usize, ttl_secs: u64) -> Self {
        BeamCache {
            entries: HashMap::new(),
            stats: CacheStats::new(),
            default_ttl_secs: ttl_secs,
            max_size_bytes,
            current_size: 0,
        }
    }

    /// Insert an entry.
    pub fn insert(
        &mut self,
        key: &CacheKey,
        data: Vec<u8>,
        module_name: &str,
        source_hash: &str,
    ) -> bool {
        let key_str = key.as_str().to_string();

        // Check if already exists
        if let Some(existing) = self.entries.get(&key_str) {
            self.stats.record_hit();
            return false;
        }

        // Calculate entry size
        let entry_size = data.len() + key_str.len() + 256; // approximate metadata overhead

        // Check if we need to evict
        while self.current_size + entry_size > self.max_size_bytes && !self.entries.is_empty() {
            self.evict_oldest();
        }

        // Create entry
        let metadata = CacheMetadata::with_module_info(module_name, source_hash);
        let entry = CacheEntry::with_metadata(key_str.clone(), data, metadata);

        self.entries.insert(key_str, entry);
        self.current_size += entry_size;
        self.stats.record_add(entry_size as u64);

        true
    }

    /// Retrieve an entry.
    pub fn get(&mut self, key: &CacheKey) -> Option<Vec<u8>> {
        let key_str = key.as_str().to_string();

        if let Some(entry) = self.entries.get_mut(&key_str) {
            // Check if expired
            if entry.is_expired(self.default_ttl_secs) {
                self.remove_entry(&key_str);
                self.stats.record_expired();
                self.stats.record_miss();
                return None;
            }

            // Update access time
            entry.metadata.touch();
            self.stats.record_hit();
            Some(entry.data.clone())
        } else {
            self.stats.record_miss();
            None
        }
    }

    /// Check if key exists and is valid.
    pub fn contains(&mut self, key: &CacheKey) -> bool {
        let key_str = key.as_str().to_string();

        if let Some(entry) = self.entries.get(&key_str) {
            if entry.is_expired(self.default_ttl_secs) {
                self.remove_entry(&key_str);
                self.stats.record_expired();
                return false;
            }
            true
        } else {
            false
        }
    }

    /// Remove an entry.
    pub fn remove(&mut self, key: &CacheKey) -> bool {
        let key_str = key.as_str().to_string();
        self.remove_entry(&key_str)
    }

    fn remove_entry(&mut self, key_str: &str) -> bool {
        if let Some(entry) = self.entries.remove(key_str) {
            let size = entry.size() + key_str.len() + 256;
            self.current_size = self.current_size.saturating_sub(size);
            true
        } else {
            false
        }
    }

    /// Evict oldest entry.
    fn evict_oldest(&mut self) {
        // Find oldest entry by created_at
        let oldest_key = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.metadata.created_at)
            .map(|(k, _)| k.clone());

        if let Some(key) = oldest_key {
            if let Some(entry) = self.entries.remove(&key) {
                let size = entry.size() + key.len() + 256;
                self.current_size = self.current_size.saturating_sub(size);
                self.stats.record_eviction(size as u64);
            }
        }
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_size = 0;
        self.stats = CacheStats::new();
    }

    /// Get statistics.
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get entry count.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get current size.
    pub fn size_bytes(&self) -> usize {
        self.current_size
    }

    /// Invalidate entries by module name.
    pub fn invalidate_module(&mut self, module_name: &str) -> usize {
        let keys_to_remove: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.metadata.module_name == module_name)
            .map(|(k, _)| k.clone())
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.remove_entry(&key);
        }

        count
    }

    /// Invalidate entries by source hash.
    pub fn invalidate_by_hash(&mut self, source_hash: &str) -> usize {
        let keys_to_remove: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.metadata.source_hash == source_hash)
            .map(|(k, _)| k.clone())
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.remove_entry(&key);
        }

        count
    }

    /// Clean up expired entries.
    pub fn cleanup(&mut self) -> usize {
        let keys_to_remove: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.is_expired(self.default_ttl_secs))
            .map(|(k, _)| k.clone())
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.remove_entry(&key);
            self.stats.record_expired();
        }

        count
    }
}

impl Default for BeamCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let cache = BeamCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = BeamCache::new();
        let key = CacheKey::for_module("mod", b"source", &[]);

        assert!(cache.insert(&key, vec![1, 2, 3], "mod", "hash"));
        assert!(cache.contains(&key));
        assert_eq!(cache.get(&key), Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_cache_get_after_insert() {
        let mut cache = BeamCache::new();
        let key = CacheKey::for_module("mod", b"source", &[]);

        cache.insert(&key, vec![1, 2, 3], "mod", "hash");
        let data = cache.get(&key);
        assert_eq!(data, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = BeamCache::new();
        let key = CacheKey::for_module("mod", b"source", &[]);

        cache.insert(&key, vec![1, 2, 3], "mod", "hash");
        assert!(cache.contains(&key));

        cache.remove(&key);
        assert!(!cache.contains(&key));
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = BeamCache::new();
        let key = CacheKey::for_module("mod", b"source", &[]);

        cache.insert(&key, vec![1, 2, 3], "mod", "hash");
        cache.get(&key);
        cache.get(&key);
        cache.get(&key);

        let stats = cache.stats();
        assert_eq!(stats.hits, 3);
    }

    #[test]
    fn test_cache_invalidate_module() {
        let mut cache = BeamCache::new();
        let key1 = CacheKey::for_module("mod1", b"source1", &[]);
        let key2 = CacheKey::for_module("mod2", b"source2", &[]);

        cache.insert(&key1, vec![1], "mod1", "hash1");
        cache.insert(&key2, vec![2], "mod2", "hash2");

        let count = cache.invalidate_module("mod1");
        assert_eq!(count, 1);
        assert!(!cache.contains(&key1));
        assert!(cache.contains(&key2));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = BeamCache::new();
        let key = CacheKey::for_module("mod", b"source", &[]);

        cache.insert(&key, vec![1, 2, 3], "mod", "hash");
        assert!(!cache.is_empty());

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_len() {
        let mut cache = BeamCache::new();
        let key1 = CacheKey::for_module("mod1", b"source1", &[]);
        let key2 = CacheKey::for_module("mod2", b"source2", &[]);

        assert_eq!(cache.len(), 0);

        cache.insert(&key1, vec![1], "mod1", "hash1");
        assert_eq!(cache.len(), 1);

        cache.insert(&key2, vec![2], "mod2", "hash2");
        assert_eq!(cache.len(), 2);
    }
}
