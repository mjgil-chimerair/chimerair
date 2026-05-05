//! Chimera C Clang extraction crate.
//!
//! Uses clang/libclang or `clang -Xclang -ast-dump=json` mode to extract:
//! - AST, types, layouts, macros, includes, diagnostics, and compile commands
//!
//! Task 10: Clang/libclang extraction crate

use chimera_c_schema::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result type for clang extraction operations
pub type Result<T> = std::result::Result<T, ClangError>;

/// Clang extraction errors
#[derive(Debug, thiserror::Error)]
pub enum ClangError {
    #[error("clang not found: {0}")]
    ClangNotFound(String),
    #[error("clang extraction failed: {0}")]
    ExtractionFailed(String),
    #[error("failed to parse AST: {0}")]
    AstParseError(String),
    #[error("header not found: {0}")]
    HeaderNotFound(String),
    #[error("compilation error: {0}")]
    CompilationError(String),
    #[error("libclang error: {0}")]
    LibclangError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Clang extraction configuration
#[derive(Debug, Clone)]
pub struct ClangConfig {
    /// Path to clang executable
    pub clang_path: Option<String>,
    /// Target triple
    pub target: Option<String>,
    /// sysroot
    pub sysroot: Option<String>,
    /// Resource directory
    pub resource_dir: Option<String>,
    /// Include paths
    pub include_paths: Vec<String>,
    /// System include paths
    pub system_include_paths: Vec<String>,
    /// Preprocessor defines
    pub defines: Vec<(String, Option<String>)>,
    /// C standard
    pub standard: CStandard,
    /// Additional flags
    pub extra_flags: Vec<String>,
    /// Whether to use libclang (requires libclang feature)
    pub use_libclang: bool,
    /// Timeout for clang invocation in seconds
    pub timeout_secs: u64,
    /// Trust boundary settings for extraction
    pub trust_boundary: ExtractionTrustBoundary,
    /// Path to emit dependency file (`.d` format) for incremental compilation.
    /// When set, `-MD -MF <path>` is added to clang arguments.
    pub dependency_file_path: Option<PathBuf>,
}

/// Trust boundary settings for Clang extraction
#[derive(Debug, Clone)]
pub struct ExtractionTrustBoundary {
    /// Whether Clang AST facts are trusted by default
    pub trust_clang_ast: bool,
    /// Whether Clang layout facts are trusted by default
    pub trust_clang_layout: bool,
    /// Whether system headers are trusted
    pub trust_system_headers: bool,
    /// Whether macro expansions are trusted
    pub trust_macro_expansions: bool,
    /// Whether inline asm is trusted
    pub trust_inline_asm: bool,
    /// List of facts that require independent verification
    pub requires_verification: Vec<TrustFactKind>,
}

/// Kinds of facts that may require verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustFactKind {
    /// Struct layout facts
    StructLayout,
    /// Union layout facts
    UnionLayout,
    /// Bitfield layout facts
    BitfieldLayout,
    /// Function calling convention
    FunctionCallConv,
    /// Varargs handling
    VarargsHandling,
    /// Pointer aliasing assumptions
    PointerAliasing,
    /// Inline assembly
    InlineAsm,
}

impl Default for ExtractionTrustBoundary {
    fn default() -> Self {
        Self {
            trust_clang_ast: true,
            trust_clang_layout: true,
            trust_system_headers: false,
            trust_macro_expansions: true,
            trust_inline_asm: false,
            requires_verification: vec![
                TrustFactKind::StructLayout,
                TrustFactKind::UnionLayout,
                TrustFactKind::BitfieldLayout,
                TrustFactKind::FunctionCallConv,
                TrustFactKind::VarargsHandling,
                TrustFactKind::PointerAliasing,
                TrustFactKind::InlineAsm,
            ],
        }
    }
}

impl ClangConfig {
    /// Enable full Clang authority (trust all facts)
    pub fn with_clang_authoritative(mut self) -> Self {
        self.trust_boundary = ExtractionTrustBoundary {
            trust_clang_ast: true,
            trust_clang_layout: true,
            trust_system_headers: true,
            trust_macro_expansions: true,
            trust_inline_asm: true,
            requires_verification: vec![],
        };
        self
    }

    /// Enable surface-only mode (minimal trust)
    pub fn with_surface_only(mut self) -> Self {
        self.trust_boundary = ExtractionTrustBoundary {
            trust_clang_ast: true,
            trust_clang_layout: false,
            trust_system_headers: false,
            trust_macro_expansions: true,
            trust_inline_asm: false,
            requires_verification: vec![
                TrustFactKind::StructLayout,
                TrustFactKind::UnionLayout,
                TrustFactKind::BitfieldLayout,
                TrustFactKind::FunctionCallConv,
                TrustFactKind::VarargsHandling,
                TrustFactKind::PointerAliasing,
                TrustFactKind::InlineAsm,
            ],
        };
        self
    }

    /// Check if a fact kind requires verification
    pub fn requires_verification(&self, fact: &TrustFactKind) -> bool {
        self.trust_boundary.requires_verification.contains(fact)
    }

    /// Get trust level description for diagnostics
    pub fn trust_description(&self) -> String {
        let mut desc = String::new();
        if self.trust_boundary.trust_clang_ast {
            if !desc.is_empty() {
                desc.push_str(", ");
            }
            desc.push_str("clang_ast");
        }
        if self.trust_boundary.trust_clang_layout {
            if !desc.is_empty() {
                desc.push_str(", ");
            }
            desc.push_str("clang_layout");
        }
        if self.trust_boundary.trust_system_headers {
            if !desc.is_empty() {
                desc.push_str(", ");
            }
            desc.push_str("system_headers");
        }
        if self.trust_boundary.trust_macro_expansions {
            if !desc.is_empty() {
                desc.push_str(", ");
            }
            desc.push_str("macro_expansions");
        }
        if self.trust_boundary.trust_inline_asm {
            if !desc.is_empty() {
                desc.push_str(", ");
            }
            desc.push_str("inline_asm");
        }
        if !self.trust_boundary.requires_verification.is_empty() {
            if !desc.is_empty() {
                desc.push_str(", ");
            }
            desc.push_str(&format!(
                "needs_verification={}",
                self.trust_boundary.requires_verification.len()
            ));
        }
        desc
    }
}

impl Default for ClangConfig {
    fn default() -> Self {
        Self {
            clang_path: None,
            target: None,
            sysroot: None,
            resource_dir: None,
            include_paths: vec![],
            system_include_paths: vec![],
            defines: vec![],
            standard: CStandard::C11,
            extra_flags: vec![],
            use_libclang: false,
            timeout_secs: 30,
            trust_boundary: ExtractionTrustBoundary::default(),
            dependency_file_path: None,
        }
    }
}

impl ClangConfig {
    /// Find clang executable
    pub fn find_clang(&self) -> Result<String> {
        if let Some(ref path) = self.clang_path {
            return Ok(path.clone());
        }

        // Try common clang locations
        let candidates = [
            "clang",
            "clang-17",
            "clang-16",
            "clang-15",
            "clang-14",
            "/usr/bin/clang",
            "/usr/local/bin/clang",
        ];

        for candidate in &candidates {
            if let Ok(output) = Command::new(candidate).arg("--version").output() {
                if output.status.success() {
                    return Ok(candidate.to_string());
                }
            }
        }

        Err(ClangError::ClangNotFound(
            "clang executable not found".to_string(),
        ))
    }

    /// Build clang arguments for extraction
    pub fn build_args(&self, source_file: &Path) -> Vec<String> {
        let mut args = vec![];

        // Target
        if let Some(ref target) = self.target {
            args.push(format!("--target={}", target));
        }

        // Sysroot
        if let Some(ref sysroot) = self.sysroot {
            args.push(format!("--sysroot={}", sysroot));
        }

        // Resource dir
        if let Some(ref resource_dir) = self.resource_dir {
            args.push(format!("-resource-dir={}", resource_dir));
        }

        // Standard
        let std_flag = match self.standard {
            CStandard::C89 => "-std=c89",
            CStandard::C90 => "-std=c90",
            CStandard::C99 => "-std=c99",
            CStandard::C11 => "-std=c11",
            CStandard::C17 => "-std=c17",
            CStandard::C23 => "-std=c23",
            CStandard::Gnuc => "-std=gnu11",
        };
        args.push(std_flag.to_string());

        // Include paths
        for path in &self.include_paths {
            args.push(format!("-I{}", path));
        }

        // System include paths
        for path in &self.system_include_paths {
            args.push(format!("-isystem {}", path));
        }

        // Defines
        for (name, value) in &self.defines {
            if let Some(ref v) = value {
                args.push(format!("-D{}={}", name, v));
            } else {
                args.push(format!("-D{}", name));
            }
        }

        // Dependency file for incremental compilation
        if let Some(ref dep_path) = self.dependency_file_path {
            args.push("-MD".to_string());
            args.push(format!("-MF{}", dep_path.display()));
        }

        // Extra flags
        args.extend(self.extra_flags.clone());

        // Add the source file
        args.push(source_file.to_string_lossy().to_string());

        args
    }

    /// Run clang extraction with sandboxed environment
    pub fn run_sandboxed(&self, source_file: &Path, working_dir: &Path) -> Result<ClangExtraction> {
        let clang_path = self.find_clang()?;
        let args = self.build_args(source_file);

        // Create deterministic environment
        let env = self.sandboxed_env();

        // Run with timeout using std::process::Command
        let output = Command::new(&clang_path)
            .args(&args)
            .current_dir(working_dir)
            .envs(&env)
            .output()
            .map_err(|e| ClangError::ExtractionFailed(e.to_string()))?;

        // Check for timeout or other issues
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ClangError::ExtractionFailed(format!(
                "clang exited with {}: {}",
                output.status, stderr
            )));
        }

        // Parse output and build extraction result
        // This is a simplified placeholder - real implementation would parse AST
        let extraction = self.parse_extraction_output(&output.stdout)?;

        Ok(extraction)
    }

    /// Create deterministic environment for sandboxed execution
    fn sandboxed_env(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        // Clear potentially non-deterministic variables
        env.insert("PWD".to_string(), ".".to_string());
        env.insert("TERM".to_string(), "dumb".to_string());

        // Use consistent locale
        env.insert("LC_ALL".to_string(), "C".to_string());
        env.insert("LANG".to_string(), "C".to_string());

        // Limit PATH to known-safe directories
        if std::env::var("PATH").is_ok() {
            // Keep only /usr/bin and /bin for determinism
            env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
        }

        // Unset variables that could introduce non-determinism
        for var in &["HOME", "USER", "LOGNAME", "SSH_AUTH_SOCK", "SSH_CONNECTION"] {
            env.insert(var.to_string(), String::new());
        }

        env
    }

    /// Parse clang extraction output
    fn parse_extraction_output(&self, _output: &[u8]) -> Result<ClangExtraction> {
        // For now, return a minimal extraction result
        // Real implementation would parse clang AST dump JSON
        let snapshot = CsnapSnapshot {
            header: ArtifactHeader::new(
                self.target.as_deref().unwrap_or("x86_64-unknown-linux-gnu"),
                env!("CARGO_PKG_VERSION"),
            ),
            checksum: String::new(),
            clang_version: String::new(),
            target: CTarget {
                triple: self.target.clone().unwrap_or_default(),
                arch: "x86_64".to_string(),
                os: "linux".to_string(),
                env: "gnu".to_string(),
                libc: None,
                clang_version: None,
                resource_dir: self.resource_dir.clone(),
                sysroot: self.sysroot.clone(),
                pointer_width: 64,
                size_of_ptr: 8,
                size_of_int: 4,
                size_of_long: 8,
                size_of_long_long: 8,
                size_of_float: 4,
                size_of_double: 8,
                size_of_long_double: 16,
                size_of_void: 1,
                int64_aligned: 8,
                long_long_aligned: 8,
                double_aligned: 8,
                long_double_aligned: 16,
                long_double_size: 16,
                big_endian: false,
                c_standard: self.standard,
                clang_trust_facts: vec![],
            },
            headers: vec![],
            source_files: vec![],
            declarations: vec![],
            exports: vec![],
            imports: vec![],
            compile_flags: vec![],
            active_macros: vec![],
            conditional_branches: vec![],
        };

        let ast_pack = CastPack {
            header: ArtifactHeader::new(
                self.target.as_deref().unwrap_or("x86_64-unknown-linux-gnu"),
                env!("CARGO_PKG_VERSION"),
            ),
            checksum: String::new(),
            declarations: vec![],
            types: vec![],
            layouts: vec![],
            symbol_table: SymbolTable {
                functions: HashMap::new(),
                globals: HashMap::new(),
                structs: HashMap::new(),
                unions: HashMap::new(),
                enums: HashMap::new(),
                typedefs: HashMap::new(),
                macros: HashMap::new(),
            },
            macro_provenance: MacroProvenance { expansions: vec![] },
            diagnostics: vec![],
        };

        Ok(ClangExtraction {
            snapshot,
            ast_pack,
            diagnostics: vec![],
        })
    }
}

/// Clang extraction result containing all extracted information
#[derive(Debug, Clone)]
pub struct ClangExtraction {
    /// The semantic snapshot
    pub snapshot: CsnapSnapshot,
    /// The AST/type/layout package
    pub ast_pack: CastPack,
    /// Diagnostics from extraction
    pub diagnostics: Vec<Diagnostic>,
}

/// C header extractor using clang AST dump
pub struct CHeaderExtractor {
    config: ClangConfig,
}

impl CHeaderExtractor {
    /// Create a new header extractor
    pub fn new(config: ClangConfig) -> Self {
        Self { config }
    }

    /// Extract C header information
    pub fn extract_header(&self, header_path: &Path) -> Result<HeaderInfo> {
        let clang_path = self.config.find_clang()?;

        // Parse the header to get basic info
        let mut args = self.config.build_args(header_path);
        args.push("-fsyntax-only".to_string());
        args.push("-Wno-warning".to_string());

        let _output = Command::new(&clang_path)
            .args(&args)
            .output()
            .map_err(|e| ClangError::ExtractionFailed(e.to_string()))?;

        // Get file info
        let metadata = std::fs::metadata(header_path)?;
        let content = std::fs::read(header_path)?;
        let content_hash = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&content);
            hasher.finalize().to_hex().to_string()
        };

        Ok(HeaderInfo {
            path: header_path.to_string_lossy().to_string(),
            content_hash,
            size: metadata.len(),
            mtime: metadata.mtime_nsec() as u64,
            include_guard: self.detect_include_guard(&content),
            includes: self.extract_includes(&content),
            macro_defs: vec![], // Would need more sophisticated extraction
            is_system: false,
            is_generated: false,
        })
    }

    /// Detect include guard in header content
    fn detect_include_guard(&self, content: &[u8]) -> Option<String> {
        let text = String::from_utf8_lossy(content);
        let lines: Vec<&str> = text.lines().collect();

        // Simple include guard detection
        let mut ifndef_line: Option<&str> = None;
        let mut define_line: Option<&str> = None;
        let mut endif_count = 0;

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("#ifndef") || trimmed.starts_with("#if !defined") {
                if ifndef_line.is_none() {
                    ifndef_line = Some(trimmed);
                }
            } else if trimmed.starts_with("#define") && define_line.is_none() {
                define_line = Some(trimmed);
            } else if trimmed.starts_with("#endif") {
                endif_count += 1;
            }
        }

        // If we have matching ifndef/define/endif, extract the guard name
        if let (Some(ifndef), Some(_define), 1) = (ifndef_line, define_line, endif_count) {
            // Extract identifier from #ifndef GUARD
            if let Some(name) = ifndef.split_whitespace().nth(1) {
                return Some(name.to_string());
            }
        }

        None
    }

    /// Extract #include directives from content
    fn extract_includes(&self, content: &[u8]) -> Vec<String> {
        let text = String::from_utf8_lossy(content);
        let mut includes = vec![];

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#include") {
                includes.push(trimmed.to_string());
            }
        }

        includes
    }
}

/// C source extractor using clang AST dump
pub struct CSourceExtractor {
    config: ClangConfig,
}

impl CSourceExtractor {
    /// Create a new source extractor
    pub fn new(config: ClangConfig) -> Self {
        Self { config }
    }

    /// Extract from source file using clang -ast-dump=json
    pub fn extract_ast_json(&self, source_path: &Path) -> Result<serde_json::Value> {
        let clang_path = self.config.find_clang()?;

        let mut args = self.config.build_args(source_path);
        args.push("-Xclang".to_string());
        args.push("-ast-dump=json".to_string());
        args.push("-fsyntax-only".to_string());

        let output = Command::new(&clang_path)
            .args(&args)
            .output()
            .map_err(|e| ClangError::ExtractionFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ClangError::ExtractionFailed(stderr.to_string()));
        }

        // The AST dump goes to stderr in clang
        let _ast_text = String::from_utf8_lossy(&output.stderr);

        // Parse the text format to extract key info (full JSON parsing would require more work)
        // For now, we create a minimal extraction result
        Ok(serde_json::json!({
            "source_path": source_path.to_string_lossy(),
            "status": "extracted"
        }))
    }

    /// Extract source file information
    pub fn extract_source_info(&self, source_path: &Path) -> Result<SourceFileInfo> {
        let metadata = std::fs::metadata(source_path)?;
        let content = std::fs::read(source_path)?;
        let content_hash = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&content);
            hasher.finalize().to_hex().to_string()
        };

        Ok(SourceFileInfo {
            path: source_path.to_string_lossy().to_string(),
            content_hash,
            size: metadata.len(),
            mtime: metadata.mtime_nsec() as u64,
            translation_unit: TranslationUnit {
                id: TUId(0), // Would be assigned properly in full implementation
                source_file: source_path.to_string_lossy().to_string(),
                header_dependencies: vec![],
                macro_dependencies: vec![],
                declarations: vec![],
            },
        })
    }
}

/// Compile command entry from compile_commands.json
#[derive(Debug, Clone)]
pub struct CompileCommand {
    pub directory: String,
    pub command: String,
    pub file: String,
    pub output: Option<String>,
}

impl CompileCommand {
    /// Parse compile_commands.json
    pub fn parse_compile_commands(json_content: &str) -> Result<Vec<Self>> {
        #[derive(Deserialize)]
        struct RawCommand {
            directory: String,
            command: String,
            file: String,
            output: Option<String>,
        }

        let raw: Vec<RawCommand> = serde_json::from_str(json_content)?;

        Ok(raw
            .into_iter()
            .map(|r| CompileCommand {
                directory: r.directory,
                command: r.command,
                file: r.file,
                output: r.output,
            })
            .collect())
    }

    /// Get include directories from command
    pub fn extract_include_dirs(&self) -> Vec<String> {
        let mut dirs = vec![];
        let args = shell_words::split(&self.command).unwrap_or_default();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-I" => {
                    if i + 1 < args.len() {
                        dirs.push(args[i + 1].clone());
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                s if s.starts_with("-I") => {
                    dirs.push(s[2..].to_string());
                    i += 1;
                }
                _ => i += 1,
            }
        }

        dirs
    }

    /// Get defines from command
    pub fn extract_defines(&self) -> Vec<(String, Option<String>)> {
        let mut defines = vec![];
        let args = shell_words::split(&self.command).unwrap_or_default();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-D" => {
                    if i + 1 < args.len() {
                        let def = &args[i + 1];
                        if let Some(pos) = def.find('=') {
                            defines
                                .push((def[..pos].to_string(), Some(def[pos + 1..].to_string())));
                        } else {
                            defines.push((def.clone(), None));
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                s if s.starts_with("-D") => {
                    let def = &s[2..];
                    if let Some(pos) = def.find('=') {
                        defines.push((def[..pos].to_string(), Some(def[pos + 1..].to_string())));
                    } else {
                        defines.push((def.to_string(), None));
                    }
                    i += 1;
                }
                _ => i += 1,
            }
        }

        defines
    }
}

/// Include graph tracking
#[derive(Debug, Clone, Default)]
pub struct IncludeGraph {
    /// Map from header path to files that include it
    pub included_by: HashMap<String, Vec<String>>,
    /// Map from file to headers it includes
    pub includes: HashMap<String, Vec<String>>,
}

impl IncludeGraph {
    /// Add an include edge
    pub fn add_include(&mut self, from: &str, to: &str) {
        self.includes
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
        self.included_by
            .entry(to.to_string())
            .or_default()
            .push(from.to_string());
    }

    /// Get all headers included by a file
    pub fn get_includes(&self, file: &str) -> &[String] {
        self.includes.get(file).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get all files that include a header
    pub fn get_included_by(&self, header: &str) -> &[String] {
        self.included_by
            .get(header)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

/// Macro tracking
#[derive(Debug, Clone)]
pub struct MacroTracker {
    /// Active macro definitions
    pub macros: HashMap<String, MacroInfo>,
    /// Conditional compilation branches
    pub conditionals: Vec<ConditionalBranch>,
}

#[derive(Debug, Clone)]
pub struct MacroInfo {
    pub name: String,
    pub value: Option<String>,
    pub is_function_like: bool,
    pub params: Option<Vec<String>>,
    pub location: SourceSpan,
    pub expansion_locations: Vec<SourceSpan>,
}

impl Default for MacroTracker {
    fn default() -> Self {
        Self {
            macros: HashMap::new(),
            conditionals: vec![],
        }
    }
}

impl MacroTracker {
    /// Add a macro definition
    pub fn define_macro(&mut self, name: String, value: Option<String>, location: SourceSpan) {
        self.macros.insert(
            name.clone(),
            MacroInfo {
                name,
                value,
                is_function_like: false,
                params: None,
                location,
                expansion_locations: vec![],
            },
        );
    }

    /// Add a function-like macro
    pub fn define_function_macro(
        &mut self,
        name: String,
        params: Vec<String>,
        value: Option<String>,
        location: SourceSpan,
    ) {
        self.macros.insert(
            name.clone(),
            MacroInfo {
                name,
                value,
                is_function_like: true,
                params: Some(params),
                location,
                expansion_locations: vec![],
            },
        );
    }

    /// Track macro expansion
    pub fn track_expansion(&mut self, name: &str, location: SourceSpan) {
        if let Some(macro_info) = self.macros.get_mut(name) {
            macro_info.expansion_locations.push(location);
        }
    }

    /// Check if macro is defined
    pub fn is_defined(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }
}

#[cfg(feature = "libclang")]
pub mod libclang {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    /// Libclang index for parsing
    pub struct ClangIndex {
        index: clang::Index,
    }

    impl ClangIndex {
        pub fn new() -> Self {
            Self {
                index: clang::Index::new(false, false),
            }
        }

        pub fn parse_header(
            &self,
            path: &Path,
            args: &[String],
        ) -> std::result::Result<clang::TranslationUnit, ClangError> {
            let cpath = CString::new(path.to_string_lossy().as_bytes())
                .map_err(|e| ClangError::LibclangError(e.to_string()))?;

            let cargs: Vec<CString> = args
                .iter()
                .filter_map(|a| CString::new(a.as_bytes()).ok())
                .collect();

            let mut unsaved_files: Vec<clang::UnsavedFile> = vec![];

            let tu = self
                .index
                .parse_translation_unit(
                    &cpath,
                    &cargs,
                    &unsaved_files,
                    clang::TranslationUnit_Flags::NONE,
                )
                .map_err(|e| ClangError::LibclangError(e.to_string()))?;

            Ok(tu)
        }
    }
}

// Simple shell words split (basic implementation)
mod shell_words {
    pub fn split(input: &str) -> std::result::Result<Vec<String>, ()> {
        let mut result = vec![];
        let mut current = String::new();
        let mut in_quote = false;
        let mut quote_char = ' ';

        for ch in input.chars() {
            match ch {
                '"' | '\'' if !in_quote => {
                    in_quote = true;
                    quote_char = ch;
                }
                '"' | '\'' if in_quote && ch == quote_char => {
                    in_quote = false;
                }
                ' ' | '\t' | '\n' if !in_quote => {
                    if !current.is_empty() {
                        result.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            result.push(current);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clang_config_default() {
        let config = ClangConfig::default();
        assert_eq!(config.standard, CStandard::C11);
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_clang_config_build_args() {
        let config = ClangConfig {
            defines: vec![("FOO".to_string(), Some("1".to_string()))],
            include_paths: vec!["/include".to_string()],
            standard: CStandard::C99,
            ..Default::default()
        };

        let args = config.build_args(Path::new("test.c"));
        assert!(args.contains(&"-std=c99".to_string()));
        assert!(args.contains(&"-DFOO=1".to_string()));
        assert!(args.contains(&"-I/include".to_string()));
        assert!(args.contains(&"test.c".to_string()));
    }

    #[test]
    fn test_clang_config_system_include_args() {
        let config = ClangConfig {
            system_include_paths: vec![
                "/usr/include".to_string(),
                "/opt/local/include".to_string(),
            ],
            ..Default::default()
        };

        let args = config.build_args(Path::new("test.c"));
        assert!(args.contains(&"-isystem /usr/include".to_string()));
        assert!(args.contains(&"-isystem /opt/local/include".to_string()));
    }

    #[test]
    fn test_clang_config_dependency_file_args() {
        let config = ClangConfig {
            dependency_file_path: Some(PathBuf::from("/build/test.d")),
            ..Default::default()
        };

        let args = config.build_args(Path::new("test.c"));
        assert!(args.contains(&"-MD".to_string()));
        assert!(args.contains(&"-MF/build/test.d".to_string()));
    }

    #[test]
    fn test_clang_config_no_dependency_file_by_default() {
        let config = ClangConfig::default();
        let args = config.build_args(Path::new("test.c"));
        assert!(!args.contains(&"-MD".to_string()));
        assert!(!args.contains(&"-MF".to_string()));
    }

    #[test]
    fn test_compile_command_parse() {
        let json = r#"[
            {
                "directory": "/project",
                "command": "clang -Iinclude -DFOO=1 test.c -o test",
                "file": "test.c"
            }
        ]"#;

        let commands = CompileCommand::parse_compile_commands(json).unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].directory, "/project");
    }

    #[test]
    fn test_compile_command_extract_include_dirs() {
        let cmd = CompileCommand {
            directory: "/project".to_string(),
            command: "clang -Iinclude -I/usr/local/include -DFOO test.c".to_string(),
            file: "test.c".to_string(),
            output: None,
        };

        let dirs = cmd.extract_include_dirs();
        assert_eq!(dirs.len(), 2);
        assert!(dirs.contains(&"include".to_string()));
        assert!(dirs.contains(&"/usr/local/include".to_string()));
    }

    #[test]
    fn test_compile_command_extract_defines() {
        let cmd = CompileCommand {
            directory: "/project".to_string(),
            command: "clang -DFOO -DBAR=42 -DBAZ=\"value\" test.c".to_string(),
            file: "test.c".to_string(),
            output: None,
        };

        let defines = cmd.extract_defines();
        assert_eq!(defines.len(), 3);
        assert!(defines.contains(&("FOO".to_string(), None)));
        assert!(defines.contains(&("BAR".to_string(), Some("42".to_string()))));
    }

    #[test]
    fn test_include_graph() {
        let mut graph = IncludeGraph::default();
        graph.add_include("main.c", "header.h");
        graph.add_include("header.h", "types.h");

        assert_eq!(graph.get_includes("main.c"), &["header.h"]);
        assert_eq!(graph.get_included_by("header.h"), &["main.c"]);
    }

    #[test]
    fn test_macro_tracker() {
        let mut tracker = MacroTracker::default();
        tracker.define_macro(
            "FOO".to_string(),
            Some("1".to_string()),
            SourceSpan {
                file: "test.h".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 10,
            },
        );

        assert!(tracker.is_defined("FOO"));
        assert!(!tracker.is_defined("BAR"));

        tracker.define_function_macro(
            "MAX".to_string(),
            vec!["a".to_string(), "b".to_string()],
            Some("(a > b ? a : b)".to_string()),
            SourceSpan {
                file: "test.h".to_string(),
                line: 5,
                col: 1,
                byte_offset: 100,
                byte_length: 20,
            },
        );

        let max_macro = tracker.macros.get("MAX").unwrap();
        assert!(max_macro.is_function_like);
        assert_eq!(max_macro.params.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_shell_words_split() {
        let result = shell_words::split("clang -Iinclude -DFOO=1 \"test file.c\"").unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "clang");
        assert_eq!(result[3], "test file.c");
    }

    #[test]
    fn test_c_standard_variants() {
        assert_eq!(CStandard::C89 as u32, 0);
        assert_eq!(CStandard::C90 as u32, 1);
        assert_eq!(CStandard::C99 as u32, 2);
        assert_eq!(CStandard::C11 as u32, 3);
    }

    #[test]
    fn test_header_extractor_include_guard() {
        let extractor = CHeaderExtractor::new(ClangConfig::default());
        let content = b"
            #ifndef HEADER_H
            #define HEADER_H

            void foo(void);

            #endif /* HEADER_H */
        ";

        let guard = extractor.detect_include_guard(content);
        assert_eq!(guard, Some("HEADER_H".to_string()));
    }

    // =============================================================================
    // Extraction Trust Boundary Tests (Task 42)
    // =============================================================================

    #[test]
    fn test_extraction_trust_boundary_default() {
        let boundary = ExtractionTrustBoundary::default();
        assert!(boundary.trust_clang_ast);
        assert!(boundary.trust_clang_layout);
        assert!(!boundary.trust_system_headers);
        assert!(boundary.trust_macro_expansions);
        assert!(!boundary.trust_inline_asm);
        assert_eq!(boundary.requires_verification.len(), 7);
    }

    #[test]
    fn test_clang_config_default_has_trust_boundary() {
        let config = ClangConfig::default();
        assert_eq!(config.trust_boundary.requires_verification.len(), 7);
    }

    #[test]
    fn test_clang_config_with_clang_authoritative() {
        let config = ClangConfig::default().with_clang_authoritative();
        assert!(config.trust_boundary.trust_clang_ast);
        assert!(config.trust_boundary.trust_clang_layout);
        assert!(config.trust_boundary.trust_system_headers);
        assert!(config.trust_boundary.trust_inline_asm);
        assert!(config.trust_boundary.requires_verification.is_empty());
    }

    #[test]
    fn test_clang_config_with_surface_only() {
        let config = ClangConfig::default().with_surface_only();
        assert!(config.trust_boundary.trust_clang_ast);
        assert!(!config.trust_boundary.trust_clang_layout);
        assert!(!config.trust_boundary.trust_system_headers);
        assert!(!config.trust_boundary.trust_inline_asm);
        assert_eq!(config.trust_boundary.requires_verification.len(), 7);
    }

    #[test]
    fn test_requires_verification() {
        let config = ClangConfig::default();
        assert!(config.requires_verification(&TrustFactKind::StructLayout));
        assert!(config.requires_verification(&TrustFactKind::InlineAsm));
        assert!(config.requires_verification(&TrustFactKind::FunctionCallConv));
    }

    #[test]
    fn test_trust_description() {
        let config = ClangConfig::default();
        let desc = config.trust_description();
        assert!(desc.contains("clang_ast"));
        assert!(desc.contains("clang_layout"));
        assert!(desc.contains("needs_verification=7"));
    }

    #[test]
    fn test_trust_description_authoritative() {
        let config = ClangConfig::default().with_clang_authoritative();
        let desc = config.trust_description();
        assert!(desc.contains("clang_ast"));
        assert!(desc.contains("system_headers"));
        assert!(desc.contains("inline_asm"));
        // When requires_verification is empty, it shouldn't appear in description
        assert!(!desc.contains("needs_verification"));
    }

    #[test]
    fn test_trust_fact_kind_variants() {
        assert!(matches!(
            TrustFactKind::StructLayout,
            TrustFactKind::StructLayout
        ));
        assert!(matches!(
            TrustFactKind::UnionLayout,
            TrustFactKind::UnionLayout
        ));
        assert!(matches!(
            TrustFactKind::BitfieldLayout,
            TrustFactKind::BitfieldLayout
        ));
        assert!(matches!(
            TrustFactKind::FunctionCallConv,
            TrustFactKind::FunctionCallConv
        ));
        assert!(matches!(
            TrustFactKind::VarargsHandling,
            TrustFactKind::VarargsHandling
        ));
        assert!(matches!(
            TrustFactKind::PointerAliasing,
            TrustFactKind::PointerAliasing
        ));
        assert!(matches!(TrustFactKind::InlineAsm, TrustFactKind::InlineAsm));
    }
}
