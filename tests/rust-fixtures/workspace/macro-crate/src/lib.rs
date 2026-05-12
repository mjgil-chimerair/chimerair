//! Macro crate for workspace fixture - exports a simple attribute macro.

use proc_macro::TokenStream;
use quote::quote;
use syn;

/// A simple attribute macro that wraps a function in a panic handler.
#[proc_macro_attribute]
pub fn with_panic_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn: syn::ItemFn = syn::parse_macro_input!(item);
    let fn_ident = &input_fn.sig.ident;
    let fn_block = &input_fn.block;

    quote! {
        fn #fn_ident() #fn_block
    }
    .into()
}

/// A procedural macro that generates a wrapper struct.
#[proc_macro]
pub fn make_wrapper(type_name: TokenStream) -> TokenStream {
    let name: syn::Ident = syn::parse_macro_input!(type_name);
    quote! {
        struct Wrapper {
            inner: #name,
            called: bool,
        }

        impl Wrapper {
            pub fn new(inner: #name) -> Self {
                Wrapper { inner, called: false }
            }

            pub fn call(&mut self) -> &mut #name {
                self.called = true;
                &mut self.inner
            }

            pub fn was_called(&self) -> bool {
                self.called
            }
        }
    }
    .into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_macro_compiles() {
        assert!(true);
    }
}