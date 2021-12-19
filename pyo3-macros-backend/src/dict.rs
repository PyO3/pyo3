use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::iter::FromIterator;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::Token;
use syn::{braced, Expr};

#[derive(Debug)]
pub struct PyDictLiteral {
    pub py: Ident,
    pub items: Vec<KeyValue>,
}

#[derive(Debug)]
pub struct KeyValue {
    key: syn::Expr,
    value: syn::Expr,
}

#[derive(Debug)]
struct Key(syn::Expr);

impl Parse for Key {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut tokens = vec![];

        while !input.peek(Token![:]) || input.peek(Token![::]) {
            let tt = input.parse::<TokenTree>()?;
            tokens.push(tt);
        }
        let stream = TokenStream::from_iter(tokens.into_iter());

        let expr = syn::parse2::<Expr>(stream)?;
        Ok(Self(expr))
    }
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Key = input.parse()?;
        let _sep: Token![:] = input.parse()?;
        let value: syn::Expr = input.parse()?;

        Ok(Self { key: key.0, value })
    }
}

impl Parse for PyDictLiteral {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let py: Ident = input.parse()?;
        let _arrow: Token![=>] = input.parse()?;

        let body: ParseBuffer;
        braced!(body in input);

        let items: Punctuated<KeyValue, Token![,]> = Punctuated::parse_terminated(&body)?;

        Ok(Self {
            py,
            items: items.into_iter().collect(),
        })
    }
}

impl ToTokens for KeyValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let key = &self.key;
        let value = &self.value;
        let ts = quote! {(#key, #value)};
        tokens.extend(ts);
    }
}
