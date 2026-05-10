//! BEAM process ID generation.
//!
//! BEAM pids are unique within a node. We generate sequential pids
//! with a node suffix for distributed scenarios.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PidError {
    #[error("PID overflow: max {0} processes exceeded")]
    Overflow(usize),
    #[error("invalid PID format: {0}")]
    InvalidFormat(String),
}

pub type PidResult<T> = Result<T, PidError>;

/// A BEAM process identifier.
///
/// Pids are structured as: (index, serial, node)
/// Where index is the slot in the process table, serial is a counter
/// for reused slots, and node identifies the BEAM node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BeamPid(pub u64, pub u64, pub u64);

impl BeamPid {
    /// Create a new process ID from components.
    pub fn new(index: u64, serial: u64, node: u64) -> Self {
        BeamPid(index, serial, node)
    }

    /// Get the index component (slot in process table).
    pub fn index(&self) -> u64 {
        self.0
    }

    /// Get the serial component (for slot reuse detection).
    pub fn serial(&self) -> u64 {
        self.1
    }

    /// Get the node component.
    pub fn node(&self) -> u64 {
        self.2
    }

    /// Check if this pid refers to the current process.
    #[allow(unused)]
    pub fn is_self(&self) -> bool {
        false // Would need process-local context
    }

    /// Convert to u64 representation (index only for simplicity).
    pub fn to_u64(&self) -> u64 {
        self.0
    }

    /// Create from u64.
    pub fn from_u64(v: u64) -> Self {
        BeamPid(v, 0, 0)
    }
}

impl std::fmt::Display for BeamPid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}.{}.{}>", self.0, self.1, self.2)
    }
}

impl PartialOrd for BeamPid {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BeamPid {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

/// PID generator for creating new process IDs.
#[derive(Debug, Clone)]
pub struct PidGenerator {
    next_index: u64,
    next_serial: u64,
    node: u64,
    max_processes: usize,
}

impl PidGenerator {
    /// Create a new PID generator.
    pub fn new(node: u64) -> Self {
        PidGenerator {
            next_index: 0,
            next_serial: 0,
            node,
            max_processes: super::MAX_PROCESSES,
        }
    }

    /// Create a generator with custom max processes.
    #[allow(unused)]
    pub fn with_max(node: u64, max: usize) -> Self {
        PidGenerator {
            next_index: 0,
            next_serial: 0,
            node,
            max_processes: max,
        }
    }

    /// Generate the next PID.
    pub fn next(&mut self) -> PidResult<BeamPid> {
        if self.next_index >= self.max_processes as u64 {
            return Err(PidError::Overflow(self.max_processes));
        }

        let pid = BeamPid(self.next_index, self.next_serial, self.node);
        self.next_index += 1;

        // Every ~16M processes, the serial wraps (BEAM behavior)
        if self.next_index % (1 << 20) == 0 {
            self.next_serial = self.next_serial.wrapping_add(1) & 0xFF;
        }

        Ok(pid)
    }

    /// Generate a PID for a specific slot (for recycled PIDs).
    #[allow(unused)]
    pub fn recycled(&mut self, index: u64, serial: u64) -> PidResult<BeamPid> {
        if index >= self.max_processes as u64 {
            return Err(PidError::Overflow(self.max_processes));
        }

        // Only update next_index if this is beyond current
        if index >= self.next_index {
            self.next_index = index + 1;
        }

        Ok(BeamPid(index, serial, self.node))
    }

    /// Current serial number for a given index.
    #[allow(unused)]
    pub fn serial_for(&self, _index: u64) -> u64 {
        // This would need to track serials per index in a real implementation
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_creation() {
        let pid = BeamPid::new(123, 456, 0);
        assert_eq!(pid.index(), 123);
        assert_eq!(pid.serial(), 456);
        assert_eq!(pid.node(), 0);
    }

    #[test]
    fn test_pid_display() {
        let pid = BeamPid::new(1, 2, 3);
        assert_eq!(format!("{}", pid), "<1.2.3>");
    }

    #[test]
    fn test_pid_generator_next() {
        let mut gen = PidGenerator::new(0);
        let pid1 = gen.next().unwrap();
        let pid2 = gen.next().unwrap();
        assert!(pid1 < pid2);
    }

    #[test]
    fn test_pid_generator_overflow() {
        let mut gen = PidGenerator::with_max(0, 2);
        let _ = gen.next().unwrap();
        let _ = gen.next().unwrap();
        let result = gen.next();
        assert!(matches!(result, Err(PidError::Overflow(_))));
    }

    #[test]
    fn test_pid_to_u64() {
        let pid = BeamPid::new(42, 0, 0);
        assert_eq!(pid.to_u64(), 42);
    }

    #[test]
    fn test_pid_from_u64() {
        let pid = BeamPid::from_u64(99);
        assert_eq!(pid.to_u64(), 99);
    }
}
