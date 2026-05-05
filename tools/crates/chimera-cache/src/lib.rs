//! Chimera build cache
//!
//! Implements build-cache keys for objects, metadata, wrapper source, and proof results.
//!
//! # Public API
//!
//! This crate exposes a stable public API through this module. All public items are
//! documented and considered part of the API guarantee. Internal items are marked
//! as `pub(crate)` or kept private.
//!
//! ## Core Types
//! - [`CacheKey`] - Cache key computation result with full component tracking
//! - [`CacheEntry`] - Cache entry metadata
//! - [`CacheResult`] - Cache hit/miss/stale result
//! - [`TypedArtifactCacheEntry`] - Typed wrapper for cached artifacts
//!
//! ## Cache Operations
//! - [`Cache`] - Main cache struct for get/put/invalidate operations
//! - [`CacheEvictionPolicy`] - Configuration for LRU eviction
//!
//! ## Error Handling
//! - [`CacheError`] - Error types for cache operations
//! - [`RecoveryAction`] - Actions for corruption recovery
//!
//! ## Reuse Analysis
//! - [`ReuseCheckResult`] - Result of artifact reuse checking
//! - [`ReuseReason`] - Reason for reuse decision
//! - [`check_artifact_reuse()`] - Check if artifact can be reused
//! - [`check_air_body_reuse()`] - Check AIR function body reuse
//! - [`check_comptime_reuse()`] - Check comptime value reuse
//! - [`check_generic_reuse()`] - Check generic instantiation reuse
//! - [`BatchReuseResult`] - Aggregate reuse statistics
//!
//! # Private Items
//!
//! The following are internal implementation details and subject to change:
//! - `EvictionCandidate` struct
//! - Helper functions for integrity checking
//! - Internal module organization

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use zigmera_schema::{
    ArtifactKind, ZAIRPACK_MAGIC, ZCHMETA_MAGIC, ZCHPROOF_MAGIC, ZDEP_MAGIC, ZSNAP_MAGIC,
};

/// Cache key computation result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKey {
    pub key: String,
    pub sources_hash: String,
    pub target_hash: String,
    pub abi_version_hash: String,
    pub toolchain_version_hash: String,
    pub metadata_version_hash: String,
    pub proof_version_hash: String,
    pub time_hash: Option<String>,
    /// Artifact kind (zsnap, zdep, zairpack, etc.)
    pub artifact_kind: Option<String>,
    /// Schema version for the artifact format
    pub schema_version: Option<u32>,
    /// Zig compiler commit hash
    pub zig_commit: Option<String>,
    /// Backend identifier (e.g., "c", "cpp", "spirv")
    pub backend: Option<String>,
    /// Optimization mode (e.g., "debug", "release")
    pub optimize_mode: Option<String>,
    /// Build options hash (compiler flags, etc.)
    pub build_options_hash: Option<String>,
    /// Semantic fingerprint hash
    pub semantic_fingerprint: Option<String>,
    /// Dependency fingerprint hashes
    pub dependency_fingerprints: Vec<String>,
    /// Component identity
    pub component_id: Option<String>,
    /// Manifest schema version (v0.1, v0.2)
    pub manifest_schema_version: Option<String>,
    /// Language backend version (e.g., clang-17, rustc-1.75)
    pub language_backend_version: Option<String>,
    /// Wrapper generation policy
    pub wrapper_policy: Option<String>,
    /// Proof verification policy
    pub proof_policy: Option<String>,
    /// Runtime delivery mode
    pub runtime_delivery_mode: Option<String>,
}

impl CacheKey {
    pub fn new(sources_hash: &str, target_hash: &str) -> Self {
        Self {
            key: Self::compute_key(sources_hash, target_hash, "", "", "", ""),
            sources_hash: sources_hash.to_string(),
            target_hash: target_hash.to_string(),
            abi_version_hash: String::new(),
            toolchain_version_hash: String::new(),
            metadata_version_hash: String::new(),
            proof_version_hash: String::new(),
            time_hash: None,
            artifact_kind: None,
            schema_version: None,
            zig_commit: None,
            backend: None,
            optimize_mode: None,
            build_options_hash: None,
            semantic_fingerprint: None,
            dependency_fingerprints: Vec::new(),
            component_id: None,
            manifest_schema_version: None,
            language_backend_version: None,
            wrapper_policy: None,
            proof_policy: None,
            runtime_delivery_mode: None,
        }
    }

    pub fn with_versions(
        mut self,
        abi_version: &str,
        toolchain_version: &str,
        metadata_version: &str,
        proof_version: &str,
    ) -> Self {
        self.abi_version_hash = Self::hash_string(abi_version);
        self.toolchain_version_hash = Self::hash_string(toolchain_version);
        self.metadata_version_hash = Self::hash_string(metadata_version);
        self.proof_version_hash = Self::hash_string(proof_version);
        self.key = Self::compute_key(
            &self.sources_hash,
            &self.target_hash,
            &self.abi_version_hash,
            &self.toolchain_version_hash,
            &self.metadata_version_hash,
            &self.proof_version_hash,
        );
        self
    }

    pub fn with_time(mut self, time_hash: &str) -> Self {
        self.time_hash = Some(time_hash.to_string());
        self
    }

    pub fn with_component_info(
        mut self,
        component_id: &str,
        schema_version: &str,
        backend_version: &str,
    ) -> Self {
        self.component_id = Some(component_id.to_string());
        self.manifest_schema_version = Some(schema_version.to_string());
        self.language_backend_version = Some(backend_version.to_string());
        self
    }

    pub fn with_policies(
        mut self,
        wrapper_policy: &str,
        proof_policy: &str,
        runtime_mode: &str,
    ) -> Self {
        self.wrapper_policy = Some(wrapper_policy.to_string());
        self.proof_policy = Some(proof_policy.to_string());
        self.runtime_delivery_mode = Some(runtime_mode.to_string());
        self
    }

    /// Set all cache key components at once (Task 67)
    pub fn with_full_components(
        mut self,
        artifact_kind: &str,
        schema_version: u32,
        zig_commit: &str,
        backend: &str,
        optimize_mode: &str,
        build_options: &str,
        semantic_fingerprint: &str,
        dependency_fingerprints: &[&str],
    ) -> Self {
        self.artifact_kind = Some(artifact_kind.to_string());
        self.schema_version = Some(schema_version);
        self.zig_commit = Some(zig_commit.to_string());
        self.backend = Some(backend.to_string());
        self.optimize_mode = Some(optimize_mode.to_string());
        self.build_options_hash = Some(Self::hash_string(build_options));
        self.semantic_fingerprint = Some(semantic_fingerprint.to_string());
        self.dependency_fingerprints = dependency_fingerprints
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        // Recompute the full key
        self.key = self.compute_full_key();
        self
    }

    /// Compute the full cache key including all components
    fn compute_full_key(&self) -> String {
        let mut hasher = Sha256::new();

        // Base components
        hasher.update(self.sources_hash.as_bytes());
        hasher.update(self.target_hash.as_bytes());

        // Version components
        if !self.abi_version_hash.is_empty() {
            hasher.update(self.abi_version_hash.as_bytes());
        }
        if !self.toolchain_version_hash.is_empty() {
            hasher.update(self.toolchain_version_hash.as_bytes());
        }
        if !self.metadata_version_hash.is_empty() {
            hasher.update(self.metadata_version_hash.as_bytes());
        }
        if !self.proof_version_hash.is_empty() {
            hasher.update(self.proof_version_hash.as_bytes());
        }

        // Extended components (Task 67)
        if let Some(ref artifact_kind) = self.artifact_kind {
            hasher.update(artifact_kind.as_bytes());
        }
        if let Some(schema_version) = self.schema_version {
            hasher.update(&schema_version.to_le_bytes());
        }
        if let Some(ref zig_commit) = self.zig_commit {
            hasher.update(zig_commit.as_bytes());
        }
        if let Some(ref backend) = self.backend {
            hasher.update(backend.as_bytes());
        }
        if let Some(ref optimize_mode) = self.optimize_mode {
            hasher.update(optimize_mode.as_bytes());
        }
        if let Some(ref build_options_hash) = self.build_options_hash {
            hasher.update(build_options_hash.as_bytes());
        }
        if let Some(ref semantic_fp) = self.semantic_fingerprint {
            hasher.update(semantic_fp.as_bytes());
        }
        for dep_fp in &self.dependency_fingerprints {
            hasher.update(dep_fp.as_bytes());
        }

        hex::encode(&hasher.finalize()[..16])
    }

    fn hash_string(s: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        hex::encode(&hasher.finalize()[..8]) // First 8 bytes = 16 hex chars
    }

    fn compute_key(
        sources: &str,
        target: &str,
        abi_version: &str,
        toolchain_version: &str,
        metadata_version: &str,
        proof_version: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(sources.as_bytes());
        hasher.update(target.as_bytes());
        if !abi_version.is_empty() {
            hasher.update(abi_version.as_bytes());
        }
        if !toolchain_version.is_empty() {
            hasher.update(toolchain_version.as_bytes());
        }
        if !metadata_version.is_empty() {
            hasher.update(metadata_version.as_bytes());
        }
        if !proof_version.is_empty() {
            hasher.update(proof_version.as_bytes());
        }
        let result = hasher.finalize();
        hex::encode(&result[..16]) // First 16 bytes = 32 hex chars
    }

    pub fn from_file(path: &Path) -> Result<Self, CacheError> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|_| CacheError::InvalidFormat)
    }

    pub fn save(&self, path: &Path) -> Result<(), CacheError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| CacheError::SerializationFailed(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }
}

/// Cache entry metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub artifact_path: PathBuf,
    pub created_at: SystemTime,
    pub source_paths: Vec<PathBuf>,
    pub target: String,
}

impl CacheEntry {
    pub fn new(
        key: String,
        artifact_path: PathBuf,
        source_paths: Vec<PathBuf>,
        target: String,
    ) -> Self {
        Self {
            key,
            artifact_path,
            created_at: SystemTime::now(),
            source_paths,
            target,
        }
    }

    pub fn is_expired(&self, max_age: Duration) -> bool {
        let age = SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or(Duration::ZERO);
        age > max_age
    }
}

/// Cache hit/miss result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheResult {
    Hit(CacheEntry),
    Miss(CacheKey),
    Stale(CacheEntry),
}

/// Typed artifact cache entry for specific artifact kinds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedArtifactCacheEntry {
    /// Base cache entry
    pub base: CacheEntry,
    /// Kind of artifact stored
    pub artifact_kind: ArtifactKind,
    /// Original size of the artifact
    pub artifact_size: u64,
    /// Checksum of artifact contents (BLAKE3 hex)
    pub checksum: String,
    /// Schema version if applicable (for versioned artifacts)
    pub schema_version: Option<u32>,
    /// Whether this entry is protected from eviction
    pub protected: bool,
}

impl TypedArtifactCacheEntry {
    /// Create a new typed artifact cache entry
    pub fn new(
        base: CacheEntry,
        artifact_kind: ArtifactKind,
        artifact_size: u64,
        checksum: String,
    ) -> Self {
        Self {
            base,
            artifact_kind,
            artifact_size,
            checksum,
            schema_version: None,
            protected: false,
        }
    }

    /// Set schema version for versioned artifacts
    pub fn with_schema_version(mut self, version: u32) -> Self {
        self.schema_version = Some(version);
        self
    }

    /// Mark entry as protected from eviction
    pub fn protected(mut self) -> Self {
        self.protected = true;
        self
    }

    /// Check if artifact kind matches
    pub fn matches_kind(&self, kind: ArtifactKind) -> bool {
        self.artifact_kind == kind
    }

    /// Verify artifact integrity by checking magic bytes
    pub fn verify_magic(&self, data: &[u8]) -> bool {
        let expected_magic = match self.artifact_kind {
            ArtifactKind::Zsnap => ZSNAP_MAGIC,
            ArtifactKind::Zdep => ZDEP_MAGIC,
            ArtifactKind::Zairpack => ZAIRPACK_MAGIC,
            ArtifactKind::Zchmeta => ZCHMETA_MAGIC,
            ArtifactKind::Zchproof => ZCHPROOF_MAGIC,
            ArtifactKind::Chobject | ArtifactKind::Chir => return true,
        };
        data.len() >= 8 && &data[0..8] == expected_magic
    }
}

/// Build cache
#[derive(Debug)]
pub struct Cache {
    cache_dir: PathBuf,
    entries: HashMap<String, CacheEntry>,
    max_age: Duration,
}

/// Cache eviction policy configuration
#[derive(Debug, Clone)]
pub struct CacheEvictionPolicy {
    /// Maximum cache size in bytes (0 = unlimited)
    pub max_size_bytes: u64,
    /// Maximum number of entries (0 = unlimited)
    pub max_entries: usize,
    /// Whether to partition by target
    pub partition_by_target: bool,
    /// Whether to partition by profile (debug/release)
    pub partition_by_profile: bool,
    /// Minimum age before eviction (0 = evict any age)
    pub min_age: Duration,
}

impl Default for CacheEvictionPolicy {
    fn default() -> Self {
        Self {
            max_size_bytes: 1024 * 1024 * 1024, // 1 GB default
            max_entries: 100_000,
            partition_by_target: true,
            partition_by_profile: true,
            min_age: Duration::ZERO,
        }
    }
}

impl CacheEvictionPolicy {
    /// Create a new policy with a size cap
    pub fn with_size_cap(mut self, size_bytes: u64) -> Self {
        self.max_size_bytes = size_bytes;
        self
    }

    /// Create a new policy with an entry limit
    pub fn with_entry_limit(mut self, max_entries: usize) -> Self {
        self.max_entries = max_entries;
        self
    }

    /// Enable target partitioning
    pub fn with_target_partitioning(mut self) -> Self {
        self.partition_by_target = true;
        self
    }

    /// Enable profile partitioning
    pub fn with_profile_partitioning(mut self) -> Self {
        self.partition_by_profile = true;
        self
    }
}

/// Entry with access metadata for LRU tracking
#[derive(Debug, Clone)]
pub struct EvictionCandidate {
    key: String,
    last_accessed: SystemTime,
    size_bytes: u64,
    target: String,
    protected: bool,
}

impl Cache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            entries: HashMap::new(),
            max_age: Duration::from_secs(7 * 24 * 3600), // 7 days default
        }
    }

    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = max_age;
        self
    }

    /// Initialize cache from disk
    pub fn load(&mut self) -> Result<(), CacheError> {
        if !self.cache_dir.exists() {
            fs::create_dir_all(&self.cache_dir)?;
            return Ok(());
        }

        // Scan cache directory for entries
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("cache") {
                if let Ok(cache_entry) = CacheEntry::from_path(&path) {
                    self.entries.insert(cache_entry.key.clone(), cache_entry);
                }
            }
        }

        Ok(())
    }

    /// Compute cache key for sources with versions
    pub fn compute_key(
        &self,
        sources: &[&Path],
        target: &str,
        abi_version: &str,
        toolchain_version: &str,
        metadata_version: &str,
        proof_version: &str,
    ) -> Result<CacheKey, CacheError> {
        // Hash all source files
        let mut hasher = Sha256::new();
        for source in sources {
            let content = fs::read(source).map_err(|e| CacheError::IOError(e.to_string()))?;
            hasher.update(&content);
        }
        let sources_hash = hex::encode(hasher.finalize());

        // Hash target
        let mut target_hasher = Sha256::new();
        target_hasher.update(target.as_bytes());
        let target_hash = hex::encode(target_hasher.finalize());

        let mut key = CacheKey::new(&sources_hash, &target_hash);
        key = key.with_versions(
            abi_version,
            toolchain_version,
            metadata_version,
            proof_version,
        );

        Ok(key)
    }

    /// Check if cache hit exists
    pub fn check(&self, key: &CacheKey) -> CacheResult {
        if let Some(entry) = self.entries.get(&key.key) {
            if entry.is_expired(self.max_age) {
                CacheResult::Stale(entry.clone())
            } else {
                CacheResult::Hit(entry.clone())
            }
        } else {
            CacheResult::Miss(key.clone())
        }
    }

    /// Store artifact in cache
    pub fn store(
        &mut self,
        key: CacheKey,
        artifact_path: &Path,
        source_paths: Vec<PathBuf>,
        target: &str,
    ) -> Result<(), CacheError> {
        // Copy artifact to cache
        let cached_path = self.cache_dir.join(format!("{}.artifact", key.key));
        fs::create_dir_all(&self.cache_dir)?;
        fs::copy(artifact_path, &cached_path).map_err(|e| CacheError::IOError(e.to_string()))?;

        // Create and save entry
        let entry = CacheEntry::new(
            key.key.clone(),
            cached_path,
            source_paths,
            target.to_string(),
        );

        let entry_path = self.cache_dir.join(format!("{}.cache", key.key));
        entry.save(&entry_path)?;

        self.entries.insert(key.key, entry);
        Ok(())
    }

    /// Get cached artifact path
    pub fn get(&self, key: &str) -> Option<PathBuf> {
        self.entries.get(key).map(|e| e.artifact_path.clone())
    }

    /// Invalidate cache entry
    pub fn invalidate(&mut self, key: &str) {
        if let Some(entry) = self.entries.remove(key) {
            let _ = fs::remove_file(&entry.artifact_path);
            let cache_path = self.cache_dir.join(format!("{}.cache", key));
            let _ = fs::remove_file(cache_path);
        }
    }

    /// Select entries for eviction based on policy
    pub fn select_eviction_candidates(
        &self,
        policy: &CacheEvictionPolicy,
        protected_keys: &std::collections::HashSet<String>,
    ) -> Vec<EvictionCandidate> {
        let mut candidates: Vec<EvictionCandidate> = Vec::new();
        let total_size: u64 = self
            .entries
            .values()
            .map(|e| fs::metadata(&e.artifact_path).map(|m| m.len()).unwrap_or(0))
            .sum();
        let total_entries = self.entries.len();

        let size_exceeded = policy.max_size_bytes > 0 && total_size > policy.max_size_bytes;
        let entries_exceeded = policy.max_entries > 0 && total_entries > policy.max_entries;

        if !size_exceeded && !entries_exceeded {
            return candidates;
        }

        for (key, entry) in &self.entries {
            if protected_keys.contains(key) {
                continue;
            }

            let size = fs::metadata(&entry.artifact_path)
                .map(|m| m.len())
                .unwrap_or(0);

            candidates.push(EvictionCandidate {
                key: key.clone(),
                last_accessed: entry.created_at,
                size_bytes: size,
                target: entry.target.clone(),
                protected: false,
            });
        }

        // Sort by last accessed (oldest first) for LRU
        candidates.sort_by(|a, b| a.last_accessed.cmp(&b.last_accessed));
        candidates
    }

    /// Evict entries based on policy, respecting protected entries
    pub fn evict_by_policy(
        &mut self,
        policy: &CacheEvictionPolicy,
        protected_keys: &std::collections::HashSet<String>,
    ) -> Vec<String> {
        let candidates = self.select_eviction_candidates(policy, protected_keys);
        let mut evicted = Vec::new();
        let mut current_size: u64 = self
            .entries
            .values()
            .map(|e| fs::metadata(&e.artifact_path).map(|m| m.len()).unwrap_or(0))
            .sum();
        let mut current_count = self.entries.len();

        for candidate in candidates {
            // Check if we need to evict more
            let size_ok = policy.max_size_bytes == 0 || current_size <= policy.max_size_bytes;
            let count_ok = policy.max_entries == 0 || current_count <= policy.max_entries;

            if size_ok && count_ok {
                break;
            }

            // Evict this candidate
            if let Some(entry) = self.entries.remove(&candidate.key) {
                let _ = fs::remove_file(&entry.artifact_path);
                let cache_path = self.cache_dir.join(format!("{}.cache", candidate.key));
                let _ = fs::remove_file(cache_path);
                current_size = current_size.saturating_sub(candidate.size_bytes);
                current_count -= 1;
                evicted.push(candidate.key);
            }
        }

        evicted
    }

    /// Get entries partitioned by target
    pub fn entries_by_target(&self) -> std::collections::HashMap<String, Vec<&CacheEntry>> {
        let mut by_target: std::collections::HashMap<String, Vec<&CacheEntry>> =
            std::collections::HashMap::new();
        for entry in self.entries.values() {
            by_target
                .entry(entry.target.clone())
                .or_default()
                .push(entry);
        }
        by_target
    }

    /// Clear entire cache
    pub fn clear(&mut self) {
        for entry in self.entries.values() {
            let _ = fs::remove_file(&entry.artifact_path);
        }
        self.entries.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_size: u64 = self
            .entries
            .values()
            .map(|e| fs::metadata(&e.artifact_path).map(|m| m.len()).unwrap_or(0))
            .sum();

        CacheStats {
            entry_count: self.entries.len(),
            total_size_bytes: total_size,
            cache_dir: self.cache_dir.clone(),
        }
    }
}

impl CacheEntry {
    pub fn from_path(path: &Path) -> Result<Self, CacheError> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|_| CacheError::InvalidFormat)
    }

    pub fn save(&self, path: &Path) -> Result<(), CacheError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| CacheError::SerializationFailed(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub entry_count: usize,
    pub total_size_bytes: u64,
    pub cache_dir: PathBuf,
}

/// Cache errors
#[derive(Debug, Clone)]
pub enum CacheError {
    IOError(String),
    InvalidFormat,
    SerializationFailed(String),
    KeyNotFound,
    ChecksumMismatch { expected: String, actual: String },
    CorruptedEntry(String),
    PartialWrite,
    OrphanArtifact(String),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::IOError(s) => write!(f, "I/O error: {}", s),
            CacheError::InvalidFormat => write!(f, "invalid cache format"),
            CacheError::SerializationFailed(s) => write!(f, "serialization failed: {}", s),
            CacheError::KeyNotFound => write!(f, "key not found in cache"),
            CacheError::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "checksum mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            CacheError::CorruptedEntry(s) => write!(f, "corrupted cache entry: {}", s),
            CacheError::PartialWrite => write!(f, "partial write detected"),
            CacheError::OrphanArtifact(s) => write!(f, "orphan artifact: {}", s),
        }
    }
}

/// Cache corruption recovery result
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// Entry is valid, use as-is
    Keep,
    /// Entry can be recovered by rebuilding
    Rebuild,
    /// Entry is corrupted, quarantine it
    Quarantine,
    /// Entry is orphaned, delete it
    Delete,
}

/// Check if a cache entry's artifact is corrupted
pub fn check_artifact_integrity(
    artifact_path: &Path,
    expected_checksum: &str,
) -> Result<RecoveryAction, CacheError> {
    if !artifact_path.exists() {
        return Err(CacheError::CorruptedEntry(
            "artifact file missing".to_string(),
        ));
    }

    let data = fs::read(artifact_path)?;
    let actual_checksum = compute_blake3_checksum(&data);

    if actual_checksum != expected_checksum {
        return Ok(RecoveryAction::Quarantine);
    }

    // Check if file is truncated (common partial write scenario)
    if let Ok(meta) = fs::metadata(artifact_path) {
        if meta.len() == 0 {
            return Ok(RecoveryAction::Quarantine);
        }
    }

    Ok(RecoveryAction::Keep)
}

/// Compute BLAKE3 checksum of data
fn compute_blake3_checksum(data: &[u8]) -> String {
    let mut hasher = zigmera_hash::Blake3Hasher::with_schema_tag("cache-integrity");
    hasher.update(data);
    hasher.finalize().as_hex()
}

/// Find orphaned artifacts (artifacts without corresponding cache entries)
pub fn find_orphan_artifacts(
    cache_dir: &Path,
    valid_keys: &std::collections::HashSet<String>,
) -> Vec<PathBuf> {
    let mut orphans = Vec::new();

    if let Ok(entries) = fs::read_dir(cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "artifact" {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if !valid_keys.contains(stem) {
                            orphans.push(path);
                        }
                    }
                }
            }
        }
    }

    orphans
}

/// Quarantine a corrupted entry
pub fn quarantine_entry(
    artifact_path: &Path,
    quarantine_dir: &Path,
) -> Result<PathBuf, CacheError> {
    fs::create_dir_all(quarantine_dir)?;

    let filename = artifact_path
        .file_name()
        .ok_or_else(|| CacheError::CorruptedEntry("invalid filename".to_string()))?;

    let quarantine_path = quarantine_dir.join(filename);

    // Move to quarantine instead of deleting (preserves evidence)
    fs::rename(artifact_path, &quarantine_path).map_err(|e| CacheError::IOError(e.to_string()))?;

    Ok(quarantine_path)
}

impl std::error::Error for CacheError {}

impl From<std::io::Error> for CacheError {
    fn from(e: std::io::Error) -> Self {
        CacheError::IOError(e.to_string())
    }
}

/// Compute cache key for proof results
pub fn compute_proof_key(
    source_paths: &[&Path],
    target: &str,
    proof_results: &[(String, bool)], // (obligation_id, verified)
) -> Result<CacheKey, CacheError> {
    let mut hasher = Sha256::new();

    for path in source_paths {
        let content = fs::read(path)?;
        hasher.update(&content);
    }

    hasher.update(target.as_bytes());

    for (id, verified) in proof_results {
        hasher.update(id.as_bytes());
        hasher.update(&[*verified as u8]);
    }

    let hash = hex::encode(hasher.finalize());
    Ok(CacheKey::new(&hash, target))
}

/// Check if source files changed since cache entry
pub fn sources_changed_since(entry: &CacheEntry, sources: &[&Path]) -> bool {
    for source in sources {
        if let Ok(meta) = fs::metadata(source) {
            if let Ok(modified) = meta.modified() {
                if modified > entry.created_at {
                    return true;
                }
            }
        }
    }
    false
}

/// Partial artifact reuse checking
#[derive(Debug, Clone)]
pub struct ReuseCheckResult {
    /// Whether artifact can be reused
    pub can_reuse: bool,
    /// Reason for the decision
    pub reason: ReuseReason,
    /// List of changed components (for debugging)
    pub changed_components: Vec<String>,
}

impl ReuseCheckResult {
    /// Create a successful reuse result
    pub fn reusable(reason: ReuseReason) -> Self {
        Self {
            can_reuse: true,
            reason,
            changed_components: Vec::new(),
        }
    }

    /// Create a non-reusable result
    pub fn not_reusable(reason: ReuseReason, changed: Vec<String>) -> Self {
        Self {
            can_reuse: false,
            reason,
            changed_components: changed,
        }
    }
}

/// Reason for reuse decision
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReuseReason {
    /// Artifact is unchanged and can be reused
    Unchanged,
    /// Only unrelated functions changed, this artifact is unaffected
    UnrelatedFunctionChanged,
    /// Only layout changed (ABI-preserving)
    LayoutChangedOnly,
    /// Type layout changed but not used by this artifact
    UnaffectedTypeChanged,
    /// Generic instantiation unchanged
    GenericUnchanged,
    /// Comptime value unchanged
    ComptimeUnchanged,
    /// Changed component affects this artifact
    AffectedByChange,
    /// No fingerprint available for comparison
    NoFingerprint,
}

/// Check if a specific artifact can be reused given changed fingerprints
pub fn check_artifact_reuse(
    artifact_id: &str,
    artifact_fingerprint: &str,
    changed_fingerprints: &std::collections::HashMap<String, String>,
    affected_nodes: &std::collections::HashSet<String>,
) -> ReuseCheckResult {
    // If artifact fingerprint is not in changed list, it's unchanged
    if !changed_fingerprints.contains_key(artifact_id) {
        return ReuseCheckResult::reusable(ReuseReason::Unchanged);
    }

    // If artifact is in affected nodes, it must be rebuilt
    if affected_nodes.contains(artifact_id) {
        return ReuseCheckResult::not_reusable(
            ReuseReason::AffectedByChange,
            vec![artifact_id.to_string()],
        );
    }

    // Artifact fingerprint changed but it's not in affected nodes
    // This can happen for layout-only changes that don't affect ABI
    let changed_value = changed_fingerprints.get(artifact_id);
    if let Some(fp) = changed_value {
        // Check if only layout changed (we'd need to store layout-only info)
        // For now, assume any change to an unknown artifact is affecting
        return ReuseCheckResult::not_reusable(
            ReuseReason::AffectedByChange,
            vec![artifact_id.to_string()],
        );
    }

    ReuseCheckResult::reusable(ReuseReason::Unchanged)
}

/// Check if an AIR body can be reused
pub fn check_air_body_reuse(
    func_id: &str,
    func_fingerprint: &str,
    previous_fingerprint: Option<&str>,
    changed_functions: &std::collections::HashSet<String>,
) -> ReuseCheckResult {
    // Function not changed at all - can reuse
    if !changed_functions.contains(func_id) {
        return ReuseCheckResult::reusable(ReuseReason::Unchanged);
    }

    // Function changed - need to rebuild
    ReuseCheckResult::not_reusable(ReuseReason::AffectedByChange, vec![func_id.to_string()])
}

/// Check if a comptime value can be reused
pub fn check_comptime_reuse(
    comptime_id: &str,
    comptime_value_hash: &str,
    previous_hash: Option<&str>,
    changed_comptime: &std::collections::HashSet<String>,
) -> ReuseCheckResult {
    if !changed_comptime.contains(comptime_id) {
        return ReuseCheckResult::reusable(ReuseReason::ComptimeUnchanged);
    }

    ReuseCheckResult::not_reusable(ReuseReason::AffectedByChange, vec![comptime_id.to_string()])
}

/// Check if a generic instantiation can be reused
pub fn check_generic_reuse(
    generic_id: &str,
    type_fingerprint: &str,
    previous_fingerprint: Option<&str>,
    changed_types: &std::collections::HashSet<String>,
) -> ReuseCheckResult {
    if !changed_types.contains(generic_id) {
        return ReuseCheckResult::reusable(ReuseReason::GenericUnchanged);
    }

    ReuseCheckResult::not_reusable(ReuseReason::AffectedByChange, vec![generic_id.to_string()])
}

/// Check if an object section can be reused
pub fn check_object_section_reuse(
    section_name: &str,
    section_fingerprint: &str,
    changed_sections: &std::collections::HashSet<String>,
) -> ReuseCheckResult {
    if !changed_sections.contains(section_name) {
        return ReuseCheckResult::reusable(ReuseReason::Unchanged);
    }

    ReuseCheckResult::not_reusable(
        ReuseReason::AffectedByChange,
        vec![section_name.to_string()],
    )
}

/// Aggregate reuse check for multiple artifacts
pub fn check_batch_reuse(artifacts: &[(&str, &str, ReuseCheckResult)]) -> BatchReuseResult {
    let mut reusable_count = 0;
    let mut non_reusable_count = 0;
    let mut reasons: std::collections::HashMap<ReuseReason, usize> =
        std::collections::HashMap::new();

    for (_, _, result) in artifacts {
        if result.can_reuse {
            reusable_count += 1;
        } else {
            non_reusable_count += 1;
        }
        *reasons.entry(result.reason.clone()).or_insert(0) += 1;
    }

    BatchReuseResult {
        total: artifacts.len(),
        reusable_count,
        non_reusable_count,
        reasons,
    }
}

/// Batch reuse statistics
#[derive(Debug, Clone)]
pub struct BatchReuseResult {
    pub total: usize,
    pub reusable_count: usize,
    pub non_reusable_count: usize,
    pub reasons: std::collections::HashMap<ReuseReason, usize>,
}

impl BatchReuseResult {
    /// Overall reuse ratio
    pub fn reuse_ratio(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.reusable_count as f64 / self.total as f64
        }
    }

    /// Check if batch is mostly reusable (>80%)
    pub fn is_mostly_reusable(&self) -> bool {
        self.reuse_ratio() > 0.8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_new() {
        let key = CacheKey::new("abc123", "x86_64");
        assert_eq!(key.sources_hash, "abc123");
        assert_eq!(key.target_hash, "x86_64");
        assert!(!key.key.is_empty());
    }

    #[test]
    fn test_cache_key_deterministic() {
        let key1 = CacheKey::new("source", "target");
        let key2 = CacheKey::new("source", "target");
        assert_eq!(key1.key, key2.key);
    }

    #[test]
    fn test_cache_key_different_inputs() {
        let key1 = CacheKey::new("source1", "target");
        let key2 = CacheKey::new("source2", "target");
        assert_ne!(key1.key, key2.key);
    }

    #[test]
    fn test_cache_new() {
        let cache = Cache::new(PathBuf::from("/tmp/test-cache"));
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_cache_check_miss() {
        let cache = Cache::new(PathBuf::from("/tmp/test-cache"));
        let key = CacheKey::new("source", "target");
        let result = cache.check(&key);
        assert!(matches!(result, CacheResult::Miss(_)));
    }

    #[test]
    fn test_cache_store_and_retrieve() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let mut cache = Cache::new(cache_dir.clone());

        // Create a source file
        let source = temp_dir.path().join("source.txt");
        fs::write(&source, "test content").unwrap();

        // Create an artifact
        let artifact = temp_dir.path().join("artifact.o");
        fs::write(&artifact, "artifact content").unwrap();

        let key = cache
            .compute_key(
                &[&source],
                "x86_64-unknown-linux-gnu",
                "0.1.0", // abi_version
                "1.75",  // toolchain_version
                "0.1.0", // metadata_version
                "0.1.0", // proof_version
            )
            .unwrap();

        cache
            .store(
                key.clone(),
                &artifact,
                vec![source.clone()],
                "x86_64-unknown-linux-gnu",
            )
            .unwrap();

        let result = cache.check(&key);
        assert!(matches!(result, CacheResult::Hit(_)));
    }

    #[test]
    fn test_cache_invalidate() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let mut cache = Cache::new(cache_dir);

        let source = temp_dir.path().join("source.txt");
        fs::write(&source, "test").unwrap();

        let artifact = temp_dir.path().join("artifact.o");
        fs::write(&artifact, "content").unwrap();

        let key = cache
            .compute_key(&[&source], "x86_64", "0.1.0", "1.75", "0.1.0", "0.1.0")
            .unwrap();
        cache
            .store(key.clone(), &artifact, vec![source], "x86_64")
            .unwrap();

        cache.invalidate(&key.key);
        assert!(cache.get(&key.key).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let mut cache = Cache::new(cache_dir);

        let source = temp_dir.path().join("source.txt");
        fs::write(&source, "test").unwrap();

        let artifact = temp_dir.path().join("artifact.o");
        fs::write(&artifact, "content").unwrap();

        let key = cache
            .compute_key(&[&source], "x86_64", "0.1.0", "1.75", "0.1.0", "0.1.0")
            .unwrap();
        cache
            .store(key.clone(), &artifact, vec![source], "x86_64")
            .unwrap();

        assert_eq!(cache.entries.len(), 1);
        cache.clear();
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let cache = Cache::new(cache_dir);

        let stats = cache.stats();
        assert_eq!(stats.entry_count, 0);
        assert_eq!(stats.total_size_bytes, 0);
    }

    #[test]
    fn test_cache_entry_expired() {
        let entry = CacheEntry::new(
            "key".to_string(),
            PathBuf::from("artifact.o"),
            vec![],
            "x86_64".to_string(),
        );

        // Fresh entry should not be expired
        assert!(!entry.is_expired(Duration::from_secs(3600)));
    }

    #[test]
    fn test_cache_key_with_time() {
        let key = CacheKey::new("source", "target");
        let key_with_time = key.with_time("time123");
        assert!(key_with_time.time_hash.is_some());
    }

    #[test]
    fn test_cache_key_save_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let key_path = temp_dir.path().join("key.cache");

        let key = CacheKey::new("source", "target");
        key.save(&key_path).unwrap();

        let loaded = CacheKey::from_file(&key_path).unwrap();
        assert_eq!(loaded.key, key.key);
    }

    #[test]
    fn test_cache_key_with_versions() {
        let key = CacheKey::new("source_hash", "target_hash");
        let key_with_versions = key.with_versions("abi_v1", "toolchain_v1", "meta_v1", "proof_v1");

        assert!(!key_with_versions.abi_version_hash.is_empty());
        assert!(!key_with_versions.toolchain_version_hash.is_empty());
        assert!(!key_with_versions.metadata_version_hash.is_empty());
        assert!(!key_with_versions.proof_version_hash.is_empty());

        // Version hashes should be deterministic
        let key2 = CacheKey::new("source_hash", "target_hash");
        let key2_with_versions =
            key2.with_versions("abi_v1", "toolchain_v1", "meta_v1", "proof_v1");
        assert_eq!(
            key_with_versions.abi_version_hash,
            key2_with_versions.abi_version_hash
        );
    }

    #[test]
    fn test_cache_key_versions_change_key() {
        let key1 = CacheKey::new("source", "target")
            .with_versions("abi_v1", "tool_v1", "meta_v1", "proof_v1");
        let key2 = CacheKey::new("source", "target")
            .with_versions("abi_v2", "tool_v1", "meta_v1", "proof_v1");

        // Different ABI versions should produce different keys
        assert_ne!(key1.key, key2.key);
    }

    #[test]
    fn test_compute_proof_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        let source = temp_dir.path().join("source.ch");
        fs::write(&source, "source content").unwrap();

        let results = vec![
            ("obligation_1".to_string(), true),
            ("obligation_2".to_string(), false),
        ];

        let key = compute_proof_key(&[&source], "x86_64", &results).unwrap();
        assert!(!key.key.is_empty());
    }

    #[test]
    fn test_typed_artifact_cache_entry_new() {
        let base = CacheEntry::new(
            "test_key".to_string(),
            PathBuf::from("/tmp/artifact.zsnap"),
            vec![PathBuf::from("/tmp/source.zig")],
            "x86_64-linux".to_string(),
        );
        let entry =
            TypedArtifactCacheEntry::new(base, ArtifactKind::Zsnap, 1024, "abc123".to_string());

        assert_eq!(entry.artifact_kind, ArtifactKind::Zsnap);
        assert_eq!(entry.artifact_size, 1024);
        assert_eq!(entry.checksum, "abc123");
        assert!(!entry.protected);
        assert!(entry.schema_version.is_none());
    }

    #[test]
    fn test_typed_artifact_cache_entry_with_schema_version() {
        let base = CacheEntry::new(
            "test_key".to_string(),
            PathBuf::from("/tmp/artifact.zsnap"),
            vec![],
            "x86_64-linux".to_string(),
        );
        let entry = TypedArtifactCacheEntry::new(base, ArtifactKind::Zsnap, 512, "xyz".to_string())
            .with_schema_version(1);

        assert_eq!(entry.schema_version, Some(1));
    }

    #[test]
    fn test_typed_artifact_cache_entry_protected() {
        let base = CacheEntry::new(
            "test_key".to_string(),
            PathBuf::from("/tmp/artifact.zsnap"),
            vec![],
            "x86_64-linux".to_string(),
        );
        let entry = TypedArtifactCacheEntry::new(base, ArtifactKind::Zsnap, 256, "chk".to_string())
            .protected();

        assert!(entry.protected);
    }

    #[test]
    fn test_typed_artifact_cache_entry_matches_kind() {
        let base = CacheEntry::new(
            "test_key".to_string(),
            PathBuf::from("/tmp/artifact.zdep"),
            vec![],
            "x86_64-linux".to_string(),
        );
        let entry = TypedArtifactCacheEntry::new(base, ArtifactKind::Zdep, 128, "chk".to_string());

        assert!(entry.matches_kind(ArtifactKind::Zdep));
        assert!(!entry.matches_kind(ArtifactKind::Zsnap));
    }

    #[test]
    fn test_typed_artifact_cache_entry_verify_magic() {
        let base = CacheEntry::new(
            "test_key".to_string(),
            PathBuf::from("/tmp/artifact.zsnap"),
            vec![],
            "x86_64-linux".to_string(),
        );
        let entry = TypedArtifactCacheEntry::new(base, ArtifactKind::Zsnap, 64, "chk".to_string());

        // Valid ZSNAP magic bytes
        let valid_zsnap = b"ZSNAP001xxxx";
        assert!(entry.verify_magic(valid_zsnap));

        // Invalid magic bytes
        let invalid = b"INVALID00xxxx";
        assert!(!entry.verify_magic(invalid));

        // Too short
        assert!(!entry.verify_magic(b"ZSNAP"));
    }

    #[test]
    fn test_typed_artifact_cache_entry_verify_magic_chir() {
        let base = CacheEntry::new(
            "test_key".to_string(),
            PathBuf::from("/tmp/artifact.chir"),
            vec![],
            "x86_64-linux".to_string(),
        );
        let entry = TypedArtifactCacheEntry::new(base, ArtifactKind::Chir, 64, "chk".to_string());

        // Chir has no magic, always returns true
        assert!(entry.verify_magic(b"anything"));
        assert!(entry.verify_magic(&[]));
    }

    #[test]
    fn test_check_artifact_integrity_valid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let artifact_path = temp_dir.path().join("test.artifact");
        fs::write(&artifact_path, "test content").unwrap();

        // Compute expected checksum
        let data = fs::read(&artifact_path).unwrap();
        let expected = compute_blake3_checksum(&data);

        let result = check_artifact_integrity(&artifact_path, &expected).unwrap();
        assert!(matches!(result, RecoveryAction::Keep));
    }

    #[test]
    fn test_check_artifact_integrity_checksum_mismatch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let artifact_path = temp_dir.path().join("test.artifact");
        fs::write(&artifact_path, "test content").unwrap();

        let wrong_checksum = "wrong_checksum_value_00000000000000000000000000000000";

        let result = check_artifact_integrity(&artifact_path, wrong_checksum).unwrap();
        assert!(matches!(result, RecoveryAction::Quarantine));
    }

    #[test]
    fn test_check_artifact_integrity_missing_file() {
        let missing_path = PathBuf::from("/tmp/nonexistent_artifact_12345");

        let result = check_artifact_integrity(&missing_path, "any_checksum");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_orphan_artifacts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_dir = temp_dir.path();

        // Create some artifacts
        fs::write(cache_dir.join("key1.artifact"), "content1").unwrap();
        fs::write(cache_dir.join("key2.artifact"), "content2").unwrap();
        fs::write(cache_dir.join("key3.artifact"), "content3").unwrap();

        // Valid keys exclude key2
        let valid_keys: std::collections::HashSet<String> =
            ["key1".to_string(), "key3".to_string()]
                .into_iter()
                .collect();

        let orphans = find_orphan_artifacts(cache_dir, &valid_keys);

        assert_eq!(orphans.len(), 1);
        assert!(orphans[0].to_string_lossy().contains("key2"));
    }

    #[test]
    fn test_quarantine_entry() {
        let temp_dir = tempfile::tempdir().unwrap();
        let artifact_path = temp_dir.path().join("corrupted.artifact");
        fs::write(&artifact_path, "corrupted content").unwrap();

        let quarantine_dir = temp_dir.path().join("quarantine");
        let result = quarantine_entry(&artifact_path, &quarantine_dir).unwrap();

        // Original should be moved
        assert!(!artifact_path.exists());
        // Quarantine copy should exist
        assert!(result.exists());
        assert!(result.to_string_lossy().contains("corrupted"));
    }

    #[test]
    fn test_eviction_policy_default() {
        let policy = CacheEvictionPolicy::default();
        assert_eq!(policy.max_size_bytes, 1024 * 1024 * 1024);
        assert_eq!(policy.max_entries, 100_000);
        assert!(policy.partition_by_target);
        assert!(policy.partition_by_profile);
    }

    #[test]
    fn test_eviction_policy_with_size_cap() {
        let policy = CacheEvictionPolicy::default().with_size_cap(500 * 1024 * 1024);
        assert_eq!(policy.max_size_bytes, 500 * 1024 * 1024);
    }

    #[test]
    fn test_eviction_policy_with_entry_limit() {
        let policy = CacheEvictionPolicy::default().with_entry_limit(50_000);
        assert_eq!(policy.max_entries, 50_000);
    }

    #[test]
    fn test_eviction_policy_builder_methods() {
        let policy = CacheEvictionPolicy::default()
            .with_target_partitioning()
            .with_profile_partitioning();
        assert!(policy.partition_by_target);
        assert!(policy.partition_by_profile);
    }

    #[test]
    fn test_select_eviction_candidates_empty_cache() {
        let cache = Cache::new(PathBuf::from("/tmp/test-cache"));
        let policy = CacheEvictionPolicy::default();
        let protected: std::collections::HashSet<String> = std::collections::HashSet::new();

        let candidates = cache.select_eviction_candidates(&policy, &protected);
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_select_eviction_candidates_under_limit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let cache = Cache::new(cache_dir);
        let policy = CacheEvictionPolicy::default();
        let protected: std::collections::HashSet<String> = std::collections::HashSet::new();

        // With empty cache, no candidates even if policy has limits
        let candidates = cache.select_eviction_candidates(&policy, &protected);
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_evict_by_policy_empty_cache() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let mut cache = Cache::new(cache_dir);
        let policy = CacheEvictionPolicy::default();
        let protected: std::collections::HashSet<String> = std::collections::HashSet::new();

        let evicted = cache.evict_by_policy(&policy, &protected);
        assert!(evicted.is_empty());
    }

    #[test]
    fn test_entries_by_target_empty_cache() {
        let cache = Cache::new(PathBuf::from("/tmp/test-cache"));
        let by_target = cache.entries_by_target();
        assert!(by_target.is_empty());
    }

    #[test]
    fn test_entries_by_target_with_entries() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let mut cache = Cache::new(cache_dir);

        // Add entries with different targets
        let source1 = temp_dir.path().join("source1.txt");
        let source2 = temp_dir.path().join("source2.txt");
        fs::write(&source1, "test1").unwrap();
        fs::write(&source2, "test2").unwrap();

        let artifact1 = temp_dir.path().join("artifact1.o");
        let artifact2 = temp_dir.path().join("artifact2.o");
        fs::write(&artifact1, "content1").unwrap();
        fs::write(&artifact2, "content2").unwrap();

        let key1 = cache
            .compute_key(
                &[&source1],
                "x86_64-unknown-linux-gnu",
                "0.1.0",
                "1.75",
                "0.1.0",
                "0.1.0",
            )
            .unwrap();
        let key2 = cache
            .compute_key(
                &[&source2],
                "aarch64-unknown-linux-gnu",
                "0.1.0",
                "1.75",
                "0.1.0",
                "0.1.0",
            )
            .unwrap();

        cache
            .store(
                key1,
                &artifact1,
                vec![source1.clone()],
                "x86_64-unknown-linux-gnu",
            )
            .unwrap();
        cache
            .store(
                key2,
                &artifact2,
                vec![source2.clone()],
                "aarch64-unknown-linux-gnu",
            )
            .unwrap();

        let by_target = cache.entries_by_target();
        assert_eq!(by_target.len(), 2);
        assert!(by_target.contains_key("x86_64-unknown-linux-gnu"));
        assert!(by_target.contains_key("aarch64-unknown-linux-gnu"));
    }

    #[test]
    fn test_reuse_check_result_reusable() {
        let result = ReuseCheckResult::reusable(ReuseReason::Unchanged);
        assert!(result.can_reuse);
        assert!(result.changed_components.is_empty());
    }

    #[test]
    fn test_reuse_check_result_not_reusable() {
        let result = ReuseCheckResult::not_reusable(
            ReuseReason::AffectedByChange,
            vec!["fn:add".to_string()],
        );
        assert!(!result.can_reuse);
        assert_eq!(result.changed_components.len(), 1);
    }

    #[test]
    fn test_check_artifact_reuse_unchanged() {
        let changed: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let affected: std::collections::HashSet<String> = std::collections::HashSet::new();

        let result = check_artifact_reuse("fn:add", "hash123", &changed, &affected);
        assert!(result.can_reuse);
        assert_eq!(result.reason, ReuseReason::Unchanged);
    }

    #[test]
    fn test_check_artifact_reuse_affected() {
        let mut changed: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        changed.insert("fn:add".to_string(), "new_hash".to_string());

        let mut affected: std::collections::HashSet<String> = std::collections::HashSet::new();
        affected.insert("fn:add".to_string());

        let result = check_artifact_reuse("fn:add", "old_hash", &changed, &affected);
        assert!(!result.can_reuse);
        assert_eq!(result.reason, ReuseReason::AffectedByChange);
    }

    #[test]
    fn test_check_artifact_reuse_changed_not_affected() {
        // Fingerprint changed but not in affected set - edge case
        let mut changed: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        changed.insert("fn:add".to_string(), "new_hash".to_string());

        let affected: std::collections::HashSet<String> = std::collections::HashSet::new();

        let result = check_artifact_reuse("fn:add", "old_hash", &changed, &affected);
        assert!(!result.can_reuse);
    }

    #[test]
    fn test_check_air_body_reuse_unchanged() {
        let changed: std::collections::HashSet<String> = std::collections::HashSet::new();

        let result = check_air_body_reuse("fn:add", "hash123", None, &changed);
        assert!(result.can_reuse);
        assert_eq!(result.reason, ReuseReason::Unchanged);
    }

    #[test]
    fn test_check_air_body_reuse_changed() {
        let mut changed: std::collections::HashSet<String> = std::collections::HashSet::new();
        changed.insert("fn:add".to_string());

        let result = check_air_body_reuse("fn:add", "new_hash", Some("old_hash"), &changed);
        assert!(!result.can_reuse);
        assert_eq!(result.reason, ReuseReason::AffectedByChange);
    }

    #[test]
    fn test_check_comptime_reuse_unchanged() {
        let changed: std::collections::HashSet<String> = std::collections::HashSet::new();

        let result = check_comptime_reuse("comptime:table", "hash123", None, &changed);
        assert!(result.can_reuse);
        assert_eq!(result.reason, ReuseReason::ComptimeUnchanged);
    }

    #[test]
    fn test_check_comptime_reuse_changed() {
        let mut changed: std::collections::HashSet<String> = std::collections::HashSet::new();
        changed.insert("comptime:table".to_string());

        let result = check_comptime_reuse("comptime:table", "new_hash", Some("old_hash"), &changed);
        assert!(!result.can_reuse);
    }

    #[test]
    fn test_check_generic_reuse_unchanged() {
        let changed: std::collections::HashSet<String> = std::collections::HashSet::new();

        let result = check_generic_reuse("Point", "hash123", None, &changed);
        assert!(result.can_reuse);
        assert_eq!(result.reason, ReuseReason::GenericUnchanged);
    }

    #[test]
    fn test_check_generic_reuse_changed() {
        let mut changed: std::collections::HashSet<String> = std::collections::HashSet::new();
        changed.insert("Point".to_string());

        let result = check_generic_reuse("Point", "new_hash", Some("old_hash"), &changed);
        assert!(!result.can_reuse);
    }

    #[test]
    fn test_check_object_section_reuse_unchanged() {
        let changed: std::collections::HashSet<String> = std::collections::HashSet::new();

        let result = check_object_section_reuse(".text.fn_add", "hash123", &changed);
        assert!(result.can_reuse);
    }

    #[test]
    fn test_check_object_section_reuse_changed() {
        let mut changed: std::collections::HashSet<String> = std::collections::HashSet::new();
        changed.insert(".text.fn_add".to_string());

        let result = check_object_section_reuse(".text.fn_add", "new_hash", &changed);
        assert!(!result.can_reuse);
    }

    #[test]
    fn test_batch_reuse_result_reuse_ratio() {
        let result = BatchReuseResult {
            total: 10,
            reusable_count: 9,
            non_reusable_count: 1,
            reasons: std::collections::HashMap::new(),
        };
        assert_eq!(result.reuse_ratio(), 0.9);
        assert!(result.is_mostly_reusable());
    }

    #[test]
    fn test_batch_reuse_result_empty() {
        let result = BatchReuseResult {
            total: 0,
            reusable_count: 0,
            non_reusable_count: 0,
            reasons: std::collections::HashMap::new(),
        };
        assert_eq!(result.reuse_ratio(), 0.0);
        assert!(!result.is_mostly_reusable());
    }

    #[test]
    fn test_batch_reuse_result_not_mostly_reusable() {
        let result = BatchReuseResult {
            total: 10,
            reusable_count: 5,
            non_reusable_count: 5,
            reasons: std::collections::HashMap::new(),
        };
        assert_eq!(result.reuse_ratio(), 0.5);
        assert!(!result.is_mostly_reusable());
    }

    #[test]
    fn test_check_batch_reuse() {
        let artifacts = vec![
            (
                "fn:add",
                "hash1",
                ReuseCheckResult::reusable(ReuseReason::Unchanged),
            ),
            (
                "fn:sub",
                "hash2",
                ReuseCheckResult::reusable(ReuseReason::Unchanged),
            ),
            (
                "fn:mul",
                "hash3",
                ReuseCheckResult::not_reusable(
                    ReuseReason::AffectedByChange,
                    vec!["fn:mul".to_string()],
                ),
            ),
        ];

        let result = check_batch_reuse(&artifacts);
        assert_eq!(result.total, 3);
        assert_eq!(result.reusable_count, 2);
        assert_eq!(result.non_reusable_count, 1);
    }

    // Task 67: Cache key composition tests

    #[test]
    fn test_cache_key_with_full_components() {
        let key = CacheKey::new("sources123", "target456").with_full_components(
            "zairpack",
            1,
            "abc123def456",
            "c",
            "release",
            "-O3 -ffast-math",
            "semantic_fp_hash",
            &["dep1_fp", "dep2_fp"],
        );

        assert_eq!(key.artifact_kind, Some("zairpack".to_string()));
        assert_eq!(key.schema_version, Some(1));
        assert_eq!(key.zig_commit, Some("abc123def456".to_string()));
        assert_eq!(key.backend, Some("c".to_string()));
        assert_eq!(key.optimize_mode, Some("release".to_string()));
        assert!(key.build_options_hash.is_some());
        assert_eq!(
            key.semantic_fingerprint,
            Some("semantic_fp_hash".to_string())
        );
        assert_eq!(key.dependency_fingerprints.len(), 2);
    }

    #[test]
    fn test_cache_key_full_components_deterministic() {
        let key1 = CacheKey::new("sources123", "target456").with_full_components(
            "zairpack",
            1,
            "abc123def456",
            "c",
            "release",
            "-O3",
            "semantic_fp",
            &["dep1"],
        );

        let key2 = CacheKey::new("sources123", "target456").with_full_components(
            "zairpack",
            1,
            "abc123def456",
            "c",
            "release",
            "-O3",
            "semantic_fp",
            &["dep1"],
        );

        // Same inputs should produce same key
        assert_eq!(key1.key, key2.key);
    }

    #[test]
    fn test_cache_key_full_components_different_artifact_kind() {
        let key1 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit",
            "c",
            "release",
            "-O3",
            "fp",
            &[],
        );

        let key2 = CacheKey::new("sources", "target").with_full_components(
            "zsnap",
            1,
            "commit",
            "c",
            "release",
            "-O3",
            "fp",
            &[],
        );

        assert_ne!(key1.key, key2.key);
    }

    #[test]
    fn test_cache_key_full_components_different_schema_version() {
        let key1 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit",
            "c",
            "release",
            "-O3",
            "fp",
            &[],
        );

        let key2 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            2,
            "commit",
            "c",
            "release",
            "-O3",
            "fp",
            &[],
        );

        assert_ne!(key1.key, key2.key);
    }

    #[test]
    fn test_cache_key_full_components_different_zig_commit() {
        let key1 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit1",
            "c",
            "release",
            "-O3",
            "fp",
            &[],
        );

        let key2 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit2",
            "c",
            "release",
            "-O3",
            "fp",
            &[],
        );

        assert_ne!(key1.key, key2.key);
    }

    #[test]
    fn test_cache_key_full_components_different_backend() {
        let key1 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit",
            "c",
            "release",
            "-O3",
            "fp",
            &[],
        );

        let key2 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit",
            "cpp",
            "release",
            "-O3",
            "fp",
            &[],
        );

        assert_ne!(key1.key, key2.key);
    }

    #[test]
    fn test_cache_key_full_components_different_optimize_mode() {
        let key1 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit",
            "c",
            "debug",
            "-O0",
            "fp",
            &[],
        );

        let key2 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit",
            "c",
            "release",
            "-O3",
            "fp",
            &[],
        );

        assert_ne!(key1.key, key2.key);
    }

    #[test]
    fn test_cache_key_full_components_different_build_options() {
        let key1 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit",
            "c",
            "release",
            "-O2",
            "fp",
            &[],
        );

        let key2 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit",
            "c",
            "release",
            "-O3",
            "fp",
            &[],
        );

        assert_ne!(key1.key, key2.key);
    }

    #[test]
    fn test_cache_key_full_components_different_semantic_fingerprint() {
        let key1 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit",
            "c",
            "release",
            "-O3",
            "fp1",
            &[],
        );

        let key2 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit",
            "c",
            "release",
            "-O3",
            "fp2",
            &[],
        );

        assert_ne!(key1.key, key2.key);
    }

    #[test]
    fn test_cache_key_full_components_different_dependency_fingerprints() {
        let key1 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit",
            "c",
            "release",
            "-O3",
            "fp",
            &["dep1"],
        );

        let key2 = CacheKey::new("sources", "target").with_full_components(
            "zairpack",
            1,
            "commit",
            "c",
            "release",
            "-O3",
            "fp",
            &["dep2"],
        );

        assert_ne!(key1.key, key2.key);
    }
}
