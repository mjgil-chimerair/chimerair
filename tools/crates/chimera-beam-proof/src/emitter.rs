//! Proof artifact emitter for BEAM modules.
//!
//! Collects and emits proof facts for BEAM constructs ensuring
//! ownership invariants, memory safety, and message protocol compliance.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use super::fact::{FactId, ProofFact, ProofKind, ProofTarget};
use super::validate::{ProofValidator, ValidationInput};

/// Statistics for proof emission.
#[derive(Debug, Clone, Default)]
pub struct EmissionStats {
    /// Number of facts emitted.
    pub facts_emitted: u64,
    /// Number of facts proven.
    pub facts_proven: u64,
    /// Number of facts skipped.
    pub facts_skipped: u64,
    /// Number of validation errors.
    pub validation_errors: u64,
}

impl EmissionStats {
    /// Create a new emission stats.
    pub fn new() -> Self {
        EmissionStats {
            facts_emitted: 0,
            facts_proven: 0,
            facts_skipped: 0,
            validation_errors: 0,
        }
    }

    /// Increment facts emitted.
    pub fn record_fact(&mut self) {
        self.facts_emitted += 1;
    }

    /// Increment facts proven.
    pub fn record_proven(&mut self) {
        self.facts_proven += 1;
    }

    /// Increment facts skipped.
    pub fn record_skipped(&mut self) {
        self.facts_skipped += 1;
    }

    /// Increment validation errors.
    pub fn record_error(&mut self) {
        self.validation_errors += 1;
    }
}

/// Proof artifact emitter.
#[derive(Debug, Clone)]
pub struct ProofEmitter {
    /// Facts collected so far.
    facts: HashMap<FactId, ProofFact>,
    /// Next fact index.
    next_index: u32,
    /// Module fingerprint.
    module_fingerprint: u32,
    /// Current module name.
    current_module: Option<String>,
    /// Statistics.
    stats: EmissionStats,
    /// Whether to validate facts.
    validate: bool,
    /// Validator reference.
    validator: Option<Box<ProofValidator>>,
}

impl ProofEmitter {
    /// Create a new proof emitter.
    pub fn new(module_fingerprint: u32) -> Self {
        ProofEmitter {
            facts: HashMap::new(),
            next_index: 0,
            module_fingerprint,
            current_module: None,
            stats: EmissionStats::new(),
            validate: false,
            validator: None,
        }
    }

    /// Create with a validator.
    pub fn with_validator(module_fingerprint: u32, validator: ProofValidator) -> Self {
        ProofEmitter {
            facts: HashMap::new(),
            next_index: 0,
            module_fingerprint,
            current_module: None,
            stats: EmissionStats::new(),
            validate: true,
            validator: Some(Box::new(validator)),
        }
    }

    /// Set current module context.
    pub fn set_module(&mut self, module_name: impl Into<String>) {
        self.current_module = Some(module_name.into());
    }

    /// Clear current module context.
    pub fn clear_module(&mut self) {
        self.current_module = None;
    }

    /// Generate a new fact ID.
    fn next_fact_id(&mut self) -> FactId {
        let id = FactId::new(self.next_index, self.module_fingerprint);
        self.next_index += 1;
        id
    }

    /// Add a fact to the emitter.
    fn add_fact(&mut self, fact: ProofFact) -> bool {
        // Validate if enabled
        if self.validate {
            if let Some(ref validator) = self.validator {
                let input = ValidationInput {
                    fact: &fact,
                    context: self.current_module.as_deref(),
                };
                if !validator.validate(&input) {
                    self.stats.record_error();
                    return false;
                }
            }
        }

        let id = fact.id;
        self.facts.insert(id, fact);
        self.stats.record_fact();
        true
    }

    /// Emit a memory safety fact.
    pub fn emit_memory_safety(
        &mut self,
        target: ProofTarget,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let fact = ProofFact::new(self.next_fact_id(), ProofKind::MemorySafety, target, claim);

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Emit an ownership validity fact.
    pub fn emit_ownership_valid(
        &mut self,
        target: ProofTarget,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let fact = ProofFact::new(
            self.next_fact_id(),
            ProofKind::OwnershipValid,
            target,
            claim,
        );

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Emit a message encode fact.
    pub fn emit_message_encode(
        &mut self,
        target: ProofTarget,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let fact = ProofFact::new(self.next_fact_id(), ProofKind::MessageEncode, target, claim);

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Emit a message decode fact.
    pub fn emit_message_decode(
        &mut self,
        target: ProofTarget,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let fact = ProofFact::new(self.next_fact_id(), ProofKind::MessageDecode, target, claim);

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Emit a process spawn fact.
    pub fn emit_process_spawn(
        &mut self,
        target: ProofTarget,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let fact = ProofFact::new(self.next_fact_id(), ProofKind::ProcessSpawn, target, claim);

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Emit a process exit fact.
    pub fn emit_process_exit(
        &mut self,
        target: ProofTarget,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let fact = ProofFact::new(self.next_fact_id(), ProofKind::ProcessExit, target, claim);

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Emit a link validity fact.
    pub fn emit_link_valid(
        &mut self,
        target: ProofTarget,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let fact = ProofFact::new(self.next_fact_id(), ProofKind::LinkValid, target, claim);

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Emit a monitor validity fact.
    pub fn emit_monitor_valid(
        &mut self,
        target: ProofTarget,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let fact = ProofFact::new(self.next_fact_id(), ProofKind::MonitorValid, target, claim);

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Emit a registry entry fact.
    pub fn emit_registry_entry(
        &mut self,
        target: ProofTarget,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let fact = ProofFact::new(self.next_fact_id(), ProofKind::RegistryEntry, target, claim);

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Emit a supervisor spec fact.
    pub fn emit_supervisor_spec(
        &mut self,
        target: ProofTarget,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let fact = ProofFact::new(
            self.next_fact_id(),
            ProofKind::SupervisorSpec,
            target,
            claim,
        );

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Emit a boundary contract fact.
    pub fn emit_boundary_contract(
        &mut self,
        from_module: &str,
        to_module: &str,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let target = ProofTarget::Boundary {
            from_module: from_module.to_string(),
            to_module: to_module.to_string(),
        };

        let fact = ProofFact::new(
            self.next_fact_id(),
            ProofKind::BoundaryContract,
            target,
            claim,
        );

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Emit an effect safety fact.
    pub fn emit_effect_safety(
        &mut self,
        target: ProofTarget,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let fact = ProofFact::new(self.next_fact_id(), ProofKind::EffectSafety, target, claim);

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Emit a timer validity fact.
    pub fn emit_timer_valid(
        &mut self,
        target: ProofTarget,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let fact = ProofFact::new(self.next_fact_id(), ProofKind::TimerValid, target, claim);

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Emit a code load safety fact.
    pub fn emit_code_load_safety(
        &mut self,
        target: ProofTarget,
        claim: impl Into<String>,
    ) -> Option<FactId> {
        let fact = ProofFact::new(
            self.next_fact_id(),
            ProofKind::CodeLoadSafety,
            target,
            claim,
        );

        let id = fact.id;
        if self.add_fact(fact) {
            Some(id)
        } else {
            None
        }
    }

    /// Get all facts.
    pub fn facts(&self) -> Vec<&ProofFact> {
        self.facts.values().collect()
    }

    /// Get facts by kind.
    pub fn facts_by_kind(&self, kind: ProofKind) -> Vec<&ProofFact> {
        self.facts.values().filter(|f| f.kind == kind).collect()
    }

    /// Get facts by target.
    pub fn facts_by_target(&self, target: &ProofTarget) -> Vec<&ProofFact> {
        self.facts
            .values()
            .filter(|f| &f.target == target)
            .collect()
    }

    /// Get total fact count.
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }

    /// Get statistics.
    pub fn stats(&self) -> &EmissionStats {
        &self.stats
    }

    /// Get the module fingerprint.
    pub fn module_fingerprint(&self) -> u32 {
        self.module_fingerprint
    }

    /// Compute artifact hash.
    pub fn artifact_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.module_fingerprint.to_le_bytes());
        hasher.update((self.facts.len() as u64).to_le_bytes());

        for fact in self.facts.values() {
            hasher.update(fact.id.index.to_le_bytes());
            hasher.update(fact.kind.as_str().as_bytes());
        }

        hex::encode(hasher.finalize())
    }

    /// Clear all facts.
    pub fn clear(&mut self) {
        self.facts.clear();
        self.next_index = 0;
        self.stats = EmissionStats::new();
    }

    /// Check if emitter is empty.
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }
}

/// Proof artifact with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofArtifact {
    /// Artifact version.
    pub version: u32,
    /// Module fingerprint.
    pub module_fingerprint: u32,
    /// Module name.
    pub module_name: Option<String>,
    /// All facts.
    pub facts: Vec<ProofFact>,
    /// Artifact hash.
    pub artifact_hash: String,
    /// Generation timestamp.
    pub generated_at: u64,
}

impl ProofArtifact {
    /// Create a new proof artifact from an emitter.
    pub fn from_emitter(
        emitter: &ProofEmitter,
        module_name: Option<String>,
        generated_at: u64,
    ) -> Self {
        let mut facts: Vec<_> = emitter.facts().into_iter().cloned().collect();
        facts.sort_by_key(|f| f.id.index);

        ProofArtifact {
            version: 1,
            module_fingerprint: emitter.module_fingerprint(),
            module_name,
            facts,
            artifact_hash: emitter.artifact_hash(),
            generated_at,
        }
    }

    /// Get fact count.
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_emitter() -> ProofEmitter {
        ProofEmitter::new(0x1234)
    }

    #[test]
    fn test_emitter_new() {
        let emitter = create_test_emitter();
        assert!(emitter.is_empty());
        assert_eq!(emitter.fact_count(), 0);
    }

    #[test]
    fn test_emitter_module_fingerprint() {
        let emitter = ProofEmitter::new(0xABCD);
        assert_eq!(emitter.module_fingerprint(), 0xABCD);
    }

    #[test]
    fn test_emitter_set_module() {
        let mut emitter = create_test_emitter();
        emitter.set_module("test_mod");
        emitter.clear_module();
        assert!(emitter.current_module.is_none());
    }

    #[test]
    fn test_emitter_fact_id_sequence() {
        let mut emitter = create_test_emitter();

        let id1 = emitter.emit_memory_safety(ProofTarget::Module("mod".to_string()), "Test fact 1");
        let id2 = emitter.emit_memory_safety(ProofTarget::Module("mod".to_string()), "Test fact 2");

        assert!(id1.is_some());
        assert!(id2.is_some());
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_emitter_emit_memory_safety() {
        let mut emitter = create_test_emitter();

        let id = emitter.emit_memory_safety(
            ProofTarget::Module("test_mod".to_string()),
            "Heap allocation is within bounds",
        );

        assert!(id.is_some());
        assert_eq!(emitter.fact_count(), 1);
    }

    #[test]
    fn test_emitter_emit_ownership_valid() {
        let mut emitter = create_test_emitter();

        let id = emitter.emit_ownership_valid(
            ProofTarget::Function {
                module: "mod".to_string(),
                function: "fun".to_string(),
                arity: 1,
            },
            "Ownership invariant holds",
        );

        assert!(id.is_some());
        assert_eq!(emitter.fact_count(), 1);
    }

    #[test]
    fn test_emitter_emit_message_encode() {
        let mut emitter = create_test_emitter();

        let id = emitter.emit_message_encode(
            ProofTarget::Message("msg1".to_string()),
            "Message can be binary encoded",
        );

        assert!(id.is_some());
        assert_eq!(emitter.fact_count(), 1);
    }

    #[test]
    fn test_emitter_emit_message_decode() {
        let mut emitter = create_test_emitter();

        let id = emitter.emit_message_decode(
            ProofTarget::Message("msg1".to_string()),
            "Binary can be decoded to term",
        );

        assert!(id.is_some());
        assert_eq!(emitter.fact_count(), 1);
    }

    #[test]
    fn test_emitter_emit_process_spawn() {
        let mut emitter = create_test_emitter();

        let id = emitter.emit_process_spawn(
            ProofTarget::Function {
                module: "mod".to_string(),
                function: "start".to_string(),
                arity: 0,
            },
            "Spawn parameters are valid",
        );

        assert!(id.is_some());
        assert_eq!(emitter.fact_count(), 1);
    }

    #[test]
    fn test_emitter_emit_process_exit() {
        let mut emitter = create_test_emitter();

        let id = emitter.emit_process_exit(
            ProofTarget::Process("pid1".to_string()),
            "Exit reason is valid",
        );

        assert!(id.is_some());
        assert_eq!(emitter.fact_count(), 1);
    }

    #[test]
    fn test_emitter_emit_link_valid() {
        let mut emitter = create_test_emitter();

        let id = emitter.emit_link_valid(
            ProofTarget::Boundary {
                from_module: "mod1".to_string(),
                to_module: "mod2".to_string(),
            },
            "Link is valid",
        );

        assert!(id.is_some());
        assert_eq!(emitter.fact_count(), 1);
    }

    #[test]
    fn test_emitter_emit_boundary_contract() {
        let mut emitter = create_test_emitter();

        let id = emitter.emit_boundary_contract(
            "rust_mod",
            "beam_mod",
            "Cross-language boundary is safe",
        );

        assert!(id.is_some());
        assert_eq!(emitter.fact_count(), 1);
    }

    #[test]
    fn test_emitter_facts_by_kind() {
        let mut emitter = create_test_emitter();

        emitter.emit_memory_safety(ProofTarget::Module("mod".to_string()), "Memory fact");
        emitter.emit_ownership_valid(ProofTarget::Module("mod".to_string()), "Ownership fact");
        emitter.emit_memory_safety(
            ProofTarget::Module("mod".to_string()),
            "Another memory fact",
        );

        let memory_facts = emitter.facts_by_kind(ProofKind::MemorySafety);
        assert_eq!(memory_facts.len(), 2);
    }

    #[test]
    fn test_emitter_facts_by_target() {
        let mut emitter = create_test_emitter();

        let target = ProofTarget::Module("mod".to_string());
        emitter.emit_memory_safety(target.clone(), "Fact 1");
        emitter.emit_ownership_valid(target.clone(), "Fact 2");

        let facts = emitter.facts_by_target(&target);
        assert_eq!(facts.len(), 2);
    }

    #[test]
    fn test_emitter_artifact_hash() {
        let mut emitter = create_test_emitter();

        emitter.emit_memory_safety(ProofTarget::Module("mod".to_string()), "Test");

        let hash1 = emitter.artifact_hash();
        let hash2 = emitter.artifact_hash();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_emitter_clear() {
        let mut emitter = create_test_emitter();

        emitter.emit_memory_safety(ProofTarget::Module("mod".to_string()), "Test");

        assert_eq!(emitter.fact_count(), 1);

        emitter.clear();

        assert!(emitter.is_empty());
        assert_eq!(emitter.fact_count(), 0);
    }

    #[test]
    fn test_proof_artifact_from_emitter() {
        let mut emitter = create_test_emitter();

        emitter.emit_memory_safety(ProofTarget::Module("mod".to_string()), "Test fact");

        let artifact = ProofArtifact::from_emitter(&emitter, Some("test_mod".to_string()), 1000);

        assert_eq!(artifact.version, 1);
        assert_eq!(artifact.module_fingerprint, 0x1234);
        assert_eq!(artifact.module_name, Some("test_mod".to_string()));
        assert_eq!(artifact.fact_count(), 1);
        assert_eq!(artifact.generated_at, 1000);
    }

    #[test]
    fn test_emission_stats() {
        let mut stats = EmissionStats::new();
        stats.record_fact();
        stats.record_fact();
        stats.record_proven();
        stats.record_skipped();
        stats.record_error();

        assert_eq!(stats.facts_emitted, 2);
        assert_eq!(stats.facts_proven, 1);
        assert_eq!(stats.facts_skipped, 1);
        assert_eq!(stats.validation_errors, 1);
    }
}
