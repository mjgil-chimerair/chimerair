//! Canonical BLAKE3/SHA-256 hashing with schema domain tags for Zigmera artifacts.
//!
//! Replaces ad hoc `DefaultHasher` usage with cryptographically secure,
//! canonical hashing that is stable across processes and platforms.

pub mod blake3_hasher;
pub mod canonical;
pub mod domain;
pub mod sha256_hasher;
pub mod sort;

pub use blake3_hasher::Blake3Hasher;
pub use canonical::{CanonicalFormatter, Canonicalize};
pub use domain::{DomainTag, SchemaDomain};
pub use sha256_hasher::Sha256Hasher;
pub use sort::{
    deterministic_sort, deterministic_sort_by, deterministic_sort_by_str,
    deterministic_sort_by_u32, deterministic_sort_by_u64, sort_by_canonical_name, sort_byte_slices,
    sort_pairs_by_first, sort_pairs_by_second, sort_strings, sort_u32s, sort_u64s, sort_vec,
    CanonicalOption, CanonicalOrd, CanonicalPair, CanonicalString, DeterministicOrd,
};
