use syn::{__private::{TokenStream2, quote::quote, ToTokens}, DeriveInput, LitStr, Fields, punctuated::Iter, Field, Ident, Type, Visibility, parse::{Parse, Parser}, Data, parse_quote, PathArguments, GenericArgument};

pub fn impl_debug(input: &DeriveInput) -> Result<TokenStream2, syn::Error> {
    let name  = &input.ident;
    let generics = &input.generics;

    

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

    let phantom_data_fields = find_phantom_data_field_inner_type(&field_stream)?;
    let params: Vec<&syn::TypeParam> = generics.type_params().collect();
    let gen_where_clause = gen_where_clause(params, phantom_data_fields);
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

fn gen_where_clause(type_params: Vec<&syn::TypeParam>, phantom_data_fields: Vec<&Ident>) -> syn::WhereClause {

    let mut where_clause: syn::WhereClause = parse_quote! {
        where
    };


    for ty_param in type_params {
        let type_name = &ty_param.ident;
        let matched = phantom_data_fields.iter().any(|inner_ty|inner_ty.to_string().eq(type_name.to_string().as_str()));
        if matched {
         
            let where_predicate: syn::WherePredicate = parse_quote! {
                std::marker::PhantomData<#type_name>: std::fmt::Debug
            };
          
            where_clause.predicates.push(where_predicate);
        } else {
            let where_predicate: syn::WherePredicate = parse_quote! {
                #type_name: std::fmt::Debug
            };
          
            where_clause.predicates.push(where_predicate);
        }

       
    }


    where_clause
}

fn find_phantom_data_field_inner_type<'a>(field_stream: &'a FieldStream) -> Result<Vec<&'a Ident>, syn::Error> {
    let mut fields: Vec<&Ident> = Vec::default();

    for field in field_stream.iter() {
        let field = field?;
        if !field.is_phantom_data_field {
            continue;
        }
        let ty = field.ty;
        let inner_type = get_inner_type(ty);
        if inner_type.is_none() {
            continue;
        }

        let inner_type = inner_type.unwrap();
        match inner_type {
            syn::Type::Path(path_type) => {
                let p = &path_type.path;
                let ident = p.get_ident();
                if ident.is_none() {
                    continue;
                }
                let ident = ident.unwrap();
                fields.push(ident);
            },
            _ => {
                continue;
            }
        }

    }

    Ok(fields)
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
    vis: &'a Visibility,
    attrs: Vec<FieldAttr>,
    is_phantom_data_field: bool,
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

        let vis: &Visibility = &field.vis;
        let attrs = &field.attrs;

        let is_phantom_data_field = ty_eq(ty, "PhantomData");

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
        vis,
        attrs: field_attrs,
        is_phantom_data_field
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


fn get_inner_type(ty: &Type) -> Option<&Type> {

    match ty {
        Type::Path(path) => {
            let path_se = path.path.segments.first();
            if path_se.is_none() {
                return None;
            }
            let path_se = path_se.unwrap();
            match path_se.arguments {
                PathArguments::AngleBracketed(ref args) => {
                    let first_arg = args.args.first();
                    if first_arg.is_none() {
                        return None;
                    }

                    let first_arg = first_arg.unwrap();
                    match first_arg {
                        GenericArgument::Type(arg_type) => {
                            return Some(arg_type);
                        },
                        _ => {
                            return None;
                        }
                    }
                },
                _ => {
                    return None;
                }
            }

        },
        _ => return None
    }
}
