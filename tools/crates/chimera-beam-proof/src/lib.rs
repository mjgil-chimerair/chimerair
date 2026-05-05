//! BEAM proof artifact emitter for ChimeraIR.
//!
//! Emits proof facts for BEAM boundaries verifying ownership invariants,
//! memory safety, and message protocol compliance.

pub mod emitter;
pub mod fact;
pub mod serialize;
pub mod validate;

pub use emitter::ProofEmitter;
pub use fact::{FactId, ProofFact, ProofKind, ProofTarget};
pub use serialize::{deserialize_proof, serialize_proof};
pub use validate::{ProofResult, ProofValidator, ValidationInput};

/// Maximum facts per proof artifact.
pub const MAX_FACTS_PER_ARTIFACT: usize = 65536;

/// Current proof format version.
pub const PROOF_VERSION: u32 = 1;
