use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};


mod handler;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let mut derive_input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let token = handler::impl_debug(&mut derive_input);

    if let Err(e) = token {
        return e.into_compile_error().into();
    }

    token.unwrap().into()
}
