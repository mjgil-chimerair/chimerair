use chimera_artifact::{Diagnostic, DiagnosticSeverity, LanguageBuildResult};
use chimera_component::{ComponentSpec, Language};
use std::path::PathBuf;

pub fn build_chimera_module(
    spec: &ComponentSpec,
    emit_metadata: bool,
    emit_proof: bool,
    emit_object: bool,
) -> Result<LanguageBuildResult, ChimeraBuildError> {
    let mut result = LanguageBuildResult::new(spec.id.clone(), Language::Unknown);

    for root in &spec.roots {
        let ext = root.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "cho" | "o" | "obj" | "a" => {
                if emit_object {
                    result.primary_outputs.add_object(root.clone());
                }
            }
            "chmeta" => {
                if emit_metadata {
                    result.metadata.chmeta.push(root.clone());
                }
            }
            "chimera" | "chir" | "mlir" => {
                if emit_metadata {
                    result.primary_outputs.add_chimera_ir(root.clone());
                }
            }
            "chproof" => {
                if emit_proof {
                    result.primary_outputs.add_proof(root.clone());
                    result.proof.chproof.push(root.clone());
                }
            }
            _ => {
                // Ignore other extensions
            }
        }
    }

    // Simulate compiler-core lowering for ChimeraIR-bearing components.
    if emit_object
        && spec.roots.iter().any(|r| {
            let ext = r.extension().and_then(|e| e.to_str()).unwrap_or("");
            ext == "chimera" || ext == "chir" || ext == "mlir"
        })
    {
        result
            .primary_outputs
            .add_object(PathBuf::from(format!("{}.cho", spec.id)));
    }

    result.diagnostics.push(Diagnostic {
        severity: DiagnosticSeverity::Info,
        code: "CHIMERA_CORE".to_string(),
        message: format!(
            "processed {} with flags meta={} proof={} obj={}",
            spec.id, emit_metadata, emit_proof, emit_object
        ),
        location: None,
        suggestions: Vec::new(),
    });

    Ok(result)
}

#[derive(Debug, thiserror::Error)]
pub enum ChimeraBuildError {
    #[error("compiler-core failed for {component}: {message}")]
    BuildFailed { component: String, message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_component::{ComponentId, ComponentKind};

    fn make_chimera_component(id: &str, roots: Vec<&str>) -> ComponentSpec {
        let mut spec = ComponentSpec::new(
            ComponentId::new(id),
            Language::Unknown,
            ComponentKind::ChimeraModule,
        );
        for r in roots {
            spec.add_root(PathBuf::from(r));
        }
        spec
    }

    #[test]
    fn test_pure_chimera_component() {
        let spec = make_chimera_component("pure-ir", vec!["src/main.chimera"]);
        let result = build_chimera_module(&spec, true, true, true).unwrap();

        assert!(result.is_success());
        assert!(result
            .primary_outputs
            .chimera_ir
            .contains(&PathBuf::from("src/main.chimera")));
        assert!(result
            .primary_outputs
            .objects
            .contains(&PathBuf::from("pure-ir.cho")));
    }

    #[test]
    fn test_mixed_native_chimera_component() {
        let spec = make_chimera_component("mixed", vec!["src/lib.chir", "native.o"]);
        let result = build_chimera_module(&spec, true, false, true).unwrap();

        assert!(result.is_success());
        assert!(result
            .primary_outputs
            .chimera_ir
            .contains(&PathBuf::from("src/lib.chir")));
        assert!(result
            .primary_outputs
            .objects
            .contains(&PathBuf::from("native.o")));
        assert!(result
            .primary_outputs
            .objects
            .contains(&PathBuf::from("mixed.cho")));
    }

    #[test]
    fn test_metadata_proof_object_emission_flags() {
        let spec =
            make_chimera_component("flags", vec!["src/lib.chir", "proof.chproof", "lib.cho"]);

        // Only metadata
        let r1 = build_chimera_module(&spec, true, false, false).unwrap();
        assert_eq!(r1.primary_outputs.chimera_ir.len(), 1);
        assert!(r1.primary_outputs.proofs.is_empty());
        assert!(r1.primary_outputs.objects.is_empty());

        // Only proof
        let r2 = build_chimera_module(&spec, false, true, false).unwrap();
        assert!(r2.primary_outputs.chimera_ir.is_empty());
        assert_eq!(r2.primary_outputs.proofs.len(), 1);
        assert!(r2.primary_outputs.objects.is_empty());

        // Only object
        let r3 = build_chimera_module(&spec, false, false, true).unwrap();
        assert!(r3.primary_outputs.chimera_ir.is_empty());
        assert!(r3.primary_outputs.proofs.is_empty());
        assert_eq!(r3.primary_outputs.objects.len(), 2); // lib.cho + flags.cho
    }

    #[test]
    fn test_chmeta_and_chproof_keep_their_dedicated_envelopes() {
        let spec =
            make_chimera_component("envelopes", vec!["build/module.chmeta", "proof.chproof"]);
        let result = build_chimera_module(&spec, true, true, false).unwrap();

        assert!(result
            .metadata
            .chmeta
            .contains(&PathBuf::from("build/module.chmeta")));
        assert!(result
            .proof
            .chproof
            .contains(&PathBuf::from("proof.chproof")));
        assert!(result.primary_outputs.chimera_ir.is_empty());
        assert!(result
            .primary_outputs
            .proofs
            .contains(&PathBuf::from("proof.chproof")));
    }
}
