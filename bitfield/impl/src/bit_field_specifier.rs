use proc_macro2::{TokenStream, Span};
use syn::parse::{Parse, Parser};

pub struct BitfieldSpecifierGen {
    name: syn::Ident,
    generics: syn::Generics,
    variants: syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,

}

impl Parse for BitfieldSpecifierGen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item_enum = input.parse::<syn::ItemEnum>()?;
        let name = item_enum.ident;
        let generics = item_enum.generics;
        let variants = item_enum.variants;

        Ok(Self {
            name,
            generics,
            variants
        })
    }
}

impl BitfieldSpecifierGen {

    pub fn to_token_stream(self) -> syn::Result<TokenStream> {

        todo!()
    }
}

fn gen_total_bits_const_expr(variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>) -> syn::Result<TokenStream> {
    let mut token_stream = TokenStream::default();

    for variant in variants {
        let discriminant = variant.discriminant.as_ref();
        let expr = if discriminant.is_none() {
            quote::quote!{
                1
            }

        } else {
            let (_, expr) = discriminant.unwrap();
            quote::quote!{
                #expr
            }
        };

        if !token_stream.is_empty() {
            token_stream.extend(quote::quote! {
                +
            });
        }

        token_stream.extend(expr);
    }

   Ok(quote::quote! {
        const ENUM_TOTAL_BITS: usize = #token_stream;
   })
}


fn gen_impl_specifier_block(name: &syn::Ident, generics: &syn::Generics, variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>) -> syn::Result<TokenStream> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote::quote! {
        impl #impl_generics bitfield::Specifier for #name #ty_generics #where_clause {
            const BITS: usize = ENUM_TOTAL_BITS;
            type Type = #name;

            fn get_data(bits: &[u8], mut start_index: usize, mut remain_bits: usize) -> Self::Type {
                let raw_value = <<<Self as bitfield::BitfieldSpecifier>::BType as bitfield::BType>::Type as bitfield::Specifier>::get_data(bits, start_index, remain_bits);
                <Self as bitfield::BitfieldSpecifier>::from_raw_value(raw_value);
            }

            fn set_data(bits: &mut[u8], mut start_index: usize, mut remain_bits: usize, mut arg: Self::Type) {
                let raw_value = arg.raw_value();

            }

        }
    })
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