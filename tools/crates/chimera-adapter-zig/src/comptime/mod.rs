//! Zig Comptime Cache Model
//!
//! This module models Zig's comptime evaluation cache for incremental compilation.
//! Comptime (compile-time) evaluation is a key feature of Zig where functions can
//! be evaluated at compile time, with results embedded in the binary.
//!
//! # Cache Tracking
//!
//! The cache tracks:
//! - Comptime function bodies
//! - Arguments (types, values)
//! - Target architecture
//! - Builtins used
//! - Referenced declarations, types, layouts
//! - Embedded files consumed
//!
//! # Cache Key Structure
//!
//! A cache key consists of:
//! 1. Function identity (file, name, line)
//! 2. Argument values/types
//! 3. Target triple
//! 4. Builtin version
//! 5. Dependencies (decls, types, layouts, embeds)

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use zigmera_schema::zchproof::{CacheProofFact, ChproofHeader, ChproofSchema};

/// Key for identifying a comptime evaluation
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComptimeKey {
    /// File containing the comptime function
    pub file: String,
    /// Function name
    pub name: String,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Hash of argument values
    pub args_hash: String,
    /// Target triple
    pub target: String,
    /// Hash of builtin versions used
    pub builtins_hash: String,
}

impl ComptimeKey {
    /// Create a new comptime key
    pub fn new(file: &str, name: &str, line: u32, column: u32) -> Self {
        Self {
            file: file.to_string(),
            name: name.to_string(),
            line,
            column,
            args_hash: String::new(),
            target: String::new(),
            builtins_hash: String::new(),
        }
    }

    /// Set argument hash
    pub fn with_args(mut self, args_hash: &str) -> Self {
        self.args_hash = args_hash.to_string();
        self
    }

    /// Set target
    pub fn with_target(mut self, target: &str) -> Self {
        self.target = target.to_string();
        self
    }

    /// Set builtins hash
    pub fn with_builtins(mut self, builtins_hash: &str) -> Self {
        self.builtins_hash = builtins_hash.to_string();
        self
    }

    /// Generate a cache key string for storage
    pub fn cache_key(&self) -> String {
        let mut hasher = zigmera_hash::Blake3Hasher::with_schema_tag("comptime-key");
        hasher.update_str(&self.file);
        hasher.update_str(&self.name);
        hasher.update_u64(self.line as u64);
        hasher.update_u64(self.column as u64);
        hasher.update_str(&self.args_hash);
        hasher.update_str(&self.target);
        hasher.update_str(&self.builtins_hash);
        format!("comptime_{}", hasher.finalize().as_hex())
    }

    pub fn components(&self) -> CacheKeyComponents {
        CacheKeyComponents {
            file: self.file.clone(),
            name: self.name.clone(),
            line: self.line,
            column: self.column,
            args_hash: self.args_hash.clone(),
            target: self.target.clone(),
            builtins_hash: self.builtins_hash.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKeyComponents {
    pub file: String,
    pub name: String,
    pub line: u32,
    pub column: u32,
    pub args_hash: String,
    pub target: String,
    pub builtins_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheDependencyFingerprint {
    pub kind: String,
    pub id: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheReuseChecks {
    pub cached_entry_valid: bool,
    pub dep_graph_hash: String,
    pub build_options_hash: String,
    pub dependency_fingerprints: Vec<CacheDependencyFingerprint>,
    pub embed_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheExplainStatus {
    Hit,
    Miss,
    Rebuild,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CacheExplainReason {
    CacheHit,
    NoEntry,
    InvalidatedEntry,
    DependencyChanged {
        dependency_kind: String,
        dependency_id: String,
    },
    EmbedChanged {
        path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheExplanation {
    pub artifact_kind: String,
    pub cache_key: String,
    pub status: CacheExplainStatus,
    pub reason: CacheExplainReason,
    pub key_components: CacheKeyComponents,
    pub reuse_checks: CacheReuseChecks,
}

/// A cached comptime result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComptimeCacheEntry {
    /// The cache key
    pub key: ComptimeKey,
    /// The computed result type
    pub result_kind: ComptimeResultKind,
    /// Dependencies on other declarations
    pub dependencies: Vec<ComptimeDependency>,
    /// Embedded files consumed
    pub embed_files: Vec<String>,
    /// Size of the cached result (for memory estimation)
    pub result_size: u64,
    /// When this entry was created
    pub created_at: u64,
    /// Whether the result is still valid
    pub valid: bool,
}

impl ComptimeCacheEntry {
    /// Create a new cache entry
    pub fn new(key: ComptimeKey) -> Self {
        Self {
            key,
            result_kind: ComptimeResultKind::Unknown,
            dependencies: Vec::new(),
            embed_files: Vec::new(),
            result_size: 0,
            created_at: current_timestamp(),
            valid: true,
        }
    }

    /// Set the result kind
    pub fn with_result_kind(mut self, kind: ComptimeResultKind) -> Self {
        self.result_kind = kind;
        self
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, dep: ComptimeDependency) {
        self.dependencies.push(dep);
    }

    /// Add an embed file
    pub fn add_embed_file(&mut self, path: &str) {
        self.embed_files.push(path.to_string());
    }

    /// Invalidate this entry
    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    pub fn to_cache_proof_fact(
        &self,
        schema_version: u32,
        build_options_hash: [u8; 32],
        fallback_target: &str,
    ) -> CacheProofFact {
        CacheProofFact {
            cache_key: self.key.cache_key(),
            semantic_fingerprint: digest_string(&self.key.cache_key()),
            dependency_fingerprints: self
                .dependencies
                .iter()
                .map(|dep| digest_string(&dep.content_hash))
                .collect(),
            schema_version,
            target: if self.key.target.is_empty() {
                fallback_target.to_string()
            } else {
                self.key.target.clone()
            },
            build_options_hash,
            reusable: self.valid,
        }
    }
}

/// Kind of comptime result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComptimeResultKind {
    /// Result type not yet determined
    Unknown,
    /// Compile-time constant value
    Constant,
    /// Computed type
    Type,
    /// Computed layout
    Layout,
    /// Function pointer
    FunctionPointer,
    /// Void/unit
    Void,
}

/// A dependency of a comptime evaluation
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComptimeDependency {
    /// What kind of thing this is
    pub kind: ComptimeDepKind,
    /// The identifier (name/path)
    pub id: String,
    /// Hash of the dependency's content at evaluation time
    pub content_hash: String,
}

impl ComptimeDependency {
    /// Create a new dependency
    pub fn new(kind: ComptimeDepKind, id: &str, content_hash: &str) -> Self {
        Self {
            kind,
            id: id.to_string(),
            content_hash: content_hash.to_string(),
        }
    }
}

/// Kind of comptime dependency
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComptimeDepKind {
    /// Depends on a type declaration
    Type,
    /// Depends on a struct declaration
    Struct,
    /// Depends on a function
    Function,
    /// Depends on a constant
    Constant,
    /// Depends on a layout
    Layout,
    /// Uses a builtin function
    Builtin,
}

/// The comptime cache
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComptimeCache {
    /// All cache entries keyed by cache key
    entries: HashMap<String, ComptimeCacheEntry>,
    /// Index: file -> set of cache keys
    file_index: HashMap<String, Vec<String>>,
    /// Index: dependency -> set of cache keys
    dependency_index: HashMap<String, Vec<String>>,
    /// Global cache statistics
    stats: CacheStats,
}

impl ComptimeCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self::default()
    }

    /// Get an entry by cache key
    pub fn get(&self, key: &str) -> Option<&ComptimeCacheEntry> {
        self.entries.get(key)
    }

    /// Check if an entry exists and is valid
    pub fn is_valid(&self, key: &str) -> bool {
        self.entries.get(key).map(|e| e.valid).unwrap_or(false)
    }

    /// Insert a new cache entry
    pub fn insert(&mut self, entry: ComptimeCacheEntry) -> &ComptimeCacheEntry {
        let key = entry.key.cache_key();

        // Update dependency index
        for dep in &entry.dependencies {
            let dep_key = format!("{:?}:{}", dep.kind, dep.id);
            self.dependency_index
                .entry(dep_key)
                .or_default()
                .push(key.clone());
        }

        // Update file index
        self.file_index
            .entry(entry.key.file.clone())
            .or_default()
            .push(key.clone());

        // Insert entry
        self.stats.total_entries += 1;
        self.entries.insert(key.clone(), entry);
        self.entries
            .get(&key)
            .expect("cache entry should exist after insert")
    }

    /// Invalidate entries that depend on a given dependency
    pub fn invalidate_by_dependency(&mut self, kind: ComptimeDepKind, id: &str) -> Vec<String> {
        let dep_key = format!("{:?}:{}", kind, id);
        let keys = self.dependency_index.remove(&dep_key).unwrap_or_default();
        let mut invalidated = Vec::new();

        for key in keys {
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.invalidate();
                invalidated.push(key.clone());
            }
        }

        self.stats.invalidated_entries += invalidated.len() as u64;
        invalidated
    }

    /// Invalidate all entries in a file
    pub fn invalidate_file(&mut self, file: &str) -> Vec<String> {
        let keys = self.file_index.remove(file).unwrap_or_default();
        let mut invalidated = Vec::new();

        for key in keys {
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.invalidate();
                invalidated.push(key.clone());
            }
        }

        self.stats.invalidated_entries += invalidated.len() as u64;
        invalidated
    }

    /// Invalidate entries by a specific key pattern
    pub fn invalidate_by_pattern(&mut self, pattern: &str) -> Vec<String> {
        let mut invalidated = Vec::new();

        let keys: Vec<String> = self
            .entries
            .keys()
            .filter(|k| k.contains(pattern))
            .cloned()
            .collect();

        for key in keys {
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.invalidate();
                invalidated.push(key.clone());
            }
        }

        self.stats.invalidated_entries += invalidated.len() as u64;
        invalidated
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        let count = self.entries.len();
        self.entries.clear();
        self.file_index.clear();
        self.dependency_index.clear();
        self.stats.cleared_entries += count as u64;
    }

    /// Number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over valid entries
    pub fn valid_entries(&self) -> impl Iterator<Item = &ComptimeCacheEntry> {
        self.entries.values().filter(|e| e.valid)
    }

    /// Serialize the cache to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total entries ever inserted
    pub total_entries: u64,
    /// Entries that were invalidated
    pub invalidated_entries: u64,
    /// Entries cleared
    pub cleared_entries: u64,
}

/// Simple timestamp (would be actual system time in production)
fn current_timestamp() -> u64 {
    // Simplified - in production would use std::time::SystemTime
    0
}

fn digest_string(value: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hasher.finalize().into()
}

fn hex_digest(digest: [u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ============================================================================
// Comptime Evaluation Context
// ============================================================================

/// Context for tracking comptime evaluation
#[derive(Debug, Clone)]
pub struct ComptimeContext {
    /// The cache being used
    cache: ComptimeCache,
    /// Current target
    target: String,
    /// Builtin versions
    builtins: HashMap<String, String>,
    /// Hash of current dependency graph state
    dep_graph_hash: String,
    /// Simulated changed dependency set used by invalidation/explain checks.
    invalid_dependencies: HashSet<String>,
    /// Simulated changed embed-file set used by invalidation/explain checks.
    invalid_embed_files: HashSet<String>,
}

impl ComptimeContext {
    /// Create a new comptime context
    pub fn new(target: &str) -> Self {
        Self {
            cache: ComptimeCache::new(),
            target: target.to_string(),
            builtins: HashMap::new(),
            dep_graph_hash: String::new(),
            invalid_dependencies: HashSet::new(),
            invalid_embed_files: HashSet::new(),
        }
    }

    /// Get the cache
    pub fn cache(&self) -> &ComptimeCache {
        &self.cache
    }

    /// Get a mutable cache reference
    pub fn cache_mut(&mut self) -> &mut ComptimeCache {
        &mut self.cache
    }

    /// Get the current target triple.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Get the current dependency-graph hash.
    pub fn dep_graph_hash(&self) -> &str {
        &self.dep_graph_hash
    }

    /// Get a builtin version if one has been recorded.
    pub fn builtin_version(&self, builtin: &str) -> Option<&str> {
        self.builtins.get(builtin).map(String::as_str)
    }

    pub fn build_options_hash(&self) -> [u8; 32] {
        let mut builtins: Vec<_> = self.builtins.iter().collect();
        builtins.sort_by(|lhs, rhs| lhs.0.cmp(rhs.0));

        let mut hasher = Sha256::new();
        hasher.update(self.target.as_bytes());
        hasher.update(self.dep_graph_hash.as_bytes());
        for (builtin, version) in builtins {
            hasher.update(builtin.as_bytes());
            hasher.update(version.as_bytes());
        }
        hasher.finalize().into()
    }

    pub fn cache_proof_facts(&self, schema_version: u32) -> Vec<CacheProofFact> {
        let build_options_hash = self.build_options_hash();
        self.cache
            .entries
            .values()
            .map(|entry| {
                entry.to_cache_proof_fact(schema_version, build_options_hash, &self.target)
            })
            .collect()
    }

    pub fn proof_artifact(
        &self,
        zig_commit: [u8; 20],
        timestamp_ns: u64,
        schema_version: u32,
    ) -> ChproofSchema {
        let cache_facts = self.cache_proof_facts(schema_version);
        ChproofSchema {
            header: ChproofHeader {
                magic: *zigmera_schema::zchproof::ZCHPROOF_MAGIC,
                schema_version,
                zig_commit,
                target: self.target.clone(),
                timestamp_ns,
                proof_count: cache_facts.len() as u32,
                checksum: [0u8; 32],
            },
            obligations: vec![],
            cache_facts,
            invalidation_facts: vec![],
        }
    }

    pub fn explain_reuse(&self, key: &ComptimeKey) -> CacheExplanation {
        let cache_key = key.cache_key();
        let base_checks = CacheReuseChecks {
            cached_entry_valid: false,
            dep_graph_hash: self.dep_graph_hash.clone(),
            build_options_hash: hex_digest(self.build_options_hash()),
            dependency_fingerprints: Vec::new(),
            embed_files: Vec::new(),
        };

        let Some(entry) = self.cache.get(&cache_key) else {
            return CacheExplanation {
                artifact_kind: "comptime".to_string(),
                cache_key,
                status: CacheExplainStatus::Miss,
                reason: CacheExplainReason::NoEntry,
                key_components: key.components(),
                reuse_checks: base_checks,
            };
        };

        let reuse_checks = CacheReuseChecks {
            cached_entry_valid: entry.valid,
            dep_graph_hash: self.dep_graph_hash.clone(),
            build_options_hash: hex_digest(self.build_options_hash()),
            dependency_fingerprints: entry
                .dependencies
                .iter()
                .map(|dep| CacheDependencyFingerprint {
                    kind: format!("{:?}", dep.kind),
                    id: dep.id.clone(),
                    content_hash: dep.content_hash.clone(),
                })
                .collect(),
            embed_files: entry.embed_files.clone(),
        };

        if !entry.valid {
            return CacheExplanation {
                artifact_kind: "comptime".to_string(),
                cache_key,
                status: CacheExplainStatus::Rebuild,
                reason: CacheExplainReason::InvalidatedEntry,
                key_components: entry.key.components(),
                reuse_checks,
            };
        }

        for dep in &entry.dependencies {
            if !self.verify_dependency_still_valid(dep) {
                return CacheExplanation {
                    artifact_kind: "comptime".to_string(),
                    cache_key,
                    status: CacheExplainStatus::Rebuild,
                    reason: CacheExplainReason::DependencyChanged {
                        dependency_kind: format!("{:?}", dep.kind),
                        dependency_id: dep.id.clone(),
                    },
                    key_components: entry.key.components(),
                    reuse_checks,
                };
            }
        }

        for embed in &entry.embed_files {
            if !self.verify_embed_still_valid(embed) {
                return CacheExplanation {
                    artifact_kind: "comptime".to_string(),
                    cache_key,
                    status: CacheExplainStatus::Rebuild,
                    reason: CacheExplainReason::EmbedChanged {
                        path: embed.clone(),
                    },
                    key_components: entry.key.components(),
                    reuse_checks,
                };
            }
        }

        CacheExplanation {
            artifact_kind: "comptime".to_string(),
            cache_key,
            status: CacheExplainStatus::Hit,
            reason: CacheExplainReason::CacheHit,
            key_components: entry.key.components(),
            reuse_checks,
        }
    }

    /// Check if a comptime evaluation can be reused
    pub fn try_reuse(&self, key: &ComptimeKey) -> Option<&ComptimeCacheEntry> {
        let cache_key = key.cache_key();
        let entry = self.cache.get(&cache_key)?;

        if !entry.valid {
            return None;
        }

        // Verify dependencies haven't changed
        for dep in &entry.dependencies {
            if !self.verify_dependency_still_valid(dep) {
                return None;
            }
        }

        // Verify embed files haven't changed
        for embed in &entry.embed_files {
            if !self.verify_embed_still_valid(embed) {
                return None;
            }
        }

        Some(entry)
    }

    /// Verify a dependency is still valid
    fn verify_dependency_still_valid(&self, dep: &ComptimeDependency) -> bool {
        let dep_key = format!("{:?}:{}", dep.kind, dep.id);
        !self.invalid_dependencies.contains(&dep_key)
    }

    /// Verify an embed file is still valid
    fn verify_embed_still_valid(&self, path: &str) -> bool {
        !self.invalid_embed_files.contains(path)
    }

    /// Record a new comptime evaluation
    pub fn record(&mut self, entry: ComptimeCacheEntry) {
        self.cache.insert(entry);
    }

    /// Invalidate when a dependency changes
    pub fn invalidate_dependency(&mut self, kind: ComptimeDepKind, id: &str) -> Vec<String> {
        self.cache.invalidate_by_dependency(kind, id)
    }

    /// Invalidate when a file changes
    pub fn invalidate_file(&mut self, file: &str) -> Vec<String> {
        self.cache.invalidate_file(file)
    }

    /// Update the dependency graph hash
    pub fn update_dep_graph_hash(&mut self, hash: &str) {
        self.dep_graph_hash = hash.to_string();
    }

    pub fn mark_dependency_changed(&mut self, kind: ComptimeDepKind, id: &str) {
        self.invalid_dependencies
            .insert(format!("{:?}:{}", kind, id));
    }

    pub fn mark_embed_changed(&mut self, path: &str) {
        self.invalid_embed_files.insert(path.to_string());
    }

    /// Set builtin version
    pub fn set_builtin_version(&mut self, builtin: &str, version: &str) {
        self.builtins
            .insert(builtin.to_string(), version.to_string());
    }
}

// ============================================================================
// Builder for ComptimeCacheEntry
// ============================================================================

/// Builder for creating comptime cache entries
pub struct ComptimeCacheEntryBuilder {
    key: ComptimeKey,
    result_kind: ComptimeResultKind,
    dependencies: Vec<ComptimeDependency>,
    embed_files: Vec<String>,
}

impl ComptimeCacheEntryBuilder {
    /// Start building a new cache entry
    pub fn new(file: &str, name: &str, line: u32, column: u32) -> Self {
        Self {
            key: ComptimeKey::new(file, name, line, column),
            result_kind: ComptimeResultKind::Unknown,
            dependencies: Vec::new(),
            embed_files: Vec::new(),
        }
    }

    /// Set arguments hash
    pub fn args_hash(mut self, hash: &str) -> Self {
        self.key = self.key.with_args(hash);
        self
    }

    /// Set target
    pub fn target(mut self, target: &str) -> Self {
        self.key = self.key.with_target(target);
        self
    }

    /// Set builtins hash
    pub fn builtins_hash(mut self, hash: &str) -> Self {
        self.key = self.key.with_builtins(hash);
        self
    }

    /// Set result kind
    pub fn result_kind(mut self, kind: ComptimeResultKind) -> Self {
        self.result_kind = kind;
        self
    }

    /// Add a type dependency
    pub fn depends_on_type(mut self, type_name: &str, hash: &str) -> Self {
        self.dependencies.push(ComptimeDependency::new(
            ComptimeDepKind::Type,
            type_name,
            hash,
        ));
        self
    }

    /// Add a struct dependency
    pub fn depends_on_struct(mut self, struct_name: &str, hash: &str) -> Self {
        self.dependencies.push(ComptimeDependency::new(
            ComptimeDepKind::Struct,
            struct_name,
            hash,
        ));
        self
    }

    /// Add a function dependency
    pub fn depends_on_function(mut self, func_name: &str, hash: &str) -> Self {
        self.dependencies.push(ComptimeDependency::new(
            ComptimeDepKind::Function,
            func_name,
            hash,
        ));
        self
    }

    /// Add a constant dependency
    pub fn depends_on_constant(mut self, const_name: &str, hash: &str) -> Self {
        self.dependencies.push(ComptimeDependency::new(
            ComptimeDepKind::Constant,
            const_name,
            hash,
        ));
        self
    }

    /// Add a layout dependency
    pub fn depends_on_layout(mut self, layout_name: &str, hash: &str) -> Self {
        self.dependencies.push(ComptimeDependency::new(
            ComptimeDepKind::Layout,
            layout_name,
            hash,
        ));
        self
    }

    /// Add a builtin dependency
    pub fn uses_builtin(mut self, builtin_name: &str, version: &str) -> Self {
        self.dependencies.push(ComptimeDependency::new(
            ComptimeDepKind::Builtin,
            builtin_name,
            version,
        ));
        self
    }

    /// Add an embed file
    pub fn embed_file(mut self, path: &str) -> Self {
        self.embed_files.push(path.to_string());
        self
    }

    /// Build the cache entry
    pub fn build(self) -> ComptimeCacheEntry {
        let mut entry = ComptimeCacheEntry::new(self.key).with_result_kind(self.result_kind);
        entry.dependencies = self.dependencies;
        entry.embed_files = self.embed_files;
        entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comptime_key_creation() {
        let key = ComptimeKey::new("test.zig", "my_const", 10, 1)
            .with_args("abc123")
            .with_target("x86_64-linux-gnu")
            .with_builtins("def456");

        assert_eq!(key.file, "test.zig");
        assert_eq!(key.name, "my_const");
        assert_eq!(key.line, 10);
        assert!(!key.cache_key().is_empty());
        assert!(key.cache_key().starts_with("comptime_"));
    }

    #[test]
    fn test_comptime_key_cache_key_is_stable_and_input_sensitive() {
        let base = ComptimeKey::new("test.zig", "my_const", 10, 1)
            .with_args("abc123")
            .with_target("x86_64-linux-gnu")
            .with_builtins("def456");
        let same = ComptimeKey::new("test.zig", "my_const", 10, 1)
            .with_args("abc123")
            .with_target("x86_64-linux-gnu")
            .with_builtins("def456");
        let changed = ComptimeKey::new("test.zig", "my_const", 10, 2)
            .with_args("abc123")
            .with_target("x86_64-linux-gnu")
            .with_builtins("def456");

        assert_eq!(base.cache_key(), same.cache_key());
        assert_ne!(base.cache_key(), changed.cache_key());
    }

    #[test]
    fn test_cache_insert_and_lookup() {
        let mut cache = ComptimeCache::new();

        let key = ComptimeKey::new("test.zig", "SIZE", 5, 1)
            .with_args("1024")
            .with_target("x86_64-linux-gnu");

        let entry = ComptimeCacheEntryBuilder::new("test.zig", "SIZE", 5, 1)
            .args_hash("1024")
            .target("x86_64-linux-gnu")
            .result_kind(ComptimeResultKind::Constant)
            .build();

        cache.insert(entry);

        let cache_key = key.cache_key();
        assert!(cache.is_valid(&cache_key));
    }

    #[test]
    fn test_dependency_invalidation() {
        let mut cache = ComptimeCache::new();

        // Insert an entry that depends on a type
        let entry = ComptimeCacheEntryBuilder::new("test.zig", "my_func", 10, 1)
            .args_hash("type=MyType")
            .target("x86_64-linux-gnu")
            .depends_on_type("MyType", "hash123")
            .build();

        cache.insert(entry);

        // Invalidate the type
        let invalidated = cache.invalidate_by_dependency(ComptimeDepKind::Type, "MyType");

        // The entry should be invalidated
        assert!(!invalidated.is_empty());
    }

    #[test]
    fn test_file_invalidation() {
        let mut cache = ComptimeCache::new();

        // Insert entries from the same file
        for i in 1..=3 {
            let entry =
                ComptimeCacheEntryBuilder::new("test.zig", &format!("const_{}", i), i as u32, 1)
                    .args_hash(&format!("val{}", i))
                    .build();
            cache.insert(entry);
        }

        // Invalidate the file
        let invalidated = cache.invalidate_file("test.zig");

        // All three entries should be invalidated
        assert_eq!(invalidated.len(), 3);
    }

    #[test]
    fn test_cache_serialization() {
        let mut cache = ComptimeCache::new();

        let entry = ComptimeCacheEntryBuilder::new("test.zig", "SIZE", 5, 1)
            .args_hash("1024")
            .result_kind(ComptimeResultKind::Constant)
            .build();

        cache.insert(entry);

        let json = cache.to_json().unwrap();
        let restored = ComptimeCache::from_json(&json).unwrap();

        assert_eq!(cache.len(), restored.len());
    }

    #[test]
    fn test_comptime_context_reuse() {
        let mut ctx = ComptimeContext::new("x86_64-linux-gnu");

        let entry = ComptimeCacheEntryBuilder::new("test.zig", "my_const", 10, 1)
            .args_hash("1024")
            .target("x86_64-linux-gnu")
            .result_kind(ComptimeResultKind::Constant)
            .build();

        ctx.record(entry);

        // Try to reuse should return the cached entry
        let key = ComptimeKey::new("test.zig", "my_const", 10, 1)
            .with_args("1024")
            .with_target("x86_64-linux-gnu");

        let reused = ctx.try_reuse(&key);
        assert!(reused.is_some());
        assert_eq!(ctx.target(), "x86_64-linux-gnu");
    }

    #[test]
    fn test_comptime_context_metadata_tracking() {
        let mut ctx = ComptimeContext::new("x86_64-linux-gnu");
        ctx.update_dep_graph_hash("graph-hash-1");
        ctx.set_builtin_version("@sizeOf", "0.13.0");

        assert_eq!(ctx.dep_graph_hash(), "graph-hash-1");
        assert_eq!(ctx.builtin_version("@sizeOf"), Some("0.13.0"));
    }

    #[test]
    fn test_cache_entry_builder() {
        let entry = ComptimeCacheEntryBuilder::new("test.zig", "compute_size", 20, 5)
            .args_hash("type=Point")
            .target("x86_64-linux-gnu")
            .depends_on_type("Point", "hash456")
            .depends_on_struct("Point", "layout_hash")
            .uses_builtin("@sizeOf", "1.0")
            .embed_file("data/default.bin")
            .result_kind(ComptimeResultKind::Constant)
            .build();

        assert_eq!(entry.key.name, "compute_size");
        assert_eq!(entry.dependencies.len(), 3);
        assert_eq!(entry.embed_files.len(), 1);
        assert_eq!(entry.result_kind, ComptimeResultKind::Constant);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = ComptimeCache::new();

        assert_eq!(cache.stats().total_entries, 0);

        let entry = ComptimeCacheEntryBuilder::new("test.zig", "SIZE", 5, 1).build();
        cache.insert(entry);

        assert_eq!(cache.stats().total_entries, 1);

        cache.invalidate_file("test.zig");

        assert_eq!(cache.stats().invalidated_entries, 1);
    }

    #[test]
    fn test_cache_proof_fact_emission_tracks_dependencies() {
        let mut ctx = ComptimeContext::new("x86_64-linux-gnu");
        ctx.update_dep_graph_hash("graph-v1");
        ctx.set_builtin_version("@sizeOf", "0.13.0");

        let entry = ComptimeCacheEntryBuilder::new("test.zig", "compute_size", 20, 5)
            .args_hash("type=Point")
            .target("x86_64-linux-gnu")
            .depends_on_type("Point", "hash456")
            .depends_on_layout("PointLayout", "layout_hash")
            .result_kind(ComptimeResultKind::Constant)
            .build();
        ctx.record(entry);

        let facts = ctx.cache_proof_facts(1);
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].target, "x86_64-linux-gnu");
        assert_eq!(facts[0].schema_version, 1);
        assert_eq!(facts[0].dependency_fingerprints.len(), 2);
        assert!(facts[0].reusable);
    }

    #[test]
    fn test_cache_proof_fact_marks_invalidated_entries_not_reusable() {
        let mut ctx = ComptimeContext::new("x86_64-linux-gnu");
        let entry = ComptimeCacheEntryBuilder::new("test.zig", "SIZE", 5, 1)
            .args_hash("1024")
            .target("x86_64-linux-gnu")
            .depends_on_type("MyType", "hash123")
            .build();
        ctx.record(entry);
        let invalidated = ctx.invalidate_dependency(ComptimeDepKind::Type, "MyType");

        assert_eq!(invalidated.len(), 1);
        let facts = ctx.cache_proof_facts(1);
        assert_eq!(facts.len(), 1);
        assert!(!facts[0].reusable);
    }

    #[test]
    fn test_proof_artifact_includes_cache_facts() {
        let mut ctx = ComptimeContext::new("x86_64-linux-gnu");
        let entry = ComptimeCacheEntryBuilder::new("test.zig", "my_const", 10, 1)
            .args_hash("1024")
            .target("x86_64-linux-gnu")
            .result_kind(ComptimeResultKind::Constant)
            .build();
        ctx.record(entry);

        let artifact = ctx.proof_artifact([0u8; 20], 42, 1);
        assert!(artifact.header_magic_valid());
        assert_eq!(artifact.header.proof_count, 1);
        assert_eq!(artifact.cache_facts.len(), 1);
        assert!(artifact.obligations.is_empty());
    }

    #[test]
    fn test_cache_explanation_reports_hit_and_key_components() {
        let mut ctx = ComptimeContext::new("x86_64-linux-gnu");
        ctx.update_dep_graph_hash("graph-v2");
        ctx.set_builtin_version("@sizeOf", "0.13.0");

        let entry = ComptimeCacheEntryBuilder::new("math.zig", "compute_size", 20, 5)
            .args_hash("type=Point")
            .target("x86_64-linux-gnu")
            .builtins_hash("builtin-hash-1")
            .depends_on_type("Point", "hash456")
            .embed_file("assets/point.bin")
            .result_kind(ComptimeResultKind::Constant)
            .build();
        let key = entry.key.clone();
        ctx.record(entry);

        let explanation = ctx.explain_reuse(&key);
        assert_eq!(explanation.status, CacheExplainStatus::Hit);
        assert_eq!(explanation.reason, CacheExplainReason::CacheHit);
        assert_eq!(explanation.key_components.file, "math.zig");
        assert_eq!(explanation.key_components.builtins_hash, "builtin-hash-1");
        assert_eq!(explanation.reuse_checks.dependency_fingerprints.len(), 1);
        assert_eq!(
            explanation.reuse_checks.embed_files,
            vec!["assets/point.bin"]
        );
    }

    #[test]
    fn test_cache_explanation_reports_invalidated_dependency_rebuild() {
        let mut ctx = ComptimeContext::new("x86_64-linux-gnu");
        let entry = ComptimeCacheEntryBuilder::new("test.zig", "compute", 1, 1)
            .args_hash("arg1")
            .target("x86_64-linux-gnu")
            .depends_on_type("Point", "hash456")
            .build();
        let key = entry.key.clone();
        ctx.record(entry);
        ctx.mark_dependency_changed(ComptimeDepKind::Type, "Point");

        let explanation = ctx.explain_reuse(&key);
        assert_eq!(explanation.status, CacheExplainStatus::Rebuild);
        assert_eq!(
            explanation.reason,
            CacheExplainReason::DependencyChanged {
                dependency_kind: "Type".to_string(),
                dependency_id: "Point".to_string(),
            }
        );
    }

    #[test]
    fn test_cache_explanation_reports_cache_miss() {
        let ctx = ComptimeContext::new("x86_64-linux-gnu");
        let key = ComptimeKey::new("test.zig", "missing", 10, 2)
            .with_args("abc123")
            .with_target("x86_64-linux-gnu")
            .with_builtins("builtin-hash");

        let explanation = ctx.explain_reuse(&key);
        assert_eq!(explanation.status, CacheExplainStatus::Miss);
        assert_eq!(explanation.reason, CacheExplainReason::NoEntry);
        assert_eq!(explanation.key_components.name, "missing");
        assert!(!explanation.reuse_checks.cached_entry_valid);
    }
}
