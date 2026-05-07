//! Manifest parsing for per-file build targets.
//!
//! Reads file_targets.json manifest that maps source files to output objects.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::SystemTime;

/// Represents a single file target (one .zig file compiling to one .o)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTarget {
    /// Path to the source .zig file (relative or absolute)
    pub source: String,
    /// Path to the output .o file (relative to build dir)
    pub output: String,
    /// List of imported .zig files this file depends on
    #[serde(default)]
    pub imports: Vec<String>,
}

/// Manifest containing all file targets for a build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Version of the manifest format
    pub version: String,
    /// All file targets
    pub targets: Vec<FileTarget>,
    /// Build configuration options
    #[serde(default)]
    pub build_opts: BuildOptions,
}

/// Build options extracted from the original zig build command
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildOptions {
    pub target: Option<String>,
    pub optimize: Option<String>,
    pub cache_dir: Option<String>,
    #[serde(default)]
    pub other_args: Vec<String>,
}

impl Manifest {
    /// Load manifest from a JSON file
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read manifest: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse manifest: {}", e))
    }

    /// Get the target for a specific source file
    pub fn target_for_source(&self, source: &str) -> Option<&FileTarget> {
        self.targets.iter().find(|t| t.source == source)
    }

    /// Get all targets that depend on a given source file
    pub fn targets_depending_on(&self, source: &str) -> Vec<&FileTarget> {
        self.targets
            .iter()
            .filter(|t| t.imports.contains(&source.to_string()))
            .collect()
    }
}

/// Represents content hash information for cache validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentHash {
    /// The source file path
    pub source: String,
    /// Hash of the file content
    pub hash: u64,
    /// Modification time when hash was computed
    pub mtime_ns: u64,
}

/// Hash cache that tracks content hashes for all source files
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HashCache {
    /// Map of source file -> content hash info
    pub hashes: HashMap<String, ContentHash>,
}

impl HashCache {
    /// Create a new empty hash cache
    pub fn new() -> Self {
        Self {
            hashes: HashMap::new(),
        }
    }

    /// Load hash cache from a file
    pub fn from_file(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read hash cache: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse hash cache: {}", e))
    }

    /// Save hash cache to a file
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize hash cache: {}", e))?;
        fs::write(path, json).map_err(|e| format!("Failed to write hash cache: {}", e))
    }

    /// Compute hash for a source file based on content and mtime
    pub fn compute_hash(source: &Path) -> Option<ContentHash> {
        let metadata = fs::metadata(source).ok()?;
        let mtime = metadata.modified().ok()?;
        let mtime_ns = mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let content = fs::read(source).ok()?;
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash = hasher.finish();

        Some(ContentHash {
            source: source.display().to_string(),
            hash,
            mtime_ns,
        })
    }

    /// Check if a source file has changed since last hash
    pub fn has_changed(&self, source: &Path) -> bool {
        let Some(current) = Self::compute_hash(source) else {
            return true; // File doesn't exist or can't be read, consider it changed
        };

        match self.hashes.get(&current.source) {
            Some(prev) if prev.hash == current.hash && prev.mtime_ns == current.mtime_ns => false,
            _ => true,
        }
    }

    /// Update hash for a source file
    pub fn update(&mut self, source: &Path) {
        if let Some(hash_info) = Self::compute_hash(source) {
            self.hashes.insert(hash_info.source.clone(), hash_info);
        }
    }

    /// Get sources that have changed from the manifest
    pub fn changed_sources(&self, manifest: &Manifest) -> Vec<String> {
        let mut changed = Vec::new();
        for target in &manifest.targets {
            let source_path = Path::new(&target.source);
            if self.has_changed(source_path) {
                changed.push(target.source.clone());
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_manifest_parse() {
        let json = r#"{
            "version": "1.0",
            "targets": [
                {"source": "src/main.zig", "output": "main.o"},
                {"source": "src/util.zig", "output": "util.o", "imports": ["src/main.zig"]}
            ],
            "build_opts": {
                "target": "x86_64-linux-gnu",
                "optimize": "Debug"
            }
        }"#;

        let manifest: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.version, "1.0");
        assert_eq!(manifest.targets.len(), 2);
        assert_eq!(manifest.targets[0].source, "src/main.zig");
        assert_eq!(manifest.targets[1].imports, vec!["src/main.zig"]);
    }

    #[test]
    fn test_hash_cache_change_detection() {
        let tmp = TempDir::new().unwrap();
        let source_file = tmp.path().join("test.zig");
        fs::write(&source_file, "const x = 1;").unwrap();

        let mut cache = HashCache::new();
        assert!(cache.has_changed(&source_file)); // First time, should be changed

        cache.update(&source_file);
        assert!(!cache.has_changed(&source_file)); // After update, should match

        // Modify the file
        fs::write(&source_file, "const x = 2;").unwrap();
        assert!(cache.has_changed(&source_file)); // After modification, should be changed
    }

    #[test]
    fn test_hash_cache_round_trip() {
        let tmp = TempDir::new().unwrap();
        let cache_file = tmp.path().join("hash_cache.json");

        let mut cache = HashCache::new();
        let source_file = tmp.path().join("test.zig");
        fs::write(&source_file, "test").unwrap();
        cache.update(&source_file);

        cache.save(&cache_file).unwrap();

        let loaded = HashCache::from_file(&cache_file).unwrap();
        assert_eq!(loaded.hashes.len(), cache.hashes.len());
    }

    #[test]
    fn test_targets_depending_on() {
        let manifest = Manifest {
            version: "1.0".to_string(),
            targets: vec![
                FileTarget {
                    source: "a.zig".to_string(),
                    output: "a.o".to_string(),
                    imports: vec![],
                },
                FileTarget {
                    source: "b.zig".to_string(),
                    output: "b.o".to_string(),
                    imports: vec!["a.zig".to_string()],
                },
                FileTarget {
                    source: "c.zig".to_string(),
                    output: "c.o".to_string(),
                    imports: vec!["a.zig".to_string(), "b.zig".to_string()],
                },
            ],
            build_opts: BuildOptions::default(),
        };

        let deps_on_a: Vec<_> = manifest.targets_depending_on("a.zig");
        assert_eq!(deps_on_a.len(), 2);
        assert!(deps_on_a.iter().any(|t| t.source == "b.zig"));
        assert!(deps_on_a.iter().any(|t| t.source == "c.zig"));

        let deps_on_b: Vec<_> = manifest.targets_depending_on("b.zig");
        assert_eq!(deps_on_b.len(), 1);
        assert_eq!(deps_on_b[0].source, "c.zig");
    }
}
