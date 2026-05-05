//! Schema definitions for BEAM adapter artifacts.
//!
//! Defines schemas for:
//! - `.beam_snap` — BEAM semantic snapshot
//! - `.beam_dep` — BEAM dependency graph
//! - `.beam_pack` — BEAM package

pub mod beam_dep;
pub mod beam_pack;
pub mod beam_snap;
pub mod versioning;

pub use beam_dep::{
    BeamDepEdge, BeamDepHeader, BeamDepNode, BeamDepSchema, DepKind, BEAM_DEP_MAGIC,
};
pub use beam_pack::{
    BeamChecksum, BeamPackHeader, BeamPackModule, BeamPackSchema, BEAM_PACK_MAGIC,
};
pub use beam_snap::{
    validate_binary, Atom, Attribute, BeamCodeHash, BeamEffectInfo, BeamModuleInfo,
    BeamProcessInfo, BeamRegistryInfo, BeamSnapHeader, BeamSnapSchema, BeamSupervisorInfo,
    BinaryParseError, BinaryParseResult, BinaryParser, CompileInfo, ExportEntry, FunctionInfo,
    FunctionRef, ImportEntry, MonitorRef, MonitorTarget, Priority, ProcessFlags, ProcessId, Term,
    BEAM_SNAP_MAGIC, SCHEMA_VERSION,
};
pub use versioning::{
    VersionChecker, VersionError, CURRENT_SCHEMA_VERSION, MAX_SUPPORTED_VERSION,
    MIN_SUPPORTED_VERSION,
};
