//! Chimera C build system ingestion crate.
//!
//! Parses compile_commands.json, direct cc/clang flags, include paths,
//! macro definitions, target triple, sysroot, standard version, and linker inputs.
//!
//! Task 12: C build-system ingestion crate

use chimera_c_clang::{ClangConfig, CompileCommand};
use chimera_c_schema::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Result type for build operations
pub type Result<T> = std::result::Result<T, BuildError>;

/// Build system ingestion errors
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("compile database not found: {0}")]
    CompileDatabaseNotFound(String),
    #[error("failed to parse compile database: {0}")]
    ParseError(String),
    #[error("missing include directory: {0}")]
    MissingIncludeDir(String),
    #[error("invalid target triple: {0}")]
    InvalidTargetTriple(String),
    #[error("invalid standard version: {0}")]
    InvalidStandard(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("clang error: {0}")]
    ClangError(String),
}

impl From<chimera_c_clang::ClangError> for BuildError {
    fn from(e: chimera_c_clang::ClangError) -> Self {
        BuildError::ClangError(e.to_string())
    }
}

/// C build configuration
#[derive(Debug, Clone)]
pub struct CBuildConfig {
    /// Working directory for compilation
    pub working_dir: PathBuf,
    /// Compile commands database path (compile_commands.json)
    pub compile_commands_path: Option<PathBuf>,
    /// Direct compiler flags (when no compile_commands.json)
    pub compiler_flags: Vec<String>,
    /// Include directories
    pub include_dirs: Vec<IncludeDirectory>,
    /// Macro definitions
    pub defines: Vec<MacroDefinition>,
    /// Target triple
    pub target: Option<String>,
    /// sysroot
    pub sysroot: Option<PathBuf>,
    /// Resource directory
    pub resource_dir: Option<PathBuf>,
    /// C standard
    pub standard: CStandard,
    /// Linker libraries
    pub linker_libs: Vec<LinkerLibrary>,
    /// Compiler executable (cc, clang, etc.)
    pub compiler: Option<String>,
    /// Generated headers (to track invalidation)
    pub generated_headers: Vec<GeneratedHeader>,
}

impl Default for CBuildConfig {
    fn default() -> Self {
        Self {
            working_dir: PathBuf::from("."),
            compile_commands_path: None,
            compiler_flags: vec![],
            include_dirs: vec![],
            defines: vec![],
            target: None,
            sysroot: None,
            resource_dir: None,
            standard: CStandard::C11,
            linker_libs: vec![],
            compiler: None,
            generated_headers: vec![],
        }
    }
}

impl CBuildConfig {
    /// Compute a hash of compilation flags for cache key generation.
    ///
    /// This includes optimization level, debug flags, warning flags,
    /// and other flags that affect compilation behavior.
    pub fn flags_hash(&self) -> String {
        let mut hasher = zigmera_hash::Blake3Hasher::with_schema_tag("c-build-flags");

        // Sort flags for deterministic hashing
        let mut sorted_flags = self.compiler_flags.clone();
        sorted_flags.sort();

        for flag in sorted_flags {
            hasher.update_str(&flag);
        }

        // Include standard
        hasher.update_str(match self.standard {
            CStandard::C89 => "c89",
            CStandard::C90 => "c90",
            CStandard::C99 => "c99",
            CStandard::C11 => "c11",
            CStandard::C17 => "c17",
            CStandard::C23 => "c23",
            CStandard::Gnuc => "gnu",
        });

        // Include target if set
        if let Some(ref target) = self.target {
            hasher.update_str(target);
        }

        hasher.finalize().as_hex()[..16].to_string()
    }
}

/// Include directory with search information
#[derive(Debug, Clone)]
pub struct IncludeDirectory {
    /// Path to include directory
    pub path: PathBuf,
    /// Whether this is a system include
    pub is_system: bool,
    /// Whether this is a framework include (macOS)
    pub is_framework: bool,
    /// Priority order
    pub priority: u32,
}

impl IncludeDirectory {
    /// Add to clang config
    pub fn apply_to_clang_config(&self, config: &mut ClangConfig) {
        if self.is_framework {
            config
                .system_include_paths
                .push(self.path.to_string_lossy().to_string());
        } else if self.is_system {
            config
                .system_include_paths
                .push(self.path.to_string_lossy().to_string());
        } else {
            config
                .include_paths
                .push(self.path.to_string_lossy().to_string());
        }
    }
}

/// Macro definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroDefinition {
    /// Macro name
    pub name: String,
    /// Macro value (None for defined without value)
    pub value: Option<String>,
    /// Whether this affects ABI
    pub affects_abi: bool,
}

impl MacroDefinition {
    /// Parse from command line -D argument
    pub fn from_arg(arg: &str) -> Option<Self> {
        if !arg.starts_with("-D") && !arg.starts_with("D") {
            return None;
        }

        let arg = arg.trim_start_matches("-D").trim_start_matches('D');
        if arg.is_empty() {
            return None;
        }

        if let Some(pos) = arg.find('=') {
            Some(MacroDefinition {
                name: arg[..pos].to_string(),
                value: Some(arg[pos + 1..].to_string()),
                affects_abi: true, // Conservative default
            })
        } else {
            Some(MacroDefinition {
                name: arg.to_string(),
                value: None,
                affects_abi: true,
            })
        }
    }

    /// Apply to clang config
    pub fn apply_to_clang_config(&self, config: &mut ClangConfig) {
        config.defines.push((self.name.clone(), self.value.clone()));
    }
}

/// Linker library
#[derive(Debug, Clone)]
pub struct LinkerLibrary {
    /// Library name (e.g., "m" for libm)
    pub name: String,
    /// Full path if specified
    pub path: Option<PathBuf>,
    /// Whether this is a system library
    pub is_system: bool,
    /// Static or shared
    pub kind: LinkerLibraryKind,
}

/// Kind of linker library
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkerLibraryKind {
    Static,
    Shared,
    Framework,
}

/// Link search directory for library lookup
#[derive(Debug, Clone)]
pub struct LinkSearchDir {
    /// Path to search directory
    pub path: PathBuf,
    /// Whether this is a system directory
    pub is_system: bool,
}

/// Link plan combining all link inputs
#[derive(Debug, Clone)]
pub struct LinkPlan {
    /// Static libraries to link
    pub static_libraries: Vec<LinkerLibrary>,
    /// Shared libraries to link
    pub shared_libraries: Vec<LinkerLibrary>,
    /// Framework libraries (macOS)
    pub frameworks: Vec<LinkerLibrary>,
    /// Link search directories
    pub search_dirs: Vec<LinkSearchDir>,
    /// Linker flags
    pub linker_flags: Vec<String>,
    /// RPATH entries
    pub rpath_entries: Vec<String>,
    /// SONAME for shared libraries
    pub soname: Option<String>,
}

impl LinkPlan {
    /// Create a new empty link plan
    pub fn new() -> Self {
        Self {
            static_libraries: Vec::new(),
            shared_libraries: Vec::new(),
            frameworks: Vec::new(),
            search_dirs: Vec::new(),
            linker_flags: Vec::new(),
            rpath_entries: Vec::new(),
            soname: None,
        }
    }

    /// Add a library from a flag string (e.g., "-lm" or "/path/to/libfoo.a")
    pub fn add_library_from_flag(&mut self, flag: &str) -> Option<()> {
        // Handle -l<name>
        if flag.starts_with("-l") && flag.len() > 2 {
            let name = flag[2..].to_string();
            let kind = LinkerLibraryKind::Shared; // Default to shared unless -static seen
            self.shared_libraries.push(LinkerLibrary {
                name,
                path: None,
                is_system: true,
                kind,
            });
            return Some(());
        }

        // Handle -L<path>
        if flag.starts_with("-L") && flag.len() > 2 {
            let path = PathBuf::from(&flag[2..]);
            self.search_dirs.push(LinkSearchDir {
                path,
                is_system: false,
            });
            return Some(());
        }

        // Handle -Wl,-rpath,<path> style RPATH
        if flag.starts_with("-Wl,-rpath,") {
            let rpath = flag[11..].to_string();
            self.rpath_entries.push(rpath);
            return Some(());
        }

        // Handle -soname:<name>
        if flag.starts_with("-soname:") {
            self.soname = Some(flag[8..].to_string());
            return Some(());
        }

        // Handle explicit path to .a or .so file
        if flag.ends_with(".a") {
            let path = PathBuf::from(flag);
            let name = path
                .file_stem()?
                .to_string_lossy()
                .trim_start_matches("lib")
                .to_string();
            self.static_libraries.push(LinkerLibrary {
                name,
                path: Some(path),
                is_system: false,
                kind: LinkerLibraryKind::Static,
            });
            return Some(());
        }
        if flag.ends_with(".so") || flag.ends_with(".dylib") {
            let path = PathBuf::from(flag);
            let name = path
                .file_stem()?
                .to_string_lossy()
                .trim_start_matches("lib")
                .to_string();
            self.shared_libraries.push(LinkerLibrary {
                name,
                path: Some(path),
                is_system: false,
                kind: LinkerLibraryKind::Shared,
            });
            return Some(());
        }

        None
    }

    /// Check if a library is a system library
    pub fn is_system_library(&self, name: &str) -> bool {
        self.static_libraries
            .iter()
            .any(|l| l.name == name && l.is_system)
            || self
                .shared_libraries
                .iter()
                .any(|l| l.name == name && l.is_system)
    }

    /// Get all unique library names
    pub fn library_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        names.extend(self.static_libraries.iter().map(|l| l.name.clone()));
        names.extend(self.shared_libraries.iter().map(|l| l.name.clone()));
        names.extend(self.frameworks.iter().map(|l| l.name.clone()));
        names.sort();
        names.dedup();
        names
    }
}

impl Default for LinkPlan {
    fn default() -> Self {
        Self::new()
    }
}

/// Generated header tracking
#[derive(Debug, Clone)]
pub struct GeneratedHeader {
    /// Path to generated header
    pub path: PathBuf,
    /// Command that generates it (for invalidation)
    pub generator_command: Option<String>,
    /// Content hash (for invalidation)
    pub content_hash: String,
}

impl GeneratedHeader {
    /// Check if this generated header is stale and needs rebuild
    pub fn is_stale(&self) -> bool {
        // If the header file doesn't exist, it's stale
        if !self.path.exists() {
            return true;
        }

        // Compute current content hash
        if let Ok(content) = std::fs::read_to_string(&self.path) {
            let current_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
            current_hash != self.content_hash
        } else {
            true
        }
    }

    /// Record the current content hash after successful generation
    pub fn record_generation(&mut self) -> std::io::Result<()> {
        if self.path.exists() {
            let content = std::fs::read_to_string(&self.path)?;
            self.content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
        }
        Ok(())
    }
}

/// Compile database
#[derive(Debug, Clone)]
pub struct CompileDatabase {
    /// All compile commands
    pub commands: Vec<CompileCommand>,
    /// Directory the database was found in
    pub directory: PathBuf,
}

impl CompileDatabase {
    /// Load compile_commands.json from a directory
    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let db_path = dir.join("compile_commands.json");

        if !db_path.exists() {
            return Err(BuildError::CompileDatabaseNotFound(
                db_path.to_string_lossy().to_string(),
            ));
        }

        Self::load_from_file(&db_path)
    }

    /// Load compile_commands.json from a specific file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let commands = CompileCommand::parse_compile_commands(&content)?;

        let directory = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        Ok(Self {
            commands,
            directory,
        })
    }

    /// Get commands for a specific file
    pub fn get_commands_for(&self, file: &Path) -> Vec<&CompileCommand> {
        self.commands
            .iter()
            .filter(|cmd| Path::new(&cmd.file) == file)
            .collect()
    }

    /// Extract include directories from all commands
    pub fn extract_include_dirs(&self) -> Vec<IncludeDirectory> {
        let mut dirs: HashMap<String, IncludeDirectory> = HashMap::new();
        let mut priority = 0u32;

        for cmd in &self.commands {
            for dir in cmd.extract_include_dirs() {
                if !dirs.contains_key(&dir) {
                    dirs.insert(
                        dir.clone(),
                        IncludeDirectory {
                            path: PathBuf::from(&dir),
                            is_system: false,
                            is_framework: false,
                            priority,
                        },
                    );
                    priority += 1;
                }
            }
        }

        let mut result: Vec<_> = dirs.into_values().collect();
        result.sort_by_key(|d| d.priority);
        result
    }

    /// Extract defines from all commands
    pub fn extract_defines(&self) -> Vec<MacroDefinition> {
        let mut defs: HashMap<String, MacroDefinition> = HashMap::new();

        for cmd in &self.commands {
            for (name, value) in cmd.extract_defines() {
                defs.entry(name.clone()).or_insert(MacroDefinition {
                    name,
                    value,
                    affects_abi: true,
                });
            }
        }

        defs.into_values().collect()
    }

    /// Extract target triple from commands
    pub fn extract_target(&self) -> Option<String> {
        for cmd in &self.commands {
            for arg in shell_words::split(&cmd.command).ok()? {
                if arg.starts_with("--target=") {
                    return Some(arg.trim_start_matches("--target=").to_string());
                }
            }
        }
        None
    }

    /// Extract sysroot from commands
    pub fn extract_sysroot(&self) -> Option<PathBuf> {
        for cmd in &self.commands {
            for arg in shell_words::split(&cmd.command).ok()? {
                if arg.starts_with("--sysroot=") {
                    return Some(PathBuf::from(arg.trim_start_matches("--sysroot=")));
                }
            }
        }
        None
    }
}

/// C build context combining all build information
#[derive(Debug, Clone)]
pub struct CBuildContext {
    /// Build configuration
    pub config: CBuildConfig,
    /// Compile database (if available)
    pub compile_database: Option<CompileDatabase>,
    /// Compiler identity
    pub compiler_identity: Option<CompilerIdentity>,
    /// Per-TU snapshots for multiple translation unit support
    pub tu_snapshots: Vec<TuSnapshot>,
}

/// A snapshot for a single translation unit
#[derive(Debug, Clone)]
pub struct TuSnapshot {
    /// Unique TU identifier
    pub id: TUId,
    /// Source file path
    pub source_file: PathBuf,
    /// Object file path (if compiled)
    pub object_file: Option<PathBuf>,
    /// Directories this TU depends on
    pub include_dirs: Vec<PathBuf>,
    /// Macros defined for this TU
    pub defines: Vec<MacroDefinition>,
    /// Compile flags for this TU
    pub flags: Vec<String>,
    /// Headers this TU includes
    pub header_dependencies: Vec<PathBuf>,
    /// Whether this TU has definitions that could conflict with other TUs
    pub has_definitions: bool,
    /// Symbol linkage map for duplicate detection
    pub symbol_linkage: HashMap<String, SymbolLinkage>,
}

/// Symbol linkage information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolLinkage {
    /// Symbol name
    pub name: String,
    /// Linkage kind
    pub linkage: chimera_c_schema::Linkage,
    /// Storage class
    pub storage: chimera_c_schema::StorageClass,
    /// Whether it's defined in this TU
    pub is_definition: bool,
}

/// Compiler identity information
#[derive(Debug, Clone)]
pub struct CompilerIdentity {
    /// Compiler executable path
    pub executable: String,
    /// Compiler version
    pub version: String,
    /// Target triple
    pub target: String,
    /// Resource directory
    pub resource_dir: Option<String>,
    /// Sysroot
    pub sysroot: Option<String>,
    /// libc name
    pub libc: Option<String>,
}

impl CBuildContext {
    /// Create from compile_commands.json
    pub fn from_compile_commands(working_dir: &Path) -> Result<Self> {
        let compile_database = Some(CompileDatabase::load_from_dir(working_dir)?);
        let db = compile_database.as_ref().unwrap();

        // Extract configuration from database
        let mut config = CBuildConfig::default();
        config.working_dir = working_dir.to_path_buf();
        config.compile_commands_path = Some(db.directory.join("compile_commands.json"));
        config.include_dirs = db.extract_include_dirs();
        config.defines = db.extract_defines();
        config.target = db.extract_target();
        config.sysroot = db.extract_sysroot();

        Ok(Self {
            config,
            compile_database,
            compiler_identity: None,
            tu_snapshots: Vec::new(),
        })
    }

    /// Create from direct compiler flags
    pub fn from_flags(
        working_dir: &Path,
        compiler_flags: Vec<String>,
        include_dirs: Vec<PathBuf>,
        defines: Vec<(String, Option<String>)>,
    ) -> Self {
        let config = CBuildConfig {
            working_dir: working_dir.to_path_buf(),
            compile_commands_path: None,
            compiler_flags: compiler_flags.clone(),
            include_dirs: include_dirs
                .into_iter()
                .enumerate()
                .map(|(i, p)| IncludeDirectory {
                    path: p,
                    is_system: false,
                    is_framework: false,
                    priority: i as u32,
                })
                .collect(),
            defines: defines
                .into_iter()
                .map(|(name, value)| MacroDefinition {
                    name,
                    value,
                    affects_abi: true,
                })
                .collect(),
            ..Default::default()
        };

        Self {
            config,
            compile_database: None,
            compiler_identity: None,
            tu_snapshots: Vec::new(),
        }
    }

    /// Build clang config from this context
    pub fn to_clang_config(&self) -> ClangConfig {
        let mut config = ClangConfig::default();

        config.target = self.config.target.clone();
        config.sysroot = self
            .config
            .sysroot
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        config.resource_dir = self
            .config
            .resource_dir
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        config.standard = self.config.standard;

        if let Some(ref compiler) = self.config.compiler {
            config.clang_path = Some(compiler.clone());
        }

        for dir in &self.config.include_dirs {
            dir.apply_to_clang_config(&mut config);
        }

        for def in &self.config.defines {
            def.apply_to_clang_config(&mut config);
        }

        config.extra_flags = self.config.compiler_flags.clone();

        config
    }

    /// Get all source files from compile database
    pub fn source_files(&self) -> Vec<PathBuf> {
        self.compile_database
            .as_ref()
            .map(|db| db.commands.iter().map(|c| PathBuf::from(&c.file)).collect())
            .unwrap_or_default()
    }

    /// Get all header files from compile database
    pub fn header_files(&self) -> Vec<PathBuf> {
        // Headers are not directly tracked in compile_commands.json
        // but could be extracted via other means
        vec![]
    }

    /// Add a translation unit snapshot
    pub fn add_tu_snapshot(&mut self, snapshot: TuSnapshot) {
        self.tu_snapshots.push(snapshot);
    }

    /// Get a TU snapshot by ID
    pub fn get_tu_snapshot(&self, id: &TUId) -> Option<&TuSnapshot> {
        self.tu_snapshots.iter().find(|s| &s.id == id)
    }

    /// Get all TU IDs
    pub fn tu_ids(&self) -> Vec<TUId> {
        self.tu_snapshots.iter().map(|s| s.id).collect()
    }

    /// Build per-TU snapshots from compile database
    pub fn build_tu_snapshots(&mut self) {
        if let Some(ref db) = self.compile_database {
            for (idx, cmd) in db.commands.iter().enumerate() {
                let source_file = PathBuf::from(&cmd.file);
                let mut include_dirs = Vec::new();
                let mut defines = Vec::new();
                let mut flags = Vec::new();
                let mut header_deps = Vec::new();

                if let Ok(args) = shell_words::split(&cmd.command) {
                    for arg in &args {
                        if arg.starts_with("-I") && arg.len() > 2 {
                            include_dirs.push(PathBuf::from(&arg[2..]));
                        } else if arg.starts_with("-D") {
                            if let Some(def) = MacroDefinition::from_arg(arg) {
                                defines.push(def);
                            }
                        } else if !arg.starts_with("-") || arg == "-c" || arg == "-o" {
                            // Skip flags that aren't include paths or defines
                        } else {
                            flags.push(arg.clone());
                        }
                    }
                }

                // Extract header dependencies from clang output if available
                for dir in &cmd.extract_include_dirs() {
                    header_deps.push(PathBuf::from(dir));
                }

                let snapshot = TuSnapshot {
                    id: TUId(idx as u64),
                    source_file,
                    object_file: None,
                    include_dirs,
                    defines,
                    flags,
                    header_dependencies: header_deps,
                    has_definitions: true, // Assume definitions unless proven otherwise
                    symbol_linkage: HashMap::new(),
                };

                self.tu_snapshots.push(snapshot);
            }
        }
    }

    /// Merge metadata from multiple TUs, detecting duplicate symbols
    pub fn merged_symbols(&self) -> HashMap<String, Vec<TUId>> {
        let mut symbols: HashMap<String, Vec<TUId>> = HashMap::new();

        for snapshot in &self.tu_snapshots {
            for (name, linkage) in &snapshot.symbol_linkage {
                // Only track definitions, not declarations
                if linkage.is_definition {
                    symbols
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push(snapshot.id);
                }
            }
        }

        symbols
    }

    /// Find duplicate symbol definitions across TUs
    pub fn find_duplicate_symbols(&self) -> Vec<(String, Vec<TUId>)> {
        let merged = self.merged_symbols();
        merged
            .into_iter()
            .filter(|(_, tus)| tus.len() > 1)
            .collect()
    }
}

// Shell words split (from chimera-c-clang, re-used here)
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
    use tempfile::TempDir;

    #[test]
    fn test_cbuild_config_default() {
        let config = CBuildConfig::default();
        assert_eq!(config.standard, CStandard::C11);
        assert!(config.compile_commands_path.is_none());
    }

    #[test]
    fn test_macro_definition_from_arg() {
        let def = MacroDefinition::from_arg("-DFOO=42").unwrap();
        assert_eq!(def.name, "FOO");
        assert_eq!(def.value, Some("42".to_string()));

        let def = MacroDefinition::from_arg("-DBAR").unwrap();
        assert_eq!(def.name, "BAR");
        assert!(def.value.is_none());

        assert!(MacroDefinition::from_arg("not-a-macro").is_none());
    }

    #[test]
    fn test_macro_definition_apply() {
        let def = MacroDefinition::from_arg("-DNDEBUG").unwrap();
        let mut config = ClangConfig::default();
        def.apply_to_clang_config(&mut config);
        assert_eq!(config.defines, vec![("NDEBUG".to_string(), None)]);
    }

    #[test]
    fn test_include_directory_apply() {
        let dir = IncludeDirectory {
            path: PathBuf::from("/usr/include"),
            is_system: true,
            is_framework: false,
            priority: 0,
        };

        let mut config = ClangConfig::default();
        dir.apply_to_clang_config(&mut config);

        assert!(config.include_paths.is_empty());
        assert!(config
            .system_include_paths
            .contains(&"/usr/include".to_string()));
    }

    #[test]
    fn test_shell_words_split() {
        let result = shell_words::split("clang -Iinclude -DFOO=1 \"test file.c\"").unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "clang");
        assert_eq!(result[3], "test file.c");
    }

    #[test]
    fn test_build_context_from_flags() {
        let context = CBuildContext::from_flags(
            Path::new("/project"),
            vec!["-O2".to_string()],
            vec![PathBuf::from("/include")],
            vec![("FOO".to_string(), Some("1".to_string()))],
        );

        assert!(context.compile_database.is_none());
        assert_eq!(context.config.compiler_flags, vec!["-O2"]);
        assert_eq!(context.config.include_dirs.len(), 1);
        assert_eq!(context.config.defines.len(), 1);
    }

    #[test]
    fn test_compile_database_missing() {
        let temp = TempDir::new().unwrap();
        let result = CompileDatabase::load_from_dir(temp.path());
        assert!(matches!(
            result,
            Err(BuildError::CompileDatabaseNotFound(_))
        ));
    }

    #[test]
    fn test_compiler_identity_default() {
        let identity = CompilerIdentity {
            executable: "clang".to_string(),
            version: "17.0.0".to_string(),
            target: "x86_64-unknown-linux-gnu".to_string(),
            resource_dir: None,
            sysroot: None,
            libc: Some("glibc".to_string()),
        };
        assert_eq!(identity.target, "x86_64-unknown-linux-gnu");
    }

    #[test]
    fn test_linker_library_kind() {
        assert_eq!(
            std::mem::discriminant(&LinkerLibraryKind::Static),
            std::mem::discriminant(&LinkerLibraryKind::Static)
        );
        assert_ne!(
            std::mem::discriminant(&LinkerLibraryKind::Static),
            std::mem::discriminant(&LinkerLibraryKind::Shared)
        );
    }

    #[test]
    fn test_generated_header() {
        let header = GeneratedHeader {
            path: PathBuf::from("build/gen.h"),
            generator_command: Some("gcc -MM gen.h".to_string()),
            content_hash: "abc123".to_string(),
        };
        assert_eq!(header.content_hash, "abc123");
    }

    #[test]
    fn test_generated_header_is_stale_when_missing() {
        let header = GeneratedHeader {
            path: PathBuf::from("/nonexistent/path/gen.h"),
            generator_command: Some("gcc -MM gen.h".to_string()),
            content_hash: "abc123".to_string(),
        };
        assert!(header.is_stale());
    }

    #[test]
    fn test_generated_header_record_generation() {
        let temp_dir = std::env::temp_dir();
        let gen_path = temp_dir.join("test_gen_header.h");
        std::fs::write(&gen_path, "const int VALUE = 42;\n").unwrap();

        let mut header = GeneratedHeader {
            path: gen_path.clone(),
            generator_command: Some("echo '#define VALUE 42' > gen.h".to_string()),
            content_hash: "old_hash".to_string(),
        };

        header.record_generation().unwrap();
        assert!(header.content_hash != "old_hash");

        std::fs::remove_file(&gen_path).ok();
    }

    // =============================================================================
    // Multiple Translation Unit Tests (Task 38)
    // =============================================================================

    #[test]
    fn test_tu_snapshot_basic() {
        let snapshot = TuSnapshot {
            id: TUId(0),
            source_file: PathBuf::from("/project/main.c"),
            object_file: None,
            include_dirs: vec![PathBuf::from("/project/include")],
            defines: vec![MacroDefinition::from_arg("-DFOO=1").unwrap()],
            flags: vec!["-O2".to_string()],
            header_dependencies: vec![PathBuf::from("/project/include/foo.h")],
            has_definitions: true,
            symbol_linkage: HashMap::new(),
        };

        assert_eq!(snapshot.id, TUId(0));
        assert_eq!(snapshot.source_file.to_str().unwrap(), "/project/main.c");
        assert!(snapshot.has_definitions);
    }

    #[test]
    fn test_cbuild_context_tu_snapshots_empty() {
        let context = CBuildContext::from_flags(
            Path::new("/project"),
            vec!["-O2".to_string()],
            vec![PathBuf::from("/include")],
            vec![("FOO".to_string(), Some("1".to_string()))],
        );

        assert!(context.tu_snapshots.is_empty());
        assert_eq!(context.tu_ids(), vec![]);
    }

    #[test]
    fn test_cbuild_context_add_tu_snapshot() {
        let mut context = CBuildContext::from_flags(Path::new("/project"), vec![], vec![], vec![]);

        let snapshot = TuSnapshot {
            id: TUId(42),
            source_file: PathBuf::from("/project/src.c"),
            object_file: None,
            include_dirs: vec![],
            defines: vec![],
            flags: vec![],
            header_dependencies: vec![],
            has_definitions: false,
            symbol_linkage: HashMap::new(),
        };

        context.add_tu_snapshot(snapshot);

        assert_eq!(context.tu_snapshots.len(), 1);
        assert_eq!(context.tu_ids(), vec![TUId(42)]);

        let retrieved = context.get_tu_snapshot(&TUId(42));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, TUId(42));
    }

    #[test]
    fn test_cbuild_context_get_tu_snapshot_not_found() {
        let context = CBuildContext::from_flags(Path::new("/project"), vec![], vec![], vec![]);

        let retrieved = context.get_tu_snapshot(&TUId(999));
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_symbol_linkage_struct() {
        let linkage = SymbolLinkage {
            name: "my_function".to_string(),
            linkage: chimera_c_schema::Linkage::External,
            storage: chimera_c_schema::StorageClass::Extern,
            is_definition: true,
        };

        assert_eq!(linkage.name, "my_function");
        assert!(linkage.is_definition);
        assert_eq!(linkage.linkage, chimera_c_schema::Linkage::External);
    }

    #[test]
    fn test_find_duplicate_symbols_empty() {
        let context = CBuildContext::from_flags(Path::new("/project"), vec![], vec![], vec![]);

        let duplicates = context.find_duplicate_symbols();
        assert!(duplicates.is_empty());
    }

    #[test]
    fn test_cbuild_context_default_has_no_tu_snapshots() {
        let _config = CBuildConfig::default();
        // CBuildContext doesn't have Default, so we test via from_flags
        let context = CBuildContext::from_flags(Path::new("."), vec![], vec![], vec![]);
        assert!(context.tu_snapshots.is_empty());
    }

    // =============================================================================
    // Static and Shared Library Support Tests (Task 39)
    // =============================================================================

    #[test]
    fn test_link_plan_new() {
        let plan = LinkPlan::new();
        assert!(plan.static_libraries.is_empty());
        assert!(plan.shared_libraries.is_empty());
        assert!(plan.frameworks.is_empty());
        assert!(plan.search_dirs.is_empty());
        assert!(plan.linker_flags.is_empty());
        assert!(plan.rpath_entries.is_empty());
        assert!(plan.soname.is_none());
    }

    #[test]
    fn test_link_plan_add_library_flag_m() {
        let mut plan = LinkPlan::new();
        let result = plan.add_library_from_flag("-lm");
        assert!(result.is_some());
        assert_eq!(plan.shared_libraries.len(), 1);
        assert_eq!(plan.shared_libraries[0].name, "m");
        assert!(plan.shared_libraries[0].is_system);
    }

    #[test]
    fn test_link_plan_add_library_static() {
        let mut plan = LinkPlan::new();
        plan.add_library_from_flag("-lm").unwrap();
        // Static link can be forced with -static flag (handled by linker, not library parser)
        // We just verify add_library_from_flag handles it gracefully
        let result = plan.add_library_from_flag("-static");
        // -static is not a library flag, returns None
        assert!(result.is_none());
    }

    #[test]
    fn test_link_plan_add_explicit_static_library() {
        let mut plan = LinkPlan::new();
        let result = plan.add_library_from_flag("/usr/lib/libfoo.a");
        assert!(result.is_some());
        assert_eq!(plan.static_libraries.len(), 1);
        assert_eq!(plan.static_libraries[0].name, "foo");
        assert_eq!(
            plan.static_libraries[0]
                .path
                .as_ref()
                .unwrap()
                .to_str()
                .unwrap(),
            "/usr/lib/libfoo.a"
        );
    }

    #[test]
    fn test_link_plan_add_explicit_shared_library() {
        let mut plan = LinkPlan::new();
        let result = plan.add_library_from_flag("/usr/lib/libbar.so");
        assert!(result.is_some());
        assert_eq!(plan.shared_libraries.len(), 1);
        assert_eq!(plan.shared_libraries[0].name, "bar");
    }

    #[test]
    fn test_link_plan_add_search_dir() {
        let mut plan = LinkPlan::new();
        let result = plan.add_library_from_flag("-L/usr/local/lib");
        assert!(result.is_some());
        assert_eq!(plan.search_dirs.len(), 1);
        assert_eq!(plan.search_dirs[0].path.to_str().unwrap(), "/usr/local/lib");
        assert!(!plan.search_dirs[0].is_system);
    }

    #[test]
    fn test_link_plan_add_rpath() {
        let mut plan = LinkPlan::new();
        let result = plan.add_library_from_flag("-Wl,-rpath,/usr/local/lib");
        assert!(result.is_some());
        assert_eq!(plan.rpath_entries.len(), 1);
        assert_eq!(plan.rpath_entries[0], "/usr/local/lib");
    }

    #[test]
    fn test_link_plan_add_soname() {
        let mut plan = LinkPlan::new();
        let result = plan.add_library_from_flag("-soname:libfoo.so.1");
        assert!(result.is_some());
        assert_eq!(plan.soname.as_ref().unwrap(), "libfoo.so.1");
    }

    #[test]
    fn test_link_plan_library_names() {
        let mut plan = LinkPlan::new();
        plan.add_library_from_flag("-lm").unwrap();
        plan.add_library_from_flag("-lpthread").unwrap();
        plan.add_library_from_flag("/usr/lib/libssl.a").unwrap();

        let names = plan.library_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"m".to_string()));
        assert!(names.contains(&"pthread".to_string()));
        assert!(names.contains(&"ssl".to_string()));
    }

    #[test]
    fn test_link_plan_is_system_library() {
        let mut plan = LinkPlan::new();
        plan.add_library_from_flag("-lm").unwrap();
        plan.add_library_from_flag("/usr/lib/libfoo.a").unwrap();

        assert!(plan.is_system_library("m"));
        assert!(!plan.is_system_library("foo")); // Not system since explicit path
        assert!(!plan.is_system_library("nonexistent"));
    }

    #[test]
    fn test_link_plan_default() {
        let plan = LinkPlan::default();
        assert!(plan.static_libraries.is_empty());
        assert!(plan.soname.is_none());
    }

    #[test]
    fn test_link_search_dir_struct() {
        let search = LinkSearchDir {
            path: PathBuf::from("/opt/lib"),
            is_system: true,
        };
        assert_eq!(search.path.to_str().unwrap(), "/opt/lib");
        assert!(search.is_system);
    }

    // =============================================================================
    // C Fixture Workspace Tests (Task 30)
    // =============================================================================

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root should resolve")
    }

    #[test]
    fn test_c_fixtures_directory_exists() {
        // Verify C fixtures directory exists at expected location
        let fixtures_path = repo_root().join("tests/c-fixtures");
        assert!(
            fixtures_path.exists(),
            "tests/c-fixtures directory should exist"
        );
    }

    #[test]
    fn test_c_fixtures_basic_fixture_exists() {
        let fixtures_path = repo_root().join("tests/c-fixtures/basic");
        assert!(fixtures_path.exists(), "basic fixture should exist");
        let header = fixtures_path.join("basic.h");
        assert!(header.exists(), "basic.h should exist");
        let source = fixtures_path.join("basic.c");
        assert!(source.exists(), "basic.c should exist");
    }

    #[test]
    fn test_c_fixtures_header_only_fixture_exists() {
        let fixtures_path = repo_root().join("tests/c-fixtures/header-only");
        assert!(fixtures_path.exists(), "header-only fixture should exist");
        let header = fixtures_path.join("header.h");
        assert!(header.exists(), "header.h should exist");
    }

    #[test]
    fn test_c_fixtures_layout_fixture_exists() {
        let fixtures_path = repo_root().join("tests/c-fixtures/layout");
        assert!(fixtures_path.exists(), "layout fixture should exist");
        let header = fixtures_path.join("layout.h");
        assert!(header.exists(), "layout.h should exist");
    }

    #[test]
    fn test_c_fixtures_bitfields_fixture_exists() {
        let fixtures_path = repo_root().join("tests/c-fixtures/bitfields");
        assert!(fixtures_path.exists(), "bitfields fixture should exist");
        let header = fixtures_path.join("bitfield.h");
        assert!(header.exists(), "bitfield.h should exist");
    }

    #[test]
    fn test_c_fixtures_errors_fixture_exists() {
        let fixtures_path = repo_root().join("tests/c-fixtures/errors");
        assert!(fixtures_path.exists(), "errors fixture should exist");
        let header = fixtures_path.join("errors.h");
        assert!(header.exists(), "errors.h should exist");
    }

    #[test]
    fn test_c_fixtures_callbacks_fixture_exists() {
        let fixtures_path = repo_root().join("tests/c-fixtures/callbacks");
        assert!(fixtures_path.exists(), "callbacks fixture should exist");
        let header = fixtures_path.join("callbacks.h");
        assert!(header.exists(), "callbacks.h should exist");
    }

    #[test]
    fn test_c_fixtures_basic_has_compile_commands() {
        let fixtures_path = repo_root().join("tests/c-fixtures/basic/compile_commands.json");
        assert!(
            fixtures_path.exists(),
            "compile_commands.json should exist for basic fixture"
        );
    }

    #[test]
    fn test_c_fixtures_readme_exists() {
        let readme_path = repo_root().join("tests/c-fixtures/README.md");
        assert!(readme_path.exists(), "C fixtures README should exist");
    }

    #[test]
    fn test_flags_hash_deterministic() {
        let config1 = CBuildConfig {
            compiler_flags: vec!["-O2".to_string(), "-Wall".to_string()],
            standard: CStandard::C11,
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            ..Default::default()
        };

        let config2 = CBuildConfig {
            compiler_flags: vec!["-Wall".to_string(), "-O2".to_string()],
            standard: CStandard::C11,
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            ..Default::default()
        };

        // Flags are sorted, so order shouldn't matter
        assert_eq!(config1.flags_hash(), config2.flags_hash());
    }

    #[test]
    fn test_flags_hash_changes_with_optimization() {
        let config1 = CBuildConfig {
            compiler_flags: vec!["-O0".to_string()],
            standard: CStandard::C11,
            ..Default::default()
        };

        let config2 = CBuildConfig {
            compiler_flags: vec!["-O2".to_string()],
            standard: CStandard::C11,
            ..Default::default()
        };

        assert_ne!(config1.flags_hash(), config2.flags_hash());
    }

    #[test]
    fn test_flags_hash_includes_target() {
        let config1 = CBuildConfig {
            compiler_flags: vec!["-O2".to_string()],
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            ..Default::default()
        };

        let config2 = CBuildConfig {
            compiler_flags: vec!["-O2".to_string()],
            target: Some("aarch64-unknown-linux-gnu".to_string()),
            ..Default::default()
        };

        assert_ne!(config1.flags_hash(), config2.flags_hash());
    }

    #[test]
    fn test_flags_hash_empty_for_default() {
        let config = CBuildConfig::default();
        // With no flags, target, etc., we still get a hash (it's deterministic)
        let hash = config.flags_hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 16); // 16 hex chars = 8 bytes
    }
}
