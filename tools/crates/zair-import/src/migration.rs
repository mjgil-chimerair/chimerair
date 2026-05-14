//! `.zairpack` migration support for schema version compatibility.
//!
//! Task 49: Add `.zairpack` migration support.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Migration errors
#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("unsupported schema version: {0}")]
    UnsupportedVersion(u32),
    #[error("schema too new: {0} (maximum supported: {1})")]
    SchemaTooNew(u32, u32),
    #[error("migration not available for version: {0} -> {1}")]
    NoMigrationPath(u32, u32),
    #[error("invalid schema data during migration")]
    InvalidData,
}

/// Supported schema versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaVersion {
    /// Version 1 (current)
    V1 = 1,
    /// Version 2 (future)
    V2 = 2,
}

impl SchemaVersion {
    /// Get the numeric value
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    /// Parse from u32
    pub fn from_u32(val: u32) -> Option<Self> {
        match val {
            1 => Some(SchemaVersion::V1),
            2 => Some(SchemaVersion::V2),
            _ => None,
        }
    }

    /// Check if migration is needed to reach target version
    pub fn needs_migration(&self, target: SchemaVersion) -> bool {
        self.as_u32() != target.as_u32()
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        SchemaVersion::V1
    }
}

/// Migration path information
#[derive(Debug, Clone)]
pub struct MigrationPath {
    /// Source version
    pub from: SchemaVersion,
    /// Target version
    pub to: SchemaVersion,
    /// Is migration available?
    pub available: bool,
}

/// Migration support status
#[derive(Debug, Clone)]
pub struct MigrationSupport {
    /// Current schema version
    pub current_version: SchemaVersion,
    /// Minimum supported version
    pub min_supported: SchemaVersion,
    /// Maximum supported version
    pub max_supported: SchemaVersion,
    /// Available migration paths
    pub paths: Vec<MigrationPath>,
}

impl MigrationSupport {
    /// Create migration support for current version
    pub fn new(current: SchemaVersion) -> Self {
        let min_supported = SchemaVersion::V1;
        let max_supported = SchemaVersion::V2;

        let paths = vec![
            MigrationPath {
                from: SchemaVersion::V1,
                to: SchemaVersion::V1,
                available: true, // No migration needed
            },
            MigrationPath {
                from: SchemaVersion::V1,
                to: SchemaVersion::V2,
                available: false, // V1 -> V2 not yet implemented
            },
        ];

        Self {
            current_version: current,
            min_supported,
            max_supported,
            paths,
        }
    }

    /// Check if a version is supported
    pub fn is_version_supported(&self, version: SchemaVersion) -> bool {
        version.as_u32() >= self.min_supported.as_u32()
            && version.as_u32() <= self.max_supported.as_u32()
    }

    /// Check if migration is available between versions
    pub fn can_migrate(&self, from: SchemaVersion, to: SchemaVersion) -> bool {
        self.paths.iter()
            .find(|p| p.from == from && p.to == to)
            .map(|p| p.available)
            .unwrap_or(false)
    }

    /// Get migration error for unsupported version
    pub fn check_version(&self, version: u32) -> Result<SchemaVersion, MigrationError> {
        SchemaVersion::from_u32(version)
            .filter(|v| self.is_version_supported(*v))
            .ok_or_else(|| {
                if version > self.max_supported.as_u32() {
                    MigrationError::SchemaTooNew(version, self.max_supported.as_u32())
                } else {
                    MigrationError::UnsupportedVersion(version)
                }
            })
    }
}

/// Migrator for converting between schema versions
#[derive(Debug, Clone)]
pub struct SchemaMigrator {
    /// Current migration support
    support: MigrationSupport,
}

impl SchemaMigrator {
    /// Create a new migrator
    pub fn new(current: SchemaVersion) -> Self {
        Self {
            support: MigrationSupport::new(current),
        }
    }

    /// Get migration support info
    pub fn support(&self) -> &MigrationSupport {
        &self.support
    }

    /// Migrate data from one version to another
    pub fn migrate(&self, _data: &[u8], from: SchemaVersion, to: SchemaVersion) -> Result<Vec<u8>, MigrationError> {
        if !from.needs_migration(to) {
            return Ok(_data.to_vec());
        }

        if !self.support.can_migrate(from, to) {
            return Err(MigrationError::NoMigrationPath(from.as_u32(), to.as_u32()));
        }

        // Placeholder for actual migration logic
        // In production, this would transform the binary data
        Err(MigrationError::InvalidData)
    }

    /// Validate a version is acceptable
    pub fn validate_version(&self, version: u32) -> Result<(), MigrationError> {
        self.support.check_version(version).map(|_| ())
    }
}

impl Default for SchemaMigrator {
    fn default() -> Self {
        Self::new(SchemaVersion::V1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version_from_u32() {
        assert_eq!(SchemaVersion::from_u32(1), Some(SchemaVersion::V1));
        assert_eq!(SchemaVersion::from_u32(2), Some(SchemaVersion::V2));
        assert_eq!(SchemaVersion::from_u32(3), None);
    }

    #[test]
    fn test_schema_version_as_u32() {
        assert_eq!(SchemaVersion::V1.as_u32(), 1);
        assert_eq!(SchemaVersion::V2.as_u32(), 2);
    }

    #[test]
    fn test_needs_migration() {
        assert!(!SchemaVersion::V1.needs_migration(SchemaVersion::V1));
        assert!(SchemaVersion::V1.needs_migration(SchemaVersion::V2));
    }

    #[test]
    fn test_migration_support_creation() {
        let support = MigrationSupport::new(SchemaVersion::V1);
        assert_eq!(support.current_version, SchemaVersion::V1);
        assert!(support.is_version_supported(SchemaVersion::V1));
        // V2 is supported as max supported
        assert!(support.is_version_supported(SchemaVersion::V2));
    }

    #[test]
    fn test_check_version_valid() {
        let support = MigrationSupport::new(SchemaVersion::V1);
        let result = support.check_version(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_version_too_new() {
        let support = MigrationSupport::new(SchemaVersion::V1);
        let result = support.check_version(100);
        assert!(matches!(result, Err(MigrationError::SchemaTooNew(100, 2))));
    }

    #[test]
    fn test_check_version_unsupported() {
        let support = MigrationSupport::new(SchemaVersion::V1);
        let result = support.check_version(0);
        assert!(matches!(result, Err(MigrationError::UnsupportedVersion(0))));
    }

    #[test]
    fn test_migrator_creation() {
        let migrator = SchemaMigrator::new(SchemaVersion::V1);
        assert_eq!(migrator.support().current_version, SchemaVersion::V1);
    }

    #[test]
    fn test_validate_version_valid() {
        let migrator = SchemaMigrator::new(SchemaVersion::V1);
        assert!(migrator.validate_version(1).is_ok());
    }

    #[test]
    fn test_validate_version_invalid() {
        let migrator = SchemaMigrator::new(SchemaVersion::V1);
        assert!(migrator.validate_version(0).is_err());
        assert!(migrator.validate_version(100).is_err());
    }

    #[test]
    fn test_migrate_no_migration_needed() {
        let migrator = SchemaMigrator::new(SchemaVersion::V1);
        let data = vec![1, 2, 3];
        let result = migrator.migrate(&data, SchemaVersion::V1, SchemaVersion::V1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_migrate_unavailable_path() {
        let migrator = SchemaMigrator::new(SchemaVersion::V1);
        let data = vec![1, 2, 3];
        let result = migrator.migrate(&data, SchemaVersion::V1, SchemaVersion::V2);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MigrationError::NoMigrationPath(1, 2)));
    }
}