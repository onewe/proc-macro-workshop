use syn::{Ident, Type, Visibility, Field, Fields, punctuated::Iter, __private::{quote::{quote, spanned::Spanned}, Span, TokenStream2, ToTokens}, parse::{Parse, Parser}, FieldValue, parse_quote, ItemFn, PathArguments, GenericArgument};

pub struct FieldInfo<'a> {
    name: Option<&'a Ident>,
    ty: &'a Type,
    vis: &'a Visibility,
    attrs: Vec<FieldAttr>,
    is_option: bool,
    is_vec: bool,
}

#[derive(Debug)]
pub struct FieldAttr {
    _name: String,
    value: String,
}

impl<'a> FieldInfo<'a> {

    pub fn new(field: &'a Field) -> Result<Self, syn::Error> {
        let name: Option<&Ident> = field.ident.as_ref();
        let ty: &Type = &field.ty;
        let vis: &Visibility = &field.vis;
        let attrs = &field.attrs;

        
        let is_option = Self::ty_eq(ty, "Option");
        let is_vec = Self::ty_eq(ty, "Vec");


        let mut field_attrs = Vec::<FieldAttr>::new();

        for attr in attrs {
            let meta = &attr.meta;
            let require_list = meta.require_list()?;
            let args = require_list.parse_args::<syn::ExprAssign>()?;
            let left_token = args.left.into_token_stream();
            let left_ident = <syn::Ident as Parse>::parse.parse2(left_token)?;
            let left = left_ident.to_string();

            if !left.eq("each") {
                return Err(syn::Error::new(left_ident.__span(), "expected `builder(each = \"...\")`"));
            }


            let right_token = args.right.into_token_stream();
            let right_lit = <syn::LitStr as Parse>::parse.parse2(right_token)?;
            let right = right_lit.value();

            let field_attr = FieldAttr {
                _name: left,
                value: right
            };
            field_attrs.push(field_attr);
        }



       Ok(Self {
        name,
        ty,
        vis,
        attrs: field_attrs,
        is_option,
        is_vec
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


pub fn gen_builder_field(field_stream: &FieldStream) -> Result<Vec<Field>, syn::Error> {

    let mut builder_fields: Vec<Field> = Vec::default();

    for field_info in field_stream.iter() {
        let field_info = field_info?;
        let name = field_info.name;
        if name.is_none() {
            continue;
        }
        let name = name.unwrap();
        let ty = field_info.ty;
        let vis = field_info.vis;
        let is_option_field = field_info.is_option;

        let builder_field: Field;

        if is_option_field {
            builder_field = Field::parse_named.parse2(quote! {
               #vis #name: #ty
            })?;
        } else {
            builder_field = Field::parse_named.parse2(quote! {
               #vis #name: std::option::Option<#ty>
            })?;
        }

        builder_fields.push(builder_field);

    }

   Ok(builder_fields)
}


pub fn gen_builder_field_default_value(field_stream: &FieldStream) -> Result<Vec<FieldValue>, syn::Error> {

    let mut field_values: Vec<FieldValue> = Vec::default();

    for field_info in field_stream.iter(){
        let field_info = field_info?;
        let name = field_info.name;
        if name.is_none() {
            continue;
        }
        let name = name.unwrap();
        let field_value: FieldValue = parse_quote! {
            #name: std::option::Option::None
        };

        field_values.push(field_value);
    }

    Ok(field_values)
}

pub fn gen_field_method(field_stream: &FieldStream) -> Result<Vec<ItemFn>, syn::Error> {

    let mut methods: Vec<ItemFn> = Vec::default();

    for field_info in field_stream.iter() {
        let field_info = field_info?;
        let name = field_info.name;
        if name.is_none() {
            continue;
        }
        let name = name.unwrap();
        let ty = field_info.ty;
        let visi = field_info.vis;
        

        let is_option = field_info.is_option;
        let is_vec = field_info.is_vec;
        let attrs = field_info.attrs;


        if attrs.is_empty() {

            let fn_item = gen_builder_field_method(name, ty, visi, is_option);
            if fn_item.is_none() {
                continue;
            } 
            methods.push(fn_item.unwrap());
        } else {
            for attr in attrs.iter() {
                let attr_value = &attr.value;
                if name.ne(attr_value) {
                   if let Some(f) = gen_builder_field_method(name, ty, visi, is_option) {
                    methods.push(f);
                   }
                  
                   let each_name = Ident::new(attr_value.as_str(), Span::call_site());
                   if let Some(f) = gen_each_field_method(name, &each_name, ty, visi, is_vec) {
                    methods.push(f);
                   }
                 

                } else {
                    let each_name = Ident::new(attr_value.as_str(), Span::call_site());
                    if let Some(f) = gen_each_field_method(name, &each_name, ty, visi, is_vec) {
                        methods.push(f);
                    }
                };
               
            }

        }

        
    }

    Ok(methods)
}

pub fn gen_builder_method(target: &Ident, field_stream: &FieldStream) -> Result<ItemFn, syn::Error> {

    let mut tokens: Vec<TokenStream2> = Vec::default();
    let mut field_names: Vec<&Ident> = Vec::default();

    for field_info in field_stream.iter() {
        let field_info = field_info?;
        let name = field_info.name;
        if name.is_none() {
            continue;
        }
        let name: &Ident = name.unwrap();
        field_names.push(name);

        let attrs = field_info.attrs;
        let is_option = field_info.is_option;

        if is_option {
            let token = quote! {
                let #name = self.#name.take();
            };
            tokens.push(token);
        } else if !attrs.is_empty() {
            let token = quote! {
                let #name = if self.#name.is_none() {
                    std::vec::Vec::default()
                } else {
                    self.#name.take().unwrap()
                };
            };
            tokens.push(token);
        } else {
            let token = quote! {
                if self.#name.is_none() {
                    let err_msg = format!("{} field missing", stringify!(#name));
                    return std::result::Result::Err(std::boxed::Box::<dyn std::error::Error>::from(err_msg.to_string()));
                }

                let #name = self.#name.take().unwrap();
            };
            tokens.push(token);
        }

    }

    let fn_item: ItemFn = parse_quote! {
        pub fn build(&mut self) -> std::result::Result<#target, std::boxed::Box<dyn std::error::Error>>{
            #(#tokens)*
            let target = #target {
                #(#field_names),*
            };

            std::result::Result::Ok(target)
        }
    };

   Ok(fn_item)
}

fn gen_each_field_method(name: &Ident, each_name: &Ident, ty: &Type, visi: &Visibility, is_vec: bool) -> Option<ItemFn> {
    if !is_vec {
        return None;
    }

    let inner_ty =  get_inner_type(ty);

    if inner_ty.is_none() {
        return None;
    }

    let inner_ty = inner_ty.unwrap();

    let item_fn: ItemFn = parse_quote! {
        #visi fn #each_name(&mut self, #each_name: #inner_ty) -> &mut Self {
            let mut vec = if self.#name.is_none() {
                std::vec::Vec::default()
            } else {
                self.#name.take().unwrap()
            };
            vec.push(#each_name);
            self.#name = std::option::Option::Some(vec);
            self
        }
    };

    Some(item_fn)
}

fn gen_builder_field_method(name: &Ident, ty: &Type, visi: &Visibility, is_option: bool) -> Option<ItemFn> {
    if is_option {
        let inner_ty =  get_inner_type(ty);

         if inner_ty.is_none() {
             return None;
         }

         let inner_ty = inner_ty.unwrap();

         
         let item_fn: ItemFn = parse_quote! {
             #visi fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                 self.#name = std::option::Option::Some(#name);
                 self
             }
         };

         return Some(item_fn);

     } else {
         let item_fn: ItemFn = parse_quote! {
             #visi fn #name(&mut self, #name: #ty) -> &mut Self {
                 self.#name = std::option::Option::Some(#name);
                 self
             }
         };
         return Some(item_fn);

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