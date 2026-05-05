//! BEAM dialect for ChimeraIR MLIR.
//!
//! This dialect models BEAM (Bogdan/Björn's Erlang Abstract Machine) semantics
//! as an intermediate representation before lowering to the Actor dialect.

pub mod dialect;
pub mod ops;
pub mod types;

pub use dialect::BeamDialect;
pub use ops::BeamOp;
pub use ops::BeamOpKind;
pub use types::BeamType;
