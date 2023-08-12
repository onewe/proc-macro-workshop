use proc_macro2::TokenStream;
use quote::{format_ident, spanned::Spanned};
use syn::parse::Parse;

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
        let fields = &self.fields;
        let attrs = self.attrs;
        let generics = &self.generics;
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
       
        let const_size_expr = gen_const_size_expr(fields)?;
        let getter_fn_methods = gen_get_fn(fields)?;
        let setter_fn_methods = gen_set_fn(fields)?;
        let new_fn_method = gen_new_fn()?;
        let assert_field_bits_total_expr = gen_assert_field_bits_total_expr(fields)?;
        
        
       let token_stream =  quote::quote! {
            #const_size_expr

            #(#attrs)*
            #visi struct #name #generics {
                data: [u8;MAX_LEN]
            }

            impl #impl_generics #name #ty_generics #where_clause {
                #new_fn_method
                #getter_fn_methods
                #setter_fn_methods
            }

            const _:() = {
                struct _AssertMod8 where <bitfield::checks::BitSizeMod<{TOTAL_SIZE % 8}> as bitfield::checks::AssertMod8>::CheckType: bitfield::checks::TotalSizeIsMultipleOfEightBits;
                #assert_field_bits_total_expr
            };
           
            
        };


        Ok(token_stream)
    }
}


fn gen_assert_field_bits_total_expr(fields: &syn::Fields) -> syn::Result<TokenStream> {
    
    let mut token_streams = TokenStream::default();

    for field in fields {
        let field_ident = field.ident.as_ref();
        let Some(field_ident) = field_ident else {
            continue;
        };

        let assert_ident = format_ident!("_assert_{}_bits", field_ident);
       

        let field_ty = &field.ty;
        let field_attrs = &field.attrs;

        for field_attr in field_attrs {
            
            let meta = &field_attr.meta;

            let syn::Meta::NameValue(meta) = meta else {
                continue;
            };

            let path = meta.path.get_ident(); 
            let Some(path) = path else {
                continue;
            };
            let ident = path.to_string();

            if ident.ne("bits") {
                continue;
            }

            let meta_value_expr = &meta.value;

            let span = meta_value_expr.__span();

            let syn::Expr::Lit(lit_expr) = meta_value_expr else {
                continue;
            };

            let syn::Lit::Int(int_lit) = &lit_expr.lit else {
                continue;
            };

            let bits_total = int_lit.base10_parse::<usize>()?;

            let token_stream = quote::quote_spanned!{span=>
                let #assert_ident: [u8;#bits_total] = [0u8;<#field_ty as bitfield::Specifier>::BITS];
            };
            token_streams.extend(token_stream);
            break;
        }
    }

    Ok(token_streams)
}

fn gen_const_size_expr(fields: &syn::Fields) -> syn::Result<TokenStream> {
    let mut token_streams: Vec<TokenStream> = Vec::default();

    let fields_len = fields.len();
    let fields_iter = fields.into_iter();

    for (idx, field) in fields_iter.enumerate() {
        let ty = &field.ty;
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
            const TOTAL_SIZE: usize = 0;
            const MAX_LEN: usize = 0;
        }
    } else {
        let calc_expr = TokenStream::from_iter(token_streams.into_iter());
        quote::quote! {
            const TOTAL_SIZE: usize = #calc_expr;
            const MAX_LEN: usize = (#calc_expr) / 8;
        }
    };

    Ok(token_stream)
}


fn gen_get_fn(fields: &syn::Fields) -> syn::Result<TokenStream> {
    let mut getter_fn_methods: Vec<TokenStream> = Vec::default();
    let mut current_field_lens = TokenStream::default();

    let mut previous_field_type: Option<&syn::Type> = None;

    for field in fields.iter() {
        let field_name = field.ident.as_ref();
        if field_name.is_none() {
            continue;
        }
        let field_name = field_name.unwrap();
        let field_name = format_ident!("get_{}", field_name);

        let current_field_ty = &field.ty;

        let const_start_index_expr =  if previous_field_type.is_none() {
            previous_field_type = Some(current_field_ty);
    
            quote::quote! {
                const BIT_START_INDEX: usize = 0;
            }
        } else {

            let unwrap_previous_field_type = previous_field_type.unwrap();
            previous_field_type = Some(current_field_ty);

            let current_field_len = if current_field_lens.is_empty() {
                quote::quote! {
                     <#unwrap_previous_field_type as bitfield::Specifier>::BITS
                }
            } else {
                quote::quote! {
                    +  <#unwrap_previous_field_type as bitfield::Specifier>::BITS
                }
            };

            current_field_lens.extend(current_field_len);

            quote::quote! {
                const BIT_START_INDEX: usize = #current_field_lens;
            }
        };

        let getter_method = quote::quote! {
            pub fn #field_name(&self) -> <#current_field_ty as bitfield::Specifier>::Type {
                #const_start_index_expr
                const BITS: usize = <#current_field_ty as bitfield::Specifier>::BITS;
               
                let mut start_index = BIT_START_INDEX;
                let mut remain_bits = BITS;

                <#current_field_ty as bitfield::Specifier>::get_data(&self.data, start_index, remain_bits)
            }
        };

        getter_fn_methods.push(getter_method);


    }

    Ok(TokenStream::from_iter(getter_fn_methods.into_iter()))
}


fn gen_set_fn(fields: &syn::Fields) -> syn::Result<TokenStream> {

    let mut setter_fn_methods: Vec<TokenStream> = Vec::default();
    let mut current_field_lens = TokenStream::default();

    let mut previous_field_type: Option<&syn::Type> = None;

    for field in fields.iter() { 
        let field_name = field.ident.as_ref();
        if field_name.is_none() {
            continue;
        }
        let field_name = field_name.unwrap();
        let field_name = format_ident!("set_{}", field_name);

        let current_field_ty = &field.ty;

        let const_start_index_expr =  if previous_field_type.is_none() {
            previous_field_type = Some(current_field_ty);
    
            quote::quote! {
                const BIT_START_INDEX: usize = 0;
            }
        } else {

            let unwrap_previous_field_type = previous_field_type.unwrap();
            previous_field_type = Some(current_field_ty);

            let current_field_len = if current_field_lens.is_empty() {
                quote::quote! {
                     <#unwrap_previous_field_type as bitfield::Specifier>::BITS
                }
            } else {
                quote::quote! {
                    +  <#unwrap_previous_field_type as bitfield::Specifier>::BITS
                }
            };

            current_field_lens.extend(current_field_len);

            quote::quote! {
                const BIT_START_INDEX: usize = #current_field_lens;
            }
        };

        let setter_method = quote::quote! {
            pub fn #field_name(&mut self, mut arg: <#current_field_ty as bitfield::Specifier>::Type){
                #const_start_index_expr
                const BITS: usize = <#current_field_ty as bitfield::Specifier>::BITS;
               
                let mut start_index = BIT_START_INDEX;
                let mut remain_bits = BITS;

                <#current_field_ty as bitfield::Specifier>::set_data(&mut self.data, start_index, remain_bits, arg)
                
            }
        };
        setter_fn_methods.push(setter_method);
    }

    Ok(TokenStream::from_iter(setter_fn_methods.into_iter()))
}

fn gen_new_fn() -> syn::Result<TokenStream> {
    let token_stream = quote::quote! {

        pub fn new() -> Self {
            Self {
                data: [0u8; MAX_LEN]
            }
        }
    };

    Ok(token_stream)
}