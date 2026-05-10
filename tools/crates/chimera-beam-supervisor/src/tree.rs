//! Supervisor tree structure.
//!
//! Models the hierarchical supervision tree with parent-child relationships.

use chimera_beam_process::BeamPid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::child::ChildSpec;
use super::strategy::{RestartIntensity, RestartStrategy};
use super::MAX_CHILDREN_PER_SUPERVISOR;

/// A node in the supervisor tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorNode {
    /// Node ID (supervisor name).
    pub id: String,
    /// Strategy for this supervisor.
    pub strategy: RestartStrategy,
    /// Intensity configuration.
    pub intensity: RestartIntensity,
    /// Child specifications.
    pub children: Vec<ChildSpec>,
    /// Active child PIDs (pid -> child_id).
    pub active_children: HashMap<String, BeamPid>,
    /// Restart count within current period.
    pub restart_count: u32,
    /// Period start timestamp (for intensity tracking).
    pub period_start: u64,
}

impl SupervisorNode {
    /// Create a new supervisor node.
    pub fn new(
        id: impl Into<String>,
        strategy: RestartStrategy,
        intensity: RestartIntensity,
    ) -> Self {
        SupervisorNode {
            id: id.into(),
            strategy,
            intensity,
            children: vec![],
            active_children: HashMap::new(),
            restart_count: 0,
            period_start: 0,
        }
    }

    /// Add a child specification.
    pub fn add_child(&mut self, spec: ChildSpec) -> Result<(), super::error::SupervisorError> {
        if self.children.len() >= MAX_CHILDREN_PER_SUPERVISOR {
            return Err(super::error::SupervisorError::TooManyChildren);
        }
        self.children.push(spec);
        Ok(())
    }

    /// Remove a child by ID.
    pub fn remove_child(&mut self, child_id: &str) -> Option<ChildSpec> {
        if let Some(pos) = self.children.iter().position(|c| c.id == child_id) {
            let spec = self.children.remove(pos);
            self.active_children.remove(child_id);
            Some(spec)
        } else {
            None
        }
    }

    /// Get a child by ID.
    pub fn get_child(&self, child_id: &str) -> Option<&ChildSpec> {
        self.children.iter().find(|c| c.id == child_id)
    }

    /// Check if a child is active (has a PID).
    pub fn is_child_active(&self, child_id: &str) -> bool {
        self.active_children.contains_key(child_id)
    }

    /// Register a child's PID.
    pub fn register_child(&mut self, child_id: String, pid: BeamPid) {
        self.active_children.insert(child_id, pid);
    }

    /// Unregister a child's PID.
    pub fn unregister_child(&mut self, child_id: &str) -> Option<BeamPid> {
        self.active_children.remove(child_id)
    }

    /// Get all active child IDs.
    pub fn active_child_ids(&self) -> Vec<String> {
        self.active_children.keys().cloned().collect()
    }

    /// Check if within restart intensity.
    pub fn is_within_intensity(&self) -> bool {
        self.intensity.is_within_bounds(self.restart_count)
    }

    /// Increment restart count.
    pub fn increment_restart_count(&mut self) {
        self.restart_count += 1;
    }

    /// Reset restart count.
    pub fn reset_restart_count(&mut self) {
        self.restart_count = 0;
        self.period_start = current_time_ms();
    }

    /// Get restart count.
    pub fn restart_count(&self) -> u32 {
        self.restart_count
    }
}

/// A child node (wrapper for supervisor tree integration).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildNode {
    /// Child specification.
    pub spec: ChildSpec,
    /// Current PID (None if not started).
    pub pid: Option<BeamPid>,
    /// Last exit reason (if terminated).
    pub last_exit: Option<String>,
    /// Start count (for simple_one_for_one).
    pub start_count: u32,
}

impl ChildNode {
    /// Create a new child node.
    pub fn new(spec: ChildSpec) -> Self {
        ChildNode {
            spec,
            pid: None,
            last_exit: None,
            start_count: 0,
        }
    }

    /// Check if child is running.
    pub fn is_running(&self) -> bool {
        self.pid.is_some()
    }

    /// Mark child as started.
    pub fn mark_started(&mut self, pid: BeamPid) {
        self.pid = Some(pid);
        self.start_count += 1;
        self.last_exit = None;
    }

    /// Mark child as terminated.
    pub fn mark_terminated(&mut self, exit_reason: Option<String>) {
        self.pid = None;
        self.last_exit = exit_reason;
    }

    /// Get the PID if running.
    pub fn pid(&self) -> Option<BeamPid> {
        self.pid
    }
}

/// Full supervisor tree with hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorTree {
    /// Root supervisor.
    pub root: SupervisorNode,
    /// Child supervisors (nested).
    pub children: HashMap<String, SupervisorTree>,
}

impl SupervisorTree {
    /// Create a new supervisor tree.
    pub fn new(root: SupervisorNode) -> Self {
        SupervisorTree {
            root,
            children: HashMap::new(),
        }
    }

    /// Create with a root supervisor using default intensity.
    pub fn with_strategy(id: impl Into<String>, strategy: RestartStrategy) -> Self {
        let intensity = RestartIntensity::default_intensity();
        SupervisorTree::new(SupervisorNode::new(id, strategy, intensity))
    }

    /// Add a child supervisor.
    pub fn add_child_supervisor(
        &mut self,
        child: SupervisorTree,
    ) -> Result<(), super::error::SupervisorError> {
        let child_id = child.root.id.clone();
        if self.children.len() >= MAX_CHILDREN_PER_SUPERVISOR {
            return Err(super::error::SupervisorError::TooManyChildren);
        }
        // Check that child ID doesn't already exist
        if self.root.get_child(&child_id).is_some() {
            return Err(super::error::SupervisorError::ChildAlreadyExists);
        }
        self.children.insert(child_id.clone(), child);
        Ok(())
    }

    /// Get a child supervisor by ID.
    pub fn get_child_supervisor(&self, id: &str) -> Option<&SupervisorTree> {
        self.children.get(id)
    }

    /// Get a mutable child supervisor by ID.
    pub fn get_child_supervisor_mut(&mut self, id: &str) -> Option<&mut SupervisorTree> {
        self.children.get_mut(id)
    }

    /// Remove a child supervisor by ID.
    pub fn remove_child_supervisor(&mut self, id: &str) -> Option<SupervisorTree> {
        self.children.remove(id)
    }

    /// Get the root supervisor.
    pub fn root_supervisor(&self) -> &SupervisorNode {
        &self.root
    }

    /// Get a mutable reference to the root supervisor.
    pub fn root_supervisor_mut(&mut self) -> &mut SupervisorNode {
        &mut self.root
    }

    /// Get all child supervisors.
    pub fn child_supervisors(&self) -> &HashMap<String, SupervisorTree> {
        &self.children
    }

    /// Get total number of children (direct and nested).
    pub fn total_children(&self) -> usize {
        let direct = self.root.children.len();
        let nested: usize = self.children.values().map(|c| c.total_children()).sum();
        direct + nested
    }
}

/// Get current time in milliseconds (stub for now).
fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::super::strategy::RestartKind;
    use super::*;

    #[test]
    fn test_supervisor_node_new() {
        let node = SupervisorNode::new(
            "test_sup",
            RestartStrategy::OneForOne,
            RestartIntensity::default(),
        );
        assert_eq!(node.id, "test_sup");
        assert_eq!(node.strategy, RestartStrategy::OneForOne);
        assert!(node.children.is_empty());
        assert!(node.active_children.is_empty());
    }

    #[test]
    fn test_supervisor_node_add_child() {
        let mut node = SupervisorNode::new(
            "sup",
            RestartStrategy::OneForOne,
            RestartIntensity::default(),
        );
        let spec = ChildSpec::worker("child1", "mod", "fun");
        assert!(node.add_child(spec).is_ok());
        assert_eq!(node.children.len(), 1);
    }

    #[test]
    fn test_supervisor_node_remove_child() {
        let mut node = SupervisorNode::new(
            "sup",
            RestartStrategy::OneForOne,
            RestartIntensity::default(),
        );
        let spec = ChildSpec::worker("child1", "mod", "fun");
        node.add_child(spec).unwrap();

        let removed = node.remove_child("child1");
        assert!(removed.is_some());
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_supervisor_node_is_child_active() {
        let mut node = SupervisorNode::new(
            "sup",
            RestartStrategy::OneForOne,
            RestartIntensity::default(),
        );
        let spec = ChildSpec::worker("child1", "mod", "fun");
        node.add_child(spec).unwrap();

        assert!(!node.is_child_active("child1"));
        node.register_child("child1".to_string(), BeamPid::new(1, 1, 0));
        assert!(node.is_child_active("child1"));
    }

    #[test]
    fn test_supervisor_node_within_intensity() {
        let mut node = SupervisorNode::new(
            "sup",
            RestartStrategy::OneForOne,
            RestartIntensity::new(3, 5),
        );
        assert!(node.is_within_intensity());
        node.increment_restart_count();
        node.increment_restart_count();
        assert!(node.is_within_intensity());
        node.increment_restart_count();
        assert!(!node.is_within_intensity());
    }

    #[test]
    fn test_child_node_new() {
        let spec = ChildSpec::worker("child1", "mod", "fun");
        let node = ChildNode::new(spec);
        assert!(!node.is_running());
        assert!(node.pid().is_none());
    }

    #[test]
    fn test_child_node_mark_started() {
        let spec = ChildSpec::worker("child1", "mod", "fun");
        let mut node = ChildNode::new(spec);
        node.mark_started(BeamPid::new(1, 1, 0));

        assert!(node.is_running());
        assert!(node.pid().is_some());
        assert_eq!(node.start_count, 1);
    }

    #[test]
    fn test_child_node_mark_terminated() {
        let spec = ChildSpec::worker("child1", "mod", "fun");
        let mut node = ChildNode::new(spec);
        node.mark_started(BeamPid::new(1, 1, 0));
        node.mark_terminated(Some("normal".to_string()));

        assert!(!node.is_running());
        assert_eq!(node.last_exit, Some("normal".to_string()));
    }

    #[test]
    fn test_supervisor_tree_new() {
        let root = SupervisorNode::new(
            "top",
            RestartStrategy::OneForOne,
            RestartIntensity::default(),
        );
        let tree = SupervisorTree::new(root);
        assert_eq!(tree.root.id, "top");
        assert!(tree.children.is_empty());
    }

    #[test]
    fn test_supervisor_tree_with_strategy() {
        let tree = SupervisorTree::with_strategy("top", RestartStrategy::OneForAll);
        assert_eq!(tree.root.strategy, RestartStrategy::OneForAll);
    }

    #[test]
    fn test_supervisor_tree_total_children() {
        let mut root = SupervisorNode::new(
            "top",
            RestartStrategy::OneForOne,
            RestartIntensity::default(),
        );
        root.add_child(ChildSpec::worker("child1", "mod", "fun"))
            .unwrap();
        root.add_child(ChildSpec::worker("child2", "mod", "fun"))
            .unwrap();

        let mut tree = SupervisorTree::new(root);

        // Child supervisor has no children
        let child_sup = SupervisorTree::with_strategy("child_sup", RestartStrategy::OneForOne);
        tree.add_child_supervisor(child_sup).unwrap();

        // 2 direct children in root, child_sup has 0
        assert_eq!(tree.total_children(), 2);

        // Add a child to child_sup
        let mut child_sup = tree.remove_child_supervisor("child_sup").unwrap();
        child_sup
            .root
            .add_child(ChildSpec::worker("nested_child", "mod", "fun"))
            .unwrap();
        tree.add_child_supervisor(child_sup).unwrap();

        // Now 2 in root + 1 in child_sup = 3
        assert_eq!(tree.total_children(), 3);
    }
}
