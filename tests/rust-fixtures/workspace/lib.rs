// Workspace member: lib.rs
// @verify chimera-rust-cargo
// @expected ch_status SUCCESS

pub mod utils {
    pub fn helper() -> u32 { 42 }
}

pub fn public_api() -> String {
    format!("version 0.1.0")
}
