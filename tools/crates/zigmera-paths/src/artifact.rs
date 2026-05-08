//! Artifact path handling.

use std::path::{Path, PathBuf};

/// An artifact path for Zigmera output files.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtifactPath {
    target: String,
    profile: String,
    artifact_kind: String,
    name: String,
}

impl ArtifactPath {
    pub fn new(target: &str, profile: &str, artifact_kind: &str, name: &str) -> Self {
        Self {
            target: target.to_string(),
            profile: profile.to_string(),
            artifact_kind: artifact_kind.to_string(),
            name: name.to_string(),
        }
    }

    pub fn zsnap(target: &str, profile: &str, name: &str) -> Self {
        Self::new(target, profile, "zsnap", name)
    }

    pub fn zdep(target: &str, profile: &str, name: &str) -> Self {
        Self::new(target, profile, "zdep", name)
    }

    pub fn zairpack(target: &str, profile: &str, name: &str) -> Self {
        Self::new(target, profile, "zairpack", name)
    }

    pub fn zchmeta(target: &str, profile: &str, name: &str) -> Self {
        Self::new(target, profile, "zchmeta", name)
    }

    pub fn zchproof(target: &str, profile: &str, name: &str) -> Self {
        Self::new(target, profile, "zchproof", name)
    }

    pub fn cho(target: &str, profile: &str, name: &str) -> Self {
        Self::new(target, profile, "cho", name)
    }

    pub fn chir(target: &str, profile: &str, name: &str) -> Self {
        Self::new(target, profile, "chir", name)
    }

    /// Build the full artifact path under a base directory.
    pub fn resolve(&self, base_dir: &PathBuf) -> PathBuf {
        base_dir
            .join(&self.target)
            .join(&self.profile)
            .join(&self.artifact_kind)
            .join(&self.name)
    }

    /// Resolve the artifact using the shared workspace `.zigmera/artifacts` layout.
    pub fn resolve_in_workspace(&self, workspace_root: &Path) -> PathBuf {
        self.resolve(&workspace_root.join(".zigmera").join("artifacts"))
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn profile(&self) -> &str {
        &self.profile
    }

    pub fn artifact_kind(&self) -> &str {
        &self.artifact_kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_path_zsnap() {
        let p = ArtifactPath::zsnap("x86_64-linux", "release", "main.zsnap");
        assert_eq!(p.target(), "x86_64-linux");
        assert_eq!(p.profile(), "release");
        assert_eq!(p.artifact_kind(), "zsnap");
        assert_eq!(p.name(), "main.zsnap");
    }

    #[test]
    fn test_artifact_path_resolve() {
        let p = ArtifactPath::zchmeta("aarch64-linux", "debug", "lib.zchmeta");
        let base = PathBuf::from("/artifacts");
        let resolved = p.resolve(&base);
        assert!(resolved.to_string_lossy().contains("aarch64-linux"));
        assert!(resolved.to_string_lossy().contains("debug"));
        assert!(resolved.to_string_lossy().contains("zchmeta"));
        assert!(resolved.to_string_lossy().contains("lib.zchmeta"));
    }

    #[test]
    fn test_artifact_path_zairpack() {
        let p = ArtifactPath::zairpack("x86_64-windows", "release", "foo.zairpack");
        assert_eq!(p.artifact_kind(), "zairpack");
    }

    #[test]
    fn test_artifact_path_workspace_contract() {
        let p = ArtifactPath::zchproof("x86_64-unknown-linux-gnu", "release", "main.zchproof");
        let resolved = p.resolve_in_workspace(Path::new("/workspace"));
        assert_eq!(
            resolved,
            PathBuf::from(
                "/workspace/.zigmera/artifacts/x86_64-unknown-linux-gnu/release/zchproof/main.zchproof"
            )
        );
    }
}
