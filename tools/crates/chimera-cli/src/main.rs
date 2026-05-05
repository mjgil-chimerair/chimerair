//! chimera-cli - Chimera toolchain CLI
//!
//! Main entrypoint for the Chimera polyglot IR toolchain.

mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::LevelFilter;
use std::env;

#[derive(Parser)]
#[command(
    name = "chimera",
    about = "Chimera polyglot IR toolchain",
    version = "0.1.0"
)]
struct Cli {
    #[arg(long, global = true, help = "Enable verbose logging")]
    verbose: bool,

    #[arg(long, global = true, help = "Colorize output", default_value_t = true)]
    color: bool,

    #[arg(long, global = true, help = "Input language (c, rust, zig)")]
    input_lang: Option<String>,

    #[arg(long, global = true, help = "Input file path")]
    input: Option<std::path::PathBuf>,

    #[arg(long, global = true, help = "Emit metadata output")]
    emit_metadata: bool,

    #[arg(long, global = true, help = "Emit object file output")]
    emit_object: bool,

    #[arg(long, global = true, help = "Emit proof output")]
    emit_proof: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check project validity without building
    Check {
        #[arg(short, long, help = "Project manifest path")]
        manifest: Option<std::path::PathBuf>,

        #[arg(short, long, help = "Target triple")]
        target: Option<String>,
    },

    /// Build project and produce output
    Build {
        #[arg(short, long, help = "Project manifest path")]
        manifest: Option<std::path::PathBuf>,

        #[arg(short, long, help = "Target triple")]
        target: Option<String>,

        #[arg(short, long, help = "Output directory")]
        output: Option<std::path::PathBuf>,

        #[arg(long, help = "Skip proof verification")]
        skip_proof: bool,
    },

    /// Link prebuilt artifacts
    Link {
        #[arg(help = "Object files to link")]
        objects: Vec<std::path::PathBuf>,

        #[arg(short, long, help = "Output binary name")]
        output: Option<String>,

        #[arg(short, long, help = "Target triple")]
        target: Option<String>,
    },

    /// Explain proof failures and diagnostics
    Explain {
        #[arg(help = "Proof or diagnostic file to explain")]
        file: std::path::PathBuf,

        #[arg(short, long, help = "Explanation level")]
        level: Option<String>,
    },

    /// Clean build artifacts
    Clean {
        #[arg(short, long, help = "Remove all artifacts")]
        all: bool,
    },

    /// C compiler integration commands
    C {
        #[command(subcommand)]
        command: commands::c::CCommands,
    },

    /// Rust compiler integration commands
    Rust {
        #[command(subcommand)]
        command: commands::rust::RustCommands,
    },

    /// Read and validate Zig semantic snapshot files
    Snapshot {
        #[command(subcommand)]
        command: commands::snapshot::SnapshotCommand,
    },

    /// Bun toolchain integration commands
    Bun {
        #[command(subcommand)]
        command: commands::bun::BunCommands,
    },

    /// Display version information
    Version,
}

fn setup_logging(verbose: bool) {
    let level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    env_logger::Builder::from_default_env()
        .filter_level(level)
        .format_timestamp_millis()
        .init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    setup_logging(cli.verbose);

    // Handle Rust frontend input path (Task 118)
    if let (Some(input_lang), Some(input)) = (&cli.input_lang, &cli.input) {
        if input_lang == "rust" {
            let mut emit_metadata = false;
            let mut emit_object = false;
            let mut emit_proof = false;
            // Access global args via cli (already parsed)
            return commands::rust::run_frontend(
                input,
                cli.emit_metadata,
                cli.emit_object,
                cli.emit_proof,
            );
        }
    }

    match cli.command {
        Commands::Check { manifest, target } => {
            commands::check::run(manifest, target)?;
        }
        Commands::Build {
            manifest,
            target,
            output,
            skip_proof,
        } => {
            commands::build::run(manifest, target, output, skip_proof)?;
        }
        Commands::Link {
            objects,
            output,
            target,
        } => {
            commands::link::run(objects, output, target)?;
        }
        Commands::Explain { file, level } => {
            commands::explain::run(file, level)?;
        }
        Commands::Clean { all } => {
            commands::clean::run(all)?;
        }
        Commands::C { command } => match command {
            commands::c::CCommands::Snapshot {
                input,
                output,
                target,
                compiler,
            } => {
                commands::c::snapshot(&input, output.as_deref(), &target, &compiler)?;
            }
            commands::c::CCommands::Lower { input, output } => {
                commands::c::lower(&input, output.as_deref())?;
            }
            commands::c::CCommands::Verify { input, reference } => {
                commands::c::verify(&input, &reference)?;
            }
            commands::c::CCommands::Explain { input, symbol } => {
                commands::c::explain(&input, &symbol)?;
            }
            commands::c::CCommands::Cache { action } => {
                commands::c::cache(&action)?;
            }
            commands::c::CCommands::Proof { input, output } => {
                commands::c::proof(&input, output.as_deref())?;
            }
        },
        Commands::Rust { command } => match command {
            commands::rust::RustCommands::Snapshot {
                input,
                output,
                target,
                mode,
            } => {
                commands::rust::snapshot(&input, output.as_deref(), &target, &mode)?;
            }
            commands::rust::RustCommands::Lower { input, output } => {
                commands::rust::lower(&input, output.as_deref())?;
            }
            commands::rust::RustCommands::Verify { input, reference } => {
                commands::rust::verify(&input, &reference)?;
            }
            commands::rust::RustCommands::Explain { input, symbol } => {
                commands::rust::explain(&input, &symbol)?;
            }
            commands::rust::RustCommands::Cache { action } => {
                commands::rust::cache(&action)?;
            }
            commands::rust::RustCommands::Proof { input, output } => {
                commands::rust::proof(&input, output.as_deref())?;
            }
        },
        Commands::Snapshot { command } => {
            commands::snapshot::run(command)?;
        }
        Commands::Bun { command } => {
            commands::bun::run(command)?;
        }
        Commands::Version => {
            println!("chimera {}", env!("CARGO_PKG_VERSION"));
            println!("target: {}", std::env::consts::ARCH);
            println!("os: {}", std::env::consts::OS);
        }
    }

    Ok(())
}
