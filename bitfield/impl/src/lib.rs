use handler::BTypeGenerator;
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod handler;

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let bit_field = parse_macro_input!(input as handler::BitField);

    bit_field.to_token_stream().unwrap_or_else(|e|e.into_compile_error()).into()
}


#[proc_macro]
pub fn gen_bit_type(input: TokenStream)  -> TokenStream {
    
    let b_type_generator = parse_macro_input!(input as BTypeGenerator);

    b_type_generator.to_token_stream().unwrap_or_else(|e|e.to_compile_error()).into()
}