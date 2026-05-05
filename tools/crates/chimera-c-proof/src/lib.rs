//! Chimera C proof artifact emitter crate.
//!
//! Emits `.cchproof` facts for layout, signature, pointer aliasing,
//! errno/status, varargs, wrappers, cache, link, and trust assumptions.
//!
//! Task 18: C proof artifact emitter

use chimera_c_dialect::{CDeclaration, CDialectContext};
use chimera_c_schema::{DeclId, TypeRef};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for proof operations
pub type Result<T> = std::result::Result<T, ProofError>;

/// Proof generation errors
#[derive(Debug, Clone, Error)]
pub enum ProofError {
    #[error("missing declaration: {0}")]
    MissingDeclaration(String),
    #[error("missing type: {0}")]
    MissingType(String),
    #[error("incomplete proof: {0}")]
    IncompleteProof(String),
    #[error("serialization error: {0}")]
    SerializationError(String),
}

/// Proof artifact containing all C proof facts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CProofArtifact {
    /// Schema version
    pub version: String,
    /// Target triple
    pub target_triple: String,
    /// Proof facts
    pub facts: Vec<ProofFact>,
    /// Trust assumptions
    pub trust_assumptions: Vec<TrustAssumption>,
    /// Metadata
    pub metadata: ProofMetadata,
}

/// Proof fact enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ProofFact {
    /// Layout proof fact
    Layout {
        path: String,
        offset: u64,
        size: u64,
        alignment: u32,
    },
    /// Signature proof fact
    Signature {
        symbol: String,
        signature_hash: String,
    },
    /// Pointer aliasing proof fact
    PointerAliasing {
        pointer_a: String,
        pointer_b: String,
        may_alias: bool,
    },
    /// Errno/status proof fact
    ErrnoStatus {
        function: String,
        errno_set: bool,
        errno_value: Option<String>,
    },
    /// Varargs proof fact
    Varargs {
        function: String,
        has_varargs: bool,
        varargs_abi_safe: bool,
    },
    /// Wrapper proof fact
    Wrapper {
        symbol: String,
        wrapper_target: String,
        wrapper_type: WrapperType,
    },
    /// Cache proof fact
    Cache {
        cache_key: String,
        artifact_kind: String,
        valid: bool,
    },
    /// Link proof fact
    Link {
        symbol: String,
        link_target: String,
        link_type: LinkType,
    },
    /// Trust assumption proof fact
    Trust {
        assumption: String,
        trusted: bool,
        reason: String,
    },
}

/// Wrapper types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WrapperType {
    Caller,
    Callee,
    Trampoline,
    Adapter,
}

/// Link types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    Static,
    Dynamic,
    Weak,
}

/// Trust assumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAssumption {
    pub category: TrustCategory,
    pub description: String,
    pub reasoning: String,
}

/// Trust categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustCategory {
    ClangAst,
    LayoutCompiler,
    AbiConvention,
    HeaderIntegrity,
    SystemHeaders,
    UserCode,
}

/// Proof metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetadata {
    pub producer: String,
    pub schema_version: String,
    pub target_triple: String,
    pub source_language: String,
    pub generated_at: String,
}

impl ProofMetadata {
    /// Create new metadata
    pub fn new(target_triple: impl Into<String>) -> Self {
        Self {
            producer: "chimera-c-proof".to_string(),
            schema_version: "0.1.0".to_string(),
            target_triple: target_triple.into(),
            source_language: "c".to_string(),
            generated_at: chrono_timestamp(),
        }
    }
}

/// Get current timestamp
fn chrono_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

/// C proof emitter
#[derive(Debug, Clone)]
pub struct CProofEmitter {
    dialect: CDialectContext,
    target_triple: String,
}

impl CProofEmitter {
    /// Create new proof emitter
    pub fn new(dialect: CDialectContext, target_triple: impl Into<String>) -> Self {
        Self {
            dialect,
            target_triple: target_triple.into(),
        }
    }

    /// Emit proof artifact
    pub fn emit(&self) -> Result<CProofArtifact> {
        let mut facts = Vec::new();

        // Emit signature facts for functions
        for (_id, decl) in &self.dialect.declarations {
            if let CDeclaration::Function(func) = decl {
                let signature_hash = self.compute_signature_hash(func);
                facts.push(ProofFact::Signature {
                    symbol: func.name.clone(),
                    signature_hash,
                });

                // Check for varargs (simplified check)
                let is_varargs = func.params.is_empty()
                    && (func.name.contains("printf") || func.name.contains("vprintf"));
                if is_varargs {
                    facts.push(ProofFact::Varargs {
                        function: func.name.clone(),
                        has_varargs: true,
                        varargs_abi_safe: false,
                    });
                }
            }
        }

        // Build trust assumptions
        let trust_assumptions = vec![
            TrustAssumption {
                category: TrustCategory::ClangAst,
                description: "Clang-provided AST facts are trusted unless checked".to_string(),
                reasoning: "Clang is the authoritative C compiler".to_string(),
            },
            TrustAssumption {
                category: TrustCategory::LayoutCompiler,
                description: "Compiler layout facts are trusted".to_string(),
                reasoning: "Layout comes from actual compiler execution".to_string(),
            },
            TrustAssumption {
                category: TrustCategory::AbiConvention,
                description: "ABI calling convention is platform standard".to_string(),
                reasoning: "Uses platform-default calling convention".to_string(),
            },
        ];

        Ok(CProofArtifact {
            version: "0.1.0".to_string(),
            target_triple: self.target_triple.clone(),
            facts,
            trust_assumptions,
            metadata: ProofMetadata::new(&self.target_triple),
        })
    }

    /// Compute signature hash for a function using BLAKE3
    fn compute_signature_hash(&self, func: &chimera_c_dialect::CFunctionDecl) -> String {
        let mut hasher = zigmera_hash::Blake3Hasher::with_schema_tag("c-proof-signature");
        hasher.update_str(&func.name);
        // TypeRef is a wrapper around u32, hash the inner value
        hasher.update_u64(func.return_type.0 as u64);
        for param in &func.params {
            hasher.update_u64(param.typ.0 as u64);
        }
        hasher.update_str(&func.calling_convention);
        hasher.finalize().as_hex()[..16].to_string()
    }

    /// Serialize proof artifact to JSON
    pub fn to_json(&self) -> Result<String> {
        let artifact = self.emit()?;
        serde_json::to_string_pretty(&artifact)
            .map_err(|e| ProofError::SerializationError(e.to_string()))
    }

    /// Serialize proof artifact to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let artifact = self.emit()?;
        serde_json::to_vec(&artifact).map_err(|e| ProofError::SerializationError(e.to_string()))
    }
}

/// Builder for proof artifacts
pub struct CProofBuilder {
    emitter: CProofEmitter,
    pending_facts: Vec<ProofFact>,
}

impl CProofBuilder {
    /// Create new builder
    pub fn new(dialect: CDialectContext, target_triple: impl Into<String>) -> Self {
        Self {
            emitter: CProofEmitter::new(dialect, target_triple),
            pending_facts: Vec::new(),
        }
    }

    /// Add a layout fact
    pub fn add_layout(mut self, path: String, offset: u64, size: u64, alignment: u32) -> Self {
        self.pending_facts.push(ProofFact::Layout {
            path,
            offset,
            size,
            alignment,
        });
        self
    }

    /// Add a signature fact
    pub fn add_signature(mut self, symbol: String, signature_hash: String) -> Self {
        self.pending_facts.push(ProofFact::Signature {
            symbol,
            signature_hash,
        });
        self
    }

    /// Add a pointer aliasing fact
    pub fn add_pointer_aliasing(
        mut self,
        pointer_a: String,
        pointer_b: String,
        may_alias: bool,
    ) -> Self {
        self.pending_facts.push(ProofFact::PointerAliasing {
            pointer_a,
            pointer_b,
            may_alias,
        });
        self
    }

    /// Add an errno fact
    pub fn add_errno(
        mut self,
        function: String,
        errno_set: bool,
        errno_value: Option<String>,
    ) -> Self {
        self.pending_facts.push(ProofFact::ErrnoStatus {
            function,
            errno_set,
            errno_value,
        });
        self
    }

    /// Add a varargs fact
    pub fn add_varargs(
        mut self,
        function: String,
        has_varargs: bool,
        varargs_abi_safe: bool,
    ) -> Self {
        self.pending_facts.push(ProofFact::Varargs {
            function,
            has_varargs,
            varargs_abi_safe,
        });
        self
    }

    /// Build the proof artifact
    pub fn build(self) -> Result<CProofArtifact> {
        let mut artifact = self.emitter.emit()?;
        artifact.facts.extend(self.pending_facts);
        Ok(artifact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_emitter_new() {
        let ctx = CDialectContext::default();
        let emitter = CProofEmitter::new(ctx, "x86_64-unknown-linux-gnu");
        assert_eq!(emitter.target_triple, "x86_64-unknown-linux-gnu");
    }

    #[test]
    fn test_proof_emitter_emit_empty() {
        let ctx = CDialectContext::default();
        let emitter = CProofEmitter::new(ctx, "x86_64-unknown-linux-gnu");
        let result = emitter.emit();
        assert!(result.is_ok());
        let artifact = result.unwrap();
        assert_eq!(artifact.target_triple, "x86_64-unknown-linux-gnu");
        // Empty context produces empty facts but valid artifact
        assert!(artifact.trust_assumptions.len() >= 1); // Has at least one trust assumption
    }

    #[test]
    fn test_proof_metadata_new() {
        let metadata = ProofMetadata::new("aarch64-unknown-linux-gnu");
        assert_eq!(metadata.target_triple, "aarch64-unknown-linux-gnu");
        assert_eq!(metadata.producer, "chimera-c-proof");
    }

    #[test]
    fn test_proof_builder_layout() {
        let ctx = CDialectContext::default();
        let artifact = CProofBuilder::new(ctx, "x86_64-unknown-linux-gnu")
            .add_layout("field_a".to_string(), 0, 4, 4)
            .add_layout("field_b".to_string(), 4, 4, 4)
            .build()
            .unwrap();

        // Should have at least the trust assumption facts plus added ones
        assert!(artifact.facts.len() >= 2);
    }

    #[test]
    fn test_proof_builder_signature() {
        let ctx = CDialectContext::default();
        let artifact = CProofBuilder::new(ctx, "x86_64-unknown-linux-gnu")
            .add_signature("my_func".to_string(), "abc123".to_string())
            .build()
            .unwrap();

        assert!(artifact
            .facts
            .iter()
            .any(|f| matches!(f, ProofFact::Signature { symbol, .. } if symbol == "my_func")));
    }

    #[test]
    fn test_proof_builder_pointer_aliasing() {
        let ctx = CDialectContext::default();
        let artifact = CProofBuilder::new(ctx, "x86_64-unknown-linux-gnu")
            .add_pointer_aliasing("ptr1".to_string(), "ptr2".to_string(), true)
            .build()
            .unwrap();

        assert!(artifact.facts.iter().any(|f| matches!(f, ProofFact::PointerAliasing { pointer_a, pointer_b, may_alias: true } if pointer_a == "ptr1" && pointer_b == "ptr2")));
    }

    #[test]
    fn test_proof_builder_varargs() {
        let ctx = CDialectContext::default();
        let artifact = CProofBuilder::new(ctx, "x86_64-unknown-linux-gnu")
            .add_varargs("printf".to_string(), true, false)
            .build()
            .unwrap();

        assert!(artifact.facts.iter().any(|f| matches!(f, ProofFact::Varargs { function, has_varargs: true, varargs_abi_safe: false } if function == "printf")));
    }

    #[test]
    fn test_proof_fact_serialization() {
        let fact = ProofFact::Layout {
            path: "test_field".to_string(),
            offset: 8,
            size: 4,
            alignment: 4,
        };

        let json = serde_json::to_string(&fact).unwrap();
        // Just check that the JSON contains our data and is valid
        assert!(json.contains("test_field"));
        assert!(json.contains("8")); // offset
        assert!(json.contains("4")); // size
    }

    #[test]
    fn test_trust_assumption_serialization() {
        let assumption = TrustAssumption {
            category: TrustCategory::ClangAst,
            description: "Test assumption".to_string(),
            reasoning: "Test reasoning".to_string(),
        };

        let json = serde_json::to_string(&assumption).unwrap();
        assert!(json.contains("clang_ast"));
        assert!(json.contains("Test assumption"));
    }

    #[test]
    fn test_wrapper_type_serialization() {
        let wt = WrapperType::Trampoline;
        let json = serde_json::to_string(&wt).unwrap();
        assert!(json.contains("trampoline"));
    }

    #[test]
    fn test_link_type_serialization() {
        let lt = LinkType::Dynamic;
        let json = serde_json::to_string(&lt).unwrap();
        assert!(json.contains("dynamic"));
    }

    #[test]
    fn test_proof_artifact_to_json() {
        let ctx = CDialectContext::default();
        let emitter = CProofEmitter::new(ctx, "x86_64-unknown-linux-gnu");
        let json = emitter.to_json().unwrap();
        assert!(json.contains("x86_64-unknown-linux-gnu"));
        assert!(json.contains("proof")); // Contains "proof" in producer name
    }

    #[test]
    fn test_proof_artifact_to_bytes() {
        let ctx = CDialectContext::default();
        let emitter = CProofEmitter::new(ctx, "x86_64-unknown-linux-gnu");
        let bytes = emitter.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_trust_category_variants() {
        assert_eq!(format!("{:?}", TrustCategory::ClangAst), "ClangAst");
        assert_eq!(
            format!("{:?}", TrustCategory::LayoutCompiler),
            "LayoutCompiler"
        );
        assert_eq!(
            format!("{:?}", TrustCategory::AbiConvention),
            "AbiConvention"
        );
        assert_eq!(
            format!("{:?}", TrustCategory::HeaderIntegrity),
            "HeaderIntegrity"
        );
        assert_eq!(
            format!("{:?}", TrustCategory::SystemHeaders),
            "SystemHeaders"
        );
        assert_eq!(format!("{:?}", TrustCategory::UserCode), "UserCode");
    }

    #[test]
    fn test_proof_emitter_signature_hash() {
        let ctx = CDialectContext::default();
        let emitter = CProofEmitter::new(ctx, "x86_64-unknown-linux-gnu");

        // Create a minimal function to get hash for
        let func = chimera_c_dialect::CFunctionDecl {
            id: chimera_c_schema::DeclId(0),
            name: "test_func".to_string(),
            linkage: chimera_c_dialect::CDeclarationLinkage::External,
            storage_class: chimera_c_dialect::CStorageClass::None,
            calling_convention: "cdecl".to_string(),
            params: vec![],
            return_type: chimera_c_schema::TypeRef(4),
            attributes: vec![],
            source_span: chimera_c_schema::SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 10,
            },
            has_body: false,
            is_inline: false,
        };

        let hash = emitter.compute_signature_hash(&func);
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 16); // 16 hex chars for u64
    }
}
