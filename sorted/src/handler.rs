use std::cmp::Ordering;

use proc_macro2::{TokenStream, Span};
use syn::{parse::Parse, __private::ToTokens, visit_mut::VisitMut, spanned::Spanned};



pub struct SortedStruct {
    enum_item: syn::ItemEnum
}

impl Parse for SortedStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let enum_item = input.parse::<syn::ItemEnum>();
        if enum_item.is_err() {
            return Err(syn::Error::new(Span::call_site(), "expected enum or match expression"));
        }
        let enum_item = enum_item.unwrap();

        let mut errors: Vec<_> = Vec::default();
        let mut variants: Vec<&syn::Variant> = enum_item.variants.iter().collect();
        variants.sort_by(|a, b| {
            

            let ret = a.ident.cmp(&b.ident);
            if ret.is_lt() {
                let msg = format!("{} should sort before {}", a.ident.to_string(), b.ident.to_string());
                errors.push(syn::Error::new(a.span(), msg));
            }
            ret
        });

        if !errors.is_empty() {
            let first_error = errors.pop();
            if let Some(e) = first_error {
                return Err(e);
            }
        }

        Ok(Self { enum_item })
    }
}

impl SortedStruct {

    pub fn to_token_stream(self) -> TokenStream {
        self.enum_item.into_token_stream()
    }
}


pub struct CheckStruct {
    fn_item: syn::ItemFn
}


impl Parse for CheckStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
       let fn_item = input.parse()?;
       Ok(Self { fn_item })
    }
}

struct CheckVisitMut(Vec<syn::Error>);

impl VisitMut for CheckVisitMut {

    fn visit_item_fn_mut(&mut self, item: &mut syn::ItemFn) {
        let body = item.block.as_mut();
        let stmts_iter_mut = body.stmts.iter_mut();

        for stmt in stmts_iter_mut {
            match stmt {
                syn::Stmt::Expr(exp, _) => {
                    if let syn::Expr::Match(exp) = exp  {
                        let vec: Vec<usize> = exp.attrs.iter().enumerate().filter(|(_, attr)| {
                            let ident = attr.path().get_ident();
                            if ident.is_none() {
                                return false;
                            }
                            let ident = ident.unwrap();
                            let ident_str = ident.to_string();
                            return ident_str.eq("sorted");
                        }).map(|(idx, _)|idx).collect();

                        
                        
                        if !vec.is_empty() {
                            exp.arms.sort_by(|a, b| {
                                let a_pat: &syn::Pat = &a.pat;
                                let b_pat: &syn::Pat = &b.pat;
                                let ret = cmp_pat(a_pat, b_pat);

                                if ret.is_none() {
                                    self.0.push(syn::Error::new(b.span(), "unsupported by #[sorted]"));
                                    return Ordering::Equal;
                                }

                                let (ordering, a_str, b_str) = ret.unwrap();

                              
                                if ordering.is_lt() {
                                    let msg = format!("{} should sort before {}", a_str, b_str);
                                    self.0.push(syn::Error::new(a.span(), msg));
                                }
                                ordering
                            })
                        }

                        for idx in vec {
                            exp.attrs.remove(idx);
                        }
                    }
                },
                _ => {}
            }
        }
    }

}


impl CheckStruct {
    pub fn to_token_stream(mut self) -> TokenStream {
        let fn_item =&mut self.fn_item;
        
    
        let mut check_visit_mut = CheckVisitMut(Vec::default());
        
        check_visit_mut.visit_item_fn_mut(fn_item);

        let errors = check_visit_mut.0;
        
        let error = errors.into_iter().next();

        if let Some(e) = error {
            let mut token_stream = fn_item.into_token_stream();
            token_stream.extend(e.into_compile_error());
            token_stream
        } else {
            fn_item.into_token_stream()
        }
    }
}

fn cmp_pat(a: &syn::Pat, b: &syn::Pat) -> Option<(Ordering, String, String)> {
    let a = pat_to_string(a);
    if a.is_none() {
        return None;
    }

    let a = a.unwrap();
    let b = pat_to_string(b);
    if b.is_none() {
        return None;
    }
    let b = b.unwrap();

    if a.ne(&b) && (a.eq("_") || b.eq("_")) {
        if a.eq("_") {
            return Some((Ordering::Greater, a, b));
        } 
        return Some((Ordering::Less, a, b));
    }
    

    Some((a.cmp(&b), a, b))
}

fn pat_to_string(pat: &syn::Pat) -> Option<String> {
    match pat {
        syn::Pat::TupleStruct(p) => {

           Some(path_to_string(&p.path))

        },
        syn::Pat::Struct(p) => {
            Some(path_to_string(&p.path))
        }
        syn::Pat::Path(p) => {
            Some(path_to_string(&p.path))
        }
        syn::Pat::Wild(_) => {
            Some("_".to_string())   
        },
        syn::Pat::Ident(ident) => {
            Some(ident.ident.to_string())
        }
        _ => None
    }
}

fn path_to_string(path: &syn::Path) -> String {
    let mut ret = String::default(); 
    let segments = &path.segments;
    let pairs = segments.pairs();
    pairs.for_each(|pair| {
         let value = pair.value();
         let value = value.ident.to_string();
         let punct = pair.punct();

         ret.push_str(value.as_str());
         if punct.is_some() {
             ret.push_str("::");
         }
    });
    ret
}