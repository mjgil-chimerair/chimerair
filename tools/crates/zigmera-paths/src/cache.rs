//! Cache path handling.

use std::path::{Path, PathBuf};

/// A cache path with hash components for content-addressable storage.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CachePath {
    target: String,
    profile: String,
    key_hash: String,
    kind: CacheKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheKind {
    Snapshot,
    DepGraph,
    Airpack,
    Lowered,
    Object,
    Wrapper,
}

/// Cache key components for content-addressable storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheKeyComponents {
    pub target: String,
    pub profile: String,
    pub semantic_fingerprint: String,
    pub dependency_hashes: Vec<String>,
    pub schema_version: u32,
}

impl CachePath {
    pub fn new(target: &str, profile: &str, key_hash: &str, kind: CacheKind) -> Self {
        Self {
            target: target.to_string(),
            profile: profile.to_string(),
            key_hash: key_hash.to_string(),
            kind,
        }
    }

    pub fn from_components(
        base_dir: &Path,
        components: &CacheKeyComponents,
        kind: CacheKind,
    ) -> PathBuf {
        let kind_dir = match kind {
            CacheKind::Snapshot => "snapshots",
            CacheKind::DepGraph => "depgraphs",
            CacheKind::Airpack => "airpacks",
            CacheKind::Lowered => "lowered",
            CacheKind::Object => "objects",
            CacheKind::Wrapper => "wrappers",
        };
        base_dir
            .join(&components.target)
            .join(&components.profile)
            .join(kind_dir)
            .join(&components.semantic_fingerprint)
    }

    pub fn from_components_in_workspace(
        workspace_root: &Path,
        components: &CacheKeyComponents,
        kind: CacheKind,
    ) -> PathBuf {
        Self::from_components(
            &workspace_root.join(".zigmera").join("cache"),
            components,
            kind,
        )
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn profile(&self) -> &str {
        &self.profile
    }

    pub fn key_hash(&self) -> &str {
        &self.key_hash
    }

    pub fn kind(&self) -> CacheKind {
        self.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_path_new() {
        let p = CachePath::new("x86_64-linux", "release", "abc123", CacheKind::Snapshot);
        assert_eq!(p.target(), "x86_64-linux");
        assert_eq!(p.profile(), "release");
        assert_eq!(p.key_hash(), "abc123");
        assert_eq!(p.kind(), CacheKind::Snapshot);
    }

    #[test]
    fn test_cache_kind_variants() {
        assert!(matches!(CacheKind::Snapshot, CacheKind::Snapshot));
        assert!(matches!(CacheKind::Airpack, CacheKind::Airpack));
        assert!(matches!(CacheKind::Lowered, CacheKind::Lowered));
    }

    #[test]
    fn test_cache_key_components() {
        let components = CacheKeyComponents {
            target: "x86_64-linux".to_string(),
            profile: "release".to_string(),
            semantic_fingerprint: "fp123".to_string(),
            dependency_hashes: vec!["dep1".to_string(), "dep2".to_string()],
            schema_version: 1,
        };
        let base = PathBuf::from("/cache");
        let path = CachePath::from_components(&base, &components, CacheKind::Airpack);
        assert!(path.to_string_lossy().contains("airpacks"));
    }

    #[test]
    fn test_cache_workspace_contract() {
        let components = CacheKeyComponents {
            target: "x86_64-unknown-linux-gnu".to_string(),
            profile: "debug".to_string(),
            semantic_fingerprint: "semfp123".to_string(),
            dependency_hashes: vec![],
            schema_version: 1,
        };
        let path = CachePath::from_components_in_workspace(
            Path::new("/workspace"),
            &components,
            CacheKind::Snapshot,
        );
        assert_eq!(
            path,
            PathBuf::from(
                "/workspace/.zigmera/cache/x86_64-unknown-linux-gnu/debug/snapshots/semfp123"
            )
        );
    }
}
