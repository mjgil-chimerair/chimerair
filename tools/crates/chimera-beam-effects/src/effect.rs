//! BEAM effect types.
//!
//! Defines the effect system for modeling BEAM observable behavior.

use serde::{Deserialize, Serialize};

/// Effect type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectType {
    /// Process spawn (spawn, spawn_link, spawn_monitor).
    ProcessSpawn,
    /// Link between processes (link, unlink).
    ProcessLink,
    /// Monitor a process (monitor, demonitor).
    ProcessMonitor,
    /// Exit signal (exit, exit2, kill).
    ProcessExit,
    /// Message send (!, send).
    MessageSend,
    /// Message receive (receive, recv_next).
    MessageReceive,
    /// Timer schedule (start_timer, after).
    TimerSchedule,
    /// Code loading (code:load_file, code_change).
    CodeLoad,
    /// Process registry (register, whereis, unregister).
    Registry,
    /// NIF call (erlang:nif_*).
    NifCall,
    /// Distribution (spawn on remote node).
    Distribution,
    /// Process info (self, process_info).
    ProcessInfo,
    /// Memory allocation (erts:alloc).
    MemoryAlloc,
}

impl EffectType {
    /// Get effect type name.
    pub fn as_str(&self) -> &'static str {
        match self {
            EffectType::ProcessSpawn => "process_spawn",
            EffectType::ProcessLink => "process_link",
            EffectType::ProcessMonitor => "process_monitor",
            EffectType::ProcessExit => "process_exit",
            EffectType::MessageSend => "message_send",
            EffectType::MessageReceive => "message_receive",
            EffectType::TimerSchedule => "timer_schedule",
            EffectType::CodeLoad => "code_load",
            EffectType::Registry => "registry",
            EffectType::NifCall => "nif_call",
            EffectType::Distribution => "distribution",
            EffectType::ProcessInfo => "process_info",
            EffectType::MemoryAlloc => "memory_alloc",
        }
    }

    /// Check if this effect may spawn a process.
    pub fn may_spawn(&self) -> bool {
        matches!(self, EffectType::ProcessSpawn)
    }

    /// Check if this effect may send a message.
    pub fn may_message(&self) -> bool {
        matches!(self, EffectType::MessageSend)
    }

    /// Check if this effect may receive a message.
    pub fn may_receive(&self) -> bool {
        matches!(self, EffectType::MessageReceive)
    }

    /// Check if this effect may schedule a timer.
    pub fn may_schedule(&self) -> bool {
        matches!(self, EffectType::TimerSchedule)
    }

    /// Check if this effect may link to another process.
    pub fn may_link(&self) -> bool {
        matches!(self, EffectType::ProcessLink)
    }

    /// Check if this effect may exit a process.
    pub fn may_exit(&self) -> bool {
        matches!(self, EffectType::ProcessExit)
    }
}

/// Effect severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectSeverity {
    /// Pure computation, no side effects.
    Pure,
    /// May have side effects but contained.
    Local,
    /// May affect other processes or external state.
    Global,
    /// May fail or crash.
    Critical,
}

impl Default for EffectSeverity {
    fn default() -> Self {
        EffectSeverity::Local
    }
}

impl EffectSeverity {
    /// Get severity level.
    pub fn as_str(&self) -> &'static str {
        match self {
            EffectSeverity::Pure => "pure",
            EffectSeverity::Local => "local",
            EffectSeverity::Global => "global",
            EffectSeverity::Critical => "critical",
        }
    }

    /// Check if this severity is safe for optimization.
    pub fn is_safe(&self) -> bool {
        matches!(self, EffectSeverity::Pure)
    }

    /// Check if this severity requires synchronization.
    pub fn requires_sync(&self) -> bool {
        matches!(self, EffectSeverity::Global | EffectSeverity::Critical)
    }
}

/// Location in source code where effect occurs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectLocation {
    /// Module name.
    pub module: String,
    /// Function name.
    pub function: String,
    /// Line number (0 if unknown).
    pub line: u32,
    /// Column number (0 if unknown).
    pub column: u32,
}

impl EffectLocation {
    /// Create a new effect location.
    pub fn new(
        module: impl Into<String>,
        function: impl Into<String>,
        line: u32,
        column: u32,
    ) -> Self {
        EffectLocation {
            module: module.into(),
            function: function.into(),
            line,
            column,
        }
    }

    /// Create with unknown location.
    pub fn unknown() -> Self {
        EffectLocation {
            module: String::new(),
            function: String::new(),
            line: 0,
            column: 0,
        }
    }

    /// Check if location is known.
    pub fn is_unknown(&self) -> bool {
        self.module.is_empty() && self.function.is_empty()
    }
}

impl Default for EffectLocation {
    fn default() -> Self {
        Self::unknown()
    }
}

/// Effect information with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectInfo {
    /// Effect type.
    pub effect_type: EffectType,
    /// Severity level.
    pub severity: EffectSeverity,
    /// Location in source.
    pub location: EffectLocation,
    /// Target process or resource (if applicable).
    pub target: Option<String>,
    /// Description of the effect.
    pub description: String,
    /// Tags for filtering (e.g., "may_spawn", "may_message").
    pub tags: Vec<String>,
}

impl EffectInfo {
    /// Create a new effect info.
    pub fn new(
        effect_type: EffectType,
        severity: EffectSeverity,
        location: EffectLocation,
    ) -> Self {
        EffectInfo {
            effect_type,
            severity,
            location,
            target: None,
            description: String::new(),
            tags: vec![],
        }
    }

    /// Create with description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set target.
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Add a tag.
    pub fn add_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Build from effect type at location.
    pub fn at(effect_type: EffectType, location: EffectLocation) -> Self {
        let severity = match effect_type {
            EffectType::ProcessSpawn => EffectSeverity::Global,
            EffectType::ProcessLink => EffectSeverity::Global,
            EffectType::ProcessMonitor => EffectSeverity::Local,
            EffectType::ProcessExit => EffectSeverity::Critical,
            EffectType::MessageSend => EffectSeverity::Local,
            EffectType::MessageReceive => EffectSeverity::Local,
            EffectType::TimerSchedule => EffectSeverity::Local,
            EffectType::CodeLoad => EffectSeverity::Global,
            EffectType::Registry => EffectSeverity::Global,
            EffectType::NifCall => EffectSeverity::Critical,
            EffectType::Distribution => EffectSeverity::Global,
            EffectType::ProcessInfo => EffectSeverity::Local,
            EffectType::MemoryAlloc => EffectSeverity::Local,
        };

        EffectInfo {
            effect_type,
            severity,
            location,
            target: None,
            description: format!("{:?}", effect_type),
            tags: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_type_as_str() {
        assert_eq!(EffectType::ProcessSpawn.as_str(), "process_spawn");
        assert_eq!(EffectType::MessageSend.as_str(), "message_send");
        assert_eq!(EffectType::Registry.as_str(), "registry");
    }

    #[test]
    fn test_effect_type_helpers() {
        assert!(EffectType::ProcessSpawn.may_spawn());
        assert!(!EffectType::MessageSend.may_spawn());
        assert!(EffectType::MessageSend.may_message());
        assert!(EffectType::MessageReceive.may_receive());
        assert!(EffectType::TimerSchedule.may_schedule());
        assert!(EffectType::ProcessLink.may_link());
        assert!(EffectType::ProcessExit.may_exit());
    }

    #[test]
    fn test_effect_severity_as_str() {
        assert_eq!(EffectSeverity::Pure.as_str(), "pure");
        assert_eq!(EffectSeverity::Local.as_str(), "local");
        assert_eq!(EffectSeverity::Global.as_str(), "global");
        assert_eq!(EffectSeverity::Critical.as_str(), "critical");
    }

    #[test]
    fn test_effect_severity_helpers() {
        assert!(EffectSeverity::Pure.is_safe());
        assert!(!EffectSeverity::Local.is_safe());
        assert!(EffectSeverity::Global.requires_sync());
        assert!(EffectSeverity::Critical.requires_sync());
    }

    #[test]
    fn test_effect_location() {
        let loc = EffectLocation::new("mod", "fun", 10, 5);
        assert_eq!(loc.module, "mod");
        assert_eq!(loc.function, "fun");
        assert_eq!(loc.line, 10);
        assert!(!loc.is_unknown());
    }

    #[test]
    fn test_effect_location_unknown() {
        let loc = EffectLocation::unknown();
        assert!(loc.is_unknown());
    }

    #[test]
    fn test_effect_info_new() {
        let loc = EffectLocation::new("mod", "fun", 1, 1);
        let info = EffectInfo::new(EffectType::MessageSend, EffectSeverity::Local, loc);
        assert_eq!(info.effect_type, EffectType::MessageSend);
    }

    #[test]
    fn test_effect_info_builder() {
        let loc = EffectLocation::new("mod", "fun", 1, 1);
        let info = EffectInfo::at(EffectType::ProcessSpawn, loc)
            .with_description("spawns a process")
            .with_target("pid")
            .add_tag("may_spawn");
        assert_eq!(info.effect_type, EffectType::ProcessSpawn);
        assert_eq!(info.target, Some("pid".to_string()));
        assert!(info.tags.contains(&"may_spawn".to_string()));
    }

    #[test]
    fn test_effect_info_at() {
        let loc = EffectLocation::new("mod", "fun", 1, 1);
        let info = EffectInfo::at(EffectType::ProcessExit, loc);
        assert_eq!(info.severity, EffectSeverity::Critical);
    }
}
