//! Zigmera diagnostic codes and reporting.
//!
//! Diagnostic codes for:
//! - Hook errors (compiler boundary)
//! - Schema parse errors (.zsnap, .zdep, .zairpack)
//! - Graph construction errors
//! - Invalidation engine errors
//! - Cache errors
//! - Lowering errors (Zig → Chimera)
//! - Proof generation errors
//! - CLI errors

use serde::{Deserialize, Serialize};

/// Zigmera diagnostic code categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagCategory {
    Hook,
    Schema,
    Graph,
    Invalidation,
    Cache,
    Lowering,
    Proof,
    Cli,
}

impl std::fmt::Display for DiagCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagCategory::Hook => write!(f, "hook"),
            DiagCategory::Schema => write!(f, "schema"),
            DiagCategory::Graph => write!(f, "graph"),
            DiagCategory::Invalidation => write!(f, "invalidation"),
            DiagCategory::Cache => write!(f, "cache"),
            DiagCategory::Lowering => write!(f, "lowering"),
            DiagCategory::Proof => write!(f, "proof"),
            DiagCategory::Cli => write!(f, "cli"),
        }
    }
}

/// Zigmera diagnostic codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagCode {
    // Hook codes (0xxx)
    HookSnapshotMagicInvalid,
    HookSnapshotVersionMismatch,
    HookSnapshotChecksumMismatch,
    HookSnapshotIncompleteWrite,
    HookSnapshotParseFailed,
    HookCompilerUnavailable,
    HookSnapshotNotFound,

    // Schema codes (1xxx)
    SchemaMagicInvalid,
    SchemaVersionUnsupported,
    SchemaChecksumMismatch,
    SchemaSectionMissing,
    SchemaSectionOrderInvalid,
    SchemaStringTableExhausted,
    SchemaFingerprintDiverges,

    // Graph codes (2xxx)
    GraphNodeNotFound,
    GraphEdgeNotFound,
    GraphCycleDetected,
    GraphDuplicateNode,
    GraphIdMappingFailed,

    // Invalidation codes (3xxx)
    InvalidationReasonUnknown,
    InvalidationPathNotFound,
    InvalidationFingerprintMismatch,
    InvalidationPropagationFailed,

    // Cache codes (4xxx)
    CacheEntryCorrupted,
    CacheEntryNotFound,
    CacheManifestMissing,
    CacheArtifactMissing,
    CacheSizeLimitExceeded,
    CacheEvictionFailed,
    CacheLockFailed,

    // Lowering codes (5xxx)
    LoweringTypeNotSupported,
    LoweringLayoutNotSupported,
    LoweringControlFlowNotSupported,
    LoweringOwnershipNotSupported,
    LoweringEffectNotSupported,
    LoweringAsyncNotSupported,
    LoweringSimdNotSupported,
    LoweringAsmNotSupported,
    LoweringGenericNotSupported,
    LoweringComptimeNotSupported,
    LoweringErrorUnionNotSupported,
    LoweringOptionalNotSupported,
    LoweringSliceNotSupported,
    LoweringWrapperNotSupported,

    // Proof codes (6xxx)
    ProofObligationNotMet,
    ProofFingerprintMismatch,
    ProofCacheInvalid,
    ProofLayoutInvalid,
    ProofOwnershipInvalid,
    ProofEffectInvalid,

    // CLI codes (7xxx)
    CliArgParseFailed,
    CliArtifactNotFound,
    CliDriverFailed,
    CliIntegrationFailed,
}

impl DiagCode {
    /// Get the category for this diagnostic code
    pub fn category(&self) -> DiagCategory {
        match self {
            // Hook codes
            DiagCode::HookSnapshotMagicInvalid
            | DiagCode::HookSnapshotVersionMismatch
            | DiagCode::HookSnapshotChecksumMismatch
            | DiagCode::HookSnapshotIncompleteWrite
            | DiagCode::HookSnapshotParseFailed
            | DiagCode::HookCompilerUnavailable
            | DiagCode::HookSnapshotNotFound => DiagCategory::Hook,

            // Schema codes
            DiagCode::SchemaMagicInvalid
            | DiagCode::SchemaVersionUnsupported
            | DiagCode::SchemaChecksumMismatch
            | DiagCode::SchemaSectionMissing
            | DiagCode::SchemaSectionOrderInvalid
            | DiagCode::SchemaStringTableExhausted
            | DiagCode::SchemaFingerprintDiverges => DiagCategory::Schema,

            // Graph codes
            DiagCode::GraphNodeNotFound
            | DiagCode::GraphEdgeNotFound
            | DiagCode::GraphCycleDetected
            | DiagCode::GraphDuplicateNode
            | DiagCode::GraphIdMappingFailed => DiagCategory::Graph,

            // Invalidation codes
            DiagCode::InvalidationReasonUnknown
            | DiagCode::InvalidationPathNotFound
            | DiagCode::InvalidationFingerprintMismatch
            | DiagCode::InvalidationPropagationFailed => DiagCategory::Invalidation,

            // Cache codes
            DiagCode::CacheEntryCorrupted
            | DiagCode::CacheEntryNotFound
            | DiagCode::CacheManifestMissing
            | DiagCode::CacheArtifactMissing
            | DiagCode::CacheSizeLimitExceeded
            | DiagCode::CacheEvictionFailed
            | DiagCode::CacheLockFailed => DiagCategory::Cache,

            // Lowering codes
            DiagCode::LoweringTypeNotSupported
            | DiagCode::LoweringLayoutNotSupported
            | DiagCode::LoweringControlFlowNotSupported
            | DiagCode::LoweringOwnershipNotSupported
            | DiagCode::LoweringEffectNotSupported
            | DiagCode::LoweringAsyncNotSupported
            | DiagCode::LoweringSimdNotSupported
            | DiagCode::LoweringAsmNotSupported
            | DiagCode::LoweringGenericNotSupported
            | DiagCode::LoweringComptimeNotSupported
            | DiagCode::LoweringErrorUnionNotSupported
            | DiagCode::LoweringOptionalNotSupported
            | DiagCode::LoweringSliceNotSupported
            | DiagCode::LoweringWrapperNotSupported => DiagCategory::Lowering,

            // Proof codes
            DiagCode::ProofObligationNotMet
            | DiagCode::ProofFingerprintMismatch
            | DiagCode::ProofCacheInvalid
            | DiagCode::ProofLayoutInvalid
            | DiagCode::ProofOwnershipInvalid
            | DiagCode::ProofEffectInvalid => DiagCategory::Proof,

            // CLI codes
            DiagCode::CliArgParseFailed
            | DiagCode::CliArtifactNotFound
            | DiagCode::CliDriverFailed
            | DiagCode::CliIntegrationFailed => DiagCategory::Cli,
        }
    }

    /// Get the numeric code for this diagnostic
    pub fn code(&self) -> u32 {
        match self {
            // Hook codes (0xxx)
            DiagCode::HookSnapshotMagicInvalid => 0x0001,
            DiagCode::HookSnapshotVersionMismatch => 0x0002,
            DiagCode::HookSnapshotChecksumMismatch => 0x0003,
            DiagCode::HookSnapshotIncompleteWrite => 0x0004,
            DiagCode::HookSnapshotParseFailed => 0x0005,
            DiagCode::HookCompilerUnavailable => 0x0006,
            DiagCode::HookSnapshotNotFound => 0x0007,

            // Schema codes (1xxx)
            DiagCode::SchemaMagicInvalid => 0x1001,
            DiagCode::SchemaVersionUnsupported => 0x1002,
            DiagCode::SchemaChecksumMismatch => 0x1003,
            DiagCode::SchemaSectionMissing => 0x1004,
            DiagCode::SchemaSectionOrderInvalid => 0x1005,
            DiagCode::SchemaStringTableExhausted => 0x1006,
            DiagCode::SchemaFingerprintDiverges => 0x1007,

            // Graph codes (2xxx)
            DiagCode::GraphNodeNotFound => 0x2001,
            DiagCode::GraphEdgeNotFound => 0x2002,
            DiagCode::GraphCycleDetected => 0x2003,
            DiagCode::GraphDuplicateNode => 0x2004,
            DiagCode::GraphIdMappingFailed => 0x2005,

            // Invalidation codes (3xxx)
            DiagCode::InvalidationReasonUnknown => 0x3001,
            DiagCode::InvalidationPathNotFound => 0x3002,
            DiagCode::InvalidationFingerprintMismatch => 0x3003,
            DiagCode::InvalidationPropagationFailed => 0x3004,

            // Cache codes (4xxx)
            DiagCode::CacheEntryCorrupted => 0x4001,
            DiagCode::CacheEntryNotFound => 0x4002,
            DiagCode::CacheManifestMissing => 0x4003,
            DiagCode::CacheArtifactMissing => 0x4004,
            DiagCode::CacheSizeLimitExceeded => 0x4005,
            DiagCode::CacheEvictionFailed => 0x4006,
            DiagCode::CacheLockFailed => 0x4007,

            // Lowering codes (5xxx)
            DiagCode::LoweringTypeNotSupported => 0x5001,
            DiagCode::LoweringLayoutNotSupported => 0x5002,
            DiagCode::LoweringControlFlowNotSupported => 0x5003,
            DiagCode::LoweringOwnershipNotSupported => 0x5004,
            DiagCode::LoweringEffectNotSupported => 0x5005,
            DiagCode::LoweringAsyncNotSupported => 0x5006,
            DiagCode::LoweringSimdNotSupported => 0x5007,
            DiagCode::LoweringAsmNotSupported => 0x5008,
            DiagCode::LoweringGenericNotSupported => 0x5009,
            DiagCode::LoweringComptimeNotSupported => 0x500A,
            DiagCode::LoweringErrorUnionNotSupported => 0x500B,
            DiagCode::LoweringOptionalNotSupported => 0x500C,
            DiagCode::LoweringSliceNotSupported => 0x500D,
            DiagCode::LoweringWrapperNotSupported => 0x500E,

            // Proof codes (6xxx)
            DiagCode::ProofObligationNotMet => 0x6001,
            DiagCode::ProofFingerprintMismatch => 0x6002,
            DiagCode::ProofCacheInvalid => 0x6003,
            DiagCode::ProofLayoutInvalid => 0x6004,
            DiagCode::ProofOwnershipInvalid => 0x6005,
            DiagCode::ProofEffectInvalid => 0x6006,

            // CLI codes (7xxx)
            DiagCode::CliArgParseFailed => 0x7001,
            DiagCode::CliArtifactNotFound => 0x7002,
            DiagCode::CliDriverFailed => 0x7003,
            DiagCode::CliIntegrationFailed => 0x7004,
        }
    }

    /// Get a human-readable message for this diagnostic code
    pub fn message(&self) -> &'static str {
        match self {
            // Hook messages
            DiagCode::HookSnapshotMagicInvalid => "snapshot magic bytes invalid or missing",
            DiagCode::HookSnapshotVersionMismatch => "snapshot schema version mismatch",
            DiagCode::HookSnapshotChecksumMismatch => "snapshot checksum mismatch",
            DiagCode::HookSnapshotIncompleteWrite => "snapshot write was incomplete",
            DiagCode::HookSnapshotParseFailed => "failed to parse snapshot data",
            DiagCode::HookCompilerUnavailable => "patched Zig compiler is not available",
            DiagCode::HookSnapshotNotFound => "snapshot file not found",

            // Schema messages
            DiagCode::SchemaMagicInvalid => "artifact magic bytes invalid",
            DiagCode::SchemaVersionUnsupported => "artifact schema version not supported",
            DiagCode::SchemaChecksumMismatch => "artifact checksum mismatch",
            DiagCode::SchemaSectionMissing => "required artifact section missing",
            DiagCode::SchemaSectionOrderInvalid => "artifact section order invalid",
            DiagCode::SchemaStringTableExhausted => "string table capacity exhausted",
            DiagCode::SchemaFingerprintDiverges => "fingerprint differs from schema constant",

            // Graph messages
            DiagCode::GraphNodeNotFound => "dependency graph node not found",
            DiagCode::GraphEdgeNotFound => "dependency graph edge not found",
            DiagCode::GraphCycleDetected => "dependency graph cycle detected",
            DiagCode::GraphDuplicateNode => "dependency graph contains duplicate node",
            DiagCode::GraphIdMappingFailed => "failed to map compiler ID to graph ID",

            // Invalidation messages
            DiagCode::InvalidationReasonUnknown => "unknown invalidation reason",
            DiagCode::InvalidationPathNotFound => "invalidation path not found in graph",
            DiagCode::InvalidationFingerprintMismatch => "fingerprint mismatch during invalidation",
            DiagCode::InvalidationPropagationFailed => "invalidation propagation failed",

            // Cache messages
            DiagCode::CacheEntryCorrupted => "cache entry is corrupted",
            DiagCode::CacheEntryNotFound => "cache entry not found",
            DiagCode::CacheManifestMissing => "cache manifest is missing",
            DiagCode::CacheArtifactMissing => "cache artifact file is missing",
            DiagCode::CacheSizeLimitExceeded => "cache size limit exceeded",
            DiagCode::CacheEvictionFailed => "cache eviction failed",
            DiagCode::CacheLockFailed => "failed to acquire cache lock",

            // Lowering messages
            DiagCode::LoweringTypeNotSupported => "type lowering not supported",
            DiagCode::LoweringLayoutNotSupported => "layout lowering not supported",
            DiagCode::LoweringControlFlowNotSupported => "control flow lowering not supported",
            DiagCode::LoweringOwnershipNotSupported => "ownership lowering not supported",
            DiagCode::LoweringEffectNotSupported => "effect lowering not supported",
            DiagCode::LoweringAsyncNotSupported => "async/frame lowering not supported",
            DiagCode::LoweringSimdNotSupported => "SIMD/vector lowering not supported",
            DiagCode::LoweringAsmNotSupported => "inline assembly lowering not supported",
            DiagCode::LoweringGenericNotSupported => "generic lowering not supported",
            DiagCode::LoweringComptimeNotSupported => "comptime lowering not supported",
            DiagCode::LoweringErrorUnionNotSupported => "error union lowering not supported",
            DiagCode::LoweringOptionalNotSupported => "optional lowering not supported",
            DiagCode::LoweringSliceNotSupported => "slice lowering not supported",
            DiagCode::LoweringWrapperNotSupported => "wrapper generation not supported",

            // Proof messages
            DiagCode::ProofObligationNotMet => "proof obligation not met",
            DiagCode::ProofFingerprintMismatch => "fingerprint proof failed",
            DiagCode::ProofCacheInvalid => "cache validity proof failed",
            DiagCode::ProofLayoutInvalid => "layout preservation proof failed",
            DiagCode::ProofOwnershipInvalid => "ownership proof failed",
            DiagCode::ProofEffectInvalid => "effect proof failed",

            // CLI messages
            DiagCode::CliArgParseFailed => "CLI argument parsing failed",
            DiagCode::CliArtifactNotFound => "required artifact not found",
            DiagCode::CliDriverFailed => "compiler driver command failed",
            DiagCode::CliIntegrationFailed => "integration test failed",
        }
    }
}

/// Diagnostic with location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diag {
    pub code: DiagCode,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub notes: Vec<String>,
}

impl Diag {
    /// Create a new diagnostic
    pub fn new(code: DiagCode) -> Self {
        Self {
            code,
            message: code.message().to_string(),
            file: None,
            line: None,
            column: None,
            notes: Vec::new(),
        }
    }

    /// Add a note to this diagnostic
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Set the location for this diagnostic
    pub fn at(mut self, file: impl Into<String>, line: u32, column: u32) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self.column = Some(column);
        self
    }
}

impl std::fmt::Display for Diag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.code, self.message)?;
        if let (Some(file), Some(line), Some(col)) = (&self.file, self.line, self.column) {
            write!(f, " at {}:{}:{}", file, line, col)?;
        }
        for note in &self.notes {
            write!(f, "\nnote: {}", note)?;
        }
        Ok(())
    }
}

/// Diagnostic bag for collecting multiple diagnostics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagBag {
    diags: Vec<Diag>,
}

impl DiagBag {
    /// Create a new diagnostic bag
    pub fn new() -> Self {
        Self { diags: Vec::new() }
    }

    /// Add a diagnostic to the bag
    pub fn push(&mut self, diag: Diag) {
        self.diags.push(diag);
    }

    /// Add an error with a code
    pub fn error(&mut self, code: DiagCode, msg: &str) -> &mut Diag {
        let mut diag = Diag::new(code);
        diag.message = msg.to_string();
        self.diags.push(diag);
        self.diags.last_mut().unwrap()
    }

    /// Add a warning
    #[allow(dead_code)]
    pub fn warning(&mut self, code: DiagCode, msg: &str) -> &mut Diag {
        let mut diag = Diag::new(code);
        diag.message = msg.to_string();
        self.diags.push(diag);
        self.diags.last_mut().unwrap()
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.diags.is_empty()
    }

    /// Get all diagnostics
    pub fn diags(&self) -> &[Diag] {
        &self.diags
    }

    /// Get diagnostics as JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.diags).unwrap_or_else(|_| "[]".to_string())
    }

    /// Clear all diagnostics
    pub fn clear(&mut self) {
        self.diags.clear();
    }

    /// Get the number of diagnostics
    pub fn len(&self) -> usize {
        self.diags.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.diags.is_empty()
    }
}

impl std::fmt::Display for DiagBag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for diag in &self.diags {
            writeln!(f, "{}", diag)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diag_code_category() {
        assert_eq!(
            DiagCode::HookSnapshotMagicInvalid.category(),
            DiagCategory::Hook
        );
        assert_eq!(
            DiagCode::SchemaMagicInvalid.category(),
            DiagCategory::Schema
        );
        assert_eq!(DiagCode::GraphNodeNotFound.category(), DiagCategory::Graph);
        assert_eq!(
            DiagCode::InvalidationReasonUnknown.category(),
            DiagCategory::Invalidation
        );
        assert_eq!(
            DiagCode::CacheEntryCorrupted.category(),
            DiagCategory::Cache
        );
        assert_eq!(
            DiagCode::LoweringTypeNotSupported.category(),
            DiagCategory::Lowering
        );
        assert_eq!(
            DiagCode::ProofObligationNotMet.category(),
            DiagCategory::Proof
        );
        assert_eq!(DiagCode::CliArgParseFailed.category(), DiagCategory::Cli);
    }

    #[test]
    fn test_diag_code_message() {
        assert_eq!(
            DiagCode::HookSnapshotMagicInvalid.message(),
            "snapshot magic bytes invalid or missing"
        );
        assert_eq!(
            DiagCode::LoweringAsyncNotSupported.message(),
            "async/frame lowering not supported"
        );
    }

    #[test]
    fn test_diag_creation() {
        let diag = Diag::new(DiagCode::LoweringTypeNotSupported)
            .at("test.zig", 10, 5)
            .with_note("consider using a wrapper type");
        assert_eq!(diag.code, DiagCode::LoweringTypeNotSupported);
        assert!(diag.message.contains("type lowering"));
        assert_eq!(diag.file.unwrap(), "test.zig");
        assert_eq!(diag.line.unwrap(), 10);
        assert_eq!(diag.column.unwrap(), 5);
        assert_eq!(diag.notes.len(), 1);
    }

    #[test]
    fn test_diag_bag() {
        let mut bag = DiagBag::new();
        bag.error(
            DiagCode::LoweringAsyncNotSupported,
            "async is not supported",
        );
        bag.error(DiagCode::LoweringSimdNotSupported, "SIMD is not supported");
        assert!(bag.has_errors());
        assert_eq!(bag.len(), 2);
    }

    #[test]
    fn test_diag_json() {
        let mut bag = DiagBag::new();
        bag.error(DiagCode::HookSnapshotNotFound, "file missing");
        let json = bag.to_json();
        assert!(json.contains("hook_snapshot_not_found"));
        assert!(json.contains("file missing"));
    }

    #[test]
    fn test_diag_display() {
        let diag = Diag::new(DiagCode::SchemaMagicInvalid);
        let s = format!("{}", diag);
        assert!(s.contains("SchemaMagicInvalid"));
    }
}
