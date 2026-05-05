//! BEAM incremental cache for ChimeraIR.
//!
//! Provides caching for BEAM module analysis to enable
//! incremental compilation without reprocessing unchanged modules.

pub mod cache;
pub mod entry;
pub mod key;

pub use cache::BeamCache;
pub use entry::{CacheEntry, CacheMetadata, CacheStats};
pub use key::CacheKey;

/// Current cache format version.
pub const CACHE_VERSION: u32 = key::CACHE_VERSION;

/// Maximum cache entries per module.
pub const MAX_CACHE_ENTRIES: usize = 65536;

/// Default cache TTL in seconds (24 hours).
pub const DEFAULT_CACHE_TTL_SECS: u64 = 86400;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_version() {
        assert_eq!(CACHE_VERSION, 1);
    }

    #[test]
    fn test_max_cache_entries() {
        assert!(MAX_CACHE_ENTRIES > 0);
    }

    #[test]
    fn test_default_cache_ttl() {
        assert_eq!(DEFAULT_CACHE_TTL_SECS, 86400);
    }
}
