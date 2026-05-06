//! BEAM dialect definition.
//!
//! The BEAM dialect is the MLIR representation for BEAM semantics.

use serde::{Deserialize, Serialize};

/// BEAM dialect identifier.
pub const BEAM_DIALECT: &str = "beam";

/// BEAM dialect namespace.
pub const BEAM_NAMESPACE: &str = "beam";

/// BEAM dialect configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamDialectConfig {
    /// Enable strict mode (all operations must be well-typed).
    pub strict: bool,
    /// Enable runtime checks in generated code.
    pub runtime_checks: bool,
    /// Default timeout for receive operations (ms).
    pub default_recv_timeout_ms: u64,
}

impl Default for BeamDialectConfig {
    fn default() -> Self {
        BeamDialectConfig {
            strict: true,
            runtime_checks: false,
            default_recv_timeout_ms: 5000,
        }
    }
}

/// BEAM dialect context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamDialect {
    /// Configuration.
    pub config: BeamDialectConfig,
    /// Registered types.
    pub types: Vec<String>,
    /// Registered operations.
    pub operations: Vec<String>,
}

impl BeamDialect {
    /// Create a new BEAM dialect.
    pub fn new() -> Self {
        BeamDialect {
            config: BeamDialectConfig::default(),
            types: vec![
                "beam.process".to_string(),
                "beam.pid".to_string(),
                "beam.port".to_string(),
                "beam.reference".to_string(),
                "beam.atom".to_string(),
                "beam.tuple".to_string(),
                "beam.list".to_string(),
                "beam.binary".to_string(),
                "beam.closure".to_string(),
                "beam.map".to_string(),
                "beam.catch".to_string(),
                "beam.noreturn".to_string(),
            ],
            operations: vec![
                "beam.spawn".to_string(),
                "beam.spawn_link".to_string(),
                "beam.spawn_monitor".to_string(),
                "beam.exit".to_string(),
                "beam.exit2".to_string(),
                "beam.kill".to_string(),
                "beam.link".to_string(),
                "beam.unlink".to_string(),
                "beam.monitor".to_string(),
                "beam.demonitor".to_string(),
                "beam.send".to_string(),
                "beam.send_after".to_string(),
                "beam.recv".to_string(),
                "beam.recv_next".to_string(),
                "beam.msg_peek".to_string(),
                "beam.register".to_string(),
                "beam.unregister".to_string(),
                "beam.whereis".to_string(),
                "beam.get_state".to_string(),
                "beam.put_state".to_string(),
                "beam.gc_collect".to_string(),
                "beam.now".to_string(),
                "beam.timestamp".to_string(),
                "beam.sleep".to_string(),
                "beam.supervisor_start".to_string(),
                "beam.supervisor_init".to_string(),
                "beam.child_define".to_string(),
                "beam.child_spec".to_string(),
                "beam.code_load".to_string(),
                "beam.code_replace".to_string(),
                "beam.code_check".to_string(),
            ],
        }
    }

    /// Create with custom config.
    pub fn with_config(config: BeamDialectConfig) -> Self {
        BeamDialect {
            config,
            types: vec![],
            operations: vec![],
        }
    }

    /// Get dialect name.
    pub fn name(&self) -> &'static str {
        BEAM_DIALECT
    }

    /// Get namespace.
    pub fn namespace(&self) -> &'static str {
        BEAM_NAMESPACE
    }

    /// Check if a type is registered.
    pub fn is_type_registered(&self, type_name: &str) -> bool {
        self.types.iter().any(|t| t == type_name)
    }

    /// Check if an operation is registered.
    pub fn is_op_registered(&self, op_name: &str) -> bool {
        self.operations.iter().any(|o| o == op_name)
    }

    /// Get all registered types.
    pub fn registered_types(&self) -> &[String] {
        &self.types
    }

    /// Get all registered operations.
    pub fn registered_operations(&self) -> &[String] {
        &self.operations
    }
}

impl Default for BeamDialect {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialect_name() {
        let dialect = BeamDialect::new();
        assert_eq!(dialect.name(), "beam");
        assert_eq!(dialect.namespace(), "beam");
    }

    #[test]
    fn test_default_config() {
        let config = BeamDialectConfig::default();
        assert!(config.strict);
        assert!(!config.runtime_checks);
        assert_eq!(config.default_recv_timeout_ms, 5000);
    }

    #[test]
    fn test_type_registration() {
        let dialect = BeamDialect::new();
        assert!(dialect.is_type_registered("beam.pid"));
        assert!(dialect.is_type_registered("beam.atom"));
        assert!(!dialect.is_type_registered("beam.invalid"));
    }

    #[test]
    fn test_op_registration() {
        let dialect = BeamDialect::new();
        assert!(dialect.is_op_registered("beam.spawn"));
        assert!(dialect.is_op_registered("beam.send"));
        assert!(!dialect.is_op_registered("beam.invalid"));
    }

    #[test]
    fn test_registered_types() {
        let dialect = BeamDialect::new();
        let types = dialect.registered_types();
        assert!(types.len() >= 12);
    }

    #[test]
    fn test_registered_operations() {
        let dialect = BeamDialect::new();
        let ops = dialect.registered_operations();
        assert!(ops.len() >= 30);
    }

    #[test]
    fn test_custom_config() {
        let config = BeamDialectConfig {
            strict: false,
            runtime_checks: true,
            default_recv_timeout_ms: 10000,
        };
        let dialect = BeamDialect::with_config(config);
        assert!(!dialect.config.strict);
        assert!(dialect.config.runtime_checks);
        assert_eq!(dialect.config.default_recv_timeout_ms, 10000);
    }
}
