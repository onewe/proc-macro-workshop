use std::process::id;

use proc_macro2::{TokenStream, Span};
use quote::format_ident;
use syn::{parse::{Parse, Parser}, Token};


pub struct BitField {
    visi: syn::Visibility,
    name: syn::Ident,
    fields: syn::Fields,
    generics: syn::Generics,
    attrs: Vec<syn::Attribute>
}


impl Parse for BitField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let struct_item = input.parse::<syn::ItemStruct>()?;
        let visi = struct_item.vis;
        let name = struct_item.ident;
        let fields = struct_item.fields;
        let generics: syn::Generics = struct_item.generics;
        let attrs = struct_item.attrs;
        
        Ok(Self { visi, name, fields, generics, attrs })
    }
}

impl BitField {

    pub fn to_token_stream(self) -> syn::Result<TokenStream> {
        let name  = self.name;
        let visi = self.visi;
        let fields = self.fields;
        let attrs = self.attrs;
        let generics = self.generics;

       
        let const_size_expr = gen_const_size_expr(fields)?;
        
        
       let token_stream =  quote::quote! {
            #const_size_expr

            #(#attrs)*
            #visi struct #name #generics {
                data: [u8;MAX_LEN]
            }
        };


        Ok(token_stream)
    }
}

fn gen_const_size_expr(fields: syn::Fields) -> syn::Result<TokenStream> {
    let mut token_streams: Vec<TokenStream> = Vec::default();

    let fields_len = fields.len();
    let fields_iter = fields.into_iter();

    for (idx, field) in fields_iter.enumerate() {
        let ty = field.ty;
        if idx == fields_len -1 {
            // the last
            let last_expr =  quote::quote! {
                <#ty as bitfield::Specifier>::BITS
            };
            token_streams.push(last_expr);
        } else {
           let left_expr =  quote::quote! {
                <#ty as bitfield::Specifier>::BITS +
            };
            token_streams.push(left_expr);
        }
    }

   let token_stream =  if token_streams.is_empty() {
        quote::quote! {
            const MAX_LEN: usize = 0;
        }
    } else {
        let calc_expr = TokenStream::from_iter(token_streams.into_iter());
        quote::quote! {
            const MAX_LEN: usize = (#calc_expr) / 8;
        }
    };

    Ok(token_stream)
}




pub struct BTypeGenerator {
    start: usize,
    end: usize
}


impl Parse for BTypeGenerator {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let start = input.parse::<syn::LitInt>()?;
        input.parse::<Token![..]>()?;
        let end = input.parse::<syn::LitInt>()?;
        let start = start.base10_parse::<usize>()?;
        let end = end.base10_parse::<usize>()?;
        Ok(Self { start, end })
    }
}

impl BTypeGenerator {

    pub fn to_token_stream(self) -> syn::Result<TokenStream> {
        let mut token_streams = Vec::default();
        let start = self.start;
        let end = self.end;

        for idx in start..end {
            let ident = format_ident!("B{}", idx);
            let data_type = gen_data_type(idx)?;
            let bits = proc_macro2::Literal::usize_unsuffixed(idx);
            let token_stream = quote::quote! {
                pub struct #ident {

                }

                impl Specifier for #ident {
                    const BITS: usize = #bits;
                    type Type = #data_type;
                }
            };

            token_streams.push(token_stream);

        }
        Ok(TokenStream::from_iter(token_streams.into_iter()))
    }
}

fn gen_data_type(bit_count: usize) -> syn::Result<syn::Type> {
    match bit_count {
        65..=128 => {
            Ok(syn::Type::parse.parse2(quote::quote![u128])?)
        }
        33..=64 => {
            Ok(syn::Type::parse.parse2(quote::quote![u64])?)
        }
        17..=32 => {
            Ok(syn::Type::parse.parse2(quote::quote![u32])?)
        }
        9..=16 => {
            Ok(syn::Type::parse.parse2(quote::quote![u16])?)
        }
        1..=8 => {
            Ok(syn::Type::parse.parse2(quote::quote![u8])?)
        }
        _ => {
            Err(syn::Error::new(Span::call_site(), "the bit count is error!"))
        }
    }
}