//! Cache entry for BEAM module analysis.
//!
//! Stores cached analysis results with metadata.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A cache entry containing analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Cache key.
    pub key: String,
    /// Cached data (serialized).
    pub data: Vec<u8>,
    /// Entry metadata.
    pub metadata: CacheMetadata,
}

impl CacheEntry {
    /// Create a new entry.
    pub fn new(key: impl Into<String>, data: Vec<u8>) -> Self {
        CacheEntry {
            key: key.into(),
            data,
            metadata: CacheMetadata::new(),
        }
    }

    /// Create with custom metadata.
    pub fn with_metadata(key: impl Into<String>, data: Vec<u8>, metadata: CacheMetadata) -> Self {
        CacheEntry {
            key: key.into(),
            data,
            metadata,
        }
    }

    /// Check if entry is valid.
    pub fn is_valid(&self) -> bool {
        self.metadata.is_valid()
    }

    /// Check if entry is expired.
    pub fn is_expired(&self, ttl_secs: u64) -> bool {
        self.metadata.is_expired(ttl_secs)
    }

    /// Get data size.
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Metadata for a cache entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Creation timestamp.
    pub created_at: u64,
    /// Last access timestamp.
    pub last_accessed: u64,
    /// Access count.
    pub access_count: u64,
    /// Original module name.
    pub module_name: String,
    /// Original source hash.
    pub source_hash: String,
    /// Entry version.
    pub version: u32,
}

impl CacheMetadata {
    /// Create new metadata.
    pub fn new() -> Self {
        let now = current_time_secs();
        CacheMetadata {
            created_at: now,
            last_accessed: now,
            access_count: 0,
            module_name: String::new(),
            source_hash: String::new(),
            version: 1,
        }
    }

    /// Create with module info.
    pub fn with_module_info(
        module_name: impl Into<String>,
        source_hash: impl Into<String>,
    ) -> Self {
        let now = current_time_secs();
        CacheMetadata {
            created_at: now,
            last_accessed: now,
            access_count: 0,
            module_name: module_name.into(),
            source_hash: source_hash.into(),
            version: 1,
        }
    }

    /// Update last accessed time.
    pub fn touch(&mut self) {
        self.last_accessed = current_time_secs();
        self.access_count += 1;
    }

    /// Check if valid based on version.
    pub fn is_valid(&self) -> bool {
        self.version == 1
    }

    /// Check if expired.
    pub fn is_expired(&self, ttl_secs: u64) -> bool {
        let now = current_time_secs();
        now - self.created_at > ttl_secs
    }

    /// Get age in seconds.
    pub fn age_secs(&self) -> u64 {
        current_time_secs() - self.created_at
    }
}

impl Default for CacheMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total entries.
    pub total_entries: u64,
    /// Total size in bytes.
    pub total_size: u64,
    /// Hits.
    pub hits: u64,
    /// Misses.
    pub misses: u64,
    /// Evictions.
    pub evictions: u64,
    /// Expired entries.
    pub expired: u64,
}

impl CacheStats {
    /// Create new stats.
    pub fn new() -> Self {
        CacheStats::default()
    }

    /// Record a hit.
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Record a miss.
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// Record an eviction.
    pub fn record_eviction(&mut self, size: u64) {
        self.evictions += 1;
        self.total_size = self.total_size.saturating_sub(size);
    }

    /// Record an entry addition.
    pub fn record_add(&mut self, size: u64) {
        self.total_entries += 1;
        self.total_size += size;
    }

    /// Record expiration.
    pub fn record_expired(&mut self) {
        self.expired += 1;
    }

    /// Get hit rate.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Check if cache is healthy.
    pub fn is_healthy(&self) -> bool {
        self.hit_rate() >= 0.5 || self.total_entries == 0
    }
}

fn current_time_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_new() {
        let entry = CacheEntry::new("key", vec![1, 2, 3]);
        assert_eq!(entry.key, "key");
        assert_eq!(entry.data, vec![1, 2, 3]);
        assert!(entry.is_valid());
    }

    #[test]
    fn test_cache_entry_expired() {
        let entry = CacheEntry::new("key", vec![1, 2, 3]);
        // Entry just created, should not be expired with 0 ttl
        assert!(!entry.is_expired(0)); // 0 ttl = immediately expired, but created now
    }

    #[test]
    fn test_cache_metadata_new() {
        let meta = CacheMetadata::new();
        assert_eq!(meta.access_count, 0);
        assert!(meta.is_valid());
    }

    #[test]
    fn test_cache_metadata_touch() {
        let mut meta = CacheMetadata::new();
        assert_eq!(meta.access_count, 0);
        meta.touch();
        assert_eq!(meta.access_count, 1);
    }

    #[test]
    fn test_cache_stats_record_hit() {
        let mut stats = CacheStats::new();
        stats.record_hit();
        stats.record_hit();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let mut stats = CacheStats::new();
        stats.record_hit();
        stats.record_hit();
        stats.record_miss();
        // 2 hits / 3 total = 0.666...
        assert!((stats.hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_cache_stats_empty_hit_rate() {
        let stats = CacheStats::new();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_stats_healthy() {
        let mut stats = CacheStats::new();
        stats.record_hit();
        stats.record_hit();
        assert!(stats.is_healthy());
    }
}
