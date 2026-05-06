//! `.zchproof` proof obligations schema.

use serde::{Deserialize, Serialize};

/// Magic bytes for `.zchproof` format.
pub const ZCHPROOF_MAGIC: &[u8; 8] = b"ZCHPRF01";

/// Current schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// `.zchproof` header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChproofHeader {
    pub magic: [u8; 8],
    pub schema_version: u32,
    pub zig_commit: [u8; 20],
    pub target: String,
    pub timestamp_ns: u64,
    pub proof_count: u32,
    pub checksum: [u8; 32],
}

/// Kind of proof obligation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofKind {
    InvalidationSoundness,
    CacheSoundness,
    LayoutPreservation,
    ResultLowering,
    OwnershipDefer,
}

/// A proof obligation to be verified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofObligation {
    pub id: u64,
    pub kind: ProofKind,
    pub description: String,
    pub antecedents: Vec<Fact>,
    pub claim: Fact,
}

/// A fact used in proof obligations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fact {
    pub predicate: String,
    pub subject: String,
    pub value: serde_json::Value,
}

/// Cache reuse proof fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheProofFact {
    pub cache_key: String,
    pub semantic_fingerprint: [u8; 32],
    pub dependency_fingerprints: Vec<[u8; 32]>,
    pub schema_version: u32,
    pub target: String,
    pub build_options_hash: [u8; 32],
    pub reusable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeanCacheProofFact {
    pub cache_key: String,
    pub semantic_fingerprint_hex: String,
    pub dependency_fingerprints_hex: Vec<String>,
    pub schema_version: u32,
    pub target: String,
    pub build_options_hash_hex: String,
    pub reusable: bool,
}

/// Invalidation soundness proof fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvalidationProofFact {
    pub changed_node: u64,
    pub reason: InvalidationReason,
    pub affected_exports: Vec<u64>,
    pub sound: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeanInvalidationProofFact {
    pub changed_node: u64,
    pub reason: String,
    pub affected_exports: Vec<u64>,
    pub sound: bool,
}

/// Reason for invalidation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidationReason {
    SourceChange,
    TypeChanged,
    LayoutChanged,
    ComptimeChanged,
    ExportChanged,
    LinkChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeanProofInputSchema {
    pub version: u32,
    pub target: String,
    pub cache_facts: Vec<LeanCacheProofFact>,
    pub invalidation_facts: Vec<LeanInvalidationProofFact>,
}

/// Complete `.zchproof` proof obligations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChproofSchema {
    pub header: ChproofHeader,
    pub obligations: Vec<ProofObligation>,
    pub cache_facts: Vec<CacheProofFact>,
    pub invalidation_facts: Vec<InvalidationProofFact>,
}

impl ChproofSchema {
    pub fn header_magic_valid(&self) -> bool {
        &self.header.magic == ZCHPROOF_MAGIC
    }

    pub fn header_version_compatible(&self) -> bool {
        self.header.schema_version <= SCHEMA_VERSION
    }

    pub fn to_lean_proof_input(&self) -> LeanProofInputSchema {
        LeanProofInputSchema {
            version: self.header.schema_version,
            target: self.header.target.clone(),
            cache_facts: self
                .cache_facts
                .iter()
                .map(|fact| LeanCacheProofFact {
                    cache_key: fact.cache_key.clone(),
                    semantic_fingerprint_hex: hex::encode(fact.semantic_fingerprint),
                    dependency_fingerprints_hex: fact
                        .dependency_fingerprints
                        .iter()
                        .map(hex::encode)
                        .collect(),
                    schema_version: fact.schema_version,
                    target: fact.target.clone(),
                    build_options_hash_hex: hex::encode(fact.build_options_hash),
                    reusable: fact.reusable,
                })
                .collect(),
            invalidation_facts: self
                .invalidation_facts
                .iter()
                .map(|fact| LeanInvalidationProofFact {
                    changed_node: fact.changed_node,
                    reason: fact.reason.as_token().to_string(),
                    affected_exports: fact.affected_exports.clone(),
                    sound: fact.sound,
                })
                .collect(),
        }
    }

    pub fn to_lean_wire(&self) -> String {
        self.to_lean_proof_input().serialize()
    }

    pub fn from_lean_wire(wire: &str) -> Option<Self> {
        let lean = LeanProofInputSchema::deserialize(wire)?;
        let cache_facts = lean
            .cache_facts
            .into_iter()
            .map(|fact| {
                Some(CacheProofFact {
                    cache_key: fact.cache_key,
                    semantic_fingerprint: decode_hex_32(&fact.semantic_fingerprint_hex)?,
                    dependency_fingerprints: fact
                        .dependency_fingerprints_hex
                        .iter()
                        .map(|dep| decode_hex_32(dep))
                        .collect::<Option<Vec<_>>>()?,
                    schema_version: fact.schema_version,
                    target: fact.target,
                    build_options_hash: decode_hex_32(&fact.build_options_hash_hex)?,
                    reusable: fact.reusable,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let invalidation_facts = lean
            .invalidation_facts
            .into_iter()
            .map(|fact| {
                Some(InvalidationProofFact {
                    changed_node: fact.changed_node,
                    reason: InvalidationReason::from_token(&fact.reason)?,
                    affected_exports: fact.affected_exports,
                    sound: fact.sound,
                })
            })
            .collect::<Option<Vec<_>>>()?;

        Some(Self {
            header: ChproofHeader {
                magic: *ZCHPROOF_MAGIC,
                schema_version: lean.version,
                zig_commit: [0u8; 20],
                target: lean.target,
                timestamp_ns: 0,
                proof_count: (cache_facts.len() + invalidation_facts.len()) as u32,
                checksum: [0u8; 32],
            },
            obligations: vec![],
            cache_facts,
            invalidation_facts,
        })
    }
}

impl InvalidationReason {
    pub fn as_token(&self) -> &'static str {
        match self {
            Self::SourceChange => "source_change",
            Self::TypeChanged => "type_changed",
            Self::LayoutChanged => "layout_changed",
            Self::ComptimeChanged => "comptime_changed",
            Self::ExportChanged => "export_changed",
            Self::LinkChanged => "link_changed",
        }
    }

    pub fn from_token(value: &str) -> Option<Self> {
        match value {
            "source_change" => Some(Self::SourceChange),
            "type_changed" => Some(Self::TypeChanged),
            "layout_changed" => Some(Self::LayoutChanged),
            "comptime_changed" => Some(Self::ComptimeChanged),
            "export_changed" => Some(Self::ExportChanged),
            "link_changed" => Some(Self::LinkChanged),
            _ => None,
        }
    }
}

impl LeanProofInputSchema {
    pub fn serialize(&self) -> String {
        let mut rows = vec![format!("zig-proof-input|{}|{}", self.version, self.target)];
        for fact in &self.cache_facts {
            rows.push(format!(
                "cache|{}|{}|{}|{}|{}|{}|{}",
                fact.cache_key,
                fact.semantic_fingerprint_hex,
                fact.dependency_fingerprints_hex.join(","),
                fact.schema_version,
                fact.target,
                fact.build_options_hash_hex,
                if fact.reusable { "true" } else { "false" }
            ));
        }
        for fact in &self.invalidation_facts {
            rows.push(format!(
                "invalidate|{}|{}|{}|{}",
                fact.changed_node,
                fact.reason,
                fact.affected_exports
                    .iter()
                    .map(|export| export.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                if fact.sound { "true" } else { "false" }
            ));
        }
        rows.join("\n")
    }

    pub fn deserialize(wire: &str) -> Option<Self> {
        let mut rows = wire.lines();
        let header = rows.next()?;
        let mut header_parts = header.split('|');
        if header_parts.next()? != "zig-proof-input" {
            return None;
        }
        let version = header_parts.next()?.parse().ok()?;
        let target = header_parts.next()?.to_string();
        if header_parts.next().is_some() {
            return None;
        }

        let mut cache_facts = Vec::new();
        let mut invalidation_facts = Vec::new();

        for row in rows {
            let parts: Vec<_> = row.split('|').collect();
            match parts.as_slice() {
                ["cache", cache_key, semantic_fingerprint_hex, dependency_fingerprints_hex, schema_version, target, build_options_hash_hex, reusable] =>
                {
                    cache_facts.push(LeanCacheProofFact {
                        cache_key: (*cache_key).to_string(),
                        semantic_fingerprint_hex: (*semantic_fingerprint_hex).to_string(),
                        dependency_fingerprints_hex: if dependency_fingerprints_hex.is_empty() {
                            vec![]
                        } else {
                            dependency_fingerprints_hex
                                .split(',')
                                .map(str::to_string)
                                .collect()
                        },
                        schema_version: schema_version.parse().ok()?,
                        target: (*target).to_string(),
                        build_options_hash_hex: (*build_options_hash_hex).to_string(),
                        reusable: parse_bool(reusable)?,
                    });
                }
                ["invalidate", changed_node, reason, affected_exports, sound] => {
                    invalidation_facts.push(LeanInvalidationProofFact {
                        changed_node: changed_node.parse().ok()?,
                        reason: (*reason).to_string(),
                        affected_exports: if affected_exports.is_empty() {
                            vec![]
                        } else {
                            affected_exports
                                .split(',')
                                .map(str::parse)
                                .collect::<Result<Vec<_>, _>>()
                                .ok()?
                        },
                        sound: parse_bool(sound)?,
                    });
                }
                _ => return None,
            }
        }

        Some(Self {
            version,
            target,
            cache_facts,
            invalidation_facts,
        })
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn decode_hex_32(value: &str) -> Option<[u8; 32]> {
    let bytes = hex::decode(value).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_schema() -> ChproofSchema {
        ChproofSchema {
            header: ChproofHeader {
                magic: *ZCHPROOF_MAGIC,
                schema_version: 1,
                zig_commit: [0u8; 20],
                target: "x86_64-unknown-linux-gnu".to_string(),
                timestamp_ns: 7,
                proof_count: 2,
                checksum: [0u8; 32],
            },
            obligations: vec![],
            cache_facts: vec![CacheProofFact {
                cache_key: "cache-key-1".to_string(),
                semantic_fingerprint: [1u8; 32],
                dependency_fingerprints: vec![[2u8; 32], [3u8; 32]],
                schema_version: 1,
                target: "x86_64-unknown-linux-gnu".to_string(),
                build_options_hash: [4u8; 32],
                reusable: true,
            }],
            invalidation_facts: vec![InvalidationProofFact {
                changed_node: 9,
                reason: InvalidationReason::LayoutChanged,
                affected_exports: vec![11, 12],
                sound: true,
            }],
        }
    }

    #[test]
    fn test_lean_proof_input_roundtrip() {
        let schema = sample_schema();
        let wire = schema.to_lean_wire();
        let restored = ChproofSchema::from_lean_wire(&wire).expect("wire should deserialize");

        assert_eq!(restored.header.schema_version, schema.header.schema_version);
        assert_eq!(restored.header.target, schema.header.target);
        assert_eq!(restored.cache_facts, schema.cache_facts);
        assert_eq!(restored.invalidation_facts, schema.invalidation_facts);
    }

    #[test]
    fn test_lean_proof_input_rejects_bad_hex() {
        let wire = "zig-proof-input|1|x86_64-unknown-linux-gnu\ncache|cache-key|zz|aa|1|x86_64-unknown-linux-gnu|bb|true";
        assert!(ChproofSchema::from_lean_wire(wire).is_none());
    }
}
