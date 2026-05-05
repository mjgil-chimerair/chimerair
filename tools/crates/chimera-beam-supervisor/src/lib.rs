//! BEAM supervision tree for ChimeraIR.
//!
//! Models Erlang/OTP supervision trees with restart strategies,
//! child specifications, and hierarchical fault recovery.

pub mod child;
pub mod error;
pub mod strategy;
pub mod tree;

pub use child::{ChildSpec, ChildType, ShutdownTimeout};
pub use error::{RestartError, ShutdownError, SupervisorError};
pub use strategy::{RestartIntensity, RestartStrategy};
pub use tree::{ChildNode, SupervisorNode, SupervisorTree};

/// Maximum children per supervisor (BEAM default).
pub const MAX_CHILDREN_PER_SUPERVISOR: usize = 128;

/// Maximum restart intensity (per period).
pub const MAX_RESTART_INTENSITY: u32 = 100;

/// Default shutdown timeout in milliseconds.
pub const DEFAULT_SHUTDOWN_TIMEOUT_MS: u32 = 5000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_children() {
        assert!(MAX_CHILDREN_PER_SUPERVISOR > 0);
    }

    #[test]
    fn test_max_restart_intensity() {
        assert!(MAX_RESTART_INTENSITY > 0);
    }

    #[test]
    fn test_default_shutdown_timeout() {
        assert_eq!(DEFAULT_SHUTDOWN_TIMEOUT_MS, 5000);
    }
}
