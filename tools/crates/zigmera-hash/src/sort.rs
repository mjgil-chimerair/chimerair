//! Deterministic ordering utilities for Zigmera artifacts.
//!
//! Task 24: Add deterministic compiler-side ordering (consumer-side)
//!
//! Ensures consistent ordering of emitted data across platforms
//! and build environments for byte-for-byte reproducibility.

use serde::{Deserialize, Serialize};

/// Trait for types that can be deterministically ordered.
pub trait DeterministicOrd: Ord {
    /// Convert to a canonical byte representation for comparison.
    fn to_canonical_bytes(&self) -> Vec<u8>;
}

/// Wrapper that implements Ord based on canonical byte representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalOrd<T: DeterministicOrd> {
    value: T,
}

impl<T: DeterministicOrd> CanonicalOrd<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub fn get(&self) -> &T {
        &self.value
    }
}

impl<T: DeterministicOrd> Ord for CanonicalOrd<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value
            .to_canonical_bytes()
            .cmp(&other.value.to_canonical_bytes())
    }
}

impl<T: DeterministicOrd> PartialOrd for CanonicalOrd<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: DeterministicOrd> Eq for CanonicalOrd<T> {}

impl<T: DeterministicOrd> PartialEq for CanonicalOrd<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.to_canonical_bytes() == other.value.to_canonical_bytes()
    }
}

/// Sort iterator items deterministically by canonical bytes.
pub fn deterministic_sort<T: DeterministicOrd>(items: &mut [T]) {
    items.sort_by(|a, b| a.to_canonical_bytes().cmp(&b.to_canonical_bytes()));
}

/// Sort items by a key function, using canonical bytes for comparison.
pub fn deterministic_sort_by<T, F>(items: &mut [T], mut key: F)
where
    F: FnMut(&T) -> Vec<u8>,
{
    items.sort_by_cached_key(|item| key(item));
}

/// Sort items by a string key function for deterministic ordering.
pub fn deterministic_sort_by_str<T, F>(items: &mut [T], mut key: F)
where
    F: FnMut(&T) -> String,
{
    items.sort_by_cached_key(|item| key(item));
}

/// Sort items by a u32 key function for deterministic ordering.
pub fn deterministic_sort_by_u32<T, F>(items: &mut [T], mut key: F)
where
    F: FnMut(&T) -> u32,
{
    items.sort_by(|a, b| key(a).cmp(&key(b)));
}

/// Sort items by a u64 key function for deterministic ordering.
pub fn deterministic_sort_by_u64<T, F>(items: &mut [T], mut key: F)
where
    F: FnMut(&T) -> u64,
{
    items.sort_by(|a, b| key(a).cmp(&key(b)));
}

/// Trait for sortable string items with canonical ordering.
pub trait SortByCanonicalName {
    fn canonical_name(&self) -> &[u8];
}

/// Sort items by their canonical name.
pub fn sort_by_canonical_name<T: SortByCanonicalName>(items: &mut [T]) {
    items.sort_by(|a, b| a.canonical_name().cmp(b.canonical_name()));
}

/// Canonical string wrapper that sorts consistently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalString {
    bytes: Vec<u8>,
}

impl CanonicalString {
    pub fn new(s: &str) -> Self {
        Self {
            bytes: s.as_bytes().to_vec(),
        }
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes).unwrap_or("")
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl std::fmt::Display for CanonicalString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Ord for CanonicalString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl PartialOrd for CanonicalString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl DeterministicOrd for CanonicalString {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

/// Canonical option wrapper for deterministic ordering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalOption<T: DeterministicOrd> {
    present: bool,
    value: Option<T>,
}

impl<T: DeterministicOrd> CanonicalOption<T> {
    pub fn some(value: T) -> Self {
        Self {
            present: true,
            value: Some(value),
        }
    }

    pub fn none() -> Self {
        Self {
            present: false,
            value: None,
        }
    }

    pub fn is_some(&self) -> bool {
        self.present
    }

    pub fn is_none(&self) -> bool {
        !self.present
    }
}

impl<T: DeterministicOrd> Ord for CanonicalOption<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_canonical_bytes().cmp(&other.to_canonical_bytes())
    }
}

impl<T: DeterministicOrd> PartialOrd for CanonicalOption<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: DeterministicOrd> Eq for CanonicalOption<T> {}

impl<T: DeterministicOrd> PartialEq for CanonicalOption<T> {
    fn eq(&self, other: &Self) -> bool {
        self.to_canonical_bytes() == other.to_canonical_bytes()
    }
}

impl<T: DeterministicOrd> DeterministicOrd for CanonicalOption<T> {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(if self.present { 1 } else { 0 });
        if let Some(ref v) = self.value {
            bytes.extend_from_slice(&v.to_canonical_bytes());
        }
        bytes
    }
}

/// Sort a slice of strings deterministically.
pub fn sort_strings(items: &mut [String]) {
    items.sort();
}

/// Sort a slice of byte vectors deterministically.
pub fn sort_byte_slices(items: &mut [Vec<u8>]) {
    items.sort();
}

/// Sort a slice of u32 values deterministically.
pub fn sort_u32s(items: &mut [u32]) {
    items.sort();
}

/// Sort a slice of u64 values deterministically.
pub fn sort_u64s(items: &mut [u64]) {
    items.sort();
}

/// Sort tuples by first element, then second.
pub fn sort_tuples_by_first<T: Ord>(items: &mut [(T,)]) {
    items.sort_by(|a, b| a.0.cmp(&b.0));
}

/// Canonical pair for deterministic sorting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPair<A: DeterministicOrd, B: DeterministicOrd> {
    first: A,
    second: B,
}

impl<A: DeterministicOrd, B: DeterministicOrd> CanonicalPair<A, B> {
    pub fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A: DeterministicOrd, B: DeterministicOrd> Ord for CanonicalPair<A, B> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_canonical_bytes().cmp(&other.to_canonical_bytes())
    }
}

impl<A: DeterministicOrd, B: DeterministicOrd> PartialOrd for CanonicalPair<A, B> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: DeterministicOrd, B: DeterministicOrd> Eq for CanonicalPair<A, B> {}

impl<A: DeterministicOrd, B: DeterministicOrd> PartialEq for CanonicalPair<A, B> {
    fn eq(&self, other: &Self) -> bool {
        self.to_canonical_bytes() == other.to_canonical_bytes()
    }
}

impl<A: DeterministicOrd, B: DeterministicOrd> DeterministicOrd for CanonicalPair<A, B> {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.first.to_canonical_bytes());
        bytes.extend_from_slice(&self.second.to_canonical_bytes());
        bytes
    }
}

/// Sort items in a Vec deterministically.
pub fn sort_vec<T: DeterministicOrd>(items: &mut Vec<T>) {
    items.sort_by(|a, b| a.to_canonical_bytes().cmp(&b.to_canonical_bytes()));
}

/// Sort Vec of pairs by first element.
pub fn sort_pairs_by_first<A: DeterministicOrd, B: DeterministicOrd>(items: &mut Vec<(A, B)>) {
    items.sort_by(|(a, _), (b, _)| a.to_canonical_bytes().cmp(&b.to_canonical_bytes()));
}

/// Sort Vec of pairs by second element.
pub fn sort_pairs_by_second<A: DeterministicOrd, B: DeterministicOrd>(items: &mut Vec<(A, B)>) {
    items.sort_by(|(_, a), (_, b)| a.to_canonical_bytes().cmp(&b.to_canonical_bytes()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_string_ord() {
        let a = CanonicalString::new("beta");
        let b = CanonicalString::new("alpha");
        let c = CanonicalString::new("alpha");

        assert!(b < a);
        assert_eq!(b, c);
    }

    #[test]
    fn test_canonical_option_some() {
        let some = CanonicalOption::some(CanonicalString::new("value"));
        let none = CanonicalOption::<CanonicalString>::none();

        assert!(some.is_some());
        assert!(none.is_none());
        assert!(some > none);
    }

    #[test]
    fn test_canonical_pair() {
        let pair = CanonicalPair::new(
            CanonicalString::new("first"),
            CanonicalString::new("second"),
        );

        let bytes = pair.to_canonical_bytes();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_sort_strings() {
        let mut strings = vec!["gamma", "alpha", "beta"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        sort_strings(&mut strings);
        assert_eq!(&strings, &["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_sort_u32s() {
        let mut nums = vec![30, 10, 20];
        sort_u32s(&mut nums);
        assert_eq!(nums, vec![10, 20, 30]);
    }

    #[test]
    fn test_sort_byte_slices() {
        let mut slices = vec![vec![3], vec![1], vec![2]];
        sort_byte_slices(&mut slices);
        assert_eq!(slices, vec![vec![1], vec![2], vec![3]]);
    }

    #[test]
    fn test_canonical_ord_wrapper() {
        let a = CanonicalString::new("beta");
        let b = CanonicalString::new("alpha");

        let wrapped_a = CanonicalOrd::new(a.clone());
        let wrapped_b = CanonicalOrd::new(b.clone());

        assert!(wrapped_b < wrapped_a);
    }

    #[test]
    fn test_deterministic_sort() {
        #[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
        struct TestItem {
            name: String,
            id: u32,
        }

        impl DeterministicOrd for TestItem {
            fn to_canonical_bytes(&self) -> Vec<u8> {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(self.name.as_bytes());
                bytes.extend_from_slice(&self.id.to_le_bytes());
                bytes
            }
        }

        let mut items = vec![
            TestItem {
                name: "zebra".into(),
                id: 1,
            },
            TestItem {
                name: "alpha".into(),
                id: 2,
            },
            TestItem {
                name: "beta".into(),
                id: 3,
            },
        ];

        deterministic_sort(&mut items);
        assert_eq!(items[0].name, "alpha");
        assert_eq!(items[1].name, "beta");
        assert_eq!(items[2].name, "zebra");
    }

    #[test]
    fn test_sort_by_canonical_name() {
        #[derive(Debug, Clone)]
        struct Named {
            name: Vec<u8>,
        }

        impl SortByCanonicalName for Named {
            fn canonical_name(&self) -> &[u8] {
                &self.name
            }
        }

        let mut items = vec![
            Named {
                name: b"zebra".to_vec(),
            },
            Named {
                name: b"alpha".to_vec(),
            },
            Named {
                name: b"beta".to_vec(),
            },
        ];

        sort_by_canonical_name(&mut items);
        assert_eq!(&items[0].name, b"alpha");
        assert_eq!(&items[1].name, b"beta");
        assert_eq!(&items[2].name, b"zebra");
    }
}
