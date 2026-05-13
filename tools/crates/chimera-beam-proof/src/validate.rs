//! Proof validation for BEAM facts.
//!
//! Validates proof facts against BEAM semantics ensuring
//! memory safety, ownership invariants, and protocol compliance.

use serde::{Deserialize, Serialize};

use super::fact::{ProofFact, ProofKind, ProofTarget};

/// Input for proof validation.
#[derive(Debug, Clone)]
pub struct ValidationInput<'a> {
    /// The fact to validate.
    pub fact: &'a ProofFact,
    /// Optional context (module name).
    pub context: Option<&'a str>,
}

/// Result of proof validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofResult {
    /// Fact is valid.
    Valid,
    /// Fact has a warning (not an error).
    Warning(String),
    /// Fact is invalid with reason.
    Invalid(String),
    /// Fact could not be validated (insufficient info).
    Unknown(String),
}

impl ProofResult {
    /// Check if result indicates validity.
    pub fn is_valid(&self) -> bool {
        matches!(self, ProofResult::Valid)
    }

    /// Check if result indicates a warning.
    pub fn is_warning(&self) -> bool {
        matches!(self, ProofResult::Warning(_))
    }

    /// Check if result indicates invalidity.
    pub fn is_invalid(&self) -> bool {
        matches!(self, ProofResult::Invalid(_))
    }

    /// Get the reason if any.
    pub fn reason(&self) -> Option<&str> {
        match self {
            ProofResult::Valid => None,
            ProofResult::Warning(s) => Some(s),
            ProofResult::Invalid(s) => Some(s),
            ProofResult::Unknown(s) => Some(s),
        }
    }
}

/// Proof validator for BEAM facts.
#[derive(Debug, Clone)]
pub struct ProofValidator {
    /// Enable strict validation.
    strict: bool,
    /// Require evidence for all facts.
    require_evidence: bool,
    /// Require source location.
    require_source_loc: bool,
    /// Maximum fact count per module.
    max_facts_per_module: usize,
}

impl ProofValidator {
    /// Create a new proof validator.
    pub fn new() -> Self {
        ProofValidator {
            strict: false,
            require_evidence: true,
            require_source_loc: false,
            max_facts_per_module: 65536,
        }
    }

    /// Enable strict mode.
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Enable evidence requirement.
    pub fn require_evidence(mut self) -> Self {
        self.require_evidence = true;
        self
    }

    /// Disable evidence requirement.
    pub fn no_evidence(mut self) -> Self {
        self.require_evidence = false;
        self
    }

    /// Enable source location requirement.
    pub fn require_source_location(mut self) -> Self {
        self.require_source_loc = true;
        self
    }

    /// Set maximum facts per module.
    pub fn max_facts_per_module(mut self, max: usize) -> Self {
        self.max_facts_per_module = max;
        self
    }

    /// Validate a proof fact.
    pub fn validate(&self, input: &ValidationInput<'_>) -> bool {
        let fact = input.fact;

        // Check basic fact validity
        if !fact.verify() {
            return false;
        }

        // Validate based on fact kind
        let result = match fact.kind {
            ProofKind::MemorySafety => self.validate_memory_safety(fact),
            ProofKind::OwnershipValid => self.validate_ownership_valid(fact),
            ProofKind::MessageEncode => self.validate_message_encode(fact),
            ProofKind::MessageDecode => self.validate_message_decode(fact),
            ProofKind::ProcessSpawn => self.validate_process_spawn(fact),
            ProofKind::ProcessExit => self.validate_process_exit(fact),
            ProofKind::LinkValid => self.validate_link_valid(fact),
            ProofKind::MonitorValid => self.validate_monitor_valid(fact),
            ProofKind::RegistryEntry => self.validate_registry_entry(fact),
            ProofKind::SupervisorSpec => self.validate_supervisor_spec(fact),
            ProofKind::BoundaryContract => self.validate_boundary_contract(fact),
            ProofKind::EffectSafety => self.validate_effect_safety(fact),
            ProofKind::TimerValid => self.validate_timer_valid(fact),
            ProofKind::CodeLoadSafety => self.validate_code_load_safety(fact),
        };

        // Additional checks in strict mode
        if self.strict && result.is_valid() {
            self.validate_strict_checks(fact)
        } else {
            result.is_valid()
        }
    }

    fn validate_memory_safety(&self, fact: &ProofFact) -> ProofResult {
        // Memory safety facts must have size evidence
        let has_size = fact
            .evidence
            .iter()
            .any(|e| matches!(e, super::fact::EvidenceItem::Size(_)));
        if self.require_evidence && !has_size {
            return ProofResult::Invalid("Memory safety fact requires size evidence".to_string());
        }

        // Must target a module or function
        match &fact.target {
            ProofTarget::Module(_) | ProofTarget::Function { .. } => ProofResult::Valid,
            _ => ProofResult::Invalid(
                "Memory safety fact must target module or function".to_string(),
            ),
        }
    }

    fn validate_ownership_valid(&self, fact: &ProofFact) -> ProofResult {
        // Ownership facts must have ownership evidence
        let has_ownership = fact
            .evidence
            .iter()
            .any(|e| matches!(e, super::fact::EvidenceItem::Ownership(_)));
        if self.require_evidence && !has_ownership {
            return ProofResult::Invalid(
                "Ownership validity fact requires ownership evidence".to_string(),
            );
        }

        ProofResult::Valid
    }

    fn validate_message_encode(&self, fact: &ProofFact) -> ProofResult {
        // Message encode facts must have type evidence
        let has_type = fact
            .evidence
            .iter()
            .any(|e| matches!(e, super::fact::EvidenceItem::Type(_)));
        if self.require_evidence && !has_type {
            return ProofResult::Invalid("Message encode fact requires type evidence".to_string());
        }

        // Must target a message or function
        match &fact.target {
            ProofTarget::Message(_) | ProofTarget::Function { .. } => ProofResult::Valid,
            _ => ProofResult::Warning(
                "Message encode targeting non-message may be unusual".to_string(),
            ),
        }
    }

    fn validate_message_decode(&self, fact: &ProofFact) -> ProofResult {
        let has_type = fact
            .evidence
            .iter()
            .any(|e| matches!(e, super::fact::EvidenceItem::Type(_)));
        if self.require_evidence && !has_type {
            return ProofResult::Invalid("Message decode fact requires type evidence".to_string());
        }

        ProofResult::Valid
    }

    fn validate_process_spawn(&self, fact: &ProofFact) -> ProofResult {
        // Spawn facts must target a function
        match &fact.target {
            ProofTarget::Function { arity, .. } => {
                if *arity > 255 {
                    return ProofResult::Invalid("Function arity too large".to_string());
                }
                ProofResult::Valid
            }
            _ => ProofResult::Invalid("Process spawn fact must target function".to_string()),
        }
    }

    fn validate_process_exit(&self, fact: &ProofFact) -> ProofResult {
        // Exit facts should have non-empty claim
        if fact.claim.is_empty() {
            return ProofResult::Invalid("Process exit fact requires a claim".to_string());
        }
        ProofResult::Valid
    }

    fn validate_link_valid(&self, fact: &ProofFact) -> ProofResult {
        // Link facts must be boundaries
        if !fact.target.is_boundary() {
            return ProofResult::Warning("Link validity fact should target boundary".to_string());
        }
        ProofResult::Valid
    }

    fn validate_monitor_valid(&self, fact: &ProofFact) -> ProofResult {
        // Monitor facts should have evidence
        if self.require_evidence && fact.is_empty() {
            return ProofResult::Invalid("Monitor validity fact requires evidence".to_string());
        }
        ProofResult::Valid
    }

    fn validate_registry_entry(&self, fact: &ProofFact) -> ProofResult {
        // Registry facts should have type evidence
        let has_type = fact
            .evidence
            .iter()
            .any(|e| matches!(e, super::fact::EvidenceItem::Type(_)));
        if self.require_evidence && !has_type {
            return ProofResult::Invalid("Registry entry fact requires type evidence".to_string());
        }
        ProofResult::Valid
    }

    fn validate_supervisor_spec(&self, fact: &ProofFact) -> ProofResult {
        // Supervisor specs must have evidence
        if self.require_evidence && fact.is_empty() {
            return ProofResult::Invalid("Supervisor spec fact requires evidence".to_string());
        }

        // Must target module
        if !fact.target.is_module() {
            return ProofResult::Warning(
                "Supervisor spec targeting non-module may be unusual".to_string(),
            );
        }
        ProofResult::Valid
    }

    fn validate_boundary_contract(&self, fact: &ProofFact) -> ProofResult {
        // Boundary contract facts must be boundaries
        if !fact.target.is_boundary() {
            return ProofResult::Invalid("Boundary contract fact must target boundary".to_string());
        }

        // Must have evidence
        if self.require_evidence && fact.is_empty() {
            return ProofResult::Invalid("Boundary contract fact requires evidence".to_string());
        }
        ProofResult::Valid
    }

    fn validate_effect_safety(&self, fact: &ProofFact) -> ProofResult {
        // Effect safety facts should have effect evidence
        let has_effect = fact
            .evidence
            .iter()
            .any(|e| matches!(e, super::fact::EvidenceItem::Effect(_)));
        if self.require_evidence && !has_effect {
            return ProofResult::Warning(
                "Effect safety fact should have effect evidence".to_string(),
            );
        }
        ProofResult::Valid
    }

    fn validate_timer_valid(&self, fact: &ProofFact) -> ProofResult {
        // Timer facts should have evidence
        if self.require_evidence && fact.is_empty() {
            return ProofResult::Warning("Timer validity fact should have evidence".to_string());
        }
        ProofResult::Valid
    }

    fn validate_code_load_safety(&self, fact: &ProofFact) -> ProofResult {
        // Code load safety facts must have module hash
        let has_hash = fact
            .evidence
            .iter()
            .any(|e| matches!(e, super::fact::EvidenceItem::BytecodeHash(_)));
        if self.require_evidence && !has_hash {
            return ProofResult::Invalid(
                "Code load safety fact requires bytecode hash".to_string(),
            );
        }
        ProofResult::Valid
    }

    fn validate_strict_checks(&self, fact: &ProofFact) -> bool {
        // In strict mode, require source location for critical facts
        if fact.kind.severity_level() >= 3 {
            if self.require_source_loc && fact.source_loc.is_none() {
                return false;
            }
        }

        // Require evidence for proven facts in strict mode
        if fact.is_proven && fact.is_empty() {
            return false;
        }

        true
    }
}

impl Default for ProofValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::fact::FactId;
    use super::*;

    fn create_test_validator() -> ProofValidator {
        ProofValidator::new()
    }

    fn create_valid_memory_fact() -> ProofFact {
        ProofFact::new(
            FactId::new(1, 0x1234),
            ProofKind::MemorySafety,
            ProofTarget::Module("test_mod".to_string()),
            "Heap allocation is within bounds",
        )
        .add_evidence(super::super::fact::EvidenceItem::Size(1024))
        .mark_proven()
    }

    #[test]
    fn test_validator_new() {
        let validator = create_test_validator();
        assert!(!validator.strict);
    }

    #[test]
    fn test_validator_strict() {
        let validator = ProofValidator::new().strict();
        assert!(validator.strict);
    }

    #[test]
    fn test_validator_validate_valid_fact() {
        let validator = create_test_validator();
        let fact = create_valid_memory_fact();
        let input = ValidationInput {
            fact: &fact,
            context: None,
        };

        assert!(validator.validate(&input));
    }

    #[test]
    fn test_validator_validate_empty_fact() {
        let validator = create_test_validator();
        let fact = ProofFact::new(
            FactId::new(1, 0x1234),
            ProofKind::MemorySafety,
            ProofTarget::Module("test".to_string()),
            "",
        );
        let input = ValidationInput {
            fact: &fact,
            context: None,
        };

        assert!(!validator.validate(&input));
    }

    #[test]
    fn test_validator_validate_message_encode() {
        let validator = create_test_validator();
        let fact = ProofFact::new(
            FactId::new(1, 0x1234),
            ProofKind::MessageEncode,
            ProofTarget::Message("msg1".to_string()),
            "Message can be encoded",
        )
        .add_type_evidence("binary");

        let input = ValidationInput {
            fact: &fact,
            context: None,
        };
        assert!(validator.validate(&input));
    }

    #[test]
    fn test_validator_validate_process_spawn() {
        let validator = create_test_validator();
        let fact = ProofFact::new(
            FactId::new(1, 0x1234),
            ProofKind::ProcessSpawn,
            ProofTarget::Function {
                module: "mod".to_string(),
                function: "start".to_string(),
                arity: 0,
            },
            "Spawn parameters are valid",
        )
        .add_note("spawn test");

        let input = ValidationInput {
            fact: &fact,
            context: None,
        };
        assert!(validator.validate(&input));
    }

    #[test]
    fn test_validator_validate_process_spawn_invalid_arity() {
        let validator = create_test_validator();
        let fact = ProofFact::new(
            FactId::new(1, 0x1234),
            ProofKind::ProcessSpawn,
            ProofTarget::Function {
                module: "mod".to_string(),
                function: "start".to_string(),
                arity: 200,
            },
            "Spawn parameters are valid",
        );

        let input = ValidationInput {
            fact: &fact,
            context: None,
        };
        // Should return Invalid, not just false
        let result = validator.validate(&input);
        assert!(!result);
    }

    #[test]
    fn test_validator_validate_boundary_contract() {
        let validator = create_test_validator();
        let fact = ProofFact::new(
            FactId::new(1, 0x1234),
            ProofKind::BoundaryContract,
            ProofTarget::Boundary {
                from_module: "rust_mod".to_string(),
                to_module: "beam_mod".to_string(),
            },
            "Cross-language boundary is safe",
        )
        .add_type_evidence("safe");

        let input = ValidationInput {
            fact: &fact,
            context: None,
        };
        assert!(validator.validate(&input));
    }

    #[test]
    fn test_validator_validate_boundary_contract_not_boundary() {
        let validator = create_test_validator();
        let fact = ProofFact::new(
            FactId::new(1, 0x1234),
            ProofKind::BoundaryContract,
            ProofTarget::Module("test_mod".to_string()),
            "Not a boundary",
        );

        let input = ValidationInput {
            fact: &fact,
            context: None,
        };
        assert!(!validator.validate(&input));
    }

    #[test]
    fn test_validator_validate_code_load_safety() {
        let validator = create_test_validator();
        let fact = ProofFact::new(
            FactId::new(1, 0x1234),
            ProofKind::CodeLoadSafety,
            ProofTarget::Module("test_mod".to_string()),
            "Code loading is safe",
        )
        .add_evidence(super::super::fact::EvidenceItem::BytecodeHash(
            "abc123".to_string(),
        ));

        let input = ValidationInput {
            fact: &fact,
            context: None,
        };
        assert!(validator.validate(&input));
    }

    #[test]
    fn test_validator_validate_code_load_safety_missing_hash() {
        let validator = create_test_validator().require_evidence();
        let fact = ProofFact::new(
            FactId::new(1, 0x1234),
            ProofKind::CodeLoadSafety,
            ProofTarget::Module("test_mod".to_string()),
            "Code loading is safe",
        );

        let input = ValidationInput {
            fact: &fact,
            context: None,
        };
        assert!(!validator.validate(&input));
    }

    #[test]
    fn test_proof_result_is_valid() {
        assert!(ProofResult::Valid.is_valid());
        assert!(!ProofResult::Invalid("test".to_string()).is_valid());
        assert!(!ProofResult::Warning("test".to_string()).is_valid());
    }

    #[test]
    fn test_proof_result_is_invalid() {
        assert!(ProofResult::Invalid("test".to_string()).is_invalid());
        assert!(!ProofResult::Valid.is_invalid());
    }

    #[test]
    fn test_proof_result_reason() {
        assert!(ProofResult::Valid.reason().is_none());
        assert_eq!(
            ProofResult::Invalid("test reason".to_string()).reason(),
            Some("test reason")
        );
    }
}
