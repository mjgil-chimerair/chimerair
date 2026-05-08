//! Config parser binary
//!
//! Simple binary to demonstrate config parsing.

use chimera_config_parser::Config;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <config-file>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    match Config::parse(&content) {
        Ok(config) => {
            println!("Parsed {} entries:", config.len());
            for (key, value) in &config.entries {
                println!("  {} = {}", key, value);
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
}