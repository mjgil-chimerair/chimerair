//! Schema versioning for Zigmera artifacts.
//!
//! Task 23: Add schema versioning (consumer-side)
//!
//! This module provides schema version validation, compatibility checking,
//! and rejection of incompatible versions before processing.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Current schema version for all Zigmera artifacts
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Minimum supported schema version
pub const MIN_SUPPORTED_VERSION: u32 = 1;

/// Maximum supported schema version (for forward compatibility checks)
pub const MAX_SUPPORTED_VERSION: u32 = 1;

/// Versioning errors
#[derive(Debug, Error)]
pub enum VersionError {
    #[error("schema version {version} is not supported (minimum: {min}, maximum: {max})")]
    VersionOutOfRange { version: u32, min: u32, max: u32 },
    #[error("schema version {version} is too new (maximum supported: {max})")]
    VersionTooNew { version: u32, max: u32 },
    #[error("schema version {version} is too old (minimum supported: {min})")]
    VersionTooOld { version: u32, min: u32 },
    #[error("schema magic mismatch: expected {expected:?}, got {actual:?}")]
    MagicMismatch { expected: [u8; 8], actual: [u8; 8] },
    #[error("checksum verification failed")]
    ChecksumMismatch,
    #[error("corrupt or truncated data")]
    CorruptData,
    #[error("missing required version metadata")]
    MissingVersionMetadata,
}

/// Schema version information embedded in artifact headers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaHeader {
    /// Magic bytes identifying the artifact type
    pub magic: [u8; 8],
    /// Schema version number
    pub version: u32,
    /// Zig compiler commit hash
    pub zig_commit: [u8; 20],
    /// Target triple (as bytes)
    pub target: Vec<u8>,
    /// Backend identifier (as bytes)
    pub backend: Vec<u8>,
    /// Format flags
    pub flags: u32,
}

impl SchemaHeader {
    /// Parse from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, VersionError> {
        if bytes.len() < 132 {
            return Err(VersionError::CorruptData);
        }

        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);

        let mut version_bytes = [0u8; 4];
        version_bytes.copy_from_slice(&bytes[8..12]);
        let version = u32::from_le_bytes(version_bytes);

        let mut zig_commit = [0u8; 20];
        zig_commit.copy_from_slice(&bytes[12..32]);

        let target = bytes[32..96].to_vec();
        let backend = bytes[96..128].to_vec();

        let mut flags_bytes = [0u8; 4];
        flags_bytes.copy_from_slice(&bytes[128..132]);
        let flags = u32::from_le_bytes(flags_bytes);

        Ok(Self {
            magic,
            version,
            zig_commit,
            target,
            backend,
            flags,
        })
    }

    /// Encode to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(132);
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.zig_commit);

        let mut target = vec![0u8; 64];
        target[..self.target.len().min(64)]
            .copy_from_slice(&self.target[..self.target.len().min(64)]);
        bytes.extend_from_slice(&target);

        let mut backend = vec![0u8; 32];
        backend[..self.backend.len().min(32)]
            .copy_from_slice(&self.backend[..self.backend.len().min(32)]);
        bytes.extend_from_slice(&backend);

        bytes.extend_from_slice(&self.flags.to_le_bytes());
        bytes
    }

    /// Get version as u32
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Get target as string
    pub fn target_str(&self) -> String {
        let s = std::str::from_utf8(&self.target)
            .unwrap_or("unknown")
            .trim_end_matches('\0');
        s.to_string()
    }

    /// Get backend as string
    pub fn backend_str(&self) -> String {
        let s = std::str::from_utf8(&self.backend)
            .unwrap_or("unknown")
            .trim_end_matches('\0');
        s.to_string()
    }

    /// Check if a flag is set
    pub fn has_flag(&self, flag: u32) -> bool {
        (self.flags & flag) != 0
    }
}

/// Version compatibility status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityStatus {
    /// Version is fully supported and compatible
    Compatible,
    /// Version is supported but has known limitations
    CompatibleWithWarnings,
    /// Version requires migration (not yet implemented)
    RequiresMigration,
    /// Version is not supported
    Incompatible,
    /// Version is too new to determine compatibility
    UnknownFutureVersion,
}

/// Version compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionCompatibility {
    /// The version being checked
    pub version: u32,
    /// Current schema version
    pub current_version: u32,
    /// Compatibility status
    pub status: CompatibilityStatus,
    /// Human-readable explanation
    pub explanation: String,
    /// Minimum version that can be safely used
    pub min_compatible: Option<u32>,
    /// Maximum version that can be safely used
    pub max_compatible: Option<u32>,
}

impl VersionCompatibility {
    /// Create a new version compatibility result
    pub fn new(version: u32, current: u32, status: CompatibilityStatus, explanation: &str) -> Self {
        Self {
            version,
            current_version: current,
            status,
            explanation: explanation.to_string(),
            min_compatible: Some(MIN_SUPPORTED_VERSION),
            max_compatible: Some(MAX_SUPPORTED_VERSION),
        }
    }

    /// Check if this version is usable
    pub fn is_usable(&self) -> bool {
        matches!(
            self.status,
            CompatibilityStatus::Compatible | CompatibilityStatus::CompatibleWithWarnings
        )
    }

    /// Check if this version requires migration
    pub fn requires_migration(&self) -> bool {
        matches!(self.status, CompatibilityStatus::RequiresMigration)
    }

    /// Check if this version is compatible
    pub fn is_compatible(&self) -> bool {
        matches!(
            self.status,
            CompatibilityStatus::Compatible | CompatibilityStatus::CompatibleWithWarnings
        )
    }
}

/// Schema version checker
#[derive(Debug, Clone)]
pub struct VersionChecker {
    current_version: u32,
    min_supported: u32,
    max_supported: u32,
    strict_mode: bool,
}

impl VersionChecker {
    /// Create a new version checker
    pub fn new() -> Self {
        Self {
            current_version: CURRENT_SCHEMA_VERSION,
            min_supported: MIN_SUPPORTED_VERSION,
            max_supported: MAX_SUPPORTED_VERSION,
            strict_mode: true,
        }
    }

    /// Set strict mode (reject any version outside supported range)
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Get current schema version
    pub fn current_version(&self) -> u32 {
        self.current_version
    }

    /// Get minimum supported version
    pub fn min_supported(&self) -> u32 {
        self.min_supported
    }

    /// Get maximum supported version
    pub fn max_supported(&self) -> u32 {
        self.max_supported
    }

    /// Check if a version is supported
    pub fn is_version_supported(&self, version: u32) -> bool {
        version >= self.min_supported && version <= self.max_supported
    }

    /// Validate a version, returning an error if unsupported
    pub fn validate_version(&self, version: u32) -> Result<(), VersionError> {
        if version < self.min_supported {
            return Err(VersionError::VersionTooOld {
                version,
                min: self.min_supported,
            });
        }
        if version > self.max_supported {
            if self.strict_mode {
                return Err(VersionError::VersionTooNew {
                    version,
                    max: self.max_supported,
                });
            }
        }
        Ok(())
    }

    /// Get compatibility information for a version
    pub fn check_compatibility(&self, version: u32) -> VersionCompatibility {
        if version < self.min_supported {
            return VersionCompatibility::new(
                version,
                self.current_version,
                CompatibilityStatus::Incompatible,
                &format!(
                    "Schema version {} is too old. Minimum supported: {}",
                    version, self.min_supported
                ),
            );
        }

        if version == self.current_version {
            return VersionCompatibility::new(
                version,
                self.current_version,
                CompatibilityStatus::Compatible,
                "Schema version is current and fully supported",
            );
        }

        if version > self.current_version && version <= self.max_supported {
            return VersionCompatibility::new(
                version,
                self.current_version,
                CompatibilityStatus::CompatibleWithWarnings,
                &format!(
                    "Schema version {} is newer than current {}. Some features may be unavailable.",
                    version, self.current_version
                ),
            );
        }

        if version > self.max_supported {
            return VersionCompatibility::new(
                version,
                self.current_version,
                CompatibilityStatus::UnknownFutureVersion,
                &format!(
                    "Schema version {} is too new (max supported: {}). Forward compatibility not guaranteed.",
                    version, self.max_supported
                ),
            );
        }

        // version < current_version but >= min_supported
        VersionCompatibility::new(
            version,
            self.current_version,
            CompatibilityStatus::RequiresMigration,
            &format!(
                "Schema version {} requires migration to current version {}",
                version, self.current_version
            ),
        )
    }

    /// Validate header version
    pub fn validate_header(&self, header: &SchemaHeader) -> Result<(), VersionError> {
        // Validate magic (should be set by caller)
        if header.magic == [0u8; 8] {
            return Err(VersionError::MissingVersionMetadata);
        }

        // Validate version range
        if header.version < self.min_supported {
            return Err(VersionError::VersionTooOld {
                version: header.version,
                min: self.min_supported,
            });
        }

        if header.version > self.max_supported {
            if self.strict_mode {
                return Err(VersionError::VersionTooNew {
                    version: header.version,
                    max: self.max_supported,
                });
            }
        }

        Ok(())
    }

    /// Reject incompatible schema versions (for CI/test)
    pub fn reject_incompatible_version(&self, version: u32) -> Result<(), VersionError> {
        let compatibility = self.check_compatibility(version);
        if !compatibility.is_usable() {
            return Err(VersionError::VersionOutOfRange {
                version,
                min: self.min_supported,
                max: self.max_supported,
            });
        }
        Ok(())
    }
}

impl Default for VersionChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Artifact header validator combining magic, version, and checksum checks
#[derive(Debug, Clone)]
pub struct ArtifactValidator {
    version_checker: VersionChecker,
    expected_magic: Option<[u8; 8]>,
    compute_checksum: bool,
}

impl ArtifactValidator {
    /// Create a new artifact validator
    pub fn new() -> Self {
        Self {
            version_checker: VersionChecker::new(),
            expected_magic: None,
            compute_checksum: false,
        }
    }

    /// Set expected magic bytes
    pub fn with_expected_magic(mut self, magic: [u8; 8]) -> Self {
        self.expected_magic = Some(magic);
        self
    }

    /// Enable checksum computation
    pub fn with_checksum_computation(mut self, enabled: bool) -> Self {
        self.compute_checksum = enabled;
        self
    }

    /// Validate raw artifact bytes
    pub fn validate_raw(&self, data: &[u8]) -> Result<SchemaHeader, VersionError> {
        if data.len() < 132 {
            return Err(VersionError::CorruptData);
        }

        let header = SchemaHeader::from_bytes(data)?;

        // Validate magic if set
        if let Some(expected) = self.expected_magic {
            if header.magic != expected {
                return Err(VersionError::MagicMismatch {
                    expected,
                    actual: header.magic,
                });
            }
        }

        // Validate version
        self.version_checker.validate_header(&header)?;

        Ok(header)
    }

    /// Get the version checker
    pub fn version_checker(&self) -> &VersionChecker {
        &self.version_checker
    }
}

impl Default for ArtifactValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_checker_current() {
        let checker = VersionChecker::new();
        assert_eq!(checker.current_version(), 1);
        assert_eq!(checker.min_supported(), 1);
        assert_eq!(checker.max_supported(), 1);
    }

    #[test]
    fn test_version_checker_valid_current() {
        let checker = VersionChecker::new();
        assert!(checker.validate_version(1).is_ok());
    }

    #[test]
    fn test_version_checker_reject_old() {
        let checker = VersionChecker::new();
        let result = checker.validate_version(0);
        assert!(matches!(result, Err(VersionError::VersionTooOld { .. })));
    }

    #[test]
    fn test_version_checker_reject_future_strict() {
        let checker = VersionChecker::new().with_strict_mode(true);
        let result = checker.validate_version(2);
        assert!(matches!(result, Err(VersionError::VersionTooNew { .. })));
    }

    #[test]
    fn test_version_checker_accept_future_lenient() {
        let checker = VersionChecker::new().with_strict_mode(false);
        assert!(checker.validate_version(2).is_ok());
    }

    #[test]
    fn test_check_compatibility_current() {
        let checker = VersionChecker::new();
        let compat = checker.check_compatibility(1);
        assert_eq!(compat.status, CompatibilityStatus::Compatible);
        assert!(compat.is_usable());
    }

    #[test]
    fn test_check_compatibility_old() {
        let checker = VersionChecker::new();
        let compat = checker.check_compatibility(0);
        assert_eq!(compat.status, CompatibilityStatus::Incompatible);
        assert!(!compat.is_usable());
    }

    #[test]
    fn test_check_compatibility_newer() {
        let checker = VersionChecker::new();
        let compat = checker.check_compatibility(2);
        assert_eq!(compat.status, CompatibilityStatus::UnknownFutureVersion);
    }

    #[test]
    fn test_schema_header_from_bytes() {
        let mut data = vec![0u8; 132];
        data[0..8].copy_from_slice(b"TEST0001");
        data[8..12].copy_from_slice(&1u32.to_le_bytes());

        let header = SchemaHeader::from_bytes(&data).unwrap();
        assert_eq!(header.version, 1);
    }

    #[test]
    fn test_schema_header_to_bytes() {
        let header = SchemaHeader {
            magic: *b"TEST0001",
            version: 1,
            zig_commit: [0u8; 20],
            target: b"x86_64-linux-gnu".to_vec(),
            backend: b"llvm".to_vec(),
            flags: 0,
        };

        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), 132);
    }

    #[test]
    fn test_schema_header_roundtrip() {
        let header = SchemaHeader {
            magic: *b"ZSCHMETA",
            version: 1,
            zig_commit: [1u8; 20],
            target: b"x86_64-linux-gnu".to_vec(),
            backend: b"llvm".to_vec(),
            flags: 0,
        };

        let bytes = header.to_bytes();
        let parsed = SchemaHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.magic, header.magic);
        assert_eq!(parsed.version, header.version);
    }

    #[test]
    fn test_artifact_validator_rejects_invalid_version() {
        let validator = ArtifactValidator::new().with_expected_magic(*b"ZSCHMETA");
        let mut data = vec![0u8; 132];
        data[0..8].copy_from_slice(b"ZSCHMETA");
        data[8..12].copy_from_slice(&100u32.to_le_bytes());

        let result = validator.validate_raw(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_artifact_validator_accepts_valid() {
        let validator = ArtifactValidator::new().with_expected_magic(*b"ZSCHMETA");
        let mut data = vec![0u8; 132];
        data[0..8].copy_from_slice(b"ZSCHMETA");
        data[8..12].copy_from_slice(&1u32.to_le_bytes());

        let result = validator.validate_raw(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_incompatible_version_in_test() {
        let checker = VersionChecker::new();
        assert!(checker.reject_incompatible_version(1).is_ok());
        assert!(checker.reject_incompatible_version(0).is_err());
    }

    #[test]
    fn test_version_compatibility_is_usable() {
        let checker = VersionChecker::new();
        let compat = checker.check_compatibility(1);
        assert!(compat.is_usable());
        assert!(!compat.requires_migration());
        assert!(compat.is_compatible());
    }

    #[test]
    fn test_schema_header_target_str() {
        let header = SchemaHeader {
            magic: [0u8; 8],
            version: 1,
            zig_commit: [0u8; 20],
            target: b"x86_64-linux-gnu".to_vec(),
            backend: b"llvm".to_vec(),
            flags: 0,
        };
        assert_eq!(header.target_str(), "x86_64-linux-gnu");
    }
}
