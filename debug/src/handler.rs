use syn::{__private::{TokenStream2, quote::quote, ToTokens}, DeriveInput, LitStr, Fields, punctuated::Iter, Field, Ident, Type, Visibility, parse::{Parse, Parser}, Data, parse_quote};



pub fn impl_debug(input: &mut DeriveInput) -> Result<TokenStream2, syn::Error> {
    let name  = &input.ident;

    let generics = &mut input.generics;

    let where_clause = generics.make_where_clause();


    let (impl_generics, ty_generics, _) = generics.split_for_impl();
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
    
   


    let predicate_debug: syn::WherePredicate = parse_quote! {
            #ty_generics::: std::fmt::Debug
    };

    let exp_lets = gen_debug_field_exp(&field_stream)?;
    
    let token = quote! {
        impl<T: std::fmt::Debug>  std::fmt::Debug for #name<T> {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut debug_struct = fmt.debug_struct(#lit_str);
                #(#exp_lets;)*
                debug_struct.finish()
            }
        }
    };

   Ok(token)
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
        attrs: field_attrs
       })
    }

    fn ty_eq(ty: &Type, other: &str) -> bool {
        if let Type::Path(path) = ty {
            path.path.segments.first().map(|p|p.ident.eq(other)).unwrap_or(false)
        } else {
            false
        }
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

