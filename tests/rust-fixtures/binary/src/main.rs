//! Binary fixture that uses the basic library.
//!
//! This fixture tests that libraries can depend on other workspace
//! members and that the FFI boundary is respected.

use basic::add;

fn main() {
    let result = add(2, 3);
    println!("2 + 3 = {}", result);
    assert_eq!(result, 5);
}