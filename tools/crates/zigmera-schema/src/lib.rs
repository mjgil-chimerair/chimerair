//! Schema definitions for Zigmera artifacts.
//!
//! Defines binary and JSON schemas for:
//! - `.zsnap` — semantic snapshot
//! - `.zdep` — dependency graph
//! - `.zairpack` — AIR type/layout/function bundle
//! - `.zchmeta` — Chimera metadata
//! - `.zchproof` — proof obligations

pub mod corruption;
pub mod manifest;
pub mod version;
pub mod versioning;
pub mod zairpack;
pub mod zchmeta;
pub mod zchproof;
pub mod zdep;
pub mod zsnap;

pub use corruption::{
    Artifact, ArtifactHeader, CorruptionDetector, CorruptionError, CorruptionScanResult, Section,
    SectionTable,
};
pub use manifest::{ArtifactEntry, ArtifactKind, ArtifactManifest};
pub use version::SchemaVersion;
pub use versioning::{
    ArtifactValidator, CompatibilityStatus, SchemaHeader, VersionChecker, VersionCompatibility,
    VersionError, CURRENT_SCHEMA_VERSION, MAX_SUPPORTED_VERSION, MIN_SUPPORTED_VERSION,
};
pub use zairpack::{
    AirBlock, AirFunction, AirInst, AirOp, AirpackHeader, AirpackSchema, ZAIRPACK_MAGIC,
};
pub use zchmeta::{ChmetaHeader, ChmetaSchema, SemanticSignature, ZCHMETA_MAGIC};
pub use zchproof::{ChproofHeader, ChproofSchema, ProofKind, ProofObligation, ZCHPROOF_MAGIC};
pub use zdep::{DepHeader, DepSchema, Edge, EdgeDirection, EdgeKind, Node, NodeKind, ZDEP_MAGIC};
pub use zsnap::{
    validate_binary, BinaryParseError, BinaryParseResult, BinaryParser, BuildOptions, SnapHeader,
    SnapSchema, SourceFile, ValidationReport, SCHEMA_VERSION, ZSNAP_MAGIC,
};
