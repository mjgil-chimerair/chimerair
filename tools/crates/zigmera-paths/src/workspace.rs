//! Workspace-relative path handling.

use std::path::{Path, PathBuf};

/// A workspace-relative path with package context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspacePath {
    package_id: Option<String>,
    relative: PathBuf,
}

impl WorkspacePath {
    pub fn root() -> Self {
        Self {
            package_id: None,
            relative: PathBuf::from(""),
        }
    }

    pub fn package(package_id: &str, relative: PathBuf) -> Self {
        Self {
            package_id: Some(package_id.to_string()),
            relative,
        }
    }

    pub fn file(relative: PathBuf) -> Self {
        Self {
            package_id: None,
            relative,
        }
    }

    pub fn package_id(&self) -> Option<&str> {
        self.package_id.as_deref()
    }

    pub fn relative(&self) -> &Path {
        &self.relative
    }

    pub fn to_string_lossy(&self) -> String {
        match &self.package_id {
            Some(pkg) => format!("{}:{}", pkg, self.relative.display()),
            None => self.relative.to_string_lossy().to_string(),
        }
    }
}

impl From<&str> for WorkspacePath {
    fn from(s: &str) -> Self {
        if let Some((pkg, rel)) = s.split_once(':') {
            Self::package(pkg, PathBuf::from(rel))
        } else {
            Self::file(PathBuf::from(s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_path_root() {
        let p = WorkspacePath::root();
        assert!(p.package_id().is_none());
        assert_eq!(p.relative(), Path::new(""));
    }

    #[test]
    fn test_workspace_path_package() {
        let p = WorkspacePath::package("my-pkg", PathBuf::from("src/lib.zig"));
        assert_eq!(p.package_id(), Some("my-pkg"));
        assert_eq!(p.relative(), Path::new("src/lib.zig"));
    }

    #[test]
    fn test_workspace_path_from_str() {
        let p = WorkspacePath::from("my-pkg:src/lib.zig");
        assert_eq!(p.package_id(), Some("my-pkg"));
        assert_eq!(p.relative(), Path::new("src/lib.zig"));
    }

    #[test]
    fn test_workspace_path_from_str_no_package() {
        let p = WorkspacePath::from("src/lib.zig");
        assert!(p.package_id().is_none());
        assert_eq!(p.relative(), Path::new("src/lib.zig"));
    }

    #[test]
    fn test_workspace_path_to_string() {
        let p = WorkspacePath::package("foo", PathBuf::from("bar.zig"));
        assert_eq!(p.to_string_lossy(), "foo:bar.zig");
    }
}
