use proc_macro2::TokenStream;
use syn::{Ident, parse::Parse, Token, Block, __private::ToTokens};

#[derive(Debug)]
pub struct Seq {
    name: Ident,
    start: syn::LitInt,
    end: syn::LitInt,
    body: Block
}


impl Parse for Seq {

    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let start: syn::LitInt = input.parse()?;
        input.parse::<Token![..]>()?;
        let end: syn::LitInt = input.parse()?;
        let body: Block = input.parse()?;
        Ok(Self { name, start, end, body})
    }
}


impl Seq {

    pub fn to_token_stream(self) -> Result<TokenStream, syn::Error> {
        let var = &self.name;
        let start = &self.start;
        let end = &self.end;
        let body = &self.body;

        let range = range_exp_to_iter(start, end)?;

        let mut vec = Vec::default();

        for idx in range {
            for stmt in body.stmts.iter() {
                let token_stream = eval_stmt(var, idx, stmt.to_token_stream());
                vec.push(token_stream);
            }
        }

        let ret = TokenStream::from_iter(vec.into_iter());

    
        Ok(ret)
        
    }
}

fn range_exp_to_iter(start: &syn::LitInt, end: &syn::LitInt) -> Result<std::ops::Range<u16>, syn::Error> {
   let start = start.base10_parse::<u16>()?;
   let end = end.base10_parse::<u16>()?;
   Ok(std::ops::Range{start, end})
}

fn eval_stmt(var: &Ident, value: u16, stmt_token: TokenStream) -> TokenStream {
    let token_trees: Vec<proc_macro2::TokenTree> = stmt_token.into_iter().map(|token_tree|{
        eval_token_tree(var, value, token_tree)
    }).collect();

    TokenStream::from_iter(token_trees.into_iter())
}

fn eval_token_tree(var: &Ident, value: u16, token_tree: proc_macro2::TokenTree) -> proc_macro2::TokenTree {
    match token_tree {
        proc_macro2::TokenTree::Ident(ref ident) => {
            if ident.eq(var) {
                proc_macro2::TokenTree::Literal(proc_macro2::Literal::u16_unsuffixed(value))
            } else {
                token_tree
            }
        },
        proc_macro2::TokenTree::Group(ref group) => {
            let delimiter = group.delimiter();
            let token_stream = group.stream();
            
            let mapped_token_tree: Vec<_> = token_stream.into_iter().map(|tt|eval_token_tree(var, value, tt)).collect();

            let token_stream = TokenStream::from_iter(mapped_token_tree.into_iter());
            proc_macro2::TokenTree::Group(proc_macro2::Group::new(delimiter, token_stream))
        },
        _ => token_tree
    }
}