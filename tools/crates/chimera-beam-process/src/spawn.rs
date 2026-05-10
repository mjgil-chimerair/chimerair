//! BEAM process spawn configuration and results.
//!
//! Handles process creation with various spawn options.

use chimera_beam_schema::Atom;
use serde::{Deserialize, Serialize};

/// Configuration for spawning a new process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnConfig {
    /// Module name.
    pub module: Atom,
    /// Function name.
    pub function: Atom,
    /// Arguments to the function.
    pub args: Vec<Term>,
    /// Parent PID (for linkage tracking).
    pub parent: Option<u64>,
    /// Spawn flags.
    pub flags: SpawnFlags,
    /// Initial heap size hint.
    pub heap_size_hint: Option<u32>,
    /// Stack size hint.
    pub stack_size_hint: Option<u32>,
}

impl SpawnConfig {
    /// Create a new spawn config.
    pub fn new(module: impl Into<String>, function: impl Into<String>, args: Vec<Term>) -> Self {
        SpawnConfig {
            module: Atom::new(module),
            function: Atom::new(function),
            args,
            parent: None,
            flags: SpawnFlags::default(),
            heap_size_hint: None,
            stack_size_hint: None,
        }
    }

    /// Set parent PID.
    pub fn with_parent(mut self, pid: u64) -> Self {
        self.parent = Some(pid);
        self
    }

    /// Set spawn flags.
    pub fn with_flags(mut self, flags: SpawnFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Set heap size hint.
    pub fn with_heap_size(mut self, size: u32) -> Self {
        self.heap_size_hint = Some(size);
        self
    }

    /// Set stack size hint.
    pub fn with_stack_size(mut self, size: u32) -> Self {
        self.stack_size_hint = Some(size);
        self
    }
}

/// Spawn-related flags.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SpawnFlags {
    /// Spawn linked to parent.
    pub linked: bool,
    /// Spawn monitored.
    pub monitored: bool,
    /// Process priority (0=low, 1=normal, 2=high, 3=max).
    pub priority: u8,
    /// Disable GC for this process.
    pub no_gc: bool,
}

impl SpawnFlags {
    /// Create default flags.
    pub fn new() -> Self {
        SpawnFlags::default()
    }

    /// Enable linked spawn.
    pub fn linked(mut self) -> Self {
        self.linked = true;
        self
    }

    /// Enable monitored spawn.
    pub fn monitored(mut self) -> Self {
        self.monitored = true;
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, p: u8) -> Self {
        self.priority = p.min(3);
        self
    }
}

/// Erlang term representation for spawn arguments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Term {
    Atom(String),
    Int(i64),
    Float(f64),
    Tuple(Vec<Term>),
    List(Vec<Term>),
    Binary(Vec<u8>),
    Pid(u64),
    Ref(u64),
}

impl Term {
    /// Create an atom term.
    pub fn atom(s: impl Into<String>) -> Self {
        Term::Atom(s.into())
    }

    /// Create an integer term.
    pub fn int(i: i64) -> Self {
        Term::Int(i)
    }

    /// Create a tuple term.
    pub fn tuple(items: Vec<Term>) -> Self {
        Term::Tuple(items)
    }

    /// Create a list term.
    pub fn list(items: Vec<Term>) -> Self {
        Term::List(items)
    }

    /// Check if this is an atom.
    pub fn is_atom(&self) -> bool {
        matches!(self, Term::Atom(_))
    }

    /// Check if this is an integer.
    pub fn is_int(&self) -> bool {
        matches!(self, Term::Int(_))
    }

    /// Get atom value if this is an atom.
    pub fn as_atom(&self) -> Option<&str> {
        match self {
            Term::Atom(s) => Some(s),
            _ => None,
        }
    }

    /// Get integer value if this is an integer.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Term::Int(i) => Some(*i),
            _ => None,
        }
    }
}

/// Result of a spawn operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnResult {
    /// The PID of the spawned process.
    pub pid: u64,
    /// Monitor reference if monitored spawn.
    pub monitor_ref: Option<u64>,
    /// Spawn failed (error).
    pub error: Option<String>,
}

impl SpawnResult {
    /// Create a successful spawn result.
    pub fn ok(pid: u64) -> Self {
        SpawnResult {
            pid,
            monitor_ref: None,
            error: None,
        }
    }

    /// Create a successful monitored spawn result.
    pub fn monitored(pid: u64, monitor_ref: u64) -> Self {
        SpawnResult {
            pid,
            monitor_ref: Some(monitor_ref),
            error: None,
        }
    }

    /// Create a failed spawn result.
    pub fn error(msg: impl Into<String>) -> Self {
        SpawnResult {
            pid: 0,
            monitor_ref: None,
            error: Some(msg.into()),
        }
    }

    /// Check if spawn was successful.
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }

    /// Check if spawn failed.
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

/// Function for initializing a new process.
pub trait ProcessInitializer: Send + Sync {
    /// Initialize the process with its PID.
    fn init(&self, pid: u64) -> Result<(), String>;

    /// Get the entry function.
    fn entry(&self) -> (String, String, Vec<Term>);
}

/// Simple process initializer from config.
#[derive(Debug, Clone)]
pub struct ConfigInitializer {
    config: SpawnConfig,
}

impl ConfigInitializer {
    /// Create from spawn config.
    pub fn new(config: SpawnConfig) -> Self {
        ConfigInitializer { config }
    }
}

impl ProcessInitializer for ConfigInitializer {
    fn init(&self, _pid: u64) -> Result<(), String> {
        // In real implementation, this would create the process and schedule it
        Ok(())
    }

    fn entry(&self) -> (String, String, Vec<Term>) {
        (
            self.config.module.0.clone(),
            self.config.function.0.clone(),
            self.config.args.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_config_creation() {
        let config = SpawnConfig::new("module", "function", vec![Term::int(1), Term::int(2)]);
        assert_eq!(config.module.0, "module");
        assert_eq!(config.function.0, "function");
        assert_eq!(config.args.len(), 2);
    }

    #[test]
    fn test_spawn_config_with_parent() {
        let config = SpawnConfig::new("m", "f", vec![])
            .with_parent(100)
            .with_heap_size(1024);
        assert_eq!(config.parent, Some(100));
        assert_eq!(config.heap_size_hint, Some(1024));
    }

    #[test]
    fn test_spawn_flags() {
        let flags = SpawnFlags::new().linked().monitored().with_priority(2);
        assert!(flags.linked);
        assert!(flags.monitored);
        assert_eq!(flags.priority, 2);
    }

    #[test]
    fn test_term_constructors() {
        assert!(Term::atom("test").is_atom());
        assert!(Term::int(42).is_int());
        assert_eq!(Term::atom("foo").as_atom(), Some("foo"));
        assert_eq!(Term::int(99).as_int(), Some(99));
    }

    #[test]
    fn test_spawn_result_ok() {
        let result = SpawnResult::ok(123);
        assert!(result.is_ok());
        assert!(!result.is_error());
        assert_eq!(result.pid, 123);
        assert!(result.monitor_ref.is_none());
    }

    #[test]
    fn test_spawn_result_monitored() {
        let result = SpawnResult::monitored(456, 789);
        assert!(result.is_ok());
        assert_eq!(result.pid, 456);
        assert_eq!(result.monitor_ref, Some(789));
    }

    #[test]
    fn test_spawn_result_error() {
        let result = SpawnResult::error("bad function");
        assert!(!result.is_ok());
        assert!(result.is_error());
        assert_eq!(result.pid, 0);
        assert_eq!(result.error, Some("bad function".to_string()));
    }

    #[test]
    fn test_config_initializer() {
        let config = SpawnConfig::new("mod", "fun", vec![Term::int(1)]);
        let init = ConfigInitializer::new(config);
        let (m, f, args) = init.entry();
        assert_eq!(m, "mod");
        assert_eq!(f, "fun");
        assert_eq!(args.len(), 1);
    }
}
