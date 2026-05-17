//! `.beam_snap` BEAM semantic snapshot schema v1.
//!
//! Header plus modules, processes, supervisors, registries, effects,
//! and dependency edge references from BEAM compilation.

use serde::{Deserialize, Serialize};

/// Magic bytes for `.beam_snap` binary format.
pub const BEAM_SNAP_MAGIC: &[u8; 8] = b"BeamSnap";

/// Current schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// `.beam_snap` semantic snapshot header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamSnapHeader {
    pub magic: [u8; 8],
    pub schema_version: u32,
    pub min_adapter_version: u32,
    pub erlang_version: String,
    pub otp_release: String,
    pub target: String,
    pub timestamp_ns: u64,
    pub module_count: u32,
    pub process_count: u32,
    pub supervisor_count: u32,
    pub registry_count: u32,
    pub effect_count: u32,
    pub checksum: [u8; 32],
}

/// Module information extracted from BEAM compilation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamModuleInfo {
    pub module_name: Atom,
    pub exports: Vec<ExportEntry>,
    pub imports: Vec<ImportEntry>,
    pub functions: Vec<FunctionInfo>,
    pub attributes: Vec<Attribute>,
    pub compile_info: CompileInfo,
}

/// Atom table entry (interned string).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Atom(pub String);

impl Atom {
    pub fn new(s: impl Into<String>) -> Self {
        Atom(s.into())
    }
}

/// Export entry for a function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEntry {
    pub function: Atom,
    pub arity: u32,
    pub label: u32,
}

/// Import entry for an external function call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportEntry {
    pub module: Atom,
    pub function: Atom,
    pub arity: u32,
}

/// Function information including Arity and body reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: Atom,
    pub arity: u32,
    pub label: u32,
    pub code_version: u32,
    pub num_args: u32,
    pub num_locals: u32,
}

/// Module attribute (compiler metadata).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub key: Atom,
    pub value: Term,
}

/// Compilation information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileInfo {
    pub options: Vec<Term>,
    pub version: Option<String>,
    pub time: Option<String>,
}

/// Process information for a spawned BEAM process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamProcessInfo {
    pub pid: ProcessId,
    pub initial_function: FunctionRef,
    pub parent: Option<ProcessId>,
    pub links: Vec<ProcessId>,
    pub monitors: Vec<MonitorRef>,
    pub flags: ProcessFlags,
    pub heap_size: u32,
    pub stack_size: u32,
}

/// Process identifier (unique within node).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ProcessId(pub u64);

impl ProcessId {
    pub fn new(id: u64) -> Self {
        ProcessId(id)
    }
}

/// Function reference (module, function, arity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionRef {
    pub module: Atom,
    pub function: Atom,
    pub arity: u32,
}

/// Monitor reference for process monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorRef {
    pub ref_id: u64,
    pub target: MonitorTarget,
}

/// Monitor target (either a pid or a registered name).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitorTarget {
    Pid(ProcessId),
    Name(Atom),
}

/// Process flags and settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessFlags {
    pub traps_exit: bool,
    pub priority: Priority,
    pub message_queue_len: u32,
}

/// Process priority levels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Max,
    High,
    Normal,
    Low,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// Supervisor information for a supervision tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamSupervisorInfo {
    pub supervisor_name: Atom,
    pub strategy: RestartStrategy,
    pub intensity: u32,
    pub period: u32,
    pub children: Vec<ChildSpec>,
    pub restart_count: u32,
}

/// Restart strategy for a supervisor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RestartStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
    SimpleOneForOne,
}

/// Child specification for a supervised process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildSpec {
    pub id: ChildId,
    pub start: FunctionRef,
    pub restart: Restart,
    pub shutdown: ShutdownTimeout,
    pub child_type: ChildType,
    pub modules: Vec<Atom>,
}

/// Child identifier (usually an atom).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildId(pub Atom);

/// Restart policy for a child.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Restart {
    Permanent,
    Temporary,
    Transient,
}

/// Shutdown timeout value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShutdownTimeout {
    Int(u32),
    BrutalKill,
    Infinity,
}

/// Child type (worker or supervisor).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChildType {
    Worker,
    Supervisor,
}

/// Registry information for named processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamRegistryInfo {
    pub name: Atom,
    pub entries: Vec<(Atom, ProcessId)>,
    pub protected: bool,
}

/// Effect information for effect inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamEffectInfo {
    pub effect_type: EffectType,
    pub location: EffectLocation,
    pub severity: EffectSeverity,
}

/// Type of effect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffectType {
    MaySpawn,
    MayMessage,
    MayReceive,
    MaySchedule,
    MayLink,
    MayExit,
    MayReloadCode,
    MayRegister,
    MayNif,
    MayDistribute,
}

/// Location of the effect in source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectLocation {
    pub module: Atom,
    pub function: Atom,
    pub line: u32,
}

/// Severity level of the effect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffectSeverity {
    Pure,
    IO,
    SideEffect,
}

/// Code hash for version tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamCodeHash {
    pub module: Atom,
    pub md5: [u8; 16],
    pub size: u32,
}

/// Term representation for Erlang terms (attributes, etc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Term {
    Atom(Atom),
    Int(i64),
    Float(f64),
    Tuple(Vec<Term>),
    List(Vec<Term>),
    Binary(Vec<u8>),
    String(String),
}

impl Term {
    pub fn atom(s: impl Into<String>) -> Self {
        Term::Atom(Atom::new(s))
    }

    pub fn int(i: i64) -> Self {
        Term::Int(i)
    }
}

/// Binary parser for `.beam_snap` files.
pub struct BinaryParser;

impl BinaryParser {
    pub fn parse(_data: &[u8]) -> Result<BeamSnapSchema, BinaryParseError> {
        Err(BinaryParseError::Parse(
            "BEAM snapshot binary parser is not implemented".to_string(),
        ))
    }

    pub fn parse_json(_json: &str) -> Result<BeamSnapSchema, BinaryParseError> {
        Err(BinaryParseError::Parse(
            "BEAM snapshot JSON parser is not implemented".to_string(),
        ))
    }
}

/// Binary parse error.
#[derive(Debug, thiserror::Error)]
pub enum BinaryParseError {
    #[error("invalid magic bytes")]
    InvalidMagic,
    #[error("unsupported schema version: {0}")]
    UnsupportedVersion(u32),
    #[error("truncated data: {0}")]
    TruncatedData(String),
    #[error("checksum mismatch")]
    ChecksumMismatch,
    #[error("parse error: {0}")]
    Parse(String),
}

/// Result type for binary parsing.
pub type BinaryParseResult<T> = Result<T, BinaryParseError>;

/// Full BEAM semantic snapshot schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamSnapSchema {
    pub header: BeamSnapHeader,
    pub modules: Vec<BeamModuleInfo>,
    pub processes: Vec<BeamProcessInfo>,
    pub supervisors: Vec<BeamSupervisorInfo>,
    pub registries: Vec<BeamRegistryInfo>,
    pub effects: Vec<BeamEffectInfo>,
    pub code_hashes: Vec<BeamCodeHash>,
}

impl BeamSnapSchema {
    pub fn new() -> Self {
        BeamSnapSchema {
            header: BeamSnapHeader {
                magic: *BEAM_SNAP_MAGIC,
                schema_version: SCHEMA_VERSION,
                min_adapter_version: 1,
                erlang_version: String::new(),
                otp_release: String::new(),
                target: String::new(),
                timestamp_ns: 0,
                module_count: 0,
                process_count: 0,
                supervisor_count: 0,
                registry_count: 0,
                effect_count: 0,
                checksum: [0u8; 32],
            },
            modules: Vec::new(),
            processes: Vec::new(),
            supervisors: Vec::new(),
            registries: Vec::new(),
            effects: Vec::new(),
            code_hashes: Vec::new(),
        }
    }
}

impl Default for BeamSnapSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a binary `.beam_snap` file.
pub fn validate_binary(_data: &[u8]) -> Result<ValidationReport, BinaryParseError> {
    Err(BinaryParseError::Parse(
        "BEAM snapshot binary validation is not implemented".to_string(),
    ))
}

/// Validation report for a `.beam_snap` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beam_snap_schema_new() {
        let schema = BeamSnapSchema::new();
        assert_eq!(schema.header.schema_version, SCHEMA_VERSION);
        assert_eq!(schema.header.magic, *BEAM_SNAP_MAGIC);
    }

    #[test]
    fn test_atom_creation() {
        let atom = Atom::new("test_module");
        assert_eq!(atom.0, "test_module");
    }

    #[test]
    fn test_process_id_creation() {
        let pid = ProcessId::new(12345);
        assert_eq!(pid.0, 12345);
    }

    #[test]
    fn test_term_constructors() {
        assert!(matches!(Term::atom("hello"), Term::Atom(_)));
        assert!(matches!(Term::int(42), Term::Int(42)));
    }

    #[test]
    fn test_restart_strategy_serialization() {
        let strategy = RestartStrategy::OneForOne;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"OneForOne\"");
    }

    #[test]
    fn test_priority_default() {
        let priority = Priority::default();
        assert_eq!(priority, Priority::Normal);
    }
}
