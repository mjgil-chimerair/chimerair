//! `chimera-package` - Runtime packaging for ChimeraIR.
//!
//! This crate handles runtime delivery for `runtime-dlopen` and `dynamic-link` ABI edges.
//! It packages cdylibs, config files, generated wrappers, and dynamic-loader path hints.

use chimera_artifact::{RuntimeDelivery, RuntimeFile};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("source file not found: {0}")]
    SourceNotFound(PathBuf),
    #[error("destination already exists: {0}")]
    DestExists(PathBuf),
    #[error("failed to copy file: {0}")]
    CopyFailed(String),
    #[error("invalid package layout")]
    InvalidLayout,
}

/// Runtime packager for ChimeraIR.
pub struct Packager {
    /// Package root directory
    root: PathBuf,
    /// Platform-specific settings
    platform: PlatformSettings,
}

/// Platform-specific packaging settings.
#[derive(Debug, Clone)]
pub struct PlatformSettings {
    /// Shared library extension (.so, .dll, .dylib)
    pub lib_ext: String,
    /// Shared library prefix (lib, "")
    pub lib_prefix: String,
    /// rpath prefix (@executable_path/, $ORIGIN/)
    pub rpath_prefix: String,
    /// Install name prefix (macOS)
    pub install_name_prefix: String,
}

impl Default for PlatformSettings {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        {
            PlatformSettings {
                lib_ext: "dylib".to_string(),
                lib_prefix: "lib".to_string(),
                rpath_prefix: "@executable_path/".to_string(),
                install_name_prefix: "/usr/local/lib/".to_string(),
            }
        }
        #[cfg(target_os = "windows")]
        {
            PlatformSettings {
                lib_ext: "dll".to_string(),
                lib_prefix: "".to_string(),
                rpath_prefix: "".to_string(),
                install_name_prefix: "".to_string(),
            }
        }
        #[cfg(target_os = "linux")]
        {
            PlatformSettings {
                lib_ext: "so".to_string(),
                lib_prefix: "lib".to_string(),
                rpath_prefix: "$ORIGIN/".to_string(),
                install_name_prefix: "".to_string(),
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            PlatformSettings {
                lib_ext: "so".to_string(),
                lib_prefix: "lib".to_string(),
                rpath_prefix: "".to_string(),
                install_name_prefix: "".to_string(),
            }
        }
    }
}

impl Packager {
    /// Create a new packager.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Packager {
            root: root.into(),
            platform: PlatformSettings::default(),
        }
    }

    /// Get the package root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Package runtime files according to the delivery spec.
    pub fn package(&self, delivery: &RuntimeDelivery) -> Result<PathBuf, PackageError> {
        let pkg_dir = self.root.join("runtime");

        for file in &delivery.files {
            let source = &file.source;
            if !source.exists() {
                return Err(PackageError::SourceNotFound(source.clone()));
            }

            let dest = pkg_dir.join(&file.dest_name);

            // Copy the file
            if let Err(e) = std::fs::copy(source, &dest) {
                return Err(PackageError::CopyFailed(e.to_string()));
            }

            // Set executable if needed
            if file.executable {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = std::fs::metadata(&dest)
                        .map_err(|_| PackageError::CopyFailed("failed to get perms".to_string()))?
                        .permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(&dest, perms)
                        .map_err(|_| PackageError::CopyFailed("failed to set perms".to_string()))?;
                }
            }
        }

        Ok(pkg_dir)
    }

    /// Generate an rpath for a runtime library.
    pub fn make_rpath(&self, lib_path: &Path) -> String {
        format!("{}{}", self.platform.rpath_prefix, lib_path.display())
    }

    /// Generate an install name for macOS.
    #[cfg(target_os = "macos")]
    pub fn make_install_name(&self, lib_path: &Path) -> String {
        format!(
            "{}{}",
            self.platform.install_name_prefix,
            lib_path.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_platform_settings_default() {
        let settings = PlatformSettings::default();
        assert!(!settings.lib_ext.is_empty());
    }

    #[test]
    fn test_packager_root() {
        let packager = Packager::new("/tmp/package");
        assert_eq!(packager.root(), Path::new("/tmp/package"));
    }

    #[test]
    fn test_packager_rpath() {
        let packager = Packager::new("/tmp/package");
        let rpath = packager.make_rpath(Path::new("lib/libfoo.so"));
        #[cfg(target_os = "linux")]
        assert_eq!(rpath, "$ORIGIN/lib/libfoo.so");
    }
}
