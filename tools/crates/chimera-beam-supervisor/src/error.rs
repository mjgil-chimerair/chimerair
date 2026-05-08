//! Error types for BEAM supervision.
//!
//! Defines errors that can occur during supervision operations.

use serde::{Deserialize, Serialize};

/// Supervisor-level errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupervisorError {
    /// Too many children (exceeds MAX_CHILDREN_PER_SUPERVISOR).
    TooManyChildren,
    /// Child already exists.
    ChildAlreadyExists,
    /// Child not found.
    ChildNotFound,
    /// Invalid restart strategy.
    InvalidStrategy,
    /// Intensity exceeded (too many restarts).
    IntensityExceeded,
    /// Supervisor already terminated.
    AlreadyTerminated,
    /// Shutdown timeout expired.
    ShutdownTimeout,
    /// Invalid child specification.
    InvalidChildSpec(String),
}

impl std::fmt::Display for SupervisorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupervisorError::TooManyChildren => write!(f, "too many children"),
            SupervisorError::ChildAlreadyExists => write!(f, "child already exists"),
            SupervisorError::ChildNotFound => write!(f, "child not found"),
            SupervisorError::InvalidStrategy => write!(f, "invalid restart strategy"),
            SupervisorError::IntensityExceeded => write!(f, "restart intensity exceeded"),
            SupervisorError::AlreadyTerminated => write!(f, "supervisor already terminated"),
            SupervisorError::ShutdownTimeout => write!(f, "shutdown timeout expired"),
            SupervisorError::InvalidChildSpec(msg) => write!(f, "invalid child spec: {}", msg),
        }
    }
}

impl std::error::Error for SupervisorError {}

/// Restart-specific errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartError {
    /// Child does not support restart.
    NotRestartable,
    /// Restart intensity exceeded.
    IntensityExceeded,
    /// Child is already restarting.
    AlreadyRestarting,
    /// Invalid restart parameters.
    InvalidParams(String),
}

impl std::fmt::Display for RestartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RestartError::NotRestartable => write!(f, "child does not support restart"),
            RestartError::IntensityExceeded => write!(f, "restart intensity exceeded"),
            RestartError::AlreadyRestarting => write!(f, "child is already restarting"),
            RestartError::InvalidParams(msg) => write!(f, "invalid restart params: {}", msg),
        }
    }
}

impl std::error::Error for RestartError {}

/// Shutdown-specific errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShutdownError {
    /// Shutdown timeout expired.
    Timeout,
    /// Child refused to shut down.
    Refused,
    /// Shutdown already in progress.
    InProgress,
    /// Invalid shutdown reason.
    InvalidReason(String),
}

impl std::fmt::Display for ShutdownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShutdownError::Timeout => write!(f, "shutdown timeout expired"),
            ShutdownError::Refused => write!(f, "child refused to shut down"),
            ShutdownError::InProgress => write!(f, "shutdown already in progress"),
            ShutdownError::InvalidReason(msg) => write!(f, "invalid shutdown reason: {}", msg),
        }
    }
}

impl std::error::Error for ShutdownError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supervisor_error_display() {
        assert_eq!(
            SupervisorError::TooManyChildren.to_string(),
            "too many children"
        );
        assert_eq!(
            SupervisorError::ChildNotFound.to_string(),
            "child not found"
        );
        assert_eq!(
            SupervisorError::InvalidChildSpec("bad field".to_string()).to_string(),
            "invalid child spec: bad field"
        );
    }

    #[test]
    fn test_restart_error_display() {
        assert_eq!(
            RestartError::NotRestartable.to_string(),
            "child does not support restart"
        );
        assert_eq!(
            RestartError::IntensityExceeded.to_string(),
            "restart intensity exceeded"
        );
    }

    #[test]
    fn test_shutdown_error_display() {
        assert_eq!(
            ShutdownError::Timeout.to_string(),
            "shutdown timeout expired"
        );
        assert_eq!(
            ShutdownError::Refused.to_string(),
            "child refused to shut down"
        );
    }
}
