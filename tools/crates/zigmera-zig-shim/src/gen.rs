//! Manifest generator for Zig projects.
//!
//! Parses Zig build.zig files and ninja logs to generate file target manifests.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::manifest::{FileTarget, Manifest};

/// Configuration for manifest generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenConfig {
    /// Source directory (default: "src")
    pub source_dir: String,
    /// Output directory (default: "zig-out")
    pub output_dir: String,
    /// File patterns to include (default: ["**.zig"])
    pub patterns: Vec<String>,
    /// Target architecture (default: "x86_64-linux-gnu")
    pub target: String,
    /// Optimization level (default: "ReleaseFast")
    pub optimize: String,
}

impl Default for GenConfig {
    fn default() -> Self {
        Self {
            source_dir: "src".to_string(),
            output_dir: "zig-out".to_string(),
            patterns: vec!["**/*.zig".to_string()],
            target: "x86_64-linux-gnu".to_string(),
            optimize: "ReleaseFast".to_string(),
        }
    }
}

/// Generate a manifest by scanning source files
pub fn generate_from_sources(config: &GenConfig) -> Result<Manifest, String> {
    let source_path = Path::new(&config.source_dir);
    if !source_path.exists() {
        return Err(format!(
            "Source directory '{}' does not exist",
            config.source_dir
        ));
    }

    let mut targets = Vec::new();
    let mut import_map: HashMap<String, Vec<String>> = HashMap::new();

    // Scan for .zig files
    collect_zig_files(source_path, &config.patterns, &mut targets, &mut import_map)?;

    // Build FileTargets with imports
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
        build_opts: crate::manifest::BuildOptions {
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
    patterns: &[String],
    targets: &mut Vec<(String, String)>,
    import_map: &mut HashMap<String, Vec<String>>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            collect_zig_files(&path, patterns, targets, import_map)?;
        } else if path.extension().map_or(false, |ext| ext == "zig") {
            let source = path.display().to_string();
            let output = file_to_object(&path, "zig-out");

            // Parse imports from the file
            let imports = parse_imports(&path);
            import_map.insert(source.clone(), imports);

            targets.push((source, output));
        }
    }

    Ok(())
}

/// Parse @import statements from a Zig source file
pub fn parse_imports(source: &Path) -> Vec<String> {
    let content = match fs::read_to_string(source) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut imports = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        // Match: const X = @import("path.zig");
        // Or: const X = @import("path");
        if let Some(start) = line.find("@import(\"") {
            let after_import = &line[start + 9..];
            if let Some(end) = after_import.find("\")") {
                let import_path = &after_import[..end];
                // Normalize: add .zig if missing
                let normalized = if import_path.ends_with(".zig") {
                    import_path.to_string()
                } else {
                    format!("{}.zig", import_path)
                };
                imports.push(normalized);
            }
        }
    }

    imports
}

/// Convert file path to object file name
fn file_to_object(source: &Path, output_dir: &str) -> String {
    let stem = source.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
    format!("{}/{}.o", output_dir, stem)
}

/// Generate manifest from ninja build log
pub fn generate_from_ninja(ninja_content: &str, _output_dir: &Path) -> Manifest {
    let mut targets = Vec::new();

    for line in ninja_content.lines() {
        if line.contains("zig build obj") {
            if let Some((output, sources)) = parse_zig_build_line(line) {
                for source in sources {
                    targets.push(FileTarget {
                        source,
                        output: output.clone(),
                        imports: Vec::new(),
                    });
                }
            }
        }
    }

    Manifest {
        version: "1.0".to_string(),
        targets,
        build_opts: crate::manifest::BuildOptions::default(),
    }
}

/// Parse a ninja build line to extract output and source files
fn parse_zig_build_line(line: &str) -> Option<(String, Vec<String>)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    let output = parts[1].trim_end_matches(':').to_string();

    let sources: Vec<String> = parts[3..]
        .iter()
        .filter(|s| s.ends_with(".zig"))
        .map(|s| s.to_string())
        .collect();

    if sources.is_empty() {
        return None;
    }

    Some((output, sources))
}

/// Save manifest to file
pub fn save_manifest(manifest: &Manifest, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    fs::write(path, json).map_err(|e| format!("Failed to write manifest: {}", e))?;
    Ok(())
}

/// Load manifest from file
pub fn load_manifest(path: &Path) -> Result<Manifest, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read manifest: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse manifest: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parse_imports_simple() {
        let tmp = TempDir::new().unwrap();
        let source_file = tmp.path().join("test.zig");
        fs::write(
            &source_file,
            r#"
const std = @import("std");
const builtin = @import("builtin");
const my_module = @import("my_module.zig");
"#,
        )
        .unwrap();

        let imports = parse_imports(&source_file);
        assert!(imports.contains(&"std.zig".to_string()));
        assert!(imports.contains(&"builtin.zig".to_string()));
        assert!(imports.contains(&"my_module.zig".to_string()));
    }

    #[test]
    fn test_generate_from_sources() {
        let tmp = TempDir::new().unwrap();
        let src_dir = tmp.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        fs::write(src_dir.join("main.zig"), "const x = @import(\"util.zig\");").unwrap();
        fs::write(src_dir.join("util.zig"), "pub fn foo() void {}").unwrap();

        let config = GenConfig {
            source_dir: src_dir.display().to_string(),
            output_dir: "zig-out".to_string(),
            patterns: vec!["**/*.zig".to_string()],
            target: "x86_64-linux-gnu".to_string(),
            optimize: "ReleaseFast".to_string(),
        };

        let manifest = generate_from_sources(&config).unwrap();
        assert_eq!(manifest.targets.len(), 2);

        // Check that main.zig has util.zig as import
        let main_target = manifest
            .targets
            .iter()
            .find(|t| t.source.contains("main.zig"))
            .unwrap();
        assert!(main_target.imports.iter().any(|i| i.contains("util.zig")));
    }

    #[test]
    fn test_file_to_object() {
        let path = Path::new("/src/foo/bar.zig");
        let output = file_to_object(path, "zig-out");
        assert_eq!(output, "zig-out/bar.o");
    }
}
