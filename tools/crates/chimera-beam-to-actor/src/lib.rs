//! BEAM to Actor dialect lowering.
//!
//! Converts BEAM dialect operations to the shared Actor dialect.

pub mod context;
pub mod lower;
pub mod patterns;

pub use context::LoweringContext;
pub use lower::{BeamToActorLowerer, LoweringResult};
pub use patterns::LoweringPattern;
