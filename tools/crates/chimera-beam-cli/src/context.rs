//! CLI context for BEAM operations.
//!
//! Provides shared context and configuration for CLI operations.

use std::collections::HashMap;
use std::path::PathBuf;

/// CLI context for BEAM operations.
#[derive(Debug, Clone)]
pub struct BeamCliContext {
    /// Cache directory.
    pub cache_dir: PathBuf,
    /// Output directory.
    pub output_dir: PathBuf,
    /// Verbosity level.
    pub verbose: u8,
    /// Working directory.
    pub working_dir: PathBuf,
    /// Environment variables.
    pub env: HashMap<String, String>,
}

impl BeamCliContext {
    /// Create a new context.
    pub fn new() -> anyhow::Result<Self> {
        let working_dir = std::env::current_dir()?;
        let cache_dir = working_dir.join(".beam_cache");
        let output_dir = working_dir.join("beam_output");

        // Create directories if needed
        std::fs::create_dir_all(&cache_dir)?;
        std::fs::create_dir_all(&output_dir)?;

        let mut env = HashMap::new();
        for (key, value) in std::env::vars() {
            env.insert(key, value);
        }

        Ok(BeamCliContext {
            cache_dir,
            output_dir,
            verbose: 0,
            working_dir,
            env,
        })
    }

    /// Create with custom cache directory.
    pub fn with_cache_dir(cache_dir: PathBuf) -> anyhow::Result<Self> {
        let mut ctx = Self::new()?;
        ctx.cache_dir = cache_dir;
        std::fs::create_dir_all(&ctx.cache_dir)?;
        Ok(ctx)
    }

    /// Set verbosity.
    pub fn set_verbose(&mut self, level: u8) {
        self.verbose = level;
    }

    /// Get a cache file path.
    pub fn cache_path(&self, module_name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.cache", module_name))
    }

    /// Get an output file path.
    pub fn output_path(&self, module_name: &str, extension: &str) -> PathBuf {
        self.output_dir
            .join(format!("{}.{}", module_name, extension))
    }
}

impl Default for BeamCliContext {
    fn default() -> Self {
        Self::new().expect("Failed to create default context")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = BeamCliContext::new();
        assert!(ctx.is_ok());
    }

    #[test]
    fn test_context_cache_path() {
        let ctx = BeamCliContext::new().unwrap();
        let path = ctx.cache_path("test_module");
        assert!(path.to_string_lossy().contains("test_module"));
    }
}
