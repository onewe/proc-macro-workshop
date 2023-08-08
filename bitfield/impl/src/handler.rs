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
        let fields = &self.fields;
        let attrs = self.attrs;
        let generics = &self.generics;
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
       
        let const_size_expr = gen_const_size_expr(fields)?;
        let getter_fn_methods = gen_get_fn(fields)?;
        let setter_fn_methods = gen_set_fn(fields)?;
        let new_fn_method = gen_new_fn()?;
        
        
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
            
        };


        Ok(token_stream)
    }
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

                let mut ret_number = <#current_field_ty as bitfield::Specifier>::Type::MIN;

                while remain_bits > 0 {
                    let byte_mul = start_index / 8;
                    let byte_mod = start_index % 8;
            
                    let element = self.data[byte_mul];
            
                    let element = if byte_mod == 0 {
                        if remain_bits >= 8 {
                            remain_bits -= 8;
                            start_index += 8;
                            element
                        } else {
                            let element = element >> (8 - remain_bits);
                            remain_bits -= remain_bits;
                            start_index += remain_bits;
                            element
                        }
                       
                    } else {
                        let mut element = (element << byte_mod) >> byte_mod;

                        if remain_bits <= (8 - byte_mod) {
                            element = element >> (8 - remain_bits - byte_mod);
                            remain_bits -= remain_bits;
                            start_index += remain_bits;
                        } else {
                            remain_bits -= 8 - byte_mod;
                            start_index += 8 - byte_mod;
                        }
                        
                        element
                    };

                    let offset = 8 - element.leading_zeros();

                    ret_number = ret_number << offset;

                    ret_number = ret_number | element as <#current_field_ty as bitfield::Specifier>::Type;

                }
            
                ret_number
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


                while remain_bits > 0 {
                    let byte_mul = start_index / 8;
                    let byte_mod = start_index % 8;
            
                    let element = &mut self.data[byte_mul];
            
                    if byte_mod == 0 {
                        let require_bits = if remain_bits >= 8 {
                            8
                        } else {
                            remain_bits
                        };
                        let offset = remain_bits - require_bits;
                        *element = (*element) | (((arg >> offset) as u8) << (8 - require_bits));

                        let mask = !(<#current_field_ty as bitfield::Specifier>::Type::MAX << offset);
                        arg = arg & mask;
                        
                        start_index += require_bits;
                        remain_bits -= require_bits;
                    } else {

                        if remain_bits <= (8 - byte_mod) {
                            let offset = 8 - byte_mod - remain_bits;
                            *element = (*element) | (arg << offset) as u8;
                            start_index += remain_bits;
                            remain_bits -= remain_bits;
                        } else {
                            let require_bits = 8 - byte_mod;
                            let offset = remain_bits - require_bits;
                            *element = (*element) | (arg >> offset) as u8;

                            let mask = !(<#current_field_ty as bitfield::Specifier>::Type::MAX << offset);
                            arg = arg & mask;

                            start_index += require_bits;
                            remain_bits -= require_bits;
                        }

                    }

                }
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


fn gen_assert_const_expr() -> syn::Result<TokenStream> {

    let token_stream = TokenStream::default();

    let assert_multiple_of_8bits_fn = quote::quote! {
        const fn _assert_multiple_of_8bits_fn() {
            
        }
    };


    todo!()

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
                pub struct #ident;

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