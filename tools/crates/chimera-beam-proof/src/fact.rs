//! Proof fact types for BEAM boundaries.
//!
//! Represents individual facts that can be proven about BEAM constructs
//! including memory safety, ownership validity, and message protocol compliance.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a proof fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactId {
    /// Local fact index.
    pub index: u32,
    /// Module fingerprint.
    pub module_fingerprint: u32,
}

impl FactId {
    /// Create a new fact ID.
    pub fn new(index: u32, module_fingerprint: u32) -> Self {
        FactId {
            index,
            module_fingerprint,
        }
    }

    /// Get the fact string representation.
    pub fn as_str(&self) -> String {
        format!("F{}.{:08x}", self.index, self.module_fingerprint)
    }
}

impl fmt::Display for FactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Kind of proof fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofKind {
    /// Memory safety: heap allocation is valid.
    MemorySafety,
    /// Ownership: value is properly owned.
    OwnershipValid,
    /// Message encode: term can be binary-encoded.
    MessageEncode,
    /// Message decode: binary can be term-decoded.
    MessageDecode,
    /// Process spawn: spawn parameters are valid.
    ProcessSpawn,
    /// Process exit: exit reason is valid.
    ProcessExit,
    /// Link validity: link between processes is valid.
    LinkValid,
    /// Monitor validity: monitor reference is valid.
    MonitorValid,
    /// Registry entry: registration is valid.
    RegistryEntry,
    /// Supervisor spec: supervisor specification is valid.
    SupervisorSpec,
    /// Boundary contract: cross-language boundary is safe.
    BoundaryContract,
    /// Effect safety: effect does not violate guarantees.
    EffectSafety,
    /// Timer validity: timer operation is valid.
    TimerValid,
    /// Code load safety: code loading is safe.
    CodeLoadSafety,
}

impl ProofKind {
    /// Get the fact kind name.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProofKind::MemorySafety => "memory_safety",
            ProofKind::OwnershipValid => "ownership_valid",
            ProofKind::MessageEncode => "message_encode",
            ProofKind::MessageDecode => "message_decode",
            ProofKind::ProcessSpawn => "process_spawn",
            ProofKind::ProcessExit => "process_exit",
            ProofKind::LinkValid => "link_valid",
            ProofKind::MonitorValid => "monitor_valid",
            ProofKind::RegistryEntry => "registry_entry",
            ProofKind::SupervisorSpec => "supervisor_spec",
            ProofKind::BoundaryContract => "boundary_contract",
            ProofKind::EffectSafety => "effect_safety",
            ProofKind::TimerValid => "timer_valid",
            ProofKind::CodeLoadSafety => "code_load_safety",
        }
    }

    /// Get the severity level of this fact kind.
    pub fn severity_level(&self) -> u8 {
        match self {
            ProofKind::BoundaryContract | ProofKind::EffectSafety => 3,
            ProofKind::ProcessSpawn | ProofKind::ProcessExit => 2,
            _ => 1,
        }
    }
}

impl fmt::Display for ProofKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Target of a proof fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofTarget {
    /// Fact applies to an entire module.
    Module(String),
    /// Fact applies to a specific function.
    Function {
        module: String,
        function: String,
        arity: u8,
    },
    /// Fact applies to a specific process.
    Process(String),
    /// Fact applies to a specific message.
    Message(String),
    /// Fact applies across a boundary.
    Boundary {
        from_module: String,
        to_module: String,
    },
}

impl ProofTarget {
    /// Get the target name.
    pub fn name(&self) -> String {
        match self {
            ProofTarget::Module(m) => m.clone(),
            ProofTarget::Function {
                module,
                function,
                arity,
            } => {
                format!("{}/{}/{}", module, function, arity)
            }
            ProofTarget::Process(p) => p.clone(),
            ProofTarget::Message(m) => m.clone(),
            ProofTarget::Boundary {
                from_module,
                to_module,
            } => {
                format!("{}->{}", from_module, to_module)
            }
        }
    }

    /// Check if target is a module.
    pub fn is_module(&self) -> bool {
        matches!(self, ProofTarget::Module(_))
    }

    /// Check if target is a function.
    pub fn is_function(&self) -> bool {
        matches!(self, ProofTarget::Function { .. })
    }

    /// Check if target is a boundary.
    pub fn is_boundary(&self) -> bool {
        matches!(self, ProofTarget::Boundary { .. })
    }
}

impl fmt::Display for ProofTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A single proof fact with evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofFact {
    /// Unique fact identifier.
    pub id: FactId,
    /// Kind of fact.
    pub kind: ProofKind,
    /// Target this fact applies to.
    pub target: ProofTarget,
    /// Human-readable claim.
    pub claim: String,
    /// Evidence supporting the claim.
    pub evidence: Vec<EvidenceItem>,
    /// Whether this fact is proven or assumed.
    pub is_proven: bool,
    /// Optional source location.
    pub source_loc: Option<SourceLocation>,
}

/// Source location for a fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path.
    pub file: String,
    /// Line number.
    pub line: u32,
    /// Column number.
    pub col: u32,
}

impl SourceLocation {
    /// Create a new source location.
    pub fn new(file: impl Into<String>, line: u32, col: u32) -> Self {
        SourceLocation {
            file: file.into(),
            line,
            col,
        }
    }

    /// Format as a string.
    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.file, self.line, self.col)
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Evidence item supporting a proof fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceItem {
    /// Type evidence.
    Type(String),
    /// Ownership evidence.
    Ownership(String),
    /// Effect evidence.
    Effect(String),
    /// Size evidence.
    Size(u64),
    /// Depth evidence for receive.
    ReceiveDepth(u32),
    /// Arity evidence.
    Arity(u8),
    /// Bytecode hash.
    BytecodeHash(String),
    /// Module hash.
    ModuleHash(String),
    /// Description note.
    Note(String),
}

impl EvidenceItem {
    /// Get the evidence as a string.
    pub fn as_str(&self) -> &str {
        match self {
            EvidenceItem::Type(s) => s,
            EvidenceItem::Ownership(s) => s,
            EvidenceItem::Effect(s) => s,
            EvidenceItem::Note(s) => s,
            _ => "",
        }
    }
}

impl ProofFact {
    /// Create a new proof fact.
    pub fn new(id: FactId, kind: ProofKind, target: ProofTarget, claim: impl Into<String>) -> Self {
        ProofFact {
            id,
            kind,
            target,
            claim: claim.into(),
            evidence: Vec::new(),
            is_proven: false,
            source_loc: None,
        }
    }

    /// Mark the fact as proven.
    pub fn mark_proven(mut self) -> Self {
        self.is_proven = true;
        self
    }

    /// Add an evidence item.
    pub fn add_evidence(mut self, evidence: EvidenceItem) -> Self {
        self.evidence.push(evidence);
        self
    }

    /// Add type evidence.
    pub fn add_type_evidence(mut self, type_str: impl Into<String>) -> Self {
        self.evidence.push(EvidenceItem::Type(type_str.into()));
        self
    }

    /// Add ownership evidence.
    pub fn add_ownership_evidence(mut self, ownership: impl Into<String>) -> Self {
        self.evidence
            .push(EvidenceItem::Ownership(ownership.into()));
        self
    }

    /// Add effect evidence.
    pub fn add_effect_evidence(mut self, effect: impl Into<String>) -> Self {
        self.evidence.push(EvidenceItem::Effect(effect.into()));
        self
    }

    /// Add a note.
    pub fn add_note(mut self, note: impl Into<String>) -> Self {
        self.evidence.push(EvidenceItem::Note(note.into()));
        self
    }

    /// Set source location.
    pub fn at_location(mut self, loc: SourceLocation) -> Self {
        self.source_loc = Some(loc);
        self
    }

    /// Check if this fact is empty (no evidence).
    pub fn is_empty(&self) -> bool {
        self.evidence.is_empty()
    }

    /// Get the number of evidence items.
    pub fn evidence_count(&self) -> usize {
        self.evidence.len()
    }

    /// Verify fact consistency.
    pub fn verify(&self) -> bool {
        if !self.is_proven && self.evidence.is_empty() {
            return false;
        }

        if self.claim.is_empty() {
            return false;
        }

        if let Some(loc) = &self.source_loc {
            if loc.line == 0 || loc.col == 0 {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_id_new() {
        let id = FactId::new(1, 0xABCD);
        assert_eq!(id.index, 1);
        assert_eq!(id.module_fingerprint, 0xABCD);
    }

    #[test]
    fn test_fact_id_as_str() {
        let id = FactId::new(42, 0x1234);
        assert_eq!(id.as_str(), "F42.00001234");
    }

    #[test]
    fn test_proof_kind_as_str() {
        assert_eq!(ProofKind::MemorySafety.as_str(), "memory_safety");
        assert_eq!(ProofKind::OwnershipValid.as_str(), "ownership_valid");
        assert_eq!(ProofKind::MessageEncode.as_str(), "message_encode");
    }

    #[test]
    fn test_proof_kind_severity() {
        assert_eq!(ProofKind::BoundaryContract.severity_level(), 3);
        assert_eq!(ProofKind::ProcessSpawn.severity_level(), 2);
        assert_eq!(ProofKind::MessageEncode.severity_level(), 1);
    }

    #[test]
    fn test_proof_target_name() {
        let module_target = ProofTarget::Module("test_mod".to_string());
        assert_eq!(module_target.name(), "test_mod");

        let func_target = ProofTarget::Function {
            module: "mod".to_string(),
            function: "fun".to_string(),
            arity: 2,
        };
        assert_eq!(func_target.name(), "mod/fun/2");
    }

    #[test]
    fn test_proof_target_is_module() {
        assert!(ProofTarget::Module("mod".to_string()).is_module());
        assert!(!ProofTarget::Function {
            module: "mod".to_string(),
            function: "fun".to_string(),
            arity: 0
        }
        .is_module());
    }

    #[test]
    fn test_proof_target_is_boundary() {
        assert!(ProofTarget::Boundary {
            from_module: "mod1".to_string(),
            to_module: "mod2".to_string()
        }
        .is_boundary());
        assert!(!ProofTarget::Module("mod".to_string()).is_boundary());
    }

    #[test]
    fn test_proof_fact_new() {
        let id = FactId::new(1, 0x1234);
        let fact = ProofFact::new(
            id,
            ProofKind::MemorySafety,
            ProofTarget::Module("test_mod".to_string()),
            "Heap allocation is within bounds",
        );

        assert_eq!(fact.id, id);
        assert_eq!(fact.kind, ProofKind::MemorySafety);
        assert!(!fact.is_proven);
        assert!(fact.evidence.is_empty());
    }

    #[test]
    fn test_proof_fact_add_evidence() {
        let id = FactId::new(1, 0x1234);
        let fact = ProofFact::new(
            id,
            ProofKind::MessageEncode,
            ProofTarget::Function {
                module: "mod".to_string(),
                function: "enc".to_string(),
                arity: 1,
            },
            "Message can be encoded to binary",
        )
        .add_type_evidence("binary")
        .add_evidence(EvidenceItem::Size(64));

        assert_eq!(fact.evidence_count(), 2);
    }

    #[test]
    fn test_proof_fact_verify_valid() {
        let fact = ProofFact::new(
            FactId::new(1, 0x1234),
            ProofKind::MemorySafety,
            ProofTarget::Module("test".to_string()),
            "Valid claim",
        )
        .add_type_evidence("valid_type")
        .mark_proven();

        assert!(fact.verify());
    }

    #[test]
    fn test_proof_fact_verify_empty_claim() {
        let fact = ProofFact::new(
            FactId::new(1, 0x1234),
            ProofKind::MemorySafety,
            ProofTarget::Module("test".to_string()),
            "",
        )
        .add_type_evidence("type")
        .mark_proven();

        assert!(!fact.verify());
    }

    #[test]
    fn test_proof_fact_mark_proven() {
        let fact = ProofFact::new(
            FactId::new(1, 0x1234),
            ProofKind::ProcessSpawn,
            ProofTarget::Module("test".to_string()),
            "Process spawn parameters are valid",
        );

        assert!(!fact.is_proven);

        let proven = fact.mark_proven();
        assert!(proven.is_proven);
    }

    #[test]
    fn test_source_location() {
        let loc = SourceLocation::new("test.erl", 10, 5);
        assert_eq!(loc.file, "test.erl");
        assert_eq!(loc.line, 10);
        assert_eq!(loc.col, 5);
    }

    #[test]
    fn test_source_location_to_string() {
        let loc = SourceLocation::new("test.erl", 10, 5);
        assert_eq!(loc.to_string(), "test.erl:10:5");
    }
}
