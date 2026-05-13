//! BEAM process links.
//!
//! Bidirectional links between processes. When a process exits,
//! linked processes receive exit signals.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Kind of link relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkKind {
    /// Normal link (processes die together).
    Normal,
    /// Process is a group leader.
    GroupLeader,
}

/// A link between two processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    /// PID of the other process in the link.
    pub other_pid: u64,
    /// Kind of link.
    pub kind: LinkKind,
    /// When the link was created.
    pub created_at: u64,
}

impl Link {
    /// Create a new link.
    pub fn new(other_pid: u64, kind: LinkKind, created_at: u64) -> Self {
        Link {
            other_pid,
            kind,
            created_at,
        }
    }

    /// Create a normal link.
    pub fn normal(other_pid: u64, created_at: u64) -> Self {
        Link::new(other_pid, LinkKind::Normal, created_at)
    }
}

/// Handle for managing links (holds the actual link data behind Arc for sharing).
#[derive(Debug, Clone)]
pub struct LinkHandle(Arc<Link>);

impl LinkHandle {
    /// Create a new link handle.
    pub fn new(link: Link) -> Self {
        LinkHandle(Arc::new(link))
    }

    /// Get the linked PID.
    pub fn other_pid(&self) -> u64 {
        self.0.other_pid
    }

    /// Get the link kind.
    pub fn kind(&self) -> LinkKind {
        self.0.kind
    }

    /// Get creation timestamp.
    pub fn created_at(&self) -> u64 {
        self.0.created_at
    }

    /// Check if this link points to the given PID.
    pub fn is_to(&self, pid: u64) -> bool {
        self.0.other_pid == pid
    }
}

impl From<Link> for LinkHandle {
    fn from(link: Link) -> Self {
        LinkHandle::new(link)
    }
}

/// Link table tracking all links for a process.
#[derive(Debug, Clone, Default)]
pub struct LinkTable {
    /// Links from this process to others.
    links: Vec<LinkHandle>,
}

impl LinkTable {
    /// Create a new link table.
    pub fn new() -> Self {
        LinkTable { links: Vec::new() }
    }

    /// Add a link.
    pub fn add_link(&mut self, link: Link) {
        // Avoid duplicates
        if !self.has_link_to(link.other_pid) {
            self.links.push(link.into());
        }
    }

    /// Remove a link by PID.
    pub fn remove_link(&mut self, pid: u64) -> bool {
        let idx = self.links.iter().position(|h| h.is_to(pid));
        if let Some(idx) = idx {
            self.links.swap_remove(idx);
            true
        } else {
            false
        }
    }

    /// Check if linked to a PID.
    pub fn has_link_to(&self, pid: u64) -> bool {
        self.links.iter().any(|h| h.is_to(pid))
    }

    /// Get all linked PIDs.
    pub fn linked_pids(&self) -> Vec<u64> {
        self.links.iter().map(|h| h.other_pid()).collect()
    }

    /// Number of links.
    pub fn len(&self) -> usize {
        self.links.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }
}

/// Exit reason for a process.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    /// Normal exit (process completed normally).
    Normal,
    /// Process was killed.
    Killed,
    /// Exit with a value.
    Value(i64),
    /// Abnormal exit (crashed).
    Abnormal,
    /// Custom exit term.
    Term(String),
}

impl ExitReason {
    /// Check if this is a normal exit.
    pub fn is_normal(&self) -> bool {
        matches!(self, ExitReason::Normal)
    }

    /// Check if this exit can be trapped.
    pub fn is_trappable(&self) -> bool {
        !matches!(self, ExitReason::Killed)
    }
}

impl std::fmt::Display for ExitReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExitReason::Normal => write!(f, "normal"),
            ExitReason::Killed => write!(f, "killed"),
            ExitReason::Value(v) => write!(f, "{{{}, []}}", v),
            ExitReason::Abnormal => write!(f, "abnormal"),
            ExitReason::Term(t) => write!(f, "{}", t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_creation() {
        let link = Link::normal(123, 1000);
        assert_eq!(link.other_pid, 123);
        assert_eq!(link.kind, LinkKind::Normal);
        assert_eq!(link.created_at, 1000);
    }

    #[test]
    fn test_link_handle() {
        let handle = LinkHandle::new(Link::normal(456, 2000));
        assert_eq!(handle.other_pid(), 456);
        assert!(handle.is_to(456));
        assert!(!handle.is_to(789));
    }

    #[test]
    fn test_link_table() {
        let mut table = LinkTable::new();
        table.add_link(Link::normal(100, 1));
        table.add_link(Link::normal(200, 2));

        assert_eq!(table.len(), 2);
        assert!(table.has_link_to(100));
        assert!(table.has_link_to(200));
        assert!(!table.has_link_to(300));

        let pids = table.linked_pids();
        assert!(pids.contains(&100));
        assert!(pids.contains(&200));
    }

    #[test]
    fn test_link_table_remove() {
        let mut table = LinkTable::new();
        table.add_link(Link::normal(100, 1));
        table.add_link(Link::normal(200, 2));

        assert!(table.remove_link(100));
        assert_eq!(table.len(), 1);
        assert!(!table.has_link_to(100));
        assert!(table.has_link_to(200));

        // Remove non-existent
        assert!(!table.remove_link(999));
    }

    #[test]
    fn test_link_table_no_duplicates() {
        let mut table = LinkTable::new();
        table.add_link(Link::normal(100, 1));
        table.add_link(Link::normal(100, 2)); // duplicate

        assert_eq!(table.len(), 1);
    }

    #[test]
    fn test_exit_reason_display() {
        assert_eq!(ExitReason::Normal.to_string(), "normal");
        assert_eq!(ExitReason::Killed.to_string(), "killed");
        assert_eq!(ExitReason::Value(42).to_string(), "{42, []}");
        assert_eq!(ExitReason::Abnormal.to_string(), "abnormal");
        assert_eq!(ExitReason::Term("bad".to_string()).to_string(), "bad");
    }
}
