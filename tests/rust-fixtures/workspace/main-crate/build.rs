fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=BUILD_SCRIPT_RAN=1");
}