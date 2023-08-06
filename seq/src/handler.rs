use proc_macro2::{TokenStream, TokenTree};
use syn::{Ident, parse::Parse, Token, __private::quote::format_ident, buffer::TokenBuffer};


#[derive(Debug)]
pub struct Seq {
    name: Ident,
    start: syn::LitInt,
    end: syn::LitInt,
    limits: syn::RangeLimits,
    body: TokenStream
}


impl Parse for Seq {

    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let start: syn::LitInt = input.parse()?;
        let limits = input.parse::<syn::RangeLimits>()?;
        let end: syn::LitInt = input.parse()?;
        let body;
        syn::braced!(body in input);
        let body: TokenStream = body.parse()?;
        Ok(Self { name, start, end, limits, body})
    }
}


impl Seq {

    pub fn to_token_stream(self) -> Result<TokenStream, syn::Error> {
        let var = &self.name;
        let start = &self.start;
        let end = &self.end;
        let limits = &self.limits;
        let body = self.body;

        let range = range_exp_to_iter(start, end, limits)?;


        let buffer_token = TokenBuffer::new2(body);
        let start = buffer_token.begin();
        let exist = exist_repeat_exp(start);

        let token_stream = if exist {
            prase_repeat_exp(var, range, buffer_token.begin())?
        } else {
            let mut token_streams = Vec::default();
            for idx in range {
                let token_stream = parse_replace_ident_exp(var, idx, buffer_token.begin())?;
                token_streams.push(token_stream);
            }
            TokenStream::from_iter(token_streams.into_iter())
        };
    
        Ok(token_stream)
        
    }
}




fn range_exp_to_iter(start: &syn::LitInt, end: &syn::LitInt, limits: &syn::RangeLimits) -> Result<std::ops::Range<u16>, syn::Error> {
    let start = start.base10_parse::<u16>()?;
    let end = end.base10_parse::<u16>()?;
    
    let range = match limits {
        syn::RangeLimits::Closed(_) => {
            let end = end + 1;
            let range = std::ops::Range{start, end};
            range
           
        },
        syn::RangeLimits::HalfOpen(_) => {
            let range = std::ops::Range{start, end};
            range
        }
    };

    Ok(range)
}
 
 

fn exist_repeat_exp(mut current: syn::buffer::Cursor<'_>) -> bool {

    while !current.eof() {
        let token = current.token_tree();
        if token.is_none() {
            return false;
        }
        let (token, next) = token.unwrap();
        match token {
            proc_macro2::TokenTree::Group(group) => {
                let token_buffer = TokenBuffer::new2(group.stream());
                let is_repeat_exp = exist_repeat_exp(token_buffer.begin());
                if is_repeat_exp {
                    return true;
                }
                current = next;
            },
            proc_macro2::TokenTree::Punct(punct) => {
                let punct_ch = punct.as_char();
                if punct_ch != '#' {
                    current = next;
                    continue;
                }

                let next_token = next.token_tree();
                if next_token.is_none() {
                    return false;
                }

                let (token, next) = next_token.unwrap();
                let proc_macro2::TokenTree::Group(group) = token else {
                    current = next;
                    continue;
                };

                let token_buffer = TokenBuffer::new2(group.stream());
                let is_repeat_exp =  exist_repeat_exp(token_buffer.begin());
                if is_repeat_exp {
                    return true;
                }

                let next_token = next.token_tree();
                if next_token.is_none() {
                    return false;
                }

                let (token, next) = next_token.unwrap();
                let proc_macro2::TokenTree::Punct(punct) = token else {
                    current = next;
                    continue;
                };
                let punct_ch = punct.as_char();
                if punct_ch != '*' {
                    current = next;
                    continue;
                }
                return true;
            }
            _ => current = next
        }
    }
    false
}

fn parse_replace_ident_exp(var: &Ident, value: u16, mut current: syn::buffer::Cursor<'_>) -> syn::Result<TokenStream> {
    let mut token_trees: Vec<TokenTree> = Vec::default();

    while !current.eof() {
        let token = current.token_tree();
        if token.is_none() {
            return Err(syn::Error::new(current.span(), "it's not replace exp"));
        }
        let (token, next) = token.unwrap();

        match token {
            proc_macro2::TokenTree::Ident(ident) => {
                let ident_str = ident.to_string();
                let var_str = var.to_string();
                if ident_str.ne(&var_str) {
                    token_trees.push(proc_macro2::TokenTree::Ident(ident));
                    current = next;
                    continue;
                }
                let pre_token_tree = token_trees.pop();
                if pre_token_tree.is_none() {
                    let int_token_tree = proc_macro2::TokenTree::Literal(proc_macro2::Literal::u16_unsuffixed(value));
                    token_trees.push(int_token_tree);
                    current = next;
                    continue;
                }

                let pre_token_tree = pre_token_tree.unwrap();

                let proc_macro2::TokenTree::Punct(pre_token_tree) = pre_token_tree else {
                    let int_token_tree = proc_macro2::TokenTree::Literal(proc_macro2::Literal::u16_unsuffixed(value));
                    token_trees.push(pre_token_tree);
                    token_trees.push(int_token_tree);
                    current = next;
                    continue;
                };

                if pre_token_tree.as_char() != '~' {
                    let int_token_tree = proc_macro2::TokenTree::Literal(proc_macro2::Literal::u16_unsuffixed(value));
                    token_trees.push(proc_macro2::TokenTree::Punct(pre_token_tree));
                    token_trees.push(int_token_tree);
                    current = next;
                    continue;
                } 

                let last_second = token_trees.pop();

                if last_second.is_none() {
                    let int_token_tree = proc_macro2::TokenTree::Literal(proc_macro2::Literal::u16_unsuffixed(value));
                    token_trees.push(proc_macro2::TokenTree::Punct(pre_token_tree));
                    token_trees.push(int_token_tree);
                    current = next;
                    continue;
                }

                let last_second = last_second.unwrap();

                let proc_macro2::TokenTree::Ident(second_ident) = last_second else {
                    let int_token_tree = proc_macro2::TokenTree::Literal(proc_macro2::Literal::u16_unsuffixed(value));
                    token_trees.push(last_second);
                    token_trees.push(proc_macro2::TokenTree::Punct(pre_token_tree));
                    token_trees.push(int_token_tree);
                    current = next;
                    continue;
                };

                let second_span = ident.span();

                let var_span = var.span();

                let join_span = second_span.join(var_span);

                let mut finally_ident = format_ident!("{}{}", second_ident, value);
                
                if join_span.is_some() {
                    finally_ident.set_span(join_span.unwrap());
                }
                

                let finally_ident_tree = proc_macro2::TokenTree::Ident(finally_ident);
            
                token_trees.push(finally_ident_tree);
                current = next;

            },
            proc_macro2::TokenTree::Group(group) => {
                let group_span = group.span();
                let group_delimiter = group.delimiter();
                let group_token_stream = group.stream();
                let group_token_stream = TokenBuffer::new2(group_token_stream);

                let group_token_stream = parse_replace_ident_exp(var, value, group_token_stream.begin())?;

                let mut new_group = proc_macro2::Group::new(group_delimiter, group_token_stream);
                new_group.set_span(group_span);
                
                token_trees.push(proc_macro2::TokenTree::Group(new_group));

                current = next;
            }
            _ => {
                current = next;
                token_trees.push(token);
            }

        }
    }


    Ok(TokenStream::from_iter(token_trees.into_iter()))
}


fn prase_repeat_exp(var: &Ident, range: std::ops::Range<u16>, mut current: syn::buffer::Cursor<'_>) -> syn::Result<TokenStream> {
    let mut vec: Vec<TokenTree> = Vec::default();

    while !current.eof() {
        let token = current.token_tree();
        if token.is_none() {
            return Err(syn::Error::new(current.span(), "it's not repeat exp"));
        }

        let (token, next) = token.unwrap();

        match token {
            proc_macro2::TokenTree::Punct(punct) => {

                let punct_ch = punct.as_char();
                if punct_ch != '#' {
                    current = next;
                    vec.push(proc_macro2::TokenTree::Punct(punct));
                    continue;
                }

                let next_token = next.token_tree();
                if next_token.is_none() {
                    return Err(syn::Error::new(next.span(), "it's not repeat exp"));
                };

                let (token, next) = next_token.unwrap();
                let proc_macro2::TokenTree::Group(group)  = token else {
                    current = next;
                    vec.push(proc_macro2::TokenTree::Punct(punct));
                    vec.push(token);
                    continue;
                };

              
                let next_token = next.token_tree();
                if next_token.is_none() {
                    return Err(syn::Error::new(next.span(), "it's not repeat exp"));
                };

                let (token, next) = next_token.unwrap();
                let proc_macro2::TokenTree::Punct(last_punct)  = token else {
                    current = next;
                    vec.push(proc_macro2::TokenTree::Punct(punct));
                    vec.push(proc_macro2::TokenTree::Group(group));
                    vec.push(token);
                    continue;
                };

                let punct_ch = last_punct.as_char();
                if punct_ch != '*' {
                    current = next;
                    vec.push(proc_macro2::TokenTree::Punct(punct));
                    vec.push(proc_macro2::TokenTree::Group(group));
                    vec.push(proc_macro2::TokenTree::Punct(last_punct));
                    continue;
                }

 
                let group_token_stream = group.stream();
                let group_token_stream = TokenBuffer::new2(group_token_stream);

                let mut vec_group_token_streams: Vec<TokenStream> = Vec::default();

                for idx in range.clone() {
                    let new_group_token_stream = parse_replace_ident_exp(var, idx, group_token_stream.begin())?;
                    vec_group_token_streams.push(new_group_token_stream);
                }


                vec.extend(TokenStream::from_iter(vec_group_token_streams.into_iter()).into_iter());

                current = next;

            },
            proc_macro2::TokenTree::Group(group) => {

                let group_span = group.span();
                let group_delimiter = group.delimiter();
                let group_token_stream = group.stream();
                let group_token_stream = TokenBuffer::new2(group_token_stream);

                let group_token_stream = prase_repeat_exp(var, range.clone(), group_token_stream.begin())?;

                let mut new_group = proc_macro2::Group::new(group_delimiter, group_token_stream);
                new_group.set_span(group_span);
                
                vec.push(proc_macro2::TokenTree::Group(new_group));

                current = next;

            },
            _ => {

                vec.push(token);

                current = next;
            }
        }

    }

    Ok(TokenStream::from_iter(vec.into_iter()))
}