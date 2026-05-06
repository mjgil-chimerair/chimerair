//! ZigMera CLI library
//!
//! Provides functions for per-file incremental Zig builds.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Re-export from zigmera-zig-shim
pub use zigmera_zig_shim::manifest::{BuildOptions, FileTarget, HashCache, Manifest};
pub use zigmera_zig_shim::per_file::PerFileBuilder;

/// Configuration for manifest generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenConfig {
    pub source_dir: String,
    pub output_dir: String,
    pub target: String,
    pub optimize: String,
}

impl Default for GenConfig {
    fn default() -> Self {
        Self {
            source_dir: "src".to_string(),
            output_dir: "zig-out".to_string(),
            target: "x86_64-linux-gnu".to_string(),
            optimize: "ReleaseFast".to_string(),
        }
    }
}

/// Generate a manifest by scanning source files
pub fn generate_manifest(config: &GenConfig) -> Result<Manifest, String> {
    let source_path = Path::new(&config.source_dir);
    if !source_path.exists() {
        return Err(format!(
            "Source directory '{}' does not exist",
            config.source_dir
        ));
    }

    let mut targets = Vec::new();
    let mut import_map: HashMap<String, Vec<String>> = HashMap::new();

    collect_zig_files(
        source_path,
        &config.output_dir,
        &mut targets,
        &mut import_map,
    )?;

    let targets: Vec<FileTarget> = targets
        .into_iter()
        .map(|(source, output)| {
            let imports = import_map.get(&source).cloned().unwrap_or_default();
            FileTarget {
                source,
                output,
                imports,
            }
        })
        .collect();

    Ok(Manifest {
        version: "1.0".to_string(),
        targets,
        build_opts: BuildOptions {
            target: Some(config.target.clone()),
            optimize: Some(config.optimize.clone()),
            cache_dir: None,
            other_args: Vec::new(),
        },
    })
}

/// Recursively collect .zig files and their imports
fn collect_zig_files(
    dir: &Path,
    output_dir: &str,
    targets: &mut Vec<(String, String)>,
    import_map: &mut HashMap<String, Vec<String>>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            collect_zig_files(&path, output_dir, targets, import_map)?;
        } else if path.extension().map_or(false, |ext| ext == "zig") {
            let source = path.display().to_string();
            let output = file_to_object_name(&path, output_dir);

            let imports = parse_imports(&path);
            import_map.insert(source.clone(), imports);

            targets.push((source, output));
        }
    }

    Ok(())
}

/// Parse @import statements from a Zig source file
/// Returns imports resolved relative to the source file's directory
pub fn parse_imports(source: &Path) -> Vec<String> {
    let content = match fs::read_to_string(source) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let source_dir = source.parent().unwrap_or(Path::new(""));
    let mut imports = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if let Some(start) = line.find("@import(\"") {
            let after_import = &line[start + 9..];
            if let Some(end) = after_import.find("\")") {
                let import_path = &after_import[..end];
                // Resolve relative to source file's directory
                let full_path = source_dir.join(import_path);
                let normalized = full_path.display().to_string();
                imports.push(normalized);
            }
        }
    }

    imports
}

fn file_to_object_name(source: &Path, output_dir: &str) -> String {
    let stem = source.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
    format!("{}/{}.o", output_dir, stem)
}

/// Initialize a project for per-file incremental builds
pub fn init_project(config: &GenConfig, cache_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(cache_dir).map_err(|e| format!("Failed to create cache dir: {}", e))?;

    let manifest = generate_manifest(config)?;
    let manifest_path = cache_dir.join("manifest.json");

    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    fs::write(&manifest_path, json).map_err(|e| format!("Failed to write manifest: {}", e))?;

    let hash_cache = HashCache::new();
    let hash_cache_path = cache_dir.join("hash_cache.json");
    hash_cache.save(&hash_cache_path)?;

    println!("Initialized!");
    println!("  Targets: {}", manifest.targets.len());
    println!("  Manifest: {}", manifest_path.display());

    Ok(())
}

/// Build with per-file incremental compilation
pub fn build_project(zig_path: &Path, cache_dir: &Path, build_dir: &Path) -> Result<(), String> {
    let manifest_path = cache_dir.join("manifest.json");
    let hash_cache_path = cache_dir.join("hash_cache.json");

    if !manifest_path.exists() {
        return Err("No manifest found. Run 'zigmera init' first.".to_string());
    }

    let mut builder = PerFileBuilder::new(
        manifest_path,
        hash_cache_path,
        zig_path.to_path_buf(),
        build_dir.to_path_buf(),
    )?;

    match builder.build() {
        Ok((exit_code, built, cached)) => {
            println!("Build: {} built, {} cached", built, cached);
            if exit_code != std::process::ExitCode::SUCCESS {
                return Err("Build failed".to_string());
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Show cache status
pub fn show_status(cache_dir: &Path) -> Result<(), String> {
    let manifest_path = cache_dir.join("manifest.json");
    let hash_cache_path = cache_dir.join("hash_cache.json");

    println!("ZigMera Status");
    println!("  Cache: {}", cache_dir.display());
    println!();

    if !manifest_path.exists() {
        println!("  No manifest - run 'zigmera init' first");
        return Ok(());
    }

    let manifest = Manifest::from_file(&manifest_path)?;
    let hash_cache = HashCache::from_file(&hash_cache_path)?;

    println!("  Manifest: {} targets", manifest.targets.len());
    println!("  Hash Cache: {} entries", hash_cache.hashes.len());

    let changed = hash_cache.changed_sources(&manifest);
    if changed.is_empty() {
        println!("  Status: All files up to date");
    } else {
        println!("  Status: {} files changed", changed.len());
        for source in changed.iter().take(5) {
            println!("    - {}", source);
        }
        if changed.len() > 5 {
            println!("    ... and {} more", changed.len() - 5);
        }
    }

    Ok(())
}

/// Clean build artifacts and cache
pub fn clean_cache(cache_dir: &Path) -> Result<(), String> {
    if cache_dir.exists() {
        fs::remove_dir_all(cache_dir).map_err(|e| format!("Failed to remove cache: {}", e))?;
    }
    Ok(())
}

/// Save manifest to file
pub fn save_manifest(manifest: &Manifest, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(path, json).map_err(|e| format!("Failed to write: {}", e))?;
    Ok(())
}
