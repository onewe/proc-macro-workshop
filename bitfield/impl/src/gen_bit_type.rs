use proc_macro2::{TokenStream, Span};
use quote::format_ident;
use syn::{parse::{Parse, Parser}, Token};


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

                impl BType for Bits<{#bits}> {
                    type Type = #ident;
                }

                impl Specifier for #ident {
                    const BITS: usize = #bits;
                    type Type = #data_type;

                    fn get_data(bits: &[u8], mut start_index: usize, mut remain_bits: usize) -> Self::Type {

                        let mut ret_number = Self::Type::MIN;
                        let mut offset = 0;
                        while remain_bits > 0 {
                            let byte_mul = start_index / 8;
                            let byte_mod = start_index % 8;
                    
                            let element = bits[byte_mul];

                       
                    
                            let element = if byte_mod == 0 {
                                if remain_bits >= 8 {
                                    offset = 8;
                                    remain_bits -= 8;
                                    start_index += 8;
                                    element
                                } else {
                                    let element = element >> (8 - remain_bits);
                                    offset = remain_bits;
                                    remain_bits -= remain_bits;
                                    start_index += remain_bits;
                                    element
                                }
                               
                            } else {
                                let mut element = (element << byte_mod) >> byte_mod;
        
                                if remain_bits <= (8 - byte_mod) {
                                    element = element >> (8 - remain_bits - byte_mod);
                                    offset = remain_bits;
                                    remain_bits -= remain_bits;
                                    start_index += remain_bits;
                                } else {
                                    offset = 8;
                                    remain_bits -= 8 - byte_mod;
                                    start_index += 8 - byte_mod;
                                }
                                
                                element
                            };
        
                            ret_number = ret_number << offset;
        
                            ret_number = ret_number | element as Self::Type;
        
                        }
                        ret_number
                    }
                    
                    fn set_data(bits: &mut[u8], mut start_index: usize, mut remain_bits: usize, mut arg: Self::Type) {
                    
                        while remain_bits > 0 {
                            let byte_mul = start_index / 8;
                            let byte_mod = start_index % 8;
                    
                            let element = &mut bits[byte_mul];
                    
                            if byte_mod == 0 {
                                let require_bits = if remain_bits >= 8 {
                                    8
                                } else {
                                    remain_bits
                                };
                                let offset = remain_bits - require_bits;
                                *element = (*element) | (((arg >> offset) as u8) << (8 - require_bits));

                                let mask = !(Self::Type::MAX << offset);
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
                                    
        
                                    let mask = !(Self::Type::MAX << offset);
                                    arg = arg & mask;
        
                                    start_index += require_bits;
                                    remain_bits -= require_bits;
                                }
        
                            }
        
                        }
                    }
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