//! BEAM CLI integration for ChimeraIR.
//!
//! Provides command-line interface for BEAM analysis, compilation,
//! and integration with the ChimeraIR toolchain.

pub mod commands;
pub mod context;
pub mod output;

pub use commands::{AnalyzeCommand, BeamCommand, CompileCommand, InspectCommand};
pub use context::BeamCliContext;
pub use output::{OutputFormat, OutputWriter};

/// CLI version.
pub const CLI_VERSION: &str = "0.1.0";
