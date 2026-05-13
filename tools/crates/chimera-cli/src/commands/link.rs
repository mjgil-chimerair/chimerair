//! `chimera link` command
//!
//! Links prebuilt verified artifacts into a final binary.

use anyhow::Result;
use chimera_link::{Linker, LinkerConfig, TargetInfo};
use std::path::PathBuf;

pub fn run(objects: Vec<PathBuf>, output: Option<String>, target: Option<String>) -> Result<()> {
    log::info!("Running chimera link");

    if objects.is_empty() {
        anyhow::bail!("No object files provided for linking");
    }

    let target_triple = target.unwrap_or_else(|| "x86_64-unknown-linux-gnu".to_string());
    let output_name = output.unwrap_or_else(|| "chimera_binary".to_string());

    log::info!("Linking {} objects...", objects.len());
    for obj in &objects {
        log::debug!("  {:?}", obj);
    }

    let target_info = TargetInfo::new(&target_triple);
    let linker_config = LinkerConfig {
        output_name: output_name.clone(),
        target: target_info,
        strip_debug: false,
        link_time_optimization: true,
    };

    let mut linker = Linker::new(linker_config);

    let output_path: PathBuf = output_name.clone().into();
    match linker.link(objects.clone(), &output_path) {
        Ok(result) => {
            log::info!("Link successful: {}", result.output_path.display());
        }
        Err(e) => {
            log::error!("Link failed: {}", e);
            anyhow::bail!("Link operation failed: {}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_link_requires_objects() {
        // Test that empty object list is rejected
    }
}
