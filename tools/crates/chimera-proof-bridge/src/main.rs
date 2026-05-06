use chimera_proof_bridge::{extract_proof_obligations, verify_proof_sidecar};
use std::path::PathBuf;

fn print_usage() {
    eprintln!("usage: chimera-proof-bridge <command> [args]");
    eprintln!("  verify <proof-sidecar>      Verify a proof sidecar file");
    eprintln!(
        "  extract <artifacts-dir> <component-id>   Extract proof obligations from build artifacts"
    );
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        std::process::exit(2);
    };

    match command.as_str() {
        "verify" => {
            let Some(path) = args.next() else {
                print_usage();
                std::process::exit(2);
            };
            if args.next().is_some() {
                print_usage();
                std::process::exit(2);
            }

            let input = PathBuf::from(path);
            match verify_proof_sidecar(&input) {
                Ok(verification) => {
                    println!(
                        "verified {} obligations={} trust_assumptions={}",
                        input.display(),
                        verification.verified_obligations,
                        verification.trust_assumptions
                    );
                }
                Err(error) => {
                    eprintln!(
                        "proof verification failed for {}: {}",
                        input.display(),
                        error
                    );
                    std::process::exit(1);
                }
            }
        }
        "extract" => {
            let Some(artifacts_dir) = args.next() else {
                print_usage();
                std::process::exit(2);
            };
            let Some(component_id) = args.next() else {
                print_usage();
                std::process::exit(2);
            };
            if args.next().is_some() {
                print_usage();
                std::process::exit(2);
            }

            let dir = PathBuf::from(artifacts_dir);
            match extract_proof_obligations(&dir, &component_id) {
                Ok(extraction) => {
                    println!(
                        "extracted {} obligations={} trust_assumptions={}",
                        component_id, extraction.verified_obligations, extraction.trust_assumptions
                    );
                }
                Err(error) => {
                    eprintln!("proof extraction failed: {}", error);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            print_usage();
            std::process::exit(2);
        }
    }
}
