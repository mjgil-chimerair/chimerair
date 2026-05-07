//! Schema versioning for BEAM adapter artifacts.

use serde::{Deserialize, Serialize};

/// Current schema version for all BEAM artifacts.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Minimum supported schema version.
pub const MIN_SUPPORTED_VERSION: u32 = 1;

/// Maximum supported schema version (for forward compatibility).
pub const MAX_SUPPORTED_VERSION: u32 = 1;

/// Version error for schema compatibility issues.
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("schema version {version} is not supported (min: {min}, max: {max})", version = version, min = MIN_SUPPORTED_VERSION, max = MAX_SUPPORTED_VERSION)]
    UnsupportedVersion { version: u32 },
    #[error("truncated version header")]
    TruncatedHeader,
    #[error("version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },
}

/// Schema header trait for version checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaHeader {
    pub magic: [u8; 8],
    pub schema_version: u32,
    pub min_adapter_version: u32,
}

impl SchemaHeader {
    pub fn validate_version(&self) -> Result<(), VersionError> {
        if self.schema_version < MIN_SUPPORTED_VERSION
            || self.schema_version > MAX_SUPPORTED_VERSION
        {
            return Err(VersionError::UnsupportedVersion {
                version: self.schema_version,
            });
        }
        Ok(())
    }
}

/// Version compatibility status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompatibilityStatus {
    /// Schema versions are fully compatible.
    Compatible,
    /// Schema versions are compatible with some considerations.
    CompatibleWithWarning,
    /// Schema versions are incompatible.
    Incompatible,
}

/// Version checker for BEAM schemas.
pub struct VersionChecker;

impl VersionChecker {
    pub fn check(adapter_version: u32, artifact_version: u32) -> CompatibilityStatus {
        if artifact_version < MIN_SUPPORTED_VERSION || artifact_version > MAX_SUPPORTED_VERSION {
            return CompatibilityStatus::Incompatible;
        }
        if adapter_version > artifact_version {
            return CompatibilityStatus::CompatibleWithWarning;
        }
        CompatibilityStatus::Compatible
    }

    pub fn validate_adapter(
        min_adapter_version: u32,
        adapter_version: u32,
    ) -> Result<(), VersionError> {
        if adapter_version < min_adapter_version {
            return Err(VersionError::VersionMismatch {
                expected: min_adapter_version,
                actual: adapter_version,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        assert_eq!(CURRENT_SCHEMA_VERSION, 1);
        assert_eq!(MIN_SUPPORTED_VERSION, 1);
        assert_eq!(MAX_SUPPORTED_VERSION, 1);
    }

    #[test]
    fn test_schema_header_validate_version() {
        let header = SchemaHeader {
            magic: *b"BeamSnap",
            schema_version: 1,
            min_adapter_version: 1,
        };
        assert!(header.validate_version().is_ok());
    }

    #[test]
    fn test_schema_header_invalid_version() {
        let header = SchemaHeader {
            magic: *b"BeamSnap",
            schema_version: 99,
            min_adapter_version: 1,
        };
        assert!(header.validate_version().is_err());
    }

    #[test]
    fn test_version_checker_compatible() {
        let status = VersionChecker::check(1, 1);
        assert_eq!(status, CompatibilityStatus::Compatible);
    }

    #[test]
    fn test_version_checker_incompatible() {
        let status = VersionChecker::check(1, 99);
        assert_eq!(status, CompatibilityStatus::Incompatible);
    }

    #[test]
    fn test_version_checker_adapter_newer() {
        let status = VersionChecker::check(2, 1);
        assert_eq!(status, CompatibilityStatus::CompatibleWithWarning);
    }

    #[test]
    fn test_validate_adapter_ok() {
        assert!(VersionChecker::validate_adapter(1, 1).is_ok());
        assert!(VersionChecker::validate_adapter(1, 2).is_ok());
    }

    #[test]
    fn test_validate_adapter_too_old() {
        let result = VersionChecker::validate_adapter(2, 1);
        assert!(result.is_err());
    }
}
