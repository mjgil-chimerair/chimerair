//! BEAM mailbox structure.
//!
//! The mailbox is the queue of messages for a BEAM process.
//! Messages are enqueued at the back and dequeued via selective receive.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::RwLock;

use super::{MailboxStats, Message, MessageBody, MessageFlags};
use super::{DEFAULT_MAX_QUEUE_LENGTH, MAX_RECEIVE_TIMEOUT_MS};

/// BEAM mailbox for a process.
#[derive(Debug)]
pub struct Mailbox {
    /// Message queue.
    queue: RwLock<VecDeque<Message>>,
    /// Mailbox statistics.
    stats: RwLock<MailboxStats>,
    /// Maximum queue length before warnings.
    max_length: usize,
}

impl Mailbox {
    /// Create a new mailbox.
    pub fn new() -> Self {
        Mailbox {
            queue: RwLock::new(VecDeque::new()),
            stats: RwLock::new(MailboxStats::new()),
            max_length: DEFAULT_MAX_QUEUE_LENGTH,
        }
    }

    /// Create with custom max length.
    pub fn with_max_length(max: usize) -> Self {
        Mailbox {
            queue: RwLock::new(VecDeque::new()),
            stats: RwLock::new(MailboxStats::new()),
            max_length: max,
        }
    }

    /// Enqueue a message.
    ///
    /// Returns `true` if enqueued, `false` if queue is full.
    pub fn enqueue(&self, msg: Message) -> bool {
        let mut queue = self.queue.write().unwrap();
        if queue.len() >= self.max_length {
            return false;
        }
        queue.push_back(msg);
        let new_len = queue.len();
        drop(queue);
        let mut stats = self.stats.write().unwrap();
        stats.queue_len = new_len;
        true
    }

    /// Enqueue with custom flags.
    #[allow(unused)]
    pub fn enqueue_with_flags(&self, msg: Message, flags: MessageFlags) -> bool {
        let mut msg_with_flags = msg;
        msg_with_flags.flags = flags;
        self.enqueue(msg_with_flags)
    }

    /// Dequeue the next message (FIFO, no pattern matching).
    pub fn dequeue(&self) -> Option<Message> {
        let mut queue = self.queue.write().unwrap();
        let msg = queue.pop_front();
        let new_len = queue.len();
        drop(queue);
        if msg.is_some() {
            let mut stats = self.stats.write().unwrap();
            stats.queue_len = new_len;
            stats.inc_processed();
        }
        msg
    }

    /// Peek at the next message without removing.
    pub fn peek(&self) -> Option<Message> {
        let queue = self.queue.read().unwrap();
        queue.front().cloned()
    }

    /// Get queue length.
    pub fn len(&self) -> usize {
        let queue = self.queue.read().unwrap();
        queue.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if at capacity.
    pub fn is_full(&self) -> bool {
        self.len() >= self.max_length
    }

    /// Get statistics.
    pub fn stats(&self) -> MailboxStats {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }

    /// Clear all messages.
    pub fn clear(&self) {
        let mut queue = self.queue.write().unwrap();
        queue.clear();
        drop(queue);
        let mut stats = self.stats.write().unwrap();
        stats.queue_len = 0;
    }

    /// Get all messages (for debugging).
    pub fn all_messages(&self) -> Vec<Message> {
        let queue = self.queue.read().unwrap();
        queue.iter().cloned().collect()
    }

    /// Find messages matching a predicate (for selective receive).
    pub fn find_matching<F>(&self, pred: F) -> Vec<Message>
    where
        F: Fn(&Message) -> bool,
    {
        let queue = self.queue.read().unwrap();
        queue.iter().filter(|m| pred(m)).cloned().collect()
    }

    /// Remove a specific message by ID.
    pub fn remove(&self, msg_id: u64) -> bool {
        let mut queue = self.queue.write().unwrap();
        let pos = queue.iter().position(|m| m.id == msg_id);
        if let Some(idx) = pos {
            queue.remove(idx);
            let new_len = queue.len();
            drop(queue);
            let mut stats = self.stats.write().unwrap();
            stats.queue_len = new_len;
            return true;
        }
        drop(queue);
        false
    }
}

impl Default for Mailbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating mailboxes with custom settings.
#[derive(Debug, Clone)]
pub struct MailboxBuilder {
    max_length: usize,
}

impl MailboxBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        MailboxBuilder {
            max_length: DEFAULT_MAX_QUEUE_LENGTH,
        }
    }

    /// Set maximum queue length.
    #[allow(unused)]
    pub fn max_length(mut self, len: usize) -> Self {
        self.max_length = len;
        self
    }

    /// Build the mailbox.
    pub fn build(self) -> Mailbox {
        Mailbox::with_max_length(self.max_length)
    }
}

impl Default for MailboxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_beam_process::Term;

    fn make_msg(id: u64, term: i64) -> Message {
        Message::from_term(id, Term::int(term), 1000 + id)
    }

    #[test]
    fn test_mailbox_enqueue_dequeue() {
        let mailbox = Mailbox::new();
        assert!(mailbox.enqueue(make_msg(1, 100)));
        assert!(mailbox.enqueue(make_msg(2, 200)));

        assert_eq!(mailbox.len(), 2);

        let msg = mailbox.dequeue();
        assert!(msg.is_some());
        assert_eq!(msg.unwrap().id, 1);
    }

    #[test]
    fn test_mailbox_peek() {
        let mailbox = Mailbox::new();
        mailbox.enqueue(make_msg(1, 100));
        mailbox.enqueue(make_msg(2, 200));

        let peeked = mailbox.peek();
        assert!(peeked.is_some());
        assert_eq!(peeked.unwrap().id, 1);

        // Peek doesn't remove
        assert_eq!(mailbox.len(), 2);
    }

    #[test]
    fn test_mailbox_is_empty() {
        let mailbox = Mailbox::new();
        assert!(mailbox.is_empty());

        mailbox.enqueue(make_msg(1, 100));
        assert!(!mailbox.is_empty());
    }

    #[test]
    fn test_mailbox_remove() {
        let mailbox = Mailbox::new();
        mailbox.enqueue(make_msg(1, 100));
        mailbox.enqueue(make_msg(2, 200));

        assert!(mailbox.remove(1));
        assert_eq!(mailbox.len(), 1);

        let remaining = mailbox.dequeue().unwrap();
        assert_eq!(remaining.id, 2);
    }

    #[test]
    fn test_mailbox_find_matching() {
        let mailbox = Mailbox::new();
        mailbox.enqueue(make_msg(1, 100));
        mailbox.enqueue(make_msg(2, 200));
        mailbox.enqueue(make_msg(3, 300));

        let matches = mailbox.find_matching(|m| {
            if let MessageBody::Term(t) = &m.body {
                if let Term::Int(n) = t {
                    return *n > 150;
                }
            }
            false
        });

        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_mailbox_stats() {
        let mailbox = Mailbox::new();
        mailbox.enqueue(make_msg(1, 100));
        mailbox.dequeue();

        let stats = mailbox.stats();
        assert_eq!(stats.messages_processed, 1);
    }

    #[test]
    fn test_mailbox_builder() {
        let mailbox = MailboxBuilder::new().max_length(50).build();

        assert!(mailbox.is_empty());
    }
}
