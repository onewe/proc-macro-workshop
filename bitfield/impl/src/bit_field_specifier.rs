use std::cmp::Ordering;

use proc_macro2::TokenStream;
use quote::spanned::Spanned;
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

        let total_bits_const_expr = gen_total_bits_const_expr(variants)?;

        let impl_specifier_block = gen_impl_specifier_block(name, generics)?;

        let impl_bit_field_specifier_block = gen_impl_bit_field_specifier_block(name, generics, variants)?;

        Ok(quote::quote! {
            const _:() = {
                #total_bits_const_expr

                #impl_specifier_block

                #impl_bit_field_specifier_block
            };
        })
    }
}

fn gen_total_bits_const_expr(variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>) -> syn::Result<TokenStream> {
    

    let count = variants.iter().count() -1;
    let bits = (usize::BITS - count.leading_zeros()) as usize;
    let bits = proc_macro2::Literal::usize_unsuffixed(bits);
   Ok(quote::quote! {
        const ENUM_TOTAL_BITS: usize = #bits;
   })
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
    

    let mut variants: Vec<_> =  variants.iter().enumerate().map(|(idx, variant)| {
        let variant_name = &variant.ident;
        let variant_name = syn::Path::parse.parse2(quote::quote!{
            #name::#variant_name
        }).unwrap();

        let discriminant = variant.discriminant.as_ref();
        let order = if discriminant.is_none() {
            None 
        } else {
            let (_, expr) = discriminant.unwrap();
            let syn::Expr::Lit(lit_expr) = expr else {
                return (idx, None, variant_name);
            };

            let syn::Lit::Int(int_lit) = &lit_expr.lit else {
                return (idx, None, variant_name);
            };

            let order = int_lit.base10_parse::<usize>();
            if order.is_err() {
                return (idx, None, variant_name);
            }
            let order = order.unwrap();
            Some(order)
        };

        (idx, order, variant_name)
    }).collect();

    variants.sort_by(|(a_index, a_order, _),(b_index, b_order, _)| {
        if a_order.is_some() && b_order.is_some() {
            return a_order.unwrap().cmp(&b_order.unwrap())
        }

        if a_order.is_some() && b_order.is_none() {
            let a_order = a_order.unwrap();
            if a_order == *b_index {
                return Ordering::Less;
            }
            return a_order.cmp(b_index);
        }

        if a_order.is_none() && b_order.is_some() {
            let b_order = b_order.unwrap();
            if b_order == *a_index {
                return Ordering::Less;
            }
            return a_index.cmp(&b_order);
        }

        return a_index.cmp(b_index)
    });

    let variants: Vec<_> = variants.into_iter().map(|(_, _, ident)|ident).collect();

    let mut raw_value_match_arms = TokenStream::default();
    let mut from_raw_value_match_arms = TokenStream::default();

    for (idx, variant) in variants.into_iter().enumerate() {

        let idx = proc_macro2::Literal::usize_unsuffixed(idx);
        let raw_value_match_arm = quote::quote! {
            #variant => #idx,
        };
        raw_value_match_arms.extend(raw_value_match_arm);


        let from_raw_value_match_arm = quote::quote! {
            #idx => #variant,
        };
        from_raw_value_match_arms.extend(from_raw_value_match_arm);
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