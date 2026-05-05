//! BEAM process model.
//!
//! Models BEAM lightweight processes with pid, mailbox, state,
//! linkage, monitoring, and registry integration.

pub mod link;
pub mod monitor;
pub mod pid;
pub mod registry;
pub mod spawn;
pub mod state;

pub use link::{Link, LinkHandle, LinkKind};
pub use monitor::{Monitor, MonitorRef, MonitorTarget};
pub use pid::{BeamPid, PidGenerator};
pub use registry::{ProcessRegistry, Registration, RegistrationKey};
pub use spawn::{ProcessInitializer, SpawnConfig, SpawnResult, Term};
pub use state::{Priority, ProcessState};

/// Maximum processes per node (BEAM default is ~16M but we cap lower for safety).
pub const MAX_PROCESSES: usize = 1_000_000;

/// Maximum links per process.
pub const MAX_LINKS_PER_PROCESS: usize = 65536;

/// Maximum monitors per process.
pub const MAX_MONITORS_PER_PROCESS: usize = 65536;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_generator() {
        let mut generator = PidGenerator::new(0);
        let pid1 = generator.next().unwrap();
        let pid2 = generator.next().unwrap();
        assert!(pid1 < pid2);
        assert_eq!(pid1.index(), 0);
        assert_eq!(pid2.index(), 1);
    }

    #[test]
    fn test_process_state_default() {
        let state = ProcessState::default();
        assert_eq!(state, ProcessState::Running);
    }

    #[test]
    fn test_priority_default() {
        let priority = Priority::default();
        assert_eq!(priority, Priority::Normal);
    }

    #[test]
    fn test_max_constants() {
        assert!(MAX_PROCESSES > 0);
        assert!(MAX_LINKS_PER_PROCESS > 0);
        assert!(MAX_MONITORS_PER_PROCESS > 0);
    }
}
