use syn::{__private::{TokenStream2, quote::quote, ToTokens}, DeriveInput, LitStr, Fields, punctuated::Iter, Field, Ident, Type, parse::{Parse, Parser}, Data, parse_quote};

pub fn impl_debug(input: &DeriveInput) -> Result<TokenStream2, syn::Error> {
    let name  = &input.ident;
    let generics = &input.generics;
    let struct_attrs: &Vec<syn::Attribute> = &input.attrs;
    

    let fields = match &input.data {
        Data::Struct(data) => {
            &data.fields
        },
        _ => {
            return Err(syn::Error::new(name.span(), "the field attribute format error"));
        }
    };

    let field_stream = FieldStream::new(fields);

    let name_str = name.to_string();
    let lit_str = LitStr::new(name_str.as_str(), name.span());

    let exp_lets = gen_debug_field_exp(&field_stream)?;

    let params: Vec<&syn::TypeParam> = generics.type_params().collect();
    let gen_where_clause = gen_where_clause(struct_attrs, params, &field_stream)?;
    let (_, ty_generics, _) = generics.split_for_impl();
    
    let token = quote! {
        impl #ty_generics  std::fmt::Debug for #name #ty_generics #gen_where_clause {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut debug_struct = fmt.debug_struct(#lit_str);
                #(#exp_lets;)*
                debug_struct.finish()
            }
        }
    };

   Ok(token)
}

fn gen_where_clause(struct_attrs: &Vec<syn::Attribute>, type_params: Vec<&syn::TypeParam>, field_stream: &FieldStream) -> Result<syn::WhereClause, syn::Error>  {

    let mut where_clause: syn::WhereClause = parse_quote! {
        where
    };

    let where_predicates = gen_where_predicates(type_params, field_stream, struct_attrs)?;

    for where_predicate in where_predicates {
        where_clause.predicates.push(where_predicate);
    }

    // let where_predicates = gen_where_predicates_for_attr(struct_attrs)?;

    // for where_predicate in where_predicates {
    //     where_clause.predicates.push(where_predicate);
    // } 

    Ok(where_clause)
}

fn gen_where_predicates(type_params: Vec<&syn::TypeParam>, field_stream: &FieldStream, struct_attrs: &Vec<syn::Attribute>) -> Result<Vec<syn::WherePredicate>, syn::Error> {
    
    let type_param_str: Vec<String> =  type_params.iter().map(|t|t.ident.to_string()).collect();

    let mut type_vec: Vec<String> = Vec::default();
    let mut vec: Vec<syn::WherePredicate> = Vec::default();


    for type_param in type_params.iter() {
        let is_empty = type_param.bounds.is_empty();
        if is_empty {
          continue;  
        }

        let where_predicate: syn::WherePredicate = parse_quote! {
            #type_param
        };

        vec.push(where_predicate);
    }

    let where_predicates_for_attr = gen_where_predicates_for_attr(struct_attrs)?;

    if !where_predicates_for_attr.is_empty() {
        vec.extend(where_predicates_for_attr);
        return Ok(vec);
    }

    for field in field_stream.iter() {
        let field = field?;
        let ty = field.ty;

        let where_predicates = gen_where_predicates_for_every_field(&type_param_str, ty);
        if where_predicates.is_none() {
            continue;
        }
        
        let where_predicates = where_predicates.unwrap();

        if where_predicates.is_empty() {
            continue;
        }

        for (where_predicate, type_string) in where_predicates {
            if !type_vec.contains(&type_string) {
                vec.push(where_predicate);
                type_vec.push(type_string);
            }
        }
    }

   

    Ok(vec)
}



fn gen_where_predicates_for_attr(struct_attrs: &Vec<syn::Attribute>) -> Result<Vec<syn::WherePredicate>, syn::Error>{
    let mut where_predicates: Vec<syn::WherePredicate>= Vec::default();
    for attr in struct_attrs {
        let meta = &attr.meta;
        let list = meta.require_list()?;
        let tokens = &list.tokens;
        let exp_assign: syn::ExprAssign = parse_quote! {
            #tokens
        };
        
        let left = &exp_assign.left;

        let left_ident: Ident = parse_quote! {
            #left
        };

        let left_str = left_ident.to_string();
        if !left_str.eq("bound") {
            continue;
        }

        let right = &exp_assign.right;
        let right_str: LitStr = parse_quote! {
            #right
        };
        
        let token = right_str.parse().unwrap();
        let where_predicate = syn::WherePredicate::parse.parse2(token)?;

        where_predicates.push(where_predicate);
    }

    Ok(where_predicates)
}


fn gen_where_predicates_for_every_field(type_param: &Vec<String>, ty: &Type) -> Option<Vec<(syn::WherePredicate, String)>> {
    let mut predicates: Vec<(syn::WherePredicate, String)> = Vec::default();
    match ty {
        syn::Type::Path(path) => {
            let path = &path.path;
            let ident = path.get_ident();
            
            if ident.is_none() {
                let inner_types = get_inner_types(ty);
                if inner_types.is_none() {
                    return None;
                }
                let inner_types = inner_types.unwrap();
                if inner_types.is_empty() {
                    let (is_associated_type, type_str) = is_associated_type(type_param, ty);
                    if is_associated_type {
                        let where_predicate: syn::WherePredicate = parse_quote! {
                            #ty: std::fmt::Debug
                        };
                        predicates.push((where_predicate, type_str));
                        return Some(predicates);
                    }
                    return None;
                }

                
                let ph_data_field = ty_eq(ty, "PhantomData");
                if  ph_data_field {
                    let inner_type = inner_types.first().unwrap();
                    let inner_type_str = type_to_ident_str(inner_type);
                    if  inner_type_str.is_none(){
                        return None;
                    }
                    let inner_type_str = inner_type_str.unwrap();
                    let contain = type_param.contains(&inner_type_str);
                    if contain {
                        let where_predicate: syn::WherePredicate = parse_quote! {
                            std::marker::PhantomData<#inner_type>: std::fmt::Debug
                        };
                        predicates.push((where_predicate, inner_type_str));
                        return Some(predicates);
                    } else {
                        return None;
                    }
                }

                for inner_type in inner_types {
                    let ret = gen_where_predicates_for_every_field(type_param, inner_type);
                    if ret.is_none() {
                        continue;
                    }
                    let ret = ret.unwrap();
                    predicates.extend(ret);

                }
                return Some(predicates);
            }
           
            let ident = ident.unwrap();
            let ident_str = ident.to_string();
            let contain = type_param.contains(&ident_str);
            if contain {
                let where_predicate: syn::WherePredicate = parse_quote! {
                    #ty: std::fmt::Debug
                };
                predicates.push((where_predicate, ident_str));
                return Some(predicates);
            } else {
                let (is_associated_type, _) = is_associated_type(type_param, ty);
                if is_associated_type {
                    let where_predicate: syn::WherePredicate = parse_quote! {
                        #ty: std::fmt::Debug
                    };
                    predicates.push((where_predicate, ident_str));
                    return Some(predicates);
                }
               return None
            }

        },
        _ => return None
    }

}


fn type_to_ident_str(ty: &Type) -> Option<String> {
    match ty {
        syn::Type::Path(path) => {
            let path = &path.path;
            let ident = path.get_ident();
            if ident.is_none() {
                return None;
            }
            let ident = ident.unwrap();
            let ident_str = ident.to_string();
            return Some(ident_str);
        },
        _ => None
    }
}

fn gen_debug_field_exp(field_stream: &FieldStream) -> Result<Vec<syn::ExprLet>, syn::Error> {

    let mut exp_lets:Vec<syn::ExprLet> = Vec::default();

    for field in field_stream.iter() {
        let field = field?;
        let name = field.name;
        if name.is_none() {
            continue;
        }
        let name = name.unwrap();
        let name_lit_str = LitStr::new(name.to_string().as_str(), name.span());

       let attrs = field.attrs;
       if attrs.is_empty() {
            let expr_let: syn::ExprLet = parse_quote! {
                let mut debug_struct = debug_struct.field(#name_lit_str, &self.#name)
            };

            exp_lets.push(expr_let);
       } else {
        for attr in attrs.iter() {
            let attr_name = &attr.name;

            if attr_name.eq("debug") {
                let value = &attr.value;
                let format_str = LitStr::new(&value, name.span());
                let expr_let: syn::ExprLet = parse_quote! {
                    let mut debug_struct = debug_struct.field(#name_lit_str, &std::format_args!(#format_str, &self.#name))
                };
                exp_lets.push(expr_let);
            }
        }
       }
        

    }
    Ok(exp_lets)
}


pub struct FieldInfo<'a> {
    name: Option<&'a Ident>,
    ty: &'a Type,
    attrs: Vec<FieldAttr>,
}

#[derive(Debug)]
pub struct FieldAttr {
    name: String,
    value: String,
}

impl<'a> FieldInfo<'a> {

    pub fn new(field: &'a Field) -> Result<Self, syn::Error> {
        let name: Option<&Ident> = field.ident.as_ref();
        let ty: &Type = &field.ty;
        let attrs = &field.attrs;

        let mut field_attrs = Vec::<FieldAttr>::new();

        for attr in attrs {
            let meta = &attr.meta;
            let meta_and_value = meta.require_name_value()?;
            let left_path = &meta_and_value.path;
            let left_ident  = left_path.get_ident();
            if left_ident.is_none() {
                continue;
            }
            let left_ident = left_ident.unwrap();
            let left = left_ident.to_string();
            

            let right_value = &meta_and_value.value;
            let right_token = right_value.into_token_stream();
            let right_lit = <syn::LitStr as Parse>::parse.parse2(right_token)?;
            let right = right_lit.value();

            let field_attr = FieldAttr {
                name: left,
                value: right
            };
            field_attrs.push(field_attr);
        }

        


       Ok(Self {
        name,
        ty,
        attrs: field_attrs
       })
    }

}

fn ty_eq(ty: &Type, other: &str) -> bool {
    if let Type::Path(path) = ty {
        path.path.segments.first().map(|p|p.ident.eq(other)).unwrap_or(false)
    } else {
        false
    }
}

pub struct FieldStream<'a> {
    fields: &'a Fields,
}

impl<'a> FieldStream<'a> {

    pub fn new(fields: &'a Fields) -> Self {
        Self {
            fields
        }
    }

    pub fn iter(&'a self) -> FieldIter<'a> {
        let iter = self.fields.iter();
        FieldIter {
            inner: iter
        }
    }
}


pub struct FieldIter<'a> {
    inner: Iter<'a, Field>
}

impl<'a> Iterator for FieldIter<'a> {
    type Item = Result<FieldInfo<'a>, syn::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|f|FieldInfo::new(f))
    }
}


fn get_inner_types(ty: &Type) -> Option<Vec<&Type>> {
    let mut inner_types: Vec<&Type> = Vec::default();
    match ty {
        Type::Path(path) => {
            let path = &path.path;
            let segments = &path.segments;
            if segments.is_empty() {
                return None;
            }
            
            for segment in segments {
                let arguments = &segment.arguments;
                if arguments.is_none() {
                    continue;
                }
                match arguments {
                    syn::PathArguments::AngleBracketed(gen_args) => {
                        let gen_args = &gen_args.args;
                        for gen_arg in gen_args {
                            match gen_arg {
                                syn::GenericArgument::Type(gen_type) => {
                                    inner_types.push(gen_type);
                                },
                                _ => {}
                            }
                        }
                    },
                    _ => {}
                }
               
            }
            return Some(inner_types);

        },
        _ => return None
    }
}

fn is_associated_type(type_param: &Vec<String>, ty: &Type) -> (bool, String) {
    
    match ty {
        Type::Path(path) => {
            let path = &path.path;
            let segments = &path.segments;
            if segments.is_empty() {
                return (false, "NONE".to_owned());
            }
            let first_segment = segments.first();
            if first_segment.is_none() {
                return (false, "NONE".to_owned());
            }
            let first_segment = first_segment.unwrap();
            let ident_str = first_segment.ident.to_string();
            return (type_param.contains(&ident_str), ident_str);
        },
        _ => return (false, "NONE".to_owned())
    }
}