//! Ownership tracking for BEAM values.
//!
//! Tracks ownership references throughout the program lifecycle.

use super::categories::{HeapOwnership, OwnershipCategory, ProcessOwnership};
use chimera_beam_process::BeamPid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An ownership reference.
#[derive(Debug, Clone)]
pub struct OwnershipRef {
    /// Reference ID.
    pub id: u64,
    /// Referenced PID (if process reference).
    pub pid: Option<BeamPid>,
    /// Address (if heap reference).
    pub address: Option<u64>,
    /// Category.
    pub category: OwnershipCategory,
    /// Creation timestamp.
    pub created_at: u64,
}

impl OwnershipRef {
    /// Create a new reference.
    pub fn new(id: u64, category: OwnershipCategory) -> Self {
        OwnershipRef {
            id,
            pid: None,
            address: None,
            category,
            created_at: current_time_ms(),
        }
    }

    /// Create for a process.
    pub fn for_process(pid: BeamPid) -> Self {
        OwnershipRef {
            id: 0,
            pid: Some(pid),
            address: None,
            category: OwnershipCategory::Owned,
            created_at: current_time_ms(),
        }
    }

    /// Create for a heap address.
    pub fn for_heap(address: u64) -> Self {
        OwnershipRef {
            id: 0,
            pid: None,
            address: Some(address),
            category: OwnershipCategory::Owned,
            created_at: current_time_ms(),
        }
    }

    /// Check if this is a process reference.
    pub fn is_process_ref(&self) -> bool {
        self.pid.is_some()
    }

    /// Check if this is a heap reference.
    pub fn is_heap_ref(&self) -> bool {
        self.address.is_some()
    }
}

/// Ownership tracker for BEAM values.
#[derive(Debug, Clone)]
pub struct OwnershipTracker {
    /// Next reference ID.
    next_ref_id: u64,
    /// Process ownership map.
    process_ownership: HashMap<u64, ProcessOwnership>,
    /// Heap ownership map.
    heap_ownership: HashMap<u64, HeapOwnership>,
    /// Active references.
    references: HashMap<u64, OwnershipRef>,
    /// Reference by owner (pid -> ref_ids).
    refs_by_owner: HashMap<u64, Vec<u64>>,
}

impl OwnershipTracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        OwnershipTracker {
            next_ref_id: 1,
            process_ownership: HashMap::new(),
            heap_ownership: HashMap::new(),
            references: HashMap::new(),
            refs_by_owner: HashMap::new(),
        }
    }

    /// Get next reference ID.
    fn next_id(&mut self) -> u64 {
        let id = self.next_ref_id;
        self.next_ref_id += 1;
        id
    }

    /// Register process ownership.
    pub fn register_process(&mut self, pid: BeamPid) -> OwnershipRef {
        let id = self.next_id();
        let ownership = ProcessOwnership::new(pid);

        self.process_ownership.insert(pid.to_u64(), ownership);

        let mut reference = OwnershipRef::new(id, OwnershipCategory::Owned);
        reference.pid = Some(pid);

        self.references.insert(id, reference.clone());

        self.refs_by_owner
            .entry(pid.to_u64())
            .or_insert_with(Vec::new)
            .push(id);

        reference
    }

    /// Register heap ownership.
    pub fn register_heap(&mut self, address: u64, size: usize) -> OwnershipRef {
        let id = self.next_id();
        let ownership = HeapOwnership::new(address, size);

        self.heap_ownership.insert(address, ownership);

        let mut reference = OwnershipRef::new(id, OwnershipCategory::Owned);
        reference.address = Some(address);

        self.references.insert(id, reference.clone());

        reference
    }

    /// Get process ownership.
    pub fn get_process_ownership(&self, pid: u64) -> Option<&ProcessOwnership> {
        self.process_ownership.get(&pid)
    }

    /// Get heap ownership.
    pub fn get_heap_ownership(&self, address: u64) -> Option<&HeapOwnership> {
        self.heap_ownership.get(&address)
    }

    /// Get reference by ID.
    pub fn get_reference(&self, id: u64) -> Option<&OwnershipRef> {
        self.references.get(&id)
    }

    /// Get references for a PID.
    pub fn refs_for_pid(&self, pid: u64) -> Vec<&OwnershipRef> {
        self.refs_by_owner
            .get(&pid)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.references.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Update ownership category.
    pub fn update_category(&mut self, ref_id: u64, category: OwnershipCategory) -> bool {
        if let Some(reference) = self.references.get_mut(&ref_id) {
            reference.category = category;
            return true;
        }
        false
    }

    /// Transfer ownership to another PID.
    pub fn transfer(&mut self, ref_id: u64, new_pid: BeamPid) -> bool {
        let old_pid = if let Some(reference) = self.references.get(&ref_id) {
            reference.pid
        } else {
            return false;
        };

        // Remove from old owner
        if let Some(pid) = old_pid {
            if let Some(ids) = self.refs_by_owner.get_mut(&pid.to_u64()) {
                ids.retain(|&id| id != ref_id);
            }
        }

        // Add to new owner
        self.refs_by_owner
            .entry(new_pid.to_u64())
            .or_insert_with(Vec::new)
            .push(ref_id);

        // Update reference
        if let Some(reference) = self.references.get_mut(&ref_id) {
            reference.pid = Some(new_pid);
            return true;
        }

        false
    }

    /// Drop a reference.
    pub fn drop_reference(&mut self, ref_id: u64) -> bool {
        if let Some(reference) = self.references.remove(&ref_id) {
            // Remove from owner's list
            if let Some(pid) = reference.pid {
                if let Some(ids) = self.refs_by_owner.get_mut(&pid.to_u64()) {
                    ids.retain(|&id| id != ref_id);
                }
            }

            // Remove heap ownership if present
            if let Some(addr) = reference.address {
                self.heap_ownership.remove(&addr);
            }

            return true;
        }
        false
    }

    /// Get total reference count.
    pub fn ref_count(&self) -> usize {
        self.references.len()
    }

    /// Get process count.
    pub fn process_count(&self) -> usize {
        self.process_ownership.len()
    }

    /// Get heap value count.
    pub fn heap_count(&self) -> usize {
        self.heap_ownership.len()
    }

    /// Clear all tracking.
    pub fn clear(&mut self) {
        self.process_ownership.clear();
        self.heap_ownership.clear();
        self.references.clear();
        self.refs_by_owner.clear();
    }
}

impl Default for OwnershipTracker {
    fn default() -> Self {
        Self::new()
    }
}

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_ref_new() {
        let reference = OwnershipRef::new(1, OwnershipCategory::Owned);
        assert_eq!(reference.id, 1);
        assert_eq!(reference.category, OwnershipCategory::Owned);
    }

    #[test]
    fn test_ownership_ref_for_process() {
        let pid = BeamPid::new(1, 1, 0);
        let reference = OwnershipRef::for_process(pid);
        assert!(reference.is_process_ref());
        assert_eq!(reference.pid, Some(pid));
    }

    #[test]
    fn test_ownership_ref_for_heap() {
        let reference = OwnershipRef::for_heap(0x1000);
        assert!(reference.is_heap_ref());
        assert_eq!(reference.address, Some(0x1000));
    }

    #[test]
    fn test_ownership_tracker_new() {
        let tracker = OwnershipTracker::new();
        assert_eq!(tracker.ref_count(), 0);
    }

    #[test]
    fn test_ownership_tracker_register_process() {
        let mut tracker = OwnershipTracker::new();
        let pid = BeamPid::new(1, 1, 0);
        let reference = tracker.register_process(pid);

        assert_eq!(tracker.ref_count(), 1);
        assert_eq!(tracker.process_count(), 1);
        assert!(reference.is_process_ref());
    }

    #[test]
    fn test_ownership_tracker_register_heap() {
        let mut tracker = OwnershipTracker::new();
        let reference = tracker.register_heap(0x1000, 32);

        assert_eq!(tracker.ref_count(), 1);
        assert_eq!(tracker.heap_count(), 1);
        assert!(reference.is_heap_ref());
    }

    #[test]
    fn test_ownership_tracker_transfer() {
        let mut tracker = OwnershipTracker::new();
        let pid1 = BeamPid::new(1, 1, 0);
        let pid2 = BeamPid::new(2, 1, 0);
        let reference = tracker.register_process(pid1);

        tracker.transfer(reference.id, pid2);

        let refs = tracker.refs_for_pid(pid2.to_u64());
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn test_ownership_tracker_drop() {
        let mut tracker = OwnershipTracker::new();
        let pid = BeamPid::new(1, 1, 0);
        let reference = tracker.register_process(pid);

        tracker.drop_reference(reference.id);

        assert_eq!(tracker.ref_count(), 0);
    }
}
