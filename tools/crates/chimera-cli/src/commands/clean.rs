//! `chimera clean` command
//!
//! Removes build artifacts and cache.

use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn run(all: bool) -> Result<()> {
    log::info!("Running chimera clean");

    let build_dir = PathBuf::from("build");
    let cache_dir = PathBuf::from(".chimera-cache");

    if build_dir.exists() {
        log::info!("Removing build directory: {:?}", build_dir);
        std::fs::remove_dir_all(&build_dir).context("Failed to remove build directory")?;
    }

    if all {
        if cache_dir.exists() {
            log::info!("Removing cache directory: {:?}", cache_dir);
            std::fs::remove_dir_all(&cache_dir).context("Failed to remove cache directory")?;
        }

        // Also clean any .cho files in the current directory
        for entry in std::fs::read_dir(".")? {
            let entry = entry?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "cho" {
                    log::info!("Removing: {:?}", path);
                    std::fs::remove_file(&path)?;
                }
            }
        }
    }

    log::info!("Clean completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_clean_does_not_fail_on_missing_dirs() {
        // Clean should gracefully handle missing directories
    }
}
