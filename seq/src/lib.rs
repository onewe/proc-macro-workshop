use proc_macro::TokenStream;
use syn::parse_macro_input;

mod handler;
#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let seq_struct = parse_macro_input!(input as handler::Seq);
    seq_struct.to_token_stream().unwrap_or_else(|e|e.into_compile_error()).into()
}
