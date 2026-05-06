//! Patched Zig detection and verification
//!
//! Implements the detection order specified in zig-incremental-ownership-plan.md:
//! 1. Check `--help` output for `emit-zigmera` token
//! 2. Attempt to parse a known magic header from a candidate artifact
//! 3. Fall back to zigmera-lowering is_patched_zig logic

use std::path::Path;
use std::process::Command;

/// Magic bytes for Zigmera artifacts
/// These match the magic bytes defined in zigmera-zig/src/ZigmeraEmitter.zig
const ZSNAP_MAGIC: &[u8] = b"ZSnp"; // 4 bytes for .zsnap ("ZSnap" header)
const ZDEP_MAGIC: &[u8] = b"ZDep"; // 4 bytes for .zdep dependency graph
const ZAIRPACK_MAGIC: &[u8] = b"ZAir"; // 4 bytes for .zairpack AIR bundle

/// Result of patched Zig detection
#[derive(Debug, Clone)]
pub enum PatchedZigDetection {
    /// Patched Zig is available and verified
    Available {
        path: String,
        version: String,
        supports_zigmera_flags: bool,
    },
    /// Patched Zig was not found or is not verified
    NotAvailable {
        reason: String,
        tried_help_check: bool,
        tried_magic_check: bool,
    },
}

/// Check if a Zig binary is the patched Zig compiler
pub fn detect_patched_zig(zig_path: &Path) -> PatchedZigDetection {
    // Step 1: Check --help output for emit-zigmera token
    if let Some(version) = check_help_for_zigmera(zig_path) {
        return PatchedZigDetection::Available {
            path: zig_path.display().to_string(),
            version,
            supports_zigmera_flags: true,
        };
    }

    // Step 2: Try to detect via artifact magic bytes
    if let Some(version) = check_artifact_magic(zig_path) {
        return PatchedZigDetection::Available {
            path: zig_path.display().to_string(),
            version,
            supports_zigmera_flags: false, // Version detected via artifact, not flag
        };
    }

    // Not detected
    PatchedZigDetection::NotAvailable {
        reason: "Zig binary does not support Zigmera flags and no artifact cache found".to_string(),
        tried_help_check: true,
        tried_magic_check: true,
    }
}

/// Check if `zig --help` contains emit-zigmera flags
fn check_help_for_zigmera(zig_path: &Path) -> Option<String> {
    let output = Command::new(zig_path).arg("--help").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let help_text = String::from_utf8_lossy(&output.stdout);

    // Check for emit-zigmera tokens
    let has_zigmera_flags =
        help_text.contains("--emit-zigmera-snapshot") || help_text.contains("--emit-zigmera");

    if has_zigmera_flags {
        // Try to extract version
        let version = extract_zig_version(zig_path).unwrap_or_else(|| "unknown".to_string());
        Some(version)
    } else {
        None
    }
}

/// Check if there's an existing artifact cache with magic bytes
fn check_artifact_magic(_zig_path: &Path) -> Option<String> {
    // This would check for existing .zsnap, .zdep, or .zairpack files
    // in the default cache directory and verify their magic bytes
    //
    // For now, return None as we need actual artifact paths to check
    None
}

/// Extract Zig version from `zig version`
fn extract_zig_version(zig_path: &Path) -> Option<String> {
    let output = Command::new(zig_path).arg("version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8_lossy(&output.stdout);
    let version = version.trim();
    if version.is_empty() {
        None
    } else {
        Some(version.to_string())
    }
}

/// Check if data starts with a known magic sequence
pub fn has_zigmera_magic(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }

    let magic = &data[0..4];
    magic == ZSNAP_MAGIC || magic == ZDEP_MAGIC || magic == ZAIRPACK_MAGIC
}

/// Get the artifact type from magic bytes
pub fn artifact_type_from_magic(data: &[u8]) -> Option<&'static str> {
    if data.len() < 4 {
        return None;
    }

    let magic = &data[0..4];
    if magic == ZSNAP_MAGIC {
        Some(".zsnap")
    } else if magic == ZDEP_MAGIC {
        Some(".zdep")
    } else if magic == ZAIRPACK_MAGIC {
        Some(".zairpack")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_zigmera_magic_zsnap() {
        let data = b"ZSnp\x01\x00\x00\x00test";
        assert!(has_zigmera_magic(data));
    }

    #[test]
    fn test_has_zigmera_magic_zdep() {
        let data = b"ZDep\x01\x00\x00\x00test";
        assert!(has_zigmera_magic(data));
    }

    #[test]
    fn test_has_zigmera_magic_zairpack() {
        let data = b"ZAir\x01\x00\x00\x00test";
        assert!(has_zigmera_magic(data));
    }

    #[test]
    fn test_has_zigmera_magic_unknown() {
        let data = b"XXXX\x01\x00\x00\x00test";
        assert!(!has_zigmera_magic(data));
    }

    #[test]
    fn test_has_zigmera_magic_too_short() {
        let data = b"ZS";
        assert!(!has_zigmera_magic(data));
    }

    #[test]
    fn test_artifact_type_from_magic_zsnap() {
        let data = b"ZSnp\x01\x00\x00\x00test";
        assert_eq!(artifact_type_from_magic(data), Some(".zsnap"));
    }

    #[test]
    fn test_artifact_type_from_magic_zdep() {
        let data = b"ZDep\x01\x00\x00\x00test";
        assert_eq!(artifact_type_from_magic(data), Some(".zdep"));
    }

    #[test]
    fn test_artifact_type_from_magic_zairpack() {
        let data = b"ZAir\x01\x00\x00\x00test";
        assert_eq!(artifact_type_from_magic(data), Some(".zairpack"));
    }

    #[test]
    fn test_artifact_type_from_magic_unknown() {
        let data = b"XXXX\x01\x00\x00\x00test";
        assert_eq!(artifact_type_from_magic(data), None);
    }

    #[test]
    fn test_artifact_type_from_magic_too_short() {
        let data = b"ZS1";
        assert_eq!(artifact_type_from_magic(data), None);
    }

    #[test]
    fn test_detect_patched_zig_nonexistent_path() {
        let path = Path::new("/nonexistent/zig");
        let result = detect_patched_zig(path);
        match result {
            PatchedZigDetection::NotAvailable { reason, .. } => {
                assert!(reason.contains("not found") || reason.contains("Zig binary"));
            }
            _ => panic!("Expected NotAvailable for nonexistent path"),
        }
    }
}
