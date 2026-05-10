//! Artifact manifest schema for Zigmera artifacts.

use serde::{Deserialize, Serialize};

/// Kind of Zigmera artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtifactKind {
    Zsnap,
    Zdep,
    Zairpack,
    Zchmeta,
    Zchproof,
    Chobject,
    Chir,
}

impl ArtifactKind {
    pub fn file_extension(&self) -> &'static str {
        match self {
            ArtifactKind::Zsnap => ".zsnap",
            ArtifactKind::Zdep => ".zdep",
            ArtifactKind::Zairpack => ".zairpack",
            ArtifactKind::Zchmeta => ".zchmeta",
            ArtifactKind::Zchproof => ".zchproof",
            ArtifactKind::Chobject => ".cho",
            ArtifactKind::Chir => ".chir",
        }
    }
}

/// Entry in the artifact manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactEntry {
    pub kind: ArtifactKind,
    pub path: String,
    pub size_bytes: u64,
    pub checksum: [u8; 32],
    pub created_ns: u64,
    pub schema_version: u32,
}

/// Artifact manifest containing all produced artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactManifest {
    pub version: String,
    pub zig_commit: String,
    pub target: String,
    pub artifacts: Vec<ArtifactEntry>,
}

impl ArtifactManifest {
    pub fn new(version: &str, zig_commit: &str, target: &str) -> Self {
        Self {
            version: version.to_string(),
            zig_commit: zig_commit.to_string(),
            target: target.to_string(),
            artifacts: Vec::new(),
        }
    }

    pub fn add_artifact(&mut self, entry: ArtifactEntry) {
        self.artifacts.push(entry);
    }

    pub fn find_artifact(&self, kind: ArtifactKind) -> Option<&ArtifactEntry> {
        self.artifacts.iter().find(|a| a.kind == kind)
    }
}
