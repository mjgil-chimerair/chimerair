use chimera_artifact::NativeLinkSpec;
use chimera_component::{ImportMap, ModuleMap, ProfileSpec, TargetSpec};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputKind {
    Obj,
    StaticLib,
    SharedLib,
    Exe,
}

impl OutputKind {
    pub fn file_extension(&self) -> &'static str {
        match self {
            OutputKind::Obj => "o",
            OutputKind::StaticLib => "a",
            OutputKind::SharedLib => "so",
            OutputKind::Exe => "",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigCompileContext {
    pub root_source: PathBuf,
    pub named_modules: ModuleMap,
    pub import_map: ImportMap,
    pub include_dirs: Vec<PathBuf>,
    pub library_search_paths: Vec<PathBuf>,
    pub linked_libraries: Vec<String>,
    pub object_inputs: Vec<PathBuf>,
    pub framework_paths: Vec<PathBuf>,
    pub frameworks: Vec<String>,
    pub rpaths: Vec<String>,
    pub target: Option<TargetSpec>,
    pub optimize: Option<ProfileSpec>,
    pub output_kind: OutputKind,
}

impl ZigCompileContext {
    pub fn new(root_source: PathBuf, output_kind: OutputKind) -> Self {
        Self {
            root_source,
            named_modules: ModuleMap::new(),
            import_map: ImportMap::new(),
            include_dirs: Vec::new(),
            library_search_paths: Vec::new(),
            linked_libraries: Vec::new(),
            object_inputs: Vec::new(),
            framework_paths: Vec::new(),
            frameworks: Vec::new(),
            rpaths: Vec::new(),
            target: None,
            optimize: None,
            output_kind,
        }
    }

    pub fn with_named_module(mut self, name: impl Into<String>, path: PathBuf) -> Self {
        self.named_modules.add_module(name, path);
        self
    }

    pub fn with_import_mapping(mut self, from: impl Into<String>, to: PathBuf) -> Self {
        self.import_map.add_mapping(from, to);
        self
    }

    pub fn has_duplicate_modules(&self) -> bool {
        let mut seen = std::collections::HashSet::new();
        for m in &self.named_modules.modules {
            if !seen.insert(&m.name) {
                return true;
            }
        }
        false
    }

    pub fn validate(&self) -> Result<(), ContextError> {
        if !self.root_source.exists() {
            return Err(ContextError::RootSourceNotFound(
                self.root_source.display().to_string(),
            ));
        }
        if self.has_duplicate_modules() {
            return Err(ContextError::DuplicateModuleName);
        }
        Ok(())
    }

    pub fn target_triple(&self) -> String {
        self.target
            .as_ref()
            .map(|t| t.triple.clone())
            .unwrap_or_else(|| "native".to_string())
    }

    pub fn optimize_level(&self) -> String {
        self.optimize
            .as_ref()
            .map(|p| format!("{:?}", p.opt_level))
            .unwrap_or_else(|| "Debug".to_string())
    }

    pub fn with_link_library(mut self, lib: impl Into<String>) -> Self {
        self.linked_libraries.push(lib.into());
        self
    }

    pub fn with_library_search_path(mut self, path: PathBuf) -> Self {
        self.library_search_paths.push(path);
        self
    }

    pub fn is_direct_link(&self) -> bool {
        !self.linked_libraries.is_empty() || !self.object_inputs.is_empty()
    }

    pub fn to_link_spec(&self) -> NativeLinkSpec {
        NativeLinkSpec {
            objects: self.object_inputs.clone(),
            static_archives: Vec::new(),
            shared_libraries: Vec::new(),
            library_search_paths: self.library_search_paths.clone(),
            link_libraries: self.linked_libraries.clone(),
            linker_args: Vec::new(),
            rpaths: self.rpaths.iter().map(|r| PathBuf::from(r)).collect(),
            runtime_files: Vec::new(),
            system_libraries: Vec::new(),
        }
    }

    pub fn is_runtime_dlopen(&self) -> bool {
        !self.rpaths.is_empty()
            || self.linked_libraries.is_empty()
                && self.object_inputs.is_empty()
                && self.rpaths.is_empty()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ContextError {
    #[error("root source not found: {0}")]
    RootSourceNotFound(String),
    #[error("duplicate module name in module map")]
    DuplicateModuleName,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_basic_context_creation() {
        let ctx = ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::Obj);
        assert_eq!(ctx.root_source, PathBuf::from("main.zig"));
        assert_eq!(ctx.output_kind, OutputKind::Obj);
        assert!(ctx.named_modules.modules.is_empty());
        assert!(ctx.import_map.mappings.is_empty());
    }

    #[test]
    fn test_with_named_module() {
        let ctx = ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::StaticLib)
            .with_named_module("foo", PathBuf::from("foo.zig"))
            .with_named_module("bar", PathBuf::from("bar.zig"));
        assert_eq!(ctx.named_modules.modules.len(), 2);
        assert!(ctx.named_modules.get("foo").is_some());
        assert!(ctx.named_modules.get("bar").is_some());
    }

    #[test]
    fn test_duplicate_module_detection() {
        let ctx = ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::Exe)
            .with_named_module("foo", PathBuf::from("foo.zig"))
            .with_named_module("foo", PathBuf::from("foo2.zig"));
        assert!(ctx.has_duplicate_modules());
    }

    #[test]
    fn test_no_duplicate_modules() {
        let ctx = ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::Obj)
            .with_named_module("foo", PathBuf::from("foo.zig"))
            .with_named_module("bar", PathBuf::from("bar.zig"));
        assert!(!ctx.has_duplicate_modules());
    }

    #[test]
    fn test_validate_root_source_missing() {
        let ctx = ZigCompileContext::new(PathBuf::from("/nonexistent/path.zig"), OutputKind::Obj);
        let result = ctx.validate();
        assert!(matches!(result, Err(ContextError::RootSourceNotFound(_))));
    }

    #[test]
    fn test_validate_root_source_exists() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("main.zig");
        fs::write(&source, "export fn main() void {}").unwrap();
        let ctx = ZigCompileContext::new(source, OutputKind::Obj);
        assert!(ctx.validate().is_ok());
    }

    #[test]
    fn test_validate_duplicate_module() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("main.zig");
        fs::write(&source, "export fn main() void {}").unwrap();
        let ctx = ZigCompileContext::new(source, OutputKind::Obj)
            .with_named_module("dup", PathBuf::from("a.zig"))
            .with_named_module("dup", PathBuf::from("b.zig"));
        let result = ctx.validate();
        assert!(matches!(result, Err(ContextError::DuplicateModuleName)));
    }

    #[test]
    fn test_target_triple_default() {
        let ctx = ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::Obj);
        assert_eq!(ctx.target_triple(), "native");
    }

    #[test]
    fn test_target_triple_with_spec() {
        let target = Some(TargetSpec::new("aarch64-linux-musl"));
        let ctx = ZigCompileContext {
            target,
            ..ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::Obj)
        };
        assert_eq!(ctx.target_triple(), "aarch64-linux-musl");
    }

    #[test]
    fn test_output_kind_extensions() {
        assert_eq!(OutputKind::Obj.file_extension(), "o");
        assert_eq!(OutputKind::StaticLib.file_extension(), "a");
        assert_eq!(OutputKind::SharedLib.file_extension(), "so");
        assert_eq!(OutputKind::Exe.file_extension(), "");
    }

    #[test]
    fn test_with_import_mapping() {
        let ctx = ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::Obj)
            .with_import_mapping("std", PathBuf::from("/zig/std"))
            .with_import_mapping("foo", PathBuf::from("foo.zig"));
        assert_eq!(ctx.import_map.mappings.len(), 2);
        assert_eq!(
            ctx.import_map.resolve("std").unwrap(),
            &PathBuf::from("/zig/std")
        );
    }

    #[test]
    fn test_direct_link_detection() {
        let ctx = ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::StaticLib)
            .with_link_library("c");
        assert!(ctx.is_direct_link());
    }

    #[test]
    fn test_no_direct_link() {
        let ctx = ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::Obj);
        assert!(!ctx.is_direct_link());
    }

    #[test]
    fn test_to_link_spec() {
        let ctx = ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::Exe)
            .with_link_library("foo")
            .with_library_search_path(PathBuf::from("/usr/local/lib"))
            .with_named_module("lib", PathBuf::from("lib.zig"));

        let spec = ctx.to_link_spec();
        assert!(spec.link_libraries.contains(&"foo".to_string()));
        assert!(spec
            .library_search_paths
            .contains(&PathBuf::from("/usr/local/lib")));
    }

    #[test]
    fn test_direct_link_with_object_inputs() {
        let mut ctx = ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::Exe);
        ctx.object_inputs.push(PathBuf::from("helper.o"));
        assert!(ctx.is_direct_link());
    }

    #[test]
    fn test_runtime_dlopen_with_rpaths() {
        let mut ctx = ZigCompileContext::new(PathBuf::from("loader.zig"), OutputKind::Exe);
        ctx.rpaths.push("$ORIGIN/../lib".to_string());
        assert!(ctx.is_runtime_dlopen());
    }

    #[test]
    fn test_runtime_dlopen_no_link_libs() {
        let ctx = ZigCompileContext::new(PathBuf::from("loader.zig"), OutputKind::Exe);
        // No linked libs, no objects -> runtime dlopen scenario
        assert!(ctx.is_runtime_dlopen());
    }

    #[test]
    fn test_to_link_spec_with_rpaths() {
        let mut ctx = ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::Exe);
        ctx.rpaths.push("$ORIGIN/../lib".to_string());
        let spec = ctx.to_link_spec();
        assert_eq!(spec.rpaths.len(), 1);
    }

    #[test]
    fn test_direct_link_external_c_symbol() {
        let ctx = ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::Exe)
            .with_link_library("m")
            .with_link_library("pthread");
        let spec = ctx.to_link_spec();
        assert!(spec.link_libraries.contains(&"m".to_string()));
        assert!(spec.link_libraries.contains(&"pthread".to_string()));
    }

    #[test]
    fn test_context_serialization() {
        let ctx = ZigCompileContext::new(PathBuf::from("main.zig"), OutputKind::Exe)
            .with_named_module("lib", PathBuf::from("lib.zig"));
        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: ZigCompileContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.root_source, PathBuf::from("main.zig"));
        assert_eq!(deserialized.named_modules.modules.len(), 1);
        assert_eq!(deserialized.output_kind, OutputKind::Exe);
    }
}
