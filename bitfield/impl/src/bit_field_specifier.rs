use proc_macro2::{TokenStream, Span};
use quote::{spanned::Spanned, format_ident};
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

        for variant in &variants {
            let fields = &variant.fields;
            match fields {
                syn::Fields::Unit => {},
                _ => {
                    return Err(syn::Error::new(variant.__span(), "only support Unit Variant Type"));
                } 
            }
        }

        Ok(Self {
            name,
            generics,
            variants
        })
    }
}

impl BitfieldSpecifierGen {

    pub fn to_token_stream(self) -> syn::Result<TokenStream> {
        let name = &self.name;
        let variants = &self.variants;
        let generics = &self.generics;

        let len = variants.len();

        if !len.is_power_of_two() {
            return Err(syn::Error::new(Span::call_site(), "BitfieldSpecifier expected a number of variants which is a power of 2"));
        }

        let total_bits_const_expr = gen_total_bits_const_expr(variants)?;

        let impl_specifier_block = gen_impl_specifier_block(name, generics)?;

        let impl_bit_field_specifier_block = gen_impl_bit_field_specifier_block(name, generics, variants)?;
        let assert_discriminant_in_range_block = gen_assert_discriminant_in_range_block(name, variants)?;

        Ok(quote::quote! {
            const _:() = {
                #total_bits_const_expr

                #impl_specifier_block

                #impl_bit_field_specifier_block

                #assert_discriminant_in_range_block
            };
        })
    }
}

fn gen_total_bits_const_expr( variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>) -> syn::Result<TokenStream> {
    let count = variants.iter().count() -1;
    let bits = (usize::BITS - count.leading_zeros()) as usize;
    let bits = proc_macro2::Literal::usize_unsuffixed(bits);

   Ok(quote::quote! {
        const ENUM_TOTAL_BITS: usize = #bits;
        const POW_ENUM_TOTAL_BITS: usize = 2usize.pow(ENUM_TOTAL_BITS as u32);
   })
}


fn gen_assert_discriminant_in_range_block(name: &syn::Ident, variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>) -> syn::Result<TokenStream> {
    let mut token_streams = TokenStream::default();
    for variant in variants {
        let span = variant.__span();
        let variant_name = &variant.ident;

        let assert_ident = format_ident!("_AssertDiscriminantInRangeFor{}", variant_name);

        let variant_path = syn::Path::parse.parse2(quote::quote!{
            #name::#variant_name
        })?;

        let assert_token_stream = quote::quote_spanned! {span=>
            struct #assert_ident where <bitfield::checks::AssertDiscriminantInRange<{(#variant_path as usize) < POW_ENUM_TOTAL_BITS}> as bitfield::checks::IFDiscriminantInRange>::Type: bitfield::checks::DiscriminantInRange;
        };
        token_streams.extend(assert_token_stream);
    }
    Ok(token_streams)
}


fn gen_impl_specifier_block(name: &syn::Ident, generics: &syn::Generics) -> syn::Result<TokenStream> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

   let mut where_clause = if where_clause.is_none() {
        syn::WhereClause::parse.parse2(quote::quote!(where))?
   } else {
        where_clause.unwrap().clone()
   };

   let where_predicate = syn::WherePredicate::parse.parse2(quote::quote!{
        Self: bitfield::BitfieldSpecifier
   })?;

   where_clause.predicates.push(where_predicate);

   

    Ok(quote::quote! {
        impl #impl_generics bitfield::Specifier for #name #ty_generics #where_clause {
            const BITS: usize = ENUM_TOTAL_BITS;
            type Type = #name;

            fn get_data(bits: &[u8], mut start_index: usize, mut remain_bits: usize) -> Self::Type {
                let raw_value = <<<Self as bitfield::BitfieldSpecifier>::BType as bitfield::BType>::Type as bitfield::Specifier>::get_data(bits, start_index, remain_bits);
                <Self as bitfield::BitfieldSpecifier>::from_raw_value(raw_value)
            }

            fn set_data(bits: &mut[u8], mut start_index: usize, mut remain_bits: usize, mut arg: Self::Type) {
                let raw_value = arg.raw_value();
                <<<Self as bitfield::BitfieldSpecifier>::BType as bitfield::BType>::Type as bitfield::Specifier>::set_data(bits, start_index, remain_bits, raw_value);
            }

        }
    })

}

fn gen_impl_bit_field_specifier_block(name: &syn::Ident, generics: &syn::Generics, variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,) -> syn::Result<TokenStream> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    
    let mut variant_names = Vec::default();
    for variant in variants {
        let variant_name = &variant.ident;
        let variant_name = syn::Path::parse.parse2(quote::quote!{
            #name::#variant_name
        })?;
        variant_names.push(variant_name);
    }

   

    let mut from_raw_value_match_arms = TokenStream::default();
    let mut raw_value_match_arms = TokenStream::default();

    for variant_name in variant_names.iter() {

        let from_raw_value_match_arm = quote::quote! {
            value if value == (#variant_name as <<Self::BType as BType>::Type as Specifier>::Type) => #variant_name,
        };
        from_raw_value_match_arms.extend(from_raw_value_match_arm);

        let raw_value_match_arm = quote::quote! {
            #variant_name => (#variant_name as <<Self::BType as BType>::Type as Specifier>::Type),
        };
        raw_value_match_arms.extend(raw_value_match_arm);
    }

    Ok(quote::quote! {

        impl #impl_generics bitfield::BitfieldSpecifier for #name #ty_generics #where_clause {
            type BType = Bits<{ENUM_TOTAL_BITS}>;

            fn raw_value(&self) -> <<Self::BType as BType>::Type as Specifier>::Type {
                match self {
                    #raw_value_match_arms
                }
            }

            fn from_raw_value(value: <<Self::BType as BType>::Type as Specifier>::Type) -> Self {
                match value {
                    #from_raw_value_match_arms
                    _ => panic!("not support this value!")
                }
            }
        }
    })
}