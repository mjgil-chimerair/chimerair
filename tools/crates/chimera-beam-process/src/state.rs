//! BEAM process state.
//!
//! Tracks the lifecycle state of a BEAM process: running, waiting,
//! suspended, exiting. Also handles priority levels.

use serde::{Deserialize, Serialize};

/// BEAM process execution state.

/// BEAM process execution state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessState {
    /// Process is actively executing.
    Running,
    /// Process is waiting for messages.
    Waiting,
    /// Process is suspended (e.g., debugging).
    Suspended,
    /// Process is exiting.
    Exiting,
    /// Process is in a receive clause.
    Receiving,
    /// Process is running a garbage collection.
    GarbageCollecting,
}

impl ProcessState {
    /// Check if the process can receive messages.
    pub fn can_receive(&self) -> bool {
        matches!(
            self,
            ProcessState::Running | ProcessState::Waiting | ProcessState::Receiving
        )
    }

    /// Check if the process is alive (not exiting).
    pub fn is_alive(&self) -> bool {
        !matches!(self, ProcessState::Exiting)
    }

    /// Check if the process should be scheduled.
    pub fn is_runnable(&self) -> bool {
        matches!(self, ProcessState::Running | ProcessState::Waiting)
    }
}

impl Default for ProcessState {
    fn default() -> Self {
        ProcessState::Running
    }
}

/// Process priority levels (BEAM default is normal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    /// Maximum priority (BEAM max priority).
    Max = 3,
    /// High priority.
    High = 2,
    /// Normal priority (default).
    Normal = 1,
    /// Low priority.
    Low = 0,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// Process flags and settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessFlags {
    /// Trap exit signals (convert exits to messages).
    pub traps_exit: bool,
    /// Process priority.
    pub priority: Priority,
    /// Enable error logger.
    pub error_logger: bool,
    /// Group leader for I/O.
    pub group_leader: Option<u64>,
}

impl ProcessFlags {
    /// Create default flags.
    pub fn new() -> Self {
        ProcessFlags {
            traps_exit: false,
            priority: Priority::default(),
            error_logger: true,
            group_leader: None,
        }
    }

    /// Set trap exit flag.
    pub fn with_trap_exit(mut self, enabled: bool) -> Self {
        self.traps_exit = enabled;
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Set group leader.
    pub fn with_group_leader(mut self, pid: u64) -> Self {
        self.group_leader = Some(pid);
        self
    }
}

/// Full process snapshot for debugging/introspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub pid: u64,
    pub state: ProcessState,
    pub flags: ProcessFlags,
    pub message_queue_len: usize,
    pub reductions: u64,
    pub heap_size: u32,
    pub stack_size: u32,
    pub current_function: Option<String>,
}

impl ProcessSnapshot {
    /// Create a snapshot from process info.
    pub fn from_process(pid: u64, state: ProcessState, flags: &ProcessFlags) -> Self {
        ProcessSnapshot {
            pid,
            state,
            flags: flags.clone(),
            message_queue_len: 0,
            reductions: 0,
            heap_size: 0,
            stack_size: 0,
            current_function: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_state_defaults() {
        let state = ProcessState::default();
        assert_eq!(state, ProcessState::Running);
    }

    #[test]
    fn test_process_state_can_receive() {
        assert!(ProcessState::Running.can_receive());
        assert!(ProcessState::Waiting.can_receive());
        assert!(ProcessState::Receiving.can_receive());
        assert!(!ProcessState::Exiting.can_receive());
        assert!(!ProcessState::Suspended.can_receive());
    }

    #[test]
    fn test_process_state_is_alive() {
        assert!(ProcessState::Running.is_alive());
        assert!(ProcessState::Waiting.is_alive());
        assert!(!ProcessState::Exiting.is_alive());
    }

    #[test]
    fn test_priority_order() {
        assert!(Priority::Max > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }

    #[test]
    fn test_priority_default() {
        let p = Priority::default();
        assert_eq!(p, Priority::Normal);
    }

    #[test]
    fn test_process_flags() {
        let flags = ProcessFlags::new()
            .with_trap_exit(true)
            .with_priority(Priority::High);
        assert!(flags.traps_exit);
        assert_eq!(flags.priority, Priority::High);
    }
}
