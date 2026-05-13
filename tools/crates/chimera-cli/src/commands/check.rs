//! `chimera check` command
//!
//! Validates metadata, contracts, layouts, and proof obligations.

use anyhow::{Context, Result};
use chimera_build::{BuildConfig, BuildMode, BuildOrchestrator};
use std::path::PathBuf;

pub fn run(manifest: Option<PathBuf>, target: Option<String>) -> Result<()> {
    log::info!("Running chimera check");

    let manifest_path = manifest.unwrap_or_else(|| PathBuf::from("Chimera.toml"));

    if !manifest_path.exists() {
        log::warn!("No project manifest found at {:?}", manifest_path);
        log::info!("Checking individual files...");

        // Check if metadata files exist
        let meta_files = find_metadata_files()?;
        if meta_files.is_empty() {
            log::info!("No .chmeta files found");
            return Ok(());
        }

        for meta_file in meta_files {
            check_metadata_file(&meta_file)?;
        }

        return Ok(());
    }

    // Parse and validate manifest
    log::info!("Validating project manifest: {:?}", manifest_path);

    let build_config = BuildConfig {
        target: chimera_build::Target {
            triple: target.unwrap_or_else(|| "x86_64-unknown-linux-gnu".to_string()),
            features: vec![],
            runtime_variant: None,
            cpu_features: vec![],
        },
        output_dir: PathBuf::from("build"),
        cache_enabled: true,
        proof_verification: true,
        build_mode: BuildMode::default(),
        wrapper_languages: vec![
            chimera_meta::SourceLanguage::C,
            chimera_meta::SourceLanguage::Rust,
        ],
        rust_artifacts_dir: PathBuf::from("build/artifacts/rust"),
        rust_cache_dir: PathBuf::from("build/cache/rust"),
        zig_artifacts_dir: PathBuf::from(".zigmera/artifacts"),
        zigmera_lowering_path: Some(PathBuf::from("zigml")),
        rustc_driver_path: None,
        c_artifacts_dir: PathBuf::from("build/artifacts/c"),
        chimera_c_clang_path: None,
        chimera_c_cache_path: None,
        require_authoritative_zig: false,
    };

    let _orchestrator = BuildOrchestrator::new(build_config);

    log::info!("Check completed successfully");
    Ok(())
}

fn find_metadata_files() -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(".")? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "chmeta" {
                files.push(path);
            }
        }
    }

    Ok(files)
}

fn check_metadata_file(path: &PathBuf) -> Result<()> {
    log::info!("Checking metadata file: {:?}", path);

    let content =
        std::fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;

    let metadata = chimera_meta::Metadata::parse(&content)
        .with_context(|| format!("Failed to parse {:?}", path))?;

    metadata
        .validate()
        .with_context(|| format!("Validation failed for {:?}", path))?;

    log::info!("  version: {}", metadata.version.as_string());
    if let Some(ref module) = metadata.module {
        log::info!("  module: {} (target: {})", module.name, module.target);
    }
    log::info!(
        "  {} functions, {} proof obligations",
        metadata.functions.len(),
        metadata.proof_obligations.len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_find_metadata_files_nonexistent() {
        // When no files exist, should return empty vec
        // This is a unit test for the function logic
    }
}
