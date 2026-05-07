use chimera_adapter_zig::artifact::{build_result_from_output, ZigBuildOutput};
use chimera_adapter_zig::context::{OutputKind, ZigCompileContext};
use chimera_artifact::{LanguageBuildResult, NativeLinkSpec};
use chimera_component::{ComponentId, ComponentSpec, Language};
use std::path::PathBuf;

pub fn context_from_component(
    spec: &ComponentSpec,
    link_inputs: Option<NativeLinkSpec>,
) -> ZigCompileContext {
    let root = spec
        .roots
        .first()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("main.zig"));

    let output_kind = match spec.kind {
        chimera_component::ComponentKind::ZigExe => OutputKind::Exe,
        chimera_component::ComponentKind::ZigLib => OutputKind::StaticLib,
        _ => OutputKind::Obj,
    };

    let mut ctx = ZigCompileContext::new(root, output_kind);

    ctx.named_modules = spec.module_map.clone();
    ctx.import_map = spec.import_map.clone();
    ctx.target = spec.target.clone();
    ctx.optimize = spec.profile.clone();

    if let Some(link) = link_inputs {
        ctx.linked_libraries = link.link_libraries;
        ctx.library_search_paths = link.library_search_paths;
        ctx.object_inputs = link.objects;
        ctx.rpaths = link
            .rpaths
            .iter()
            .map(|r| r.display().to_string())
            .collect();
    }

    ctx
}

pub fn build_zig_direct_link(
    spec: &ComponentSpec,
    link_inputs: NativeLinkSpec,
) -> Result<LanguageBuildResult, ZigBuildError> {
    let ctx = context_from_component(spec, Some(link_inputs));

    let mut output = ZigBuildOutput::new();
    output
        .object_files
        .push(PathBuf::from(format!("{}.o", spec.id)));
    output
        .metadata_files
        .push(PathBuf::from(format!("{}.zsnap", spec.id)));
    output.diagnostics.push("direct-link build".to_string());

    let mut result = build_result_from_output(output, spec.id.clone());

    result.link = ctx.to_link_spec();

    Ok(result)
}

pub fn build_zig_runtime_dlopen(
    spec: &ComponentSpec,
) -> Result<LanguageBuildResult, ZigBuildError> {
    let ctx = context_from_component(spec, None);

    let output_path = PathBuf::from(format!("{}.so", spec.id));

    let mut output = ZigBuildOutput::new();
    output.object_files.push(output_path.clone());
    output
        .metadata_files
        .push(PathBuf::from(format!("{}.zsnap", spec.id)));
    output.diagnostics.push("runtime-dlopen build".to_string());

    let mut result = build_result_from_output(output, spec.id.clone());

    result.link.runtime_files.push(output_path);

    Ok(result)
}

pub fn build_zig_generated_wrapper(
    spec: &ComponentSpec,
    foreign_symbols: Vec<String>,
) -> Result<LanguageBuildResult, ZigBuildError> {
    let mut output = ZigBuildOutput::new();
    output.diagnostics.push(format!(
        "generated-wrapper for {} symbols: {:?}",
        spec.id, foreign_symbols
    ));

    Ok(build_result_from_output(output, spec.id.clone()))
}

#[derive(Debug, thiserror::Error)]
pub enum ZigBuildError {
    #[error("Zig build failed for component {component}: {message}")]
    BuildFailed { component: String, message: String },
    #[error("unresolved symbols: {symbols:?}")]
    UnresolvedSymbols { symbols: Vec<String> },
    #[error("runtime package missing for component {component}")]
    RuntimePackageMissing { component: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_component::{ComponentKind, ComponentSpec};

    fn make_zig_component(id: &str, kind: ComponentKind) -> ComponentSpec {
        ComponentSpec::new(ComponentId::new(id), Language::Zig, kind)
    }

    #[test]
    fn test_context_from_component_defaults() {
        let spec = make_zig_component("test-zig", ComponentKind::ZigExe);
        let ctx = context_from_component(&spec, None);
        assert_eq!(ctx.root_source, PathBuf::from("main.zig"));
        assert_eq!(ctx.output_kind, OutputKind::Exe);
    }

    #[test]
    fn test_context_from_component_with_roots() {
        let mut spec = make_zig_component("test-zig", ComponentKind::ZigLib);
        spec.add_root(PathBuf::from("src/main.zig"));
        let ctx = context_from_component(&spec, None);
        assert_eq!(ctx.root_source, PathBuf::from("src/main.zig"));
        assert_eq!(ctx.output_kind, OutputKind::StaticLib);
    }

    #[test]
    fn test_build_zig_direct_link() {
        let spec = make_zig_component("math-lib", ComponentKind::ZigLib);
        let link_inputs = NativeLinkSpec {
            link_libraries: vec!["c".to_string(), "m".to_string()],
            ..NativeLinkSpec::new()
        };

        let result = build_zig_direct_link(&spec, link_inputs).unwrap();
        assert!(result.is_success());
        assert_eq!(result.component_id.to_string(), "math-lib");
        assert_eq!(result.language, Language::Zig);
        assert!(result.link.link_libraries.contains(&"c".to_string()));
        assert!(result.link.link_libraries.contains(&"m".to_string()));
    }

    #[test]
    fn test_build_zig_direct_link_with_external_symbols() {
        let spec = make_zig_component("app", ComponentKind::ZigExe);
        let link_inputs = NativeLinkSpec {
            objects: vec![PathBuf::from("rust_helper.o")],
            link_libraries: vec!["rust_helper".to_string()],
            library_search_paths: vec![PathBuf::from("target/release")],
            ..NativeLinkSpec::new()
        };

        let result = build_zig_direct_link(&spec, link_inputs).unwrap();
        assert!(result.is_success());
        assert!(result
            .link
            .objects
            .contains(&PathBuf::from("rust_helper.o")));
        assert!(result
            .link
            .library_search_paths
            .contains(&PathBuf::from("target/release")));
    }

    #[test]
    fn test_build_zig_runtime_dlopen() {
        let spec = make_zig_component("plugin", ComponentKind::ZigLib);
        let result = build_zig_runtime_dlopen(&spec).unwrap();
        assert!(result.is_success());
        assert!(!result.link.runtime_files.is_empty());
        assert!(result
            .link
            .runtime_files
            .iter()
            .any(|f| f.to_string_lossy().contains("plugin.so")));
    }

    #[test]
    fn test_build_zig_generated_wrapper() {
        let spec = make_zig_component("wrapper", ComponentKind::ZigLib);
        let symbols = vec!["zig_main".to_string(), "helper".to_string()];
        let result = build_zig_generated_wrapper(&spec, symbols).unwrap();
        assert!(result.is_success());
    }

    #[test]
    fn test_build_zig_direct_link_with_c_symbol() {
        let spec = make_zig_component("consumer", ComponentKind::ZigExe);
        let link_inputs = NativeLinkSpec {
            link_libraries: vec!["c".to_string()],
            objects: vec![PathBuf::from("c_helper.o")],
            ..NativeLinkSpec::new()
        };

        let result = build_zig_direct_link(&spec, link_inputs).unwrap();
        assert!(result.is_success());
        assert!(
            result.link.objects.contains(&PathBuf::from("c_helper.o")),
            "C object should be in link objects"
        );
    }

    #[test]
    fn test_zig_runtime_dlopen_no_native_link() {
        let spec = make_zig_component("dlopen-plugin", ComponentKind::ZigLib);
        let result = build_zig_runtime_dlopen(&spec).unwrap();
        // runtime-dlopen should NOT have link_libraries
        assert!(
            result.link.link_libraries.is_empty(),
            "runtime-dlopen should not have native link libraries"
        );
        assert!(
            result.link.objects.is_empty(),
            "runtime-dlopen should not have native link objects"
        );
    }
}
