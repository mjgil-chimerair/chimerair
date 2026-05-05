//! BEAM mailbox structure and selective receive.
//!
//! Models the BEAM message queue with selective receive semantics.
//! Messages are matched against patterns in order, with timeout support.

pub mod mailbox;
pub mod message;
pub mod receive;

pub use mailbox::Mailbox;
pub use message::{MailboxStats, Message, MessageBody, MessageFlags};
pub use receive::{ReceiveResult, ReceiveState, ReceiveTimeout};

use chimera_beam_process::BeamPid;

/// Maximum queue length (BEAM default is around 1000 for warnings).
pub const DEFAULT_MAX_QUEUE_LENGTH: usize = 1000;

/// Maximum receive timeout in milliseconds.
pub const MAX_RECEIVE_TIMEOUT_MS: u64 = 4_294_967_295;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_max_queue_length() {
        assert!(DEFAULT_MAX_QUEUE_LENGTH > 0);
    }

    #[test]
    fn test_max_receive_timeout() {
        assert!(MAX_RECEIVE_TIMEOUT_MS > 0);
    }
}
