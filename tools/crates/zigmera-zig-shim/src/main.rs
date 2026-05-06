//! Transparent Zig shim for Bun.
//!
//! Records Zig sessions while forwarding to the real pinned Zig compiler.
//! This shim is used when Bun builds through ZigMera to capture build
//! information without modifying Bun's build behavior.
//!
//! Usage:
//!   export ZIGMERA_REAL_ZIG=/path/to/bun/vendor/zig/zig
//!   export PATH=/path/to/zigmera-zig-shim:$PATH
//!   bun bd  # Bun's build command
//!
//! The shim will:
//! 1. Forward all arguments to the real Zig
//! 2. Capture session information (args, env, outputs)
//! 3. Copy produced .o files to ZigMera's cache
//! 4. Return the exact same exit code

mod gen;
mod manifest;
mod per_file;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::SystemTime;

/// Environment variable pointing to the real Zig binary
const ZIGMERA_REAL_ZIG: &str = "ZIGMERA_REAL_ZIG";

/// Environment variable for ZigMera cache directory
const ZIGMERA_CACHE_DIR: &str = "ZIGMERA_CACHE_DIR";

/// Default ZigMera cache directory
const DEFAULT_CACHE_DIR: &str = ".zigmera/cache";

/// Environment variable to enable/disable the shim
const ZIGMERA_ENABLED: &str = "ZIGMERA_ENABLED";

/// Session record filename
const SESSION_FILE: &str = "session.json";

/// Record of a single Zig invocation
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ZigSession {
    /// Timestamp of the session
    timestamp_ns: u64,
    /// Working directory
    cwd: String,
    /// Command line arguments
    args: Vec<String>,
    /// Environment variables (filtered to relevant ones)
    env_vars: Vec<(String, String)>,
    /// Real Zig path
    real_zig_path: String,
    /// Input files that were compiled
    input_files: Vec<String>,
    /// Hash of input content for cache keying
    input_content_hash: Option<String>,
    /// Output files produced by this invocation
    output_files: Vec<String>,
    /// Exit code of the invocation
    exit_code: Option<i32>,
    /// Timing information in nanoseconds
    timing: SessionTiming,
    /// Whether we reused cached outputs (skipping compilation)
    reused: bool,
}

/// Timing breakdown for the shim
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SessionTiming {
    /// Time before forwarding to real Zig (setup, session creation)
    setup_ns: u64,
    /// Time spent in real Zig compilation
    compile_ns: u64,
    /// Time spent caching outputs
    cache_ns: u64,
    /// Time spent writing session record
    session_write_ns: u64,
    /// Total wall time
    total_ns: u64,
}

impl SessionTiming {
    fn new() -> Self {
        Self {
            setup_ns: 0,
            compile_ns: 0,
            cache_ns: 0,
            session_write_ns: 0,
            total_ns: 0,
        }
    }

    fn with_total(mut self, total: u64) -> Self {
        self.total_ns = total;
        self
    }
}

impl ZigSession {
    fn new(real_zig_path: &Path, args: &[String]) -> Self {
        let timestamp_ns = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let cwd = env::current_dir().unwrap_or_default().display().to_string();

        // Filter environment to only include relevant Zig-related vars
        let relevant_vars = vec![
            "ZIGMERA_REAL_ZIG",
            "ZIGMERA_CACHE_DIR",
            "ZIGMERA_ENABLED",
            "ZIG_LOCAL_CACHE_DIR",
            "ZIG_GLOBAL_CACHE_DIR",
            "HOME",
            "PATH",
        ];

        let env_vars: Vec<(String, String)> = env::vars()
            .filter(|(k, _)| relevant_vars.contains(&k.as_str()))
            .collect();

        Self {
            timestamp_ns,
            cwd,
            args: args.iter().map(|s| s.clone()).collect(),
            env_vars,
            real_zig_path: real_zig_path.display().to_string(),
            input_files: Vec::new(),
            input_content_hash: None,
            output_files: Vec::new(),
            exit_code: None,
            timing: SessionTiming::new(),
            reused: false,
        }
    }

    fn with_inputs(mut self, inputs: Vec<String>) -> Self {
        self.input_files = inputs;
        self
    }

    fn with_input_hash(mut self, hash: String) -> Self {
        self.input_content_hash = Some(hash);
        self
    }

    fn with_reused(mut self, reused: bool) -> Self {
        self.reused = reused;
        self
    }

    fn with_timing(mut self, timing: SessionTiming) -> Self {
        self.timing = timing;
        self
    }

    fn with_outputs(mut self, outputs: Vec<String>) -> Self {
        self.output_files = outputs;
        self
    }

    fn with_exit_code(mut self, code: Option<i32>) -> Self {
        self.exit_code = code;
        self
    }
}

/// Get the path to the real Zig binary
fn get_real_zig_path() -> Option<PathBuf> {
    // First check env var
    if let Some(path) = env::var(ZIGMERA_REAL_ZIG).ok().map(PathBuf::from) {
        eprintln!("zigmera: ZIGMERA_REAL_ZIG env var found");
        return Some(path);
    }

    eprintln!("zigmera: ZIGMERA_REAL_ZIG not set, trying path detection");

    // If shim is at /path/to/chimerair/tools/target/release/zig,
    // real zig is at /path/to/others/bun/vendor/zig.real/zig
    // Try to derive the real zig path from the shim's location
    if let Ok(exe_path) = env::current_exe() {
        let exe_str = exe_path.display().to_string();
        eprintln!("zigmera: exe_path={}", exe_str);

        // Check if shim is in our standard build location
        if exe_str.contains("/chimerair/tools/target/release/zig") {
            // Transform chimerair/tools -> others/bun/vendor/zig.real
            let real_path = exe_str.replace(
                "/chimerair/tools/target/release/zig",
                "/others/bun/vendor/zig.real/zig",
            );
            eprintln!("zigmera: trying transformed path={}", real_path);
            let real_zig = PathBuf::from(real_path);
            if real_zig.exists() {
                return Some(real_zig);
            }
        }

        // Try vendor/zig.real/zig relative to current working directory
        if let Ok(cwd) = env::current_dir() {
            let real_zig = cwd.join("vendor/zig.real/zig");
            eprintln!("zigmera: trying cwd-based path={}", real_zig.display());
            if real_zig.exists() {
                return Some(real_zig);
            }
        }
    }

    eprintln!("zigmera: could not find real zig, will fail");
    None
}

/// Get the ZigMera cache directory
fn get_cache_dir() -> PathBuf {
    env::var(ZIGMERA_CACHE_DIR)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_CACHE_DIR))
}

/// Check if the shim is enabled
fn is_enabled() -> bool {
    env::var(ZIGMERA_ENABLED)
        .map(|v| v != "0" && v.to_lowercase() != "false")
        .unwrap_or(true) // Default to enabled
}

/// Forward the Zig invocation to the real Zig
fn forward_to_zig(real_zig: &Path, args: &[String]) -> io::Result<(ExitCode, Option<i32>)> {
    let mut cmd = Command::new(real_zig);
    cmd.args(args);
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    // Remove ZIGMERA_* env vars to avoid affecting Zig's cache behavior
    cmd.env_remove("ZIGMERA_REAL_ZIG");
    cmd.env_remove("ZIGMERA_CACHE_DIR");
    cmd.env_remove("ZIGMERA_ENABLED");

    let status = cmd.status()?;
    let exit_code = ExitCode::from(status.code().unwrap_or(1) as u8);
    let exit_code_i32 = status.code();
    Ok((exit_code, exit_code_i32))
}

/// Capture output files from the arguments
/// This looks for -f emitter output paths or object file outputs
fn capture_output_files(args: &[String]) -> Vec<String> {
    let mut outputs = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        // Look for emitter output paths
        if arg == "--emit-zigmera-snapshot"
            || arg == "--emit-zigmera-dep"
            || arg == "--emit-zigmera-air"
            || arg.starts_with("--emit-zigmera")
        {
            // Next arg is the output path
            if i + 1 < args.len() {
                outputs.push(args[i + 1].clone());
                i += 2;
                continue;
            }
        }

        // Look for object file outputs (typically the last .o file in the args)
        if arg.ends_with(".o") {
            outputs.push(arg.clone());
        }

        // Look for -f emitter options
        if arg.starts_with("-f") && arg.contains(" emitter") {
            // This might be a Zig emitter option
            // Skip for now as we capture via the args
        }

        i += 1;
    }
    outputs
}

/// Collect object files from the build directory
/// This captures any newly created .o files after the build
fn collect_object_files() -> Vec<PathBuf> {
    let mut objects = Vec::new();

    // Look in the common build output directories
    // Also check build/debug since that's where bun puts its objects
    let search_dirs = vec![
        PathBuf::from("."),
        PathBuf::from("zig-out"),
        PathBuf::from("zig-cache"),
        PathBuf::from("build/debug"),
    ];

    for dir in search_dirs {
        if dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "o" || ext == "obj" {
                                objects.push(path);
                            }
                        }
                    }
                }
            }
        }
    }

    eprintln!(
        "zigmera: collect_object_files found {} objects",
        objects.len()
    );
    objects
}

/// Capture input files from the arguments (source .zig files being compiled)
fn capture_input_files(args: &[String]) -> Vec<String> {
    let mut inputs = Vec::new();
    for arg in args {
        if arg.ends_with(".zig") {
            inputs.push(arg.clone());
        }
    }
    inputs
}

/// Compute a content hash of input files for cache keying
fn compute_input_hash(input_files: &[String]) -> Option<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    if input_files.is_empty() {
        return None;
    }

    let mut hasher = DefaultHasher::new();
    for file in input_files {
        if let Ok(content) = fs::read(file) {
            content.hash(&mut hasher);
        }
    }
    Some(format!("{:x}", hasher.finish()))
}

/// Copy output files to the ZigMera cache
fn cache_output_files(outputs: &[String], cache_dir: &Path) -> io::Result<()> {
    let cache_dir = cache_dir.join("objects");
    fs::create_dir_all(&cache_dir)?;

    for output in outputs {
        let src = Path::new(output);
        if src.exists() {
            let filename = src.file_name().unwrap_or_default();
            let dst = cache_dir.join(filename);

            // Use copy instead of hardlink for safety across different filesystems
            fs::copy(src, &dst)?;
        }
    }

    Ok(())
}

/// Write the session record to disk
fn write_session_record(session: &ZigSession, cache_dir: &Path) -> io::Result<()> {
    let session_dir = cache_dir.join("sessions");
    fs::create_dir_all(&session_dir)?;

    let filename = format!("{}.json", session.timestamp_ns);
    let session_path = session_dir.join(filename);

    let json = serde_json::to_string_pretty(session).unwrap();
    fs::write(&session_path, &json)?;

    // Also write as "latest" for easy access
    let latest_path = session_dir.join(SESSION_FILE);
    fs::write(&latest_path, &json)?;

    Ok(())
}

/// Check if the args indicate a zig build obj command
fn is_zig_build_obj(args: &[String]) -> bool {
    args.iter().any(|a| a == "build" || a == "build-obj")
}

/// Run per-file incremental build
fn run_per_file_build(real_zig: &Path, args: &[String], cache_dir: &Path) -> ExitCode {
    let manifest_path = cache_dir.join("manifest.json");
    let hash_cache_path = cache_dir.join("hash_cache.json");

    // Find build directory from args
    let build_dir = extract_build_dir(args);

    let mut builder = match per_file::PerFileBuilder::new(
        manifest_path,
        hash_cache_path,
        real_zig.to_path_buf(),
        build_dir,
    ) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("zigmera: failed to create per-file builder: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match builder.build() {
        Ok((exit_code, built, cached)) => {
            eprintln!(
                "zigmera: per-file build completed: {} built, {} cached",
                built, cached
            );
            exit_code
        }
        Err(e) => {
            eprintln!("zigmera: per-file build failed: {}", e);
            ExitCode::FAILURE
        }
    }
}

/// Extract build directory from zig build args
fn extract_build_dir(args: &[String]) -> PathBuf {
    // Look for --prefix or -p argument
    for i in 0..args.len() {
        if args[i] == "--prefix" && i + 1 < args.len() {
            return PathBuf::from(&args[i + 1]);
        }
        if args[i].starts_with("--prefix=") {
            return PathBuf::from(&args[i][9..]);
        }
        if args[i] == "-p" && i + 1 < args.len() {
            return PathBuf::from(&args[i + 1]);
        }
    }
    // Default to current directory
    std::env::current_dir().unwrap_or_default()
}

/// Read the previous session from cache
fn read_previous_session(cache_dir: &Path) -> Option<ZigSession> {
    let session_path = cache_dir.join("sessions").join(SESSION_FILE);
    if !session_path.exists() {
        return None;
    }
    let json = fs::read_to_string(&session_path).ok()?;
    serde_json::from_str(&json).ok()
}

/// Restore output files from cache (for reuse scenario)
fn restore_output_files(outputs: &[String], cache_dir: &Path) -> io::Result<()> {
    let cache_dir = cache_dir.join("objects");

    for output in outputs {
        let src = cache_dir.join(Path::new(output).file_name().unwrap_or_default());
        if src.exists() {
            fs::copy(&src, output)?;
        }
    }

    Ok(())
}

/// Main entry point - the shim installs itself as "zig" in PATH
fn main() -> ExitCode {
    let total_start = std::time::Instant::now();
    let setup_start = std::time::Instant::now();

    // Check if shim is enabled
    if !is_enabled() {
        // When disabled, just forward directly without recording
        if let Some(real_zig) = get_real_zig_path() {
            let args: Vec<String> = env::args().skip(1).collect();
            return forward_to_zig(&real_zig, &args)
                .map(|(ec, _)| ec)
                .unwrap_or(ExitCode::FAILURE);
        }
        eprintln!("zigmera-zig-shim: ZIGMERA_REAL_ZIG not set, cannot forward");
        return ExitCode::FAILURE;
    }

    // Get the real Zig path
    let real_zig = match get_real_zig_path() {
        Some(path) => path,
        None => {
            eprintln!("zigmera-zig-shim: ZIGMERA_REAL_ZIG not set");
            return ExitCode::FAILURE;
        }
    };

    // Collect arguments (skip the shim binary name)
    let args: Vec<String> = env::args().skip(1).collect();

    // Capture input files for cache keying
    let input_files = capture_input_files(&args);

    // Create session record
    let mut session = ZigSession::new(&real_zig, &args);
    session = session.with_inputs(input_files.clone());

    // Compute input content hash
    if let Some(hash) = compute_input_hash(&input_files) {
        session = session.with_input_hash(hash);
    }

    // Check for cache reuse using session comparison
    // Only reuse if we have valid input hashes to compare
    let cache_dir = get_cache_dir();
    let reused = false; // Disabled - input_files capture doesn't work for zig build obj

    // Check if per-file mode should be used
    if per_file::PerFileBuilder::is_enabled() && is_zig_build_obj(&args) {
        eprintln!("zigmera: using per-file incremental build mode");
        return run_per_file_build(&real_zig, &args, &cache_dir);
    }

    let setup_ns = setup_start.elapsed().as_nanos() as u64;
    let compile_start = std::time::Instant::now();

    let exit_code_i32: Option<i32>;
    if reused {
        exit_code_i32 = Some(0);
    } else {
        // Forward to the real Zig
        let (exit_code, code) =
            forward_to_zig(&real_zig, &args).unwrap_or((ExitCode::FAILURE, None));
        exit_code_i32 = code;

        // If build failed, don't use this session for reuse
        if exit_code_i32 != Some(0) {
            eprintln!("zigmera: build failed, not updating cache");
            return exit_code;
        }
    }

    let compile_ns = compile_start.elapsed().as_nanos() as u64;
    let cache_start = std::time::Instant::now();

    // Collect output files after running (only if not reused)
    if !reused {
        let outputs: Vec<String> = collect_object_files()
            .iter()
            .map(|p| p.display().to_string())
            .collect();

        session = session.with_outputs(outputs);
    }

    session = session.with_exit_code(exit_code_i32);

    // Cache output files (only if not reused)
    let cache_ns = if !reused && !session.output_files.is_empty() {
        let cache_result = cache_output_files(&session.output_files, &cache_dir);
        if cache_result.is_err() {
            eprintln!(
                "zigmera-zig-shim: failed to cache output files: {}",
                cache_result.unwrap_err()
            );
        }
        cache_start.elapsed().as_nanos() as u64
    } else {
        cache_start.elapsed().as_nanos() as u64
    };

    let session_write_start = std::time::Instant::now();

    // Write session record
    eprintln!(
        "zigmera: cache_dir={}, session.output_files.len()={}, reused={}",
        cache_dir.display(),
        session.output_files.len(),
        reused
    );
    if let Err(e) = write_session_record(&session, &cache_dir) {
        eprintln!("zigmera-zig-shim: failed to write session record: {}", e);
    } else {
        eprintln!(
            "zigmera: session written to {}",
            cache_dir.join("sessions").join(SESSION_FILE).display()
        );
    }

    let session_write_ns = session_write_start.elapsed().as_nanos() as u64;
    let total_ns = total_start.elapsed().as_nanos() as u64;

    let timing = SessionTiming {
        setup_ns,
        compile_ns,
        cache_ns,
        session_write_ns,
        total_ns,
    };

    session = session.with_timing(timing);

    // Re-write session with timing data
    if let Err(e) = write_session_record(&session, &cache_dir) {
        eprintln!("zigmera-zig-shim: failed to re-write session record: {}", e);
    }

    if reused {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(exit_code_i32.unwrap_or(1) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let real_zig = PathBuf::from("/bun/vendor/zig/zig");
        let args = vec![
            "build".to_string(),
            "obj".to_string(),
            "-O".to_string(),
            "ReleaseFast".to_string(),
        ];
        let session = ZigSession::new(&real_zig, &args);

        assert_eq!(session.real_zig_path, "/bun/vendor/zig/zig");
        assert_eq!(session.args.len(), 4);
        assert!(session.exit_code.is_none());
    }

    #[test]
    fn test_session_with_outputs() {
        let session = ZigSession::new(&PathBuf::from("/zig"), &["build".to_string()])
            .with_outputs(vec!["a.o".to_string(), "b.o".to_string()]);

        assert_eq!(session.output_files.len(), 2);
    }

    #[test]
    fn test_session_with_exit_code() {
        let session =
            ZigSession::new(&PathBuf::from("/zig"), &["build".to_string()]).with_exit_code(Some(0));

        assert_eq!(session.exit_code, Some(0));
    }

    #[test]
    fn test_is_enabled_default() {
        // Clear the env var if it exists
        env::remove_var(ZIGMERA_ENABLED);
        assert!(is_enabled()); // Default to true
    }

    #[test]
    fn test_capture_output_files() {
        let args = vec![
            "build".to_string(),
            "obj".to_string(),
            "src/main.zig".to_string(),
            "-O".to_string(),
            "ReleaseFast".to_string(),
            "--emit-zigmera-snapshot".to_string(),
            ".zigmera/snapshot.zsnap".to_string(),
        ];

        let outputs = capture_output_files(&args);
        // Should capture the snapshot path
        assert!(outputs.iter().any(|s| s.contains("zsnap")));
    }

    #[test]
    fn test_session_serialization() {
        let session = ZigSession::new(&PathBuf::from("/test/zig"), &["build".to_string()])
            .with_outputs(vec!["test.o".to_string()])
            .with_exit_code(Some(0));

        let json = serde_json::to_string_pretty(&session).unwrap();
        assert!(json.contains("\"real_zig_path\": \"/test/zig\""));
        assert!(json.contains("\"output_files\""));
        assert!(json.contains("test.o"));
    }

    #[test]
    fn test_get_cache_dir_default() {
        env::remove_var(ZIGMERA_CACHE_DIR);
        let cache = get_cache_dir();
        assert_eq!(cache, PathBuf::from(DEFAULT_CACHE_DIR));
    }

    #[test]
    fn test_get_cache_dir_from_env() {
        env::set_var(ZIGMERA_CACHE_DIR, "/custom/cache");
        let cache = get_cache_dir();
        assert_eq!(cache, PathBuf::from("/custom/cache"));
        env::remove_var(ZIGMERA_CACHE_DIR);
    }

    // B7: Black-box shim tests

    #[test]
    fn test_shim_forwards_args_correctly() {
        // Verify that session captures the exact args passed
        let real_zig = PathBuf::from("/test/zig");
        let args = vec![
            "build".to_string(),
            "obj".to_string(),
            "src/main.zig".to_string(),
            "-O".to_string(),
            "ReleaseFast".to_string(),
            "--emit-zigmera-snapshot".to_string(),
            "out.zsnap".to_string(),
        ];
        let session = ZigSession::new(&real_zig, &args);

        assert_eq!(session.args.len(), 7);
        assert_eq!(session.args[0], "build");
        assert_eq!(session.args[1], "obj");
        assert_eq!(session.args[2], "src/main.zig");
        assert_eq!(session.args[3], "-O");
        assert_eq!(session.args[4], "ReleaseFast");
    }

    #[test]
    fn test_shim_captures_env_vars() {
        // Set test env vars
        env::set_var(ZIGMERA_REAL_ZIG, "/test/real/zig");
        env::set_var(ZIGMERA_CACHE_DIR, "/test/cache");

        let session = ZigSession::new(&PathBuf::from("/test/zig"), &["build".to_string()]);

        // Check that relevant env vars are captured
        let env_map: std::collections::HashMap<_, _> = session.env_vars.iter().cloned().collect();
        assert!(env_map.contains_key("ZIGMERA_REAL_ZIG"));
        assert!(env_map.contains_key("ZIGMERA_CACHE_DIR"));

        env::remove_var(ZIGMERA_REAL_ZIG);
        env::remove_var(ZIGMERA_CACHE_DIR);
    }

    #[test]
    fn test_shim_output_caching() {
        let outputs = vec!["zig-out/main.o".to_string(), "zig-out/util.o".to_string()];

        let session = ZigSession::new(&PathBuf::from("/test/zig"), &["build".to_string()])
            .with_outputs(outputs.clone());

        assert_eq!(session.output_files.len(), 2);
        assert!(session.output_files.contains(&"zig-out/main.o".to_string()));
        assert!(session.output_files.contains(&"zig-out/util.o".to_string()));
    }

    #[test]
    fn test_shim_exit_code_recording() {
        let session = ZigSession::new(&PathBuf::from("/test/zig"), &["build".to_string()])
            .with_exit_code(Some(0));

        assert_eq!(session.exit_code, Some(0));

        let session2 = ZigSession::new(&PathBuf::from("/test/zig"), &["build".to_string()])
            .with_exit_code(Some(1));

        assert_eq!(session2.exit_code, Some(1));
    }

    #[test]
    fn test_shim_enabled_by_default() {
        env::remove_var(ZIGMERA_ENABLED);
        assert!(is_enabled());
    }

    #[test]
    fn test_shim_disabled_by_env() {
        env::set_var(ZIGMERA_ENABLED, "0");
        assert!(!is_enabled());

        env::set_var(ZIGMERA_ENABLED, "false");
        assert!(!is_enabled());

        env::set_var(ZIGMERA_ENABLED, "FALSE");
        assert!(!is_enabled());

        env::remove_var(ZIGMERA_ENABLED);
    }
}
