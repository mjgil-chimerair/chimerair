//! Path normalization utilities.

use std::path::{Path, PathBuf};

/// Kind of path being normalized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathKind {
    Source,
    Package,
    Cache,
    Artifact,
    BuildOutput,
}

/// A normalized, canonical path with kind and workspace-relative ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedPath {
    kind: PathKind,
    workspace_rel: PathBuf,
    absolute: PathBuf,
}

impl std::hash::Hash for NormalizedPath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.absolute.to_string_lossy().hash(state);
    }
}

impl NormalizedPath {
    pub fn new(kind: PathKind, workspace_rel: PathBuf, absolute: PathBuf) -> Self {
        Self {
            kind,
            workspace_rel,
            absolute,
        }
    }

    pub fn kind(&self) -> PathKind {
        self.kind
    }

    pub fn workspace_relative(&self) -> &Path {
        &self.workspace_rel
    }

    pub fn absolute(&self) -> &Path {
        &self.absolute
    }

    pub fn as_str(&self) -> Option<&str> {
        self.absolute.to_str()
    }
}

impl serde::Serialize for NormalizedPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.absolute.to_str().unwrap_or(""))
    }
}

/// Normalizes paths for Zigmera workspace, handling Windows/Unix differences.
#[derive(Debug)]
pub struct PathNormalizer {
    workspace_root: PathBuf,
    cache_dir: PathBuf,
    artifact_dir: PathBuf,
}

impl PathNormalizer {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            cache_dir: PathBuf::from(".zigmera/cache"),
            artifact_dir: PathBuf::from(".zigmera/artifacts"),
        }
    }

    pub fn with_custom_dirs(mut self, cache_dir: PathBuf, artifact_dir: PathBuf) -> Self {
        self.cache_dir = cache_dir;
        self.artifact_dir = artifact_dir;
        self
    }

    /// Normalize a source path to a canonical form.
    pub fn normalize_source(&self, path: &Path) -> NormalizedPath {
        let abs = self.canonicalize(path);
        let rel = self.relativize(&abs);
        NormalizedPath::new(PathKind::Source, rel, abs)
    }

    /// Normalize a package path.
    pub fn normalize_package(&self, path: &Path) -> NormalizedPath {
        let abs = self.canonicalize(path);
        let rel = self.relativize(&abs);
        NormalizedPath::new(PathKind::Package, rel, abs)
    }

    /// Normalize a cache path.
    pub fn normalize_cache(&self, path: &Path) -> NormalizedPath {
        let abs = self.canonicalize(path);
        let rel = self.strip_prefix_if_present(&abs, &self.cache_dir);
        NormalizedPath::new(PathKind::Cache, rel.unwrap_or(abs.clone()), abs)
    }

    /// Normalize an artifact path.
    pub fn normalize_artifact(&self, path: &Path) -> NormalizedPath {
        let abs = self.canonicalize(path);
        let rel = self.strip_prefix_if_present(&abs, &self.artifact_dir);
        NormalizedPath::new(PathKind::Artifact, rel.unwrap_or(abs.clone()), abs)
    }

    /// Compute relative path from workspace root.
    pub fn relativize(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.workspace_root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| path.to_path_buf())
    }

    /// Canonicalize a path, resolving symlinks and normalizing separators.
    fn canonicalize(&self, path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| {
            // On failure, normalize separators and return as-is
            let s = path.as_os_str().to_string_lossy();
            PathBuf::from(s.replace('\\', "/"))
        })
    }

    /// Strip a prefix from a path if present.
    fn strip_prefix_if_present(&self, path: &Path, prefix: &Path) -> Option<PathBuf> {
        path.strip_prefix(prefix).ok().map(|p| p.to_path_buf())
    }

    /// Get the cache directory path for a target/profile.
    pub fn cache_dir_for(&self, target: &str, profile: &str) -> PathBuf {
        self.cache_dir.join(target).join(profile)
    }

    /// Get the artifact directory path for a target/profile.
    pub fn artifact_dir_for(&self, target: &str, profile: &str) -> PathBuf {
        self.artifact_dir.join(target).join(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_path_kind() {
        let path = NormalizedPath::new(
            PathKind::Source,
            PathBuf::from("src/main.zig"),
            PathBuf::from("/workspace/src/main.zig"),
        );
        assert_eq!(path.kind(), PathKind::Source);
        assert_eq!(path.workspace_relative(), Path::new("src/main.zig"));
    }

    #[test]
    fn test_path_kind_variants() {
        assert!(matches!(PathKind::Source, PathKind::Source));
        assert!(matches!(PathKind::Cache, PathKind::Cache));
        assert!(matches!(PathKind::Artifact, PathKind::Artifact));
    }
}
