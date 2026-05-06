//! AIR decoder for parsing real `.zairpack` from patched Zig compiler.
//!
//! Task 41: Replace mock AIR decoder with real `.zairpack` parsing.

pub mod decoder;
pub mod coverage;
pub mod source_loc;
pub mod types;
pub mod layouts;
pub mod comptime;
pub mod symbols;
pub mod id_remapper;
pub mod migration;

pub use decoder::AirDecoder;
pub use coverage::InstructionCoverage;
pub use source_loc::SourceLocationEncoder;