//! Cache key generation for BEAM modules.
//!
//! Generates stable cache keys based on module content and dependencies.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A cache key for a BEAM module or function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKey {
    /// Key version.
    pub version: u32,
    /// Full key hash.
    pub hash: String,
    /// Key type (module, function, etc.).
    pub key_type: KeyType,
}

/// Current cache format version.
pub const CACHE_VERSION: u32 = 1;

impl CacheKey {
    /// Generate a cache key from module data.
    pub fn for_module(module_name: &str, source_hash: &[u8], dependencies: &[String]) -> Self {
        let mut hasher = Sha256::new();

        // Hash version
        hasher.update(CACHE_VERSION.to_le_bytes());

        // Hash module name
        hasher.update(module_name.as_bytes());

        // Hash source
        hasher.update(source_hash);

        // Hash dependencies
        for dep in dependencies {
            hasher.update(dep.as_bytes());
        }

        let result = hasher.finalize();
        let hash = hex::encode(result);

        CacheKey {
            version: CACHE_VERSION,
            hash,
            key_type: KeyType::Module,
        }
    }

    /// Generate a cache key from function.
    pub fn for_function(
        module_name: &str,
        function_name: &str,
        arity: u8,
        source_hash: &[u8],
    ) -> Self {
        let mut hasher = Sha256::new();

        hasher.update(CACHE_VERSION.to_le_bytes());
        hasher.update(module_name.as_bytes());
        hasher.update(function_name.as_bytes());
        hasher.update([arity]);
        hasher.update(source_hash);

        let result = hasher.finalize();
        let hash = hex::encode(result);

        CacheKey {
            version: CACHE_VERSION,
            hash,
            key_type: KeyType::Function,
        }
    }

    /// Generate a cache key from bytecode.
    pub fn for_bytecode(bytecode: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(CACHE_VERSION.to_le_bytes());
        hasher.update(bytecode);

        let result = hasher.finalize();
        let hash = hex::encode(result);

        CacheKey {
            version: CACHE_VERSION,
            hash,
            key_type: KeyType::Bytecode,
        }
    }

    /// Get the key string.
    pub fn as_str(&self) -> &str {
        &self.hash
    }

    /// Check if key is valid.
    pub fn is_valid(&self) -> bool {
        self.version == CACHE_VERSION && !self.hash.is_empty()
    }
}

/// Key type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    /// Module-level cache key.
    Module,
    /// Function-level cache key.
    Function,
    /// Bytecode-level cache key.
    Bytecode,
    /// Dependency-level cache key.
    Dependency,
}

impl KeyType {
    /// Get type name.
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyType::Module => "module",
            KeyType::Function => "function",
            KeyType::Bytecode => "bytecode",
            KeyType::Dependency => "dependency",
        }
    }
}

/// A composite cache key that includes multiple keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeCacheKey {
    /// Primary key.
    pub primary: CacheKey,
    /// Additional keys for dependencies.
    pub dependencies: Vec<CacheKey>,
}

impl CompositeCacheKey {
    /// Create a new composite key.
    pub fn new(primary: CacheKey) -> Self {
        CompositeCacheKey {
            primary,
            dependencies: vec![],
        }
    }

    /// Add a dependency key.
    pub fn with_dependency(mut self, key: CacheKey) -> Self {
        self.dependencies.push(key);
        self
    }

    /// Get total number of keys.
    pub fn total_keys(&self) -> usize {
        1 + self.dependencies.len()
    }

    /// Get all keys.
    pub fn all_keys(&self) -> Vec<&CacheKey> {
        let mut keys = vec![&self.primary];
        for dep in &self.dependencies {
            keys.push(dep);
        }
        keys
    }
}

/// A cache key builder.
#[derive(Debug, Clone)]
pub struct CacheKeyBuilder {
    module_name: String,
    source_hash: Vec<u8>,
    dependencies: Vec<String>,
    key_type: KeyType,
}

impl CacheKeyBuilder {
    /// Create a new builder.
    pub fn new(module_name: impl Into<String>) -> Self {
        CacheKeyBuilder {
            module_name: module_name.into(),
            source_hash: vec![],
            dependencies: vec![],
            key_type: KeyType::Module,
        }
    }

    /// Set source hash.
    pub fn source_hash(mut self, hash: impl Into<Vec<u8>>) -> Self {
        self.source_hash = hash.into();
        self
    }

    /// Add a dependency.
    pub fn add_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    /// Set key type.
    pub fn key_type(mut self, key_type: KeyType) -> Self {
        self.key_type = key_type;
        self
    }

    /// Build the cache key.
    pub fn build(self) -> CacheKey {
        match self.key_type {
            KeyType::Module => {
                CacheKey::for_module(&self.module_name, &self.source_hash, &self.dependencies)
            }
            KeyType::Function => {
                // For function, we need function name and arity - use defaults
                CacheKey::for_function(&self.module_name, "", 0, &self.source_hash)
            }
            KeyType::Bytecode => CacheKey::for_bytecode(&self.source_hash),
            KeyType::Dependency => {
                CacheKey::for_module(&self.module_name, &self.source_hash, &self.dependencies)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_for_module() {
        let key = CacheKey::for_module("test_mod", b"source", &[]);
        assert!(key.is_valid());
        assert_eq!(key.key_type, KeyType::Module);
    }

    #[test]
    fn test_cache_key_for_function() {
        let key = CacheKey::for_function("mod", "fun", 2, b"source");
        assert!(key.is_valid());
        assert_eq!(key.key_type, KeyType::Function);
    }

    #[test]
    fn test_cache_key_for_bytecode() {
        let key = CacheKey::for_bytecode(b"bytecode");
        assert!(key.is_valid());
        assert_eq!(key.key_type, KeyType::Bytecode);
    }

    #[test]
    fn test_cache_key_not_valid() {
        let key = CacheKey {
            version: 0, // wrong version
            hash: "abc".to_string(),
            key_type: KeyType::Module,
        };
        assert!(!key.is_valid());
    }

    #[test]
    fn test_key_type_as_str() {
        assert_eq!(KeyType::Module.as_str(), "module");
        assert_eq!(KeyType::Function.as_str(), "function");
    }

    #[test]
    fn test_composite_cache_key() {
        let primary = CacheKey::for_module("mod", b"source", &[]);
        let composite = CompositeCacheKey::new(primary).with_dependency(CacheKey::for_module(
            "dep",
            b"dep_source",
            &[],
        ));

        assert_eq!(composite.total_keys(), 2);
    }

    #[test]
    fn test_cache_key_builder() {
        let key = CacheKeyBuilder::new("test_mod")
            .source_hash(b"source")
            .add_dependency("dep1")
            .add_dependency("dep2")
            .key_type(KeyType::Module)
            .build();

        assert!(key.is_valid());
    }
}
