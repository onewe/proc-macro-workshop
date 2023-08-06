use handler::FieldStream;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, __private::quote::{format_ident, quote}, Data};

mod handler;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    let name = &derive_input.ident;
    let visi = &derive_input.vis;
    let fields = match &derive_input.data {
        Data::Struct(data) => {
            &data.fields
        },
        _ => {
            return syn::Error::new(name.span(), "the field attribute format error, expect: each = ...").to_compile_error().into();
        }
    };

    let field_stream = FieldStream::new(fields);

    let default_values = handler::gen_builder_field_default_value(&field_stream);
    if let Err(e) = default_values {
        return e.to_compile_error().into();
    }

    let default_values = default_values.unwrap();

    let builder_fields = handler::gen_builder_field(&field_stream);
    if let Err(e) = builder_fields {
        return e.to_compile_error().into();
    }

    let builder_fields = builder_fields.unwrap();

    let builder_field_methods = handler::gen_field_method(&field_stream);

    if let Err(e) = builder_field_methods {
        return e.to_compile_error().into();
    }

    let builder_field_methods = builder_field_methods.unwrap();

   

    let builder_name = format_ident!("{}Builder", name);

    let builder_method = handler::gen_builder_method(name, &field_stream);
    if let Err(e) = builder_method {
        return e.to_compile_error().into();
    }

    let builder_method = builder_method.unwrap();

    let tokens = quote! {
        impl #name {

            pub fn builder() -> #builder_name {
                #builder_name {
                   #(#default_values),*
                }
            }

        }

        #visi struct #builder_name {
            #(#builder_fields),*
        }

           impl #builder_name {
    
                #(#builder_field_methods)*
    
              
               #builder_method
            
            }

        

    };

    tokens.into()
    
}