//! BEAM process monitors.
//!
//! Unidirectional monitors: unlike links, monitors don't prevent the
//! target process from dying. When the monitored process exits, the
//! monitoring process receives a 'DOWN' message.

use serde::{Deserialize, Serialize};

/// Target of a monitor (either a PID or a registered name).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonitorTarget {
    /// Monitor a specific process.
    Pid(u64),
    /// Monitor a registered process by name.
    Name(String),
}

impl MonitorTarget {
    /// Check if this target is a PID.
    pub fn is_pid(&self) -> bool {
        matches!(self, MonitorTarget::Pid(_))
    }

    /// Check if this target is a name.
    pub fn is_name(&self) -> bool {
        matches!(self, MonitorTarget::Name(_))
    }

    /// Get the PID if this is a PID target.
    pub fn as_pid(&self) -> Option<u64> {
        match self {
            MonitorTarget::Pid(pid) => Some(*pid),
            _ => None,
        }
    }

    /// Get the name if this is a name target.
    pub fn as_name(&self) -> Option<&str> {
        match self {
            MonitorTarget::Name(name) => Some(name),
            _ => None,
        }
    }
}

/// A monitor reference for tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MonitorRef(u64);

impl MonitorRef {
    /// Create a new monitor reference.
    pub fn new(id: u64) -> Self {
        MonitorRef(id)
    }

    /// Get the monitor ID.
    pub fn id(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for MonitorRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#Ref<{}>", self.0)
    }
}

/// Monitor reference generator.
#[derive(Debug, Clone)]
pub struct MonitorRefGenerator {
    next_ref: u64,
}

impl MonitorRefGenerator {
    /// Create a new generator.
    pub fn new() -> Self {
        MonitorRefGenerator { next_ref: 0 }
    }

    /// Generate the next monitor reference.
    pub fn next(&mut self) -> MonitorRef {
        let ref_id = self.next_ref;
        self.next_ref = self.next_ref.wrapping_add(1);
        MonitorRef::new(ref_id)
    }

    /// Reset the generator.
    pub fn reset(&mut self) {
        self.next_ref = 0;
    }
}

impl Default for MonitorRefGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Monitor entry in a process's monitor table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    /// Reference for this monitor.
    pub reference: MonitorRef,
    /// Target being monitored.
    pub target: MonitorTarget,
    /// Origin PID (who is monitoring).
    pub origin: u64,
    /// When the monitor was created.
    pub created_at: u64,
}

impl Monitor {
    /// Create a new monitor.
    pub fn new(reference: MonitorRef, target: MonitorTarget, origin: u64, created_at: u64) -> Self {
        Monitor {
            reference,
            target,
            origin,
            created_at,
        }
    }
}

/// Monitor table for tracking all monitors from a process.
#[derive(Debug, Clone, Default)]
pub struct MonitorTable {
    /// Monitors from this process.
    monitors: Vec<Monitor>,
}

impl MonitorTable {
    /// Create a new monitor table.
    pub fn new() -> Self {
        MonitorTable {
            monitors: Vec::new(),
        }
    }

    /// Add a monitor.
    pub fn add_monitor(&mut self, monitor: Monitor) {
        self.monitors.push(monitor);
    }

    /// Remove a monitor by reference.
    pub fn remove_monitor(&mut self, reference: MonitorRef) -> bool {
        let idx = self.monitors.iter().position(|m| m.reference == reference);
        if let Some(idx) = idx {
            self.monitors.swap_remove(idx);
            true
        } else {
            false
        }
    }

    /// Find monitor by reference.
    pub fn find(&self, reference: MonitorRef) -> Option<&Monitor> {
        self.monitors.iter().find(|m| m.reference == reference)
    }

    /// Get all monitors for a target.
    pub fn for_target(&self, target: &MonitorTarget) -> Vec<&Monitor> {
        self.monitors
            .iter()
            .filter(|m| &m.target == target)
            .collect()
    }

    /// Number of monitors.
    pub fn len(&self) -> usize {
        self.monitors.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.monitors.is_empty()
    }

    /// Clear all monitors.
    pub fn clear(&mut self) {
        self.monitors.clear();
    }
}

/// DOWN message sent when a monitored process exits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownMessage {
    /// Monitor reference.
    pub reference: MonitorRef,
    /// Monitored PID or name.
    pub target: MonitorTarget,
    /// Exit reason.
    pub reason: ExitReason,
    /// Origin PID (who received the DOWN).
    pub origin: u64,
}

/// Exit reason (reused from link module).
pub use super::link::ExitReason;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_target_pid() {
        let target = MonitorTarget::Pid(123);
        assert!(target.is_pid());
        assert!(!target.is_name());
        assert_eq!(target.as_pid(), Some(123));
        assert_eq!(target.as_name(), None);
    }

    #[test]
    fn test_monitor_target_name() {
        let target = MonitorTarget::Name("my_process".to_string());
        assert!(!target.is_pid());
        assert!(target.is_name());
        assert_eq!(target.as_pid(), None);
        assert_eq!(target.as_name(), Some("my_process"));
    }

    #[test]
    fn test_monitor_ref() {
        let ref_id = MonitorRef::new(999);
        assert_eq!(ref_id.id(), 999);
        assert_eq!(format!("{}", ref_id), "#Ref<999>");
    }

    #[test]
    fn test_monitor_ref_generator() {
        let mut gen = MonitorRefGenerator::new();
        let ref1 = gen.next();
        let ref2 = gen.next();
        assert!(ref1 < ref2);
        assert_eq!(ref1.id(), 0);
        assert_eq!(ref2.id(), 1);
    }

    #[test]
    fn test_monitor_table() {
        let mut table = MonitorTable::new();
        let ref_gen = &mut MonitorRefGenerator::new();

        table.add_monitor(Monitor::new(
            ref_gen.next(),
            MonitorTarget::Pid(100),
            1,
            1000,
        ));
        table.add_monitor(Monitor::new(
            ref_gen.next(),
            MonitorTarget::Name("other".to_string()),
            1,
            1001,
        ));

        assert_eq!(table.len(), 2);
        assert!(!table.is_empty());

        // Find by reference
        let ref1 = MonitorRef::new(0);
        let found = table.find(ref1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().target, MonitorTarget::Pid(100));
    }

    #[test]
    fn test_down_message() {
        let msg = DownMessage {
            reference: MonitorRef::new(42),
            target: MonitorTarget::Pid(123),
            reason: ExitReason::Abnormal,
            origin: 1,
        };
        assert_eq!(msg.reference.id(), 42);
        assert!(matches!(msg.target, MonitorTarget::Pid(123)));
        assert!(matches!(msg.reason, ExitReason::Abnormal));
    }
}
