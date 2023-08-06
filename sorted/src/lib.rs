use proc_macro::TokenStream;
use syn::parse_macro_input;

mod handler;

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
   let sorted_struct = parse_macro_input!(input as handler::SortedStruct);

   sorted_struct.to_token_stream().into()
}


#[proc_macro_attribute]
pub fn check(_args: TokenStream, input: TokenStream) -> TokenStream {
   let check_struct = parse_macro_input!(input as handler::CheckStruct);

   check_struct.to_token_stream().into()
}
