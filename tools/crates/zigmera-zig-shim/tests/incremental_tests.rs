//! Integration tests for zigmera-zig-shim per-file incremental builds.
//!
//! These tests verify that the per-file incremental build system correctly:
//! - Detects changed source files
//! - Rebuilds only affected targets
//! - Reuses cached objects for unchanged files

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use zigmera_zig_shim::manifest::{ContentHash, FileTarget, HashCache, Manifest};

/// Creates a minimal Zig source file for testing
fn create_zig_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, content).unwrap();
    path
}

/// Checks if zig is available for actual compilation tests
fn has_zig() -> bool {
    std::process::Command::new("zig")
        .arg("version")
        .output()
        .is_ok()
}

mod manifest_tests {
    use super::*;

    #[test]
    fn test_manifest_target_for_source() {
        let tmp = TempDir::new().unwrap();
        let manifest_path = tmp.path().join("targets.json");

        let manifest = Manifest {
            version: "1.0".to_string(),
            targets: vec![
                FileTarget {
                    source: "src/a.zig".to_string(),
                    output: "a.o".to_string(),
                    imports: vec![],
                },
                FileTarget {
                    source: "src/b.zig".to_string(),
                    output: "b.o".to_string(),
                    imports: vec!["src/a.zig".to_string()],
                },
            ],
            build_opts: Default::default(),
        };

        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        let loaded = Manifest::from_file(&manifest_path).unwrap();

        assert!(loaded.target_for_source("src/a.zig").is_some());
        assert!(loaded.target_for_source("src/b.zig").is_some());
        assert!(loaded.target_for_source("src/c.zig").is_none());
    }

    #[test]
    fn test_manifest_targets_depending_on() {
        let manifest = Manifest {
            version: "1.0".to_string(),
            targets: vec![
                FileTarget {
                    source: "src/a.zig".to_string(),
                    output: "a.o".to_string(),
                    imports: vec![],
                },
                FileTarget {
                    source: "src/b.zig".to_string(),
                    output: "b.o".to_string(),
                    imports: vec!["src/a.zig".to_string()],
                },
                FileTarget {
                    source: "src/c.zig".to_string(),
                    output: "c.o".to_string(),
                    imports: vec!["src/a.zig".to_string()],
                },
            ],
            build_opts: Default::default(),
        };

        let dependents = manifest.targets_depending_on("src/a.zig");
        assert_eq!(dependents.len(), 2);

        let dependent_outputs: Vec<_> = dependents.iter().map(|t| t.output.as_str()).collect();
        assert!(dependent_outputs.contains(&"b.o"));
        assert!(dependent_outputs.contains(&"c.o"));
    }

    #[test]
    fn test_hash_cache_compute_hash() {
        let tmp = TempDir::new().unwrap();
        let source = create_zig_file(tmp.path(), "test.zig", "pub const x = 42;");

        let hash = HashCache::compute_hash(&source);
        assert!(hash.is_some());

        let hash = hash.unwrap();
        assert_eq!(hash.source, source.display().to_string());
        assert!(hash.hash != 0);
        assert!(hash.mtime_ns != 0);
    }

    #[test]
    fn test_hash_cache_has_changed() {
        let tmp = TempDir::new().unwrap();
        let source = create_zig_file(tmp.path(), "test.zig", "pub const x = 42;");

        let mut cache = HashCache::new();

        // File should be detected as changed on first check
        assert!(cache.has_changed(&source));

        // After updating, should not be changed
        cache.update(&source);
        assert!(!cache.has_changed(&source));

        // Modify the file
        fs::write(&source, "pub const x = 100;").unwrap();
        assert!(cache.has_changed(&source));
    }

    #[test]
    fn test_hash_cache_persistence() {
        let tmp = TempDir::new().unwrap();
        let cache_path = tmp.path().join("cache.json");

        let mut cache = HashCache::new();
        let source = create_zig_file(tmp.path(), "test.zig", "pub const x = 42;");
        cache.update(&source);

        cache.save(&cache_path).unwrap();

        let loaded = HashCache::from_file(&cache_path).unwrap();
        assert!(!loaded.has_changed(&source));
    }
}

mod integration_tests {
    use super::*;

    /// Simulates the targets_to_rebuild logic from PerFileBuilder
    fn compute_targets_to_rebuild(
        manifest: &Manifest,
        changed_sources: &[String],
    ) -> HashSet<String> {
        let mut targets = HashSet::new();
        let mut sources_to_process: Vec<String> = changed_sources.to_vec();

        while let Some(source) = sources_to_process.pop() {
            for target in manifest.targets_depending_on(&source) {
                if targets.insert(target.output.clone()) {
                    sources_to_process.push(target.source.clone());
                }
            }
            if let Some(t) = manifest.target_for_source(&source) {
                targets.insert(t.output.clone());
            }
        }

        targets
    }

    #[test]
    fn test_single_file_change_detected() {
        // Create manifest with two independent files
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
                    imports: vec![],
                },
            ],
            build_opts: Default::default(),
        };

        // Simulate changing only a.zig
        let changed = vec!["a.zig".to_string()];
        let to_rebuild = compute_targets_to_rebuild(&manifest, &changed);

        // Only a.o should need rebuild
        assert!(to_rebuild.contains("a.o"));
        assert!(!to_rebuild.contains("b.o"));
    }

    #[test]
    fn test_transitive_dependency_rebuild() {
        // a.zig -> b.zig -> c.zig (c is imported by b, b is imported by a)
        let manifest = Manifest {
            version: "1.0".to_string(),
            targets: vec![
                FileTarget {
                    source: "a.zig".to_string(),
                    output: "a.o".to_string(),
                    imports: vec!["b.zig".to_string()],
                },
                FileTarget {
                    source: "b.zig".to_string(),
                    output: "b.o".to_string(),
                    imports: vec!["c.zig".to_string()],
                },
                FileTarget {
                    source: "c.zig".to_string(),
                    output: "c.o".to_string(),
                    imports: vec![],
                },
            ],
            build_opts: Default::default(),
        };

        // Change c.zig - should invalidate both b.o and a.o
        let changed = vec!["c.zig".to_string()];
        let to_rebuild = compute_targets_to_rebuild(&manifest, &changed);

        assert!(to_rebuild.contains("c.o"));
        assert!(to_rebuild.contains("b.o"));
        assert!(to_rebuild.contains("a.o"));
    }

    #[test]
    fn test_no_change_reuses_all() {
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
            ],
            build_opts: Default::default(),
        };

        // No sources changed
        let changed: Vec<String> = vec![];
        let to_rebuild = compute_targets_to_rebuild(&manifest, &changed);

        // Nothing should need rebuild
        assert!(to_rebuild.is_empty());
    }

    #[test]
    fn test_diamond_dependency_rebuild() {
        // Diamond: a.zig imports both b.zig and c.zig, both import d.zig
        let manifest = Manifest {
            version: "1.0".to_string(),
            targets: vec![
                FileTarget {
                    source: "a.zig".to_string(),
                    output: "a.o".to_string(),
                    imports: vec!["b.zig".to_string(), "c.zig".to_string()],
                },
                FileTarget {
                    source: "b.zig".to_string(),
                    output: "b.o".to_string(),
                    imports: vec!["d.zig".to_string()],
                },
                FileTarget {
                    source: "c.zig".to_string(),
                    output: "c.o".to_string(),
                    imports: vec!["d.zig".to_string()],
                },
                FileTarget {
                    source: "d.zig".to_string(),
                    output: "d.o".to_string(),
                    imports: vec![],
                },
            ],
            build_opts: Default::default(),
        };

        // Change d.zig - should invalidate all files
        let changed = vec!["d.zig".to_string()];
        let to_rebuild = compute_targets_to_rebuild(&manifest, &changed);

        assert!(to_rebuild.contains("d.o"));
        assert!(to_rebuild.contains("b.o"));
        assert!(to_rebuild.contains("c.o"));
        assert!(to_rebuild.contains("a.o"));
    }

    #[test]
    fn test_large_manifest_performance() {
        let mut targets = Vec::new();

        // Create a large manifest with 100 files in a chain
        for i in 0..100 {
            let imports = if i > 0 {
                vec![format!("{}.zig", i - 1)]
            } else {
                vec![]
            };
            targets.push(FileTarget {
                source: format!("{}.zig", i),
                output: format!("{}.o", i),
                imports,
            });
        }

        let manifest = Manifest {
            version: "1.0".to_string(),
            targets,
            build_opts: Default::default(),
        };

        // Change first file - should rebuild all 100
        let changed = vec!["0.zig".to_string()];
        let to_rebuild = compute_targets_to_rebuild(&manifest, &changed);

        assert_eq!(to_rebuild.len(), 100);
    }
}
