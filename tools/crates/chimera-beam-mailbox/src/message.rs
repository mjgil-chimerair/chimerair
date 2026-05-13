//! BEAM message representation.
//!
//! Messages in BEAM are arbitrary terms. This module defines
//! message structures and flags.

use chimera_beam_process::Term;
use serde::{Deserialize, Serialize};

/// A message in a BEAM mailbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier.
    pub id: u64,
    /// The message body.
    pub body: MessageBody,
    /// Message flags.
    pub flags: MessageFlags,
    /// Timestamp when message was sent.
    pub timestamp: u64,
}

impl Message {
    /// Create a new message.
    pub fn new(id: u64, body: MessageBody, timestamp: u64) -> Self {
        Message {
            id,
            body,
            flags: MessageFlags::default(),
            timestamp,
        }
    }

    /// Create from a term.
    pub fn from_term(id: u64, term: Term, timestamp: u64) -> Self {
        Message {
            id,
            body: MessageBody::Term(term),
            flags: MessageFlags::default(),
            timestamp,
        }
    }

    /// Get the message as a term.
    pub fn as_term(&self) -> &Term {
        match &self.body {
            MessageBody::Term(t) => t,
            _ => panic!("message is not a term"),
        }
    }
}

/// Message body variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageBody {
    /// A regular Erlang term.
    Term(Term),
    /// A cross-language message (binary payload).
    Binary(Vec<u8>),
    /// Exit signal (special message type).
    Exit { from: u64, reason: String },
    /// DOWN signal for monitors.
    Down {
        ref_id: u64,
        from: u64,
        reason: String,
    },
}

impl MessageBody {
    /// Check if this is a regular term.
    pub fn is_term(&self) -> bool {
        matches!(self, MessageBody::Term(_))
    }

    /// Get the term if this is a term.
    pub fn as_term(&self) -> Option<&Term> {
        match self {
            MessageBody::Term(t) => Some(t),
            _ => None,
        }
    }
}

/// Message flags and metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageFlags {
    /// Message has been traced.
    pub traced: bool,
    /// Message is important (high priority).
    pub important: bool,
    /// Message was delivered via distribution.
    pub distributed: bool,
}

impl MessageFlags {
    /// Create default flags.
    pub fn new() -> Self {
        MessageFlags::default()
    }

    /// Set traced flag.
    pub fn with_traced(mut self, traced: bool) -> Self {
        self.traced = traced;
        self
    }

    /// Set important flag.
    pub fn with_important(mut self, important: bool) -> Self {
        self.important = important;
        self
    }

    /// Set distributed flag.
    pub fn with_distributed(mut self, distributed: bool) -> Self {
        self.distributed = distributed;
        self
    }
}

/// Statistics about a mailbox.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MailboxStats {
    /// Current queue length.
    pub queue_len: usize,
    /// Number of messages processed.
    pub messages_processed: u64,
    /// Number of receives that timed out.
    pub receives_timeout: u64,
    /// Number of receives that found a message.
    pub receives_found: u64,
}

impl MailboxStats {
    /// Create new stats.
    pub fn new() -> Self {
        MailboxStats::default()
    }

    /// Increment messages processed.
    pub fn inc_processed(&mut self) {
        self.messages_processed += 1;
    }

    /// Increment receives timeout.
    pub fn inc_timeout(&mut self) {
        self.receives_timeout += 1;
    }

    /// Increment receives found.
    pub fn inc_found(&mut self) {
        self.receives_found += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_beam_process::Term;

    #[test]
    fn test_message_from_term() {
        let msg = Message::from_term(1, Term::int(42), 1000);
        assert_eq!(msg.id, 1);
        assert!(msg.body.is_term());
        assert_eq!(msg.as_term(), &Term::int(42));
    }

    #[test]
    fn test_message_flags() {
        let flags = MessageFlags::new().with_traced(true).with_important(false);
        assert!(flags.traced);
        assert!(!flags.important);
    }

    #[test]
    fn test_mailbox_stats() {
        let mut stats = MailboxStats::new();
        stats.queue_len = 10;
        stats.inc_processed();
        stats.inc_found();
        assert_eq!(stats.messages_processed, 1);
        assert_eq!(stats.receives_found, 1);
    }
}
