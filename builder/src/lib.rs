use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, __private::{quote::{format_ident, quote}, TokenStream2}, Data, parse_quote, Field, parse::{Parser, Parse}, FieldValue, ItemFn, Type, Expr, Ident};

mod handler;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    let name = &derive_input.ident;
    let visi = &derive_input.vis;
    let fields: Vec<Field>; 
    let field_values: Vec<FieldValue>;
    let fns: Vec<ItemFn>;
    let builder_field_check: Vec<TokenStream2>;
    let builder_field_set_value: Vec<FieldValue>;
    if let Data::Struct(data) = &derive_input.data {
        fields = data.fields
        .iter()
        .filter(|f|f.ident.is_some())
        .map(|f| {
            let name = f.ident.as_ref().unwrap();
            let vis = &f.vis;
            let ty = &f.ty;

            let is_option = if let Type::Path(path) = ty {
                path.path.segments.first().map(|p|p.ident.eq("Option")).unwrap_or(false)
            } else {
                false
            };

            let field: Field;
            if is_option {
                field = Field::parse_named.parse2(quote!(
                    #vis #name: #ty
                )).unwrap();
            } else {
                field = Field::parse_named.parse2(quote!(
                    #vis #name: Option<#ty>
                )).unwrap();
            }

            field

        })
        .collect();

        field_values = data.fields
            .iter()
            .filter(|f|f.ident.is_some())
            .map(|f|{
                let name = f.ident.as_ref().unwrap();
                let field_value = parse_quote! {
                    #name : None
                };

                field_value
            }).collect();

        fns = data.fields
            .iter()
            .filter(|f|f.ident.is_some())
            .map(|f| {
                let name = f.ident.as_ref().unwrap();
                let ty = &f.ty;
                let attrs = &f.attrs;

                let mut item_fns = Vec::<ItemFn>::new();
                let item_fn: ItemFn;

                if !attrs.is_empty() {
                    let is_vec = if let Type::Path(path) = ty {
                        path.path.segments.first().map(|p|p.ident.eq("Vec")).unwrap_or(false)
                    } else {
                        false
                    };
    
                    let inner_type = if is_vec {
                        if let Type::Path(path) = ty {
                            let ps  = path.path.segments.first().unwrap();
                            if let syn::PathArguments::AngleBracketed(ab) = &ps.arguments {
                                let ga = ab.args.first().unwrap();
                                if let syn::GenericArgument::Type(gty) = ga {
    
                                   Some(gty)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
    
                        } else {
                            None
                        }
                        
                    } else {
                        None
                    };
    
                    if let Some(gtp) = inner_type {
                        let att = attrs.first().unwrap();
                        let meta = &att.meta;
                        if let syn::Meta::List(nv) = meta {
                            
                          let exp = Expr::parse.parse2(nv.tokens.clone()).unwrap();
                          if let syn::Expr::Assign(asi) = exp {
                           
                            if let syn::Expr::Lit(li) = asi.right.as_ref() {
                                if let syn::Lit::Str(li_str) = &li.lit {
                                   let id  = li_str.parse::<Ident>().unwrap();
                                   let gtp_fn: ItemFn = parse_quote!{
                                        fn #id(&mut self, #id: #gtp) -> &mut Self {
                                            let mut container = if self.#name.is_none() {
                                                 Vec::default()
                                            } else {
                                                self.#name.take().unwrap()
                                            };
                                            
                                            container.push(#id);
                                            self.#name = Some(container);
                                            self
                                        }
    
                                   };
    
                                   item_fns.push(gtp_fn);
                                }
                            }
                          }
                          
                        }
    
                    }

                } else {

                    let is_option = if let Type::Path(path) = ty {
                        path.path.segments.first().map(|p|p.ident.eq("Option")).unwrap_or(false)
                    } else {
                        false
                    };

                

                    if is_option {

                        if let Type::Path(path) = ty {
                            let ps  = path.path.segments.first().unwrap();
                            if let syn::PathArguments::AngleBracketed(ab) = &ps.arguments {
                                let ga = ab.args.first().unwrap();
                                if let syn::GenericArgument::Type(gty) = ga {
                                    item_fn = parse_quote! {
                                        fn #name(&mut self, #name: #gty) -> &mut Self {
                                            self.#name = Some(#name);
                                            self
                                        }
                                    };
                                } else {
                                    item_fn = parse_quote! {
                                        fn #name(&mut self, #name: #ty) -> &mut Self {
                                            self.#name = Some(#name);
                                            self
                                        }
                                    };
                                }
                            } else {
                                item_fn = parse_quote! {
                                    fn #name(&mut self, #name: #ty) -> &mut Self {
                                        self.#name = Some(#name);
                                        self
                                    }
                                };
                            }

                        } else {
                            item_fn = parse_quote! {
                                fn #name(&mut self, #name: #ty) -> &mut Self {
                                    self.#name = Some(#name);
                                    self
                                }
                            };
                        }

                    } else {
                        item_fn = parse_quote! {
                            fn #name(&mut self, #name: #ty) -> &mut Self {
                                self.#name = Some(#name);
                                self
                            }
                        };
                    }
                    item_fns.push(item_fn);
                }

                item_fns
            }).flatten().collect();

    
        
        builder_field_check = data.fields
            .iter()
            .filter(|f|f.ident.is_some())
            .map(|f| {
                let name = f.ident.as_ref().unwrap();
                let ty = &f.ty;
                let attrs = &f.attrs;

                let is_option = if let Type::Path(path) = ty {
                    path.path.segments.first().map(|p|p.ident.eq("Option")).unwrap_or(false)
                } else {
                    false
                };

                
                
                let token: TokenStream2;
                if is_option {
                    token = quote! {
                        let #name = self.#name.take();
                    };

                } else {
                    
                    if attrs.is_empty() {
                        token = quote! {
                            if self.#name.is_none() {
                                let err_msg = format!("{} field missing", stringify!(#name));
                                return Err(Box::<dyn std::error::Error>::from(err_msg.to_string()));
                            }
        
                            let #name = self.#name.take().unwrap();
                        };
                    } else {
                        token = quote! {
                            let #name = if self.#name.is_none() {
                                Default::default()
                            } else {
                                self.#name.take().unwrap()
                            };
                        };
                    }

                    
                }

                token
            }).collect();

        builder_field_set_value = data.fields
            .iter()
            .filter(|f|f.ident.is_some())
            .map(|f| {
                let name = f.ident.as_ref().unwrap();
                let field_value = parse_quote! {
                    #name
                };
                field_value
            }).collect();

    } else {
        return TokenStream::default();
    }

    let builder_name = format_ident!("{}Builder", name);

    let tokens = quote! {
        impl #name {

            pub fn builder() -> #builder_name {
                #builder_name {
                   #(#field_values),*
                }
            }

        }

        #visi struct #builder_name {
            #(#fields),*
        }

      
        impl #builder_name {

            #(#fns)*

          
            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                #(#builder_field_check)*

                let ret = #name {
                    #(#builder_field_set_value),*
                };

                Ok(ret)
            }
        
        }

    };

    tokens.into()
    
}


  // pub fn build(&mut self) -> Result<#name, Box<dyn Error>> {
            //     #(#builder_field_check)*
            //     #name {
            //         #(#builder_field_set_value),*
            //     }
            // }