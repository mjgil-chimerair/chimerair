//! Fallback External-Zig Mode
//!
//! This module provides a fallback adapter when the patched Zig compiler is unavailable.
//! It supports two modes:
//! 1. Fixture mode - uses pre-recorded `.zsnap` files
//! 2. Cache-scrape mode - extracts information from Zig build cache
//!
//! # Trust Boundary
//!
//! All outputs from fallback mode are marked as **non-authoritative** because
//! they do not come from the verified patched compiler. The adapter must not
//! allow fallback outputs to be marked as production-complete.
//!
//! # Limitations
//!
//! Fallback mode has these constraints compared to patched-Zig mode:
//! - No real semantic analysis (Sema) data
//! - No real AIR (Aircraft Intermediate Representation)
//! - No type/layout information from InternPool
//! - No export/link metadata
//! - No comptime evaluation tracking
//! - No incremental invalidation precision
//!
//! Outputs should only be used for development, testing, or as a last resort.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Marker indicating the authority level of adapter output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthorityLevel {
    /// Output from the patched Zig compiler (authoritative)
    Authoritative,
    /// Output from fixture files (non-authoritative)
    Fixture,
    /// Output from cache scraping (non-authoritative)
    CacheScrape,
    /// Output from unavailable compiler (non-authoritative)
    Unavailable,
}

impl AuthorityLevel {
    /// Returns true if this authority level is authoritative
    pub fn is_authoritative(&self) -> bool {
        matches!(self, AuthorityLevel::Authoritative)
    }

    /// Returns true if this authority level is non-authoritative
    pub fn is_non_authoritative(&self) -> bool {
        !self.is_authoritative()
    }

    /// Returns a description of what this authority level means
    pub fn description(&self) -> &'static str {
        match self {
            AuthorityLevel::Authoritative => {
                "Output from the patched Zig compiler with verified semantic analysis"
            }
            AuthorityLevel::Fixture => {
                "Output from pre-recorded fixture files; not verified against current source"
            }
            AuthorityLevel::CacheScrape => {
                "Output extracted from Zig build cache; may be incomplete or stale"
            }
            AuthorityLevel::Unavailable => {
                "Output when patched Zig is unavailable; use with extreme caution"
            }
        }
    }

    /// Returns a warning message for non-authoritative output
    pub fn warning_message(&self) -> Option<&'static str> {
        if self.is_non_authoritative() {
            Some(
                "WARNING: This output is non-authoritative and should not be used \
                 for production builds or release decisions.",
            )
        } else {
            None
        }
    }
}

impl std::fmt::Display for AuthorityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthorityLevel::Authoritative => write!(f, "authoritative"),
            AuthorityLevel::Fixture => write!(f, "fixture"),
            AuthorityLevel::CacheScrape => write!(f, "cache-scrape"),
            AuthorityLevel::Unavailable => write!(f, "unavailable"),
        }
    }
}

/// Fallback mode type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackMode {
    /// Use pre-recorded fixture files
    FixtureOnly,
    /// Extract from Zig build cache
    CacheScrape,
    /// No patched Zig available
    Unavailable,
}

impl FallbackMode {
    /// Returns the authority level for this fallback mode
    pub fn authority_level(&self) -> AuthorityLevel {
        match self {
            FallbackMode::FixtureOnly => AuthorityLevel::Fixture,
            FallbackMode::CacheScrape => AuthorityLevel::CacheScrape,
            FallbackMode::Unavailable => AuthorityLevel::Unavailable,
        }
    }

    /// Returns a description of this fallback mode
    pub fn description(&self) -> &'static str {
        match self {
            FallbackMode::FixtureOnly => "Using pre-recorded fixture files for snapshot data",
            FallbackMode::CacheScrape => "Extracting snapshot data from Zig build cache",
            FallbackMode::Unavailable => "Patched Zig compiler unavailable; using minimal fallback",
        }
    }

    /// Returns the name of this fallback mode
    pub fn name(&self) -> &'static str {
        match self {
            FallbackMode::FixtureOnly => "fixture_only",
            FallbackMode::CacheScrape => "cache_scrape",
            FallbackMode::Unavailable => "unavailable",
        }
    }
}

impl std::fmt::Display for FallbackMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Information about why fallback mode was activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackReason {
    /// The fallback mode that was activated
    pub mode: FallbackMode,
    /// Why the patched Zig was not used
    pub details: String,
    /// Suggested action to resolve the issue
    pub suggested_action: String,
}

impl FallbackReason {
    /// Create a new fallback reason
    pub fn new(mode: FallbackMode, details: &str, suggested_action: &str) -> Self {
        Self {
            mode,
            details: details.to_string(),
            suggested_action: suggested_action.to_string(),
        }
    }

    /// Create a reason for fixture mode
    pub fn fixture_mode(fixture_path: &Path) -> Self {
        Self {
            mode: FallbackMode::FixtureOnly,
            details: format!("Using fixture from: {}", fixture_path.display()),
            suggested_action: "Ensure fixture files are up-to-date with source".to_string(),
        }
    }

    /// Create a reason for cache scrape mode
    pub fn cache_scrape(cache_path: &Path) -> Self {
        Self {
            mode: FallbackMode::CacheScrape,
            details: format!("Scraping cache from: {}", cache_path.display()),
            suggested_action: "Build project with Zig to generate fresh cache".to_string(),
        }
    }

    /// Create a reason for unavailable patched Zig
    pub fn patched_zig_unavailable(zig_path: Option<&Path>) -> Self {
        let details = if let Some(path) = zig_path {
            format!("Patched Zig not found at: {}", path.display())
        } else {
            "Patched Zig not found in PATH or configured location".to_string()
        };
        Self {
            mode: FallbackMode::Unavailable,
            details,
            suggested_action: "Install patched Zig or use fixture mode".to_string(),
        }
    }
}

/// Result of checking for patched Zig availability
#[derive(Debug, Clone)]
pub enum PatchedZigStatus {
    /// Patched Zig is available and can be used
    Available {
        path: PathBuf,
        version: String,
        supports_snapshot_flags: bool,
    },
    /// Patched Zig is not available, fallback required
    Unavailable { reason: FallbackReason },
}

/// Check if patched Zig is available
pub fn check_patched_zig() -> PatchedZigStatus {
    check_patched_zig_impl(None, None)
}

/// Check if patched Zig is available at a specific path
pub fn check_patched_zig_at(path: &Path) -> PatchedZigStatus {
    check_patched_zig_impl(Some(path), None)
}

/// Check if patched Zig is available with a specific zigzag binary
pub fn check_patched_zig_with(zigzag_path: &Path) -> PatchedZigStatus {
    check_patched_zig_impl(None, Some(zigzag_path))
}

fn check_patched_zig_impl(zig_path: Option<&Path>, zigzag_path: Option<&Path>) -> PatchedZigStatus {
    // In a real implementation, this would:
    // 1. Check if the configured patched Zig binary exists
    // 2. Run `zig version` to get version info
    // 3. Check if it supports --emit-zigmera-snapshot and other flags
    //
    // For now, return Unavailable to demonstrate fallback behavior
    let reason = if let Some(path) = zig_path {
        FallbackReason::patched_zig_unavailable(Some(path))
    } else if let Some(path) = zigzag_path {
        FallbackReason::patched_zig_unavailable(Some(path))
    } else {
        FallbackReason::patched_zig_unavailable(None)
    };

    PatchedZigStatus::Unavailable { reason }
}

/// A wrapper for adapter output that tracks authority level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityTagged<T> {
    /// The actual output data
    pub data: T,
    /// The authority level of this output
    pub authority: AuthorityLevel,
    /// Whether this output can be marked production-complete
    #[serde(default)]
    pub production_ready: bool,
}

impl<T> AuthorityTagged<T> {
    /// Create a new authority-tagged output
    pub fn new(data: T, authority: AuthorityLevel) -> Self {
        let production_ready = authority.is_authoritative();
        Self {
            data,
            authority,
            production_ready,
        }
    }

    /// Create an authoritative output
    pub fn authoritative(data: T) -> Self {
        Self {
            data,
            authority: AuthorityLevel::Authoritative,
            production_ready: true,
        }
    }

    /// Create a fixture-mode output
    pub fn fixture(data: T) -> Self {
        Self {
            data,
            authority: AuthorityLevel::Fixture,
            production_ready: false,
        }
    }

    /// Create a cache-scrape output
    pub fn cache_scrape(data: T) -> Self {
        Self {
            data,
            authority: AuthorityLevel::CacheScrape,
            production_ready: false,
        }
    }

    /// Create an unavailable-mode output
    pub fn unavailable(data: T) -> Self {
        Self {
            data,
            authority: AuthorityLevel::Unavailable,
            production_ready: false,
        }
    }

    /// Get a reference to the inner data
    pub fn inner(&self) -> &T {
        &self.data
    }

    /// Get a mutable reference to the inner data
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Convert to an Option, None if non-authoritative
    pub fn as_authoritative(self) -> Option<T> {
        if self.authority.is_authoritative() {
            Some(self.data)
        } else {
            None
        }
    }

    /// Map the inner data to a new type, preserving authority
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> AuthorityTagged<U> {
        AuthorityTagged {
            data: f(self.data),
            authority: self.authority,
            production_ready: self.production_ready,
        }
    }

    /// Get the authority level
    pub fn authority(&self) -> AuthorityLevel {
        self.authority
    }

    /// Check if this output is authoritative
    pub fn is_authoritative(&self) -> bool {
        self.authority.is_authoritative()
    }

    /// Check if this output can be used for production
    pub fn is_production_ready(&self) -> bool {
        self.production_ready
    }
}

impl<T> std::ops::Deref for AuthorityTagged<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> std::ops::DerefMut for AuthorityTagged<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// Fallback adapter for when patched Zig is unavailable
pub struct FallbackAdapter {
    mode: FallbackMode,
    authority: AuthorityLevel,
    fixture_paths: Vec<PathBuf>,
    cache_path: Option<PathBuf>,
}

impl FallbackAdapter {
    /// Create a new fallback adapter in fixture mode
    pub fn fixture_mode(fixture_paths: Vec<PathBuf>) -> Self {
        Self {
            mode: FallbackMode::FixtureOnly,
            authority: AuthorityLevel::Fixture,
            fixture_paths,
            cache_path: None,
        }
    }

    /// Create a new fallback adapter in cache-scrape mode
    pub fn cache_scrape_mode(cache_path: PathBuf) -> Self {
        Self {
            mode: FallbackMode::CacheScrape,
            authority: AuthorityLevel::CacheScrape,
            fixture_paths: Vec::new(),
            cache_path: Some(cache_path),
        }
    }

    /// Create a fallback adapter when patched Zig is unavailable
    pub fn unavailable() -> Self {
        Self {
            mode: FallbackMode::Unavailable,
            authority: AuthorityLevel::Unavailable,
            fixture_paths: Vec::new(),
            cache_path: None,
        }
    }

    /// Get the current fallback mode
    pub fn mode(&self) -> FallbackMode {
        self.mode
    }

    /// Get the authority level of outputs
    pub fn authority(&self) -> AuthorityLevel {
        self.authority
    }

    /// Get the fixture paths (if in fixture mode)
    pub fn fixture_paths(&self) -> &[PathBuf] {
        &self.fixture_paths
    }

    /// Get the cache path (if in cache-scrape mode)
    pub fn cache_path(&self) -> Option<&Path> {
        self.cache_path.as_deref()
    }

    /// Check if this adapter can provide authoritative output
    pub fn can_produce_authoritative(&self) -> bool {
        false // Fallback adapter never produces authoritative output
    }

    /// Get a warning message for non-authoritative output
    pub fn warning(&self) -> Option<&'static str> {
        self.authority.warning_message()
    }
}

impl std::fmt::Display for FallbackAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FallbackAdapter({})", self.mode)
    }
}

/// Trait for adapters that can produce authority-tagged output
pub trait AuthorityTaggable {
    /// The type of output this adapter produces
    type Output;

    /// Get the authority level of this adapter's output
    fn authority(&self) -> AuthorityLevel;

    /// Tag output with authority information
    fn tag_output(&self, data: Self::Output) -> AuthorityTagged<Self::Output> {
        AuthorityTagged::new(data, self.authority())
    }
}

impl AuthorityTaggable for FallbackAdapter {
    type Output = crate::snapshot::ZigSnapshot;

    fn authority(&self) -> AuthorityLevel {
        self.authority
    }
}

/// Statistics about fallback mode usage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FallbackStats {
    /// Number of times fixture mode was used
    pub fixture_mode_count: u64,
    /// Number of times cache-scrape mode was used
    pub cache_scrape_count: u64,
    /// Number of times unavailable mode was used
    pub unavailable_count: u64,
    /// Whether fallback was activated in current session
    pub fallback_active: bool,
}

impl FallbackStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a fixture mode activation
    pub fn record_fixture(&mut self) {
        self.fixture_mode_count += 1;
        self.fallback_active = true;
    }

    /// Record a cache-scrape mode activation
    pub fn record_cache_scrape(&mut self) {
        self.cache_scrape_count += 1;
        self.fallback_active = true;
    }

    /// Record an unavailable mode activation
    pub fn record_unavailable(&mut self) {
        self.unavailable_count += 1;
        self.fallback_active = true;
    }

    /// Record a fallback activation based on mode
    pub fn record(&mut self, mode: FallbackMode) {
        match mode {
            FallbackMode::FixtureOnly => self.record_fixture(),
            FallbackMode::CacheScrape => self.record_cache_scrape(),
            FallbackMode::Unavailable => self.record_unavailable(),
        }
    }

    /// Check if any fallback mode has been used
    pub fn has_used_fallback(&self) -> bool {
        self.fallback_active
    }

    /// Total number of fallback activations
    pub fn total_fallbacks(&self) -> u64 {
        self.fixture_mode_count + self.cache_scrape_count + self.unavailable_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authority_level_is_authoritative() {
        assert!(AuthorityLevel::Authoritative.is_authoritative());
        assert!(!AuthorityLevel::Fixture.is_authoritative());
        assert!(!AuthorityLevel::CacheScrape.is_authoritative());
        assert!(!AuthorityLevel::Unavailable.is_authoritative());
    }

    #[test]
    fn test_authority_level_is_non_authoritative() {
        assert!(!AuthorityLevel::Authoritative.is_non_authoritative());
        assert!(AuthorityLevel::Fixture.is_non_authoritative());
        assert!(AuthorityLevel::CacheScrape.is_non_authoritative());
        assert!(AuthorityLevel::Unavailable.is_non_authoritative());
    }

    #[test]
    fn test_authority_level_descriptions() {
        assert!(!AuthorityLevel::Authoritative.description().is_empty());
        assert!(!AuthorityLevel::Fixture.description().is_empty());
        assert!(!AuthorityLevel::CacheScrape.description().is_empty());
        assert!(!AuthorityLevel::Unavailable.description().is_empty());
    }

    #[test]
    fn test_authority_level_warning() {
        assert!(AuthorityLevel::Authoritative.warning_message().is_none());
        assert!(AuthorityLevel::Fixture.warning_message().is_some());
        assert!(AuthorityLevel::CacheScrape.warning_message().is_some());
        assert!(AuthorityLevel::Unavailable.warning_message().is_some());
    }

    #[test]
    fn test_fallback_mode_authority_level() {
        assert_eq!(
            FallbackMode::FixtureOnly.authority_level(),
            AuthorityLevel::Fixture
        );
        assert_eq!(
            FallbackMode::CacheScrape.authority_level(),
            AuthorityLevel::CacheScrape
        );
        assert_eq!(
            FallbackMode::Unavailable.authority_level(),
            AuthorityLevel::Unavailable
        );
    }

    #[test]
    fn test_fallback_reason_fixture() {
        let path = Path::new("/fixtures/snapshot.json");
        let reason = FallbackReason::fixture_mode(path);
        assert_eq!(reason.mode, FallbackMode::FixtureOnly);
        assert!(reason.details.contains("fixtures"));
        assert!(!reason.suggested_action.is_empty());
    }

    #[test]
    fn test_fallback_reason_cache_scrape() {
        let path = Path::new("/cache/zig");
        let reason = FallbackReason::cache_scrape(path);
        assert_eq!(reason.mode, FallbackMode::CacheScrape);
        assert!(reason.details.contains("cache"));
    }

    #[test]
    fn test_fallback_reason_unavailable() {
        let reason = FallbackReason::patched_zig_unavailable(None);
        assert_eq!(reason.mode, FallbackMode::Unavailable);
        assert!(reason.details.contains("not found"));
    }

    #[test]
    fn test_fallback_adapter_fixtures() {
        let paths = vec![PathBuf::from("/fixtures/test.json")];
        let adapter = FallbackAdapter::fixture_mode(paths.clone());
        assert_eq!(adapter.mode(), FallbackMode::FixtureOnly);
        assert_eq!(adapter.authority(), AuthorityLevel::Fixture);
        assert_eq!(adapter.fixture_paths(), paths.as_slice());
        assert!(!adapter.can_produce_authoritative());
    }

    #[test]
    fn test_fallback_adapter_cache_scrape() {
        let path = PathBuf::from("/cache/zig");
        let adapter = FallbackAdapter::cache_scrape_mode(path.clone());
        assert_eq!(adapter.mode(), FallbackMode::CacheScrape);
        assert_eq!(adapter.authority(), AuthorityLevel::CacheScrape);
        assert_eq!(adapter.cache_path(), Some(path.as_path()));
        assert!(!adapter.can_produce_authoritative());
    }

    #[test]
    fn test_fallback_adapter_unavailable() {
        let adapter = FallbackAdapter::unavailable();
        assert_eq!(adapter.mode(), FallbackMode::Unavailable);
        assert_eq!(adapter.authority(), AuthorityLevel::Unavailable);
        assert!(adapter.fixture_paths().is_empty());
        assert!(adapter.cache_path().is_none());
        assert!(!adapter.can_produce_authoritative());
    }

    #[test]
    fn test_authority_tagged_authoritative() {
        let tagged = AuthorityTagged::authoritative(42);
        assert_eq!(*tagged, 42);
        assert!(tagged.is_authoritative());
        assert!(tagged.is_production_ready());
        assert_eq!(tagged.as_authoritative(), Some(42));
    }

    #[test]
    fn test_authority_tagged_non_authoritative() {
        let tagged = AuthorityTagged::fixture(42);
        assert_eq!(*tagged, 42);
        assert!(!tagged.is_authoritative());
        assert!(!tagged.is_production_ready());
        assert_eq!(tagged.as_authoritative(), None);
    }

    #[test]
    fn test_authority_tagged_fixture() {
        let tagged = AuthorityTagged::fixture(42);
        assert_eq!(tagged.authority(), AuthorityLevel::Fixture);
    }

    #[test]
    fn test_authority_tagged_cache_scrape() {
        let tagged = AuthorityTagged::cache_scrape(42);
        assert_eq!(tagged.authority(), AuthorityLevel::CacheScrape);
    }

    #[test]
    fn test_authority_tagged_unavailable() {
        let tagged = AuthorityTagged::unavailable(42);
        assert_eq!(tagged.authority(), AuthorityLevel::Unavailable);
    }

    #[test]
    fn test_authority_tagged_map() {
        let tagged = AuthorityTagged::fixture(42);
        let mapped = tagged.map(|x| x * 2);
        assert_eq!(*mapped, 84);
        assert_eq!(mapped.authority(), AuthorityLevel::Fixture);
        assert!(!mapped.is_production_ready());
    }

    #[test]
    fn test_fallback_stats() {
        let mut stats = FallbackStats::new();
        assert!(!stats.has_used_fallback());

        stats.record_fixture();
        assert!(stats.has_used_fallback());
        assert_eq!(stats.fixture_mode_count, 1);
        assert_eq!(stats.total_fallbacks(), 1);

        stats.record_cache_scrape();
        assert_eq!(stats.cache_scrape_count, 1);
        assert_eq!(stats.total_fallbacks(), 2);

        stats.record_unavailable();
        assert_eq!(stats.unavailable_count, 1);
        assert_eq!(stats.total_fallbacks(), 3);
    }

    #[test]
    fn test_fallback_stats_record_by_mode() {
        let mut stats = FallbackStats::new();
        stats.record(FallbackMode::FixtureOnly);
        stats.record(FallbackMode::CacheScrape);
        stats.record(FallbackMode::Unavailable);
        assert_eq!(stats.total_fallbacks(), 3);
    }

    #[test]
    fn test_check_patched_zig_returns_unavailable() {
        // In test environment, patched Zig is not available
        let status = check_patched_zig();
        match status {
            PatchedZigStatus::Unavailable { reason } => {
                assert_eq!(reason.mode, FallbackMode::Unavailable);
            }
            PatchedZigStatus::Available { .. } => {
                panic!("Expected patched Zig to be unavailable in test");
            }
        }
    }

    #[test]
    fn test_check_patched_zig_at_returns_unavailable() {
        let path = Path::new("/nonexistent/zig");
        let status = check_patched_zig_at(path);
        match status {
            PatchedZigStatus::Unavailable { reason } => {
                assert!(reason.details.contains("/nonexistent/zig"));
            }
            PatchedZigStatus::Available { .. } => {
                panic!("Expected patched Zig to be unavailable");
            }
        }
    }

    #[test]
    fn test_authority_level_display() {
        assert_eq!(AuthorityLevel::Authoritative.to_string(), "authoritative");
        assert_eq!(AuthorityLevel::Fixture.to_string(), "fixture");
        assert_eq!(AuthorityLevel::CacheScrape.to_string(), "cache-scrape");
        assert_eq!(AuthorityLevel::Unavailable.to_string(), "unavailable");
    }

    #[test]
    fn test_fallback_mode_description() {
        assert!(!FallbackMode::FixtureOnly.description().is_empty());
        assert!(!FallbackMode::CacheScrape.description().is_empty());
        assert!(!FallbackMode::Unavailable.description().is_empty());
    }

    #[test]
    fn test_fallback_adapter_display() {
        let adapter = FallbackAdapter::unavailable();
        let display = format!("{}", adapter);
        assert!(display.contains("FallbackAdapter"));
        assert!(display.contains("unavailable"));
    }
}
