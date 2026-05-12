//! A simple procedural macro for testing.
//!
//! This fixture provides a basic proc-macro crate for testing
//! how chimera-rust-cargo handles proc-macro dependencies.

use proc_macro::TokenStream;

/// A macro that generates a function that returns its argument plus one.
///
/// # Example
///
/// ```rust
/// use proc_macro::TokenStream;
///
/// let input = TokenStream::from_tokens(tokens);
/// plus_one(input)
/// ```
#[proc_macro]
pub fn plus_one(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let num: i32 = input_str.trim().parse().unwrap();
    let result = num + 1;
    result.to_string().parse().unwrap()
}

/// A macro that generates a constant with the given value.
#[proc_macro]
pub fn make_const(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let num: i32 = input_str.trim().parse().unwrap();
    quote::quote! {
        const GENERATED_CONST: i32 = #num;
    }
    .into()
}