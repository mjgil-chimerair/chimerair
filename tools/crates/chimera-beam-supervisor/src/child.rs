//! Child specification for BEAM supervisors.
//!
//! Defines the configuration for supervised child processes.

use serde::{Deserialize, Serialize};

use super::strategy::RestartKind;
use super::DEFAULT_SHUTDOWN_TIMEOUT_MS;

/// Child type (worker or supervisor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChildType {
    /// Worker process (actual work).
    Worker,
    /// Supervisor process (manages other children).
    Supervisor,
}

impl Default for ChildType {
    fn default() -> Self {
        ChildType::Worker
    }
}

impl ChildType {
    /// Get the type as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ChildType::Worker => "worker",
            ChildType::Supervisor => "supervisor",
        }
    }
}

/// Shutdown timeout for a child.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ShutdownTimeout(u32);

impl ShutdownTimeout {
    /// Create a new shutdown timeout in milliseconds.
    pub fn millis(ms: u32) -> Self {
        ShutdownTimeout(ms)
    }

    /// Create an infinite shutdown timeout.
    pub fn infinite() -> Self {
        ShutdownTimeout(u32::MAX)
    }

    /// Create the default shutdown timeout.
    pub fn default_timeout() -> Self {
        ShutdownTimeout(DEFAULT_SHUTDOWN_TIMEOUT_MS)
    }

    /// Get the timeout value in milliseconds.
    pub fn as_millis(&self) -> u32 {
        self.0
    }

    /// Check if this is infinite.
    pub fn is_infinite(&self) -> bool {
        self.0 == u32::MAX
    }
}

impl Default for ShutdownTimeout {
    fn default() -> Self {
        Self::default_timeout()
    }
}

/// Start function for a child.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartFunction {
    /// Module name.
    pub module: String,
    /// Function name.
    pub function: String,
    /// Arguments.
    pub args: Vec<serde_json::Value>,
}

impl StartFunction {
    /// Create a new start function.
    pub fn new(
        module: impl Into<String>,
        function: impl Into<String>,
        args: Vec<serde_json::Value>,
    ) -> Self {
        StartFunction {
            module: module.into(),
            function: function.into(),
            args,
        }
    }

    /// Create with no arguments.
    pub fn simple(module: impl Into<String>, function: impl Into<String>) -> Self {
        StartFunction {
            module: module.into(),
            function: function.into(),
            args: vec![],
        }
    }
}

/// Child specification (equivalent to OTP's child_spec()).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildSpec {
    /// Child ID (name).
    pub id: String,
    /// Start function.
    pub start: StartFunction,
    /// Restart strategy.
    pub restart: RestartKind,
    /// Shutdown timeout.
    pub shutdown: ShutdownTimeout,
    /// Child type.
    pub child_type: ChildType,
    /// Modules for code version tracking.
    pub modules: Vec<String>,
}

impl ChildSpec {
    /// Create a new child specification.
    pub fn new(
        id: impl Into<String>,
        start: StartFunction,
        restart: RestartKind,
        shutdown: ShutdownTimeout,
        child_type: ChildType,
    ) -> Self {
        ChildSpec {
            id: id.into(),
            start,
            restart,
            shutdown,
            child_type,
            modules: vec![],
        }
    }

    /// Create a worker child.
    pub fn worker(
        id: impl Into<String>,
        module: impl Into<String>,
        function: impl Into<String>,
    ) -> Self {
        ChildSpec {
            id: id.into(),
            start: StartFunction::simple(module, function),
            restart: RestartKind::Permanent,
            shutdown: ShutdownTimeout::default_timeout(),
            child_type: ChildType::Worker,
            modules: vec![],
        }
    }

    /// Create a supervisor child.
    pub fn supervisor(
        id: impl Into<String>,
        module: impl Into<String>,
        function: impl Into<String>,
    ) -> Self {
        ChildSpec {
            id: id.into(),
            start: StartFunction::simple(module, function),
            restart: RestartKind::Permanent,
            shutdown: ShutdownTimeout::infinite(),
            child_type: ChildType::Supervisor,
            modules: vec![],
        }
    }

    /// Set the modules list.
    pub fn with_modules(mut self, modules: Vec<String>) -> Self {
        self.modules = modules;
        self
    }

    /// Get the restart kind.
    pub fn restart_kind(&self) -> RestartKind {
        self.restart
    }

    /// Get the child type.
    pub fn child_type(&self) -> ChildType {
        self.child_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_child_type_as_str() {
        assert_eq!(ChildType::Worker.as_str(), "worker");
        assert_eq!(ChildType::Supervisor.as_str(), "supervisor");
    }

    #[test]
    fn test_shutdown_timeout_millis() {
        let timeout = ShutdownTimeout::millis(3000);
        assert_eq!(timeout.as_millis(), 3000);
        assert!(!timeout.is_infinite());
    }

    #[test]
    fn test_shutdown_timeout_infinite() {
        let timeout = ShutdownTimeout::infinite();
        assert!(timeout.is_infinite());
    }

    #[test]
    fn test_shutdown_timeout_default() {
        let timeout = ShutdownTimeout::default();
        assert_eq!(timeout.as_millis(), 5000);
    }

    #[test]
    fn test_start_function() {
        let start = StartFunction::new("mod", "fun", vec![1.into(), 2.into()]);
        assert_eq!(start.module, "mod");
        assert_eq!(start.function, "fun");
        assert_eq!(start.args.len(), 2);
    }

    #[test]
    fn test_start_function_simple() {
        let start = StartFunction::simple("mod", "fun");
        assert_eq!(start.module, "mod");
        assert_eq!(start.function, "fun");
        assert!(start.args.is_empty());
    }

    #[test]
    fn test_child_spec_worker() {
        let spec = ChildSpec::worker("child1", "mod", "fun");
        assert_eq!(spec.id, "child1");
        assert_eq!(spec.restart, RestartKind::Permanent);
        assert_eq!(spec.child_type, ChildType::Worker);
        assert_eq!(spec.shutdown.as_millis(), 5000);
    }

    #[test]
    fn test_child_spec_supervisor() {
        let spec = ChildSpec::supervisor("sup", "mod", "fun");
        assert_eq!(spec.id, "sup");
        assert_eq!(spec.child_type, ChildType::Supervisor);
        assert!(spec.shutdown.is_infinite());
    }

    #[test]
    fn test_child_spec_with_modules() {
        let spec = ChildSpec::worker("child1", "mod", "fun").with_modules(vec!["mod".to_string()]);
        assert_eq!(spec.modules.len(), 1);
        assert_eq!(spec.modules[0], "mod");
    }
}
