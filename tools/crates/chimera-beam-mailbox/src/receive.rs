//! BEAM selective receive implementation.
//!
//! BEAM's receive is selective: messages are matched against patterns
//! in order, scanning the mailbox until a match is found.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use super::MAX_RECEIVE_TIMEOUT_MS;
use super::{Mailbox, Message, MessageBody};
use chimera_beam_process::Term;

/// A receive clause pattern and handler.
#[derive(Debug, Clone)]
pub struct ReceiveClause<T> {
    /// The pattern to match.
    pub pattern: T,
    /// Handler function index or reference.
    pub handler: usize,
}

/// State machine for receive operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReceiveState {
    /// Idle, waiting for messages.
    Idle,
    /// Scanning mailbox for match.
    Scanning,
    /// Timeout waiting.
    WaitingTimeout,
    /// Receive completed.
    Done,
    /// Receive failed.
    Failed(String),
}

impl Default for ReceiveState {
    fn default() -> Self {
        ReceiveState::Idle
    }
}

/// Timeout configuration for receive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiveTimeout {
    /// No timeout (infinite).
    Infinite,
    /// Timeout in milliseconds.
    Millis(u64),
    /// Timeout from instant.
    Until(Instant),
}

impl ReceiveTimeout {
    /// Create infinite timeout.
    pub fn infinite() -> Self {
        ReceiveTimeout::Infinite
    }

    /// Create timeout in milliseconds.
    pub fn millis(ms: u64) -> Self {
        ReceiveTimeout::Millis(ms.min(MAX_RECEIVE_TIMEOUT_MS))
    }

    /// Create timeout from duration.
    pub fn from_duration(d: Duration) -> Self {
        ReceiveTimeout::Millis(d.as_millis() as u64)
    }

    /// Check if timeout has expired.
    pub fn is_expired(&self) -> bool {
        match self {
            ReceiveTimeout::Infinite => false,
            ReceiveTimeout::Until(deadline) => Instant::now() >= *deadline,
            ReceiveTimeout::Millis(0) => true, // 0 means immediate timeout
            _ => false,
        }
    }

    /// Remaining time until expiry.
    pub fn remaining(&self) -> Option<Duration> {
        match self {
            ReceiveTimeout::Infinite => None,
            ReceiveTimeout::Until(deadline) => {
                let now = Instant::now();
                if now >= *deadline {
                    Some(Duration::ZERO)
                } else {
                    Some(deadline.duration_since(now))
                }
            }
            ReceiveTimeout::Millis(ms) => Some(Duration::from_millis(*ms)),
        }
    }
}

impl Default for ReceiveTimeout {
    fn default() -> Self {
        ReceiveTimeout::Infinite
    }
}

/// Result of a receive operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReceiveResult {
    /// Received a message matching a clause.
    Matched {
        /// Which clause matched (index).
        clause: usize,
        /// The matched message.
        message: Message,
    },
    /// Receive timed out.
    Timeout,
    /// Receive failed.
    Failed(String),
}

/// Pattern matcher for BEAM messages.
///
/// In real implementation, this would handle full pattern matching
/// against Erlang terms. Here we provide a simple string-based matcher.
pub trait MessageMatcher {
    /// Check if a message matches a pattern.
    fn matches(&self, message: &Message, pattern: &Term) -> bool;
}

/// Simple message matcher for testing.
pub struct SimpleMatcher;

impl MessageMatcher for SimpleMatcher {
    fn matches(&self, message: &Message, pattern: &Term) -> bool {
        match (&message.body, pattern) {
            (MessageBody::Term(msg_term), Term::Atom(pat)) => {
                if let Term::Atom(msg_atom) = msg_term {
                    return msg_atom == pat;
                }
            }
            _ => {}
        }
        false // Default: no match
    }
}

/// Selective receive engine.
pub struct ReceiveEngine<'a> {
    mailbox: &'a Mailbox,
    scanned_up_to: usize,
    timeout: ReceiveTimeout,
}

impl<'a> ReceiveEngine<'a> {
    /// Create a new receive engine for a mailbox.
    pub fn new(mailbox: &'a Mailbox) -> Self {
        ReceiveEngine {
            mailbox,
            scanned_up_to: 0,
            timeout: ReceiveTimeout::default(),
        }
    }

    /// Set timeout for receive.
    pub fn with_timeout(mut self, timeout: ReceiveTimeout) -> Self {
        self.timeout = timeout;
        self
    }

    /// Perform selective receive with patterns.
    ///
    /// Patterns are tried in order. First matching message is consumed,
    /// others remain in queue. If no match and timeout expires, returns Timeout.
    pub fn receive<M: MessageMatcher>(&mut self, patterns: &[Term]) -> ReceiveResult {
        // If already expired, return timeout immediately
        if self.timeout.is_expired() {
            return ReceiveResult::Timeout;
        }

        let all_messages = self.mailbox.all_messages();

        // Scan from where we left off (BEAM semantics: selective receive)
        for (idx, msg) in all_messages.iter().enumerate().skip(self.scanned_up_to) {
            // Check if expired during scan
            if self.timeout.is_expired() {
                return ReceiveResult::Timeout;
            }

            for (clause_idx, pattern) in patterns.iter().enumerate() {
                let matcher = SimpleMatcher;
                if matcher.matches(msg, pattern) {
                    // Found match! Remove and return
                    if self.mailbox.remove(msg.id) {
                        self.scanned_up_to = idx; // Next receive starts here
                        return ReceiveResult::Matched {
                            clause: clause_idx,
                            message: msg.clone(),
                        };
                    }
                }
            }
        }

        // No match found yet - would block
        ReceiveResult::Timeout
    }

    /// Reset the receive engine (for next receive).
    pub fn reset(&mut self) {
        self.scanned_up_to = 0;
    }
}

/// Builder for receive operations.
#[derive(Debug, Clone)]
pub struct ReceiveBuilder<'a> {
    mailbox: &'a Mailbox,
    timeout: ReceiveTimeout,
}

impl<'a> ReceiveBuilder<'a> {
    /// Create a new receive builder.
    pub fn new(mailbox: &'a Mailbox) -> Self {
        ReceiveBuilder {
            mailbox,
            timeout: ReceiveTimeout::default(),
        }
    }

    /// Set timeout in milliseconds.
    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout = ReceiveTimeout::millis(ms);
        self
    }

    /// Set infinite timeout.
    pub fn infinite_timeout(mut self) -> Self {
        self.timeout = ReceiveTimeout::infinite();
        self
    }

    /// Build receive engine.
    pub fn build(&self) -> ReceiveEngine<'a> {
        ReceiveEngine::new(self.mailbox).with_timeout(self.timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;
    use chimera_beam_process::Term;

    fn make_msg(id: u64, atom: &str) -> Message {
        Message::from_term(id, Term::atom(atom), 1000 + id)
    }

    #[test]
    fn test_receive_timeout_millis() {
        let timeout = ReceiveTimeout::millis(5000);
        assert!(!timeout.is_expired());
        assert_eq!(timeout.remaining(), Some(Duration::from_millis(5000)));
    }

    #[test]
    fn test_receive_timeout_zero() {
        let timeout = ReceiveTimeout::millis(0);
        assert!(timeout.is_expired());
    }

    #[test]
    fn test_receive_timeout_infinite() {
        let timeout = ReceiveTimeout::infinite();
        assert!(!timeout.is_expired());
        assert_eq!(timeout.remaining(), None);
    }

    #[test]
    fn test_receive_result_matched() {
        let msg = make_msg(1, "test");
        let result = ReceiveResult::Matched {
            clause: 0,
            message: msg.clone(),
        };

        match result {
            ReceiveResult::Matched { clause, message } => {
                assert_eq!(clause, 0);
                assert_eq!(message.id, 1);
            }
            _ => panic!("expected Matched"),
        }
    }

    #[test]
    fn test_receive_builder() {
        let mailbox = Mailbox::new();
        let engine = ReceiveBuilder::new(&mailbox).timeout_ms(1000).build();

        assert!(!engine.timeout.is_expired());
    }

    #[test]
    fn test_receive_engine_timeout() {
        let mailbox = Mailbox::new();
        let mut engine = ReceiveEngine::new(&mailbox).with_timeout(ReceiveTimeout::millis(0));

        let result = engine.receive::<SimpleMatcher>(&[Term::atom("test")]);
        assert!(matches!(result, ReceiveResult::Timeout));
    }

    #[test]
    fn test_receive_engine_reset() {
        let mailbox = Mailbox::new();
        mailbox.enqueue(make_msg(1, "a"));

        let mut engine = ReceiveEngine::new(&mailbox);
        engine.receive::<SimpleMatcher>(&[Term::atom("a")]);
        engine.reset();

        // After reset, scanned_up_to goes back to 0
        assert_eq!(engine.scanned_up_to, 0);
    }
}
