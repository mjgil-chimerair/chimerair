// Workspace member: main.rs
// @verify chimera-rust-cli
// @expected ch_status SUCCESS

use test_workspace::utils;

fn main() {
    let result = utils::helper();
    println!("Result: {}", result);
}
