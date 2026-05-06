//! BEAM dialect operations.
//!
//! Defines the operations for BEAM semantics in MLIR.

use super::types::BeamType;
use serde::{Deserialize, Serialize};

/// BEAM operation kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeamOpKind {
    // Process lifecycle
    Spawn,
    SpawnLink,
    SpawnMonitor,
    Exit,
    Exit2,
    Kill,

    // Link and monitor
    Link,
    Unlink,
    Monitor,
    Demonitor,

    // Message passing
    Send,
    SendAfter,
    Recv,
    RecvNext,
    MsgPeek,

    // Process registry
    Register,
    Unregister,
    Whereis,

    // Process state
    GetState,
    PutState,
    GcCollect,

    // Timing
    Now,
    Timestamp,
    Sleep,

    // Supervisor
    SupervisorStart,
    SupervisorInit,
    ChildDefine,
    ChildSpec,

    // Code loading
    CodeLoad,
    CodeReplace,
    CodeCheck,

    // Control flow
    Receive,
    ReceiveTimeout,
    Try,
    Catch,
    Throw,
    Reraise,
}

impl Default for BeamOpKind {
    fn default() -> Self {
        BeamOpKind::Send
    }
}

/// A BEAM operation in MLIR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamOp {
    /// Kind of operation.
    pub kind: BeamOpKind,
    /// Operation name (MLIR op name).
    pub name: String,
    /// Input types.
    pub inputs: Vec<BeamType>,
    /// Output types.
    pub outputs: Vec<BeamType>,
    /// Attributes (key-value pairs).
    pub attributes: Vec<(String, String)>,
    /// Regions (for structured ops like receive, try).
    pub regions: usize,
}

impl BeamOp {
    /// Create a spawn operation.
    pub fn spawn(module: String, function: String) -> Self {
        BeamOp {
            kind: BeamOpKind::Spawn,
            name: "beam.spawn".to_string(),
            inputs: vec![BeamType::atom(), BeamType::atom()], // module, function
            outputs: vec![BeamType::pid()],
            attributes: vec![
                ("module".to_string(), module),
                ("function".to_string(), function),
            ],
            regions: 0,
        }
    }

    /// Create a spawn_link operation.
    pub fn spawn_link(module: String, function: String) -> Self {
        BeamOp {
            kind: BeamOpKind::SpawnLink,
            name: "beam.spawn_link".to_string(),
            inputs: vec![BeamType::atom(), BeamType::atom()],
            outputs: vec![BeamType::pid()],
            attributes: vec![
                ("module".to_string(), module),
                ("function".to_string(), function),
            ],
            regions: 0,
        }
    }

    /// Create a spawn_monitor operation.
    pub fn spawn_monitor(module: String, function: String) -> Self {
        BeamOp {
            kind: BeamOpKind::SpawnMonitor,
            name: "beam.spawn_monitor".to_string(),
            inputs: vec![BeamType::atom(), BeamType::atom()],
            outputs: vec![BeamType::pid(), BeamType::reference()],
            attributes: vec![
                ("module".to_string(), module),
                ("function".to_string(), function),
            ],
            regions: 0,
        }
    }

    /// Create a send operation.
    pub fn send(dest: BeamType, msg: BeamType) -> Self {
        BeamOp {
            kind: BeamOpKind::Send,
            name: "beam.send".to_string(),
            inputs: vec![dest, msg.clone()],
            outputs: vec![msg], // send returns the message
            attributes: vec![],
            regions: 0,
        }
    }

    /// Create a receive operation.
    pub fn receive(patterns: usize) -> Self {
        BeamOp {
            kind: BeamOpKind::Recv,
            name: "beam.recv".to_string(),
            inputs: vec![],
            outputs: vec![], // result type depends on patterns
            attributes: vec![],
            regions: patterns, // one region per pattern
        }
    }

    /// Create a link operation.
    pub fn link(pid: BeamType) -> Self {
        BeamOp {
            kind: BeamOpKind::Link,
            name: "beam.link".to_string(),
            inputs: vec![pid],
            outputs: vec![],
            attributes: vec![],
            regions: 0,
        }
    }

    /// Create a monitor operation.
    pub fn monitor(target: BeamType) -> Self {
        BeamOp {
            kind: BeamOpKind::Monitor,
            name: "beam.monitor".to_string(),
            inputs: vec![target],
            outputs: vec![BeamType::reference()],
            attributes: vec![],
            regions: 0,
        }
    }

    /// Create an exit operation.
    pub fn exit(reason: BeamType) -> Self {
        BeamOp {
            kind: BeamOpKind::Exit,
            name: "beam.exit".to_string(),
            inputs: vec![reason],
            outputs: vec![],
            attributes: vec![],
            regions: 0,
        }
    }

    /// Create a register operation.
    pub fn register(name: BeamType, pid: BeamType) -> Self {
        BeamOp {
            kind: BeamOpKind::Register,
            name: "beam.register".to_string(),
            inputs: vec![name, pid],
            outputs: vec![BeamType::atom()], // ok | error
            attributes: vec![],
            regions: 0,
        }
    }

    /// Create a whereis operation.
    pub fn whereis(name: BeamType) -> Self {
        BeamOp {
            kind: BeamOpKind::Whereis,
            name: "beam.whereis".to_string(),
            inputs: vec![name],
            outputs: vec![BeamType::pid()],
            attributes: vec![],
            regions: 0,
        }
    }

    /// Get the MLIR operation name.
    pub fn op_name(&self) -> &str {
        &self.name
    }
}

/// Receive clause with pattern and handler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveClause {
    /// Pattern to match.
    pub pattern: BeamType,
    /// Handler function name.
    pub handler: String,
}

impl ReceiveClause {
    /// Create a new receive clause.
    pub fn new(pattern: BeamType, handler: impl Into<String>) -> Self {
        ReceiveClause {
            pattern,
            handler: handler.into(),
        }
    }
}

/// Spawn attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnAttributes {
    /// Module name.
    pub module: String,
    /// Function name.
    pub function: String,
    /// Arguments (if known at compile time).
    pub args: Vec<BeamType>,
    /// Parent PID for linkage.
    pub parent: Option<u64>,
    /// Priority level.
    pub priority: u8,
}

impl SpawnAttributes {
    /// Create new spawn attributes.
    pub fn new(module: impl Into<String>, function: impl Into<String>) -> Self {
        SpawnAttributes {
            module: module.into(),
            function: function.into(),
            args: vec![],
            parent: None,
            priority: 0,
        }
    }

    /// Set arguments.
    pub fn with_args(mut self, args: Vec<BeamType>) -> Self {
        self.args = args;
        self
    }

    /// Set parent PID.
    pub fn with_parent(mut self, pid: u64) -> Self {
        self.parent = Some(pid);
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, p: u8) -> Self {
        self.priority = p.min(3);
        self
    }
}

/// Supervisor specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorSpec {
    /// Strategy name.
    pub strategy: SupervisorStrategy,
    /// Restart intensity.
    pub intensity: u32,
    /// Period in seconds.
    pub period: u32,
    /// Child specifications.
    pub children: Vec<ChildSpec>,
}

impl SupervisorSpec {
    /// Create a new supervisor spec.
    pub fn new(strategy: SupervisorStrategy, intensity: u32, period: u32) -> Self {
        SupervisorSpec {
            strategy,
            intensity,
            period,
            children: vec![],
        }
    }

    /// Add a child specification.
    pub fn with_children(mut self, children: Vec<ChildSpec>) -> Self {
        self.children = children;
        self
    }
}

/// Supervisor restart strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupervisorStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
    SimpleOneForOne,
}

impl Default for SupervisorStrategy {
    fn default() -> Self {
        SupervisorStrategy::OneForOne
    }
}

/// Child specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildSpec {
    /// Child ID.
    pub id: String,
    /// Start module.
    pub module: String,
    /// Start function.
    pub function: String,
    /// Restart strategy.
    pub restart: RestartKind,
    /// Shutdown timeout in milliseconds.
    pub shutdown: u32,
    /// Child type.
    pub child_type: ChildType,
    /// Modules for code version tracking.
    pub modules: Vec<String>,
}

impl ChildSpec {
    /// Create a new child spec.
    pub fn new(
        id: impl Into<String>,
        module: impl Into<String>,
        function: impl Into<String>,
        restart: RestartKind,
    ) -> Self {
        ChildSpec {
            id: id.into(),
            module: module.into(),
            function: function.into(),
            restart,
            shutdown: 5000, // default 5 seconds
            child_type: ChildType::Worker,
            modules: vec![],
        }
    }
}

/// Restart kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartKind {
    Permanent,
    Temporary,
    Transient,
}

impl Default for RestartKind {
    fn default() -> Self {
        RestartKind::Permanent
    }
}

/// Child type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChildType {
    Worker,
    Supervisor,
}

impl Default for ChildType {
    fn default() -> Self {
        ChildType::Worker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_op() {
        let op = BeamOp::spawn("module".to_string(), "function".to_string());
        assert_eq!(op.kind, BeamOpKind::Spawn);
        assert_eq!(op.op_name(), "beam.spawn");
        assert_eq!(op.outputs.len(), 1);
    }

    #[test]
    fn test_spawn_link_op() {
        let op = BeamOp::spawn_link("mod".to_string(), "fun".to_string());
        assert_eq!(op.kind, BeamOpKind::SpawnLink);
    }

    #[test]
    fn test_spawn_monitor_op() {
        let op = BeamOp::spawn_monitor("mod".to_string(), "fun".to_string());
        assert_eq!(op.kind, BeamOpKind::SpawnMonitor);
        assert_eq!(op.outputs.len(), 2); // pid and ref
    }

    #[test]
    fn test_send_op() {
        let op = BeamOp::send(BeamType::pid(), BeamType::atom());
        assert_eq!(op.kind, BeamOpKind::Send);
        assert_eq!(op.inputs.len(), 2);
    }

    #[test]
    fn test_receive_op() {
        let op = BeamOp::receive(3);
        assert_eq!(op.kind, BeamOpKind::Recv);
        assert_eq!(op.regions, 3);
    }

    #[test]
    fn test_link_op() {
        let op = BeamOp::link(BeamType::pid());
        assert_eq!(op.kind, BeamOpKind::Link);
        assert!(op.outputs.is_empty());
    }

    #[test]
    fn test_monitor_op() {
        let op = BeamOp::monitor(BeamType::pid());
        assert_eq!(op.kind, BeamOpKind::Monitor);
        assert_eq!(op.outputs.len(), 1);
    }

    #[test]
    fn test_exit_op() {
        let op = BeamOp::exit(BeamType::atom());
        assert_eq!(op.kind, BeamOpKind::Exit);
    }

    #[test]
    fn test_register_op() {
        let op = BeamOp::register(BeamType::atom(), BeamType::pid());
        assert_eq!(op.kind, BeamOpKind::Register);
    }

    #[test]
    fn test_whereis_op() {
        let op = BeamOp::whereis(BeamType::atom());
        assert_eq!(op.kind, BeamOpKind::Whereis);
    }

    #[test]
    fn test_receive_clause() {
        let clause = ReceiveClause::new(BeamType::atom(), "handler1");
        assert_eq!(clause.handler, "handler1");
    }

    #[test]
    fn test_spawn_attributes() {
        let attrs = SpawnAttributes::new("mod", "fun")
            .with_args(vec![BeamType::tuple(0)])
            .with_parent(100)
            .with_priority(2);
        assert_eq!(attrs.module, "mod");
        assert_eq!(attrs.args.len(), 1);
        assert_eq!(attrs.parent, Some(100));
        assert_eq!(attrs.priority, 2);
    }

    #[test]
    fn test_supervisor_spec() {
        let spec = SupervisorSpec::new(SupervisorStrategy::OneForOne, 3, 5);
        assert_eq!(spec.strategy, SupervisorStrategy::OneForOne);
        assert_eq!(spec.intensity, 3);
        assert_eq!(spec.period, 5);
    }

    #[test]
    fn test_child_spec() {
        let child = ChildSpec::new("child1", "mod", "fun", RestartKind::Permanent);
        assert_eq!(child.id, "child1");
        assert_eq!(child.restart, RestartKind::Permanent);
    }
}
